import { invoke } from "@tauri-apps/api/core";
import { open as dialogOpen, ask as dialogAsk } from "@tauri-apps/plugin-dialog";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import type {
  AppSettingsView,
  BuildResult,
  ModelDef,
  PostDocument,
  PostMeta,
  PublishResult,
  RecentSite,
  SiteConfig,
  SiteInfo,
  TemplateContextInfo,
  VibeResult,
  VibeTarget,
} from "./types";

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function pickFolderDialog(title: string): Promise<string | null> {
  if (isTauriRuntime()) {
    const selected = await dialogOpen({
      directory: true,
      multiple: false,
      title,
    });
    return typeof selected === "string" ? selected : null;
  }
  const manual = window.prompt(`${title} (enter folder path):`, "");
  return manual ? manual.trim() : null;
}

export async function pickImageFileDialog(): Promise<string | null> {
  if (isTauriRuntime()) {
    const selected = await dialogOpen({
      directory: false,
      multiple: false,
      title: "Select Image",
      filters: [
        {
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "webp", "svg", "gif", "avif"],
        },
      ],
    });
    return typeof selected === "string" ? selected : null;
  }
  return null;
}

export async function confirmAction(message: string, title = "Confirm"): Promise<boolean> {
  if (isTauriRuntime()) {
    return await dialogAsk(message, { title, kind: "warning" });
  }
  return window.confirm(message);
}

export async function openUrlInBrowser(url: string): Promise<void> {
  if (isTauriRuntime()) {
    await shellOpen(url);
  } else {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

export const picoApi = {
  openSite(path: string): Promise<SiteInfo> {
    return invoke<SiteInfo>("open_site", { path });
  },
  createSite(path: string, starter: string, title: string, author: string): Promise<SiteInfo> {
    return invoke<SiteInfo>("create_site", { path, starter, title, author });
  },
  getRecentSites(): Promise<RecentSite[]> {
    return invoke<RecentSite[]>("get_recent_sites");
  },
  listContent(collection: string): Promise<PostMeta[]> {
    return invoke<PostMeta[]>("list_content", { collection });
  },
  readPost(path: string): Promise<PostDocument> {
    return invoke<PostDocument>("read_post", { path });
  },
  savePost(path: string, frontmatter: Record<string, unknown>, markdown: string): Promise<void> {
    return invoke<void>("save_post", { path, frontmatter, markdown });
  },
  deletePost(path: string): Promise<void> {
    return invoke<void>("delete_post", { path });
  },
  saveImage(sourcePath: string, postSlug: string, collection: string): Promise<string> {
    return invoke<string>("save_image", {
      sourcePath,
      postSlug,
      collection,
    });
  },
  saveImageBytes(
    fileName: string,
    bytes: number[],
    postSlug: string,
    collection: string,
    siteWide = false
  ): Promise<string> {
    return invoke<string>("save_image_bytes", {
      fileName,
      bytes,
      postSlug,
      collection,
      siteWide,
    });
  },
  checkImageSize(sourcePath: string): Promise<number> {
    return invoke<number>("check_image_size", { sourcePath });
  },
  listModels(): Promise<ModelDef[]> {
    return invoke<ModelDef[]>("list_models");
  },
  saveModel(model: ModelDef): Promise<void> {
    return invoke<void>("save_model", { model });
  },
  deleteModel(name: string): Promise<void> {
    return invoke<void>("delete_model", { name });
  },
  listTemplates(): Promise<string[]> {
    return invoke<string[]>("list_templates");
  },
  readTemplate(path: string): Promise<string> {
    return invoke<string>("read_template", { path });
  },
  saveTemplate(path: string, content: string): Promise<void> {
    return invoke<void>("save_template", { path, content });
  },
  revertTemplate(path: string): Promise<string> {
    return invoke<string>("revert_template", { path });
  },
  getTemplateContext(path: string): Promise<TemplateContextInfo> {
    return invoke<TemplateContextInfo>("get_template_context", { path });
  },
  readTheme(): Promise<string> {
    return invoke<string>("read_theme");
  },
  saveTheme(content: string): Promise<void> {
    return invoke<void>("save_theme", { content });
  },
  buildSite(includeDrafts: boolean): Promise<BuildResult> {
    return invoke<BuildResult>("build_site", { includeDrafts });
  },
  startPreview(): Promise<string> {
    return invoke<string>("start_preview");
  },
  getPreviewRevision(): Promise<number> {
    return invoke<number>("get_preview_revision");
  },
  exportSite(destPath: string): Promise<void> {
    return invoke<void>("export_site", { destPath });
  },
  publishGithubPages(): Promise<PublishResult> {
    return invoke<PublishResult>("publish_github_pages");
  },
  vibeEdit(target: VibeTarget, prompt: string): Promise<VibeResult> {
    return invoke<VibeResult>("vibe_edit", { target, prompt });
  },
  getSiteConfig(): Promise<SiteConfig> {
    return invoke<SiteConfig>("get_site_config");
  },
  saveSiteConfig(config: SiteConfig): Promise<void> {
    return invoke<void>("save_site_config", { config });
  },
  getAppSettings(): Promise<AppSettingsView> {
    return invoke<AppSettingsView>("get_app_settings");
  },
  saveAppSettings(params: {
    llmEndpoint: string;
    llmModel: string;
    llmApiKey?: string | null;
    githubToken?: string | null;
  }): Promise<AppSettingsView> {
    return invoke<AppSettingsView>("save_app_settings", params);
  },
};
