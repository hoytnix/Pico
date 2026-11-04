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
  kind: "content" | "template" | "theme";
  path: string;
}

export interface VibeResult {
  diff: string;
  proposed: string;
  validation_error?: string | null;
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
