use crate::models::BuildError;
use std::collections::HashMap;

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
