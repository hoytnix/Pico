pub mod content;
pub mod models;
pub mod settings;
pub mod site;
pub mod templates;
pub mod vibe;

use std::path::PathBuf;
use tauri::State;

use crate::models::AppState;

pub fn require_current_site(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    let guard = state.current_site.lock().map_err(|_| "State lock error")?;
    guard
        .clone()
        .ok_or_else(|| "No site folder is currently open. Please open or create a site first.".to_string())
}

// Re-export core implementation functions for test and external use
pub use content::{
    check_image_size, delete_post, list_content, list_content_impl, read_post, read_post_impl,
    save_image, save_image_bytes, save_image_impl, save_post, save_post_impl,
};
pub use models::{
    delete_model, delete_model_impl, list_models, save_model, save_model_impl, save_model_yaml,
};
pub use settings::{
    get_llm_key, get_llm_settings, list_llm_providers, log_frontend,
    save_llm_settings, test_llm_connection,
};
pub use site::{
    build_site, create_site, create_site_impl, export_site, export_site_impl, get_preview_revision,
    get_recent_sites, get_site_config, open_site, open_site_impl, publish_github_pages,
    save_site_config, start_preview,
};
pub use templates::{
    get_template_context, list_templates, read_template, read_theme, revert_template,
    revert_template_impl, save_template, save_template_impl, save_theme,
};
pub use vibe::{evaluate_vibe_proposal_cmd, get_vibe_context};
