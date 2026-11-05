use super::common::maybe_inject_live_reload;
use crate::builder::pagination::paginate_chunks;
use crate::builder::post::BuiltPost;
use crate::models::{BuildError, ModelDef, SiteConfig};
use minijinja::{context, Environment};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 5.4 Home page & 5.5 404 page
pub fn render_home_and_404_pages(
    out_tmp: &Path,
    env: &Environment,
    site_config: &SiteConfig,
    models: &[ModelDef],
    templates: &HashMap<String, String>,
    collection_posts: &HashMap<String, Vec<BuiltPost>>,
    collection_folders: &HashMap<String, String>,
    all_posts_flat: &[BuiltPost],
    all_tags_list: &[String],
    include_drafts: bool,
    inject_live_reload: bool,
    pages_built: &mut usize,
    sitemap_entries: &mut Vec<(String, String)>,
    errors: &mut Vec<BuildError>,
) {
    let home_tpl_name = if templates.contains_key("home.jinja") {
        "home.jinja"
    } else if let Some(first_m) = models.first() {
        first_m.list_template.as_str()
    } else {
        "blog-list.jinja"
    };

    let first_col_posts: Vec<BuiltPost> = models
        .first()
        .and_then(|m| collection_posts.get(&m.name))
        .cloned()
        .unwrap_or_default();
    let home_posts_vals: Vec<Value> = first_col_posts
        .iter()
        .take(10)
        .map(|p| p.to_template_value())
        .collect();
    let all_posts_vals: Vec<Value> = all_posts_flat
        .iter()
        .map(|p| p.to_template_value())
        .collect();
    let project_vals: Vec<Value> = collection_posts
        .get("project")
        .or_else(|| collection_posts.get("projects"))
        .map(|ps| ps.iter().map(|p| p.to_template_value()).collect())
        .unwrap_or_default();

    let total_home_pages = paginate_chunks(&first_col_posts, 10).len().max(1);
    let first_folder = models
        .first()
        .and_then(|m| collection_folders.get(&m.name))
        .cloned()
        .unwrap_or_else(|| "blog".to_string());
    let home_next_url = if total_home_pages > 1 {
        format!("/{}/page/2/", first_folder)
    } else {
        String::new()
    };

    if let Ok(tmpl) = env.get_template(home_tpl_name) {
        match tmpl.render(context! {
            site => site_config,
            posts => &home_posts_vals,
            projects => &project_vals,
            all_posts => &all_posts_vals,
            items => &home_posts_vals,
            page => 1,
            total_pages => total_home_pages,
            prev_url => "",
            next_url => &home_next_url,
            collection => &site_config.title,
            tags => all_tags_list,
            is_preview => include_drafts,
        }) {
            Ok(html_str) => {
                let html_final = maybe_inject_live_reload(html_str, inject_live_reload);
                let _ = fs::write(out_tmp.join("index.html"), html_final);
                *pages_built += 1;
                let lastmod = first_col_posts
                    .first()
                    .map(|p| p.date.clone())
                    .unwrap_or_else(|| "2026-11-03".to_string());
                sitemap_entries.push(("/".to_string(), lastmod));
            }
            Err(err) => {
                errors.push(BuildError {
                    file: format!("templates/{}", home_tpl_name),
                    line: err.line().unwrap_or(1),
                    message: format!("Error rendering home page: {}", err),
                });
            }
        }
    }

    // 5.5 404.html
    let html_404 = if templates.contains_key("404.jinja") {
        match env.get_template("404.jinja").and_then(|t| {
            t.render(context! {
                site => site_config,
                is_preview => include_drafts,
            })
        }) {
            Ok(s) => s,
            Err(err) => {
                errors.push(BuildError {
                    file: "templates/404.jinja".to_string(),
                    line: err.line().unwrap_or(1),
                    message: format!("Error rendering 404.jinja: {}", err),
                });
                String::new()
            }
        }
    } else {
        format!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>404 — {}</title><link rel=\"stylesheet\" href=\"/assets/theme.css\"></head><body><main class=\"site-main\"><h1>404 — Page Not Found</h1><p><a href=\"/\">Return Home</a></p></main></body></html>",
            site_config.title
        )
    };
    if !html_404.is_empty() {
        let _ = fs::write(
            out_tmp.join("404.html"),
            maybe_inject_live_reload(html_404, inject_live_reload),
        );
        *pages_built += 1;
    }
}
