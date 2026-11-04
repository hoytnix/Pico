use crate::models::{BuildError, ModelDef};
use minijinja::{AutoEscape, Environment};
use regex::Regex;
use serde_json::{Map, Value};
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

/// Slugifies a string: lowercase, non-alphanumeric -> `-`, collapse multiple `-`, trim `-`.
pub fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// Checks for duplicate slugs within a collection.
/// Each tuple is `(file_path, slug, line_number)`.
pub fn check_duplicate_slugs(entries: &[(String, String, usize)]) -> Vec<BuildError> {
    let mut seen: HashMap<&str, &str> = HashMap::new();
    let mut errors = Vec::new();
    for (file, slug, line) in entries {
        if slug.is_empty() {
            errors.push(BuildError {
                file: file.clone(),
                line: *line,
                message: "Post slug is empty and could not be derived from title.".to_string(),
            });
            continue;
        }
        if let Some(first_file) = seen.get(slug.as_str()) {
            errors.push(BuildError {
                file: file.clone(),
                line: *line,
                message: format!(
                    "Duplicate slug '{}' in collection (already used by {}).",
                    slug, first_file
                ),
            });
        } else {
            seen.insert(slug.as_str(), file.as_str());
        }
    }
    errors
}

/// Strips any leading `---\n...\n---\n` frontmatter block from a markdown body string.
pub fn strip_frontmatter_from_body(markdown: &str) -> String {
    let normalized = markdown.replace("\r\n", "\n");
    if let Some(rest) = normalized.strip_prefix("---\n") {
        if let Some(end_idx) = rest.find("\n---\n") {
            return rest[end_idx + 5..].trim_start_matches('\n').to_string();
        } else if let Some(end_idx) = rest.find("\n---") {
            let after = &rest[end_idx + 4..];
            if after.trim().is_empty() {
                return String::new();
            }
        }
    }
    markdown.to_string()
}

/// Parses a Markdown document containing YAML frontmatter (`---\n...\n---\n`) and Markdown body.
pub fn parse_markdown_document(
    raw: &str,
    file_name: &str,
) -> Result<(Value, String, String), BuildError> {
    let normalized = raw.replace("\r\n", "\n");
    let trimmed = normalized.trim_start_matches('\u{feff}');
    if !trimmed.starts_with("---\n") {
        return Err(BuildError {
            file: file_name.to_string(),
            line: 1,
            message: "Missing YAML frontmatter opening '---' on line 1.".to_string(),
        });
    }

    let after_open = &trimmed[4..];
    let (yaml_part, body_part) = if let Some(idx) = after_open.find("\n---\n") {
        (&after_open[..idx], &after_open[idx + 5..])
    } else if after_open.starts_with("---\n") {
        ("", &after_open[4..])
    } else if let Some(idx) = after_open.find("\n---") {
        let tail = &after_open[idx + 4..];
        if tail.trim().is_empty() {
            (&after_open[..idx], "")
        } else {
            return Err(BuildError {
                file: file_name.to_string(),
                line: 1,
                message: "Malformed YAML frontmatter closing '---'.".to_string(),
            });
        }
    } else {
        return Err(BuildError {
            file: file_name.to_string(),
            line: 1,
            message: "Missing YAML frontmatter closing '---'.".to_string(),
        });
    };

    let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_part).map_err(|err| {
        let line = err.location().map(|loc| loc.line() + 1).unwrap_or(2);
        BuildError {
            file: file_name.to_string(),
            line,
            message: format!("Invalid YAML in frontmatter: {}", err),
        }
    })?;

    let json_val: Value = serde_json::to_value(yaml_val).map_err(|err| BuildError {
        file: file_name.to_string(),
        line: 2,
        message: format!("Could not convert frontmatter to JSON value: {}", err),
    })?;

    let obj = match json_val {
        Value::Object(map) => Value::Object(map),
        Value::Null => Value::Object(Map::new()),
        _ => {
            return Err(BuildError {
                file: file_name.to_string(),
                line: 2,
                message: "Frontmatter must be a YAML key-value mapping.".to_string(),
            })
        }
    };

    let clean_body = strip_frontmatter_from_body(body_part.trim_start_matches('\n'));
    Ok((obj, clean_body, yaml_part.to_string()))
}

fn find_key_line_in_yaml(raw_yaml: Option<&str>, key: &str) -> usize {
    if let Some(yaml) = raw_yaml {
        let prefix = format!("{}:", key);
        for (idx, line) in yaml.lines().enumerate() {
            if line.trim_start().starts_with(&prefix) {
                return idx + 2; // +1 for 1-based, +1 for opening `---`
            }
        }
    }
    2
}

/// Validates parsed frontmatter against a collection's `ModelDef`.
/// Checks required fields, unknown fields not in the model, and field types.
pub fn validate_frontmatter(
    frontmatter: &Value,
    model: &ModelDef,
    file_name: &str,
    raw_yaml: Option<&str>,
) -> Result<(), Vec<BuildError>> {
    let obj = match frontmatter.as_object() {
        Some(m) => m,
        None => {
            return Err(vec![BuildError {
                file: file_name.to_string(),
                line: 1,
                message: "Frontmatter must be a YAML object.".to_string(),
            }])
        }
    };

    let mut errors = Vec::new();
    let known_fields: HashMap<&str, &crate::models::ModelField> = model
        .fields
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();

    // 1. Check unknown fields in frontmatter
    for key in obj.keys() {
        if !known_fields.contains_key(key.as_str()) {
            let line = find_key_line_in_yaml(raw_yaml, key);
            errors.push(BuildError {
                file: file_name.to_string(),
                line,
                message: format!(
                    "Unknown field '{}' in frontmatter (not defined in {} model).",
                    key, model.label
                ),
            });
        }
    }

    // 2. Check required fields and field types
    for field in &model.fields {
        let val_opt = obj.get(&field.name);
        let line = find_key_line_in_yaml(raw_yaml, &field.name);

        match val_opt {
            None | Some(Value::Null) => {
                if field.required {
                    errors.push(BuildError {
                        file: file_name.to_string(),
                        line: 2,
                        message: format!(
                            "Missing required field '{}' ({}) in frontmatter.",
                            field.name, field.label
                        ),
                    });
                }
            }
            Some(val) => {
                if field.required {
                    if let Some(s) = val.as_str() {
                        if s.trim().is_empty() {
                            errors.push(BuildError {
                                file: file_name.to_string(),
                                line,
                                message: format!(
                                    "Required field '{}' ({}) cannot be empty.",
                                    field.name, field.label
                                ),
                            });
                        }
                    }
                }

                // Type checking
                match field.field_type.as_str() {
                    "string" | "date" | "image" => {
                        if !val.is_string() {
                            errors.push(BuildError {
                                file: file_name.to_string(),
                                line,
                                message: format!(
                                    "Field '{}' must be a string (got {}).",
                                    field.name, val
                                ),
                            });
                        }
                    }
                    "bool" => {
                        if !val.is_boolean() {
                            errors.push(BuildError {
                                file: file_name.to_string(),
                                line,
                                message: format!(
                                    "Field '{}' must be a boolean true/false (got {}).",
                                    field.name, val
                                ),
                            });
                        }
                    }
                    "tags" => {
                        let valid_tags = val.is_string()
                            || val
                                .as_array()
                                .map(|arr| arr.iter().all(|v| v.is_string()))
                                .unwrap_or(false);
                        if !valid_tags {
                            errors.push(BuildError {
                                file: file_name.to_string(),
                                line,
                                message: format!(
                                    "Field '{}' must be a list of tag strings.",
                                    field.name
                                ),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Serializes frontmatter + Markdown body back to `---\n<yaml>\n---\n\n<markdown>\n`.
/// Ensures frontmatter is never duplicated inside the Markdown body.
pub fn serialize_markdown_document(
    frontmatter: &Value,
    markdown: &str,
    model: Option<&ModelDef>,
) -> Result<String, BuildError> {
    let obj = frontmatter.as_object().ok_or_else(|| BuildError {
        file: String::new(),
        line: 1,
        message: "Frontmatter must be a JSON object.".to_string(),
    })?;

    let mut yaml_map = serde_yaml::Mapping::new();

    if let Some(m) = model {
        for field in &m.fields {
            if let Some(v) = obj.get(&field.name) {
                let yv = serde_yaml::to_value(v).unwrap_or(serde_yaml::Value::Null);
                yaml_map.insert(serde_yaml::Value::String(field.name.clone()), yv);
            } else if let Some(def) = &field.default {
                let yv = serde_yaml::to_value(def).unwrap_or(serde_yaml::Value::Null);
                yaml_map.insert(serde_yaml::Value::String(field.name.clone()), yv);
            }
        }
    } else {
        for (k, v) in obj {
            let yv = serde_yaml::to_value(v).unwrap_or(serde_yaml::Value::Null);
            yaml_map.insert(serde_yaml::Value::String(k.clone()), yv);
        }
    }

    let yaml_str = serde_yaml::to_string(&yaml_map).map_err(|e| BuildError {
        file: String::new(),
        line: 1,
        message: format!("Failed to serialize frontmatter: {}", e),
    })?;

    let clean_body = strip_frontmatter_from_body(markdown);
    let trimmed_body = clean_body.trim_end();
    Ok(format!(
        "---\n{}---\n\n{}\n",
        yaml_str,
        trimmed_body
    ))
}

fn find_line_of_token(content: &str, needle: &str) -> usize {
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
