import { generateText } from "ai";
import { invoke } from "@tauri-apps/api/core";
import { open as dialogOpen, ask as dialogAsk } from "@tauri-apps/plugin-dialog";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import { createVibeModel, buildProviderOptions, formatLlmError } from "./lib/vibeAgent";
import { picoLog } from "./lib/logger";
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

async function apiInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  picoLog.debug("api", `[invoke -> ${cmd}]`, args ? Object.keys(args) : "");
  try {
    const res = await invoke<T>(cmd, args);
    picoLog.trace("api", `[invoke <- ${cmd} OK]`);
    return res;
  } catch (err: any) {
    picoLog.error("api", `[invoke <- ${cmd} ERROR]`, err);
    throw err;
  }
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
    return apiInvoke<SiteInfo>("open_site", { path });
  },
  createSite(path: string, starter: string, title: string, author: string): Promise<SiteInfo> {
    return apiInvoke<SiteInfo>("create_site", { path, starter, title, author });
  },
  getRecentSites(): Promise<RecentSite[]> {
    return apiInvoke<RecentSite[]>("get_recent_sites");
  },
  listContent(collection: string): Promise<PostMeta[]> {
    return apiInvoke<PostMeta[]>("list_content", { collection });
  },
  readPost(path: string): Promise<PostDocument> {
    return apiInvoke<PostDocument>("read_post", { path });
  },
  savePost(path: string, frontmatter: Record<string, unknown>, markdown: string): Promise<void> {
    return apiInvoke<void>("save_post", { path, frontmatter, markdown });
  },
  deletePost(path: string): Promise<void> {
    return apiInvoke<void>("delete_post", { path });
  },
  saveImage(sourcePath: string, postSlug: string, collection: string): Promise<string> {
    return apiInvoke<string>("save_image", {
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
    return apiInvoke<string>("save_image_bytes", {
      fileName,
      bytes,
      postSlug,
      collection,
      siteWide,
    });
  },
  checkImageSize(sourcePath: string): Promise<number> {
    return apiInvoke<number>("check_image_size", { sourcePath });
  },
  listModels(): Promise<ModelDef[]> {
    return apiInvoke<ModelDef[]>("list_models");
  },
  saveModel(model: ModelDef): Promise<void> {
    return apiInvoke<void>("save_model", { model });
  },
  deleteModel(name: string): Promise<void> {
    return apiInvoke<void>("delete_model", { name });
  },
  listTemplates(): Promise<string[]> {
    return apiInvoke<string[]>("list_templates");
  },
  readTemplate(path: string): Promise<string> {
    return apiInvoke<string>("read_template", { path });
  },
  saveTemplate(path: string, content: string): Promise<void> {
    return apiInvoke<void>("save_template", { path, content });
  },
  revertTemplate(path: string): Promise<string> {
    return apiInvoke<string>("revert_template", { path });
  },
  getTemplateContext(path: string): Promise<TemplateContextInfo> {
    return apiInvoke<TemplateContextInfo>("get_template_context", { path });
  },
  readTheme(): Promise<string> {
    return apiInvoke<string>("read_theme");
  },
  saveTheme(content: string): Promise<void> {
    return apiInvoke<void>("save_theme", { content });
  },
  buildSite(includeDrafts: boolean): Promise<BuildResult> {
    return apiInvoke<BuildResult>("build_site", { includeDrafts });
  },
  startPreview(): Promise<string> {
    return apiInvoke<string>("start_preview");
  },
  getPreviewRevision(): Promise<number> {
    return apiInvoke<number>("get_preview_revision");
  },
  exportSite(destPath: string): Promise<void> {
    return apiInvoke<void>("export_site", { destPath });
  },
  publishGithubPages(): Promise<PublishResult> {
    return apiInvoke<PublishResult>("publish_github_pages");
  },
  listLlmProviders(): Promise<ProviderInfo[]> {
    return apiInvoke<ProviderInfo[]>("list_llm_providers");
  },
  getLlmSettings(): Promise<LlmSettingsView> {
    return apiInvoke<LlmSettingsView>("get_llm_settings");
  },
  saveLlmSettings(settings: SaveLlmSettingsInput): Promise<void> {
    return apiInvoke<void>("save_llm_settings", { settings });
  },
  getLlmKey(provider: string): Promise<string> {
    return apiInvoke<string>("get_llm_key", { provider });
  },
  async testLlmConnection(provider: string, model: string): Promise<TestConnectionResult> {
    picoLog.info("llm", `Testing LLM connection: provider=${provider}, model=${model}`);
    let apiKey = "";
    try {
      apiKey = await picoApi.getLlmKey(provider);
    } catch {
      picoLog.warn("llm", `No key found for ${provider}`);
      return {
        success: false,
        model,
        elapsed_ms: 0,
        error: `Invalid API key for ${provider}. Check the key in Settings.`,
      };
    }

    if (!apiKey || !apiKey.trim()) {
      picoLog.warn("llm", `Empty key for ${provider}`);
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
      picoLog.info("llm", `LLM connection test succeeded in ${elapsed_ms}ms`);
      return {
        success: true,
        model,
        elapsed_ms,
        error: null,
      };
    } catch (err: any) {
      clearTimeout(timeoutId);
      const elapsed_ms = Date.now() - start;
      const formatted = formatLlmError(err, provider, model);
      picoLog.error("llm", `LLM connection test failed in ${elapsed_ms}ms: ${formatted}`, err);
      return {
        success: false,
        model,
        elapsed_ms,
        error: formatted,
      };
    }
  },
  getVibeContext(target: VibeTarget): Promise<{ original_content: string; system_prompt: string }> {
    return apiInvoke<{ original_content: string; system_prompt: string }>("get_vibe_context", { target });
  },
  evaluateVibeProposal(
    target: VibeTarget,
    proposed: string,
    sources?: GroundingSource[] | null
  ): Promise<VibeResult> {
    return apiInvoke<VibeResult>("evaluate_vibe_proposal_cmd", { target, proposed, sources });
  },
  getSiteConfig(): Promise<SiteConfig> {
    return apiInvoke<SiteConfig>("get_site_config");
  },
  saveSiteConfig(config: SiteConfig): Promise<void> {
    return apiInvoke<void>("save_site_config", { config });
  },
  getAppSettings(): Promise<AppSettingsView> {
    return apiInvoke<AppSettingsView>("get_app_settings");
  },
  saveAppSettings(params: {
    llmEndpoint: string;
    llmModel: string;
    llmApiKey?: string | null;
    githubToken?: string | null;
  }): Promise<AppSettingsView> {
    return apiInvoke<AppSettingsView>("save_app_settings", params);
  },
};
