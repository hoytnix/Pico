use super::registry::{
    create_default_providers_map, get_provider_registry, PROVIDER_CUSTOM, PROVIDER_GEMINI_NATIVE,
    PROVIDER_OPENAI,
};
use crate::models::{
    LlmAppSettings, LlmSettingsView, ProviderSettings, ProviderView, SaveLlmSettingsInput,
};
use std::collections::HashMap;

/// Validates base URL as per §4:
/// Must be `https://` (allow `http://localhost:*` / `http://127.0.0.1:*` only for local servers, reject all other `http://`).
pub fn validate_base_url(url: &str) -> Result<(), String> {
    let u = url.trim();
    if u.is_empty() {
        return Err("Base URL cannot be empty.".to_string());
    }
    if u.starts_with("https://") {
        return Ok(());
    }
    if u.starts_with("http://localhost") || u.starts_with("http://127.0.0.1") {
        return Ok(());
    }
    if u.starts_with("http://") {
        return Err(
            "HTTP is only allowed for localhost or 127.0.0.1. Remote providers must use HTTPS."
                .to_string(),
        );
    }
    Err("Base URL must start with https:// (or http://localhost / http://127.0.0.1 for local servers).".to_string())
}

/// Validates thinking setting as per §5.1 & §9:
/// Must be one of `low`, `medium`, `high`, `default` (case-insensitive).
pub fn validate_thinking(thinking: &str) -> Result<String, String> {
    let norm = thinking.trim().to_ascii_lowercase();
    match norm.as_str() {
        "low" | "medium" | "high" | "default" | "" => Ok(if norm.is_empty() {
            "default".to_string()
        } else {
            norm
        }),
        other => Err(format!(
            "Invalid thinking setting '{}'. Expected one of: low, medium, high, default.",
            other
        )),
    }
}

/// Validates saving of LLM settings (§9):
/// - effective model non-empty
/// - base_url valid
/// - thinking in {low, medium, high, default}
/// - grounding only true for gemini-native (reject grounding=true for other providers with friendly error)
pub fn validate_save_llm_settings(
    provider_id: &str,
    base_url: &str,
    model: &str,
    custom_model: &str,
    thinking: &str,
    grounding: bool,
) -> Result<(), String> {
    // Check provider exists in registry
    let registry = get_provider_registry();
    if !registry.iter().any(|p| p.id == provider_id) {
        return Err(format!("Unknown provider ID '{}'.", provider_id));
    }

    // Effective model validation
    let effective_model = if !custom_model.trim().is_empty() {
        custom_model.trim()
    } else {
        model.trim()
    };
    if effective_model.is_empty() {
        return Err(format!(
            "Model cannot be empty for provider '{}'.",
            provider_id
        ));
    }

    // Base URL validation (required for custom, or validate if provided)
    if provider_id == PROVIDER_CUSTOM || !base_url.trim().is_empty() {
        validate_base_url(base_url)?;
    }

    // Thinking validation
    validate_thinking(thinking)?;

    // Grounding gating (§5.2, §9):
    // "grounding only true for gemini-native (reject grounding=true for other providers with a friendly error, do not silently turn it off)."
    if grounding && provider_id != PROVIDER_GEMINI_NATIVE {
        return Err("Grounding requires Gemini Native, not an OpenAI provider.".to_string());
    }

    Ok(())
}

/// Converts an in-memory `LlmAppSettings` to the sanitized `LlmSettingsView`
/// showing SET / NOT SET key status, never exposing raw keys (§6, §9).
pub fn build_settings_view(settings: &LlmAppSettings) -> LlmSettingsView {
    let registry = get_provider_registry();
    let mut providers_map = HashMap::new();

    for p in registry {
        let prov_settings = settings
            .providers
            .get(&p.id)
            .cloned()
            .unwrap_or_else(|| ProviderSettings {
                api_key: String::new(),
                base_url: p.base_url.clone(),
                model: p.default_models.first().cloned().unwrap_or_default(),
                custom_model: String::new(),
                thinking: "medium".to_string(),
                grounding: false,
            });

        let effective_model = if !prov_settings.custom_model.trim().is_empty() {
            prov_settings.custom_model.trim().to_string()
        } else if !prov_settings.model.trim().is_empty() {
            prov_settings.model.trim().to_string()
        } else {
            p.default_models.first().cloned().unwrap_or_default()
        };

        let key_status = if !prov_settings.api_key.trim().is_empty() {
            "SET".to_string()
        } else {
            "NOT SET".to_string()
        };

        providers_map.insert(
            p.id.clone(),
            ProviderView {
                id: p.id,
                base_url: prov_settings.base_url,
                model: prov_settings.model,
                custom_model: prov_settings.custom_model,
                effective_model,
                thinking: prov_settings.thinking,
                grounding: prov_settings.grounding,
                key_status,
            },
        );
    }

    LlmSettingsView {
        current_provider: if settings.current_provider.is_empty() {
            PROVIDER_GEMINI_NATIVE.to_string()
        } else {
            settings.current_provider.clone()
        },
        providers: providers_map,
        github_token: String::new(), // Do not echo raw token in settings view
        has_github_token: !settings.github_token.trim().is_empty(),
    }
}

/// Migrates old single-endpoint settings if present (§7).
/// Preserves old endpoint, model, and key into provider `openai`.
/// Does not delete old key, does not migrate it to Gemini or Muse.
pub fn migrate_settings_json(raw: &str) -> Result<LlmAppSettings, String> {
    let val: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("Could not parse settings JSON: {}", e))?;

    // Check if new structure already exists
    if val.get("providers").is_some() {
        let parsed: LlmAppSettings = serde_json::from_value(val)
            .map_err(|e| format!("Invalid LLM settings schema: {}", e))?;
        return Ok(parsed);
    }

    // Otherwise, migrate from old single-endpoint shape (§7)
    let mut default_providers = create_default_providers_map();

    let old_endpoint = val
        .get("llm_endpoint")
        .and_then(|v| v.as_str())
        .unwrap_or("https://api.openai.com/v1/chat/completions");
    let old_model = val
        .get("llm_model")
        .and_then(|v| v.as_str())
        .unwrap_or("gpt-4o-mini");
    let old_key = val
        .get("llm_api_key")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let old_gh = val
        .get("github_token")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let old_sites: Vec<crate::models::RecentSite> = val
        .get("recent_sites")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Map old endpoint: if it ends with /chat/completions, extract base URL or preserve
    let clean_base_url = if old_endpoint.ends_with("/chat/completions") {
        old_endpoint
            .trim_end_matches("/chat/completions")
            .to_string()
    } else {
        old_endpoint.to_string()
    };

    // Store in openai provider
    default_providers.insert(
        PROVIDER_OPENAI.to_string(),
        ProviderSettings {
            api_key: old_key.to_string(),
            base_url: clean_base_url,
            model: old_model.to_string(),
            custom_model: String::new(),
            thinking: "medium".to_string(),
            grounding: false,
        },
    );

    Ok(LlmAppSettings {
        current_provider: PROVIDER_OPENAI.to_string(),
        providers: default_providers,
        github_token: old_gh.to_string(),
        recent_sites: old_sites,
    })
}

/// Applies a `SaveLlmSettingsInput` update onto current `LlmAppSettings`.
pub fn apply_save_settings_input(
    current: &mut LlmAppSettings,
    input: SaveLlmSettingsInput,
) -> Result<(), String> {
    if !input.current_provider.is_empty() {
        let registry = get_provider_registry();
        if !registry.iter().any(|p| p.id == input.current_provider) {
            return Err(format!("Unknown provider ID '{}'.", input.current_provider));
        }
        current.current_provider = input.current_provider.clone();
    }

    let target_provider = current.current_provider.clone();

    // If direct fields are provided on input, update the target provider
    {
        let entry = current
            .providers
            .entry(target_provider.clone())
            .or_insert_with(|| ProviderSettings {
                api_key: String::new(),
                base_url: String::new(),
                model: String::new(),
                custom_model: String::new(),
                thinking: "medium".to_string(),
                grounding: false,
            });

        if let Some(ref base) = input.base_url {
            entry.base_url = base.trim().to_string();
        }
        if let Some(ref m) = input.model {
            entry.model = m.trim().to_string();
        }
        if let Some(ref cm) = input.custom_model {
            entry.custom_model = cm.trim().to_string();
        }
        if let Some(ref th) = input.thinking {
            entry.thinking = th.trim().to_string();
        }
        if let Some(gr) = input.grounding {
            entry.grounding = gr;
        }
        if let Some(ref key) = input.api_key {
            // Only update if provided; empty string clears, non-empty sets
            entry.api_key = key.trim().to_string();
        }
    }

    // Apply batch updates if any
    for update in input.provider_updates {
        let p_entry = current
            .providers
            .entry(update.id.clone())
            .or_insert_with(|| ProviderSettings {
                api_key: String::new(),
                base_url: String::new(),
                model: String::new(),
                custom_model: String::new(),
                thinking: "medium".to_string(),
                grounding: false,
            });
        if let Some(ref base) = update.base_url {
            p_entry.base_url = base.trim().to_string();
        }
        if let Some(ref m) = update.model {
            p_entry.model = m.trim().to_string();
        }
        if let Some(ref cm) = update.custom_model {
            p_entry.custom_model = cm.trim().to_string();
        }
        if let Some(ref th) = update.thinking {
            p_entry.thinking = th.trim().to_string();
        }
        if let Some(gr) = update.grounding {
            p_entry.grounding = gr;
        }
        if let Some(ref key) = update.api_key {
            p_entry.api_key = key.trim().to_string();
        }

        // Validate this updated entry
        validate_save_llm_settings(
            &update.id,
            &p_entry.base_url,
            &p_entry.model,
            &p_entry.custom_model,
            &p_entry.thinking,
            p_entry.grounding,
        )?;
    }

    if let Some(ref gh) = input.github_token {
        current.github_token = gh.trim().to_string();
    }

    // Validate the target provider settings
    if let Some(entry) = current.providers.get(&target_provider) {
        validate_save_llm_settings(
            &target_provider,
            &entry.base_url,
            &entry.model,
            &entry.custom_model,
            &entry.thinking,
            entry.grounding,
        )?;
    }

    Ok(())
}
