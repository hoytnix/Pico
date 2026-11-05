use crate::llm::*;
use crate::models::{LlmAppSettings, ProviderUpdate, SaveLlmSettingsInput};

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
