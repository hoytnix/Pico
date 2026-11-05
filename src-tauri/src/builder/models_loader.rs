use crate::models::{BuildError, ModelDef, SiteConfig};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn resolve_collection_folder(site_root: &Path, model_name: &str) -> (String, PathBuf) {
    let direct = site_root.join("content").join(model_name);
    if direct.is_dir() {
        return (model_name.to_string(), direct);
    }
    let plural = format!("{}s", model_name);
    let plural_dir = site_root.join("content").join(&plural);
    if plural_dir.is_dir() {
        return (plural, plural_dir);
    }
    (model_name.to_string(), direct)
}

pub fn load_site_config(site_root: &Path) -> Result<SiteConfig, BuildError> {
    let cfg_path = site_root.join("site.yaml");
    let raw = fs::read_to_string(&cfg_path).map_err(|e| BuildError {
        file: "site.yaml".to_string(),
        line: 1,
        message: format!("Could not read site.yaml: {}", e),
    })?;
    serde_yaml::from_str::<SiteConfig>(&raw).map_err(|e| {
        let line = e.location().map(|l| l.line()).unwrap_or(1);
        BuildError {
            file: "site.yaml".to_string(),
            line,
            message: format!("Invalid site.yaml: {}", e),
        }
    })
}

pub fn load_models(site_root: &Path) -> Result<Vec<ModelDef>, BuildError> {
    let models_dir = site_root.join("models");
    if !models_dir.is_dir() {
        return Err(BuildError {
            file: "models/".to_string(),
            line: 0,
            message: "Missing models/ directory in site folder.".to_string(),
        });
    }
    let mut models = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(&models_dir)
        .map_err(|e| BuildError {
            file: "models/".to_string(),
            line: 0,
            message: format!("Could not read models/ directory: {}", e),
        })?
        .filter_map(|e| e.ok().map(|d| d.path()))
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s == "yaml" || s == "yml")
                .unwrap_or(false)
        })
        .collect();
    entries.sort();

    for path in entries {
        let rel = format!(
            "models/{}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("model.yaml")
        );
        let raw = fs::read_to_string(&path).map_err(|e| BuildError {
            file: rel.clone(),
            line: 1,
            message: format!("Could not read {}: {}", rel, e),
        })?;
        let model: ModelDef = serde_yaml::from_str(&raw).map_err(|e| {
            let line = e.location().map(|l| l.line()).unwrap_or(1);
            BuildError {
                file: rel.clone(),
                line,
                message: format!("Invalid model YAML in {}: {}", rel, e),
            }
        })?;
        models.push(model);
    }

    // Ensure "blog" comes first if present
    models.sort_by(|a, b| {
        if a.name == "blog" {
            std::cmp::Ordering::Less
        } else if b.name == "blog" {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(models)
}

pub fn load_all_templates(site_root: &Path) -> Result<HashMap<String, String>, BuildError> {
    let tpl_dir = site_root.join("templates");
    if !tpl_dir.is_dir() {
        return Err(BuildError {
            file: "templates/".to_string(),
            line: 0,
            message: "Missing templates/ directory in site folder.".to_string(),
        });
    }
    let mut map = HashMap::new();
    for entry in WalkDir::new(&tpl_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("jinja") {
                if let Ok(rel) = p.strip_prefix(&tpl_dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    // Skip backup files if any
                    if !rel_str.ends_with(".bak") {
                        let content = fs::read_to_string(p).map_err(|e| BuildError {
                            file: format!("templates/{}", rel_str),
                            line: 1,
                            message: format!("Could not read template {}: {}", rel_str, e),
                        })?;
                        map.insert(rel_str, content);
                    }
                }
            }
        }
    }
    Ok(map)
}
