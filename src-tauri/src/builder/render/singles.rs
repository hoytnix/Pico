use super::common::{maybe_inject_live_reload, write_atomic_page};
use crate::builder::post::BuiltPost;
use crate::models::{BuildError, ModelDef, SiteConfig};
use minijinja::{context, Environment};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// 5.1 Single pages (`content/<collection>/<slug>/` -> `out/<collection>/<slug>/index.html`)
pub fn render_single_pages(
    out_tmp: &Path,
    env: &Environment,
    site_config: &SiteConfig,
    models: &[ModelDef],
    collection_posts: &HashMap<String, Vec<BuiltPost>>,
    all_tags_list: &[String],
    include_drafts: bool,
    inject_live_reload: bool,
    pages_built: &mut usize,
    sitemap_entries: &mut Vec<(String, String)>,
    search_entries: &mut Vec<Value>,
    errors: &mut Vec<BuildError>,
) {
    for model in models {
        let posts = collection_posts.get(&model.name).cloned().unwrap_or_default();
        let all_vals: Vec<Value> = posts.iter().map(|p| p.to_template_value()).collect();

        for post in &posts {
            let post_val = post.to_template_value();
            let render_res = env.get_template(&model.template).and_then(|tmpl| {
                tmpl.render(context! {
                    site => site_config,
                    post => &post_val,
                    project => &post_val,
                    doc => &post_val,
                    item => &post_val,
                    all_posts => &all_vals,
                    collection => &model.label,
                    tags => all_tags_list,
                    is_preview => include_drafts,
                })
            });

            match render_res {
                Ok(html_str) => {
                    let html_final = maybe_inject_live_reload(html_str, inject_live_reload);
                    let page_dir = out_tmp.join(&post.collection).join(&post.slug);
                    if write_atomic_page(&page_dir, &html_final, &post.source_rel_path, errors) {
                        *pages_built += 1;
                        sitemap_entries.push((post.url.clone(), post.date.clone()));
                        search_entries.push(serde_json::json!({
                            "title": post.title,
                            "url": post.url,
                            "excerpt": post.excerpt,
                            "tags": post.tags,
                            "body": post.body_text,
                        }));
                    }
                }
                Err(err) => {
                    errors.push(BuildError {
                        file: format!("templates/{}", model.template),
                        line: err.line().unwrap_or(1),
                        message: format!(
                            "Error rendering {} with {}: {}",
                            post.source_rel_path, model.template, err
                        ),
                    });
                }
            }
        }
    }
}
