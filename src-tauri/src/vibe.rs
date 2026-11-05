use crate::builder::{load_all_templates, load_models};
use crate::models::{VibeResult, VibeSiteFile, VibeSiteSnapshot, VibeTarget};
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
        "model" => {
            let mut existing_models = Vec::new();
            for m in &models {
                let fnames: Vec<&str> = m.fields.iter().map(|f| f.name.as_str()).collect();
                existing_models.push(format!("- Model '{}' (collection '{}'): fields: [{}]", m.label, m.name, fnames.join(", ")));
            }
            Ok(format!(
                "You are Pico Vibe Assistant editing collection models in `models/*.yaml`.\n\
STRICT GUARDRAILS:\n\
1. Models must be valid YAML with top-level fields:\n\
   - `name`: string slug (e.g. blog, projects, team)\n\
   - `label`: human-readable label (e.g. Blog, Projects, Team Members)\n\
   - `template`: single item template (e.g. <name>.jinja)\n\
   - `list_template`: list template (e.g. <name>-list.jinja)\n\
   - `fields`: list of field definitions\n\
2. Each field in `fields` must have:\n\
   - `name`: field identifier string (must include `title` and `date`)\n\
   - `label`: display label\n\
   - `type`: one of 'string', 'date', 'tags', 'image', 'bool'\n\
   - `required`: boolean (true/false)\n\
   - optional `default`\n\
3. Existing models in site:\n{}\n\
4. Return ONLY the complete YAML content for the model without surrounding markdown code fences.",
                if existing_models.is_empty() { "   (none)".to_string() } else { existing_models.join("\n") }
            ))
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
            let norm_clean = norm.trim_start_matches('.').trim_start_matches('/');
            if norm_clean != "assets/theme.css" && norm_clean != "theme.css" {
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
        "model" => {
            let norm = target.path.replace('\\', "/");
            let rel = norm
                .trim_start_matches('.')
                .trim_start_matches('/')
                .trim_start_matches("models/")
                .trim_start_matches('/');
            let yaml_name = if rel.ends_with(".yaml") || rel.ends_with(".yml") {
                rel.to_string()
            } else {
                format!("{}.yaml", rel)
            };
            let path_label = format!("models/{}", yaml_name);

            let parsed: Result<crate::models::ModelDef, serde_yaml::Error> = serde_yaml::from_str(&cleaned);
            let validation_error = match parsed {
                Ok(model) => {
                    let mut errs = Vec::new();
                    if model.name.trim().is_empty() {
                        errs.push("Model 'name' cannot be empty.".to_string());
                    }
                    if model.label.trim().is_empty() {
                        errs.push("Model 'label' cannot be empty.".to_string());
                    }
                    let valid_types = [
                        "string", "date", "tags", "image", "bool",
                        "text", "markdown", "number", "boolean", "select",
                    ];
                    for field in &model.fields {
                        if field.name.trim().is_empty() {
                            errs.push("Field 'name' cannot be empty.".to_string());
                        }
                        if !valid_types.contains(&field.field_type.as_str()) {
                            errs.push(format!(
                                "Field '{}' has unsupported type '{}'. Valid types: string, date, tags, image, bool.",
                                field.name, field.field_type
                            ));
                        }
                    }
                    if errs.is_empty() {
                        None
                    } else {
                        Some(errs.join(" "))
                    }
                }
                Err(e) => Some(format!("Invalid YAML for model: {}", e)),
            };

            let diff = compute_unified_diff(&path_label, original_content, &cleaned);
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
        "model" => {
            let norm = target.path.replace('\\', "/");
            let rel = norm
                .trim_start_matches('.')
                .trim_start_matches('/')
                .trim_start_matches("models/")
                .trim_start_matches('/');
            let yaml_name = if rel.ends_with(".yaml") || rel.ends_with(".yml") {
                rel.to_string()
            } else {
                format!("{}.yaml", rel)
            };
            let full = site_root.join("models").join(&yaml_name);
            if full.exists() {
                fs::read_to_string(&full)
                    .map_err(|e| format!("Could not read models/{}: {}", yaml_name, e))
            } else {
                Ok(String::new())
            }
        }
        other => Err(format!("Unsupported Vibe target kind: {}", other)),
    }
}

/// Loads a complete snapshot of the site for Vibe context (site config, models, theme, templates, and content manifest).
pub fn load_vibe_site_snapshot(site_root: &Path) -> Result<VibeSiteSnapshot, String> {
    // 1. site.yaml
    let site_yaml = fs::read_to_string(site_root.join("site.yaml")).unwrap_or_default();

    // 2. assets/theme.css
    let theme_css = fs::read_to_string(site_root.join("assets").join("theme.css")).unwrap_or_default();

    // 3. models/*.yaml
    let mut models_files = Vec::new();
    let models_dir = site_root.join("models");
    if models_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&models_dir) {
            let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|d| d.path())).collect();
            paths.sort();
            for p in paths {
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if ext == "yaml" || ext == "yml" {
                        if let Ok(content) = fs::read_to_string(&p) {
                            let file_name = p
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown.yaml");
                            models_files.push(VibeSiteFile {
                                path: format!("models/{}", file_name),
                                content,
                            });
                        }
                    }
                }
            }
        }
    }

    // 4. templates/**/*.jinja
    let mut template_files = Vec::new();
    if let Ok(all_templates) = load_all_templates(site_root) {
        let mut keys: Vec<_> = all_templates.keys().cloned().collect();
        keys.sort();
        for k in keys {
            if let Some(content) = all_templates.get(&k) {
                template_files.push(VibeSiteFile {
                    path: format!("templates/{}", k),
                    content: content.clone(),
                });
            }
        }
    }

    // 5. content manifest
    let mut manifest = Vec::new();
    if let Ok(models) = load_models(site_root) {
        for m in &models {
            if let Ok(posts) = crate::commands::content::list_content_impl(site_root, &m.name) {
                manifest.extend(posts);
            }
        }
    }

    Ok(VibeSiteSnapshot {
        site_yaml,
        models: models_files,
        theme_css,
        templates: template_files,
        content_manifest: manifest,
    })
}

