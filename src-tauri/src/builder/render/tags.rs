use super::common::{calculate_pagination_urls, maybe_inject_live_reload, write_atomic_page};
use crate::builder::pagination::paginate_chunks;
use crate::builder::post::BuiltPost;
use crate::models::{BuildError, SiteConfig};
use minijinja::{context, Environment};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// 5.3 Tag pages (`/tags/<tag>/`, `/tags/<tag>/page/2/`)
pub fn render_tag_pages(
    out_tmp: &Path,
    env: &Environment,
    site_config: &SiteConfig,
    templates: &HashMap<String, String>,
    all_tags_map: &BTreeMap<String, Vec<BuiltPost>>,
    all_tags_list: &[String],
    include_drafts: bool,
    inject_live_reload: bool,
    pages_built: &mut usize,
    sitemap_entries: &mut Vec<(String, String)>,
    errors: &mut Vec<BuildError>,
) {
    let tag_tpl_name = if templates.contains_key("tag.jinja") {
        "tag.jinja"
    } else if templates.contains_key("blog-list.jinja") {
        "blog-list.jinja"
    } else {
        "blog.jinja"
    };

    for (tag_name, tag_posts_owned) in all_tags_map {
        let chunks = paginate_chunks(tag_posts_owned, 10);
        let total_pages = chunks.len().max(1);
        let pages_to_render: Vec<Vec<BuiltPost>> = if chunks.is_empty() {
            vec![Vec::new()]
        } else {
            chunks
        };

        for (idx, chunk) in pages_to_render.iter().enumerate() {
            let page_num = idx + 1;
            let tag_base_folder = format!("tags/{}", tag_name);
            let (prev_url, next_url) =
                calculate_pagination_urls(&tag_base_folder, page_num, total_pages);
            let chunk_vals: Vec<Value> = chunk.iter().map(|p| p.to_template_value()).collect();

            let render_res = env.get_template(tag_tpl_name).and_then(|tmpl| {
                tmpl.render(context! {
                    site => site_config,
                    tag => tag_name,
                    posts => &chunk_vals,
                    items => &chunk_vals,
                    page => page_num,
                    total_pages => total_pages,
                    prev_url => &prev_url,
                    next_url => &next_url,
                    collection => tag_name,
                    tags => all_tags_list,
                    is_preview => include_drafts,
                })
            });

            match render_res {
                Ok(html_str) => {
                    let html_final = maybe_inject_live_reload(html_str, inject_live_reload);
                    let (target_dir, url_path) = if page_num == 1 {
                        (
                            out_tmp.join("tags").join(tag_name),
                            format!("/tags/{}/", tag_name),
                        )
                    } else {
                        (
                            out_tmp
                                .join("tags")
                                .join(tag_name)
                                .join("page")
                                .join(page_num.to_string()),
                            format!("/tags/{}/page/{}/", tag_name, page_num),
                        )
                    };
                    let tpl_source = format!("templates/{}", tag_tpl_name);
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
                        file: format!("templates/{}", tag_tpl_name),
                        line: err.line().unwrap_or(1),
                        message: format!("Error rendering tag page {}: {}", tag_name, err),
                    });
                }
            }
        }
    }
}
