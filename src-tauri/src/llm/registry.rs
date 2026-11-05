use crate::models::{ProviderInfo, ProviderSettings};
use std::collections::HashMap;

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
                "gemini-3.6-flash".to_string(),
                "gemini-3.5-flash".to_string(),
                "gemini-3.5-flash-lite".to_string(),
                "gemini-3.1-flash-lite".to_string(),
                "gemma-4-31b-it".to_string(),
                "gemma-4-26b-a4b-it".to_string(),
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
                "gemini-3.6-flash".to_string(),
                "gemini-3.5-flash".to_string(),
                "gemini-3.5-flash-lite".to_string(),
                "gemini-3.1-flash-lite".to_string(),
                "gemma-4-31b-it".to_string(),
                "gemma-4-26b-a4b-it".to_string(),
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
