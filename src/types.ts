export interface NavItem {
  label: string;
  url: string;
}

export interface SiteConfig {
  title: string;
  base_url: string;
  description: string;
  author: string;
  nav: NavItem[];
  form_endpoint: string;
  comments_provider: string;
  comments_repo: string;
}

export interface ModelField {
  name: string;
  label: string;
  type: "string" | "date" | "tags" | "image" | "bool" | string;
  required: boolean;
  default?: unknown;
}

export interface ModelDef {
  name: string;
  label: string;
  template: string;
  list_template: string;
  fields: ModelField[];
}

export interface PostMeta {
  title: string;
  date: string;
  slug: string;
  draft: boolean;
  path: string;
  url: string;
  collection: string;
  excerpt: string;
  tags: string[];
  hero: string;
}

export interface PostDocument {
  frontmatter: Record<string, unknown>;
  markdown: string;
}

export interface RecentSite {
  title: string;
  path: string;
  last_opened: string;
}

export interface SiteInfo {
  path: string;
  config: SiteConfig;
  collections: ModelDef[];
  recent_files: string[];
  recent_sites: RecentSite[];
}

export interface BuildError {
  file: string;
  line: number;
  message: string;
}

export interface BuildResult {
  success: boolean;
  out_dir: string;
  errors: BuildError[];
  pages_built: number;
  images_generated: number;
  log: string[];
}

export interface VibeTarget {
  kind: "content" | "template" | "theme" | "model";
  path: string;
}

export interface GroundingSource {
  title: string;
  url: string;
  snippet?: string | null;
}

export interface VibeOptions {
  provider?: string;
  model?: string;
  thinking?: string;
  grounding?: boolean;
}

export interface VibeResult {
  diff: string;
  proposed: string;
  validation_error?: string | null;
  sources?: GroundingSource[] | null;
}

export interface ProviderInfo {
  id: string;
  label: string;
  base_url: string;
  default_models: string[];
  key_label: string;
  key_url: string;
  supports_thinking: boolean;
  supports_grounding: boolean;
}

export interface ProviderView {
  id: string;
  base_url: string;
  model: string;
  custom_model: string;
  effective_model: string;
  thinking: string;
  grounding: boolean;
  key_status: "SET" | "NOT SET" | string;
}

export interface LlmSettingsView {
  current_provider: string;
  providers: Record<string, ProviderView>;
  github_token: string;
  has_github_token: boolean;
}

export interface ProviderUpdate {
  id: string;
  base_url?: string;
  model?: string;
  custom_model?: string;
  thinking?: string;
  grounding?: boolean;
  api_key?: string;
}

export interface SaveLlmSettingsInput {
  current_provider: string;
  provider_updates?: ProviderUpdate[];
  base_url?: string;
  model?: string;
  custom_model?: string;
  thinking?: string;
  grounding?: boolean;
  api_key?: string;
  github_token?: string;
}

export interface TestConnectionResult {
  success: boolean;
  model: string;
  elapsed_ms: number;
  error?: string | null;
}

export interface VariableDoc {
  name: string;
  kind: string;
  description: string;
}

export interface TemplateContextInfo {
  template_path: string;
  is_skeleton: boolean;
  is_macro: boolean;
  collection_label?: string | null;
  has_revert: boolean;
  variables: VariableDoc[];
}

export interface PublishResult {
  success: boolean;
  message: string;
  url?: string | null;
}

export interface AppSettingsView {
  llm_endpoint: string;
  llm_model: string;
  has_llm_key: boolean;
  has_github_token: boolean;
  storage_notice: string;
}
