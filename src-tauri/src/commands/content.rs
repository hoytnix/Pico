use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;
use tauri::State;

use crate::builder::{load_models, resolve_collection_folder, ExtractedFrontmatter};
use crate::models::{AppState, ModelDef, PostDocument, PostMeta};
use crate::validation::{
    parse_markdown_document, serialize_markdown_document, slugify, validate_frontmatter,
};
use super::require_current_site;

/// Core implementation of `list_content(collection)` (§3.2, §5).
pub fn list_content_impl(site_root: &Path, collection: &str) -> Result<Vec<PostMeta>, String> {
    let (folder_name, col_dir) = resolve_collection_folder(site_root, collection);
    if !col_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut posts = Vec::new();
    let entries = fs::read_dir(&col_dir)
        .map_err(|e| format!("Could not read content/{}: {}", folder_name, e))?;

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md") {
            let rel_path = p
                .strip_prefix(site_root)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            if let Ok(raw) = fs::read_to_string(&p) {
                if let Ok((fm, _, _)) = parse_markdown_document(&raw, &rel_path) {
                    if let Some(extracted) = ExtractedFrontmatter::from_value(&fm) {
                        posts.push(extracted.into_post_meta(&rel_path, &folder_name, collection));
                    }
                }
            }
        }
    }

    // Sorted by date descending, then title ascending (§3.2)
    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
    Ok(posts)
}

/// Core implementation of `read_post` (§3.3, §5).
pub fn read_post_impl(site_root: &Path, path: &str) -> Result<PostDocument, String> {
    let full_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        site_root.join(path.trim_start_matches('/'))
    };
    let rel_display = full_path
        .strip_prefix(site_root)
        .unwrap_or(&full_path)
        .to_string_lossy()
        .replace('\\', "/");

    let raw = fs::read_to_string(&full_path)
        .map_err(|e| format!("{}: Could not read post: {}", rel_display, e))?;
    let (frontmatter, markdown, _) =
        parse_markdown_document(&raw, &rel_display).map_err(|e| e.to_string())?;

    Ok(PostDocument {
        frontmatter,
        markdown,
    })
}

/// Core implementation of `save_post` (§3.3, §5).
pub fn save_post_impl(
    site_root: &Path,
    path: &str,
    mut frontmatter: Value,
    markdown: &str,
) -> Result<(), String> {
    let full_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        site_root.join(path.trim_start_matches('/'))
    };
    let rel_display = full_path
        .strip_prefix(site_root)
        .unwrap_or(&full_path)
        .to_string_lossy()
        .replace('\\', "/");

    let models = load_models(site_root).map_err(|e| e.to_string())?;

    // Determine which collection model matches this post path
    let matched_model: Option<&ModelDef> = models.iter().find(|m| {
        let (folder, _) = resolve_collection_folder(site_root, &m.name);
        rel_display.starts_with(&format!("content/{}/", folder))
            || rel_display.starts_with(&format!("content/{}/", m.name))
    });

    // Normalize slug from title if empty
    if let Some(obj) = frontmatter.as_object_mut() {
        let title_str = obj
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let current_slug = obj
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let final_slug = if current_slug.is_empty() {
            slugify(&title_str)
        } else {
            slugify(&current_slug)
        };
        obj.insert("slug".to_string(), Value::String(final_slug));

        // Normalize comma-separated tags if passed as string
        if let Some(Value::String(tag_str)) = obj.get("tags").cloned() {
            let tag_arr: Vec<Value> = tag_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| Value::String(s.to_string()))
                .collect();
            obj.insert("tags".to_string(), Value::Array(tag_arr));
        }
    }

    if let Some(model) = matched_model {
        validate_frontmatter(&frontmatter, model, &rel_display, None).map_err(|errs| {
            errs.into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })?;
    }

    let serialized = serialize_markdown_document(&frontmatter, markdown, matched_model)
        .map_err(|e| e.to_string())?;

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("{}: Could not create folder: {}", rel_display, e))?;
    }
    fs::write(&full_path, serialized)
        .map_err(|e| format!("{}: Failed to save file: {}", rel_display, e))?;
    Ok(())
}

/// Core implementation of `save_image` (§3.3, §5).
pub fn save_image_impl(
    site_root: &Path,
    source_path: &Path,
    post_slug: &str,
    collection: &str,
) -> Result<String, String> {
    if !source_path.is_file() {
        return Err(format!(
            "Image file '{}' does not exist.",
            source_path.display()
        ));
    }
    let file_name = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid image file name.".to_string())?;

    if collection == "assets" || post_slug.trim().is_empty() {
        let dest_dir = site_root.join("assets");
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let dest = dest_dir.join(file_name);
        fs::copy(source_path, &dest)
            .map_err(|e| format!("Could not copy image to assets/: {}", e))?;
        Ok(format!("/assets/{}", file_name))
    } else {
        let (_, col_dir) = resolve_collection_folder(site_root, collection);
        let slug_clean = slugify(post_slug);
        let dest_dir = col_dir.join(&slug_clean);
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let dest = dest_dir.join(file_name);
        fs::copy(source_path, &dest)
            .map_err(|e| format!("Could not copy image to post folder: {}", e))?;
        Ok(file_name.to_string())
    }
}

#[tauri::command]
pub fn list_content(
    collection: String,
    state: State<'_, AppState>,
) -> Result<Vec<PostMeta>, String> {
    let site_root = require_current_site(&state)?;
    log::debug!(target: "pico::content", "list_content for collection '{}'", collection);
    let posts = list_content_impl(&site_root, &collection).map_err(|err| {
        log::error!(target: "pico::content", "list_content failed for '{}': {}", collection, err);
        err
    })?;
    log::debug!(target: "pico::content", "list_content: found {} posts in '{}'", posts.len(), collection);
    Ok(posts)
}

#[tauri::command]
pub fn read_post(path: String, state: State<'_, AppState>) -> Result<PostDocument, String> {
    let site_root = require_current_site(&state)?;
    log::debug!(target: "pico::content", "read_post: {:?}", path);
    read_post_impl(&site_root, &path).map_err(|err| {
        log::error!(target: "pico::content", "read_post failed for {:?}: {}", path, err);
        err
    })
}

#[tauri::command]
pub fn save_post(
    path: String,
    frontmatter: Value,
    markdown: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::content", "save_post: {:?} (markdown length: {} chars)", path, markdown.len());
    save_post_impl(&site_root, &path, frontmatter, &markdown).map_err(|err| {
        log::error!(target: "pico::content", "save_post failed for {:?}: {}", path, err);
        err
    })?;
    log::info!(target: "pico::content", "save_post succeeded: {:?}", path);
    Ok(())
}

#[tauri::command]
pub fn delete_post(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::content", "delete_post: {:?}", path);
    let full_path = if Path::new(&path).is_absolute() {
        PathBuf::from(&path)
    } else {
        site_root.join(path.trim_start_matches('/'))
    };
    if !full_path.exists() {
        log::warn!(target: "pico::content", "delete_post target does not exist: {:?}", full_path);
        return Err(format!("Post '{}' does not exist.", path));
    }

    // Try moving to OS Trash first (§3.2); fall back to file removal since user already confirmed in dialog
    if trash::delete(&full_path).is_err() {
        fs::remove_file(&full_path)
            .map_err(|e| {
                let msg = format!("Failed to delete '{}': {}", path, e);
                log::error!(target: "pico::content", "{}", msg);
                msg
            })?;
    }
    log::info!(target: "pico::content", "delete_post succeeded: {:?}", full_path);
    Ok(())
}

#[tauri::command]
pub fn save_image(
    source_path: String,
    post_slug: String,
    collection: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::assets", "save_image: source={:?}, post_slug='{}', collection='{}'", source_path, post_slug, collection);
    save_image_impl(&site_root, Path::new(&source_path), &post_slug, &collection)
}

#[tauri::command]
pub fn save_image_bytes(
    file_name: String,
    bytes: Vec<u8>,
    post_slug: String,
    collection: String,
    site_wide: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::assets", "save_image_bytes: file_name='{}', size={} bytes, post_slug='{}', collection='{}', site_wide={}", file_name, bytes.len(), post_slug, collection, site_wide);
    let safe_name = Path::new(&file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.png")
        .replace(' ', "-");

    if site_wide || collection == "assets" || post_slug.trim().is_empty() {
        let dest_dir = site_root.join("assets");
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let dest = dest_dir.join(&safe_name);
        fs::write(&dest, &bytes)
            .map_err(|e| format!("Could not write image to assets/: {}", e))?;
        log::info!(target: "pico::assets", "Saved site-wide image: /assets/{}", safe_name);
        Ok(format!("/assets/{}", safe_name))
    } else {
        let (_, col_dir) = resolve_collection_folder(&site_root, &collection);
        let slug_clean = slugify(&post_slug);
        let dest_dir = col_dir.join(&slug_clean);
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let dest = dest_dir.join(&safe_name);
        fs::write(&dest, &bytes)
            .map_err(|e| format!("Could not write image to post folder: {}", e))?;
        log::info!(target: "pico::assets", "Saved post image: {} (in {})", safe_name, dest_dir.display());
        Ok(safe_name)
    }
}

#[tauri::command]
pub fn check_image_size(source_path: String) -> Result<u64, String> {
    log::debug!(target: "pico::assets", "check_image_size: source={:?}", source_path);
    let meta = fs::metadata(&source_path)
        .map_err(|e| format!("Could not inspect image '{}': {}", source_path, e))?;
    Ok(meta.len())
}
