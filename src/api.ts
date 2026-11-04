import { generateText } from "ai";
import { invoke } from "@tauri-apps/api/core";
import { open as dialogOpen, ask as dialogAsk } from "@tauri-apps/plugin-dialog";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import { createVibeModel, buildProviderOptions, formatLlmError } from "./lib/vibeAgent";
import type {
  AppSettingsView,
  BuildResult,
  GroundingSource,
  LlmSettingsView,
  ModelDef,
  PostDocument,
  PostMeta,
  ProviderInfo,
  PublishResult,
  RecentSite,
  SaveLlmSettingsInput,
  SiteConfig,
  SiteInfo,
  TemplateContextInfo,
  TestConnectionResult,
  VibeOptions,
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
  listLlmProviders(): Promise<ProviderInfo[]> {
    return invoke<ProviderInfo[]>("list_llm_providers");
  },
  getLlmSettings(): Promise<LlmSettingsView> {
    return invoke<LlmSettingsView>("get_llm_settings");
  },
  saveLlmSettings(settings: SaveLlmSettingsInput): Promise<void> {
    return invoke<void>("save_llm_settings", { settings });
  },
  getLlmKey(provider: string): Promise<string> {
    return invoke<string>("get_llm_key", { provider });
  },
  async testLlmConnection(provider: string, model: string): Promise<TestConnectionResult> {
    let apiKey = "";
    try {
      apiKey = await picoApi.getLlmKey(provider);
    } catch {
      return {
        success: false,
        model,
        elapsed_ms: 0,
        error: `Invalid API key for ${provider}. Check the key in Settings.`,
      };
    }

    if (!apiKey || !apiKey.trim()) {
      return {
        success: false,
        model,
        elapsed_ms: 0,
        error: `Invalid API key for ${provider}. Check the key in Settings.`,
      };
    }

    const settings = await picoApi.getLlmSettings();
    const provView = settings.providers[provider];
    const baseUrl = provView?.base_url;

    const start = Date.now();
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30000);

    try {
      const config = {
        provider,
        model,
        apiKey: apiKey.trim(),
        baseUrl,
        thinking: "default",
      };
      const modelInstance = createVibeModel(config);

      await generateText({
        model: modelInstance,
        prompt: "Reply with exactly: OK",
        maxOutputTokens: 10,
        abortSignal: controller.signal,
        providerOptions: buildProviderOptions(config),
      });

      clearTimeout(timeoutId);
      const elapsed_ms = Date.now() - start;
      return {
        success: true,
        model,
        elapsed_ms,
        error: null,
      };
    } catch (err: any) {
      clearTimeout(timeoutId);
      const elapsed_ms = Date.now() - start;
      return {
        success: false,
        model,
        elapsed_ms,
        error: formatLlmError(err, provider, model),
      };
    }
  },
  getVibeContext(target: VibeTarget): Promise<{ original_content: string; system_prompt: string }> {
    return invoke<{ original_content: string; system_prompt: string }>("get_vibe_context", { target });
  },
  evaluateVibeProposal(
    target: VibeTarget,
    proposed: string,
    sources?: GroundingSource[] | null
  ): Promise<VibeResult> {
    return invoke<VibeResult>("evaluate_vibe_proposal_cmd", { target, proposed, sources });
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
