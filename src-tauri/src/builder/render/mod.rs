pub mod collections;
pub mod common;
pub mod home_and_404;
pub mod singles;
pub mod tags;

pub use common::{maybe_inject_live_reload, RenderedPagesResult};

use collections::render_collection_list_pages;
use home_and_404::render_home_and_404_pages;
use singles::render_single_pages;
use tags::render_tag_pages;

use crate::builder::pagination::group_by_tag;
use crate::builder::post::BuiltPost;
use crate::models::{BuildError, ModelDef, SiteConfig};
use minijinja::{AutoEscape, Environment};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub fn render_site_templates(
    out_tmp: &Path,
    site_config: &SiteConfig,
    models: &[ModelDef],
    templates: &HashMap<String, String>,
    collection_posts: &HashMap<String, Vec<BuiltPost>>,
    collection_folders: &HashMap<String, String>,
    include_drafts: bool,
    inject_live_reload: bool,
) -> RenderedPagesResult {
    let mut errors: Vec<BuildError> = Vec::new();
    let mut pages_built = 0usize;
    let mut sitemap_entries: Vec<(String, String)> = Vec::new();
    let mut search_entries: Vec<Value> = Vec::new();

    // Prepare MiniJinja Environment
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| AutoEscape::Html);
    env.set_fuel(Some(50_000));
    for (tpl_name, tpl_src) in templates {
        if let Err(err) = env.add_template_owned(tpl_name.clone(), tpl_src.clone()) {
            errors.push(BuildError {
                file: format!("templates/{}", tpl_name),
                line: err.line().unwrap_or(1),
                message: format!("Failed to load template: {}", err),
            });
        }
    }

    let mut all_posts_flat: Vec<BuiltPost> = Vec::new();
    for model in models {
        if let Some(posts) = collection_posts.get(&model.name) {
            for p in posts {
                all_posts_flat.push(p.clone());
            }
        }
    }

    let all_tags_map = group_by_tag(&all_posts_flat, |p| &p.tags);
    let all_tags_list: Vec<String> = all_tags_map.keys().cloned().collect();

    // 5.1 Single pages
    render_single_pages(
        out_tmp,
        &env,
        site_config,
        models,
        collection_posts,
        &all_tags_list,
        include_drafts,
        inject_live_reload,
        &mut pages_built,
        &mut sitemap_entries,
        &mut search_entries,
        &mut errors,
    );

    // 5.2 Collection list pages
    render_collection_list_pages(
        out_tmp,
        &env,
        site_config,
        models,
        templates,
        collection_posts,
        collection_folders,
        &all_tags_list,
        include_drafts,
        inject_live_reload,
        &mut pages_built,
        &mut sitemap_entries,
        &mut errors,
    );

    // 5.3 Tag pages
    render_tag_pages(
        out_tmp,
        &env,
        site_config,
        templates,
        &all_tags_map,
        &all_tags_list,
        include_drafts,
        inject_live_reload,
        &mut pages_built,
        &mut sitemap_entries,
        &mut errors,
    );

    // 5.4 Home and 5.5 404 pages
    render_home_and_404_pages(
        out_tmp,
        &env,
        site_config,
        models,
        templates,
        collection_posts,
        collection_folders,
        &all_posts_flat,
        &all_tags_list,
        include_drafts,
        inject_live_reload,
        &mut pages_built,
        &mut sitemap_entries,
        &mut errors,
    );

    RenderedPagesResult {
        pages_built,
        sitemap_entries,
        search_entries,
        errors,
    }
}
