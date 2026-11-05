use crate::models::{BuildError, ModelDef};
use minijinja::{AutoEscape, Environment};
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub const BUILTIN_ITEM_FIELDS: &[&str] = &[
    "url",
    "body_html",
    "body",
    "collection",
    "hero_webp",
    "thumb_webp",
    "attachments",
];

pub const SITE_CONFIG_FIELDS: &[&str] = &[
    "title",
    "base_url",
    "description",
    "author",
    "nav",
    "form_endpoint",
    "comments_provider",
    "comments_repo",
];

pub const TOP_LEVEL_CONTEXT_VARS: &[&str] = &[
    "site",
    "post",
    "posts",
    "project",
    "projects",
    "doc",
    "docs",
    "item",
    "items",
    "all_posts",
    "collections",
    "page",
    "total_pages",
    "prev_url",
    "next_url",
    "tag",
    "tags",
    "collection",
    "is_preview",
    "namespace",
    "range",
    "dict",
    "cycler",
    "joiner",
    "loop",
];

pub fn find_line_of_token(content: &str, needle: &str) -> usize {
    for (idx, line) in content.lines().enumerate() {
        if line.contains(needle) {
            return idx + 1;
        }
    }
    1
}

/// Validates a MiniJinja template for:
/// 1. Macro rules (no page HTML outside `{% macro %}` in `macros/*.jinja`, no `varargs` or `kwargs`).
/// 2. MiniJinja syntax and inheritance/import compilation.
/// 3. Model field and context variable guardrails (refuses unknown variables like `post.author_bio` with file + line + friendly message).
pub fn validate_template_with_models(
    template_rel_path: &str,
    content: &str,
    all_templates: &HashMap<String, String>,
    models: &[ModelDef],
) -> Result<(), BuildError> {
    let norm_path = template_rel_path
        .trim_start_matches('/')
        .replace('\\', "/");

    // 1. Check macro restrictions (§4.5, §4.6)
    let kwargs_re = Regex::new(r"\b(varargs|kwargs)\b|\*\*[a-zA-Z_]").unwrap();
    for (idx, line) in content.lines().enumerate() {
        if kwargs_re.is_match(line) {
            return Err(BuildError {
                file: norm_path,
                line: idx + 1,
                message: "MiniJinja macros do not support `varargs` or `kwargs`. Define explicit macro arguments only.".to_string(),
            });
        }
    }

    let is_macro_file = norm_path.starts_with("macros/");
    if is_macro_file {
        // Ensure macro file contains ONLY macros and comments/whitespace, no top-level HTML
        let mut inside_macro = 0usize;
        let mut inside_comment = false;
        for (idx, raw_line) in content.lines().enumerate() {
            let mut line = raw_line.trim();
            if inside_comment {
                if let Some(end_c) = line.find("#}") {
                    inside_comment = false;
                    line = line[end_c + 2..].trim();
                } else {
                    continue;
                }
            }
            if line.starts_with("{#") {
                if !line.contains("#}") {
                    inside_comment = true;
                }
                continue;
            }
            if line.contains("{%") && line.contains("macro ") && !line.contains("endmacro") {
                inside_macro += 1;
                continue;
            }
            if line.contains("{%") && line.contains("endmacro") {
                inside_macro = inside_macro.saturating_sub(1);
                continue;
            }
            if inside_macro == 0 && !line.is_empty() {
                return Err(BuildError {
                    file: norm_path.clone(),
                    line: idx + 1,
                    message: format!(
                        "{} is in macros/ and must contain ONLY `{{% macro %}}` definitions, no top-level page HTML.",
                        norm_path
                    ),
                });
            }
        }
    }

    // 2. Compile in MiniJinja Environment with fuel limit
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| AutoEscape::Html);
    env.set_fuel(Some(50_000));

    let mut merged = all_templates.clone();
    merged.insert(norm_path.clone(), content.to_string());

    for (name, src) in &merged {
        if name != &norm_path {
            let _ = env.add_template_owned(name.clone(), src.clone());
        }
    }

    if let Err(err) = env.add_template_owned(norm_path.clone(), content.to_string()) {
        let line = err.line().unwrap_or(1);
        return Err(BuildError {
            file: norm_path,
            line,
            message: format!("Template syntax error: {}", err.detail().unwrap_or("invalid MiniJinja syntax")),
        });
    }

    let tmpl = env.get_template(&norm_path).map_err(|err| BuildError {
        file: norm_path.clone(),
        line: err.line().unwrap_or(1),
        message: format!("Template error: {}", err),
    })?;

    // 3. Determine associated collection model (if any)
    let matched_model: Option<&ModelDef> = models.iter().find(|m| {
        m.template == norm_path
            || m.list_template == norm_path
            || norm_path.starts_with(&format!("{}.", m.name))
            || norm_path.starts_with(&format!("{}-", m.name))
    });

    // Allowed fields on post / item
    let mut allowed_item_fields: HashSet<&str> =
        BUILTIN_ITEM_FIELDS.iter().copied().collect();
    let model_label = if let Some(m) = matched_model {
        for f in &m.fields {
            allowed_item_fields.insert(f.name.as_str());
        }
        m.label.clone()
    } else {
        for m in models {
            for f in &m.fields {
                allowed_item_fields.insert(f.name.as_str());
            }
        }
        models
            .first()
            .map(|m| m.label.clone())
            .unwrap_or_else(|| "Blog".to_string())
    };

    let allowed_site_fields: HashSet<&str> = SITE_CONFIG_FIELDS.iter().copied().collect();

    // Scan expressions inside {{ ... }} and {% ... %} for dotted accesses like `post.author_bio` or `site.unknown`
    let tag_re = Regex::new(r"(?s)(\{\{.*?\}\}|\{%.*?%\})").unwrap();
    let prop_re = Regex::new(r"\b(post|project|doc|item|site)\.([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();

    for m in tag_re.find_iter(content) {
        let snippet = m.as_str();
        for cap in prop_re.captures_iter(snippet) {
            let obj_name = &cap[1];
            let prop_name = &cap[2];
            let full_expr = format!("{}.{}", obj_name, prop_name);
            let byte_offset = m.start();
            let line = content[..byte_offset].lines().count().max(1);

            if obj_name == "site" {
                if !allowed_site_fields.contains(prop_name) {
                    return Err(BuildError {
                        file: norm_path.clone(),
                        line,
                        message: format!(
                            "{} uses `{}`, but site.yaml doesn't have `{}`",
                            norm_path, full_expr, prop_name
                        ),
                    });
                }
            } else if !allowed_item_fields.contains(prop_name) {
                return Err(BuildError {
                    file: norm_path.clone(),
                    line,
                    message: format!(
                        "{} uses `{}`, but {} posts don't have `{}`",
                        norm_path, full_expr, model_label, prop_name
                    ),
                });
            }
        }
    }

    // Also check MiniJinja undeclared variables
    let undeclared_nested = tmpl.undeclared_variables(true);
    for var in &undeclared_nested {
        if let Some((obj_name, rest)) = var.split_once('.') {
            let prop_name = rest.split('.').next().unwrap_or(rest);
            if matches!(obj_name, "post" | "project" | "doc" | "item")
                && !allowed_item_fields.contains(prop_name)
            {
                let full_expr = format!("{}.{}", obj_name, prop_name);
                let line = find_line_of_token(content, &full_expr);
                return Err(BuildError {
                    file: norm_path.clone(),
                    line,
                    message: format!(
                        "{} uses `{}`, but {} posts don't have `{}`",
                        norm_path, full_expr, model_label, prop_name
                    ),
                });
            } else if obj_name == "site" && !allowed_site_fields.contains(prop_name) {
                let full_expr = format!("site.{}", prop_name);
                let line = find_line_of_token(content, &full_expr);
                return Err(BuildError {
                    file: norm_path.clone(),
                    line,
                    message: format!(
                        "{} uses `{}`, but site.yaml doesn't have `{}`",
                        norm_path, full_expr, prop_name
                    ),
                });
            }
        }
    }

    // Check top-level undeclared variables (for non-macro templates)
    if !is_macro_file {
        let mut allowed_top: HashSet<&str> = TOP_LEVEL_CONTEXT_VARS.iter().copied().collect();
        for m in models {
            allowed_top.insert(m.name.as_str());
        }
        let undeclared_top = tmpl.undeclared_variables(false);
        for var in undeclared_top {
            if !allowed_top.contains(var.as_str()) {
                let line = find_line_of_token(content, &var);
                return Err(BuildError {
                    file: norm_path.clone(),
                    line,
                    message: format!(
                        "{} uses `{}`, which is not in the {} model or template context",
                        norm_path, var, model_label
                    ),
                });
            }
        }
    }

    Ok(())
}
