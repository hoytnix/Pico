use crate::models::{LlmAppSettings, PersistedAppSettings, RecentSite};
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
