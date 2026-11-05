use crate::models::{
    AppSettingsView, LlmSettingsView, ProviderInfo, SaveLlmSettingsInput, TestConnectionResult,
};
use crate::storage::{
    load_persisted_llm_settings, load_persisted_settings, save_persisted_llm_settings,
    save_persisted_settings,
};

#[tauri::command]
pub fn get_app_settings() -> Result<AppSettingsView, String> {
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
pub fn save_app_settings(
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
pub fn list_llm_providers() -> Result<Vec<ProviderInfo>, String> {
    Ok(crate::llm::get_provider_registry())
}

#[tauri::command]
pub fn get_llm_settings() -> Result<LlmSettingsView, String> {
    let s = load_persisted_llm_settings();
    Ok(crate::llm::build_settings_view(&s))
}

#[tauri::command]
pub fn save_llm_settings(settings: SaveLlmSettingsInput) -> Result<(), String> {
    let mut current = load_persisted_llm_settings();
    crate::llm::apply_save_settings_input(&mut current, settings)?;
    save_persisted_llm_settings(&current)
}

#[tauri::command]
pub fn get_llm_key(provider: String) -> Result<String, String> {
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
pub async fn test_llm_connection(provider: String, model: String) -> Result<TestConnectionResult, String> {
    log::info!(target: "pico::llm", "test_llm_connection requested: provider='{}', model='{}'", provider, model);
    let s = load_persisted_llm_settings();
    let res = crate::llm::test_llm_connection_impl(&provider, Some(&model), &s, None).await;
    log::info!(target: "pico::llm", "test_llm_connection result: provider='{}', model='{}', success={}, elapsed={}ms, error={:?}", provider, model, res.success, res.elapsed_ms, res.error);
    Ok(res)
}

#[tauri::command]
pub fn log_frontend(level: String, tag: String, message: String) {
    let clean_tag = if tag.trim().is_empty() {
        "frontend".to_string()
    } else if tag.starts_with("frontend") {
        tag
    } else {
        format!("frontend::{}", tag)
    };

    match level.to_lowercase().as_str() {
        "trace" => log::trace!(target: &clean_tag, "{}", message),
        "debug" => log::debug!(target: &clean_tag, "{}", message),
        "warn" => log::warn!(target: &clean_tag, "{}", message),
        "error" => log::error!(target: &clean_tag, "{}", message),
        _ => log::info!(target: &clean_tag, "{}", message),
    }
}
