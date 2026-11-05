use crate::builder::models_loader::resolve_collection_folder;
use crate::builder::post::{derive_image_variant_paths, render_markdown_to_html, BuiltPost};
use crate::models::{BuildError, ModelDef};
use crate::validation::{
    check_duplicate_slugs, parse_markdown_document, slugify, validate_frontmatter,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ScannedContent {
    pub collection_posts: HashMap<String, Vec<BuiltPost>>,
    pub collection_folders: HashMap<String, String>,
    pub errors: Vec<BuildError>,
    pub log: Vec<String>,
}

pub fn scan_collection_content(
    site_root: &Path,
    models: &[ModelDef],
    include_drafts: bool,
) -> ScannedContent {
    let mut collection_posts: HashMap<String, Vec<BuiltPost>> = HashMap::new();
    let mut collection_folders: HashMap<String, String> = HashMap::new();
    let mut errors: Vec<BuildError> = Vec::new();
    let mut log: Vec<String> = Vec::new();

    for model in models {
        let (folder_name, col_dir) = resolve_collection_folder(site_root, &model.name);
        collection_folders.insert(model.name.clone(), folder_name.clone());
        let mut built_for_col: Vec<BuiltPost> = Vec::new();
        let mut slug_entries: Vec<(String, String, usize)> = Vec::new();

        if col_dir.is_dir() {
            let mut md_files: Vec<PathBuf> = fs::read_dir(&col_dir)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok().map(|d| d.path()))
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|s| s.eq_ignore_ascii_case("md"))
                            .unwrap_or(false)
                })
                .collect();
            md_files.sort();

            for md_path in md_files {
                let rel_path = md_path
                    .strip_prefix(site_root)
                    .unwrap_or(&md_path)
                    .to_string_lossy()
                    .replace('\\', "/");

                let raw = match fs::read_to_string(&md_path) {
                    Ok(s) => s,
                    Err(e) => {
                        errors.push(BuildError {
                            file: rel_path,
                            line: 1,
                            message: format!("Could not read markdown file: {}", e),
                        });
                        continue;
                    }
                };

                let (fm_val, markdown_body, raw_yaml) =
                    match parse_markdown_document(&raw, &rel_path) {
                        Ok(res) => res,
                        Err(err) => {
                            errors.push(err);
                            continue;
                        }
                    };

                if let Err(mut val_errs) =
                    validate_frontmatter(&fm_val, model, &rel_path, Some(&raw_yaml))
                {
                    errors.append(&mut val_errs);
                    continue;
                }

                let obj = fm_val.as_object().unwrap();
                let draft = obj
                    .get("draft")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // Step 2: Filter drafts unless include_drafts is true
                if draft && !include_drafts {
                    log.push(format!("Skipped draft: {}", rel_path));
                    continue;
                }

                let title = obj
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled")
                    .to_string();
                let date = obj
                    .get("date")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1970-01-01")
                    .to_string();
                let raw_slug = obj
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                let slug = if raw_slug.is_empty() {
                    slugify(&title)
                } else {
                    slugify(raw_slug)
                };

                slug_entries.push((rel_path.clone(), slug.clone(), 2));

                let excerpt = obj
                    .get("excerpt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let hero = obj
                    .get("hero")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let tags: Vec<String> = match obj.get("tags") {
                    Some(Value::Array(arr)) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect(),
                    Some(Value::String(s)) => s
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect(),
                    _ => Vec::new(),
                };

                // Step 3: Render Markdown body to HTML with pulldown-cmark (GFM)
                let (body_html, body_text, markdown_images) =
                    render_markdown_to_html(&markdown_body);

                let url = format!("/{}/{}/", folder_name, slug);

                // Check page attachments in `content/<folder_name>/<slug>/`
                let attach_dir = col_dir.join(&slug);
                let mut attachments: Vec<String> = Vec::new();
                if attach_dir.is_dir() {
                    let mut att_files: Vec<PathBuf> = fs::read_dir(&attach_dir)
                        .ok()
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok().map(|d| d.path()))
                        .filter(|p| p.is_file())
                        .collect();
                    att_files.sort();
                    for af in att_files {
                        if let Some(fname) = af.file_name().and_then(|n| n.to_str()) {
                            attachments.push(format!("{}{}", url, fname));
                        }
                    }
                }

                // Derive WebP URLs if hero is PNG/JPG
                let (hero_webp, thumb_webp) = if !hero.is_empty() {
                    if let Some((hw, tw)) = derive_image_variant_paths(&hero) {
                        let hw_url = if hw.starts_with('/') || hw.starts_with("http") {
                            hw
                        } else {
                            format!("{}{}", url, hw)
                        };
                        let tw_url = if tw.starts_with('/') || tw.starts_with("http") {
                            tw
                        } else {
                            format!("{}{}", url, tw)
                        };
                        (hw_url, tw_url)
                    } else if hero.ends_with(".webp") {
                        let u = if hero.starts_with('/') || hero.starts_with("http") {
                            hero.clone()
                        } else {
                            format!("{}{}", url, hero)
                        };
                        (u.clone(), u)
                    } else {
                        (String::new(), String::new())
                    }
                } else {
                    (String::new(), String::new())
                };

                built_for_col.push(BuiltPost {
                    title,
                    date,
                    slug,
                    draft,
                    excerpt,
                    hero,
                    hero_webp,
                    thumb_webp,
                    tags,
                    url,
                    collection: folder_name.clone(),
                    body: markdown_body,
                    body_html,
                    body_text,
                    attachments,
                    source_rel_path: rel_path,
                    markdown_images,
                    extra_fields: obj.clone(),
                });
            }
        }

        // Duplicate slug check per collection
        let mut dup_errs = check_duplicate_slugs(&slug_entries);
        errors.append(&mut dup_errs);

        // Step 4: Sort by date descending, then title ascending
        built_for_col.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
        collection_posts.insert(model.name.clone(), built_for_col);
    }

    ScannedContent {
        collection_posts,
        collection_folders,
        errors,
        log,
    }
}
