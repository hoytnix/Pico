import { GoogleGenAI } from "@google/genai";
import { invoke } from "@tauri-apps/api/core";
import { open as dialogOpen, ask as dialogAsk } from "@tauri-apps/plugin-dialog";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
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

function formatGeminiError(err: unknown, model: string): string {
  const msg = err instanceof Error ? err.message : String(err);
  const lower = msg.toLowerCase();
  if (
    lower.includes("401") ||
    lower.includes("api key") ||
    lower.includes("api_key") ||
    lower.includes("unauthenticated")
  ) {
    return `Invalid API key for gemini-native. Check the key in Settings. (${msg})`;
  }
  if (lower.includes("404") || lower.includes("not found")) {
    return `Model ${model} was not found for gemini-native. Check the model ID or pick another model. (${msg})`;
  }
  if (
    lower.includes("400") &&
    (lower.includes("thinking") || lower.includes("budget") || lower.includes("reasoning"))
  ) {
    return `Thinking setting is not supported for this model/provider combination. (${msg})`;
  }
  if (
    lower.includes("429") ||
    lower.includes("quota") ||
    lower.includes("rate limit") ||
    lower.includes("resource_exhausted")
  ) {
    return `Rate limited by gemini-native. Wait and retry. (${msg})`;
  }
  if (
    lower.includes("timed out") ||
    lower.includes("timeout") ||
    lower.includes("fetch failed") ||
    lower.includes("network")
  ) {
    return `Could not reach gemini-native. Check your connection and the base URL, then retry. (${msg})`;
  }
  return `gemini-native error: ${msg}`;
}

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
    if (provider === "gemini-native") {
      let apiKey = "";
      try {
        apiKey = await picoApi.getLlmKey("gemini-native");
      } catch (e) {
        return {
          success: false,
          model,
          elapsed_ms: 0,
          error: "Invalid API key for gemini-native. Check the key in Settings.",
        };
      }
      if (!apiKey.trim()) {
        return {
          success: false,
          model,
          elapsed_ms: 0,
          error: "Invalid API key for gemini-native. Check the key in Settings.",
        };
      }

      const start = Date.now();
      try {
        const ai = new GoogleGenAI({ apiKey: apiKey.trim() });
        await ai.models.generateContent({
          model,
          contents: "Reply with exactly: OK",
          config: {
            maxOutputTokens: 10,
          },
        });
        const elapsed_ms = Date.now() - start;
        return {
          success: true,
          model,
          elapsed_ms,
          error: null,
        };
      } catch (err) {
        const elapsed_ms = Date.now() - start;
        return {
          success: false,
          model,
          elapsed_ms,
          error: formatGeminiError(err, model),
        };
      }
    }

    return invoke<TestConnectionResult>("test_llm_connection", { provider, model });
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
  async vibeEdit(target: VibeTarget, prompt: string, options?: VibeOptions): Promise<VibeResult> {
    const settings = await picoApi.getLlmSettings();
    const currentProvider = options?.provider || settings.current_provider;
    const provView = settings.providers[currentProvider];
    const effectiveModel = options?.model || provView?.effective_model || "gemini-4-argon";
    const thinking = options?.thinking || provView?.thinking || "medium";
    const grounding =
      options?.grounding !== undefined ? options.grounding : (provView?.grounding || false);

    if (currentProvider === "gemini-native") {
      let apiKey = "";
      try {
        apiKey = await picoApi.getLlmKey("gemini-native");
      } catch {
        apiKey = "";
      }
      if (!apiKey || !apiKey.trim()) {
        throw new Error("Configure an LLM API key in Settings to use Vibe.");
      }

      const { original_content, system_prompt } = await picoApi.getVibeContext(target);
      const ai = new GoogleGenAI({ apiKey: apiKey.trim() });

      const config: Record<string, unknown> = {};
      if (grounding) {
        config.tools = [{ googleSearch: {} }];
      }

      const lowerModel = effectiveModel.toLowerCase();
      if (thinking && thinking !== "default") {
        if (lowerModel.includes("2.5")) {
          // Gemini 2.5 uses budgets, not levels (§5.1)
          const budget = thinking === "low" ? 1024 : thinking === "high" ? 24576 : 8192;
          config.thinkingConfig = { thinkingBudget: budget };
        } else {
          // Gemini 3 and Gemini 4 use thinkingLevel (§5.1)
          const level = thinking === "low" ? "LOW" : thinking === "high" ? "HIGH" : "MEDIUM";
          config.thinkingConfig = { thinkingLevel: level };
        }
      }

      const userMsg = `${system_prompt}\n\nInstruction: ${prompt.trim()}\n\nCurrent file (\`${target.path}\`):\n${original_content}`;

      try {
        const response = await ai.models.generateContent({
          model: effectiveModel,
          contents: userMsg,
          config,
        });

        // Text extraction: concatenate text parts only, ignoring thought parts (§3)
        const parts = response.candidates?.[0]?.content?.parts || [];
        const textParts = parts
          .filter((p: any) => !p.thought && typeof p.text === "string")
          .map((p: any) => p.text as string);
        const proposedRaw = textParts.join("");

        // Grounding extraction: extract grounding chunks / sources (§3, §5.2)
        const sources: GroundingSource[] = [];
        const rawMetadata = response.candidates?.[0]?.groundingMetadata as any;
        if (rawMetadata?.groundingChunks && Array.isArray(rawMetadata.groundingChunks)) {
          for (const chunk of rawMetadata.groundingChunks) {
            if (chunk.web && chunk.web.uri) {
              sources.push({
                title: chunk.web.title || "Grounding Source",
                url: chunk.web.uri,
                snippet: null,
              });
            }
          }
        }

        const sourcesOpt = grounding ? sources : null;
        return await picoApi.evaluateVibeProposal(target, proposedRaw, sourcesOpt);
      } catch (err) {
        throw new Error(formatGeminiError(err, effectiveModel));
      }
    }

    return invoke<VibeResult>("vibe_edit", {
      target,
      prompt,
      options: {
        provider: currentProvider,
        model: effectiveModel,
        thinking,
        grounding,
      },
    });
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
