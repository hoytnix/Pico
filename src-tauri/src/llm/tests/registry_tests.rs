use crate::llm::*;

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
            "gemini-3.6-flash",
            "gemini-3.5-flash",
            "gemini-3.5-flash-lite",
            "gemini-3.1-flash-lite",
            "gemma-4-31b-it",
            "gemma-4-26b-a4b-it",
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
            "gemini-3.6-flash",
            "gemini-3.5-flash",
            "gemini-3.5-flash-lite",
            "gemini-3.1-flash-lite",
            "gemma-4-31b-it",
            "gemma-4-26b-a4b-it",
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
    // 1. Gemini Native Gemini 4 / 3 & Gemma 4
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
        map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-3.6-flash", "high").unwrap(),
        ThinkingMapping::GeminiLevel("HIGH".to_string())
    );
    assert_eq!(
        map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-3.5-flash", "high").unwrap(),
        ThinkingMapping::GeminiLevel("HIGH".to_string())
    );
    assert_eq!(
        map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemini-3.1-flash-lite", "low").unwrap(),
        ThinkingMapping::GeminiLevel("LOW".to_string())
    );
    assert_eq!(
        map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemma-4-31b-it", "medium").unwrap(),
        ThinkingMapping::GeminiLevel("MEDIUM".to_string())
    );
    assert_eq!(
        map_thinking_setting(PROVIDER_GEMINI_NATIVE, "gemma-4-26b-a4b-it", "high").unwrap(),
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

#[test]
fn test_normalize_google_model_id() {
    // Native and OpenAI mode aliases
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemma-4-26b"),
        "gemma-4-26b-a4b-it"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemma 4 26b"),
        "gemma-4-26b-a4b-it"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemma-4-26b-it"),
        "gemma-4-26b-a4b-it"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemma-4-31b"),
        "gemma-4-31b-it"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemma 4 31b"),
        "gemma-4-31b-it"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_OPENAI, "gemini-3.6"),
        "gemini-3.6-flash"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_OPENAI, "gemini 3.6 flash"),
        "gemini-3.6-flash"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemini-3.1"),
        "gemini-3.1-flash-lite"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemini 3.1 flash-lite"),
        "gemini-3.1-flash-lite"
    );
    // Already canonical remains untouched
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemma-4-26b-a4b-it"),
        "gemma-4-26b-a4b-it"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_GEMINI_NATIVE, "gemini-3.6-flash"),
        "gemini-3.6-flash"
    );

    // OpenRouter prefixed aliases
    assert_eq!(
        normalize_google_model_id(PROVIDER_OPENROUTER, "google/gemma-4-26b"),
        "google/gemma-4-26b-a4b-it"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_OPENROUTER, "google/gemma-4-31b"),
        "google/gemma-4-31b-it"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_OPENROUTER, "google/gemini-3.6"),
        "google/gemini-3.6-flash"
    );
    assert_eq!(
        normalize_google_model_id(PROVIDER_OPENROUTER, "google/gemini-3.1"),
        "google/gemini-3.1-flash-lite"
    );

    // Non-Google untouched
    assert_eq!(
        normalize_google_model_id(PROVIDER_OPENAI, "gpt-4o-mini"),
        "gpt-4o-mini"
    );
}
