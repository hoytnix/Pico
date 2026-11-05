use crate::builder::pagination::{group_by_tag, paginate_chunks};
use crate::builder::post::BuiltPost;
use crate::models::{BuildError, ModelDef, SiteConfig};
use minijinja::{context, AutoEscape, Environment};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct RenderedPagesResult {
    pub pages_built: usize,
    pub sitemap_entries: Vec<(String, String)>,
    pub search_entries: Vec<Value>,
    pub errors: Vec<BuildError>,
}

pub fn maybe_inject_live_reload(html: String, inject: bool) -> String {
    if !inject {
        return html;
    }
    let script = r#"<script>
(function() {
  var rev = null;
  function check() {
    fetch('/__pico_livereload')
      .then(function(r) { return r.text(); })
      .then(function(txt) {
        var r = parseInt(txt, 10);
        if (rev === null) {
          rev = r;
        } else if (r > rev) {
          window.location.reload();
        }
      })
      .catch(function() {});
  }
  setInterval(check, 400);
})();
</script>"#;

    if let Some(pos) = html.to_lowercase().rfind("</body>") {
        let mut out = html;
        out.insert_str(pos, script);
        out
    } else {
        format!("{}{}", html, script)
    }
}

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

    // 5.1 Single pages (`content/<collection>/<slug>/` -> `out/<collection>/<slug>/index.html`)
    for model in models {
        let posts = collection_posts.get(&model.name).cloned().unwrap_or_default();
        let all_vals: Vec<Value> = posts.iter().map(|p| p.to_template_value()).collect();

        for post in &posts {
            let post_val = post.to_template_value();
            let render_res = env
                .get_template(&model.template)
                .and_then(|tmpl| {
                    tmpl.render(context! {
                        site => site_config,
                        post => &post_val,
                        project => &post_val,
                        doc => &post_val,
                        item => &post_val,
                        all_posts => &all_vals,
                        collection => &model.label,
                        tags => &all_tags_list,
                        is_preview => include_drafts,
                    })
                });

            match render_res {
                Ok(html_str) => {
                    let html_final = maybe_inject_live_reload(html_str, inject_live_reload);
                    let page_dir = out_tmp.join(&post.collection).join(&post.slug);
                    if let Err(e) = fs::create_dir_all(&page_dir) {
                        errors.push(BuildError {
                            file: post.source_rel_path.clone(),
                            line: 1,
                            message: format!("Failed to create output folder: {}", e),
                        });
                        continue;
                    }
                    if let Err(e) = fs::write(page_dir.join("index.html"), html_final) {
                        errors.push(BuildError {
                            file: post.source_rel_path.clone(),
                            line: 1,
                            message: format!("Failed to write index.html: {}", e),
                        });
                        continue;
                    }
                    pages_built += 1;
                    sitemap_entries.push((post.url.clone(), post.date.clone()));
                    search_entries.push(serde_json::json!({
                        "title": post.title,
                        "url": post.url,
                        "excerpt": post.excerpt,
                        "tags": post.tags,
                        "body": post.body_text,
                    }));
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

    // 5.2 Collection list pages (`/blog/`, `/blog/page/2/`, etc.)
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
            let prev_url = if page_num == 1 {
                String::new()
            } else if page_num == 2 {
                format!("/{}/", folder_name)
            } else {
                format!("/{}/page/{}/", folder_name, page_num - 1)
            };
            let next_url = if page_num < total_pages {
                format!("/{}/page/{}/", folder_name, page_num + 1)
            } else {
                String::new()
            };

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
                    tags => &all_tags_list,
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
                    if let Err(e) = fs::create_dir_all(&target_dir) {
                        errors.push(BuildError {
                            file: format!("templates/{}", list_tpl_name),
                            line: 1,
                            message: format!("Failed to create list page directory: {}", e),
                        });
                        continue;
                    }
                    if let Err(e) = fs::write(target_dir.join("index.html"), html_final) {
                        errors.push(BuildError {
                            file: format!("templates/{}", list_tpl_name),
                            line: 1,
                            message: format!("Failed to write list index.html: {}", e),
                        });
                        continue;
                    }
                    pages_built += 1;
                    let lastmod = chunk
                        .first()
                        .map(|p| p.date.clone())
                        .unwrap_or_else(|| "2026-11-03".to_string());
                    sitemap_entries.push((url_path, lastmod));
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

    // 5.3 Tag pages (`/tags/<tag>/`, `/tags/<tag>/page/2/`)
    let tag_tpl_name = if templates.contains_key("tag.jinja") {
        "tag.jinja"
    } else if let Some(first_m) = models.first() {
        first_m.list_template.as_str()
    } else {
        "blog-list.jinja"
    };

    for (tag_name, mut tag_posts) in all_tags_map {
        tag_posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
        let chunks = paginate_chunks(&tag_posts, 10);
        let total_pages = chunks.len().max(1);
        for (idx, chunk) in chunks.iter().enumerate() {
            let page_num = idx + 1;
            let prev_url = if page_num == 1 {
                String::new()
            } else if page_num == 2 {
                format!("/tags/{}/", tag_name)
            } else {
                format!("/tags/{}/page/{}/", tag_name, page_num - 1)
            };
            let next_url = if page_num < total_pages {
                format!("/tags/{}/page/{}/", tag_name, page_num + 1)
            } else {
                String::new()
            };
            let chunk_vals: Vec<Value> = chunk.iter().map(|p| p.to_template_value()).collect();

            if let Ok(tmpl) = env.get_template(tag_tpl_name) {
                match tmpl.render(context! {
                    site => site_config,
                    tag => &tag_name,
                    posts => &chunk_vals,
                    items => &chunk_vals,
                    page => page_num,
                    total_pages => total_pages,
                    prev_url => &prev_url,
                    next_url => &next_url,
                    collection => format!("#{}", tag_name),
                    tags => &all_tags_list,
                    is_preview => include_drafts,
                }) {
                    Ok(html_str) => {
                        let html_final = maybe_inject_live_reload(html_str, inject_live_reload);
                        let (target_dir, url_path) = if page_num == 1 {
                            (
                                out_tmp.join("tags").join(&tag_name),
                                format!("/tags/{}/", tag_name),
                            )
                        } else {
                            (
                                out_tmp
                                    .join("tags")
                                    .join(&tag_name)
                                    .join("page")
                                    .join(page_num.to_string()),
                                format!("/tags/{}/page/{}/", tag_name, page_num),
                            )
                        };
                        let _ = fs::create_dir_all(&target_dir);
                        let _ = fs::write(target_dir.join("index.html"), html_final);
                        pages_built += 1;
                        let lastmod = chunk
                            .first()
                            .map(|p| p.date.clone())
                            .unwrap_or_else(|| "2026-11-03".to_string());
                        sitemap_entries.push((url_path, lastmod));
                    }
                    Err(err) => {
                        errors.push(BuildError {
                            file: format!("templates/{}", tag_tpl_name),
                            line: err.line().unwrap_or(1),
                            message: format!("Error rendering tag page #{}: {}", tag_name, err),
                        });
                    }
                }
            }
        }
    }

    // 5.4 Home page (`templates/home.jinja` if present, otherwise first collection list)
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
            tags => &all_tags_list,
            is_preview => include_drafts,
        }) {
            Ok(html_str) => {
                let html_final = maybe_inject_live_reload(html_str, inject_live_reload);
                let _ = fs::write(out_tmp.join("index.html"), html_final);
                pages_built += 1;
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
        pages_built += 1;
    }

    RenderedPagesResult {
        pages_built,
        sitemap_entries,
        search_entries,
        errors,
    }
}
