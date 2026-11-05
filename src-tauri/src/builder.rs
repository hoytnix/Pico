use crate::models::{BuildError, BuildResult, ModelDef, PostMeta, SiteConfig};
use crate::validation::{
    check_duplicate_slugs, parse_markdown_document, slugify, validate_frontmatter,
    validate_template_with_models,
};
use image::imageops::FilterType;
use minijinja::{context, AutoEscape, Environment};
use pulldown_cmark::{ html, Event, Options, Parser, Tag, TagEnd };
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltPost {
    pub title: String,
    pub date: String,
    pub slug: String,
    pub draft: bool,
    pub excerpt: String,
    pub hero: String,
    pub hero_webp: String,
    pub thumb_webp: String,
    pub tags: Vec<String>,
    pub url: String,
    pub collection: String,
    pub body: String,
    pub body_html: String,
    pub body_text: String,
    pub attachments: Vec<String>,
    pub source_rel_path: String,
    pub markdown_images: Vec<String>,
    pub extra_fields: Map<String, Value>,
}

impl BuiltPost {
    pub fn to_template_value(&self) -> Value {
        let mut map = self.extra_fields.clone();
        map.insert("title".to_string(), Value::String(self.title.clone()));
        map.insert("date".to_string(), Value::String(self.date.clone()));
        map.insert("slug".to_string(), Value::String(self.slug.clone()));
        map.insert("draft".to_string(), Value::Bool(self.draft));
        map.insert("excerpt".to_string(), Value::String(self.excerpt.clone()));
        map.insert("hero".to_string(), Value::String(self.hero.clone()));
        map.insert("hero_webp".to_string(), Value::String(self.hero_webp.clone()));
        map.insert("thumb_webp".to_string(), Value::String(self.thumb_webp.clone()));
        map.insert(
            "tags".to_string(),
            Value::Array(
                self.tags
                    .iter()
                    .map(|t| Value::String(t.clone()))
                    .collect(),
            ),
        );
        map.insert("url".to_string(), Value::String(self.url.clone()));
        map.insert("collection".to_string(), Value::String(self.collection.clone()));
        map.insert("body".to_string(), Value::String(self.body.clone()));
        map.insert("body_html".to_string(), Value::String(self.body_html.clone()));
        map.insert(
            "attachments".to_string(),
            Value::Array(
                self.attachments
                    .iter()
                    .map(|a| Value::String(a.clone()))
                    .collect(),
            ),
        );
        Value::Object(map)
    }

    pub fn to_post_meta(&self) -> PostMeta {
        PostMeta {
            title: self.title.clone(),
            date: self.date.clone(),
            slug: self.slug.clone(),
            draft: self.draft,
            path: self.source_rel_path.clone(),
            url: self.url.clone(),
            collection: self.collection.clone(),
            excerpt: self.excerpt.clone(),
            tags: self.tags.clone(),
            hero: self.hero.clone(),
        }
    }
}

/// Chunks a slice of items into pages of `page_size` (e.g. 10).
/// Returns 0 chunks for 0 items, 1 chunk for 1..=10 items, 2 chunks for 11..=20 items, etc.
pub fn paginate_chunks<T: Clone>(items: &[T], page_size: usize) -> Vec<Vec<T>> {
    let step = page_size.max(1);
    if items.is_empty() {
        return Vec::new();
    }
    items.chunks(step).map(|c| c.to_vec()).collect()
}

/// Groups items by tag in deterministic sorted order.
pub fn group_by_tag<T: Clone, F: Fn(&T) -> &[String]>(
    items: &[T],
    get_tags: F,
) -> BTreeMap<String, Vec<T>> {
    let mut grouped: BTreeMap<String, Vec<T>> = BTreeMap::new();
    for item in items {
        for tag in get_tags(item) {
            let trimmed = tag.trim();
            if !trimmed.is_empty() {
                grouped
                    .entry(trimmed.to_string())
                    .or_default()
                    .push(item.clone());
            }
        }
    }
    grouped
}

/// Derives the hero (1200px) and card thumbnail (600px) WebP variant paths for a source JPEG/PNG path.
pub fn derive_image_variant_paths(rel_image_path: &str) -> Option<(String, String)> {
    let path = Path::new(rel_image_path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())?;

    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
        return None;
    }

    let stem = path.file_stem()?.to_str()?;
    let parent = path
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .replace('\\', "/");

    if parent.is_empty() {
        Some((format!("{}-hero.webp", stem), format!("{}-thumb.webp", stem)))
    } else {
        Some((
            format!("{}/{}-hero.webp", parent, stem),
            format!("{}/{}-thumb.webp", parent, stem),
        ))
    }
}

/// Renders Markdown to HTML using pulldown-cmark with GFM extensions (tables, strikethrough, task lists),
/// extracts plain text for `search.json`, and collects referenced Markdown image URLs.
pub fn render_markdown_to_html(markdown: &str) -> (String, String, Vec<String>) {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);
    let mut events = Vec::new();
    let mut plain_text = String::new();
    let mut images = Vec::new();
    let mut in_image = false;

    for event in parser {
        match &event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                in_image = true;
                images.push(dest_url.to_string());
            }
            Event::End(TagEnd::Image) => {
                in_image = false;
            }
            Event::Text(t) | Event::Code(t) => {
                if !in_image {
                    if !plain_text.is_empty() && !plain_text.ends_with(' ') {
                        plain_text.push(' ');
                    }
                    plain_text.push_str(t);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if !plain_text.ends_with(' ') {
                    plain_text.push(' ');
                }
            }
            _ => {}
        }
        events.push(event);
    }

    let mut html_out = String::new();
    html::push_html(&mut html_out, events.into_iter());
    (html_out, plain_text.trim().to_string(), images)
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn generate_sitemap_xml(base_url: &str, entries: &[(String, String)]) -> String {
    let base = base_url.trim_end_matches('/');
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    for (url, lastmod) in entries {
        let loc = if url.starts_with('/') {
            format!("{}{}", base, url)
        } else {
            format!("{}/{}", base, url)
        };
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", xml_escape(&loc)));
        if !lastmod.is_empty() {
            xml.push_str(&format!("    <lastmod>{}</lastmod>\n", xml_escape(lastmod)));
        }
        xml.push_str("  </url>\n");
    }
    xml.push_str("</urlset>\n");
    xml
}

pub fn generate_robots_txt(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    format!("User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n", base)
}

pub fn generate_rss_xml(site: &SiteConfig, posts: &[BuiltPost]) -> String {
    let base = site.base_url.trim_end_matches('/');
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rss version=\"2.0\">\n  <channel>\n");
    xml.push_str(&format!("    <title>{}</title>\n", xml_escape(&site.title)));
    xml.push_str(&format!("    <link>{}/</link>\n", xml_escape(base)));
    xml.push_str(&format!(
        "    <description>{}</description>\n",
        xml_escape(&site.description)
    ));
    for post in posts {
        let link = format!("{}{}", base, post.url);
        xml.push_str("    <item>\n");
        xml.push_str(&format!("      <title>{}</title>\n", xml_escape(&post.title)));
        xml.push_str(&format!("      <link>{}</link>\n", xml_escape(&link)));
        xml.push_str(&format!("      <guid>{}</guid>\n", xml_escape(&link)));
        xml.push_str(&format!("      <pubDate>{}</pubDate>\n", xml_escape(&post.date)));
        xml.push_str(&format!(
            "      <description>{}</description>\n",
            xml_escape(&post.excerpt)
        ));
        xml.push_str("    </item>\n");
    }
    xml.push_str("  </channel>\n</rss>\n");
    xml
}

pub fn generate_atom_xml(site: &SiteConfig, posts: &[BuiltPost]) -> String {
    let base = site.base_url.trim_end_matches('/');
    let updated = posts
        .first()
        .map(|p| format!("{}T00:00:00Z", p.date))
        .unwrap_or_else(|| "2026-11-03T00:00:00Z".to_string());

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    xml.push_str(&format!("  <title>{}</title>\n", xml_escape(&site.title)));
    xml.push_str(&format!("  <link href=\"{}/\"/>\n", xml_escape(base)));
    xml.push_str(&format!("  <id>{}/</id>\n", xml_escape(base)));
    xml.push_str(&format!("  <updated>{}</updated>\n", xml_escape(&updated)));
    xml.push_str(&format!(
        "  <author><name>{}</name></author>\n",
        xml_escape(&site.author)
    ));
    for post in posts {
        let link = format!("{}{}", base, post.url);
        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", xml_escape(&post.title)));
        xml.push_str(&format!("    <link href=\"{}\"/>\n", xml_escape(&link)));
        xml.push_str(&format!("    <id>{}</id>\n", xml_escape(&link)));
        xml.push_str(&format!(
            "    <updated>{}T00:00:00Z</updated>\n",
            xml_escape(&post.date)
        ));
        xml.push_str(&format!(
            "    <summary>{}</summary>\n",
            xml_escape(&post.excerpt)
        ));
        xml.push_str("  </entry>\n");
    }
    xml.push_str("</feed>\n");
    xml
}

/// Resolves the content subdirectory for a given model (e.g. `blog` -> `content/blog`, `project` -> `content/projects` or `content/project`).
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

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize, String> {
    if !src.exists() {
        return Ok(0);
    }
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create {}: {}", dst.display(), e))?;
    let mut copied = 0usize;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        let rel = p.strip_prefix(src).map_err(|e| e.to_string())?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(p, &target).map_err(|e| e.to_string())?;
            copied += 1;
        }
    }
    Ok(copied)
}

fn generate_webp_variants_for_file(
    src_path: &Path,
    out_dir_for_image: &Path,
    rel_display: &str,
) -> Result<usize, BuildError> {
    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    // SVG, GIF, AVIF are copied unchanged in MVP (§6.7)
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
        return Ok(0);
    }

    let stem = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    let img = image::open(src_path).map_err(|e| BuildError {
        file: rel_display.to_string(),
        line: 1,
        message: format!("Failed to decode image '{}': {}", rel_display, e),
    })?;

    fs::create_dir_all(out_dir_for_image).map_err(|e| BuildError {
        file: rel_display.to_string(),
        line: 1,
        message: format!("Failed to create output image folder: {}", e),
    })?;

    // 1. Hero variant: max 1200px wide (Lanczos3)
    let hero_img = if img.width() > 1200 {
        img.resize(1200, u32::MAX, FilterType::Lanczos3)
    } else {
        img.clone()
    };
    let hero_path = out_dir_for_image.join(format!("{}-hero.webp", stem));
    hero_img
        .save_with_format(&hero_path, image::ImageFormat::WebP)
        .map_err(|e| BuildError {
            file: rel_display.to_string(),
            line: 1,
            message: format!("Failed to write WebP hero variant for '{}': {}", rel_display, e),
        })?;

    // Also save `<stem>.webp` so direct `hero.webp` links work seamlessly
    let direct_webp = out_dir_for_image.join(format!("{}.webp", stem));
    if !direct_webp.exists() {
        let _ = hero_img.save_with_format(&direct_webp, image::ImageFormat::WebP);
    }

    // 2. Card thumb variant: max 600px wide (Lanczos3)
    let thumb_img = if img.width() > 600 {
        img.resize(600, u32::MAX, FilterType::Lanczos3)
    } else {
        img
    };
    let thumb_path = out_dir_for_image.join(format!("{}-thumb.webp", stem));
    thumb_img
        .save_with_format(&thumb_path, image::ImageFormat::WebP)
        .map_err(|e| BuildError {
            file: rel_display.to_string(),
            line: 1,
            message: format!(
                "Failed to write WebP thumbnail variant for '{}': {}",
                rel_display, e
            ),
        })?;

    Ok(2)
}

fn maybe_inject_live_reload(html: String, inject: bool) -> String {
    if !inject {
        return html;
    }
    let snippet = r#"<script>
(function(){
  var rev = null;
  setInterval(function(){
    fetch('/__pico_livereload', { cache: 'no-store' })
      .then(function(r){ return r.text(); })
      .then(function(t){
        if (rev === null) { rev = t; }
        else if (t && t !== rev) { window.location.reload(); }
      })
      .catch(function(){});
  }, 500);
})();
</script></body>"#;
    if html.contains("</body>") {
        html.replacen("</body>", snippet, 1)
    } else {
        format!("{}{}", html, snippet)
    }
}

/// Executes the full 10-step Pico build pipeline (§6).
/// Builds to `out.tmp/` first, then swaps to `out/` only if `errors.is_empty()`.
pub fn build_site_pipeline(
    site_root: &Path,
    include_drafts: bool,
    inject_live_reload: bool,
) -> BuildResult {
    log::info!(
        target: "pico::builder",
        "Build pipeline started: site_root={:?}, include_drafts={}, inject_live_reload={}",
        site_root,
        include_drafts,
        inject_live_reload
    );
    let out_final = site_root.join("out");
    let out_tmp = site_root.join("out.tmp");
    let mut errors: Vec<BuildError> = Vec::new();
    let mut log: Vec<String> = Vec::new();
    let mut pages_built = 0usize;
    let mut images_generated = 0usize;

    // Clean any stale out.tmp/
    if out_tmp.exists() {
        let _ = fs::remove_dir_all(&out_tmp);
    }
    if let Err(e) = fs::create_dir_all(&out_tmp) {
        log::error!(target: "pico::builder", "Could not create temporary out.tmp directory: {}", e);
        return BuildResult {
            success: false,
            out_dir: out_final.to_string_lossy().to_string(),
            errors: vec![BuildError {
                file: "out.tmp".to_string(),
                line: 0,
                message: format!("Could not create temporary build directory: {}", e),
            }],
            pages_built: 0,
            images_generated: 0,
            log: vec![format!("Failed to create out.tmp: {}", e)],
        };
    }

    let site_config = match load_site_config(site_root) {
        Ok(cfg) => cfg,
        Err(err) => {
            log::error!(target: "pico::builder", "Failed to load site.yaml: {}", err);
            let _ = fs::remove_dir_all(&out_tmp);
            return BuildResult {
                success: false,
                out_dir: out_final.to_string_lossy().to_string(),
                errors: vec![err],
                pages_built: 0,
                images_generated: 0,
                log: vec!["Failed to load site.yaml".to_string()],
            };
        }
    };

    let models = match load_models(site_root) {
        Ok(m) => m,
        Err(err) => {
            let _ = fs::remove_dir_all(&out_tmp);
            return BuildResult {
                success: false,
                out_dir: out_final.to_string_lossy().to_string(),
                errors: vec![err],
                pages_built: 0,
                images_generated: 0,
                log: vec!["Failed to load models/".to_string()],
            };
        }
    };

    let templates = match load_all_templates(site_root) {
        Ok(t) => t,
        Err(err) => {
            let _ = fs::remove_dir_all(&out_tmp);
            return BuildResult {
                success: false,
                out_dir: out_final.to_string_lossy().to_string(),
                errors: vec![err],
                pages_built: 0,
                images_generated: 0,
                log: vec!["Failed to load templates/".to_string()],
            };
        }
    };

    // Validate all templates upfront
    for (tpl_name, tpl_src) in &templates {
        if let Err(err) = validate_template_with_models(tpl_name, tpl_src, &templates, &models) {
            errors.push(err);
        }
    }

    // STEP 1, 2, 3: Walk content/**/*.md, parse YAML frontmatter + Markdown, validate against model,
    // filter drafts (unless include_drafts), slugify, check duplicate slugs, render Markdown with pulldown-cmark.
    let mut collection_posts: HashMap<String, Vec<BuiltPost>> = HashMap::new();
    let mut collection_folders: HashMap<String, String> = HashMap::new();

    for model in &models {
        let (folder_name, col_dir) = resolve_collection_folder(site_root, &model.name);
        collection_folders.insert(model.name.clone(), folder_name.clone());
        let mut built_for_col: Vec<BuiltPost> = Vec::new();
        let mut slug_entries: Vec<(String, String, usize)> = Vec::new();

        if col_dir.is_dir() {
            let mut md_files: Vec<PathBuf> = fs::read_dir(&col_dir)
                .ok()
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok().map(|d| d.path()))
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|s| s.eq_ignore_ascii_case("md"))
                            .unwrap_or(false)
                })
                .collect();
            md_files.sort();

            for md_path in md_files {
                let rel_path = md_path
                    .strip_prefix(site_root)
                    .unwrap_or(&md_path)
                    .to_string_lossy()
                    .replace('\\', "/");

                let raw = match fs::read_to_string(&md_path) {
                    Ok(s) => s,
                    Err(e) => {
                        errors.push(BuildError {
                            file: rel_path,
                            line: 1,
                            message: format!("Could not read markdown file: {}", e),
                        });
                        continue;
                    }
                };

                let (fm_val, markdown_body, raw_yaml) =
                    match parse_markdown_document(&raw, &rel_path) {
                        Ok(res) => res,
                        Err(err) => {
                            errors.push(err);
                            continue;
                        }
                    };

                if let Err(mut val_errs) =
                    validate_frontmatter(&fm_val, model, &rel_path, Some(&raw_yaml))
                {
                    errors.append(&mut val_errs);
                    continue;
                }

                let obj = fm_val.as_object().unwrap();
                let draft = obj
                    .get("draft")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                // Step 2: Filter drafts unless include_drafts is true
                if draft && !include_drafts {
                    log.push(format!("Skipped draft: {}", rel_path));
                    continue;
                }

                let title = obj
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled")
                    .to_string();
                let date = obj
                    .get("date")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1970-01-01")
                    .to_string();
                let raw_slug = obj
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                let slug = if raw_slug.is_empty() {
                    slugify(&title)
                } else {
                    slugify(raw_slug)
                };

                slug_entries.push((rel_path.clone(), slug.clone(), 2));

                let excerpt = obj
                    .get("excerpt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let hero = obj
                    .get("hero")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let tags: Vec<String> = match obj.get("tags") {
                    Some(Value::Array(arr)) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect(),
                    Some(Value::String(s)) => s
                        .split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect(),
                    _ => Vec::new(),
                };

                // Step 3: Render Markdown body to HTML with pulldown-cmark (GFM)
                let (body_html, body_text, markdown_images) =
                    render_markdown_to_html(&markdown_body);

                let url = format!("/{}/{}/", folder_name, slug);

                // Check page attachments in `content/<folder_name>/<slug>/`
                let attach_dir = col_dir.join(&slug);
                let mut attachments: Vec<String> = Vec::new();
                if attach_dir.is_dir() {
                    let mut att_files: Vec<PathBuf> = fs::read_dir(&attach_dir)
                        .ok()
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok().map(|d| d.path()))
                        .filter(|p| p.is_file())
                        .collect();
                    att_files.sort();
                    for af in att_files {
                        if let Some(fname) = af.file_name().and_then(|n| n.to_str()) {
                            attachments.push(format!("{}{}", url, fname));
                        }
                    }
                }

                // Derive WebP URLs if hero is PNG/JPG
                let (hero_webp, thumb_webp) = if !hero.is_empty() {
                    if let Some((hw, tw)) = derive_image_variant_paths(&hero) {
                        let hw_url = if hw.starts_with('/') || hw.starts_with("http") {
                            hw
                        } else {
                            format!("{}{}", url, hw)
                        };
                        let tw_url = if tw.starts_with('/') || tw.starts_with("http") {
                            tw
                        } else {
                            format!("{}{}", url, tw)
                        };
                        (hw_url, tw_url)
                    } else if hero.ends_with(".webp") {
                        let u = if hero.starts_with('/') || hero.starts_with("http") {
                            hero.clone()
                        } else {
                            format!("{}{}", url, hero)
                        };
                        (u.clone(), u)
                    } else {
                        (String::new(), String::new())
                    }
                } else {
                    (String::new(), String::new())
                };

                built_for_col.push(BuiltPost {
                    title,
                    date,
                    slug,
                    draft,
                    excerpt,
                    hero,
                    hero_webp,
                    thumb_webp,
                    tags,
                    url,
                    collection: folder_name.clone(),
                    body: markdown_body,
                    body_html,
                    body_text,
                    attachments,
                    source_rel_path: rel_path,
                    markdown_images,
                    extra_fields: obj.clone(),
                });
            }
        }

        // Duplicate slug check per collection
        let mut dup_errs = check_duplicate_slugs(&slug_entries);
        errors.append(&mut dup_errs);

        // Step 4: Sort by date descending, then title ascending
        built_for_col.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
        collection_posts.insert(model.name.clone(), built_for_col);
    }

    // Prepare MiniJinja Environment with AutoEscape::Html and Fuel limit (§4.6)
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| AutoEscape::Html);
    env.set_fuel(Some(50_000));
    for (tpl_name, tpl_src) in &templates {
        if let Err(err) = env.add_template_owned(tpl_name.clone(), tpl_src.clone()) {
            errors.push(BuildError {
                file: format!("templates/{}", tpl_name),
                line: err.line().unwrap_or(1),
                message: format!("Failed to load template: {}", err),
            });
        }
    }

    let mut sitemap_entries: Vec<(String, String)> = Vec::new();
    let mut search_entries: Vec<Value> = Vec::new();
    let mut all_posts_flat: Vec<BuiltPost> = Vec::new();

    for model in &models {
        if let Some(posts) = collection_posts.get(&model.name) {
            for p in posts {
                all_posts_flat.push(p.clone());
            }
        }
    }

    let all_tags_map = group_by_tag(&all_posts_flat, |p| &p.tags);
    let all_tags_list: Vec<String> = all_tags_map.keys().cloned().collect();

    // STEP 5: Render with MiniJinja in exact order:
    // 5.1 Single pages (`content/<collection>/<slug>/` -> `out/<collection>/<slug>/index.html`)
    for model in &models {
        let posts = collection_posts.get(&model.name).cloned().unwrap_or_default();
        let all_vals: Vec<Value> = posts.iter().map(|p| p.to_template_value()).collect();

        for post in &posts {
            let post_val = post.to_template_value();
            let render_res = env
                .get_template(&model.template)
                .and_then(|tmpl| {
                    tmpl.render(context! {
                        site => &site_config,
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
    for model in &models {
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
                    site => &site_config,
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
                    site => &site_config,
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
            site => &site_config,
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
                site => &site_config,
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

    // STEP 6: Copy `assets/` -> `out/assets/` unchanged. Copy page attachments -> `out/<collection>/<slug>/`.
    let assets_src = site_root.join("assets");
    let assets_dst = out_tmp.join("assets");
    if assets_src.is_dir() {
        if let Err(e) = copy_dir_recursive(&assets_src, &assets_dst) {
            errors.push(BuildError {
                file: "assets/".to_string(),
                line: 0,
                message: format!("Failed to copy assets/: {}", e),
            });
        }
    }

    for model in &models {
        let (_, col_dir) = resolve_collection_folder(site_root, &model.name);
        if let Some(posts) = collection_posts.get(&model.name) {
            for post in posts {
                let attach_src = col_dir.join(&post.slug);
                if attach_src.is_dir() {
                    let attach_dst = out_tmp.join(&post.collection).join(&post.slug);
                    if let Err(e) = copy_dir_recursive(&attach_src, &attach_dst) {
                        errors.push(BuildError {
                            file: post.source_rel_path.clone(),
                            line: 1,
                            message: format!("Failed to copy page attachments: {}", e),
                        });
                    }
                }
            }
        }
    }

    // STEP 7: Images: for each JPEG/PNG referenced as `hero` or in Markdown (and in assets),
    // generate WebP variants at build time with `image` crate (Lanczos3):
    // hero max 1200px wide, card thumb max 600px wide.
    // Write variants to `out/` only, never back into source.
    let mut processed_images: HashSet<PathBuf> = HashSet::new();

    for model in &models {
        let (_, col_dir) = resolve_collection_folder(site_root, &model.name);
        if let Some(posts) = collection_posts.get(&model.name) {
            for post in posts {
                let mut refs: Vec<String> = Vec::new();
                if !post.hero.is_empty() {
                    refs.push(post.hero.clone());
                }
                for md_img in &post.markdown_images {
                    refs.push(md_img.clone());
                }

                for img_ref in refs {
                    if img_ref.starts_with("http://") || img_ref.starts_with("https://") {
                        continue;
                    }
                    let clean_ref = img_ref.trim_start_matches("./").trim_start_matches('/');
                    let ext = Path::new(clean_ref)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_ascii_lowercase())
                        .unwrap_or_default();

                    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
                        continue;
                    }

                    // Candidate source locations:
                    // 1. content/<col>/<slug>/<clean_ref>
                    // 2. content/<col>/<clean_ref>
                    // 3. site_root/<clean_ref>
                    // 4. assets/<clean_ref>
                    let cand1 = col_dir.join(&post.slug).join(clean_ref);
                    let cand2 = col_dir.join(clean_ref);
                    let cand3 = site_root.join(clean_ref);
                    let cand4 = assets_src.join(clean_ref);

                    let (src_img_path, out_img_dir) = if cand1.is_file() {
                        (cand1, out_tmp.join(&post.collection).join(&post.slug))
                    } else if cand2.is_file() {
                        (cand2, out_tmp.join(&post.collection).join(&post.slug))
                    } else if cand3.is_file() {
                        let parent_rel = Path::new(clean_ref).parent().unwrap_or(Path::new(""));
                        (cand3, out_tmp.join(parent_rel))
                    } else if cand4.is_file() {
                        (cand4, out_tmp.join("assets"))
                    } else {
                        errors.push(BuildError {
                            file: post.source_rel_path.clone(),
                            line: 1,
                            message: format!(
                                "Referenced image '{}' was not found on disk.",
                                img_ref
                            ),
                        });
                        continue;
                    };

                    if processed_images.insert(src_img_path.clone()) {
                        let rel_disp = src_img_path
                            .strip_prefix(site_root)
                            .unwrap_or(&src_img_path)
                            .to_string_lossy()
                            .replace('\\', "/");
                        match generate_webp_variants_for_file(
                            &src_img_path,
                            &out_img_dir,
                            &rel_disp,
                        ) {
                            Ok(count) => {
                                images_generated += count;
                                // Also ensure original image is in the post's out folder if referenced relatively
                                if let Some(fname) = src_img_path.file_name() {
                                    let dest_orig = out_img_dir.join(fname);
                                    if !dest_orig.exists() {
                                        let _ = fs::copy(&src_img_path, &dest_orig);
                                    }
                                }
                            }
                            Err(err) => errors.push(err),
                        }
                    }
                }
            }
        }
    }

    // Also generate WebP variants for any PNG/JPEG in `assets/` (such as `assets/hero.png`)
    if assets_src.is_dir() {
        for entry in WalkDir::new(&assets_src).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let p = entry.path().to_path_buf();
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_ascii_lowercase())
                    .unwrap_or_default();
                if matches!(ext.as_str(), "png" | "jpg" | "jpeg") && processed_images.insert(p.clone()) {
                    let rel_disp = p
                        .strip_prefix(site_root)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/");
                    let rel_from_assets = p
                        .strip_prefix(&assets_src)
                        .unwrap_or(&p)
                        .parent()
                        .unwrap_or(Path::new(""));
                    let target_dir = assets_dst.join(rel_from_assets);
                    match generate_webp_variants_for_file(&p, &target_dir, &rel_disp) {
                        Ok(count) => images_generated += count,
                        Err(err) => errors.push(err),
                    }
                }
            }
        }
    }

    // STEP 8: Generated files: sitemap.xml, robots.txt, RSS 2.0 (feed.xml) + Atom (atom.xml), search.json
    let sitemap_xml = generate_sitemap_xml(&site_config.base_url, &sitemap_entries);
    let _ = fs::write(out_tmp.join("sitemap.xml"), sitemap_xml);

    let robots_txt = generate_robots_txt(&site_config.base_url);
    let _ = fs::write(out_tmp.join("robots.txt"), robots_txt);

    let blog_posts = collection_posts
        .get("blog")
        .cloned()
        .unwrap_or_else(|| first_col_posts.clone());
    let rss_xml = generate_rss_xml(&site_config, &blog_posts);
    let _ = fs::write(out_tmp.join("feed.xml"), rss_xml);

    let atom_xml = generate_atom_xml(&site_config, &blog_posts);
    let _ = fs::write(out_tmp.join("atom.xml"), atom_xml);

    let search_json =
        serde_json::to_string_pretty(&search_entries).unwrap_or_else(|_| "[]".to_string());
    let _ = fs::write(out_tmp.join("search.json"), search_json);

    // STEP 10: Build log + atomic swap `out.tmp/` -> `out/` ONLY if errors.is_empty()
    log.push(format!(
        "Pages built: {}, WebP image variants generated: {}, Errors: {}",
        pages_built,
        images_generated,
        errors.len()
    ));
    for err in &errors {
        log.push(format!("ERROR: {}", err));
    }

    if !errors.is_empty() {
        log::error!(
            target: "pico::builder",
            "Build failed with {} errors: {:?}",
            errors.len(),
            errors
        );
        // Do NOT overwrite a previous good `out/` with a partial/broken build!
        let _ = fs::remove_dir_all(&out_tmp);
        return BuildResult {
            success: false,
            out_dir: out_final.to_string_lossy().to_string(),
            errors,
            pages_built: 0,
            images_generated: 0,
            log,
        };
    }

    let _ = fs::write(out_tmp.join("build.log"), log.join("\n"));

    if out_final.exists() {
        let _ = fs::remove_dir_all(&out_final);
    }
    if let Err(e) = fs::rename(&out_tmp, &out_final) {
        // Fallback if rename across mount points
        if let Err(copy_err) = copy_dir_recursive(&out_tmp, &out_final) {
            log::error!(
                target: "pico::builder",
                "Failed to finalize out/ directory (rename err: {}, copy err: {})",
                e,
                copy_err
            );
            return BuildResult {
                success: false,
                out_dir: out_final.to_string_lossy().to_string(),
                errors: vec![BuildError {
                    file: "out/".to_string(),
                    line: 0,
                    message: format!("Failed to finalize out/ directory ({}): {}", e, copy_err),
                }],
                pages_built: 0,
                images_generated: 0,
                log,
            };
        }
        let _ = fs::remove_dir_all(&out_tmp);
    }

    log::info!(
        target: "pico::builder",
        "Build finished successfully: pages_built={}, images_generated={}, out_dir={:?}",
        pages_built,
        images_generated,
        out_final
    );

    BuildResult {
        success: true,
        out_dir: out_final.to_string_lossy().to_string(),
        errors: Vec::new(),
        pages_built,
        images_generated,
        log,
    }
}
