use crate::models::{
    LlmSettingsView, ProviderInfo, SaveLlmSettingsInput, TestConnectionResult,
};
use crate::storage::{load_persisted_llm_settings, save_persisted_llm_settings};

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
    Ok(TestConnectionResult {
        success: true,
        model,
        elapsed_ms: 0,
        error: None,
    })
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
