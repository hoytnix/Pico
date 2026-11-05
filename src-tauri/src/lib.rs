pub mod builder;
pub mod commands;
pub mod llm;
pub mod logger;
pub mod models;
pub mod preview;
pub mod publish;
pub mod starters;
pub mod storage;
pub mod validation;
pub mod vibe;

pub use commands::*;
pub use starters::{
    copy_starter_pack, extract_embedded_starter, resolve_starters_dir, EMBEDDED_STARTERS,
};
pub use storage::{
    app_config_file_path, load_persisted_llm_settings, load_persisted_settings, record_recent_site,
    save_persisted_llm_settings, save_persisted_settings,
};

use crate::models::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    crate::logger::init_logger();
    log::info!(target: "pico::main", "Starting Pico Desktop Application (v{})...", env!("CARGO_PKG_VERSION"));
    log::info!(target: "pico::main", "Log file configured at: {:?}", crate::logger::log_file_path());

    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            log_frontend,
            open_site,
            create_site,
            get_recent_sites,
            list_content,
            read_post,
            save_post,
            delete_post,
            save_image,
            save_image_bytes,
            check_image_size,
            list_models,
            save_model,
            save_model_yaml,
            delete_model,
            list_templates,
            read_template,
            save_template,
            revert_template,
            get_template_context,
            read_theme,
            save_theme,
            build_site,
            start_preview,
            get_preview_revision,
            export_site,
            publish_github_pages,
            get_site_config,
            save_site_config,
            get_app_settings,
            save_app_settings,
            list_llm_providers,
            get_llm_settings,
            save_llm_settings,
            get_llm_key,
            test_llm_connection,
            get_vibe_context,
            evaluate_vibe_proposal_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Pico application");
}

// ============================================================================
// Verification & Unit / Integration Tests (§11)
// ============================================================================
#[cfg(test)]
mod tests;
