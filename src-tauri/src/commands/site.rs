use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::State;
use walkdir::WalkDir;

use crate::builder::{build_site_pipeline, load_models, load_site_config};
use crate::models::{
    AppState, BuildResult, PreviewServerHandle, PublishResult, RecentSite, SiteConfig, SiteInfo,
};
use crate::preview::{start_axum_preview_server, start_site_watcher};
use crate::publish::publish_to_github_pages;
use crate::starters::{
    copy_starter_pack, extract_embedded_starter, resolve_starters_dir, EMBEDDED_STARTERS,
};
use crate::storage::{load_persisted_llm_settings, record_recent_site};
use super::require_current_site;

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

#[tauri::command]
pub fn open_site(path: String, state: State<'_, AppState>) -> Result<SiteInfo, String> {
    log::info!(target: "pico::site", "open_site requested: {:?}", path);
    let site_root = PathBuf::from(&path);
    let info = open_site_impl(&site_root).map_err(|err| {
        log::error!(target: "pico::site", "open_site failed for {:?}: {}", path, err);
        err
    })?;
    let canonical = PathBuf::from(&info.path);

    {
        let mut cur = state.current_site.lock().map_err(|_| "State lock error")?;
        *cur = Some(canonical.clone());
    }

    // Start live-reload file watcher (300ms debounce)
    if let Ok(watcher) = start_site_watcher(canonical.clone(), state.preview_revision.clone()) {
        if let Ok(mut w_guard) = state.watcher_handle.lock() {
            *w_guard = Some(watcher);
            log::debug!(target: "pico::site", "Preview watcher started for {:?}", canonical);
        }
    }

    log::info!(target: "pico::site", "open_site successful: title='{}', path='{}', collections={}", info.config.title, info.path, info.collections.len());
    Ok(info)
}

#[tauri::command]
pub fn create_site(
    path: String,
    starter: String,
    title: String,
    author: String,
    state: State<'_, AppState>,
) -> Result<SiteInfo, String> {
    log::info!(target: "pico::site", "create_site requested: path={:?}, starter='{}', title='{}', author='{}'", path, starter, title, author);
    let dest = PathBuf::from(&path);
    let info = create_site_impl(&dest, &starter, &title, &author).map_err(|err| {
        log::error!(target: "pico::site", "create_site failed for {:?}: {}", path, err);
        err
    })?;
    let canonical = PathBuf::from(&info.path);

    {
        let mut cur = state.current_site.lock().map_err(|_| "State lock error")?;
        *cur = Some(canonical.clone());
    }

    if let Ok(watcher) = start_site_watcher(canonical.clone(), state.preview_revision.clone()) {
        if let Ok(mut w_guard) = state.watcher_handle.lock() {
            *w_guard = Some(watcher);
            log::debug!(target: "pico::site", "Preview watcher started for {:?}", canonical);
        }
    }

    log::info!(target: "pico::site", "create_site successful: path='{}'", info.path);
    Ok(info)
}

#[tauri::command]
pub fn get_recent_sites() -> Result<Vec<RecentSite>, String> {
    let recent = load_persisted_llm_settings().recent_sites;
    log::debug!(target: "pico::site", "get_recent_sites: returned {} sites", recent.len());
    Ok(recent)
}

#[tauri::command]
pub fn build_site(include_drafts: bool, state: State<'_, AppState>) -> Result<BuildResult, String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::build", "build_site requested: include_drafts={}, site_root={:?}", include_drafts, site_root);
    let res = build_site_pipeline(&site_root, include_drafts, false);
    if res.success {
        log::info!(target: "pico::build", "build_site succeeded: pages_built={}, images_generated={}", res.pages_built, res.images_generated);
    } else {
        log::error!(target: "pico::build", "build_site completed with {} errors: {:?}", res.errors.len(), res.errors);
    }
    Ok(res)
}

#[tauri::command]
pub async fn start_preview(state: State<'_, AppState>) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::preview", "start_preview requested for {:?}", site_root);

    // Build site with include_drafts = true and live-reload script injected (§3.5)
    let build_res = build_site_pipeline(&site_root, true, true);
    if !build_res.success {
        let errs = build_res
            .errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        log::error!(target: "pico::preview", "start_preview build failed: {}", errs);
        return Err(format!("Preview build failed:\n{}", errs));
    }

    let out_dir = site_root.join("out");
    state.preview_revision.fetch_add(1, Ordering::SeqCst);

    // Check if preview server is already running for this out_dir
    {
        let guard = state.preview_server.lock().map_err(|_| "State lock error")?;
        if let Some(existing) = guard.as_ref() {
            if existing.out_dir == out_dir {
                let url = format!("http://127.0.0.1:{}/", existing.port);
                log::info!(target: "pico::preview", "Reusing running preview server at {}", url);
                return Ok(url);
            }
        }
    }

    let port = start_axum_preview_server(out_dir.clone(), state.preview_revision.clone()).await.map_err(|err| {
        log::error!(target: "pico::preview", "start_axum_preview_server failed: {}", err);
        err
    })?;
    {
        let mut guard = state.preview_server.lock().map_err(|_| "State lock error")?;
        *guard = Some(PreviewServerHandle {
            port,
            out_dir,
            revision: state.preview_revision.clone(),
        });
    }

    let url = format!("http://127.0.0.1:{}/", port);
    log::info!(target: "pico::preview", "Preview server successfully started at {}", url);
    Ok(url)
}

#[tauri::command]
pub fn get_preview_revision(state: State<'_, AppState>) -> Result<u64, String> {
    Ok(state.preview_revision.load(Ordering::SeqCst))
}

#[tauri::command]
pub fn export_site(dest_path: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::export", "export_site requested to {:?}", dest_path);
    export_site_impl(&site_root, Path::new(&dest_path)).map_err(|err| {
        log::error!(target: "pico::export", "export_site failed: {}", err);
        err
    })?;
    log::info!(target: "pico::export", "export_site succeeded to {:?}", dest_path);
    Ok(())
}

#[tauri::command]
pub async fn publish_github_pages(state: State<'_, AppState>) -> Result<PublishResult, String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::publish", "publish_github_pages requested for {:?}", site_root);
    let settings = load_persisted_llm_settings();
    let res = publish_to_github_pages(&site_root, &settings.github_token).await;
    log::info!(target: "pico::publish", "publish_github_pages result: success={}, message='{}', url={:?}", res.success, res.message, res.url);
    Ok(res)
}

#[tauri::command]
pub fn get_site_config(state: State<'_, AppState>) -> Result<SiteConfig, String> {
    let site_root = require_current_site(&state)?;
    load_site_config(&site_root).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_site_config(config: SiteConfig, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| format!("Could not serialize site.yaml: {}", e))?;
    fs::write(site_root.join("site.yaml"), yaml)
        .map_err(|e| format!("Could not write site.yaml: {}", e))
}
