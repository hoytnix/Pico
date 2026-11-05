use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// Re-export runtime AppState and PreviewServerHandle from state module
pub use crate::state::{AppState, PreviewServerHandle};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NavItem {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SiteConfig {
    pub title: String,
    pub base_url: String,
    pub description: String,
    pub author: String,
    #[serde(default)]
    pub nav: Vec<NavItem>,
    #[serde(default)]
    pub form_endpoint: String,
    #[serde(default)]
    pub comments_provider: String,
    #[serde(default)]
    pub comments_repo: String,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "My Site".to_string(),
            base_url: "https://example.com".to_string(),
            description: "A Pico site".to_string(),
            author: "Your Name".to_string(),
            nav: vec![
                NavItem {
                    label: "Blog".to_string(),
                    url: "/blog/".to_string(),
                },
                NavItem {
                    label: "Projects".to_string(),
                    url: "/projects/".to_string(),
                },
            ],
            form_endpoint: String::new(),
            comments_provider: String::new(),
            comments_repo: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelField {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelDef {
    pub name: String,
    pub label: String,
    pub template: String,
    pub list_template: String,
    #[serde(default)]
    pub fields: Vec<ModelField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostMeta {
    pub title: String,
    pub date: String,
    pub slug: String,
    pub draft: bool,
    pub path: String,
    pub url: String,
    pub collection: String,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub hero: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostDocument {
    pub frontmatter: Value,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentSite {
    pub title: String,
    pub path: String,
    pub last_opened: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteInfo {
    pub path: String,
    pub config: SiteConfig,
    pub collections: Vec<ModelDef>,
    pub recent_files: Vec<String>,
    pub recent_sites: Vec<RecentSite>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildError {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(f, "{} (line {}): {}", self.file, self.line, self.message)
        } else if !self.file.is_empty() {
            write!(f, "{}: {}", self.file, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub out_dir: String,
    pub errors: Vec<BuildError>,
    pub pages_built: usize,
    pub images_generated: usize,
    pub log: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeTarget {
    pub kind: String, // "content" | "template" | "theme" | "model"
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingSource {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeResult {
    pub diff: String,
    pub proposed: String,
    #[serde(default)]
    pub validation_error: Option<String>,
    #[serde(default)]
    pub sources: Option<Vec<GroundingSource>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VibeOptions {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub thinking: Option<String>,
    #[serde(default)]
    pub grounding: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeContext {
    pub original_content: String,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeSiteFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeSiteSnapshot {
    pub site_yaml: String,
    pub models: Vec<VibeSiteFile>,
    pub theme_css: String,
    pub templates: Vec<VibeSiteFile>,
    pub content_manifest: Vec<PostMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderInfo {
    pub id: String,
    pub label: String,
    pub base_url: String,
    pub default_models: Vec<String>,
    pub key_label: String,
    pub key_url: String,
    pub supports_thinking: bool,
    pub supports_grounding: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderSettings {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub custom_model: String,
    #[serde(default = "default_thinking_str")]
    pub thinking: String,
    #[serde(default)]
    pub grounding: bool,
}

fn default_thinking_str() -> String {
    "medium".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderView {
    pub id: String,
    pub base_url: String,
    pub model: String,
    pub custom_model: String,
    pub effective_model: String,
    pub thinking: String,
    pub grounding: bool,
    pub key_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettingsView {
    pub current_provider: String,
    pub providers: HashMap<String, ProviderView>,
    pub github_token: String,
    pub has_github_token: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUpdate {
    pub id: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub custom_model: Option<String>,
    #[serde(default)]
    pub thinking: Option<String>,
    #[serde(default)]
    pub grounding: Option<bool>,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveLlmSettingsInput {
    #[serde(default)]
    pub current_provider: String,
    #[serde(default)]
    pub provider_updates: Vec<ProviderUpdate>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub custom_model: Option<String>,
    #[serde(default)]
    pub thinking: Option<String>,
    #[serde(default)]
    pub grounding: Option<bool>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub github_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub model: String,
    pub elapsed_ms: u64,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmAppSettings {
    #[serde(default = "default_current_provider")]
    pub current_provider: String,
    #[serde(default)]
    pub providers: HashMap<String, ProviderSettings>,
    #[serde(default)]
    pub github_token: String,
    #[serde(default)]
    pub recent_sites: Vec<RecentSite>,
}

fn default_current_provider() -> String {
    "gemini-native".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDoc {
    pub name: String,
    pub kind: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContextInfo {
    pub template_path: String,
    pub is_skeleton: bool,
    pub is_macro: bool,
    pub collection_label: Option<String>,
    pub has_revert: bool,
    pub variables: Vec<VariableDoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub success: bool,
    pub message: String,
    pub url: Option<String>,
}


