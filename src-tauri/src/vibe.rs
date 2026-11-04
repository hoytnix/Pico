use crate::builder::{load_all_templates, load_models};
use crate::models::{PersistedAppSettings, VibeResult, VibeTarget};
use crate::validation::{
    BUILTIN_ITEM_FIELDS, SITE_CONFIG_FIELDS, strip_frontmatter_from_body,
    validate_template_with_models,
};
use similar::TextDiff;
use std::fs;
use std::path::Path;

/// Computes a unified diff between `original` and `proposed` content.
pub fn compute_unified_diff(path_label: &str, original: &str, proposed: &str) -> String {
    let diff = TextDiff::from_lines(original, proposed);
    diff.unified_diff()
        .context_radius(3)
        .header(&format!("a/{}", path_label), &format!("b/{}", path_label))
        .to_string()
}

/// Strips accidental markdown code fences (` ```jinja ... ``` `) if an LLM wraps its response.
pub fn strip_llm_code_fence(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("```") {
        if let Some(first_newline) = trimmed.find('\n') {
            let after_first = &trimmed[first_newline + 1..];
            if let Some(last_fence) = after_first.rfind("```") {
                return after_first[..last_fence].trim().to_string() + "\n";
            }
        }
    }
    trimmed.to_string() + "\n"
}

/// Builds the system prompt with model & guardrail constraints for the given `VibeTarget`.
pub fn build_vibe_system_prompt(
    site_root: &Path,
    target: &VibeTarget,
) -> Result<String, String> {
    let models = load_models(site_root).map_err(|e| e.to_string())?;

    match target.kind.as_str() {
        "content" => {
            Ok("You are Pico Vibe Assistant editing a Markdown post body.\n\
Return ONLY the updated Markdown body content (do NOT include YAML frontmatter `---` blocks, and do NOT wrap in outer code fences unless part of the post).".to_string())
        }
        "template" => {
            let mut vars_desc = Vec::new();
            vars_desc.push(format!("site.*: {}", SITE_CONFIG_FIELDS.join(", ")));
            vars_desc.push(format!(
                "post built-in fields: {}",
                BUILTIN_ITEM_FIELDS.join(", ")
            ));
            for m in &models {
                let fnames: Vec<&str> = m.fields.iter().map(|f| f.name.as_str()).collect();
                vars_desc.push(format!(
                    "Model '{}' ({}): post.{}",
                    m.label,
                    m.name,
                    fnames.join(", post.")
                ));
            }
            Ok(format!(
                "You are Pico Vibe Assistant editing a MiniJinja template (`{}`).\n\
STRICT GUARDRAILS:\n\
1. You may ONLY use variables defined in the site config and collection models:\n   {}\n\
2. Never use `varargs` or `kwargs` in macros.\n\
3. If editing a file in `macros/`, output ONLY `{{% macro %}}...{{% endmacro %}}` definitions and no top-level HTML.\n\
4. Return ONLY the complete updated MiniJinja template file content without surrounding markdown code fences.",
                target.path,
                vars_desc.join("\n   ")
            ))
        }
        "theme" => {
            Ok("You are Pico Vibe Assistant editing `assets/theme.css`.\n\
STRICT GUARDRAILS:\n\
1. You may ONLY modify CSS variables (such as `--bg`, `--surface`, `--text`, `--muted`, `--accent`, `--border`, `--heading-font`, `--body-font`, `--code-font`) and CSS rules in `assets/theme.css`.\n\
2. Never output HTML or Jinja tags.\n\
3. Return ONLY the complete updated `assets/theme.css` file without surrounding markdown code fences.".to_string())
        }
        other => Err(format!("Unsupported Vibe target kind: '{}'", other)),
    }
}

/// Validates and packages a raw proposed output from an LLM against Pico's guardrails.
pub fn evaluate_vibe_proposal(
    site_root: &Path,
    target: &VibeTarget,
    original_content: &str,
    raw_llm_output: &str,
) -> Result<VibeResult, String> {
    let cleaned = strip_llm_code_fence(raw_llm_output);

    match target.kind.as_str() {
        "content" => {
            let body_only = strip_frontmatter_from_body(&cleaned);
            let diff = compute_unified_diff(&target.path, original_content, &body_only);
            Ok(VibeResult {
                diff,
                proposed: body_only,
                validation_error: None,
                sources: None,
            })
        }
        "template" => {
            let models = load_models(site_root).map_err(|e| e.to_string())?;
            let all_templates = load_all_templates(site_root).map_err(|e| e.to_string())?;
            let rel_tpl = target
                .path
                .trim_start_matches("templates/")
                .trim_start_matches('/');
            let diff = compute_unified_diff(&target.path, original_content, &cleaned);
            let validation_error =
                validate_template_with_models(rel_tpl, &cleaned, &all_templates, &models)
                    .err()
                    .map(|e| e.to_string());
            Ok(VibeResult {
                diff,
                proposed: cleaned,
                validation_error,
                sources: None,
            })
        }
        "theme" => {
            let norm = target.path.replace('\\', "/");
            if norm != "assets/theme.css" && norm != "theme.css" {
                return Err(
                    "Theme vibe may ONLY edit `assets/theme.css`, never templates or skeleton.jinja."
                        .to_string(),
                );
            }
            let validation_error = if cleaned.contains("{%") || cleaned.contains("{{") || cleaned.contains("<html") {
                Some("Theme vibe output must contain only CSS rules in assets/theme.css, not HTML or Jinja templates.".to_string())
            } else {
                None
            };
            let diff = compute_unified_diff("assets/theme.css", original_content, &cleaned);
            Ok(VibeResult {
                diff,
                proposed: cleaned,
                validation_error,
                sources: None,
            })
        }
        other => Err(format!("Unknown Vibe target kind '{}'", other)),
    }
}

/// Evaluates a proposal with optional grounding sources attached.
pub fn evaluate_vibe_proposal_with_sources(
    site_root: &Path,
    target: &VibeTarget,
    original_content: &str,
    raw_llm_output: &str,
    sources: Option<Vec<crate::models::GroundingSource>>,
) -> Result<VibeResult, String> {
    let mut res = evaluate_vibe_proposal(site_root, target, original_content, raw_llm_output)?;
    res.sources = sources;
    Ok(res)
}

/// Reads the current file content for a `VibeTarget`.
pub fn read_vibe_target_content(site_root: &Path, target: &VibeTarget) -> Result<String, String> {
    match target.kind.as_str() {
        "content" => {
            let rel = target.path.trim_start_matches('/');
            let full = site_root.join(rel);
            let raw = fs::read_to_string(&full)
                .map_err(|e| format!("Could not read {}: {}", rel, e))?;
            let (_, markdown, _) = crate::validation::parse_markdown_document(&raw, rel)
                .map_err(|e| e.to_string())?;
            Ok(markdown)
        }
        "template" => {
            let rel = target
                .path
                .trim_start_matches("templates/")
                .trim_start_matches('/');
            let full = site_root.join("templates").join(rel);
            fs::read_to_string(&full)
                .map_err(|e| format!("Could not read templates/{}: {}", rel, e))
        }
        "theme" => {
            let full = site_root.join("assets").join("theme.css");
            fs::read_to_string(&full)
                .map_err(|e| format!("Could not read assets/theme.css: {}", e))
        }
        other => Err(format!("Unsupported Vibe target kind: {}", other)),
    }
}

/// Executes a Vibe edit against the user's configured OpenAI-compatible LLM endpoint.
/// Never fakes a response or applies changes automatically.
pub async fn run_vibe_edit_with_llm(
    site_root: &Path,
    target: &VibeTarget,
    prompt: &str,
    settings: &PersistedAppSettings,
) -> Result<VibeResult, String> {
    if settings.llm_api_key.trim().is_empty() {
        return Err("Configure an LLM API key in Settings to use Vibe.".to_string());
    }
    if prompt.trim().is_empty() {
        return Err("Please enter a Vibe instruction prompt.".to_string());
    }

    let original_content = read_vibe_target_content(site_root, target)?;
    let system_prompt = build_vibe_system_prompt(site_root, target)?;
    let user_msg = format!(
        "Instruction: {}\n\nCurrent file (`{}`):\n{}",
        prompt.trim(),
        target.path,
        original_content
    );

    let endpoint = if settings.llm_endpoint.trim().is_empty() {
        "https://api.openai.com/v1/chat/completions"
    } else {
        settings.llm_endpoint.trim()
    };
    let model = if settings.llm_model.trim().is_empty() {
        "gpt-4o-mini"
    } else {
        settings.llm_model.trim()
    };

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_msg }
        ]
    });

    let resp = client
        .post(endpoint)
        .bearer_auth(settings.llm_api_key.trim())
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "LLM endpoint returned HTTP {}: {}",
            status.as_u16(),
            body.lines().next().unwrap_or("request rejected")
        ));
    }

    let json_resp: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid JSON response from LLM endpoint: {}", e))?;

    let proposed_raw = json_resp
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "LLM response did not contain choices[0].message.content.".to_string())?;

    evaluate_vibe_proposal(site_root, target, &original_content, proposed_raw)
}
