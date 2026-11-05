use crate::models::PostMeta;
use crate::validation::slugify;
use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::Path;

/// Standard extracted frontmatter attributes common to both content scanning and IPC listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFrontmatter {
    pub title: String,
    pub date: String,
    pub slug: String,
    pub draft: bool,
    pub excerpt: String,
    pub hero: String,
    pub tags: Vec<String>,
}

impl ExtractedFrontmatter {
    pub fn from_value(fm: &Value) -> Option<Self> {
        let obj = fm.as_object()?;
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
        let draft = obj
            .get("draft")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
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

        Some(Self {
            title,
            date,
            slug,
            draft,
            excerpt,
            hero,
            tags,
        })
    }

    pub fn into_post_meta(self, rel_path: &str, folder_name: &str, collection: &str) -> PostMeta {
        let url = format!("/{}/{}/", folder_name, self.slug);
        PostMeta {
            title: self.title,
            date: self.date,
            slug: self.slug,
            draft: self.draft,
            path: rel_path.to_string(),
            url,
            collection: collection.to_string(),
            excerpt: self.excerpt,
            tags: self.tags,
            hero: self.hero,
        }
    }
}

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
