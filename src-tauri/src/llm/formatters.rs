use super::registry::{
    PROVIDER_GEMINI_NATIVE, PROVIDER_GEMINI_OPENAI, PROVIDER_META, PROVIDER_OPENROUTER,
};

/// Normalizes model aliases to canonical IDs.
pub fn normalize_google_model_id(provider_id: &str, model: &str) -> String {
    let trimmed = model.trim();
    let lower = trimmed.to_ascii_lowercase();

    if provider_id == PROVIDER_GEMINI_NATIVE || provider_id == PROVIDER_GEMINI_OPENAI {
        match lower.as_str() {
            "gemma-4-26b" | "gemma 4 26b" | "gemma-4-26b-it" | "gemma-4-26b-a4b" => {
                "gemma-4-26b-a4b-it".to_string()
            }
            "gemma-4-31b" | "gemma 4 31b" => "gemma-4-31b-it".to_string(),
            "gemini-3.6" | "gemini 3.6" | "gemini 3.6 flash" => "gemini-3.6-flash".to_string(),
            "gemini-3.1" | "gemini 3.1" | "gemini 3.1 flash-lite" | "gemini-3.1-flash" => {
                "gemini-3.1-flash-lite".to_string()
            }
            _ => trimmed.to_string(),
        }
    } else if provider_id == PROVIDER_OPENROUTER {
        match lower.as_str() {
            "google/gemma-4-26b" | "google/gemma-4-26b-it" | "google/gemma-4-26b-a4b" => {
                "google/gemma-4-26b-a4b-it".to_string()
            }
            "google/gemma-4-31b" => "google/gemma-4-31b-it".to_string(),
            "google/gemini-3.6" | "google/gemini 3.6" => "google/gemini-3.6-flash".to_string(),
            "google/gemini-3.1" | "google/gemini 3.1" => {
                "google/gemini-3.1-flash-lite".to_string()
            }
            _ => trimmed.to_string(),
        }
    } else {
        trimmed.to_string()
    }
}

/// Mapped thinking configuration (§5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThinkingMapping {
    None,
    GeminiLevel(String),     // "LOW" | "MEDIUM" | "HIGH"
    GeminiBudget(u32),       // 1024 | 8192 | 24576
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

/// Converts provider error payloads and network failures into clear, actionable messages.
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
