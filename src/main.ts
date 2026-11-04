import "./styles.css";
import {
  confirmAction,
  openUrlInBrowser,
  pickFolderDialog,
  pickImageFileDialog,
  picoApi,
} from "./api";
import { type CrepeEditorController, useCrepeEditorEffect } from "./crepeHook";
import type {
  AppSettingsView,
  BuildResult,
  ModelDef,
  ModelField,
  PostMeta,
  PublishResult,
  RecentSite,
  SiteConfig,
  SiteInfo,
  TemplateContextInfo,
  VibeResult,
  VibeTarget,
} from "./types";

type ActiveTab = "content" | "models" | "templates" | "theme" | "preview" | "build" | "settings";

interface PendingVibeState {
  target: VibeTarget;
  prompt: string;
  result: VibeResult;
}

class PicoStudioApp {
  private rootEl: HTMLElement;
  private site: SiteInfo | null = null;
  private recentSites: RecentSite[] = [];
  private activeTab: ActiveTab = "content";

  // Content state
  private activeCollection = "blog";
  private posts: PostMeta[] = [];
  private activePostPath: string | null = null;
  private frontmatter: Record<string, unknown> = {};
  private markdownBody = "";
  private slugManuallyEdited = false;
  private crepeController: CrepeEditorController | null = null;
  private autoSaveTimer: number | null = null;

  // Model state
  private models: ModelDef[] = [];
  private activeModelName: string | null = null;
  private editingModel: ModelDef | null = null;

  // Template state
  private templates: string[] = [];
  private activeTemplatePath: string | null = null;
  private templateContent = "";
  private templateContext: TemplateContextInfo | null = null;

  // Theme state
  private themeCssContent = "";

  // Preview & Build state
  private previewUrl: string | null = null;
  private previewRevision = 0;
  private previewPollTimer: number | null = null;
  private includeDraftsInBuild = false;
  private lastBuildResult: BuildResult | null = null;
  private lastPublishResult: PublishResult | null = null;

  // Settings & Vibe state
  private appSettings: AppSettingsView = {
    llm_endpoint: "https://api.openai.com/v1/chat/completions",
    llm_model: "gpt-4o-mini",
    has_llm_key: false,
    has_github_token: false,
    storage_notice: "",
  };
  private pendingVibe: PendingVibeState | null = null;
  private vibeLoading = false;

  // New site form state
  private newSiteStarter = "blog";
  private newSiteFolder = "";
  private newSiteTitle = "My Pico Site";
  private newSiteAuthor = "Your Name";

  // Status & banners
  private saveStatusText = "Saved";
  private hasUnsavedChanges = false;
  private errorBanner: string | null = null;
  private warningBanner: string | null = null;
  private successBanner: string | null = null;

  constructor(rootEl: HTMLElement) {
    this.rootEl = rootEl;
    window.addEventListener("beforeunload", (e) => {
      if (this.hasUnsavedChanges) {
        e.preventDefault();
        e.returnValue = "You have unsaved changes. Are you sure you want to leave?";
      }
    });
  }

  public async init(): Promise<void> {
    try {
      this.recentSites = await picoApi.getRecentSites();
    } catch {
      this.recentSites = [];
    }
    try {
      this.appSettings = await picoApi.getAppSettings();
    } catch {
      // Default settings remain
    }
    this.render();
  }

  private slugifyClient(input: string): string {
    return input
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "");
  }

  private setStatus(status: string, unsaved = false): void {
    this.saveStatusText = status;
    this.hasUnsavedChanges = unsaved;
    const el = document.getElementById("pico-status-text");
    if (el) {
      el.textContent = status;
    }
  }

  private clearBanners(): void {
    this.errorBanner = null;
    this.warningBanner = null;
    this.successBanner = null;
  }

  private async unmountCrepe(): Promise<void> {
    if (this.autoSaveTimer !== null) {
      window.clearTimeout(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }
    if (this.crepeController) {
      await this.crepeController.unmount();
      this.crepeController = null;
    }
  }

  private async guardUnsavedChanges(): Promise<boolean> {
    if (!this.hasUnsavedChanges) return true;
    return await confirmAction(
      "You have unsaved changes in the current editor. Discard them and continue?",
      "Unsaved Changes"
    );
  }

  private async handleOpenSiteFolder(path?: string): Promise<void> {
    if (!(await this.guardUnsavedChanges())) return;
    const chosen = path ?? (await pickFolderDialog("Open Pico Site Folder"));
    if (!chosen) return;
    this.clearBanners();
    try {
      await this.unmountCrepe();
      const info = await picoApi.openSite(chosen);
      this.site = info;
      this.recentSites = info.recent_sites;
      this.models = info.collections;
      this.activeModelName = info.collections[0]?.name ?? null;
      this.editingModel = info.collections[0] ? JSON.parse(JSON.stringify(info.collections[0])) : null;
      this.activeCollection = info.collections[0]?.name ?? "blog";
      this.activeTab = "content";
      await this.loadCollectionPosts(this.activeCollection, true);
      this.startPreviewWatcherPoll();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async handleCreateNewSite(): Promise<void> {
    if (!this.newSiteFolder.trim()) {
      const picked = await pickFolderDialog("Choose Folder for New Pico Site");
      if (!picked) return;
      this.newSiteFolder = picked;
    }
    this.clearBanners();
    try {
      await this.unmountCrepe();
      const info = await picoApi.createSite(
        this.newSiteFolder.trim(),
        this.newSiteStarter,
        this.newSiteTitle.trim() || "My Pico Site",
        this.newSiteAuthor.trim() || "Your Name"
      );
      this.site = info;
      this.recentSites = info.recent_sites;
      this.models = info.collections;
      this.activeModelName = info.collections[0]?.name ?? null;
      this.editingModel = info.collections[0] ? JSON.parse(JSON.stringify(info.collections[0])) : null;
      this.activeCollection = info.collections[0]?.name ?? "blog";
      this.activeTab = "content";
      this.successBanner = `Created new ${this.newSiteStarter} site at ${info.path}`;
      await this.loadCollectionPosts(this.activeCollection, true);
      this.startPreviewWatcherPoll();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private startPreviewWatcherPoll(): void {
    if (this.previewPollTimer !== null) {
      window.clearInterval(this.previewPollTimer);
    }
    this.previewPollTimer = window.setInterval(async () => {
      if (!this.site || !this.previewUrl) return;
      try {
        const rev = await picoApi.getPreviewRevision();
        if (this.previewRevision !== 0 && rev !== this.previewRevision) {
          this.previewRevision = rev;
          const iframe = document.getElementById("pico-preview-iframe") as HTMLIFrameElement | null;
          if (iframe && this.previewUrl) {
            iframe.src = `${this.previewUrl}?rev=${rev}`;
          }
        } else if (this.previewRevision === 0) {
          this.previewRevision = rev;
        }
      } catch {
        // Ignore transient poll errors
      }
    }, 500);
  }

  private getCurrentModel(): ModelDef | undefined {
    return (
      this.site?.collections.find((c) => c.name === this.activeCollection) ??
      this.site?.collections[0]
    );
  }

  private async loadCollectionPosts(collection: string, selectFirst = false): Promise<void> {
    if (!this.site) return;
    this.activeCollection = collection;
    try {
      this.posts = await picoApi.listContent(collection);
      if (selectFirst) {
        if (this.posts.length > 0) {
          await this.openPostInEditor(this.posts[0].path);
          return;
        } else {
          this.activePostPath = null;
          this.frontmatter = {};
          this.markdownBody = "";
        }
      }
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async openPostInEditor(postPath: string): Promise<void> {
    if (!(await this.guardUnsavedChanges())) return;
    await this.unmountCrepe();
    this.clearBanners();
    try {
      const doc = await picoApi.readPost(postPath);
      this.activePostPath = postPath;
      this.frontmatter = doc.frontmatter || {};
      this.markdownBody = doc.markdown || "";
      this.slugManuallyEdited = true;
      this.setStatus("Saved", false);
      this.render();
      await this.mountMilkdownEditor();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async handleCreateNewPost(): Promise<void> {
    if (!this.site) return;
    if (!(await this.guardUnsavedChanges())) return;
    await this.unmountCrepe();
    this.clearBanners();

    const today = new Date().toISOString().slice(0, 10);
    const defaultTitle = "New Post";
    const baseSlug = `new-post-${Date.now().toString().slice(-4)}`;
    const folderName =
      this.activeCollection === "project" ? "projects" : this.activeCollection;
    const newPath = `content/${folderName}/${baseSlug}.md`;

    const fm: Record<string, unknown> = {
      title: defaultTitle,
      date: today,
      tags: [],
      excerpt: "",
      hero: "",
      draft: false,
      slug: baseSlug,
    };
    const initialMd = `# ${defaultTitle}\n\nStart writing here...\n`;

    try {
      await picoApi.savePost(newPath, fm, initialMd);
      this.posts = await picoApi.listContent(this.activeCollection);
      this.activePostPath = newPath;
      this.frontmatter = fm;
      this.markdownBody = initialMd;
      this.slugManuallyEdited = false;
      this.setStatus("Saved", false);
      this.render();
      await this.mountMilkdownEditor();
    } catch (err) {
      this.errorBanner = `Save failed: ${String(err)}`;
      this.render();
    }
  }

  private async handleDeletePost(post: PostMeta, e: Event): Promise<void> {
    e.stopPropagation();
    const confirmed = await confirmAction(
      `Move "${post.title}" (${post.path}) to Trash?`,
      "Delete Post"
    );
    if (!confirmed) return;

    try {
      await picoApi.deletePost(post.path);
      if (this.activePostPath === post.path) {
        await this.unmountCrepe();
        this.activePostPath = null;
      }
      this.successBanner = `Deleted "${post.title}".`;
      await this.loadCollectionPosts(this.activeCollection, true);
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private scheduleAutoSave(): void {
    this.setStatus("Unsaved changes…", true);
    if (this.autoSaveTimer !== null) {
      window.clearTimeout(this.autoSaveTimer);
    }
    // 2 seconds idle debounced auto-save (§3.3)
    this.autoSaveTimer = window.setTimeout(() => {
      this.autoSaveTimer = null;
      void this.saveCurrentPost();
    }, 2000);
  }

  private async saveCurrentPost(): Promise<void> {
    if (!this.site || !this.activePostPath) return;
    if (this.autoSaveTimer !== null) {
      window.clearTimeout(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }

    this.setStatus("Saving…", true);
    const body = this.crepeController
      ? this.crepeController.getMarkdown()
      : this.markdownBody;
    this.markdownBody = body;

    try {
      await picoApi.savePost(this.activePostPath, this.frontmatter, body);
      this.setStatus("Saved", false);
      this.errorBanner = null;
      this.posts = await picoApi.listContent(this.activeCollection);
      this.updateSidebarPostListDOM();
    } catch (err) {
      const msg = String(err);
      this.setStatus(`Save failed: ${msg}`, true);
      this.errorBanner = `Save failed: ${msg}`;
      this.renderBannersOnly();
    }
  }

  private async handleImageFileUpload(file: File, siteWide = false): Promise<string> {
    // Warn if > 2MB and still allow it (§3.3)
    if (file.size > 2 * 1024 * 1024) {
      const mb = (file.size / (1024 * 1024)).toFixed(2);
      this.warningBanner = `Warning: "${file.name}" is ${mb} MB (>2 MB). It has been saved at original quality and will be resized into WebP variants at build time.`;
      this.renderBannersOnly();
    }

    const buf = await file.arrayBuffer();
    const bytes = Array.from(new Uint8Array(buf));
    const slug = String(
      this.frontmatter.slug ||
        this.slugifyClient(String(this.frontmatter.title || "post"))
    );

    const relPath = await picoApi.saveImageBytes(
      file.name,
      bytes,
      slug,
      this.activeCollection,
      siteWide
    );
    return relPath;
  }

  private async handlePickImageFromDisk(target: "hero" | "body-post" | "body-assets"): Promise<void> {
    const picked = await pickImageFileDialog();
    if (!picked) return;
    try {
      const sizeBytes = await picoApi.checkImageSize(picked);
      if (sizeBytes > 2 * 1024 * 1024) {
        const mb = (sizeBytes / (1024 * 1024)).toFixed(2);
        this.warningBanner = `Warning: Selected image is ${mb} MB (>2 MB). It is allowed and will be converted to WebP variants at build time.`;
      }

      const slug = String(
        this.frontmatter.slug ||
          this.slugifyClient(String(this.frontmatter.title || "post"))
      );
      const col = target === "body-assets" ? "assets" : this.activeCollection;
      const relMdPath = await picoApi.saveImage(picked, slug, col);

      if (target === "hero") {
        this.frontmatter.hero = relMdPath;
        const heroInput = document.getElementById("fm-input-hero") as HTMLInputElement | null;
        if (heroInput) heroInput.value = relMdPath;
        this.scheduleAutoSave();
      } else {
        const currentMd = this.crepeController
          ? this.crepeController.getMarkdown()
          : this.markdownBody;
        this.markdownBody = `${currentMd.trimEnd()}\n\n![Image](${relMdPath})\n`;
        await this.unmountCrepe();
        this.render();
        await this.mountMilkdownEditor();
        await this.saveCurrentPost();
      }
      this.renderBannersOnly();
    } catch (err) {
      this.errorBanner = String(err);
      this.renderBannersOnly();
    }
  }

  private async mountMilkdownEditor(): Promise<void> {
    const container = document.getElementById("pico-crepe-container");
    if (!container || !this.activePostPath) return;

    this.crepeController = await useCrepeEditorEffect({
      container,
      initialMarkdown: this.markdownBody,
      onChange: (md) => {
        this.markdownBody = md;
        this.scheduleAutoSave();
      },
      onImageFileUpload: async (file) => {
        return await this.handleImageFileUpload(file, false);
      },
      resolveImageUrl: (url) => {
        if (
          this.previewUrl &&
          !url.startsWith("http") &&
          !url.startsWith("data:") &&
          !url.startsWith("blob:")
        ) {
          const base = this.previewUrl.replace(/\/$/, "");
          if (url.startsWith("/")) {
            return `${base}${url}`;
          }
          const folder =
            this.activeCollection === "project" ? "projects" : this.activeCollection;
          const slug = String(this.frontmatter.slug || "hello-world");
          return `${base}/${folder}/${slug}/${url.replace(/^\.\//, "")}`;
        }
        return url;
      },
    });
  }

  // ==========================================================================
  // Model Editor Operations
  // ==========================================================================

  private async openModelsTab(targetModelName?: string): Promise<void> {
    if (!this.site) return;
    if (!(await this.guardUnsavedChanges())) return;
    await this.unmountCrepe();
    this.activeTab = "models";
    this.clearBanners();

    try {
      this.models = await picoApi.listModels();
      this.site.collections = this.models;
      const target =
        targetModelName ??
        this.activeModelName ??
        this.models[0]?.name ??
        null;
      if (target) {
        await this.selectModel(target);
      } else {
        this.activeModelName = null;
        this.editingModel = null;
        this.render();
      }
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async selectModel(modelName: string): Promise<void> {
    if (!(await this.guardUnsavedChanges())) return;
    this.clearBanners();
    const found = this.models.find((m) => m.name === modelName);
    if (found) {
      this.activeModelName = found.name;
      this.editingModel = JSON.parse(JSON.stringify(found)) as ModelDef;
      this.setStatus("Saved", false);
    } else if (this.models.length > 0) {
      this.activeModelName = this.models[0].name;
      this.editingModel = JSON.parse(JSON.stringify(this.models[0])) as ModelDef;
      this.setStatus("Saved", false);
    } else {
      this.activeModelName = null;
      this.editingModel = null;
    }
    this.render();
  }

  private async handleNewModel(): Promise<void> {
    if (!(await this.guardUnsavedChanges())) return;
    this.clearBanners();
    const id = `collection_${Date.now().toString().slice(-4)}`;
    const newModel: ModelDef = {
      name: id,
      label: "New Collection",
      template: `${id}.jinja`,
      list_template: `${id}-list.jinja`,
      fields: [
        { name: "title", label: "Title", type: "string", required: true },
        { name: "date", label: "Date", type: "date", required: true },
        { name: "tags", label: "Tags", type: "tags", required: false },
        { name: "excerpt", label: "Excerpt", type: "string", required: false },
        { name: "hero", label: "Hero image", type: "image", required: false },
        { name: "draft", label: "Draft", type: "bool", required: false, default: false },
        { name: "slug", label: "Slug", type: "string", required: false },
      ],
    };
    this.activeModelName = newModel.name;
    this.editingModel = newModel;
    this.setStatus("New collection model draft (unsaved)…", true);
    this.render();
  }

  private async handleSaveModel(): Promise<void> {
    if (!this.editingModel) return;
    this.clearBanners();

    const name = this.slugifyClient(this.editingModel.name);
    if (!name) {
      this.errorBanner = "Model name / ID cannot be empty.";
      this.render();
      return;
    }
    this.editingModel.name = name;
    if (!this.editingModel.label.trim()) {
      this.editingModel.label = name;
    }
    if (!this.editingModel.template.trim()) {
      this.editingModel.template = `${name}.jinja`;
    }
    if (!this.editingModel.list_template.trim()) {
      this.editingModel.list_template = `${name}-list.jinja`;
    }

    // Ensure core fields exist
    if (!this.editingModel.fields.some((f) => f.name === "title")) {
      this.editingModel.fields.unshift({
        name: "title",
        label: "Title",
        type: "string",
        required: true,
      });
    }
    if (!this.editingModel.fields.some((f) => f.name === "date")) {
      this.editingModel.fields.splice(1, 0, {
        name: "date",
        label: "Date",
        type: "date",
        required: true,
      });
    }

    // Validate duplicate field names
    const fieldNames = new Set<string>();
    for (const f of this.editingModel.fields) {
      const fn = this.slugifyClient(f.name);
      f.name = fn;
      if (!fn) {
        this.errorBanner = "Field names cannot be empty.";
        this.render();
        return;
      }
      if (fieldNames.has(fn)) {
        this.errorBanner = `Duplicate field name: "${fn}". Each field name must be unique.`;
        this.render();
        return;
      }
      fieldNames.add(fn);
    }

    this.setStatus("Saving model…", true);
    try {
      await picoApi.saveModel(this.editingModel);
      this.models = await picoApi.listModels();
      if (this.site) {
        this.site.collections = this.models;
      }
      this.activeModelName = this.editingModel.name;
      const reloaded = this.models.find((m) => m.name === this.activeModelName);
      if (reloaded) {
        this.editingModel = JSON.parse(JSON.stringify(reloaded)) as ModelDef;
      }
      this.setStatus("Saved", false);
      this.successBanner = `Saved model "${this.editingModel.label}" (models/${this.editingModel.name}.yaml).`;
      this.render();
    } catch (err) {
      const msg = String(err);
      this.setStatus(`Save failed: ${msg}`, true);
      this.errorBanner = msg;
      this.render();
    }
  }

  private async handleDeleteModel(): Promise<void> {
    if (!this.editingModel) return;
    if (this.models.length <= 1) {
      this.warningBanner = "Cannot delete the only remaining collection model in the site.";
      this.render();
      return;
    }

    const modelName = this.editingModel.name;
    const modelLabel = this.editingModel.label;
    const confirmed = await confirmAction(
      `Are you sure you want to delete the "${modelLabel}" model?\n\nThis will move models/${modelName}.yaml to the system trash. Content files in content/${modelName}/ will remain intact.`
    );
    if (!confirmed) return;

    this.clearBanners();
    try {
      await picoApi.deleteModel(modelName);
      this.models = await picoApi.listModels();
      if (this.site) {
        this.site.collections = this.models;
      }
      const nextModel = this.models[0]?.name ?? null;
      if (nextModel) {
        await this.selectModel(nextModel);
      } else {
        this.activeModelName = null;
        this.editingModel = null;
      }
      if (this.activeCollection === modelName) {
        this.activeCollection = this.models[0]?.name ?? "blog";
      }
      this.setStatus("Saved", false);
      this.successBanner = `Model "${modelLabel}" has been moved to trash.`;
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private handleAddModelField(): void {
    if (!this.editingModel) return;
    const newField: ModelField = {
      name: `field_${this.editingModel.fields.length + 1}`,
      label: `Field ${this.editingModel.fields.length + 1}`,
      type: "string",
      required: false,
    };
    this.editingModel.fields.push(newField);
    this.setStatus("Unsaved model edits…", true);
    this.render();
  }

  private handleMoveModelField(index: number, direction: -1 | 1): void {
    if (!this.editingModel) return;
    const fields = this.editingModel.fields;
    const newIdx = index + direction;
    if (newIdx < 0 || newIdx >= fields.length) return;
    const temp = fields[index];
    fields[index] = fields[newIdx];
    fields[newIdx] = temp;
    this.setStatus("Unsaved model edits…", true);
    this.render();
  }

  private handleDeleteModelField(index: number): void {
    if (!this.editingModel) return;
    const field = this.editingModel.fields[index];
    if (field && (field.name === "title" || field.name === "date")) {
      this.warningBanner = `Field "${field.name}" is a core required field and cannot be removed.`;
      this.render();
      return;
    }
    this.editingModel.fields.splice(index, 1);
    this.setStatus("Unsaved model edits…", true);
    this.render();
  }

  // ==========================================================================
  // Template Editor Operations (§3.4)
  // ==========================================================================

  private async openTemplatesTab(): Promise<void> {
    if (!this.site) return;
    if (!(await this.guardUnsavedChanges())) return;
    await this.unmountCrepe();
    this.activeTab = "templates";
    this.clearBanners();

    try {
      this.templates = await picoApi.listTemplates();
      const target = this.activeTemplatePath ?? this.templates[0] ?? "skeleton.jinja";
      await this.selectTemplate(target);
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async selectTemplate(tplPath: string): Promise<void> {
    if (!(await this.guardUnsavedChanges())) return;
    this.clearBanners();
    try {
      this.activeTemplatePath = tplPath;
      this.templateContent = await picoApi.readTemplate(tplPath);
      this.templateContext = await picoApi.getTemplateContext(tplPath);
      this.setStatus("Saved", false);
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async handleSaveTemplate(): Promise<void> {
    if (!this.activeTemplatePath) return;
    this.clearBanners();
    this.setStatus("Saving…", true);
    try {
      await picoApi.saveTemplate(this.activeTemplatePath, this.templateContent);
      this.templateContext = await picoApi.getTemplateContext(this.activeTemplatePath);
      this.setStatus("Saved", false);
      this.successBanner = `Saved templates/${this.activeTemplatePath}. Previous version backed up for one-click Revert.`;
      this.render();
    } catch (err) {
      const msg = String(err);
      this.setStatus(`Save failed: ${msg}`, true);
      this.errorBanner = msg;
      this.render();
    }
  }

  private async handleRevertTemplate(): Promise<void> {
    if (!this.activeTemplatePath) return;
    this.clearBanners();
    try {
      const restored = await picoApi.revertTemplate(this.activeTemplatePath);
      this.templateContent = restored;
      this.templateContext = await picoApi.getTemplateContext(this.activeTemplatePath);
      this.setStatus("Saved", false);
      this.successBanner = `Reverted templates/${this.activeTemplatePath} to previous saved version.`;
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  // ==========================================================================
  // Theme Panel Operations (§7)
  // ==========================================================================

  private async openThemeTab(): Promise<void> {
    if (!this.site) return;
    if (!(await this.guardUnsavedChanges())) return;
    await this.unmountCrepe();
    this.activeTab = "theme";
    this.clearBanners();
    try {
      this.themeCssContent = await picoApi.readTheme();
      this.setStatus("Saved", false);
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async handleSaveTheme(): Promise<void> {
    this.clearBanners();
    try {
      await picoApi.saveTheme(this.themeCssContent);
      this.setStatus("Saved", false);
      this.successBanner = "Saved assets/theme.css.";
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  // ==========================================================================
  // Preview, Build, Export, Publish (§3.5, §3.6)
  // ==========================================================================

  private async openPreviewTab(): Promise<void> {
    if (!this.site) return;
    if (this.activeTab === "content" && this.hasUnsavedChanges) {
      await this.saveCurrentPost();
    }
    await this.unmountCrepe();
    this.activeTab = "preview";
    this.clearBanners();
    try {
      this.previewUrl = await picoApi.startPreview();
      this.previewRevision = await picoApi.getPreviewRevision();
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async openBuildTab(runImmediately = false): Promise<void> {
    if (!this.site) return;
    if (this.activeTab === "content" && this.hasUnsavedChanges) {
      await this.saveCurrentPost();
    }
    await this.unmountCrepe();
    this.activeTab = "build";
    this.clearBanners();
    if (runImmediately) {
      await this.handleRunBuild();
    } else {
      this.render();
    }
  }

  private async handleRunBuild(): Promise<void> {
    this.clearBanners();
    try {
      const res = await picoApi.buildSite(this.includeDraftsInBuild);
      this.lastBuildResult = res;
      if (res.success) {
        this.successBanner = `Build succeeded! Built ${res.pages_built} pages and ${res.images_generated} WebP image variants into ${res.out_dir}.`;
      } else {
        this.errorBanner = `Build reported ${res.errors.length} error(s). Previous good out/ was preserved.`;
      }
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async handleExportSite(): Promise<void> {
    const dest = await pickFolderDialog("Select Destination Folder for Exported Site");
    if (!dest) return;
    this.clearBanners();
    try {
      await picoApi.exportSite(dest);
      this.successBanner = `Exported static site to ${dest}. You can open ${dest}/index.html offline in any browser or host it anywhere.`;
      this.render();
    } catch (err) {
      this.errorBanner = String(err);
      this.render();
    }
  }

  private async handlePublishGithubPages(): Promise<void> {
    this.clearBanners();
    try {
      const res = await picoApi.publishGithubPages();
      this.lastPublishResult = res;
      if (res.success) {
        this.successBanner = res.message;
      } else {
        this.warningBanner = res.message;
      }
      this.render();
    } catch (err) {
      this.warningBanner = String(err);
      this.render();
    }
  }

  // ==========================================================================
  // Vibe Editing (§7)
  // ==========================================================================

  private async handleRunVibe(target: VibeTarget, promptInputId: string): Promise<void> {
    if (!this.appSettings.has_llm_key) return;
    const inputEl = document.getElementById(promptInputId) as HTMLInputElement | null;
    const prompt = inputEl?.value.trim() ?? "";
    if (!prompt) {
      this.warningBanner = "Enter a natural-language instruction for Vibe first.";
      this.renderBannersOnly();
      return;
    }

    if (target.kind === "content" && this.hasUnsavedChanges) {
      await this.saveCurrentPost();
    }

    this.vibeLoading = true;
    this.clearBanners();
    this.render();

    try {
      const res = await picoApi.vibeEdit(target, prompt);
      this.pendingVibe = {
        target,
        prompt,
        result: res,
      };
    } catch (err) {
      this.errorBanner = String(err);
    } finally {
      this.vibeLoading = false;
      this.render();
      if (this.activeTab === "content" && !this.pendingVibe) {
        await this.mountMilkdownEditor();
      }
    }
  }

  private async handleAcceptVibe(): Promise<void> {
    if (!this.pendingVibe) return;
    const { target, result } = this.pendingVibe;

    if (result.validation_error) {
      this.errorBanner = `Cannot accept Vibe change because it violates guardrails: ${result.validation_error}`;
      this.pendingVibe = null;
      this.render();
      return;
    }

    try {
      if (target.kind === "content" && this.activePostPath) {
        this.markdownBody = result.proposed;
        await picoApi.savePost(this.activePostPath, this.frontmatter, this.markdownBody);
        this.pendingVibe = null;
        this.successBanner = "Applied and saved Vibe content edit.";
        this.render();
        await this.mountMilkdownEditor();
      } else if (target.kind === "template" && this.activeTemplatePath) {
        this.templateContent = result.proposed;
        await picoApi.saveTemplate(this.activeTemplatePath, this.templateContent);
        this.templateContext = await picoApi.getTemplateContext(this.activeTemplatePath);
        this.pendingVibe = null;
        this.successBanner = "Applied and saved Vibe template edit (previous version saved for Revert).";
        this.render();
      } else if (target.kind === "theme") {
        this.themeCssContent = result.proposed;
        await picoApi.saveTheme(this.themeCssContent);
        this.pendingVibe = null;
        this.successBanner = "Applied and saved Vibe theme edit to assets/theme.css.";
        this.render();
      }
    } catch (err) {
      this.errorBanner = String(err);
      this.pendingVibe = null;
      this.render();
    }
  }

  private async handleRejectVibe(): Promise<void> {
    this.pendingVibe = null;
    this.render();
    if (this.activeTab === "content") {
      await this.mountMilkdownEditor();
    }
  }

  // ==========================================================================
  // Rendering Helpers
  // ==========================================================================

  private escapeHtml(str: string): string {
    return str
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  private renderVibeBox(target: VibeTarget, inputId: string, placeholder: string): string {
    const disabledAttr = !this.appSettings.has_llm_key || this.vibeLoading ? "disabled" : "";
    const btnLabel = !this.appSettings.has_llm_key
      ? "Configure an LLM API key in Settings to use Vibe."
      : this.vibeLoading
        ? "Generating Vibe Diff…"
        : "Vibe Edit ✨";

    return `
      <div class="vibe-bar">
        <strong>Vibe</strong>
        <input
          type="text"
          id="${inputId}"
          class="vibe-input"
          placeholder="${this.escapeHtml(placeholder)}"
          ${disabledAttr}
        />
        <button
          type="button"
          class="btn btn-primary"
          id="btn-trigger-${inputId}"
          ${disabledAttr}
          title="${this.escapeHtml(btnLabel)}"
        >
          ${this.escapeHtml(btnLabel)}
        </button>
      </div>
    `;
  }

  private renderBannersHtml(): string {
    let html = "";
    if (this.errorBanner) {
      html += `<div class="alert-banner alert-error"><span>${this.escapeHtml(this.errorBanner)}</span><button class="btn" id="btn-dismiss-error">Dismiss</button></div>`;
    }
    if (this.warningBanner) {
      html += `<div class="alert-banner alert-warning"><span>${this.escapeHtml(this.warningBanner)}</span><button class="btn" id="btn-dismiss-warning">Dismiss</button></div>`;
    }
    if (this.successBanner) {
      html += `<div class="alert-banner alert-success"><span>${this.escapeHtml(this.successBanner)}</span><button class="btn" id="btn-dismiss-success">Dismiss</button></div>`;
    }
    return html;
  }

  private renderBannersOnly(): void {
    const container = document.getElementById("pico-banners-slot");
    if (container) {
      container.innerHTML = this.renderBannersHtml();
      this.bindBannerDismissEvents();
    }
  }

  private bindBannerDismissEvents(): void {
    document.getElementById("btn-dismiss-error")?.addEventListener("click", () => {
      this.errorBanner = null;
      this.renderBannersOnly();
    });
    document.getElementById("btn-dismiss-warning")?.addEventListener("click", () => {
      this.warningBanner = null;
      this.renderBannersOnly();
    });
    document.getElementById("btn-dismiss-success")?.addEventListener("click", () => {
      this.successBanner = null;
      this.renderBannersOnly();
    });
  }

  private updateSidebarPostListDOM(): void {
    const listContainer = document.getElementById("pico-sidebar-post-list");
    if (listContainer) {
      listContainer.innerHTML = this.renderSidebarPostsHtml();
      this.bindSidebarPostEvents();
    }
  }

  private renderSidebarPostsHtml(): string {
    if (this.posts.length === 0) {
      return `<div style="padding: 16px; color: var(--pico-muted); text-align: center;">No posts in this collection yet. Click "+ New Post" above.</div>`;
    }
    return this.posts
      .map((p) => {
        const isActive = p.path === this.activePostPath;
        return `
          <div class="post-row ${isActive ? "active" : ""}" data-post-path="${this.escapeHtml(p.path)}">
            <div style="flex: 1; min-width: 0;">
              <div class="post-row-title">
                ${this.escapeHtml(p.title)}
                ${p.draft ? `<span class="badge-draft">DRAFT</span>` : ""}
              </div>
              <div class="post-row-meta">
                <span>${this.escapeHtml(p.date)}</span>
                <span>&middot;</span>
                <code>/${this.escapeHtml(p.slug)}</code>
              </div>
            </div>
            <button
              type="button"
              class="btn btn-danger"
              style="padding: 3px 7px; font-size: 11px;"
              data-delete-path="${this.escapeHtml(p.path)}"
              title="Delete post"
            >
              Delete
            </button>
          </div>
        `;
      })
      .join("");
  }

  private bindSidebarPostEvents(): void {
    document.querySelectorAll<HTMLElement>("[data-post-path]").forEach((row) => {
      row.addEventListener("click", () => {
        const p = row.getAttribute("data-post-path");
        if (p) void this.openPostInEditor(p);
      });
    });
    document.querySelectorAll<HTMLElement>("[data-delete-path]").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const path = btn.getAttribute("data-delete-path");
        const post = this.posts.find((item) => item.path === path);
        if (post) void this.handleDeletePost(post, e);
      });
    });
  }

  // ==========================================================================
  // Main Render
  // ==========================================================================

  public render(): void {
    if (!this.site) {
      this.renderSitePickerScreen();
      return;
    }

    this.rootEl.innerHTML = `
      <header class="topbar">
        <div class="topbar-left">
          <span class="brand-badge">Pico</span>
          <span class="site-pill" title="${this.escapeHtml(this.site.path)}">${this.escapeHtml(this.site.config.title)}</span>
          <button type="button" class="btn" id="btn-switch-site">Switch Site</button>
        </div>

        <nav class="topbar-center">
          <button type="button" class="tab-btn ${this.activeTab === "content" ? "active" : ""}" id="tab-content">Content</button>
          <button type="button" class="tab-btn ${this.activeTab === "models" ? "active" : ""}" id="tab-models">Models</button>
          <button type="button" class="tab-btn ${this.activeTab === "templates" ? "active" : ""}" id="tab-templates">Templates</button>
          <button type="button" class="tab-btn ${this.activeTab === "theme" ? "active" : ""}" id="tab-theme">Theme</button>
          <button type="button" class="tab-btn ${this.activeTab === "preview" ? "active" : ""}" id="tab-preview">Preview</button>
          <button type="button" class="tab-btn ${this.activeTab === "build" ? "active" : ""}" id="tab-build">Build &amp; Export</button>
          <button type="button" class="tab-btn ${this.activeTab === "settings" ? "active" : ""}" id="tab-settings">Settings</button>
        </nav>

        <div class="topbar-right">
          <button type="button" class="btn" id="btn-quick-preview">Preview</button>
          <button type="button" class="btn btn-primary" id="btn-quick-build">Build Site</button>
        </div>
      </header>

      <div class="workspace">
        ${this.renderLeftSidebar()}
        <main class="main-pane">
          <div class="pane-scroll">
            <div id="pico-banners-slot">${this.renderBannersHtml()}</div>
            ${this.renderActiveTabPane()}
          </div>
        </main>
      </div>

      <footer class="statusbar">
        <div>
          <strong>Status:</strong> <span id="pico-status-text">${this.escapeHtml(this.saveStatusText)}</span>
        </div>
        <div>
          Site: <code>${this.escapeHtml(this.site.path)}</code>
          ${this.previewUrl ? `&middot; Preview: <a href="${this.escapeHtml(this.previewUrl)}" id="link-status-preview">${this.escapeHtml(this.previewUrl)}</a>` : ""}
        </div>
      </footer>

      ${this.renderVibeDiffModal()}
    `;

    this.bindWorkspaceEvents();
  }

  private renderSitePickerScreen(): void {
    const starters = [
      {
        id: "blog",
        name: "Blog",
        desc: "Latest posts home, paginated blog list, single post, tag pages, RSS & search.",
      },
      {
        id: "portfolio",
        name: "Portfolio",
        desc: "Projects showcase, project detail with attachment gallery, and notes/about.",
      },
      {
        id: "landing",
        name: "Landing",
        desc: "Single-page product hero, feature cards, and conditional contact form.",
      },
      {
        id: "docs",
        name: "Docs",
        desc: "Sidebar navigation from content order, documentation articles, and instant search.",
      },
    ];

    this.rootEl.innerHTML = `
      <div class="picker-screen">
        <div id="pico-banners-slot">${this.renderBannersHtml()}</div>
        <div class="picker-hero">
          <h1>Pico</h1>
          <p style="color: var(--pico-muted); font-size: 16px;">
            Visual static site studio — Markdown on disk, Milkdown Crepe WYSIWYG editor, and MiniJinja templates. No terminal required.
          </p>
        </div>

        <div class="picker-grid">
          <section class="card-panel">
            <h2 style="margin-top: 0;">Create a New Site</h2>
            <p style="color: var(--pico-muted); font-size: 13px;">Choose one of the 4 complete starter templates:</p>
            <div class="starter-grid">
              ${starters
                .map(
                  (s) => `
                <div class="starter-card ${this.newSiteStarter === s.id ? "selected" : ""}" data-starter="${s.id}">
                  <h4>${s.name}</h4>
                  <p>${s.desc}</p>
                </div>
              `
                )
                .join("")}
            </div>

            <div class="fm-field" style="margin-bottom: 10px;">
              <label>Site Title</label>
              <input type="text" id="new-site-title" class="fm-input" value="${this.escapeHtml(this.newSiteTitle)}" />
            </div>
            <div class="fm-field" style="margin-bottom: 10px;">
              <label>Author Name</label>
              <input type="text" id="new-site-author" class="fm-input" value="${this.escapeHtml(this.newSiteAuthor)}" />
            </div>
            <div class="fm-field" style="margin-bottom: 14px;">
              <label>Destination Folder</label>
              <div style="display: flex; gap: 8px;">
                <input type="text" id="new-site-folder" class="fm-input" style="flex: 1;" placeholder="/path/to/my-site" value="${this.escapeHtml(this.newSiteFolder)}" />
                <button type="button" class="btn" id="btn-pick-new-folder">Browse…</button>
              </div>
            </div>

            <button type="button" class="btn btn-primary" id="btn-create-site" style="width: 100%; justify-content: center; padding: 10px;">
              Create New Site
            </button>
          </section>

          <section class="card-panel">
            <h2 style="margin-top: 0;">Open Existing Site</h2>
            <p style="color: var(--pico-muted); font-size: 13px;">
              Select a Pico site folder on disk containing <code>site.yaml</code>, <code>models/</code>, <code>templates/</code>, and <code>content/</code>.
            </p>
            <button type="button" class="btn btn-primary" id="btn-open-existing-folder" style="width: 100%; justify-content: center; padding: 10px; margin-bottom: 20px;">
              Open Site Folder…
            </button>

            <h3 style="font-size: 13px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--pico-muted);">
              Recent Sites
            </h3>
            ${
              this.recentSites.length === 0
                ? `<p style="color: var(--pico-muted); font-size: 13px;">No recent sites yet.</p>`
                : this.recentSites
                    .map(
                      (r) => `
                  <div class="post-row" data-recent-site="${this.escapeHtml(r.path)}" style="border: 1px solid var(--pico-border);">
                    <div>
                      <div class="post-row-title">${this.escapeHtml(r.title)}</div>
                      <div class="post-row-meta"><code>${this.escapeHtml(r.path)}</code></div>
                    </div>
                    <small style="color: var(--pico-muted);">${this.escapeHtml(r.last_opened)}</small>
                  </div>
                `
                    )
                    .join("")
            }
          </section>
        </div>
      </div>
    `;

    this.bindBannerDismissEvents();
    document.querySelectorAll<HTMLElement>("[data-starter]").forEach((card) => {
      card.addEventListener("click", () => {
        const s = card.getAttribute("data-starter");
        if (s) {
          this.newSiteStarter = s;
          this.render();
        }
      });
    });

    document.getElementById("new-site-title")?.addEventListener("input", (e) => {
      this.newSiteTitle = (e.target as HTMLInputElement).value;
    });
    document.getElementById("new-site-author")?.addEventListener("input", (e) => {
      this.newSiteAuthor = (e.target as HTMLInputElement).value;
    });
    document.getElementById("new-site-folder")?.addEventListener("input", (e) => {
      this.newSiteFolder = (e.target as HTMLInputElement).value;
    });
    document.getElementById("btn-pick-new-folder")?.addEventListener("click", async () => {
      const picked = await pickFolderDialog("Choose Folder for New Pico Site");
      if (picked) {
        this.newSiteFolder = picked;
        this.render();
      }
    });
    document.getElementById("btn-create-site")?.addEventListener("click", () => {
      void this.handleCreateNewSite();
    });
    document.getElementById("btn-open-existing-folder")?.addEventListener("click", () => {
      void this.handleOpenSiteFolder();
    });
    document.querySelectorAll<HTMLElement>("[data-recent-site]").forEach((row) => {
      row.addEventListener("click", () => {
        const p = row.getAttribute("data-recent-site");
        if (p) void this.handleOpenSiteFolder(p);
      });
    });
  }

  private renderLeftSidebar(): string {
    if (!this.site) return "";

    if (this.activeTab === "models") {
      return `
        <aside class="sidebar">
          <div class="sidebar-section" style="display: flex; justify-content: space-between; align-items: center;">
            <div class="sidebar-heading" style="margin: 0;">
              <span>Models (${this.models.length})</span>
            </div>
            <button type="button" class="btn btn-primary" id="btn-new-model" style="padding: 4px 10px; font-size: 12px;">
              + New Model
            </button>
          </div>
          <div style="padding: 10px 12px; font-size: 12px; color: var(--pico-muted); border-bottom: 1px solid var(--pico-border);">
            Collection models in <code>models/*.yaml</code> define frontmatter schemas, field types, and templates.
          </div>
          <div class="post-list">
            ${this.models
              .map((m) => {
                const active = m.name === this.activeModelName;
                return `
                  <div class="post-row ${active ? "active" : ""}" data-model-name="${this.escapeHtml(m.name)}">
                    <div>
                      <div class="post-row-title">
                        ${this.escapeHtml(m.label)}
                      </div>
                      <div class="post-row-meta">
                        <code>models/${this.escapeHtml(m.name)}.yaml</code>
                        <span>&middot; ${m.fields.length} fields</span>
                      </div>
                    </div>
                  </div>
                `;
              })
              .join("")}
          </div>
        </aside>
      `;
    }

    if (this.activeTab === "templates") {
      return `
        <aside class="sidebar">
          <div class="sidebar-section">
            <div class="sidebar-heading">
              <span>MiniJinja Templates</span>
            </div>
            <p style="margin: 0; font-size: 12px; color: var(--pico-muted);">
              <code>skeleton.jinja</code> is the base layout every page template extends.
            </p>
          </div>
          <div class="post-list">
            ${this.templates
              .map((tpl) => {
                const isBase = tpl === "skeleton.jinja";
                const isMacro = tpl.startsWith("macros/");
                const active = tpl === this.activeTemplatePath;
                return `
                  <div class="post-row ${active ? "active" : ""}" data-tpl-path="${this.escapeHtml(tpl)}">
                    <div>
                      <div class="post-row-title">
                        ${this.escapeHtml(tpl)}
                      </div>
                      <div class="post-row-meta">
                        ${isBase ? `<span class="badge-base">BASE TEMPLATE</span>` : ""}
                        ${isMacro ? `<span class="badge-macro">MACROS ONLY</span>` : ""}
                      </div>
                    </div>
                  </div>
                `;
              })
              .join("")}
          </div>
        </aside>
      `;
    }

    return `
      <aside class="sidebar">
        <div class="sidebar-section">
          <div class="sidebar-heading">
            <span>Collections (models/)</span>
          </div>
          <div class="collection-tabs">
            ${this.site.collections
              .map(
                (col) => `
              <button
                type="button"
                class="collection-chip ${col.name === this.activeCollection ? "active" : ""}"
                data-col-name="${this.escapeHtml(col.name)}"
              >
                ${this.escapeHtml(col.label)}
              </button>
            `
              )
              .join("")}
          </div>
        </div>

        <div class="sidebar-section" style="display: flex; justify-content: space-between; align-items: center;">
          <span class="sidebar-heading" style="margin: 0;">Posts (${this.posts.length})</span>
          <button type="button" class="btn btn-primary" id="btn-new-post" style="padding: 4px 10px; font-size: 12px;">
            + New Post
          </button>
        </div>

        <div class="post-list" id="pico-sidebar-post-list">
          ${this.renderSidebarPostsHtml()}
        </div>
      </aside>
    `;
  }

  private renderActiveTabPane(): string {
    switch (this.activeTab) {
      case "content":
        return this.renderContentEditorPane();
      case "models":
        return this.renderModelEditorPane();
      case "templates":
        return this.renderTemplateEditorPane();
      case "theme":
        return this.renderThemeEditorPane();
      case "preview":
        return this.renderPreviewPane();
      case "build":
        return this.renderBuildExportPane();
      case "settings":
        return this.renderSettingsPane();
    }
  }

  private renderModelEditorPane(): string {
    if (!this.editingModel) {
      return `
        <div class="card-panel" style="text-align: center; padding: 48px;">
          <h2>No Model Selected</h2>
          <p style="color: var(--pico-muted);">Select a model from the left sidebar or click <strong>+ New Model</strong> to create a collection schema.</p>
        </div>
      `;
    }

    const m = this.editingModel;
    const canDelete = this.models.length > 1;

    return `
      <div class="card-panel" style="margin-bottom: 16px;">
        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 10px;">
          <div>
            <h2 style="margin: 0; font-size: 18px;">
              Collection Model: <strong>${this.escapeHtml(m.label)}</strong>
            </h2>
            <span style="font-size: 12px; color: var(--pico-muted);">
              File: <code>models/${this.escapeHtml(m.name)}.yaml</code> &middot; Content folder: <code>content/${this.escapeHtml(m.name)}/</code>
            </span>
          </div>
          <div style="display: flex; gap: 8px;">
            <button
              type="button"
              class="btn btn-danger"
              id="btn-delete-model"
              ${canDelete ? "" : "disabled"}
              title="${canDelete ? "Delete this model (moves to trash)" : "Cannot delete the only collection model"}"
            >
              Delete Model
            </button>
            <button type="button" class="btn btn-primary" id="btn-save-model">
              Save Model
            </button>
          </div>
        </div>

        <div class="fm-grid" style="margin-bottom: 16px;">
          <div class="fm-field">
            <label for="model-name">Collection ID (Folder &amp; File Name) <span style="color: var(--pico-danger);">*</span></label>
            <input
              type="text"
              id="model-name"
              class="fm-input"
              value="${this.escapeHtml(m.name)}"
              placeholder="blog"
            />
          </div>
          <div class="fm-field">
            <label for="model-label">Display Label <span style="color: var(--pico-danger);">*</span></label>
            <input
              type="text"
              id="model-label"
              class="fm-input"
              value="${this.escapeHtml(m.label)}"
              placeholder="Blog"
            />
          </div>
          <div class="fm-field">
            <label for="model-template">Single Item Template</label>
            <input
              type="text"
              id="model-template"
              class="fm-input"
              value="${this.escapeHtml(m.template)}"
              placeholder="blog.jinja"
            />
          </div>
          <div class="fm-field">
            <label for="model-list-template">List / Index Template</label>
            <input
              type="text"
              id="model-list-template"
              class="fm-input"
              value="${this.escapeHtml(m.list_template)}"
              placeholder="blog-list.jinja"
            />
          </div>
        </div>
      </div>

      <div class="card-panel">
        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
          <div>
            <h3 style="margin: 0; font-size: 15px;">Frontmatter Fields Schema</h3>
            <p style="margin: 4px 0 0 0; font-size: 12px; color: var(--pico-muted);">
              These fields generate the visual form in the Content Editor. Frontmatter is written cleanly to disk on save.
            </p>
          </div>
          <button type="button" class="btn" id="btn-add-model-field">
            + Add Field
          </button>
        </div>

        <div class="model-fields-table-container">
          <table class="model-fields-table">
            <thead>
              <tr>
                <th style="width: 25%;">Field Name (YAML Key)</th>
                <th style="width: 25%;">Display Label</th>
                <th style="width: 20%;">Field Type</th>
                <th style="width: 12%; text-align: center;">Required</th>
                <th style="width: 18%; text-align: right;">Actions</th>
              </tr>
            </thead>
            <tbody>
              ${m.fields
                .map((f, i) => {
                  const isCore = f.name === "title" || f.name === "date";
                  return `
                    <tr data-field-row="${i}">
                      <td>
                        <input
                          type="text"
                          class="fm-input model-field-prop"
                          data-field-idx="${i}"
                          data-field-key="name"
                          value="${this.escapeHtml(f.name)}"
                          ${isCore ? 'title="Core field name should remain stable for standard templates"' : ""}
                        />
                      </td>
                      <td>
                        <input
                          type="text"
                          class="fm-input model-field-prop"
                          data-field-idx="${i}"
                          data-field-key="label"
                          value="${this.escapeHtml(f.label)}"
                        />
                      </td>
                      <td>
                        <select
                          class="fm-input model-field-prop"
                          data-field-idx="${i}"
                          data-field-key="type"
                        >
                          <option value="string" ${f.type === "string" ? "selected" : ""}>String (Text)</option>
                          <option value="date" ${f.type === "date" ? "selected" : ""}>Date</option>
                          <option value="tags" ${f.type === "tags" ? "selected" : ""}>Tags (List)</option>
                          <option value="image" ${f.type === "image" ? "selected" : ""}>Image</option>
                          <option value="bool" ${f.type === "bool" ? "selected" : ""}>Boolean (Draft Toggle)</option>
                        </select>
                      </td>
                      <td style="text-align: center;">
                        <input
                          type="checkbox"
                          class="model-field-prop"
                          data-field-idx="${i}"
                          data-field-key="required"
                          ${f.required ? "checked" : ""}
                          ${isCore ? "disabled" : ""}
                        />
                      </td>
                      <td style="text-align: right;">
                        <div style="display: inline-flex; gap: 4px;">
                          <button
                            type="button"
                            class="btn btn-icon"
                            data-move-field="${i}"
                            data-move-dir="-1"
                            ${i === 0 ? "disabled" : ""}
                            title="Move Up"
                          >↑</button>
                          <button
                            type="button"
                            class="btn btn-icon"
                            data-move-field="${i}"
                            data-move-dir="1"
                            ${i === m.fields.length - 1 ? "disabled" : ""}
                            title="Move Down"
                          >↓</button>
                          <button
                            type="button"
                            class="btn btn-icon btn-danger"
                            data-delete-field="${i}"
                            ${isCore ? "disabled" : ""}
                            title="${isCore ? "Core field cannot be deleted" : "Delete Field"}"
                          >✕</button>
                        </div>
                      </td>
                    </tr>
                  `;
                })
                .join("")}
            </tbody>
          </table>
        </div>
      </div>
    `;
  }

  private renderContentEditorPane(): string {
    if (!this.activePostPath) {
      return `
        <div class="card-panel" style="text-align: center; padding: 48px;">
          <h2>No Post Selected</h2>
          <p style="color: var(--pico-muted);">Select a post from the left sidebar or click <strong>+ New Post</strong> to start writing.</p>
        </div>
      `;
    }

    const model = this.getCurrentModel();
    const fields = model?.fields ?? [];

    return `
      ${this.renderVibeBox(
        { kind: "content", path: this.activePostPath },
        "vibe-content-input",
        'Ask Vibe to refine this post, e.g. "make this post more concise"'
      )}

      <div class="frontmatter-card">
        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
          <div>
            <strong>Post Properties (${this.escapeHtml(model?.label ?? "Collection")} Model)</strong>
            <span style="color: var(--pico-muted); font-size: 12px; margin-left: 8px;"><code>${this.escapeHtml(this.activePostPath)}</code></span>
          </div>
          <div style="display: flex; gap: 8px;">
            <button type="button" class="btn" id="btn-insert-post-image">Insert Post Image…</button>
            <button type="button" class="btn" id="btn-insert-site-image">Insert Site Asset…</button>
            <button type="button" class="btn btn-primary" id="btn-save-post">Save Now</button>
          </div>
        </div>

        <div class="fm-grid">
          ${fields.map((field) => this.renderFrontmatterField(field.name, field.label, field.type, field.required)).join("")}
        </div>
      </div>

      <div class="crepe-wrapper">
        <div id="pico-crepe-container"></div>
      </div>
    `;
  }

  private renderFrontmatterField(
    name: string,
    label: string,
    fieldType: string,
    required: boolean
  ): string {
    const rawVal = this.frontmatter[name];
    const reqBadge = required ? ` <span style="color: var(--pico-danger);">*</span>` : "";

    if (fieldType === "bool") {
      const checked = Boolean(rawVal) ? "checked" : "";
      return `
        <div class="fm-field" style="justify-content: center;">
          <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px; color: var(--pico-text);">
            <input type="checkbox" id="fm-input-${name}" data-fm-field="${name}" data-fm-type="bool" ${checked} />
            ${this.escapeHtml(label)} (Draft Badge in Preview, excluded from final Build)
          </label>
        </div>
      `;
    }

    if (fieldType === "tags") {
      const tagStr = Array.isArray(rawVal)
        ? rawVal.join(", ")
        : typeof rawVal === "string"
          ? rawVal
          : "";
      return `
        <div class="fm-field">
          <label for="fm-input-${name}">${this.escapeHtml(label)} (comma-separated)${reqBadge}</label>
          <input
            type="text"
            id="fm-input-${name}"
            class="fm-input"
            data-fm-field="${name}"
            data-fm-type="tags"
            placeholder="pico, release, design"
            value="${this.escapeHtml(tagStr)}"
          />
        </div>
      `;
    }

    if (fieldType === "image") {
      const valStr = typeof rawVal === "string" ? rawVal : "";
      return `
        <div class="fm-field">
          <label for="fm-input-${name}">${this.escapeHtml(label)}${reqBadge}</label>
          <div style="display: flex; gap: 6px;">
            <input
              type="text"
              id="fm-input-${name}"
              class="fm-input"
              style="flex: 1;"
              data-fm-field="${name}"
              data-fm-type="image"
              placeholder="hero.png"
              value="${this.escapeHtml(valStr)}"
            />
            <button type="button" class="btn" id="btn-pick-hero-image">Pick…</button>
          </div>
        </div>
      `;
    }

    const valStr = typeof rawVal === "string" ? rawVal : rawVal != null ? String(rawVal) : "";
    const inputType = fieldType === "date" ? "date" : "text";
    const wideClass = name === "title" || name === "excerpt" ? "wide" : "";

    return `
      <div class="fm-field ${wideClass}">
        <label for="fm-input-${name}">${this.escapeHtml(label)}${reqBadge}</label>
        <input
          type="${inputType}"
          id="fm-input-${name}"
          class="fm-input"
          data-fm-field="${name}"
          data-fm-type="${this.escapeHtml(fieldType)}"
          value="${this.escapeHtml(valStr)}"
        />
      </div>
    `;
  }

  private renderTemplateEditorPane(): string {
    if (!this.activeTemplatePath) {
      return `<div class="card-panel">Select a template from the left sidebar.</div>`;
    }

    const ctx = this.templateContext;
    return `
      ${this.renderVibeBox(
        { kind: "template", path: this.activeTemplatePath },
        "vibe-template-input",
        'Ask Vibe to update this template, e.g. "make all post cards show the date"'
      )}

      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
        <div style="display: flex; align-items: center; gap: 10px;">
          <h2 style="margin: 0; font-size: 18px;"><code>templates/${this.escapeHtml(this.activeTemplatePath)}</code></h2>
          ${ctx?.is_skeleton ? `<span class="badge-base">BASE TEMPLATE — All templates extend skeleton.jinja</span>` : ""}
          ${ctx?.is_macro ? `<span class="badge-macro">MACROS FILE — Only {% macro %} definitions allowed</span>` : ""}
        </div>
        <div style="display: flex; gap: 8px;">
          <button
            type="button"
            class="btn"
            id="btn-revert-template"
            ${ctx?.has_revert ? "" : "disabled"}
            title="Restore previous saved version of this template"
          >
            Revert to Previous Version
          </button>
          <button type="button" class="btn btn-primary" id="btn-save-template">
            Validate &amp; Save Template
          </button>
        </div>
      </div>

      <div class="template-layout">
        <div>
          <textarea id="pico-template-textarea" class="code-editor-textarea" spellcheck="false">${this.escapeHtml(this.templateContent)}</textarea>
        </div>
        <aside class="vars-panel">
          <h3 style="margin-top: 0; font-size: 13px; text-transform: uppercase; color: var(--pico-muted);">
            Available Model Variables
          </h3>
          <p style="font-size: 12px; color: var(--pico-muted);">
            Click a variable to insert it at the cursor. Using variables not in the model will be blocked on save.
          </p>
          ${(ctx?.variables ?? [])
            .map(
              (v) => `
            <button type="button" class="var-chip" data-insert-var="{{ ${this.escapeHtml(v.name)} }}">
              <strong>{{ ${this.escapeHtml(v.name)} }}</strong>
              <div style="font-size: 11px; color: var(--pico-muted); font-family: sans-serif;">
                ${this.escapeHtml(v.description)}
              </div>
            </button>
          `
            )
            .join("")}
        </aside>
      </div>
    `;
  }

  private renderThemeEditorPane(): string {
    return `
      ${this.renderVibeBox(
        { kind: "theme", path: "assets/theme.css" },
        "vibe-theme-input",
        'Ask Vibe to restyle the theme, e.g. "make the site dark with gold headings"'
      )}

      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
        <div>
          <h2 style="margin: 0; font-size: 18px;">Theme Stylesheet (<code>assets/theme.css</code>)</h2>
          <p style="margin: 4px 0 0 0; font-size: 12px; color: var(--pico-muted);">
            Customize CSS variables (<code>--bg</code>, <code>--surface</code>, <code>--text</code>, <code>--accent</code>, <code>--heading-font</code>) without altering <code>skeleton.jinja</code> structure.
          </p>
        </div>
        <button type="button" class="btn btn-primary" id="btn-save-theme">Save Theme CSS</button>
      </div>

      <div style="height: calc(100vh - 220px);">
        <textarea id="pico-theme-textarea" class="code-editor-textarea" spellcheck="false">${this.escapeHtml(this.themeCssContent)}</textarea>
      </div>
    `;
  }

  private renderPreviewPane(): string {
    const url = this.previewUrl ?? "";
    return `
      <div style="display: flex; flex-direction: column; height: calc(100vh - 140px); gap: 12px;">
        <div class="card-panel" style="padding: 12px 16px; display: flex; align-items: center; justify-content: space-between; gap: 12px;">
          <div style="display: flex; align-items: center; gap: 10px;">
            <strong>Live Preview Server:</strong>
            <input
              type="text"
              readonly
              class="fm-input"
              style="width: 260px;"
              id="preview-url-input"
              value="${this.escapeHtml(url)}"
            />
            <button type="button" class="btn" id="btn-copy-preview-url">Copy URL</button>
            <span class="badge-draft">Drafts Visible with DRAFT Badge</span>
          </div>
          <div style="display: flex; gap: 8px;">
            <button type="button" class="btn" id="btn-refresh-preview">Rebuild &amp; Refresh</button>
            <button type="button" class="btn btn-primary" id="btn-open-preview-browser">Open in Browser ↗</button>
          </div>
        </div>

        ${
          url
            ? `<iframe id="pico-preview-iframe" class="preview-iframe" src="${this.escapeHtml(url)}?rev=${this.previewRevision}"></iframe>`
            : `<div class="card-panel">Starting preview server…</div>`
        }
      </div>
    `;
  }

  private renderBuildExportPane(): string {
    const res = this.lastBuildResult;
    return `
      <div class="card-panel" style="margin-bottom: 16px;">
        <h2 style="margin-top: 0;">Build, Export &amp; Publish</h2>
        <p style="color: var(--pico-muted);">
          Build compiles your Markdown content, MiniJinja templates, WebP image variants, RSS/Atom feeds, Sitemap, and Search index into <code>out/</code>.
        </p>

        <div style="margin: 14px 0;">
          <label style="display: inline-flex; align-items: center; gap: 8px; cursor: pointer; font-weight: 600;">
            <input type="checkbox" id="chk-include-drafts" ${this.includeDraftsInBuild ? "checked" : ""} />
            Include draft posts in final Build/Export (default off)
          </label>
        </div>

        <div style="display: flex; flex-wrap: wrap; gap: 10px;">
          <button type="button" class="btn btn-primary" id="btn-run-build">
            Build to out/
          </button>
          <button type="button" class="btn" id="btn-export-folder">
            Export out/ to Folder…
          </button>
          <button type="button" class="btn" id="btn-publish-gh-pages">
            Publish to GitHub Pages
          </button>
        </div>
      </div>

      ${
        res
          ? `
        <div class="card-panel">
          <h3 style="margin-top: 0;">
            Build Log — ${res.success ? `<span style="color: var(--pico-success);">Succeeded</span>` : `<span style="color: var(--pico-danger);">Failed (${res.errors.length} errors)</span>`}
          </h3>
          <p style="font-size: 13px; color: var(--pico-muted);">
            Output folder: <code>${this.escapeHtml(res.out_dir)}</code> &middot; Pages built: <strong>${res.pages_built}</strong> &middot; WebP image variants generated: <strong>${res.images_generated}</strong>
          </p>

          ${
            res.errors.length > 0
              ? `
            <h4>Friendly Build Errors:</h4>
            <ul style="color: var(--pico-danger);">
              ${res.errors
                .map(
                  (e) =>
                    `<li><strong>${this.escapeHtml(e.file)}</strong>${e.line > 0 ? ` (line ${e.line})` : ""}: ${this.escapeHtml(e.message)}</li>`
                )
                .join("")}
            </ul>
          `
              : ""
          }

          <pre class="diff-view">${this.escapeHtml(res.log.join("\n"))}</pre>
        </div>
      `
          : ""
      }
    `;
  }

  private renderSettingsPane(): string {
    if (!this.site) return "";
    const cfg = this.site.config;
    const navLines = cfg.nav.map((n) => `${n.label} | ${n.url}`).join("\n");

    return `
      <div class="picker-grid">
        <section class="card-panel">
          <h2 style="margin-top: 0;">Site Configuration (<code>site.yaml</code>)</h2>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>Site Title</label>
            <input type="text" id="cfg-title" class="fm-input" value="${this.escapeHtml(cfg.title)}" />
          </div>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>Base URL</label>
            <input type="text" id="cfg-base-url" class="fm-input" value="${this.escapeHtml(cfg.base_url)}" />
          </div>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>Description</label>
            <input type="text" id="cfg-description" class="fm-input" value="${this.escapeHtml(cfg.description)}" />
          </div>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>Author</label>
            <input type="text" id="cfg-author" class="fm-input" value="${this.escapeHtml(cfg.author)}" />
          </div>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>Navigation Links (one per line: <code>Label | /url/</code>)</label>
            <textarea id="cfg-nav" class="fm-input" rows="4">${this.escapeHtml(navLines)}</textarea>
          </div>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>Contact Form Endpoint (Formspree / FormSubmit URL, empty = hidden)</label>
            <input type="text" id="cfg-form-endpoint" class="fm-input" value="${this.escapeHtml(cfg.form_endpoint)}" />
          </div>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>Comments Provider (<code>""</code> | <code>"giscus"</code> | <code>"disqus"</code>)</label>
            <input type="text" id="cfg-comments-provider" class="fm-input" value="${this.escapeHtml(cfg.comments_provider)}" />
          </div>
          <div class="fm-field" style="margin-bottom: 14px;">
            <label>Comments Repo (for giscus, e.g. <code>user/repo</code>)</label>
            <input type="text" id="cfg-comments-repo" class="fm-input" value="${this.escapeHtml(cfg.comments_repo)}" />
          </div>
          <button type="button" class="btn btn-primary" id="btn-save-site-config">Save site.yaml</button>
        </section>

        <section class="card-panel">
          <h2 style="margin-top: 0;">Vibe LLM &amp; GitHub Pages Settings</h2>
          <p style="font-size: 12px; color: var(--pico-muted);">
            ${this.escapeHtml(this.appSettings.storage_notice)}
          </p>

          <div class="fm-field" style="margin-bottom: 10px;">
            <label>OpenAI-Compatible Chat Completions Endpoint</label>
            <input type="text" id="set-llm-endpoint" class="fm-input" value="${this.escapeHtml(this.appSettings.llm_endpoint)}" />
          </div>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>LLM Model Name</label>
            <input type="text" id="set-llm-model" class="fm-input" value="${this.escapeHtml(this.appSettings.llm_model)}" />
          </div>
          <div class="fm-field" style="margin-bottom: 10px;">
            <label>
              LLM API Key
              ${this.appSettings.has_llm_key ? `<span class="badge-macro">Configured</span>` : `<span class="badge-draft">Not Set</span>`}
            </label>
            <input type="password" id="set-llm-key" class="fm-input" placeholder="${this.appSettings.has_llm_key ? "•••••••• (leave blank to keep existing key)" : "sk-..."}" />
          </div>
          <div class="fm-field" style="margin-bottom: 14px;">
            <label>
              GitHub Personal Access Token (optional, for GitHub Pages setup)
              ${this.appSettings.has_github_token ? `<span class="badge-macro">Configured</span>` : ""}
            </label>
            <input type="password" id="set-gh-token" class="fm-input" placeholder="${this.appSettings.has_github_token ? "•••••••• (leave blank to keep existing token)" : "ghp_..."}" />
          </div>

          <div style="display: flex; gap: 8px;">
            <button type="button" class="btn btn-primary" id="btn-save-app-settings">Save Settings</button>
            ${
              this.appSettings.has_llm_key
                ? `<button type="button" class="btn btn-danger" id="btn-clear-llm-key">Clear LLM Key</button>`
                : ""
            }
          </div>
        </section>
      </div>
    `;
  }

  private renderVibeDiffModal(): string {
    if (!this.pendingVibe) return "";
    const { target, prompt, result } = this.pendingVibe;
    const diffLines = (result.diff || "No textual differences.")
      .split("\n")
      .map((line) => {
        if (line.startsWith("+") && !line.startsWith("+++")) {
          return `<div class="diff-line-add">${this.escapeHtml(line)}</div>`;
        }
        if (line.startsWith("-") && !line.startsWith("---")) {
          return `<div class="diff-line-del">${this.escapeHtml(line)}</div>`;
        }
        return `<div>${this.escapeHtml(line)}</div>`;
      })
      .join("");

    return `
      <div class="modal-backdrop">
        <div class="modal-card">
          <div class="modal-header">
            <div>
              <strong>Review Vibe Proposed Changes — <code>${this.escapeHtml(target.path)}</code></strong>
              <div style="font-size: 12px; color: var(--pico-muted);">Prompt: "${this.escapeHtml(prompt)}"</div>
            </div>
          </div>

          <div class="modal-body">
            ${
              result.validation_error
                ? `<div class="alert-banner alert-error"><strong>Guardrail Violation (Cannot Save):</strong> ${this.escapeHtml(result.validation_error)}</div>`
                : `<p style="margin-top: 0; font-size: 13px; color: var(--pico-muted);">Review the unified diff below. Changes are never applied unless you click <strong>Accept &amp; Save</strong>.</p>`
            }
            <div class="diff-view">${diffLines}</div>
          </div>

          <div class="modal-footer">
            <button type="button" class="btn" id="btn-vibe-reject">Reject &amp; Discard</button>
            <button
              type="button"
              class="btn btn-primary"
              id="btn-vibe-accept"
              ${result.validation_error ? "disabled" : ""}
            >
              Accept &amp; Save
            </button>
          </div>
        </div>
      </div>
    `;
  }

  private bindWorkspaceEvents(): void {
    this.bindBannerDismissEvents();

    // Topbar navigation
    document.getElementById("btn-switch-site")?.addEventListener("click", async () => {
      if (!(await this.guardUnsavedChanges())) return;
      await this.unmountCrepe();
      this.site = null;
      this.render();
    });

    document.getElementById("tab-content")?.addEventListener("click", async () => {
      if (!(await this.guardUnsavedChanges())) return;
      this.activeTab = "content";
      this.render();
      await this.mountMilkdownEditor();
    });
    document.getElementById("tab-models")?.addEventListener("click", () => {
      void this.openModelsTab();
    });
    document.getElementById("tab-templates")?.addEventListener("click", () => {
      void this.openTemplatesTab();
    });
    document.getElementById("tab-theme")?.addEventListener("click", () => {
      void this.openThemeTab();
    });
    document.getElementById("tab-preview")?.addEventListener("click", () => {
      void this.openPreviewTab();
    });
    document.getElementById("tab-build")?.addEventListener("click", () => {
      void this.openBuildTab(false);
    });
    document.getElementById("tab-settings")?.addEventListener("click", async () => {
      if (!(await this.guardUnsavedChanges())) return;
      await this.unmountCrepe();
      this.activeTab = "settings";
      this.render();
    });

    document.getElementById("btn-quick-preview")?.addEventListener("click", () => {
      void this.openPreviewTab();
    });
    document.getElementById("btn-quick-build")?.addEventListener("click", () => {
      void this.openBuildTab(true);
    });
    document.getElementById("link-status-preview")?.addEventListener("click", (e) => {
      e.preventDefault();
      if (this.previewUrl) void openUrlInBrowser(this.previewUrl);
    });

    // Sidebar collection chips & posts
    document.querySelectorAll<HTMLElement>("[data-col-name]").forEach((chip) => {
      chip.addEventListener("click", async () => {
        const col = chip.getAttribute("data-col-name");
        if (col) {
          this.activeTab = "content";
          await this.loadCollectionPosts(col, true);
        }
      });
    });

    document.getElementById("btn-new-post")?.addEventListener("click", () => {
      void this.handleCreateNewPost();
    });

    this.bindSidebarPostEvents();

    // Model sidebar selection
    document.querySelectorAll<HTMLElement>("[data-model-name]").forEach((row) => {
      row.addEventListener("click", () => {
        const name = row.getAttribute("data-model-name");
        if (name) void this.selectModel(name);
      });
    });

    document.getElementById("btn-new-model")?.addEventListener("click", () => {
      void this.handleNewModel();
    });

    // Template sidebar selection
    document.querySelectorAll<HTMLElement>("[data-tpl-path]").forEach((row) => {
      row.addEventListener("click", () => {
        const tpl = row.getAttribute("data-tpl-path");
        if (tpl) void this.selectTemplate(tpl);
      });
    });

    // Content editor form events
    if (this.activeTab === "content" && this.activePostPath) {
      document.querySelectorAll<HTMLElement>("[data-fm-field]").forEach((el) => {
        const fieldName = el.getAttribute("data-fm-field")!;
        const fieldType = el.getAttribute("data-fm-type")!;

        el.addEventListener("input", () => {
          if (fieldType === "bool") {
            this.frontmatter[fieldName] = (el as HTMLInputElement).checked;
          } else if (fieldType === "tags") {
            const raw = (el as HTMLInputElement).value;
            this.frontmatter[fieldName] = raw
              .split(",")
              .map((t) => t.trim())
              .filter(Boolean);
          } else {
            const val = (el as HTMLInputElement).value;
            this.frontmatter[fieldName] = val;
            if (fieldName === "slug") {
              this.slugManuallyEdited = true;
            }
            if (fieldName === "title" && !this.slugManuallyEdited) {
              const autoSlug = this.slugifyClient(val);
              this.frontmatter.slug = autoSlug;
              const slugEl = document.getElementById("fm-input-slug") as HTMLInputElement | null;
              if (slugEl) slugEl.value = autoSlug;
            }
          }
          this.scheduleAutoSave();
        });
      });

      document.getElementById("btn-pick-hero-image")?.addEventListener("click", () => {
        void this.handlePickImageFromDisk("hero");
      });
      document.getElementById("btn-insert-post-image")?.addEventListener("click", () => {
        void this.handlePickImageFromDisk("body-post");
      });
      document.getElementById("btn-insert-site-image")?.addEventListener("click", () => {
        void this.handlePickImageFromDisk("body-assets");
      });
      document.getElementById("btn-save-post")?.addEventListener("click", () => {
        void this.saveCurrentPost();
      });
      document.getElementById("btn-trigger-vibe-content-input")?.addEventListener("click", () => {
        if (this.activePostPath) {
          void this.handleRunVibe({ kind: "content", path: this.activePostPath }, "vibe-content-input");
        }
      });
    }

    // Model editor events
    if (this.activeTab === "models" && this.editingModel) {
      document.getElementById("model-name")?.addEventListener("input", (e) => {
        if (!this.editingModel) return;
        this.editingModel.name = (e.target as HTMLInputElement).value;
        this.setStatus("Unsaved model edits…", true);
      });
      document.getElementById("model-label")?.addEventListener("input", (e) => {
        if (!this.editingModel) return;
        this.editingModel.label = (e.target as HTMLInputElement).value;
        this.setStatus("Unsaved model edits…", true);
      });
      document.getElementById("model-template")?.addEventListener("input", (e) => {
        if (!this.editingModel) return;
        this.editingModel.template = (e.target as HTMLInputElement).value;
        this.setStatus("Unsaved model edits…", true);
      });
      document.getElementById("model-list-template")?.addEventListener("input", (e) => {
        if (!this.editingModel) return;
        this.editingModel.list_template = (e.target as HTMLInputElement).value;
        this.setStatus("Unsaved model edits…", true);
      });

      document.querySelectorAll<HTMLElement>(".model-field-prop").forEach((el) => {
        const idx = parseInt(el.getAttribute("data-field-idx") || "-1", 10);
        const key = el.getAttribute("data-field-key");
        if (idx >= 0 && key && this.editingModel && this.editingModel.fields[idx]) {
          const field = this.editingModel.fields[idx];
          el.addEventListener("change", () => {
            if (key === "required") {
              field.required = (el as HTMLInputElement).checked;
            } else if (key === "type") {
              field.type = (el as HTMLSelectElement).value;
            } else if (key === "name") {
              field.name = (el as HTMLInputElement).value;
            } else if (key === "label") {
              field.label = (el as HTMLInputElement).value;
            }
            this.setStatus("Unsaved model edits…", true);
          });
          if (key === "name" || key === "label") {
            el.addEventListener("input", () => {
              if (key === "name") {
                field.name = (el as HTMLInputElement).value;
              } else if (key === "label") {
                field.label = (el as HTMLInputElement).value;
              }
              this.setStatus("Unsaved model edits…", true);
            });
          }
        }
      });

      document.querySelectorAll<HTMLElement>("[data-move-field]").forEach((btn) => {
        btn.addEventListener("click", () => {
          const idx = parseInt(btn.getAttribute("data-move-field") || "-1", 10);
          const dir = parseInt(btn.getAttribute("data-move-dir") || "0", 10) as -1 | 1;
          this.handleMoveModelField(idx, dir);
        });
      });

      document.querySelectorAll<HTMLElement>("[data-delete-field]").forEach((btn) => {
        btn.addEventListener("click", () => {
          const idx = parseInt(btn.getAttribute("data-delete-field") || "-1", 10);
          this.handleDeleteModelField(idx);
        });
      });

      document.getElementById("btn-add-model-field")?.addEventListener("click", () => {
        this.handleAddModelField();
      });

      document.getElementById("btn-save-model")?.addEventListener("click", () => {
        void this.handleSaveModel();
      });

      document.getElementById("btn-delete-model")?.addEventListener("click", () => {
        void this.handleDeleteModel();
      });
    }

    // Template editor events
    if (this.activeTab === "templates" && this.activeTemplatePath) {
      const textarea = document.getElementById("pico-template-textarea") as HTMLTextAreaElement | null;
      textarea?.addEventListener("input", () => {
        this.templateContent = textarea.value;
        this.setStatus("Unsaved template edits…", true);
      });

      document.querySelectorAll<HTMLElement>("[data-insert-var]").forEach((btn) => {
        btn.addEventListener("click", () => {
          const snippet = btn.getAttribute("data-insert-var") ?? "";
          if (!textarea) return;
          const start = textarea.selectionStart ?? this.templateContent.length;
          const end = textarea.selectionEnd ?? this.templateContent.length;
          this.templateContent =
            this.templateContent.slice(0, start) + snippet + this.templateContent.slice(end);
          textarea.value = this.templateContent;
          this.setStatus("Unsaved template edits…", true);
          textarea.focus();
        });
      });

      document.getElementById("btn-save-template")?.addEventListener("click", () => {
        void this.handleSaveTemplate();
      });
      document.getElementById("btn-revert-template")?.addEventListener("click", () => {
        void this.handleRevertTemplate();
      });
      document.getElementById("btn-trigger-vibe-template-input")?.addEventListener("click", () => {
        if (this.activeTemplatePath) {
          void this.handleRunVibe(
            { kind: "template", path: this.activeTemplatePath },
            "vibe-template-input"
          );
        }
      });
    }

    // Theme editor events
    if (this.activeTab === "theme") {
      const textarea = document.getElementById("pico-theme-textarea") as HTMLTextAreaElement | null;
      textarea?.addEventListener("input", () => {
        this.themeCssContent = textarea.value;
        this.setStatus("Unsaved theme edits…", true);
      });
      document.getElementById("btn-save-theme")?.addEventListener("click", () => {
        void this.handleSaveTheme();
      });
      document.getElementById("btn-trigger-vibe-theme-input")?.addEventListener("click", () => {
        void this.handleRunVibe({ kind: "theme", path: "assets/theme.css" }, "vibe-theme-input");
      });
    }

    // Preview events
    if (this.activeTab === "preview") {
      document.getElementById("btn-copy-preview-url")?.addEventListener("click", () => {
        if (this.previewUrl) {
          void navigator.clipboard?.writeText(this.previewUrl);
          this.successBanner = `Copied ${this.previewUrl} to clipboard.`;
          this.renderBannersOnly();
        }
      });
      document.getElementById("btn-refresh-preview")?.addEventListener("click", () => {
        void this.openPreviewTab();
      });
      document.getElementById("btn-open-preview-browser")?.addEventListener("click", () => {
        if (this.previewUrl) void openUrlInBrowser(this.previewUrl);
      });
    }

    // Build & Export events
    if (this.activeTab === "build") {
      document.getElementById("chk-include-drafts")?.addEventListener("change", (e) => {
        this.includeDraftsInBuild = (e.target as HTMLInputElement).checked;
      });
      document.getElementById("btn-run-build")?.addEventListener("click", () => {
        void this.handleRunBuild();
      });
      document.getElementById("btn-export-folder")?.addEventListener("click", () => {
        void this.handleExportSite();
      });
      document.getElementById("btn-publish-gh-pages")?.addEventListener("click", () => {
        void this.handlePublishGithubPages();
      });
    }

    // Settings events
    if (this.activeTab === "settings" && this.site) {
      document.getElementById("btn-save-site-config")?.addEventListener("click", async () => {
        if (!this.site) return;
        const navRaw = (document.getElementById("cfg-nav") as HTMLTextAreaElement).value;
        const nav = navRaw
          .split("\n")
          .map((l) => l.trim())
          .filter(Boolean)
          .map((line) => {
            const [label, url] = line.split("|").map((s) => s.trim());
            return { label: label || "Link", url: url || "/" };
          });

        const updatedConfig: SiteConfig = {
          title: (document.getElementById("cfg-title") as HTMLInputElement).value.trim(),
          base_url: (document.getElementById("cfg-base-url") as HTMLInputElement).value.trim(),
          description: (document.getElementById("cfg-description") as HTMLInputElement).value.trim(),
          author: (document.getElementById("cfg-author") as HTMLInputElement).value.trim(),
          nav,
          form_endpoint: (document.getElementById("cfg-form-endpoint") as HTMLInputElement).value.trim(),
          comments_provider: (document.getElementById("cfg-comments-provider") as HTMLInputElement).value.trim(),
          comments_repo: (document.getElementById("cfg-comments-repo") as HTMLInputElement).value.trim(),
        };

        try {
          await picoApi.saveSiteConfig(updatedConfig);
          this.site.config = updatedConfig;
          this.successBanner = "Saved site.yaml configuration.";
          this.render();
        } catch (err) {
          this.errorBanner = String(err);
          this.render();
        }
      });

      document.getElementById("btn-save-app-settings")?.addEventListener("click", async () => {
        const llmEndpoint = (document.getElementById("set-llm-endpoint") as HTMLInputElement).value.trim();
        const llmModel = (document.getElementById("set-llm-model") as HTMLInputElement).value.trim();
        const keyInput = (document.getElementById("set-llm-key") as HTMLInputElement).value.trim();
        const ghInput = (document.getElementById("set-gh-token") as HTMLInputElement).value.trim();

        try {
          this.appSettings = await picoApi.saveAppSettings({
            llmEndpoint,
            llmModel,
            llmApiKey: keyInput ? keyInput : null,
            githubToken: ghInput ? ghInput : null,
          });
          this.successBanner = "Saved Vibe LLM and GitHub settings.";
          this.render();
        } catch (err) {
          this.errorBanner = String(err);
          this.render();
        }
      });

      document.getElementById("btn-clear-llm-key")?.addEventListener("click", async () => {
        try {
          this.appSettings = await picoApi.saveAppSettings({
            llmEndpoint: this.appSettings.llm_endpoint,
            llmModel: this.appSettings.llm_model,
            llmApiKey: "",
            githubToken: null,
          });
          this.successBanner = "Cleared LLM API key.";
          this.render();
        } catch (err) {
          this.errorBanner = String(err);
          this.render();
        }
      });
    }

    // Vibe diff modal buttons
    document.getElementById("btn-vibe-accept")?.addEventListener("click", () => {
      void this.handleAcceptVibe();
    });
    document.getElementById("btn-vibe-reject")?.addEventListener("click", () => {
      void this.handleRejectVibe();
    });
  }
}

const appRoot = document.getElementById("app");
if (appRoot) {
  const app = new PicoStudioApp(appRoot);
  void app.init();
}
