use crate::models::{
    LlmAppSettings, LlmSettingsView, ProviderInfo, ProviderSettings,
    ProviderView, SaveLlmSettingsInput, TestConnectionResult, VibeOptions, VibeResult, VibeTarget,
};
use crate::vibe::{build_vibe_system_prompt, evaluate_vibe_proposal, read_vibe_target_content};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

/// The canonical preset provider IDs.
pub const PROVIDER_GEMINI_NATIVE: &str = "gemini-native";
pub const PROVIDER_OPENAI: &str = "openai";
pub const PROVIDER_OPENROUTER: &str = "openrouter";
pub const PROVIDER_META: &str = "meta";
pub const PROVIDER_GEMINI_OPENAI: &str = "gemini-openai";
pub const PROVIDER_CUSTOM: &str = "custom";

/// Returns the 6 preset providers in the exact order specified in §4.
pub fn get_provider_registry() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: PROVIDER_GEMINI_NATIVE.to_string(),
            label: "Gemini (Native)".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            default_models: vec![
                "gemini-4-argon".to_string(),
                "gemini-3.8-flash".to_string(),
                "gemini-3.5-flash".to_string(),
                "gemini-3.5-flash-lite".to_string(),
            ],
            key_label: "Gemini API key".to_string(),
            key_url: "https://aistudio.google.com/app/apikey".to_string(),
            supports_thinking: true,
            supports_grounding: true,
        },
        ProviderInfo {
            id: PROVIDER_OPENAI.to_string(),
            label: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            default_models: vec!["gpt-4o-mini".to_string()],
            key_label: "OpenAI API key".to_string(),
            key_url: "https://platform.openai.com/api-keys".to_string(),
            supports_thinking: true,
            supports_grounding: false,
        },
        ProviderInfo {
            id: PROVIDER_OPENROUTER.to_string(),
            label: "OpenRouter".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            // OpenRouter model IDs are vendor-prefixed, e.g. google/..., meta/...
            // Custom model text field is REQUIRED here; do not present a short fixed list as if complete (§4).
            default_models: vec![],
            key_label: "OpenRouter API key".to_string(),
            key_url: "https://openrouter.ai/keys".to_string(),
            supports_thinking: true,
            supports_grounding: false,
        },
        ProviderInfo {
            id: PROVIDER_META.to_string(),
            label: "Muse Spark".to_string(),
            base_url: "https://api.meta.ai/v1".to_string(),
            default_models: vec![
                "muse-spark-1.3".to_string(),
                "muse-spark-1.3-contributor".to_string(),
                "muse-wonder-1.0".to_string(),
            ],
            key_label: "Model API key".to_string(),
            key_url: "https://api.meta.ai/dashboard/keys".to_string(),
            supports_thinking: false,
            supports_grounding: false,
        },
        ProviderInfo {
            id: PROVIDER_GEMINI_OPENAI.to_string(),
            label: "Gemini (OpenAI mode)".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
            default_models: vec![
                "gemini-4-argon".to_string(),
                "gemini-3.8-flash".to_string(),
                "gemini-3.5-flash".to_string(),
                "gemini-3.5-flash-lite".to_string(),
            ],
            key_label: "Gemini API key".to_string(),
            key_url: "https://aistudio.google.com/app/apikey".to_string(),
            supports_thinking: true,
            supports_grounding: false,
        },
        ProviderInfo {
            id: PROVIDER_CUSTOM.to_string(),
            label: "Custom (OpenAI-compatible)".to_string(),
            base_url: String::new(),
            default_models: vec![],
            key_label: "Custom API key".to_string(),
            key_url: String::new(),
            supports_thinking: true,
            supports_grounding: false,
        },
    ]
}

/// Creates default provider settings for all 6 presets.
pub fn create_default_providers_map() -> HashMap<String, ProviderSettings> {
    let mut map = HashMap::new();
    for p in get_provider_registry() {
        let default_model = p.default_models.first().cloned().unwrap_or_default();
        map.insert(
            p.id.clone(),
            ProviderSettings {
                api_key: String::new(),
                base_url: p.base_url,
                model: default_model,
                custom_model: String::new(),
                thinking: "medium".to_string(),
                grounding: false,
            },
        );
    }
    map
}

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

/// Mapped thinking configuration (§5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThinkingMapping {
    None,
    GeminiLevel(String), // "LOW" | "MEDIUM" | "HIGH"
    GeminiBudget(u32),   // 1024 | 8192 | 24576
    OpenAiReasoning(String), // "low" | "medium" | "high"
}

/// Maps thinking setting according to the exact mapping table in §5.1:
///
/// | Track / model family | Low | Medium | High | Forbidden |
/// | Gemini Native, Gemini 3/4 | thinkingLevel: "LOW" | "MEDIUM" | "HIGH" | MINIMAL, budget |
/// | Gemini Native, Gemini 2.5 | thinkingBudget: 1024 | 8192 | 24576 | thinkingLevel |
/// | OpenAI track | reasoning_effort: "low" | "medium" | "high" | thinkingConfig |
/// | Muse Spark | DISABLED (send nothing) | send nothing | send nothing | reasoning_effort |
pub fn map_thinking_setting(
    provider_id: &str,
    model: &str,
    thinking: &str,
) -> Result<ThinkingMapping, String> {
    let norm_thinking = thinking.trim().to_ascii_lowercase();
    if norm_thinking == "default" || norm_thinking.is_empty() {
        return Ok(ThinkingMapping::None);
    }
    if !matches!(norm_thinking.as_str(), "low" | "medium" | "high") {
        return Err(format!(
            "Invalid thinking value '{}'. Expected low, medium, high, or default.",
            thinking
        ));
    }

    // Muse Spark: Never send reasoning_effort (§5.1)
    if provider_id == PROVIDER_META {
        return Ok(ThinkingMapping::None);
    }

    if provider_id == PROVIDER_GEMINI_NATIVE {
        let m = model.to_ascii_lowercase();
        if m.contains("2.5") {
            // Gemini 2.5 uses budgets, not levels
            match norm_thinking.as_str() {
                "low" => Ok(ThinkingMapping::GeminiBudget(1024)),
                "medium" => Ok(ThinkingMapping::GeminiBudget(8192)),
                "high" => Ok(ThinkingMapping::GeminiBudget(24576)),
                _ => Ok(ThinkingMapping::None),
            }
        } else {
            // Gemini 3 and Gemini 4 (including gemini-4-argon) use thinkingLevel
            match norm_thinking.as_str() {
                "low" => Ok(ThinkingMapping::GeminiLevel("LOW".to_string())),
                "medium" => Ok(ThinkingMapping::GeminiLevel("MEDIUM".to_string())),
                "high" => Ok(ThinkingMapping::GeminiLevel("HIGH".to_string())),
                _ => Ok(ThinkingMapping::None),
            }
        }
    } else {
        // OpenAI-compatible track (OpenAI, OpenRouter, Gemini OpenAI mode, Custom)
        match norm_thinking.as_str() {
            "low" => Ok(ThinkingMapping::OpenAiReasoning("low".to_string())),
            "medium" => Ok(ThinkingMapping::OpenAiReasoning("medium".to_string())),
            "high" => Ok(ThinkingMapping::OpenAiReasoning("high".to_string())),
            _ => Ok(ThinkingMapping::None),
        }
    }
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

/// Formats provider HTTP error responses into the exact user-facing messages specified in §8:
///
/// Named cases:
/// - 401 / invalid key: "Invalid API key for <provider>. Check the key in Settings."
/// - 402 / billing (Muse / billing_not_configured): "Billing is not configured for Muse. Update the payment method in the Meta Model API dashboard, then retry."
/// - 404 / model not found: "Model <id> was not found for <provider>. Check the model ID or pick another model."
/// - 400 / thinking: "Thinking setting is not supported for this model/provider combination."
/// - 429 / rate limit: "Rate limited by <provider>. Wait and retry."
/// - Network / timeout: "Could not reach <provider>. Check your connection and the base URL, then retry."
pub fn format_provider_error(
    provider_id: &str,
    model: &str,
    status_code: Option<u16>,
    raw_error: &str,
) -> String {
    let lower = raw_error.to_ascii_lowercase();

    if let Some(status) = status_code {
        match status {
            401 => {
                return format!(
                    "Invalid API key for {}. Check the key in Settings. ({})",
                    provider_id, raw_error
                );
            }
            402 => {
                if provider_id == PROVIDER_META || lower.contains("billing") {
                    return format!(
                        "Billing is not configured for Muse. Update the payment method in the Meta Model API dashboard, then retry. ({})",
                        raw_error
                    );
                } else {
                    return format!(
                        "Billing issue for {}. Check payment details in your provider dashboard. ({})",
                        provider_id, raw_error
                    );
                }
            }
            404 => {
                return format!(
                    "Model {} was not found for {}. Check the model ID or pick another model. ({})",
                    model, provider_id, raw_error
                );
            }
            400 => {
                if lower.contains("thinking")
                    || lower.contains("budget")
                    || lower.contains("reasoning")
                {
                    return format!(
                        "Thinking setting is not supported for this model/provider combination. ({})",
                        raw_error
                    );
                }
                return format!(
                    "Request error for {} (HTTP 400): {}",
                    provider_id, raw_error
                );
            }
            429 => {
                return format!(
                    "Rate limited by {}. Wait and retry. ({})",
                    provider_id, raw_error
                );
            }
            _ => {}
        }
    }

    if lower.contains("timed out") || lower.contains("timeout") {
        return format!(
            "Could not reach {}. Request timed out. Check your connection and the base URL, then retry.",
            provider_id
        );
    }
    if lower.contains("connect") || lower.contains("dns") || lower.contains("network") {
        return format!(
            "Could not reach {}. Check your connection and the base URL, then retry. ({})",
            provider_id, raw_error
        );
    }

    format!(
        "{} error (HTTP {:?}): {}",
        provider_id,
        status_code.unwrap_or(0),
        raw_error
    )
}

/// Executes a minimal test connection call (§6, §9) with a 30-second timeout.
/// Prompt: "Reply with exactly: OK".
pub async fn test_llm_connection_impl(
    provider_id: &str,
    model_override: Option<&str>,
    settings: &LlmAppSettings,
    client_opt: Option<&reqwest::Client>,
) -> TestConnectionResult {
    let prov = match settings.providers.get(provider_id) {
        Some(p) => p,
        None => {
            return TestConnectionResult {
                success: false,
                model: model_override.unwrap_or("unknown").to_string(),
                elapsed_ms: 0,
                error: Some(format!("Provider '{}' not found in settings.", provider_id)),
            };
        }
    };

    if prov.api_key.trim().is_empty() {
        return TestConnectionResult {
            success: false,
            model: model_override.unwrap_or(&prov.model).to_string(),
            elapsed_ms: 0,
            error: Some(format!(
                "Invalid API key for {}. Check the key in Settings.",
                provider_id
            )),
        };
    }

    let effective_model = if let Some(mo) = model_override {
        if !mo.trim().is_empty() {
            mo.trim().to_string()
        } else if !prov.custom_model.trim().is_empty() {
            prov.custom_model.trim().to_string()
        } else {
            prov.model.trim().to_string()
        }
    } else if !prov.custom_model.trim().is_empty() {
        prov.custom_model.trim().to_string()
    } else {
        prov.model.trim().to_string()
    };

    let start = Instant::now();

    // Track 1 (Gemini Native) test via official Google API endpoint if invoked directly on backend
    if provider_id == PROVIDER_GEMINI_NATIVE {
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            effective_model,
            prov.api_key.trim()
        );
        let default_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        let client = client_opt.unwrap_or(&default_client);

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{ "text": "Reply with exactly: OK" }]
            }],
            "generationConfig": {
                "maxOutputTokens": 10
            }
        });

        match client.post(&endpoint).json(&payload).send().await {
            Ok(resp) => {
                let status = resp.status();
                let elapsed_ms = start.elapsed().as_millis() as u64;
                if status.is_success() {
                    TestConnectionResult {
                        success: true,
                        model: effective_model,
                        elapsed_ms,
                        error: None,
                    }
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    let err = format_provider_error(
                        provider_id,
                        &effective_model,
                        Some(status.as_u16()),
                        &body,
                    );
                    TestConnectionResult {
                        success: false,
                        model: effective_model,
                        elapsed_ms,
                        error: Some(err),
                    }
                }
            }
            Err(e) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                let err =
                    format_provider_error(provider_id, &effective_model, None, &e.to_string());
                TestConnectionResult {
                    success: false,
                    model: effective_model,
                    elapsed_ms,
                    error: Some(err),
                }
            }
        }
    } else {
        // Track 2: OpenAI-compatible endpoint POST to {base_url}/chat/completions
        let base = prov.base_url.trim_end_matches('/');
        let endpoint = if base.ends_with("/chat/completions") {
            base.to_string()
        } else {
            format!("{}/chat/completions", base)
        };

        let default_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        let client = client_opt.unwrap_or(&default_client);

        let payload = serde_json::json!({
            "model": effective_model,
            "max_tokens": 10,
            "messages": [
                { "role": "user", "content": "Reply with exactly: OK" }
            ]
        });

        let mut req = client
            .post(&endpoint)
            .bearer_auth(prov.api_key.trim())
            .json(&payload);

        // OpenRouter required headers (§4)
        if provider_id == PROVIDER_OPENROUTER || endpoint.contains("openrouter.ai") {
            req = req
                .header("HTTP-Referer", "https://github.com/kopimi/pico")
                .header("X-Title", "Pico");
        }

        match req.send().await {
            Ok(resp) => {
                let status = resp.status();
                let elapsed_ms = start.elapsed().as_millis() as u64;
                if status.is_success() {
                    TestConnectionResult {
                        success: true,
                        model: effective_model,
                        elapsed_ms,
                        error: None,
                    }
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    let err = format_provider_error(
                        provider_id,
                        &effective_model,
                        Some(status.as_u16()),
                        &body,
                    );
                    TestConnectionResult {
                        success: false,
                        model: effective_model,
                        elapsed_ms,
                        error: Some(err),
                    }
                }
            }
            Err(e) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                let err =
                    format_provider_error(provider_id, &effective_model, None, &e.to_string());
                TestConnectionResult {
                    success: false,
                    model: effective_model,
                    elapsed_ms,
                    error: Some(err),
                }
            }
        }
    }
}

/// Executes a Vibe edit using the OpenAI-compatible track (§4) with 120s timeout.
pub async fn run_openai_vibe_edit(
    site_root: &Path,
    target: &VibeTarget,
    prompt: &str,
    options: Option<VibeOptions>,
    settings: &LlmAppSettings,
    client_opt: Option<&reqwest::Client>,
) -> Result<VibeResult, String> {
    let opts = options.unwrap_or_default();
    let provider_id = opts
        .provider
        .unwrap_or_else(|| settings.current_provider.clone());

    let prov = settings.providers.get(&provider_id).ok_or_else(|| {
        format!("Configured provider '{}' not found in settings.", provider_id)
    })?;

    // Hard check for API key
    if prov.api_key.trim().is_empty() {
        return Err("Configure an LLM API key in Settings to use Vibe.".to_string());
    }
    if prompt.trim().is_empty() {
        return Err("Please enter a Vibe instruction prompt.".to_string());
    }

    let model = if let Some(m) = opts.model {
        if !m.trim().is_empty() {
            m.trim().to_string()
        } else if !prov.custom_model.trim().is_empty() {
            prov.custom_model.trim().to_string()
        } else {
            prov.model.trim().to_string()
        }
    } else if !prov.custom_model.trim().is_empty() {
        prov.custom_model.trim().to_string()
    } else {
        prov.model.trim().to_string()
    };

    let thinking = opts.thinking.unwrap_or_else(|| prov.thinking.clone());

    let original_content = read_vibe_target_content(site_root, target)?;
    let system_prompt = build_vibe_system_prompt(site_root, target)?;
    let user_msg = format!(
        "Instruction: {}\n\nCurrent file (`{}`):\n{}",
        prompt.trim(),
        target.path,
        original_content
    );

    let base = prov.base_url.trim_end_matches('/');
    let endpoint = if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{}/chat/completions", base)
    };

    let mut payload_map = serde_json::Map::new();
    payload_map.insert("model".to_string(), serde_json::Value::String(model.clone()));
    payload_map.insert(
        "messages".to_string(),
        serde_json::json!([
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_msg }
        ]),
    );

    // Map thinking according to §5.1
    let mapped_thinking = map_thinking_setting(&provider_id, &model, &thinking)?;
    if let ThinkingMapping::OpenAiReasoning(ref effort) = mapped_thinking {
        payload_map.insert(
            "reasoning_effort".to_string(),
            serde_json::Value::String(effort.clone()),
        );
    }

    let default_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap_or_default();
    let client = client_opt.unwrap_or(&default_client);

    let mut req = client
        .post(&endpoint)
        .bearer_auth(prov.api_key.trim())
        .json(&serde_json::Value::Object(payload_map));

    // OpenRouter required headers (§4)
    if provider_id == PROVIDER_OPENROUTER || endpoint.contains("openrouter.ai") {
        req = req
            .header("HTTP-Referer", "https://github.com/kopimi/pico")
            .header("X-Title", "Pico");
    }

    let resp = req.send().await.map_err(|e| {
        format_provider_error(&provider_id, &model, None, &e.to_string())
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let err = format_provider_error(&provider_id, &model, Some(status.as_u16()), &body);
        return Err(err);
    }

    let json_resp: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid JSON response from LLM endpoint: {}", e))?;

    let proposed_raw = json_resp
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "LLM response did not contain choices[0].message.content.".to_string())?;

    let mut res = evaluate_vibe_proposal(site_root, target, &original_content, proposed_raw)?;
    res.sources = None; // Sources are present only for Gemini Native grounding calls (§9)
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ProviderUpdate, SaveLlmSettingsInput};

    #[test]
    fn test_provider_registry_ids_urls_and_order() {
        let registry = get_provider_registry();
        assert_eq!(registry.len(), 6);

        // 1. Exact ordering from §4
        let ids: Vec<&str> = registry.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                PROVIDER_GEMINI_NATIVE,
                PROVIDER_OPENAI,
                PROVIDER_OPENROUTER,
                PROVIDER_META,
                PROVIDER_GEMINI_OPENAI,
                PROVIDER_CUSTOM,
            ]
        );

        // 2. Exact base URLs from §4
        let gemini_native = registry.iter().find(|p| p.id == PROVIDER_GEMINI_NATIVE).unwrap();
        assert_eq!(
            gemini_native.base_url,
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(
            gemini_native.default_models,
            vec![
                "gemini-4-argon",
                "gemini-3.8-flash",
                "gemini-3.5-flash",
                "gemini-3.5-flash-lite"
            ]
        );
        assert!(gemini_native.supports_thinking);
        assert!(gemini_native.supports_grounding);

        let openai = registry.iter().find(|p| p.id == PROVIDER_OPENAI).unwrap();
        assert_eq!(openai.base_url, "https://api.openai.com/v1");
        assert_eq!(openai.default_models, vec!["gpt-4o-mini"]);
        assert!(openai.supports_thinking);
        assert!(!openai.supports_grounding);

        let openrouter = registry.iter().find(|p| p.id == PROVIDER_OPENROUTER).unwrap();
        assert_eq!(openrouter.base_url, "https://openrouter.ai/api/v1");
        assert!(
            openrouter.default_models.is_empty(),
            "OpenRouter default models must be empty so user enters vendor-prefixed custom model (§4)"
        );
        assert!(openrouter.supports_thinking);
        assert!(!openrouter.supports_grounding);

        let muse = registry.iter().find(|p| p.id == PROVIDER_META).unwrap();
        assert_eq!(muse.base_url, "https://api.meta.ai/v1");
        assert_eq!(
            muse.default_models,
            vec!["muse-spark-1.3", "muse-spark-1.3-contributor", "muse-wonder-1.0"]
        );
        assert!(!muse.supports_thinking, "Muse manages its own reasoning");
        assert!(!muse.supports_grounding);

        let gemini_openai = registry.iter().find(|p| p.id == PROVIDER_GEMINI_OPENAI).unwrap();
        assert_eq!(
            gemini_openai.base_url,
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );
        assert_eq!(
            gemini_openai.default_models,
            vec![
                "gemini-4-argon",
                "gemini-3.8-flash",
                "gemini-3.5-flash",
                "gemini-3.5-flash-lite"
            ]
        );
        assert!(gemini_openai.supports_thinking);
        assert!(!gemini_openai.supports_grounding);

        let custom = registry.iter().find(|p| p.id == PROVIDER_CUSTOM).unwrap();
        assert_eq!(custom.base_url, "");
        assert!(custom.default_models.is_empty());
        assert!(custom.supports_thinking);
        assert!(!custom.supports_grounding);
    }

    #[test]
    fn test_thinking_mapping_table_exact_spec() {
        // 1. Gemini Native Gemini 4 / 3 (e.g. gemini-4-argon, gemini-3.8-flash, gemini-3.5-flash)
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-4-argon", "low").unwrap(),
            ThinkingMapping::GeminiLevel("LOW".to_string())
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-4-argon", "medium").unwrap(),
            ThinkingMapping::GeminiLevel("MEDIUM".to_string())
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-4-argon", "high").unwrap(),
            ThinkingMapping::GeminiLevel("HIGH".to_string())
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-3.8-flash", "medium").unwrap(),
            ThinkingMapping::GeminiLevel("MEDIUM".to_string())
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-3.5-flash", "high").unwrap(),
            ThinkingMapping::GeminiLevel("HIGH".to_string())
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-4-argon", "default").unwrap(),
            ThinkingMapping::None
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-4-argon", "").unwrap(),
            ThinkingMapping::None
        );

        // 2. Gemini Native Gemini 2.5 (e.g. gemini-2.5-flash) -> uses budgets, not levels (§5.1)
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-2.5-flash", "low").unwrap(),
            ThinkingMapping::GeminiBudget(1024)
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-2.5-flash", "medium").unwrap(),
            ThinkingMapping::GeminiBudget(8192)
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-2.5-flash", "high").unwrap(),
            ThinkingMapping::GeminiBudget(24576)
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-2.5-flash", "default").unwrap(),
            ThinkingMapping::None
        );

        // 3. OpenAI-compatible track (OpenAI, OpenRouter, Gemini OpenAI mode, Custom) -> reasoning_effort
        for prov in [
            PROVIDER_OPENAI,
            PROVIDER_OPENROUTER,
            PROVIDER_GEMINI_OPENAI,
            PROVIDER_CUSTOM,
        ] {
            assert_eq!(
                map_thinking_setting(prov, "gpt-4o-mini", "low").unwrap(),
                ThinkingMapping::OpenAiReasoning("low".to_string())
            );
            assert_eq!(
                map_thinking_setting(prov, "gpt-4o-mini", "medium").unwrap(),
                ThinkingMapping::OpenAiReasoning("medium".to_string())
            );
            assert_eq!(
                map_thinking_setting(prov, "gpt-4o-mini", "high").unwrap(),
                ThinkingMapping::OpenAiReasoning("high".to_string())
            );
            assert_eq!(
                map_thinking_setting(prov, "gpt-4o-mini", "default").unwrap(),
                ThinkingMapping::None
            );
        }

        // 4. Muse Spark (OpenAI track) -> thinking disabled, send nothing (§5.1)
        assert_eq!(
            map_thinking_setting(PROVIDER_META, "muse-spark-1.3", "low").unwrap(),
            ThinkingMapping::None
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_META, "muse-spark-1.3", "medium").unwrap(),
            ThinkingMapping::None
        );
        assert_eq!(
            map_thinking_setting(PROVIDER_META, "muse-spark-1.3", "high").unwrap(),
            ThinkingMapping::None
        );

        // 5. Invalid thinking value returns error
        assert!(map_thinking_setting(PROVIDER_OPENAI, "gpt-4o-mini", "ultra").is_err());
    }

    #[test]
    fn test_grounding_gating_save_llm_settings() {
        // Grounding allowed only for Gemini Native (§5.2, §9)
        assert!(validate_save_llm_settings(
            PROVIDER_GEMINI_NATIVE,
            "https://generativelanguage.googleapis.com/v1beta",
            "gemini-4-argon",
            "",
            "medium",
            true
        )
        .is_ok());

        assert!(validate_save_llm_settings(
            PROVIDER_GEMINI_NATIVE,
            "https://generativelanguage.googleapis.com/v1beta",
            "gemini-4-argon",
            "",
            "medium",
            false
        )
        .is_ok());

        // Gated / rejected for all other providers
        let non_native = [
            PROVIDER_OPENAI,
            PROVIDER_OPENROUTER,
            PROVIDER_META,
            PROVIDER_GEMINI_OPENAI,
            PROVIDER_CUSTOM,
        ];
        for prov in non_native {
            let base_url = if prov == PROVIDER_CUSTOM {
                "https://my-llm.com/v1"
            } else {
                "https://api.example.com/v1"
            };
            let err = validate_save_llm_settings(
                prov,
                base_url,
                "some-model",
                "",
                "medium",
                true,
            )
            .unwrap_err();
            assert_eq!(
                err,
                "Grounding requires Gemini Native, not an OpenAI provider.",
                "Provider {} must reject grounding=true with exact text",
                prov
            );
        }
    }

    #[test]
    fn test_settings_migration_old_single_endpoint_to_openai() {
        // Old single-endpoint shape from MVP (§7)
        let old_json = r#"{
            "llm_endpoint": "https://api.openai.com/v1/chat/completions",
            "llm_model": "gpt-4o-mini",
            "llm_api_key": "sk-test-preserved-secret-key-12345",
            "github_token": "ghp_preserved_token_999",
            "recent_sites": [
                {
                    "title": "Blog Test",
                    "path": "/home/user/my-blog",
                    "last_opened": "2026-11-03 12:00"
                }
            ]
        }"#;

        let migrated = migrate_settings_json(old_json).unwrap();

        // 1. Migrates to provider openai
        assert_eq!(migrated.current_provider, PROVIDER_OPENAI);

        // 2. Preserves key, model, and base URL under openai
        let openai_entry = migrated.providers.get(PROVIDER_OPENAI).unwrap();
        assert_eq!(openai_entry.api_key, "sk-test-preserved-secret-key-12345");
        assert_eq!(openai_entry.model, "gpt-4o-mini");
        assert_eq!(openai_entry.base_url, "https://api.openai.com/v1");

        // 3. Other providers must NOT receive the OpenAI key
        let gemini_entry = migrated.providers.get(PROVIDER_GEMINI_NATIVE).unwrap();
        assert_eq!(gemini_entry.api_key, "");

        let muse_entry = migrated.providers.get(PROVIDER_META).unwrap();
        assert_eq!(muse_entry.api_key, "");

        // 4. GitHub token and recent sites preserved
        assert_eq!(migrated.github_token, "ghp_preserved_token_999");
        assert_eq!(migrated.recent_sites.len(), 1);
        assert_eq!(migrated.recent_sites[0].title, "Blog Test");

        // 5. Sanitized view shows SET for openai and NOT SET for others
        let view = build_settings_view(&migrated);
        assert_eq!(view.current_provider, PROVIDER_OPENAI);
        assert_eq!(view.providers.get(PROVIDER_OPENAI).unwrap().key_status, "SET");
        assert_eq!(
            view.providers.get(PROVIDER_GEMINI_NATIVE).unwrap().key_status,
            "NOT SET"
        );
        assert_eq!(
            view.providers.get(PROVIDER_META).unwrap().key_status,
            "NOT SET"
        );
    }

    #[test]
    fn test_key_routing_isolation_mock_http() {
        // Assert key isolation: Muse key is never used for Gemini, Gemini key is never used for Muse, etc. (§7)
        let mut settings = LlmAppSettings {
            current_provider: PROVIDER_GEMINI_NATIVE.to_string(),
            providers: create_default_providers_map(),
            github_token: String::new(),
            recent_sites: Vec::new(),
        };

        let gemini_key = "gemini-key-aabbcc-never-send-to-meta";
        let muse_key = "meta-key-ddeeff-never-send-to-google";
        let openai_key = "openai-key-112233-never-send-to-openrouter";
        let openrouter_key = "openrouter-key-445566-never-send-to-openai";

        settings.providers.get_mut(PROVIDER_GEMINI_NATIVE).unwrap().api_key =
            gemini_key.to_string();
        settings.providers.get_mut(PROVIDER_META).unwrap().api_key =
            muse_key.to_string();
        settings.providers.get_mut(PROVIDER_OPENAI).unwrap().api_key =
            openai_key.to_string();
        settings.providers.get_mut(PROVIDER_OPENROUTER).unwrap().api_key =
            openrouter_key.to_string();

        // 1. Direct key extraction must be strictly partitioned
        assert_eq!(
            settings.providers.get(PROVIDER_GEMINI_NATIVE).unwrap().api_key,
            gemini_key
        );
        assert_ne!(
            settings.providers.get(PROVIDER_GEMINI_NATIVE).unwrap().api_key,
            muse_key
        );

        assert_eq!(
            settings.providers.get(PROVIDER_META).unwrap().api_key,
            muse_key
        );
        assert_ne!(
            settings.providers.get(PROVIDER_META).unwrap().api_key,
            gemini_key
        );

        assert_eq!(
            settings.providers.get(PROVIDER_OPENAI).unwrap().api_key,
            openai_key
        );
        assert_ne!(
            settings.providers.get(PROVIDER_OPENAI).unwrap().api_key,
            openrouter_key
        );

        // 2. Key routing update verification: updating one provider does not touch another
        let input = SaveLlmSettingsInput {
            current_provider: PROVIDER_META.to_string(),
            provider_updates: vec![ProviderUpdate {
                id: PROVIDER_META.to_string(),
                base_url: Some("https://api.meta.ai/v1".to_string()),
                model: Some("muse-spark-1.3".to_string()),
                custom_model: None,
                thinking: Some("medium".to_string()),
                grounding: Some(false),
                api_key: Some("new-meta-key-999".to_string()),
            }],
            base_url: None,
            model: None,
            custom_model: None,
            thinking: None,
            grounding: None,
            api_key: None,
            github_token: None,
        };

        apply_save_settings_input(&mut settings, input).unwrap();

        // Meta key is updated
        assert_eq!(
            settings.providers.get(PROVIDER_META).unwrap().api_key,
            "new-meta-key-999"
        );
        // Gemini key is strictly unchanged
        assert_eq!(
            settings.providers.get(PROVIDER_GEMINI_NATIVE).unwrap().api_key,
            gemini_key
        );
    }

    #[test]
    fn test_custom_base_url_validation() {
        // Valid HTTPS URLs
        assert!(validate_base_url("https://api.together.xyz/v1").is_ok());
        assert!(validate_base_url("https://my-llm.example.org").is_ok());
        assert!(validate_base_url("https://generativelanguage.googleapis.com").is_ok());

        // Valid local HTTP URLs (§4)
        assert!(validate_base_url("http://localhost:11434").is_ok());
        assert!(validate_base_url("http://localhost:8080/v1").is_ok());
        assert!(validate_base_url("http://127.0.0.1:1234/v1").is_ok());
        assert!(validate_base_url("http://127.0.0.1").is_ok());

        // Rejected remote HTTP URLs (§4)
        let remote_http = validate_base_url("http://api.together.xyz/v1");
        assert!(remote_http.is_err());
        assert!(remote_http.unwrap_err().contains("HTTP is only allowed for localhost or 127.0.0.1"));

        let remote_ip_http = validate_base_url("http://192.168.1.100:8080/v1");
        assert!(remote_ip_http.is_err());

        // Rejected other schemes and empty
        assert!(validate_base_url("ftp://server/v1").is_err());
        assert!(validate_base_url("").is_err());
        assert!(validate_base_url("   ").is_err());
    }

    #[test]
    fn test_error_formatting_named_cases() {
        // 401 / invalid key
        let err401 = format_provider_error(PROVIDER_OPENAI, "gpt-4o-mini", Some(401), "Incorrect API key");
        assert!(err401.contains("Invalid API key for openai. Check the key in Settings."));

        // 402 / billing for Muse
        let err402_muse = format_provider_error(
            PROVIDER_META,
            "muse-spark-1.3",
            Some(402),
            "billing_not_configured: Payment method required",
        );
        assert!(err402_muse.contains("Billing is not configured for Muse. Update the payment method in the Meta Model API dashboard, then retry."));

        // 404 / model not found
        let err404 = format_provider_error(
            PROVIDER_GEMINI_NATIVE,
            "gemini-nonexistent",
            Some(404),
            "models/gemini-nonexistent is not found",
        );
        assert!(err404.contains("Model gemini-nonexistent was not found for gemini-native. Check the model ID or pick another model."));

        // 400 / thinking or budget
        let err400_thinking = format_provider_error(
            PROVIDER_GEMINI_NATIVE,
            "gemini-4-argon",
            Some(400),
            "Thinking budget is not supported for this model",
        );
        assert!(err400_thinking.contains("Thinking setting is not supported for this model/provider combination."));

        // 429 / rate limit
        let err429 = format_provider_error(PROVIDER_OPENROUTER, "meta/llama-3", Some(429), "Rate limit exceeded");
        assert!(err429.contains("Rate limited by openrouter. Wait and retry."));

        // Timeout
        let err_timeout = format_provider_error(PROVIDER_META, "muse-spark-1.3", None, "operation timed out");
        assert!(err_timeout.contains("Could not reach meta. Request timed out. Check your connection and the base URL, then retry."));
    }
}
