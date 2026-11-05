use crate::models::{LlmAppSettings, RecentSite};
use std::fs;
use std::path::{Path, PathBuf};

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



pub fn record_recent_site(site_root: &Path, title: &str) -> Vec<RecentSite> {
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
