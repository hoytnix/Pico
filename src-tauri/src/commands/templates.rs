use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::State;

use crate::builder::{load_all_templates, load_models};
use crate::models::{AppState, ModelDef, TemplateContextInfo, VariableDoc};
use crate::validation::{validate_template_with_models, BUILTIN_ITEM_FIELDS, SITE_CONFIG_FIELDS};
use super::require_current_site;

/// Core implementation of `save_template` with MiniJinja validation and revert backup (§3.4, §5).
pub fn save_template_impl(
    site_root: &Path,
    template_rel_path: &str,
    content: &str,
    backups: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    let norm = template_rel_path
        .trim_start_matches("templates/")
        .trim_start_matches('/')
        .replace('\\', "/");
    let models = load_models(site_root).map_err(|e| e.to_string())?;
    let all_templates = load_all_templates(site_root).map_err(|e| e.to_string())?;

    validate_template_with_models(&norm, content, &all_templates, &models)
        .map_err(|e| e.to_string())?;

    let tpl_full_path = site_root.join("templates").join(&norm);

    // Save current version for one-click Revert (§3.4)
    if tpl_full_path.is_file() {
        if let Ok(prev_content) = fs::read_to_string(&tpl_full_path) {
            if prev_content != content {
                backups
                    .entry(norm.clone())
                    .or_default()
                    .push(prev_content.clone());
                let disk_bak = site_root
                    .join(".pico_history")
                    .join("templates")
                    .join(format!("{}.bak", norm));
                if let Some(parent) = disk_bak.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(disk_bak, prev_content);
            }
        }
    }

    if let Some(parent) = tpl_full_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&tpl_full_path, content)
        .map_err(|e| format!("templates/{}: Could not write template: {}", norm, e))?;
    Ok(())
}

/// Core implementation of `revert_template` (§3.4, §5).
pub fn revert_template_impl(
    site_root: &Path,
    template_rel_path: &str,
    backups: &mut HashMap<String, Vec<String>>,
) -> Result<String, String> {
    let norm = template_rel_path
        .trim_start_matches("templates/")
        .trim_start_matches('/')
        .replace('\\', "/");
    let tpl_full_path = site_root.join("templates").join(&norm);

    let prev = if let Some(stack) = backups.get_mut(&norm) {
        stack.pop()
    } else {
        None
    }
    .or_else(|| {
        let disk_bak = site_root
            .join(".pico_history")
            .join("templates")
            .join(format!("{}.bak", norm));
        if disk_bak.is_file() {
            let s = fs::read_to_string(&disk_bak).ok();
            let _ = fs::remove_file(&disk_bak);
            s
        } else {
            None
        }
    })
    .ok_or_else(|| format!("No previous version available to revert for `{}`.", norm))?;

    fs::write(&tpl_full_path, &prev)
        .map_err(|e| format!("Failed to restore previous version of {}: {}", norm, e))?;
    Ok(prev)
}

#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let site_root = require_current_site(&state)?;
    log::debug!(target: "pico::templates", "list_templates in {:?}", site_root);
    let map = load_all_templates(&site_root).map_err(|e| e.to_string())?;
    let mut names: Vec<String> = map.into_keys().collect();
    names.sort_by(|a, b| {
        if a == "skeleton.jinja" {
            std::cmp::Ordering::Less
        } else if b == "skeleton.jinja" {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });
    Ok(names)
}

#[tauri::command]
pub fn read_template(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    log::debug!(target: "pico::templates", "read_template: {:?}", path);
    let norm = path
        .trim_start_matches("templates/")
        .trim_start_matches('/')
        .replace('\\', "/");
    let full = site_root.join("templates").join(&norm);
    fs::read_to_string(&full).map_err(|e| format!("templates/{}: Could not read file: {}", norm, e))
}

#[tauri::command]
pub fn save_template(
    path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::templates", "save_template: {:?} ({} chars)", path, content.len());
    let mut backups = state
        .template_backups
        .lock()
        .map_err(|_| "State lock error")?;
    save_template_impl(&site_root, &path, &content, &mut backups).map_err(|err| {
        log::warn!(target: "pico::templates", "save_template rejected for {:?}: {}", path, err);
        err
    })?;
    log::info!(target: "pico::templates", "save_template succeeded: {:?}", path);
    Ok(())
}

#[tauri::command]
pub fn revert_template(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::templates", "revert_template: {:?}", path);
    let mut backups = state
        .template_backups
        .lock()
        .map_err(|_| "State lock error")?;
    revert_template_impl(&site_root, &path, &mut backups).map_err(|err| {
        log::error!(target: "pico::templates", "revert_template failed for {:?}: {}", path, err);
        err
    })
}

#[tauri::command]
pub fn get_template_context(
    path: String,
    state: State<'_, AppState>,
) -> Result<TemplateContextInfo, String> {
    let site_root = require_current_site(&state)?;
    let norm = path
        .trim_start_matches("templates/")
        .trim_start_matches('/')
        .replace('\\', "/");
    let models = load_models(&site_root).map_err(|e| e.to_string())?;

    let matched_model = models.iter().find(|m| {
        m.template == norm
            || m.list_template == norm
            || norm.starts_with(&format!("{}.", m.name))
            || norm.starts_with(&format!("{}-", m.name))
    });

    let mut variables = Vec::new();
    for sf in SITE_CONFIG_FIELDS {
        variables.push(VariableDoc {
            name: format!("site.{}", sf),
            kind: "site".to_string(),
            description: format!("Site configuration field `{}` from site.yaml", sf),
        });
    }

    let target_models: Vec<&ModelDef> = if let Some(m) = matched_model {
        vec![m]
    } else {
        models.iter().collect()
    };

    for m in &target_models {
        for f in &m.fields {
            variables.push(VariableDoc {
                name: format!("post.{}", f.name),
                kind: f.field_type.clone(),
                description: format!("{} ({})", f.label, m.label),
            });
        }
    }

    for bf in BUILTIN_ITEM_FIELDS {
        variables.push(VariableDoc {
            name: format!("post.{}", bf),
            kind: "computed".to_string(),
            description: format!("Built-in computed post property `{}`", bf),
        });
    }

    let has_mem_revert = state
        .template_backups
        .lock()
        .ok()
        .and_then(|b| b.get(&norm).map(|v| !v.is_empty()))
        .unwrap_or(false);
    let disk_bak = site_root
        .join(".pico_history")
        .join("templates")
        .join(format!("{}.bak", norm));

    Ok(TemplateContextInfo {
        template_path: norm.clone(),
        is_skeleton: norm == "skeleton.jinja",
        is_macro: norm.starts_with("macros/"),
        collection_label: matched_model.map(|m| m.label.clone()),
        has_revert: has_mem_revert || disk_bak.is_file(),
        variables,
    })
}

#[tauri::command]
pub fn read_theme(state: State<'_, AppState>) -> Result<String, String> {
    let site_root = require_current_site(&state)?;
    log::debug!(target: "pico::theme", "read_theme in {:?}", site_root);
    let p = site_root.join("assets").join("theme.css");
    fs::read_to_string(&p).map_err(|e| format!("assets/theme.css: {}", e))
}

#[tauri::command]
pub fn save_theme(content: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::theme", "save_theme: {} chars", content.len());
    if content.contains("{%") || content.contains("{{") {
        let msg = "assets/theme.css must contain CSS only, not MiniJinja tags.".to_string();
        log::warn!(target: "pico::theme", "save_theme rejected: {}", msg);
        return Err(msg);
    }
    let p = site_root.join("assets").join("theme.css");
    fs::write(&p, content).map_err(|e| {
        let msg = format!("Could not save assets/theme.css: {}", e);
        log::error!(target: "pico::theme", "{}", msg);
        msg
    })
}
