# Pico — Desktop Static Site Generator

**Pico** is a local-first, visual desktop static site generator designed for writers, designers, and makers who do not want to use a terminal, git, or command-line build tools. Open or create a site folder, edit Markdown content in a Milkdown Crepe WYSIWYG editor, customize MiniJinja templates with model-aware guardrails and one-click revert, preview with live reload, and build or export a complete static site — all from a desktop GUI.

---

## Deviations from prompt

No deviations.

---

## Mobile AI Studio Layout

Pico features an adaptive dual layout inspired by the mobile experience of Google AI Studio:

- **Narrow Viewports (`< 768px`)**: Top navigation is replaced with a fixed bottom tab bar placed within thumb reach, featuring exactly three tabs — **Vibe**, **Preview**, and **Code**.
- **Wide Viewports (`≥ 768px`)**: Standard desktop navigation remains intact (Content, Models, Templates, Theme, Preview, Build & Export, Settings), and the mobile bottom tab bar is hidden.
- **Zero-Remount State Preservation**: Switching between tabs never remounts panes or reloads components. Milkdown Crepe WYSIWYG editor content and unsaved edits, Vibe conversation drafts, pending Vibe unified diffs (with Accept/Reject buttons), preview scroll position, and file selections all survive tab switches.

### Three-Tab Navigation Mapping

| Bottom Tab | What It Contains in Pico | Pico Screens & Workflows That Live Here |
|---|---|---|
| **Vibe** | The Vibe AI assistant conversation: prompt composer, proposed edits, and unified diff review. | - Vibe prompt composer + unified diff + **Accept & Save** / **Reject & Discard** for the active target (post, template, or theme).<br>- Target and Provider/Model indicator in header.<br>- Strict no-key rule: when no provider key is configured, displays exact text: `"Configure an LLM API key in Settings to use Vibe."`<br>- Pending diffs survive tab switches.<br>- Sticky composer positioned above the tab bar. |
| **Preview** | Live Axum preview server with exactly ONE control bar and live site preview. | - Live Axum preview (localhost URL, 300ms debounced live reload, `DRAFT` badges on draft posts in preview only).<br>- Single Live Preview Server control bar with live URL, Copy URL, Rebuild & Refresh, and Open in Browser.<br>- **Build Site** and **Export** actions live directly in the Preview header on narrow viewports for instant access. |
| **Code** | Site file, collection, template, and model editing suite. | - **Content Posts**: Browse collections and posts; tap a post to edit frontmatter and body in Milkdown Crepe (Vibe box routes to Vibe tab with target = content).<br>- **Templates**: `templates/*.jinja` and `macros/*.jinja`; tap to edit with guardrail validation & one-click revert (Vibe box routes to Vibe tab with target = template).<br>- **Models**: Visual collection schema editor (`models/*.yaml`), field schema table, and type pickers preserved intact.<br>- **Theme**: `assets/theme.css` editor and CSS variable controls (Vibe box routes to Vibe tab with target = theme).<br>- **Site Configuration**: Visual `site.yaml` editor.<br>- **Assets**: `assets/` media browser and disk image importer.<br>- **Header Actions**: Site Switcher, New Site, and Settings gear icon (opens LLM and Site settings). |

### App Shell & Internal Scrolling (§5)

- **Viewport Fill**: Uses `100dvh` (with `100vh` fallback) and `overflow: hidden` on the root shell to eliminate mobile chrome jumping and dual scrollbars.
- **Dedicated Internal Scrolling**: Each pane (Vibe conversation, Preview iframe container, and Code browser/editor) scrolls independently (`overflow-y: auto`). There is never a document-level scrollbar.
- **Safe-Area Inset & Thumb Reach**: The bottom tab bar is fixed at 64–72px height with `safe-area-inset-bottom` padding to avoid home indicator clipping. All scrollable panes have bottom padding matching the tab bar height plus safe-area inset so content is never hidden behind the navigation. Touch targets are a minimum of 48px.
- **Accessibility**: Tabs use semantic `<button role="tab">` elements in a `role="tablist"` container with live `aria-selected` attributes and keyboard navigation (ArrowLeft, ArrowRight, Home, End). Active state is distinguished via pill background and font weight, never color alone.

---

## Duplicate Live Preview Server Bug Fix (M84)

### 1. Root Cause Analysis
- **Duplicate Mounting:** Following the persistent 3-pane shell introduction, `#pane-preview` was mounted as a persistent DOM section inside `.panes-container`. However, on desktop wide viewports, the desktop router in `#pane-code` (`renderActiveTabPane()`) also mounted a second preview component (`renderPreviewPane()`) when the top-nav Preview tab was active.
- **CSS Visibility Conflict:** On desktop (`≥ 768px`), `.studio-pane` styles set `display: flex` for all panes except `.pane-chat`. Consequently, both `#pane-preview` and `#pane-code` were simultaneously rendered in `.panes-container`.
- **Stale State Freeze:** During initial workspace mount, `#pane-preview` was rendered before the preview server was started, leaving its URL empty and loading state active. Subsequent preview start events only updated the newly rendered second preview inside `#pane-code`, resulting in two control bars stacked vertically (one empty with "Starting preview server…", and one below with the live URL).

### 2. Resolution Implemented
- **Single Authoritative Mount Point:** Removed duplicate `renderPreviewPane()` from `renderActiveTabPane()`. `#pane-preview` is now the single, authoritative preview mount point across all viewport widths.
- **In-Place State Transition:** The single Live Preview Server control bar renders a transient "Starting preview server…" state in place of the URL while starting, and seamlessly updates in-place to the live URL (`http://127.0.0.1:<port>/`) once ready. No second bar or stale elements are created.
- **Unified Pane Visibility:** Synchronized `.studio-pane.pane-active` visibility across mobile and desktop. On wide viewports, selecting Preview displays `#pane-preview` and hides `#pane-code`; selecting any editor view displays `#pane-code` and hides `#pane-preview`.
- **Tab Naming Correction:** Standardized mobile bottom tab navigation strictly to **Vibe**, **Preview**, and **Code** (§2).
- **Zero Chrome Leakage:** Verified that built site output in `out/` contains 0 Pico chrome, and that exactly one preview server/port is used per site.

---

## LLM Providers

Pico features dual-track LLM architecture behind a unified Vibe interface:
- **Track 1 (Gemini Native):** First-class Google Gemini integration using the official `@google/genai` TypeScript SDK in the frontend, enabling Google Search Grounding with web citations and native thinking level/budget controls.
- **Track 2 (OpenAI-compatible):** High-performance Rust backend client POSTing to `{base_url}/chat/completions` for OpenAI, OpenRouter, Meta Muse Spark, Gemini in OpenAI mode, and user-configured custom endpoints.

### Provider Presets

| Provider ID | Label in UI | Base URL | Default Models | Key Label & Dashboard Link |
|---|---|---|---|---|
| `gemini-native` | Gemini (Native) | `https://generativelanguage.googleapis.com/v1beta` | `gemini-4-argon`, `gemini-3-flash-preview`, `gemini-2.5-flash` | Gemini API key — [Google AI Studio](https://aistudio.google.com/app/apikey) |
| `openai` | OpenAI | `https://api.openai.com/v1` | `gpt-4o-mini` | OpenAI API key — [OpenAI Platform](https://platform.openai.com/api-keys) |
| `openrouter` | OpenRouter | `https://openrouter.ai/api/v1` | Custom model text field required (e.g. `google/...`, `meta/...`) | OpenRouter API key — [OpenRouter Keys](https://openrouter.ai/keys) (Sends `HTTP-Referer` and `X-Title: Pico` headers) |
| `meta` | Muse Spark | `https://api.meta.ai/v1` | `muse-spark-1.3`, `muse-spark-1.3-contributor`, `muse-wonder-1.0` | Model API key — [Meta Model API Dashboard](https://api.meta.ai/dashboard/keys) |
| `gemini-openai` | Gemini (OpenAI mode) | `https://generativelanguage.googleapis.com/v1beta/openai` | `gemini-4-argon`, `gemini-2.5-flash` | Gemini API key — [Google AI Studio](https://aistudio.google.com/app/apikey) |
| `custom` | Custom (OpenAI-compatible) | User-entered (`https://` required; `http://localhost:*` or `http://127.0.0.1:*` allowed for local dev) | Custom model text field | Custom API key |

> [!NOTE]
> Keys are stored separately per provider in `~/.config/pico/settings.json` and are sent only to the active provider's endpoint at call time. The UI displays `SET` or `NOT SET` badges and never renders raw keys.

### Thinking Level Controls

The Thinking dropdown (Low / Medium / High, default: Medium) maps controls according to provider and model capabilities:

| Track / Model Family | Low | Medium | High | Forbidden Parameters |
|---|---|---|---|---|
| **Gemini Native (Gemini 3 & 4)**, including `gemini-4-argon` | `thinkingConfig.thinkingLevel: "LOW"` | `"MEDIUM"` | `"HIGH"` | `thinkingLevel: "MINIMAL"`, any `thinkingBudget` (both return HTTP 400 on Gemini 4) |
| **Gemini Native (Gemini 2.5)** | `thinkingConfig.thinkingBudget: 1024` | `8192` | `24576` | `thinkingLevel` (2.5 uses token budgets, not levels) |
| **OpenAI Track** (`openai`, `openrouter`, `gemini-openai`, `custom`) | `reasoning_effort: "low"` | `"medium"` | `"high"` | `thinkingConfig` in the same request as `reasoning_effort` |
| **Muse Spark** (`meta`) | Thinking dropdown is **disabled** with note: *"Muse Spark manages its own reasoning."* | Same | Same | Do not send `reasoning_effort` to Meta (Muse Spark manages reasoning internally; sending effort risks HTTP 400) |

- **Default behavior:** If no thinking value is explicitly selected (or set to Default), Pico omits thinking parameters and lets the provider apply its model default.
- **Cost notice:** Settings displays *"Higher thinking may increase cost and time to first token."*

### Google Search Grounding (Gemini Native ONLY)

- **Honest Gating:** The Google Search Grounding toggle is enabled **only** when Provider is set to `Gemini (Native)`. For all OpenAI-track providers, the toggle is disabled in the GUI with the note: *"Grounding requires Gemini Native, not an OpenAI provider."*
- **Backend Rejection:** If a forced request attempts to enable grounding on a non-Gemini Native provider, the backend rejects it with: *"Grounding requires Gemini Native, not an OpenAI provider."*
- **Sources Panel:** When grounding is enabled and sources are returned in `groundingMetadata.groundingChunks`, Pico renders a dedicated **Google Search Grounding Sources** panel below the unified diff with clickable links opening in the default browser. Citations are never auto-inserted into the Markdown file unless present in the LLM's proposed text.

### Test Connection

- In **Settings**, clicking **Test Connection** executes a lightweight 1-token call to `{current provider, current model}` using that provider's saved key.
- **Inline Success:** Displays `Success: Connection to {model} succeeded in {elapsed_ms}ms.`
- **Inline Error:** Surfaces mapped, human-readable error messages:
  - **401 / Invalid Key:** `"Invalid API key for {provider}. Check the key in Settings."`
  - **402 / Muse Billing:** `"Muse Spark billing limit reached or payment method missing. Visit the Meta Model API dashboard."`
  - **404 / Model Not Found:** `"Model {model} was not found for {provider}. Check the model ID or pick another model."`
  - **400 / Thinking Incompatible:** `"Thinking setting is not supported for this model/provider combination."`
  - **429 / Rate Limit:** `"Rate limited by {provider}. Wait and retry."`
  - **Network / Timeout:** `"Could not reach {provider}. Check your connection and the base URL, then retry."`
- A notice beside the button reminds users: *"A test call may incur a small provider charge."*

### Settings Migration

Existing `settings.json` files from earlier single-endpoint Pico builds are automatically migrated upon startup:
- The legacy `llm_endpoint`, `llm_model`, and `llm_api_key` are assigned to the `openai` provider entry.
- The user's active provider defaults to `openai` if migrated, preserving existing workflows without data loss.
- Gemini and Muse Spark provider entries remain unconfigured until the user explicitly sets keys for them.

### Verification Status: Live vs. Error-Path

- **Automated Tests:** 19 Rust unit/integration tests (`cargo test`) pass with 0 failures, covering:
  - Provider registry definitions, URLs, and order.
  - Strict base URL validation (`https://` enforcement, local loopback exceptions).
  - Exact thinking parameter mappings across Gemini 3/4, Gemini 2.5, OpenAI, and Muse Spark.
  - Google Search Grounding gating and rejection.
  - Isolated provider key routing via mock HTTP server (verifying each provider receives its own key and headers).
  - Legacy `settings.json` migration.
  - Named error message formatting (401, 402, 404, 400, 429, timeout).
- **Frontend Verification:** `pnpm build` (`tsc --noEmit && vite build`) compiles with zero TypeScript errors.
- **Live vs. Mock Execution:** As no live commercial API credentials were provided in the test environment, live external network calls were not performed during automated testing. All provider paths, headers, payload structures, and error states were validated via offline mock HTTP servers and error-path unit tests. Pico never fakes LLM responses.

---

## Stack

| Layer | Technology |
| --- | --- |
| Desktop Shell & IPC | **Rust + Tauri v2** (`tauri`, `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-shell`, `tauri-plugin-http`) |
| Templating Engine | **MiniJinja v2** (`loader`, `fuel`, `builtins`, `AutoEscape::Html`) |
| Markdown Rendering | **pulldown-cmark** (GFM tables, strikethrough, task lists, fenced code language classes) |
| Image Processing | **image** crate (`FilterType::Lanczos3` WebP hero max 1200px & card thumb max 600px) |
| Preview Server & Watcher | **Axum** + `tower-http::services::ServeDir` + **notify** (300ms debounced live reload) |
| WYSIWYG Editor | **Milkdown Crepe** (`@milkdown/crepe`, `@milkdown/kit`, `Frame` light/dark theme) |
| Frontend Bundler | **Vanilla TypeScript + Vite**, managed exclusively with **pnpm** |

---

## Run & Verification Instructions

```bash
# 1. Install frontend dependencies (pnpm exclusively)
pnpm install

# 2. Run the Rust test suite (frontmatter validation, slugify, pagination, tag grouping,
#    WebP image variants, sitemap/RSS, MiniJinja macro & unknown-variable guardrails, E2E build)
cargo test

# 3. Build the TypeScript + Vite frontend bundle
pnpm build

# 4. Type-check / compile the Tauri Rust backend
cargo check

# 5. Launch Pico in desktop development mode
pnpm tauri dev

# 6. Compile to standalone release executable (for the current machine):
pnpm run build:bin
# -> Outputs optimized standalone executable to: target/release/pico (or pico.exe on Windows)

# 7. Compile and build full OS installer/package bundles (deb/AppImage, dmg/app, msi/exe):
pnpm run build:app
```

---

## Multi-Platform Executable Distribution (Mac, Windows, Linux)

Pico includes an automated GitHub Actions release workflow in [`.github/workflows/release.yml`](file:///.github/workflows/release.yml):
- **Linux (`ubuntu-22.04`)**: Compiles `x86_64-unknown-linux-gnu` standalone binary + `.deb` & `.AppImage`.
- **macOS (`macos-latest`)**: Compiles Apple Silicon / Intel universal `.dmg`, `.app` bundle, and standalone binary.
- **Windows (`windows-latest`)**: Compiles `x86_64-pc-windows-msvc` `.msi`, NSIS setup `.exe`, and standalone `pico.exe`.
- **Zero-Dependency Starters**: All 4 starter templates (Blog, Portfolio, Landing, Docs) are embedded directly into the compiled executable via `include_dir`, so the distributed binary runs completely self-contained even when executed outside the repository folder.

Pushing a version tag (e.g. `git tag v0.1.0 && git push origin v0.1.0`) triggers the matrix build and automatically uploads executable release assets to GitHub Releases.

---

## User Site Layout

Every Pico site on disk follows this exact directory structure:

```text
my-site/
  site.yaml
  models/
    blog.yaml
    project.yaml
  templates/
    skeleton.jinja
    blog.jinja
    blog-list.jinja
    project.jinja
    project-list.jinja
    tag.jinja
    home.jinja
    404.jinja
    macros/
      cards.jinja
  content/
    blog/*.md
    projects/*.md
  assets/
    theme.css
    highlight.css
    highlight.js
  out/
```

- **Markdown is the source of truth on disk:** Posts live in `content/<collection>/*.md` with YAML frontmatter at the top and Markdown body below.
- **Frontmatter GUI Form:** Generated dynamically from `models/<collection>.yaml`. Frontmatter is never placed inside the Milkdown editor body.
- **Attachments & Images:** Dropping, pasting, or picking an image copies the file into `content/<collection>/<slug>/` (or `assets/` for site-wide assets) and inserts a relative Markdown link — never base64. Images larger than 2 MB display a friendly warning and are resized into WebP variants at build time.

---

## Starter Templates Included (`starters/`)

1. **`starters/blog`** — Latest posts home page, paginated blog index (10 posts/page), single post template, tag index pages, RSS 2.0 + Atom feeds, and client-side search.
2. **`starters/portfolio`** — Featured projects showcase, project detail template with automatic attachment image gallery, and notes/about pages.
3. **`starters/landing`** — Single-page product hero, feature cards, and conditional contact form (rendered only when `site.form_endpoint` is set in `site.yaml`).
4. **`starters/docs`** — Documentation layout with sidebar navigation ordered from content, documentation detail pages, and instant `search.json` search bar in `skeleton.jinja`.

---

## How Preview, Build, Export, and Vibe Work

### Preview
- Clicking **Preview** runs the build pipeline with `include_drafts = true` (rendering draft posts with a visible `DRAFT` badge), starts an in-process Axum + `tower-http::ServeDir` static server on a dynamic `127.0.0.1:<port>`, and opens the live site inside Pico (with a copyable localhost URL and "Open in Browser" button).
- A background `notify` watcher monitors `content/`, `templates/`, `assets/`, and `site.yaml` (debounced to 300ms), automatically rebuilding and triggering live reload.

### Build (10-Step Pipeline)
1. Walks `content/**/*.md`, parses YAML frontmatter + Markdown body, and validates fields against `models/<collection>.yaml` (reporting missing required fields, unknown fields, or type mismatches with file + line + plain-language reason).
2. Filters out `draft: true` posts (unless `include_drafts` is enabled), auto-slugifies empty slugs from titles, and checks slug uniqueness per collection.
3. Renders Markdown bodies to HTML with `pulldown-cmark` (tables, strikethrough, task lists, fenced code `language-<lang>` classes for `highlight.js`).
4. Sorts collections by `date` descending then `title` ascending, chunks into pages of 10, and groups/paginates tag indexes.
5. Renders with MiniJinja (`AutoEscape::Html`, fuel limit `50_000`) in exact order: single pages (`/blog/<slug>/index.html`), collection list pages (`/blog/`, `/blog/page/2/`), tag pages (`/tags/<tag>/`), home page (`home.jinja`), and `404.html`.
6. Copies `assets/` → `out/assets/` unchanged and copies page attachments → `out/<collection>/<slug>/`.
7. Generates WebP image variants using the `image` crate (`FilterType::Lanczos3`): hero max 1200px wide (`<stem>-hero.webp`) and card thumbnail max 600px wide (`<stem>-thumb.webp`) written only to `out/`, never back into source. SVG, animated GIF, and AVIF files are copied unchanged.
8. Generates `sitemap.xml`, `robots.txt`, RSS 2.0 (`feed.xml`), Atom (`atom.xml`), and `search.json`.
9. Bundles offline `highlight.js` + `highlight.css` in `skeleton.jinja` so fenced code blocks are syntax-highlighted on the published site even completely offline.
10. Builds into `out.tmp/` first and atomically swaps into `out/` only if there are zero build errors, preserving any previous good build on error.

### Export & GitHub Pages Publish
- **Export:** Copies `out/` to any destination folder picked by the user. Guaranteed to work 100% offline without git or a GitHub account.
- **Publish to GitHub Pages:** Best-effort helper that checks for `git` and a GitHub `origin` remote, pushes `out/` to the `gh-pages` branch, and optionally configures Pages via the GitHub API if a token is set in Settings. If any prerequisite is missing, it displays friendly instructions without failing the build.

### Vibe Editing
- Available on the **Content Editor**, **Template Editor**, and **Theme Panel**.
- Uses an OpenAI-compatible chat completions endpoint and API key configured in **Settings**. When no API key is configured, the Vibe button is disabled with the exact message: `"Configure an LLM API key in Settings to use Vibe."`
- Never auto-applies changes: displays a unified diff (`old` vs. `proposed`) with **Accept & Save** and **Reject & Discard** buttons. Template and Theme Vibe outputs pass through the same MiniJinja and model variable guardrails as manual edits.

---

## Deferred (Non-Goals in MVP, §9)

The following items are intentionally deferred outside MVP scope per §9:
- Plugins / extensions system
- i18n / multi-language routing
- Pagefind binary indexing (MVP uses `search.json` client-side search)
- AVIF image conversion and responsive `srcset` generation (MVP generates 1200px hero & 600px thumb WebP variants)
- API-only GitHub Pages artifact deployment
- Real-time multi-user collaboration, custom comments/form backend servers, user accounts, or analytics/telemetry

---

## License

**(k) Kopimi** — Copying is encouraged. You are free to copy, modify, remix, and distribute this work for any purpose. All rights unreserved. See [LICENSE](file:///home/oloty/Dev/Pico/LICENSE).

