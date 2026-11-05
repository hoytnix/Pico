use super::common::{calculate_pagination_urls, maybe_inject_live_reload, write_atomic_page};
use crate::builder::pagination::paginate_chunks;
use crate::builder::post::BuiltPost;
use crate::models::{BuildError, ModelDef, SiteConfig};
use minijinja::{context, Environment};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// 5.2 Collection list pages (`/blog/`, `/blog/page/2/`, etc.)
pub fn render_collection_list_pages(
    out_tmp: &Path,
    env: &Environment,
    site_config: &SiteConfig,
    models: &[ModelDef],
    templates: &HashMap<String, String>,
    collection_posts: &HashMap<String, Vec<BuiltPost>>,
    collection_folders: &HashMap<String, String>,
    all_tags_list: &[String],
    include_drafts: bool,
    inject_live_reload: bool,
    pages_built: &mut usize,
    sitemap_entries: &mut Vec<(String, String)>,
    errors: &mut Vec<BuildError>,
) {
    for model in models {
        let folder_name = collection_folders
            .get(&model.name)
            .cloned()
            .unwrap_or_else(|| model.name.clone());
        let posts = collection_posts.get(&model.name).cloned().unwrap_or_default();
        let chunks = paginate_chunks(&posts, 10);
        let total_pages = chunks.len().max(1);
        let pages_to_render: Vec<Vec<BuiltPost>> = if chunks.is_empty() {
            vec![Vec::new()]
        } else {
            chunks
        };

        for (idx, chunk) in pages_to_render.iter().enumerate() {
            let page_num = idx + 1;
            let (prev_url, next_url) =
                calculate_pagination_urls(&folder_name, page_num, total_pages);
            let chunk_vals: Vec<Value> = chunk.iter().map(|p| p.to_template_value()).collect();
            let list_tpl_name = if templates.contains_key(&model.list_template) {
                model.list_template.as_str()
            } else {
                "blog-list.jinja"
            };

            let render_res = env.get_template(list_tpl_name).and_then(|tmpl| {
                tmpl.render(context! {
                    site => site_config,
                    posts => &chunk_vals,
                    projects => &chunk_vals,
                    items => &chunk_vals,
                    page => page_num,
                    total_pages => total_pages,
                    prev_url => &prev_url,
                    next_url => &next_url,
                    collection => &model.label,
                    tags => all_tags_list,
                    is_preview => include_drafts,
                })
            });

            match render_res {
                Ok(html_str) => {
                    let html_final = maybe_inject_live_reload(html_str, inject_live_reload);
                    let (target_dir, url_path) = if page_num == 1 {
                        (out_tmp.join(&folder_name), format!("/{}/", folder_name))
                    } else {
                        (
                            out_tmp
                                .join(&folder_name)
                                .join("page")
                                .join(page_num.to_string()),
                            format!("/{}/page/{}/", folder_name, page_num),
                        )
                    };
                    let tpl_source = format!("templates/{}", list_tpl_name);
                    if write_atomic_page(&target_dir, &html_final, &tpl_source, errors) {
                        *pages_built += 1;
                        let lastmod = chunk
                            .first()
                            .map(|p| p.date.clone())
                            .unwrap_or_else(|| "2026-11-03".to_string());
                        sitemap_entries.push((url_path, lastmod));
                    }
                }
                Err(err) => {
                    errors.push(BuildError {
                        file: format!("templates/{}", list_tpl_name),
                        line: err.line().unwrap_or(1),
                        message: format!("Error rendering list page {}: {}", list_tpl_name, err),
                    });
                }
            }
        }
    }
}
