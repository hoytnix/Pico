use crate::models::{BuildError, ModelDef};
use serde_json::{Map, Value};
use std::collections::HashMap;

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
