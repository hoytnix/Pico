pub mod builder;
pub mod llm;
pub mod models;
pub mod preview;
pub mod publish;
pub mod validation;
pub mod vibe;

use crate::builder::{
    build_site_pipeline, load_all_templates, load_models, load_site_config,
    resolve_collection_folder,
};
use crate::models::{
    AppSettingsView, AppState, BuildResult, GroundingSource, LlmAppSettings, LlmSettingsView,
    ModelDef, ModelField, PersistedAppSettings, PostDocument, PostMeta, PreviewServerHandle,
    ProviderInfo, PublishResult, RecentSite, SaveLlmSettingsInput, SiteConfig, SiteInfo,
    TemplateContextInfo, TestConnectionResult, VariableDoc, VibeContext, VibeOptions, VibeResult,
    VibeTarget,
};
use crate::preview::{start_axum_preview_server, start_site_watcher};
use crate::publish::publish_to_github_pages;
use crate::validation::{
    BUILTIN_ITEM_FIELDS, SITE_CONFIG_FIELDS, parse_markdown_document, serialize_markdown_document,
    slugify, validate_frontmatter, validate_template_with_models,
};
use crate::vibe::{build_vibe_system_prompt, read_vibe_target_content};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::State;
use walkdir::WalkDir;

pub fn app_config_file_path() -> PathBuf {
    let base = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let dir = base.join(".config").join("pico");
    let _ = fs::create_dir_all(&dir);
    dir.join("settings.json")
}

pub fn load_persisted_llm_settings() -> LlmAppSettings {
    let path = app_config_file_path();
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(migrated) = crate::llm::migrate_settings_json(&raw) {
            return migrated;
        }
    }
    LlmAppSettings {
        current_provider: crate::llm::PROVIDER_GEMINI_NATIVE.to_string(),
        providers: crate::llm::create_default_providers_map(),
        github_token: String::new(),
        recent_sites: Vec::new(),
    }
}

pub fn save_persisted_llm_settings(settings: &LlmAppSettings) -> Result<(), String> {
    let path = app_config_file_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Failed to save app settings: {}", e))
}

pub fn load_persisted_settings() -> PersistedAppSettings {
    let llm = load_persisted_llm_settings();
    let (ep, md, key) = if let Some(p) = llm.providers.get(&llm.current_provider) {
        let effective_m = if !p.custom_model.trim().is_empty() {
            p.custom_model.clone()
        } else {
            p.model.clone()
        };
        (p.base_url.clone(), effective_m, p.api_key.clone())
    } else {
        (
            crate::models::default_llm_endpoint(),
            crate::models::default_llm_model(),
            String::new(),
        )
    };
    PersistedAppSettings {
        llm_endpoint: ep,
        llm_model: md,
        llm_api_key: key,
        github_token: llm.github_token,
        recent_sites: llm.recent_sites,
    }
}

pub fn save_persisted_settings(settings: &PersistedAppSettings) -> Result<(), String> {
    let mut llm = load_persisted_llm_settings();
    llm.github_token = settings.github_token.clone();
    llm.recent_sites = settings.recent_sites.clone();
    let current_prov = if llm.current_provider.is_empty() {
        crate::llm::PROVIDER_OPENAI.to_string()
    } else {
        llm.current_provider.clone()
    };
    let entry = llm.providers.entry(current_prov).or_insert_with(|| crate::models::ProviderSettings {
        api_key: String::new(),
        base_url: settings.llm_endpoint.clone(),
        model: settings.llm_model.clone(),
        custom_model: String::new(),
        thinking: "medium".to_string(),
        grounding: false,
    });
    entry.base_url = settings.llm_endpoint.clone();
    entry.model = settings.llm_model.clone();
    if !settings.llm_api_key.trim().is_empty() {
        entry.api_key = settings.llm_api_key.trim().to_string();
    }
    save_persisted_llm_settings(&llm)
}

fn record_recent_site(site_root: &Path, title: &str) -> Vec<RecentSite> {
    let mut settings = load_persisted_llm_settings();
    let canonical = site_root
        .canonicalize()
        .unwrap_or_else(|_| site_root.to_path_buf())
        .to_string_lossy()
        .to_string();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();

    settings.recent_sites.retain(|r| r.path != canonical);
    settings.recent_sites.insert(
        0,
        RecentSite {
            title: title.to_string(),
            path: canonical,
            last_opened: now,
        },
    );
    if settings.recent_sites.len() > 12 {
        settings.recent_sites.truncate(12);
    }
    let _ = save_persisted_llm_settings(&settings);
    settings.recent_sites
}

static EMBEDDED_STARTERS: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/../starters");

fn resolve_starters_dir() -> Result<PathBuf, String> {
    let mut candidates = vec![
        PathBuf::from("starters"),
        PathBuf::from("../starters"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../starters"),
    ];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("starters"));
            candidates.push(exe_dir.join("../Resources/starters"));
            candidates.push(exe_dir.join("../lib/pico/starters"));
        }
    }

    for c in candidates {
        if c.is_dir() {
            return Ok(c);
        }
    }
    Err("Could not locate external starter templates directory on disk.".to_string())
}

fn extract_embedded_dir_recursive(
    dir: &include_dir::Dir<'_>,
    dest: &Path,
    prefix: &Path,
) -> std::io::Result<()> {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(sub_dir) => {
                let rel = sub_dir.path().strip_prefix(prefix).unwrap_or(sub_dir.path());
                fs::create_dir_all(dest.join(rel))?;
                extract_embedded_dir_recursive(sub_dir, dest, prefix)?;
            }
            include_dir::DirEntry::File(file) => {
                let rel = file.path().strip_prefix(prefix).unwrap_or(file.path());
                if let Some(parent) = dest.join(rel).parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(dest.join(rel), file.contents())?;
            }
        }
    }
    Ok(())
}

pub fn extract_embedded_starter(
    starter: &include_dir::Dir<'_>,
    dest: &Path,
    starter_key: &str,
) -> std::io::Result<()> {
    let prefix = Path::new(starter_key);
    extract_embedded_dir_recursive(starter, dest, prefix)
}

fn copy_starter_pack(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Could not create site directory: {}", e))?;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        let rel = p.strip_prefix(src).map_err(|e| e.to_string())?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(p, &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Core implementation of `open_site` (usable both from Tauri command and tests).
pub fn open_site_impl(site_root: &Path) -> Result<SiteInfo, String> {
    if !site_root.is_dir() {
        return Err(format!(
            "Folder '{}' does not exist.",
            site_root.display()
        ));
    }

    for req_dir in ["models", "templates", "content", "assets"] {
        let p = site_root.join(req_dir);
        if !p.is_dir() {
            return Err(format!(
                "Invalid Pico site folder: missing `{}/` directory.",
                req_dir
            ));
        }
    }
    let _ = fs::create_dir_all(site_root.join("out"));

    let config = load_site_config(site_root).map_err(|e| e.to_string())?;
    let collections = load_models(site_root).map_err(|e| e.to_string())?;

    let mut recent_files: Vec<String> = Vec::new();
    let content_dir = site_root.join("content");
    for entry in WalkDir::new(&content_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|e| e.to_str()) == Some("md")
        {
            if let Ok(rel) = entry.path().strip_prefix(site_root) {
                recent_files.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    recent_files.sort();

    let canonical = site_root
        .canonicalize()
        .unwrap_or_else(|_| site_root.to_path_buf());
    let recent_sites = record_recent_site(&canonical, &config.title);

    Ok(SiteInfo {
        path: canonical.to_string_lossy().to_string(),
        config,
        collections,
        recent_files,
        recent_sites,
    })
}

/// Core implementation of `create_site` (§3.1, §5).
/// Copies starter pack (`blog`, `portfolio`, `landing`, `docs`) into `dest_dir` and replaces
/// `{{SITE_TITLE}}` and `{{AUTHOR}}` placeholders in `site.yaml` ONLY.
pub fn create_site_impl(
    dest_dir: &Path,
    starter: &str,
    title: &str,
    author: &str,
) -> Result<SiteInfo, String> {
    let starter_key = starter.trim().to_ascii_lowercase();
    if !matches!(
        starter_key.as_str(),
        "blog" | "portfolio" | "landing" | "docs"
    ) {
        return Err(format!(
            "Unknown starter '{}'. Choose one of: Blog, Portfolio, Landing, Docs.",
            starter
        ));
    }

    let mut copied = false;
    if let Ok(starters_root) = resolve_starters_dir() {
        let starter_src = starters_root.join(&starter_key);
        if starter_src.is_dir() {
            copy_starter_pack(&starter_src, dest_dir)?;
            copied = true;
        }
    }

    if !copied {
        if let Some(embedded_starter) = EMBEDDED_STARTERS.get_dir(&starter_key) {
            fs::create_dir_all(dest_dir).map_err(|e| format!("Could not create site directory: {}", e))?;
            extract_embedded_starter(embedded_starter, dest_dir, &starter_key)
                .map_err(|e| format!("Could not extract embedded starter '{}': {}", starter_key, e))?;
        } else {
            return Err(format!(
                "Starter template '{}' was not found either on disk or in embedded bundle.",
                starter_key
            ));
        }
    }

    // Replace {{SITE_TITLE}} and {{AUTHOR}} in site.yaml ONLY (§3.1)
    let site_yaml_path = dest_dir.join("site.yaml");
    let raw_site_yaml = fs::read_to_string(&site_yaml_path)
        .map_err(|e| format!("Failed to read site.yaml in new site: {}", e))?;
    let clean_title = if title.trim().is_empty() {
        "My Pico Site"
    } else {
        title.trim()
    };
    let clean_author = if author.trim().is_empty() {
        "Author"
    } else {
        author.trim()
    };
    let updated_yaml = raw_site_yaml
        .replace("{{SITE_TITLE}}", clean_title)
        .replace("{{AUTHOR}}", clean_author);
    fs::write(&site_yaml_path, updated_yaml)
        .map_err(|e| format!("Failed to write site.yaml: {}", e))?;

    open_site_impl(dest_dir)
}

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
                    if let Some(obj) = fm.as_object() {
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
                        let draft = obj
                            .get("draft")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
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
                        let url = format!("/{}/{}/", folder_name, slug);
                        posts.push(PostMeta {
                            title,
                            date,
                            slug,
                            draft,
                            path: rel_path,
                            url,
                            collection: collection.to_string(),
                            excerpt,
                            tags,
                            hero,
                        });
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

/// Core implementation of `save_model`.
pub fn save_model_impl(site_root: &Path, model: &ModelDef) -> Result<(), String> {
    if model.name.trim().is_empty() {
        return Err("Model name cannot be empty.".to_string());
    }
    let slug_name = slugify(&model.name);
    let models_dir = site_root.join("models");
    fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;

    let mut clean_model = model.clone();
    clean_model.name = slug_name.clone();
    if clean_model.label.trim().is_empty() {
        clean_model.label = model.name.clone();
    }
    if clean_model.template.trim().is_empty() {
        clean_model.template = format!("{}.jinja", slug_name);
    }
    if clean_model.list_template.trim().is_empty() {
        clean_model.list_template = format!("{}-list.jinja", slug_name);
    }

    // Ensure title and date fields exist in the model if not present
    let has_title = clean_model.fields.iter().any(|f| f.name == "title");
    if !has_title {
        clean_model.fields.insert(
            0,
            ModelField {
                name: "title".to_string(),
                label: "Title".to_string(),
                field_type: "string".to_string(),
                required: true,
                default: None,
            },
        );
    }
    let has_date = clean_model.fields.iter().any(|f| f.name == "date");
    if !has_date {
        clean_model.fields.insert(
            1,
            ModelField {
                name: "date".to_string(),
                label: "Date".to_string(),
                field_type: "date".to_string(),
                required: true,
                default: None,
            },
        );
    }

    let yaml_str = serde_yaml::to_string(&clean_model)
        .map_err(|e| format!("Could not serialize model to YAML: {}", e))?;

    let file_path = models_dir.join(format!("{}.yaml", slug_name));
    fs::write(&file_path, yaml_str)
        .map_err(|e| format!("Could not write model file: {}", e))?;

    // Ensure collection content directory exists
    let content_dir = site_root.join("content").join(&slug_name);
    let _ = fs::create_dir_all(content_dir);

    Ok(())
}

/// Core implementation of `delete_model`.
pub fn delete_model_impl(site_root: &Path, name: &str) -> Result<(), String> {
    let slug_name = slugify(name);
    let models = load_models(site_root).map_err(|e| e.to_string())?;
    if models.len() <= 1 {
        return Err("Cannot delete the only remaining collection model in the site.".to_string());
    }

    let models_dir = site_root.join("models");
    let target1 = models_dir.join(format!("{}.yaml", slug_name));
    let target2 = models_dir.join(format!("{}.yml", slug_name));

    let file_to_del = if target1.is_file() {
        Some(target1)
    } else if target2.is_file() {
        Some(target2)
    } else {
        None
    };

    if let Some(f) = file_to_del {
        trash::delete(&f)
            .or_else(|_| fs::remove_file(&f))
            .map_err(|e| format!("Could not delete model file {}: {}", slug_name, e))?;
        Ok(())
    } else {
        Err(format!("Model '{}' not found on disk.", name))
    }
}

/// Core implementation of `save_template` with MiniJinja validation and revert backup (§3.4, §5).
pub fn save_template_impl(
    site_root: &Path,
    template_rel_path: &str,
    content: &str,
    backups: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    let norm = template_rel_path
        .trim_start_matches("templates/")
        .trim_start_matches('/')
        .replace('\\', "/");
    let models = load_models(site_root).map_err(|e| e.to_string())?;
    let all_templates = load_all_templates(site_root).map_err(|e| e.to_string())?;

    validate_template_with_models(&norm, content, &all_templates, &models)
        .map_err(|e| e.to_string())?;

    let tpl_full_path = site_root.join("templates").join(&norm);

    // Save current version for one-click Revert (§3.4)
    if tpl_full_path.is_file() {
        if let Ok(prev_content) = fs::read_to_string(&tpl_full_path) {
            if prev_content != content {
                backups
                    .entry(norm.clone())
                    .or_default()
                    .push(prev_content.clone());
                let disk_bak = site_root
                    .join(".pico_history")
                    .join("templates")
                    .join(format!("{}.bak", norm));
                if let Some(parent) = disk_bak.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(disk_bak, prev_content);
            }
        }
    }

    if let Some(parent) = tpl_full_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&tpl_full_path, content)
        .map_err(|e| format!("templates/{}: Could not write template: {}", norm, e))?;
    Ok(())
}

/// Core implementation of `revert_template` (§3.4, §5).
pub fn revert_template_impl(
    site_root: &Path,
    template_rel_path: &str,
    backups: &mut HashMap<String, Vec<String>>,
) -> Result<String, String> {
    let norm = template_rel_path
        .trim_start_matches("templates/")
        .trim_start_matches('/')
        .replace('\\', "/");
    let tpl_full_path = site_root.join("templates").join(&norm);

    let prev = if let Some(stack) = backups.get_mut(&norm) {
        stack.pop()
    } else {
        None
    }
    .or_else(|| {
        let disk_bak = site_root
            .join(".pico_history")
            .join("templates")
            .join(format!("{}.bak", norm));
        if disk_bak.is_file() {
            let s = fs::read_to_string(&disk_bak).ok();
            let _ = fs::remove_file(&disk_bak);
            s
        } else {
            None
        }
    })
    .ok_or_else(|| format!("No previous version available to revert for `{}`.", norm))?;

    fs::write(&tpl_full_path, &prev)
        .map_err(|e| format!("Failed to restore previous version of {}: {}", norm, e))?;
    Ok(prev)
}

/// Core implementation of `export_site` (§3.6, §5).
pub fn export_site_impl(site_root: &Path, dest_dir: &Path) -> Result<(), String> {
    let out_dir = site_root.join("out");
    if !out_dir.is_dir() || fs::read_dir(&out_dir).map(|mut d| d.next().is_none()).unwrap_or(true) {
        let res = build_site_pipeline(site_root, false, false);
        if !res.success {
            let msg = res
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            return Err(format!("Cannot export because the build has errors:\n{}", msg));
        }
    }

    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Could not create export folder {}: {}", dest_dir.display(), e))?;

    for entry in WalkDir::new(&out_dir).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        let rel = p.strip_prefix(&out_dir).map_err(|e| e.to_string())?;
        let target = dest_dir.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(p, &target).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn require_current_site(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    let guard = state.current_site.lock().map_err(|_| "State lock error")?;
    guard
        .clone()
        .ok_or_else(|| "No site folder is currently open. Please open or create a site first.".to_string())
}

// ============================================================================
// Tauri Commands (§5)
// ============================================================================

#[tauri::command]
fn open_site(path: String, state: State<'_, AppState>) -> Result<SiteInfo, String> {
    let site_root = PathBuf::from(&path);
    let info = open_site_impl(&site_root)?;
    let canonical = PathBuf::from(&info.path);

    {
        let mut cur = state.current_site.lock().map_err(|_| "State lock error")?;
        *cur = Some(canonical.clone());
    }

    // Start live-reload file watcher (300ms debounce)
    if let Ok(watcher) = start_site_watcher(canonical, state.preview_revision.clone()) {
        if let Ok(mut w_guard) = state.watcher_handle.lock() {
            *w_guard = Some(watcher);
        }
    }

    Ok(info)
}

#[tauri::command]
fn create_site(
    path: String,
    starter: String,
    title: String,
    author: String,
    state: State<'_, AppState>,
) -> Result<SiteInfo, String> {
    let dest = PathBuf::from(&path);
    let info = create_site_impl(&dest, &starter, &title, &author)?;
    let canonical = PathBuf::from(&info.path);

    {
        let mut cur = state.current_site.lock().map_err(|_| "State lock error")?;
        *cur = Some(canonical.clone());
    }

    if let Ok(watcher) = start_site_watcher(canonical, state.preview_revision.clone()) {
        if let Ok(mut w_guard) = state.watcher_handle.lock() {
            *w_guard = Some(watcher);
        }
    }

    Ok(info)
}

#[tauri::command]
fn get_recent_sites() -> Result<Vec<RecentSite>, String> {
    Ok(load_persisted_settings().recent_sites)
}

#[tauri::command]
fn list_content(
    collection: String,
    state: State<'_, AppState>,
) -> Result<Vec<PostMeta>, String> {
    let site_root = require_current_site(&state)?;
    list_content_impl(&site_root, &collection)
}

#[tauri::command]
fn read_post(path: String, state: State<'_, AppState>) -> Result<PostDocument, String> {
    let site_root = require_current_site(&state)?;
    read_post_impl(&site_root, &path)
}

#[tauri::command]
fn save_post(
    path: String,
    frontmatter: Value,
    markdown: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    save_post_impl(&site_root, &path, frontmatter, &markdown)
}

#[tauri::command]
fn delete_post(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    let full_path = if Path::new(&path).is_absolute() {
        PathBuf::from(&path)
    } else {
        site_root.join(path.trim_start_matches('/'))
    };
    if !full_path.exists() {
        return Err(format!("Post '{}' does not exist.", path));
    }

    // Try moving to OS Trash first (§3.2); fall back to file removal since user already confirmed in dialog
    if trash::delete(&full_path).is_err() {
        fs::remove_file(&full_path)
            .map_err(|e| format!("Failed to delete '{}': {}", path, e))?;
    }
    Ok(())
}

#[tauri::command]
fn save_image(
    source_path: String,
    post_slug: String,
    collection: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    save_image_impl(&site_root, Path::new(&source_path), &post_slug, &collection)
}

#[tauri::command]
fn save_image_bytes(
    file_name: String,
    bytes: Vec<u8>,
    post_slug: String,
    collection: String,
    site_wide: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
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
        Ok(format!("/assets/{}", safe_name))
    } else {
        let (_, col_dir) = resolve_collection_folder(&site_root, &collection);
        let slug_clean = slugify(&post_slug);
        let dest_dir = col_dir.join(&slug_clean);
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let dest = dest_dir.join(&safe_name);
        fs::write(&dest, &bytes)
            .map_err(|e| format!("Could not write image to post folder: {}", e))?;
        Ok(safe_name)
    }
}

#[tauri::command]
fn check_image_size(source_path: String) -> Result<u64, String> {
    let meta = fs::metadata(&source_path)
        .map_err(|e| format!("Could not inspect image '{}': {}", source_path, e))?;
    Ok(meta.len())
}

#[tauri::command]
fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelDef>, String> {
    let site_root = require_current_site(&state)?;
    load_models(&site_root).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_model(model: ModelDef, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    save_model_impl(&site_root, &model)
}

#[tauri::command]
fn delete_model(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    delete_model_impl(&site_root, &name)
}

#[tauri::command]
fn list_templates(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let site_root = require_current_site(&state)?;
    let map = load_all_templates(&site_root).map_err(|e| e.to_string())?;
    let mut names: Vec<String> = map.into_keys().collect();
    names.sort_by(|a, b| {
        if a == "skeleton.jinja" {
            std::cmp::Ordering::Less
        } else if b == "skeleton.jinja" {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });
    Ok(names)
}

#[tauri::command]
fn read_template(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    let norm = path
        .trim_start_matches("templates/")
        .trim_start_matches('/')
        .replace('\\', "/");
    let full = site_root.join("templates").join(&norm);
    fs::read_to_string(&full).map_err(|e| format!("templates/{}: Could not read file: {}", norm, e))
}

#[tauri::command]
fn save_template(
    path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    let mut backups = state
        .template_backups
        .lock()
        .map_err(|_| "State lock error")?;
    save_template_impl(&site_root, &path, &content, &mut backups)
}

#[tauri::command]
fn revert_template(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    let mut backups = state
        .template_backups
        .lock()
        .map_err(|_| "State lock error")?;
    revert_template_impl(&site_root, &path, &mut backups)
}

#[tauri::command]
fn get_template_context(
    path: String,
    state: State<'_, AppState>,
) -> Result<TemplateContextInfo, String> {
    let site_root = require_current_site(&state)?;
    let norm = path
        .trim_start_matches("templates/")
        .trim_start_matches('/')
        .replace('\\', "/");
    let models = load_models(&site_root).map_err(|e| e.to_string())?;

    let matched_model = models.iter().find(|m| {
        m.template == norm
            || m.list_template == norm
            || norm.starts_with(&format!("{}.", m.name))
            || norm.starts_with(&format!("{}-", m.name))
    });

    let mut variables = Vec::new();
    for sf in SITE_CONFIG_FIELDS {
        variables.push(VariableDoc {
            name: format!("site.{}", sf),
            kind: "site".to_string(),
            description: format!("Site configuration field `{}` from site.yaml", sf),
        });
    }

    let target_models: Vec<&ModelDef> = if let Some(m) = matched_model {
        vec![m]
    } else {
        models.iter().collect()
    };

    for m in &target_models {
        for f in &m.fields {
            variables.push(VariableDoc {
                name: format!("post.{}", f.name),
                kind: f.field_type.clone(),
                description: format!("{} ({})", f.label, m.label),
            });
        }
    }

    for bf in BUILTIN_ITEM_FIELDS {
        variables.push(VariableDoc {
            name: format!("post.{}", bf),
            kind: "computed".to_string(),
            description: format!("Built-in computed post property `{}`", bf),
        });
    }

    let has_mem_revert = state
        .template_backups
        .lock()
        .ok()
        .and_then(|b| b.get(&norm).map(|v| !v.is_empty()))
        .unwrap_or(false);
    let disk_bak = site_root
        .join(".pico_history")
        .join("templates")
        .join(format!("{}.bak", norm));

    Ok(TemplateContextInfo {
        template_path: norm.clone(),
        is_skeleton: norm == "skeleton.jinja",
        is_macro: norm.starts_with("macros/"),
        collection_label: matched_model.map(|m| m.label.clone()),
        has_revert: has_mem_revert || disk_bak.is_file(),
        variables,
    })
}

#[tauri::command]
fn read_theme(state: State<'_, AppState>) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    let p = site_root.join("assets").join("theme.css");
    fs::read_to_string(&p).map_err(|e| format!("assets/theme.css: {}", e))
}

#[tauri::command]
fn save_theme(content: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    if content.contains("{%") || content.contains("{{") {
        return Err("assets/theme.css must contain CSS only, not MiniJinja tags.".to_string());
    }
    let p = site_root.join("assets").join("theme.css");
    fs::write(&p, content).map_err(|e| format!("Could not save assets/theme.css: {}", e))
}

#[tauri::command]
fn build_site(include_drafts: bool, state: State<'_, AppState>) -> Result<BuildResult, String> {
    let site_root = require_current_site(&state)?;
    Ok(build_site_pipeline(&site_root, include_drafts, false))
}

#[tauri::command]
async fn start_preview(state: State<'_, AppState>) -> Result<String, String> {
    let site_root = require_current_site(&state)?;

    // Build site with include_drafts = true and live-reload script injected (§3.5)
    let build_res = build_site_pipeline(&site_root, true, true);
    if !build_res.success {
        let errs = build_res
            .errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!("Preview build failed:\n{}", errs));
    }

    let out_dir = site_root.join("out");
    state.preview_revision.fetch_add(1, Ordering::SeqCst);

    // Check if preview server is already running for this out_dir
    {
        let guard = state.preview_server.lock().map_err(|_| "State lock error")?;
        if let Some(existing) = guard.as_ref() {
            if existing.out_dir == out_dir {
                return Ok(format!("http://127.0.0.1:{}/", existing.port));
            }
        }
    }

    let port = start_axum_preview_server(out_dir.clone(), state.preview_revision.clone()).await?;
    {
        let mut guard = state.preview_server.lock().map_err(|_| "State lock error")?;
        *guard = Some(PreviewServerHandle {
            port,
            out_dir,
            revision: state.preview_revision.clone(),
        });
    }

    Ok(format!("http://127.0.0.1:{}/", port))
}

#[tauri::command]
fn get_preview_revision(state: State<'_, AppState>) -> Result<u64, String> {
    Ok(state.preview_revision.load(Ordering::SeqCst))
}

#[tauri::command]
fn export_site(dest_path: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    export_site_impl(&site_root, Path::new(&dest_path))
}

#[tauri::command]
async fn publish_github_pages(state: State<'_, AppState>) -> Result<PublishResult, String> {
    let site_root = require_current_site(&state)?;
    let settings = load_persisted_settings();
    Ok(publish_to_github_pages(&site_root, &settings).await)
}

#[tauri::command]
async fn vibe_edit(
    target: VibeTarget,
    prompt: String,
    options: Option<VibeOptions>,
    state: State<'_, AppState>,
) -> Result<VibeResult, String> {
    let site_root = require_current_site(&state)?;
    let settings = load_persisted_llm_settings();
    let opts = options.unwrap_or_default();
    let provider_id = opts
        .provider
        .clone()
        .unwrap_or_else(|| settings.current_provider.clone());

    if provider_id == crate::llm::PROVIDER_GEMINI_NATIVE {
        let prov = settings.providers.get(&provider_id).ok_or_else(|| {
            format!("Provider '{}' not found in settings.", provider_id)
        })?;
        if prov.api_key.trim().is_empty() {
            return Err("Configure an LLM API key in Settings to use Vibe.".to_string());
        }
        let original_content = read_vibe_target_content(&site_root, &target)?;
        let system_prompt = build_vibe_system_prompt(&site_root, &target)?;
        let user_msg = format!(
            "{}\n\nInstruction: {}\n\nCurrent file (`{}`):\n{}",
            system_prompt,
            prompt.trim(),
            target.path,
            original_content
        );
        let model = opts
            .model
            .as_ref()
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| {
                if !prov.custom_model.trim().is_empty() {
                    &prov.custom_model
                } else {
                    &prov.model
                }
            });
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model,
            prov.api_key.trim()
        );
        let thinking = opts.thinking.as_deref().unwrap_or(&prov.thinking);
        let grounding = opts.grounding.unwrap_or(prov.grounding);
        let mapped_thinking = crate::llm::map_thinking_setting(&provider_id, model, thinking)?;
        let mut config_obj = serde_json::Map::new();
        if grounding {
            config_obj.insert(
                "tools".to_string(),
                serde_json::json!([{ "googleSearch": {} }]),
            );
        }
        match mapped_thinking {
            crate::llm::ThinkingMapping::GeminiLevel(lvl) => {
                config_obj.insert(
                    "thinkingConfig".to_string(),
                    serde_json::json!({ "thinkingLevel": lvl }),
                );
            }
            crate::llm::ThinkingMapping::GeminiBudget(budget) => {
                config_obj.insert(
                    "thinkingConfig".to_string(),
                    serde_json::json!({ "thinkingBudget": budget }),
                );
            }
            _ => {}
        }
        let payload = serde_json::json!({
            "contents": [{ "parts": [{ "text": user_msg }] }],
            "generationConfig": serde_json::Value::Object(config_obj)
        });
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_default();
        let resp = client
            .post(&endpoint)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                crate::llm::format_provider_error(&provider_id, model, None, &e.to_string())
            })?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(crate::llm::format_provider_error(
                &provider_id,
                model,
                Some(status.as_u16()),
                &body,
            ));
        }
        let json_val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Invalid JSON response from Gemini API: {}", e))?;

        let mut text_parts = Vec::new();
        if let Some(parts) = json_val
            .pointer("/candidates/0/content/parts")
            .and_then(|v| v.as_array())
        {
            for part in parts {
                let is_thought = part.get("thought").and_then(|v| v.as_bool()).unwrap_or(false);
                if !is_thought {
                    if let Some(txt) = part.get("text").and_then(|v| v.as_str()) {
                        text_parts.push(txt);
                    }
                }
            }
        }
        let proposed_raw = text_parts.join("");
        let mut sources = Vec::new();
        if let Some(chunks) = json_val
            .pointer("/candidates/0/groundingMetadata/groundingChunks")
            .and_then(|v| v.as_array())
        {
            for chunk in chunks {
                if let Some(web) = chunk.get("web") {
                    let title = web
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Grounding Source")
                        .to_string();
                    let url = web.get("uri").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if !url.is_empty() {
                        sources.push(GroundingSource {
                            title,
                            url,
                            snippet: None,
                        });
                    }
                }
            }
        }
        let sources_opt = if grounding { Some(sources) } else { None };
        crate::vibe::evaluate_vibe_proposal_with_sources(
            &site_root,
            &target,
            &original_content,
            &proposed_raw,
            sources_opt,
        )
    } else {
        crate::llm::run_openai_vibe_edit(
            &site_root,
            &target,
            &prompt,
            Some(opts),
            &settings,
            None,
        )
        .await
    }
}

#[tauri::command]
fn get_site_config(state: State<'_, AppState>) -> Result<SiteConfig, String> {
    let site_root = require_current_site(&state)?;
    load_site_config(&site_root).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_site_config(config: SiteConfig, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("Could not serialize site.yaml: {}", e))?;
    fs::write(site_root.join("site.yaml"), yaml)
        .map_err(|e| format!("Could not write site.yaml: {}", e))
}

#[tauri::command]
fn get_app_settings() -> Result<AppSettingsView, String> {
    let s = load_persisted_settings();
    Ok(AppSettingsView {
        llm_endpoint: s.llm_endpoint,
        llm_model: s.llm_model,
        has_llm_key: !s.llm_api_key.trim().is_empty(),
        has_github_token: !s.github_token.trim().is_empty(),
        storage_notice: "Keys are stored locally in ~/.config/pico/settings.json and are only sent to your configured LLM or GitHub API endpoint.".to_string(),
    })
}

#[tauri::command]
fn save_app_settings(
    llm_endpoint: String,
    llm_model: String,
    llm_api_key: Option<String>,
    github_token: Option<String>,
) -> Result<AppSettingsView, String> {
    let mut s = load_persisted_settings();
    s.llm_endpoint = if llm_endpoint.trim().is_empty() {
        crate::models::default_llm_endpoint()
    } else {
        llm_endpoint.trim().to_string()
    };
    s.llm_model = if llm_model.trim().is_empty() {
        crate::models::default_llm_model()
    } else {
        llm_model.trim().to_string()
    };
    if let Some(k) = llm_api_key {
        s.llm_api_key = k.trim().to_string();
    }
    if let Some(t) = github_token {
        s.github_token = t.trim().to_string();
    }
    save_persisted_settings(&s)?;
    get_app_settings()
}

#[tauri::command]
fn list_llm_providers() -> Result<Vec<ProviderInfo>, String> {
    Ok(crate::llm::get_provider_registry())
}

#[tauri::command]
fn get_llm_settings() -> Result<LlmSettingsView, String> {
    let s = load_persisted_llm_settings();
    Ok(crate::llm::build_settings_view(&s))
}

#[tauri::command]
fn save_llm_settings(settings: SaveLlmSettingsInput) -> Result<(), String> {
    let mut current = load_persisted_llm_settings();
    crate::llm::apply_save_settings_input(&mut current, settings)?;
    save_persisted_llm_settings(&current)
}

#[tauri::command]
fn get_llm_key(provider: String) -> Result<String, String> {
    let s = load_persisted_llm_settings();
    let prov = s.providers.get(&provider).ok_or_else(|| {
        format!("Unknown provider '{}'.", provider)
    })?;
    if prov.api_key.trim().is_empty() {
        return Err("Configure an LLM API key in Settings to use Vibe.".to_string());
    }
    Ok(prov.api_key.trim().to_string())
}

#[tauri::command]
async fn test_llm_connection(provider: String, model: String) -> Result<TestConnectionResult, String> {
    let s = load_persisted_llm_settings();
    Ok(crate::llm::test_llm_connection_impl(&provider, Some(&model), &s, None).await)
}

#[tauri::command]
fn get_vibe_context(target: VibeTarget, state: State<'_, AppState>) -> Result<VibeContext, String> {
    let site_root = require_current_site(&state)?;
    let original_content = read_vibe_target_content(&site_root, &target)?;
    let system_prompt = build_vibe_system_prompt(&site_root, &target)?;
    Ok(VibeContext {
        original_content,
        system_prompt,
    })
}

#[tauri::command]
fn evaluate_vibe_proposal_cmd(
    target: VibeTarget,
    proposed: String,
    sources: Option<Vec<GroundingSource>>,
    state: State<'_, AppState>,
) -> Result<VibeResult, String> {
    let site_root = require_current_site(&state)?;
    let original_content = read_vibe_target_content(&site_root, &target)?;
    crate::vibe::evaluate_vibe_proposal_with_sources(
        &site_root,
        &target,
        &original_content,
        &proposed,
        sources,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            open_site,
            create_site,
            get_recent_sites,
            list_content,
            read_post,
            save_post,
            delete_post,
            save_image,
            save_image_bytes,
            check_image_size,
            list_models,
            save_model,
            delete_model,
            list_templates,
            read_template,
            save_template,
            revert_template,
            get_template_context,
            read_theme,
            save_theme,
            build_site,
            start_preview,
            get_preview_revision,
            export_site,
            publish_github_pages,
            vibe_edit,
            get_site_config,
            save_site_config,
            get_app_settings,
            save_app_settings,
            list_llm_providers,
            get_llm_settings,
            save_llm_settings,
            get_llm_key,
            test_llm_connection,
            get_vibe_context,
            evaluate_vibe_proposal_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Pico application");
}

// ============================================================================
// Verification & Unit / Integration Tests (§11)
// ============================================================================
#[cfg(test)]
mod tests {
    use crate::{
        create_site_impl, export_site_impl, revert_template_impl, save_post_impl,
        save_template_impl,
    };
    use crate::builder::{
        BuiltPost, build_site_pipeline, derive_image_variant_paths, generate_robots_txt,
        generate_rss_xml, generate_sitemap_xml, group_by_tag, paginate_chunks,
        render_markdown_to_html,
    };
    use crate::models::{ModelDef, ModelField, SiteConfig, VibeTarget};
    use crate::validation::{
        check_duplicate_slugs, parse_markdown_document, slugify, validate_frontmatter,
        validate_template_with_models,
    };
    use crate::vibe::evaluate_vibe_proposal;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn sample_blog_model() -> ModelDef {
        ModelDef {
            name: "blog".to_string(),
            label: "Blog".to_string(),
            template: "blog.jinja".to_string(),
            list_template: "blog-list.jinja".to_string(),
            fields: vec![
                ModelField {
                    name: "title".to_string(),
                    label: "Title".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    default: None,
                },
                ModelField {
                    name: "date".to_string(),
                    label: "Date".to_string(),
                    field_type: "date".to_string(),
                    required: true,
                    default: None,
                },
                ModelField {
                    name: "tags".to_string(),
                    label: "Tags".to_string(),
                    field_type: "tags".to_string(),
                    required: false,
                    default: None,
                },
                ModelField {
                    name: "excerpt".to_string(),
                    label: "Excerpt".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    default: None,
                },
                ModelField {
                    name: "hero".to_string(),
                    label: "Hero image".to_string(),
                    field_type: "image".to_string(),
                    required: false,
                    default: None,
                },
                ModelField {
                    name: "draft".to_string(),
                    label: "Draft".to_string(),
                    field_type: "bool".to_string(),
                    required: false,
                    default: Some(Value::Bool(false)),
                },
                ModelField {
                    name: "slug".to_string(),
                    label: "Slug".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    default: None,
                },
            ],
        }
    }

    #[test]
    fn test_frontmatter_parse_and_model_validation() {
        let model = sample_blog_model();

        // 1. Valid post matching §4.3
        let valid_md = r#"---
title: "Hello World"
date: "2026-11-03"
tags: [pico, intro]
excerpt: "First post."
hero: "hero.webp"
draft: false
slug: "hello-world"
---

# Hello World

Body in Markdown, edited in Milkdown Crepe.
"#;
        let (fm, body, raw_yaml) =
            parse_markdown_document(valid_md, "content/blog/hello-world.md").unwrap();
        assert!(!body.contains("---"));
        assert!(body.starts_with("# Hello World"));
        assert!(
            validate_frontmatter(&fm, &model, "content/blog/hello-world.md", Some(&raw_yaml))
                .is_ok()
        );

        // 2. Missing required field (`title`)
        let missing_req_md = r#"---
date: "2026-11-03"
draft: false
---

Body without title.
"#;
        let (fm_missing, _, yaml_missing) =
            parse_markdown_document(missing_req_md, "content/blog/no-title.md").unwrap();
        let err_missing = validate_frontmatter(
            &fm_missing,
            &model,
            "content/blog/no-title.md",
            Some(&yaml_missing),
        )
        .unwrap_err();
        assert!(err_missing
            .iter()
            .any(|e| e.message.contains("Missing required field 'title'")));

        // 3. Unknown field (`author_bio` not in Blog model)
        let unknown_field_md = r#"---
title: "Post with Unknown Field"
date: "2026-11-03"
author_bio: "Not in model"
---

Body text.
"#;
        let (fm_unk, _, yaml_unk) =
            parse_markdown_document(unknown_field_md, "content/blog/unk.md").unwrap();
        let err_unk =
            validate_frontmatter(&fm_unk, &model, "content/blog/unk.md", Some(&yaml_unk))
                .unwrap_err();
        assert!(err_unk
            .iter()
            .any(|e| e.message.contains("Unknown field 'author_bio'")));
    }

    #[test]
    fn test_slugify_and_duplicate_slug_detection() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(
            slugify("  Pico --- Static   Site   Generator 2026!! "),
            "pico-static-site-generator-2026"
        );
        assert_eq!(slugify("already-slugged"), "already-slugged");

        let entries = vec![
            (
                "content/blog/first.md".to_string(),
                "hello-world".to_string(),
                2,
            ),
            (
                "content/blog/second.md".to_string(),
                "unique-post".to_string(),
                2,
            ),
            (
                "content/blog/duplicate.md".to_string(),
                "hello-world".to_string(),
                2,
            ),
        ];
        let dup_errors = check_duplicate_slugs(&entries);
        assert_eq!(dup_errors.len(), 1);
        assert_eq!(dup_errors[0].file, "content/blog/duplicate.md");
        assert!(dup_errors[0].message.contains("Duplicate slug 'hello-world'"));
    }

    #[test]
    fn test_pagination_chunking_0_1_10_11_25() {
        // 0 items
        let items_0: Vec<usize> = vec![];
        assert_eq!(paginate_chunks(&items_0, 10).len(), 0);

        // 1 item
        let items_1: Vec<usize> = (1..=1).collect();
        let chunks_1 = paginate_chunks(&items_1, 10);
        assert_eq!(chunks_1.len(), 1);
        assert_eq!(chunks_1[0].len(), 1);

        // 10 items
        let items_10: Vec<usize> = (1..=10).collect();
        let chunks_10 = paginate_chunks(&items_10, 10);
        assert_eq!(chunks_10.len(), 1);
        assert_eq!(chunks_10[0].len(), 10);

        // 11 items
        let items_11: Vec<usize> = (1..=11).collect();
        let chunks_11 = paginate_chunks(&items_11, 10);
        assert_eq!(chunks_11.len(), 2);
        assert_eq!(chunks_11[0].len(), 10);
        assert_eq!(chunks_11[1].len(), 1);

        // 25 items
        let items_25: Vec<usize> = (1..=25).collect();
        let chunks_25 = paginate_chunks(&items_25, 10);
        assert_eq!(chunks_25.len(), 3);
        assert_eq!(chunks_25[0].len(), 10);
        assert_eq!(chunks_25[1].len(), 10);
        assert_eq!(chunks_25[2].len(), 5);
    }

    #[test]
    fn test_tag_grouping() {
        #[derive(Clone)]
        struct TaggedItem {
            id: &'static str,
            tags: Vec<String>,
        }
        let items = vec![
            TaggedItem {
                id: "a",
                tags: vec!["rust".to_string(), "pico".to_string()],
            },
            TaggedItem {
                id: "b",
                tags: vec!["pico".to_string(), "web".to_string()],
            },
            TaggedItem {
                id: "c",
                tags: vec!["rust".to_string()],
            },
        ];
        let grouped = group_by_tag(&items, |i| &i.tags);
        assert_eq!(grouped.get("pico").unwrap().len(), 2);
        assert_eq!(grouped.get(" pico ").is_none(), true);
        assert_eq!(grouped.get("rust").unwrap()[0].id, "a");
        assert_eq!(grouped.get("web").unwrap().len(), 1);
    }

    #[test]
    fn test_all_four_starters_build_cleanly_on_first_open() {
        let tmp = TempDir::new().unwrap();
        for starter in ["blog", "portfolio", "landing", "docs"] {
            let dir = tmp.path().join(format!("site-{}", starter));
            let info = create_site_impl(&dir, starter, "Starter Test", "Tester").unwrap();
            assert_eq!(info.config.title, "Starter Test");
            assert_eq!(info.config.author, "Tester");

            let res = build_site_pipeline(&dir, false, false);
            assert!(
                res.success,
                "Starter '{}' failed to build cleanly: {:?}",
                starter, res.errors
            );
            assert!(dir.join("out/index.html").is_file());
            assert!(dir.join("out/sitemap.xml").is_file());
            assert!(dir.join("out/robots.txt").is_file());
            assert!(dir.join("out/feed.xml").is_file());
            assert!(dir.join("out/search.json").is_file());
            assert!(dir.join("out/404.html").is_file());
        }
    }

    #[tokio::test]
    async fn test_axum_preview_server_serves_built_site_and_live_reload() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicU64;

        let tmp = TempDir::new().unwrap();
        let site_dir = tmp.path().join("preview-blog");
        create_site_impl(&site_dir, "blog", "Preview Site", "Author").unwrap();

        let build_res = build_site_pipeline(&site_dir, true, true);
        assert!(build_res.success);

        let rev = Arc::new(AtomicU64::new(42));
        let port = crate::preview::start_axum_preview_server(site_dir.join("out"), rev.clone())
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let index_resp = client
            .get(format!("http://127.0.0.1:{}/", port))
            .send()
            .await
            .unwrap();
        assert!(index_resp.status().is_success());
        let index_html = index_resp.text().await.unwrap();
        assert!(index_html.contains("Preview Site"));
        assert!(index_html.contains("/__pico_livereload"));

        let lr_resp = client
            .get(format!("http://127.0.0.1:{}/__pico_livereload", port))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert_eq!(lr_resp, "42");
    }

    #[test]
    fn test_save_and_delete_model_impl() {
        let tmp = TempDir::new().unwrap();
        let site_dir = tmp.path().join("model-site");
        create_site_impl(&site_dir, "blog", "Model Site", "Tester").unwrap();

        let new_model = ModelDef {
            name: "event".to_string(),
            label: "Events".to_string(),
            template: "event.jinja".to_string(),
            list_template: "event-list.jinja".to_string(),
            fields: vec![
                ModelField {
                    name: "title".to_string(),
                    label: "Title".to_string(),
                    field_type: "string".to_string(),
                    required: true,
                    default: None,
                },
                ModelField {
                    name: "date".to_string(),
                    label: "Event Date".to_string(),
                    field_type: "date".to_string(),
                    required: true,
                    default: None,
                },
                ModelField {
                    name: "location".to_string(),
                    label: "Location".to_string(),
                    field_type: "string".to_string(),
                    required: false,
                    default: None,
                },
            ],
        };

        crate::save_model_impl(&site_dir, &new_model).unwrap();
        let models_after_save = crate::builder::load_models(&site_dir).unwrap();
        assert!(models_after_save.iter().any(|m| m.name == "event"));

        // Delete model
        crate::delete_model_impl(&site_dir, "event").unwrap();
        let models_after_del = crate::builder::load_models(&site_dir).unwrap();
        assert!(!models_after_del.iter().any(|m| m.name == "event"));
    }

    #[test]
    fn test_embedded_starters_fallback_extracts_all_starters() {
        let tmp = TempDir::new().unwrap();
        for starter in ["blog", "portfolio", "landing", "docs"] {
            let dir = tmp.path().join(format!("embedded-{}", starter));
            let embedded = crate::EMBEDDED_STARTERS.get_dir(starter).unwrap();
            fs::create_dir_all(&dir).unwrap();
            crate::extract_embedded_starter(embedded, &dir, starter).unwrap();
            assert!(dir.join("site.yaml").is_file());
            assert!(dir.join("templates/skeleton.jinja").is_file());
            assert!(dir.join("models").is_dir());
            assert!(dir.join("content").is_dir());
        }
    }

    #[test]
    fn test_image_resize_path_derivation() {
        assert_eq!(
            derive_image_variant_paths("hero.png"),
            Some(("hero-hero.webp".to_string(), "hero-thumb.webp".to_string()))
        );
        assert_eq!(
            derive_image_variant_paths("assets/photo.JPG"),
            Some((
                "assets/photo-hero.webp".to_string(),
                "assets/photo-thumb.webp".to_string()
            ))
        );
        // SVG, GIF, AVIF are unchanged in MVP
        assert_eq!(derive_image_variant_paths("logo.svg"), None);
        assert_eq!(derive_image_variant_paths("anim.gif"), None);
        assert_eq!(derive_image_variant_paths("banner.avif"), None);
    }

    #[test]
    fn test_sitemap_robots_and_rss_generation() {
        let entries = vec![
            ("/".to_string(), "2026-11-03".to_string()),
            ("/blog/hello-world/".to_string(), "2026-11-03".to_string()),
        ];
        let sitemap = generate_sitemap_xml("https://example.com", &entries);
        assert!(sitemap.contains("<loc>https://example.com/</loc>"));
        assert!(sitemap.contains("<loc>https://example.com/blog/hello-world/</loc>"));
        assert!(sitemap.contains("<lastmod>2026-11-03</lastmod>"));

        let robots = generate_robots_txt("https://example.com");
        assert!(robots.contains("Allow: /"));
        assert!(robots.contains("Sitemap: https://example.com/sitemap.xml"));

        let site = SiteConfig::default();
        let posts = vec![BuiltPost {
            title: "Hello World".to_string(),
            date: "2026-11-03".to_string(),
            slug: "hello-world".to_string(),
            draft: false,
            excerpt: "First post.".to_string(),
            hero: String::new(),
            hero_webp: String::new(),
            thumb_webp: String::new(),
            tags: vec!["pico".to_string()],
            url: "/blog/hello-world/".to_string(),
            collection: "blog".to_string(),
            body: String::new(),
            body_html: "<p>Hello</p>".to_string(),
            body_text: "Hello".to_string(),
            attachments: vec![],
            source_rel_path: "content/blog/hello-world.md".to_string(),
            markdown_images: vec![],
            extra_fields: serde_json::Map::new(),
        }];
        let rss = generate_rss_xml(&site, &posts);
        assert!(rss.contains("<rss version=\"2.0\">"));
        assert!(rss.contains("<title>Hello World</title>"));
        assert!(rss.contains("<link>https://example.com/blog/hello-world/</link>"));
    }

    #[test]
    fn test_minijinja_macro_import_and_unknown_variable_rejection() {
        let model = sample_blog_model();
        let mut templates = HashMap::new();
        templates.insert(
            "skeleton.jinja".to_string(),
            "<!DOCTYPE html><html><head><title>{% block title %}{{ site.title }}{% endblock %}</title></head><body>{% block content %}{% endblock %}</body></html>".to_string(),
        );
        templates.insert(
            "macros/cards.jinja".to_string(),
            "{% macro post_card(post) %}\n<article><h2><a href=\"{{ post.url }}\">{{ post.title }}</a></h2><p>{{ post.excerpt }}</p></article>\n{% endmacro %}\n".to_string(),
        );

        // 1. Valid macro import in blog-list.jinja
        let valid_list = "{% extends \"skeleton.jinja\" %}\n{% from \"macros/cards.jinja\" import post_card %}\n{% block content %}{% for post in posts %}{{ post_card(post) }}{% endfor %}{% endblock %}\n";
        assert!(validate_template_with_models(
            "blog-list.jinja",
            valid_list,
            &templates,
            &[model.clone()]
        )
        .is_ok());

        // 2. Unknown variable in blog.jinja (`post.author_bio`) must be refused with friendly message naming file, line, and variable (§3.4)
        let broken_blog = "{% extends \"skeleton.jinja\" %}\n{% block content %}\n<h1>{{ post.title }}</h1>\n<p>{{ post.author_bio }}</p>\n{% endblock %}\n";
        let err = validate_template_with_models(
            "blog.jinja",
            broken_blog,
            &templates,
            &[model.clone()],
        )
        .unwrap_err();
        assert_eq!(err.file, "blog.jinja");
        assert_eq!(err.line, 4);
        assert!(
            err.message
                .contains("blog.jinja uses `post.author_bio`, but Blog posts don't have `author_bio`"),
            "Unexpected error message: {}",
            err.message
        );

        // 3. Macro file with top-level HTML outside {% macro %} must be rejected (§4.5)
        let bad_macro_html = "<div>Not allowed outside macro</div>\n{% macro post_card(post) %}<p>{{ post.title }}</p>{% endmacro %}\n";
        let err_macro = validate_template_with_models(
            "macros/cards.jinja",
            bad_macro_html,
            &templates,
            &[model.clone()],
        )
        .unwrap_err();
        assert!(err_macro.message.contains("ONLY `{% macro %}`"));

        // 4. Macro using kwargs must be rejected (§4.6)
        let bad_kwargs = "{% macro post_card(post, **kwargs) %}<p>{{ post.title }}</p>{% endmacro %}\n";
        let err_kw = validate_template_with_models(
            "macros/cards.jinja",
            bad_kwargs,
            &templates,
            &[model],
        )
        .unwrap_err();
        assert!(err_kw.message.contains("kwargs"));
    }

    #[test]
    fn test_end_to_end_starter_lifecycle_pagination_drafts_revert_export_and_vibe() {
        let tmp = TempDir::new().unwrap();
        let site_dir = tmp.path().join("my-blog");

        // 1. Create fresh Blog starter site
        let info = create_site_impl(&site_dir, "blog", "Lumina Journal", "Ada Lovelace").unwrap();
        assert_eq!(info.config.title, "Lumina Journal");
        assert_eq!(info.config.author, "Ada Lovelace");

        // 2. Add a post in Markdown with heading, bold, list, fenced rust code block, and image
        let (html, _, imgs) = render_markdown_to_html(
            "# Deep Dive\n\n**Bold insight**\n\n- Item 1\n- Item 2\n\n```rust\nfn demo() {}\n```\n\n![Hero](hero.png)\n",
        );
        assert!(html.contains("<strong>Bold insight</strong>"));
        assert!(html.contains("language-rust"));
        assert_eq!(imgs, vec!["hero.png".to_string()]);

        // 3. Edit `templates/macros/cards.jinja` via save_template_impl, then test broken template rejection & Revert
        let mut backups = HashMap::new();
        let updated_cards = "{% macro post_card(post) %}\n<article class=\"post-card custom-card\"><h2><a href=\"{{ post.url }}\">{{ post.title }}</a></h2><span class=\"card-date\">{{ post.date }}</span><p>{{ post.excerpt }}</p></article>\n{% endmacro %}\n";
        save_template_impl(&site_dir, "macros/cards.jinja", updated_cards, &mut backups).unwrap();

        // Break `blog.jinja` on purpose with `post.author_bio` -> save is refused, previous file intact
        let orig_blog_tpl = fs::read_to_string(site_dir.join("templates/blog.jinja")).unwrap();
        let bad_blog_tpl = format!("{}\n{{{{ post.author_bio }}}}", orig_blog_tpl);
        let save_err =
            save_template_impl(&site_dir, "blog.jinja", &bad_blog_tpl, &mut backups).unwrap_err();
        assert!(save_err.contains("post.author_bio"));
        assert_eq!(
            fs::read_to_string(site_dir.join("templates/blog.jinja")).unwrap(),
            orig_blog_tpl
        );

        // Revert `macros/cards.jinja` and re-apply custom card
        let reverted = revert_template_impl(&site_dir, "macros/cards.jinja", &mut backups).unwrap();
        assert!(!reverted.contains("custom-card"));
        save_template_impl(&site_dir, "macros/cards.jinja", updated_cards, &mut backups).unwrap();

        // 4. Add posts so total published posts >= 12 (starter has 3, add 10 published + 1 draft)
        for i in 1..=10 {
            let slug = format!("extra-post-{}", i);
            let fm = serde_json::json!({
                "title": format!("Extra Post {}", i),
                "date": format!("2026-10-{:02}", i),
                "tags": ["pico", "guide"],
                "excerpt": format!("Summary for post {}", i),
                "hero": "",
                "draft": false,
                "slug": slug
            });
            let path = format!("content/blog/{}.md", slug);
            save_post_impl(
                &site_dir,
                &path,
                fm,
                &format!("# Extra Post {}\n\nContent for post {}.", i, i),
            )
            .unwrap();
        }

        // Add 1 draft post
        let draft_fm = serde_json::json!({
            "title": "Secret Draft Post",
            "date": "2026-11-04",
            "tags": ["drafts"],
            "excerpt": "Work in progress",
            "hero": "",
            "draft": true,
            "slug": "secret-draft-post"
        });
        save_post_impl(
            &site_dir,
            "content/blog/secret-draft-post.md",
            draft_fm,
            "# Secret Draft\n\nNot ready yet.",
        )
        .unwrap();

        // 5. Preview build (`include_drafts = true`) includes draft with visible DRAFT badge
        let preview_res = build_site_pipeline(&site_dir, true, true);
        assert!(preview_res.success, "Preview build errors: {:?}", preview_res.errors);
        let draft_html =
            fs::read_to_string(site_dir.join("out/blog/secret-draft-post/index.html")).unwrap();
        assert!(draft_html.contains("DRAFT"));

        // 6. Final Build (`include_drafts = false`) excludes draft, paginates 13 published posts into 2 pages,
        // generates WebP variants, sitemap.xml, robots.txt, feed.xml, search.json, 404.html
        let build_res = build_site_pipeline(&site_dir, false, false);
        assert!(build_res.success, "Build errors: {:?}", build_res.errors);
        assert!(!site_dir.join("out/blog/secret-draft-post/index.html").exists());
        assert!(site_dir.join("out/index.html").is_file());
        assert!(site_dir.join("out/blog/index.html").is_file());
        assert!(site_dir.join("out/blog/page/2/index.html").is_file());
        assert!(site_dir.join("out/tags/pico/index.html").is_file());
        assert!(site_dir.join("out/sitemap.xml").is_file());
        assert!(site_dir.join("out/robots.txt").is_file());
        assert!(site_dir.join("out/feed.xml").is_file());
        assert!(site_dir.join("out/search.json").is_file());
        assert!(site_dir.join("out/404.html").is_file());
        assert!(site_dir
            .join("out/blog/hello-world/hero-hero.webp")
            .is_file());
        assert!(site_dir
            .join("out/blog/hello-world/hero-thumb.webp")
            .is_file());

        // 7. Export copies `out/` to a picked folder
        let export_dir = tmp.path().join("exported-site");
        export_site_impl(&site_dir, &export_dir).unwrap();
        assert!(export_dir.join("index.html").is_file());
        assert!(export_dir.join("blog/page/2/index.html").is_file());
        assert!(export_dir.join("search.json").is_file());

        // 8. Simulate Vibe diff + validation with mock LLM output (in tests only, §11.4)
        let vibe_target = VibeTarget {
            kind: "template".to_string(),
            path: "macros/cards.jinja".to_string(),
        };
        let valid_vibe_proposal = "{% macro post_card(post) %}\n<article class=\"post-card vibe-updated\"><h2><a href=\"{{ post.url }}\">{{ post.title }}</a></h2><time>{{ post.date }}</time></article>\n{% endmacro %}\n";
        let vibe_ok = evaluate_vibe_proposal(
            &site_dir,
            &vibe_target,
            updated_cards,
            valid_vibe_proposal,
        )
        .unwrap();
        assert!(vibe_ok.validation_error.is_none());
        assert!(vibe_ok.diff.contains("+<article class=\"post-card vibe-updated\">"));

        // Invalid LLM proposal using unknown variable is flagged by validation guardrail
        let invalid_vibe_proposal = "{% macro post_card(post) %}\n<article>{{ post.nonexistent_var }}</article>\n{% endmacro %}\n";
        let vibe_bad = evaluate_vibe_proposal(
            &site_dir,
            &vibe_target,
            updated_cards,
            invalid_vibe_proposal,
        )
        .unwrap();
        assert!(vibe_bad.validation_error.is_some());
        assert!(vibe_bad
            .validation_error
            .unwrap()
            .contains("post.nonexistent_var"));
    }
}
