use tauri::State;

use crate::models::{
    AppState, GroundingSource, VibeContext, VibeResult, VibeSiteSnapshot, VibeTarget,
};
use crate::vibe::{
    build_vibe_system_prompt, evaluate_vibe_proposal_with_sources, load_vibe_site_snapshot,
    read_vibe_target_content,
};
use super::require_current_site;

#[tauri::command]
pub fn get_vibe_site_snapshot(state: State<'_, AppState>) -> Result<VibeSiteSnapshot, String> {
    let site_root = require_current_site(&state)?;
    log::debug!(target: "pico::vibe", "get_vibe_site_snapshot requested for site: {:?}", site_root);
    load_vibe_site_snapshot(&site_root)
}

#[tauri::command]
pub fn get_vibe_context(target: VibeTarget, state: State<'_, AppState>) -> Result<VibeContext, String> {
    let site_root = require_current_site(&state)?;
    log::debug!(target: "pico::vibe", "get_vibe_context requested for target: {:?}", target);
    let original_content = read_vibe_target_content(&site_root, &target)?;
    let system_prompt = build_vibe_system_prompt(&site_root, &target)?;
    log::debug!(target: "pico::vibe", "get_vibe_context loaded: original length={} chars, system prompt length={} chars", original_content.len(), system_prompt.len());
    Ok(VibeContext {
        original_content,
        system_prompt,
    })
}

#[tauri::command]
pub fn evaluate_vibe_proposal_cmd(
    target: VibeTarget,
    proposed: String,
    sources: Option<Vec<GroundingSource>>,
    state: State<'_, AppState>,
) -> Result<VibeResult, String> {
    let site_root = require_current_site(&state)?;
    log::info!(target: "pico::vibe", "evaluate_vibe_proposal_cmd: target={:?}, proposed length={} chars, sources count={}", target, proposed.len(), sources.as_ref().map(|s| s.len()).unwrap_or(0));
    let original_content = read_vibe_target_content(&site_root, &target)?;
    let res = evaluate_vibe_proposal_with_sources(
        &site_root,
        &target,
        &original_content,
        &proposed,
        sources,
    )?;
    log::info!(target: "pico::vibe", "evaluate_vibe_proposal_cmd succeeded: target={:?}, diff length={} chars", target, res.diff.len());
    Ok(res)
}
