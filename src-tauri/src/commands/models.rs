use std::fs;
use std::path::Path;
use tauri::State;

use crate::builder::load_models;
use crate::models::{AppState, ModelDef, ModelField};
use crate::validation::slugify;
use super::require_current_site;

/// Core implementation of `save_model`.
pub fn save_model_impl(site_root: &Path, model: &ModelDef) -> Result<(), String> {
    if model.name.trim().is_empty() {
        return Err("Model name cannot be empty.".to_string());
    }
    let slug_name = slugify(&model.name);
    let models_dir = site_root.join("models");
    fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;

    let mut clean_model = model.clone();
    clean_model.name = slug_name.clone();
    if clean_model.label.trim().is_empty() {
        clean_model.label = model.name.clone();
    }
    if clean_model.template.trim().is_empty() {
        clean_model.template = format!("{}.jinja", slug_name);
    }
    if clean_model.list_template.trim().is_empty() {
        clean_model.list_template = format!("{}-list.jinja", slug_name);
    }

    // Ensure title and date fields exist in the model if not present
    let has_title = clean_model.fields.iter().any(|f| f.name == "title");
    if !has_title {
        clean_model.fields.insert(
            0,
            ModelField {
                name: "title".to_string(),
                label: "Title".to_string(),
                field_type: "string".to_string(),
                required: true,
                default: None,
            },
        );
    }
    let has_date = clean_model.fields.iter().any(|f| f.name == "date");
    if !has_date {
        clean_model.fields.insert(
            1,
            ModelField {
                name: "date".to_string(),
                label: "Date".to_string(),
                field_type: "date".to_string(),
                required: true,
                default: None,
            },
        );
    }

    let yaml_str = serde_yaml::to_string(&clean_model)
        .map_err(|e| format!("Could not serialize model to YAML: {}", e))?;

    let file_path = models_dir.join(format!("{}.yaml", slug_name));
    fs::write(&file_path, yaml_str)
        .map_err(|e| format!("Could not write model file: {}", e))?;

    // Ensure collection content directory exists
    let content_dir = site_root.join("content").join(&slug_name);
    let _ = fs::create_dir_all(content_dir);

    Ok(())
}

/// Core implementation of `delete_model`.
pub fn delete_model_impl(site_root: &Path, name: &str) -> Result<(), String> {
    let slug_name = slugify(name);
    let models = load_models(site_root).map_err(|e| e.to_string())?;
    if models.len() <= 1 {
        return Err("Cannot delete the only remaining collection model in the site.".to_string());
    }

    let models_dir = site_root.join("models");
    let target1 = models_dir.join(format!("{}.yaml", slug_name));
    let target2 = models_dir.join(format!("{}.yml", slug_name));

    let file_to_del = if target1.is_file() {
        Some(target1)
    } else if target2.is_file() {
        Some(target2)
    } else {
        None
    };

    if let Some(f) = file_to_del {
        trash::delete(&f)
            .or_else(|_| fs::remove_file(&f))
            .map_err(|e| format!("Could not delete model file {}: {}", slug_name, e))?;
        Ok(())
    } else {
        Err(format!("Model '{}' not found on disk.", name))
    }
}

#[tauri::command]
pub fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelDef>, String> {
    let site_root = require_current_site(&state)?;
    log::debug!(target: "pico::models", "list_models in {:?}", site_root);
    load_models(&site_root).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_model(model: ModelDef, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::models", "save_model: name='{}', label='{}', fields={}", model.name, model.label, model.fields.len());
    save_model_impl(&site_root, &model).map_err(|err| {
        log::error!(target: "pico::models", "save_model failed for '{}': {}", model.name, err);
        err
    })
}

#[tauri::command]
pub fn delete_model(name: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::models", "delete_model: '{}'", name);
    delete_model_impl(&site_root, &name).map_err(|err| {
        log::error!(target: "pico::models", "delete_model failed for '{}': {}", name, err);
        err
    })
}

#[tauri::command]
pub fn save_model_yaml(path: String, yaml_content: String, state: State<'_, AppState>) -> Result<(), String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::models", "save_model_yaml: path='{}', length={} bytes", path, yaml_content.len());
    let model: ModelDef = serde_yaml::from_str(&yaml_content)
        .map_err(|e| format!("Invalid YAML for model: {}", e))?;
    if model.name.trim().is_empty() {
        return Err("Model 'name' cannot be empty.".to_string());
    }
    let slug_name = slugify(&model.name);
    let models_dir = site_root.join("models");
    fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    let file_path = models_dir.join(format!("{}.yaml", slug_name));
    fs::write(&file_path, &yaml_content)
        .map_err(|e| format!("Could not write model file: {}", e))?;
    let content_dir = site_root.join("content").join(&slug_name);
    let _ = fs::create_dir_all(content_dir);
    Ok(())
}
