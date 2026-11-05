# Pico — Desktop Static Site Generator

**Pico** is a local-first, visual desktop static site generator designed for writers, designers, and creators who do not want to use a terminal, git, or command-line build tools. Open or create a site folder, write Markdown in a TipTap WYSIWYG editor ported from CaffeineMD, customize MiniJinja templates with model-aware guardrails and one-click revert, preview with live reload, chat with a full-conversation Vibe AI assistant via AI SDK v7, and build or export complete static sites — all from a single desktop GUI.

---

## Deviations from prompt

No deviations.

---

## Historical Migration: Milkdown/Crepe → TipTap & Vanilla TS → React

Pico's frontend has been rebuilt from vanilla TypeScript into **React + TypeScript + Vite** using `pnpm` exclusively. As part of this transition:
- **Milkdown & Crepe Removed:** Milkdown kit, Crepe, and associated Vue dependencies have been completely purged from `package.json`, styles, and source code.
- **CaffeineMD TipTap Editor Integrated:** Pico now uses the TipTap Markdown editor engine ported directly from sibling project `CaffeineMD`, adapted to Pico's desktop dark theme, Tauri IPC commands, and model-driven frontmatter forms.
- **AI SDK v7 Vibe Conversation:** The previous one-shot prompt-to-diff Vibe flow has been superseded by a full multi-turn conversational AI studio powered by Vercel AI SDK v7 (`ai` Core, `@ai-sdk/react`, `@ai-sdk/google`, `@ai-sdk/openai`), featuring streaming responses, tool approval gates, and unified diff proposal review with Accept/Reject.

---

## Reused from CaffeineMD

The core Markdown editor in Pico is ported directly from CaffeineMD (`~/Dev/CaffeineMD`):

| Aspect | Implementation Details |
|---|---|
| **TipTap Version** | TipTap v3 (`^3.31.4`, resolved to `3.32.2`) with ProseMirror PM packages. |
| **Package List** | `@tiptap/core`, `@tiptap/react`, `@tiptap/starter-kit`, `@tiptap/pm`, `@tiptap/extension-code-block-lowlight`, `@tiptap/extension-image`, `@tiptap/extension-link`, `@tiptap/extension-placeholder`, `@tiptap/extension-table`, `@tiptap/extension-table-row`, `@tiptap/extension-table-header`, `@tiptap/extension-table-cell`, `@tiptap/extension-task-list`, `@tiptap/extension-task-item`, `@tiptap/extension-underline`, `@tiptap/suggestion`, `lowlight`, `highlight.js`, `lucide-react`, `diff`. |
| **Extensions in Use** | `StarterKit` (headings 1–6, paragraphs, bold, italic, strike, horizontal rule), `CodeBlockLowlight` with lowlight syntax engine, `Placeholder`, `Underline`, `Link`, `TaskList`, `TaskItem`, `Table`, `TableRow`, `TableHeader`, `TableCell`, `Image`, `SlashCommands`. |
| **Markdown Parsing & Serialization** | Ported CaffeineMD's serializers (`markdownToEditorHtml`, `editorHtmlToMarkdown`, `markdownTable`, `markdownInline`, `codeLanguageDetector`). On post load, Markdown body is converted to ProseMirror-compatible HTML; on user edit/auto-save, editor HTML is walked through DOM nodes and serialized back to clean Markdown without losing indentation, code fences, or list structures. |
| **Frontmatter Separation** | Frontmatter is strictly isolated from the TipTap editor body. It is presented and edited in the visual Post Properties form driven by `models/*.yaml`. Rust backend serialization preserves YAML header fields byte-for-byte upon save. |
| **Images** | Handled via Pico's asset flow: picking an image invokes `save_image` / `save_image_bytes` to copy into `content/<collection>/<slug>/` or `assets/` and inserts a relative Markdown image tag (`![alt](path)`). |
| **Code Blocks** | Syntax-highlighted code blocks with language auto-detection and lowlight formatting. |
| **Tables** | GFM Markdown table parsing and serialization (`\| Col 1 \| Col 2 \|`), 3x3 table insertion, and column/row additions and deletions. |
| **Lists** | Unordered bullet lists, sequential numbered lists, and interactive checkbox task lists (`- [x]` / `- [ ]`). |
| **Toolbar & Styling** | Adapted from CaffeineMD's toolbar popovers and ProseMirror CSS (`src/styles/editor.css`), styled for Pico's desktop dark theme with heading selectors, formatting controls, lists, tables, image insertion, undo/redo. |
| **Save / onUpdate / External Updates** | `onUpdate` triggers a debounced (300ms) save handler marking the document Unsaved until Tauri's save command succeeds. Post switching and Vibe Accept update content via the editor's `editor.commands.setContent(targetHtml, { emitUpdate: false })` set-content path without remounting the editor or losing cursor state. |

---

## Deferred from CaffeineMD

Features present in CaffeineMD not needed for Pico v1 have been intentionally deferred:
- Split view / live side-by-side preview pane inside editor
- Focus mode & Typewriter scrolling HUD
- Scratchpad / Quick-switcher overlay
- Table of Contents HUD / Document Outline Minimap
- Complex Table GUI cell merging and alignment popovers beyond basic GFM grid
- WebAssembly code execution sandbox (`wasmSandbox` runner)
- WikiLink graph canvas view

---

## React + TipTap + AI SDK v7

### Architecture & Frontend-Direct Flow
- **Desktop-Only Shell:** React 19 + TypeScript + Vite. No mobile view, no mobile breakpoint, no mobile reflow.
- **Bottom Bar as ONLY Tab Location:** Exactly one place to switch modes: the bottom bar containing **Vibe**, **Preview**, and **Code** tabs. Zero tab buttons appear in the topbar.
- **Zero-Remount Shell:** Switching between tabs hides/reveals panes via CSS, keeping DOM nodes mounted so chat messages, composer drafts, pending diffs, editor content, and preview scroll positions are 100% preserved.
- **AI SDK Core Frontend-Direct:** Runs AI SDK v7 (`ai`, `@ai-sdk/react`, `@ai-sdk/google`, `@ai-sdk/openai`) directly in the Tauri webview. No Node server at runtime, no Next.js API routes, no server-side LLM proxies. API keys are fetched at call-time via Tauri IPC (`get_llm_key`) and held only in memory for the call.

### Providers Supported

| Provider ID | Label in UI | AI SDK v7 Provider | Default Models | Key Label & Dashboard Link |
|---|---|---|---|---|
| `gemini-native` | Gemini (Native) | `@ai-sdk/google` (native Gemini) | `gemini-4-argon`, `gemini-3.8-flash`, `gemini-3.6-flash`, `gemini-3.5-flash`, `gemini-3.5-flash-lite`, `gemini-3.1-flash-lite`, `gemma-4-31b-it`, `gemma-4-26b-a4b-it` | Gemini API key — [Google AI Studio](https://aistudio.google.com/app/apikey) |
| `openai` | OpenAI | AI SDK OpenAI provider | `gpt-4o-mini` | OpenAI API key — [OpenAI Platform](https://platform.openai.com/api-keys) |
| `openrouter` | OpenRouter | AI SDK OpenAI-compatible (`https://openrouter.ai/api/v1`) | Custom model field required (e.g. `anthropic/claude-3.5-sonnet`) | OpenRouter API key — [OpenRouter Keys](https://openrouter.ai/keys) (Sends `HTTP-Referer` and `X-Title: Pico` headers) |
| `meta` | Muse Spark | AI SDK OpenAI-compatible (`https://api.meta.ai/v1`) | `muse-spark-1.3`, `muse-spark-1.3-contributor`, `muse-wonder-1.0` | Model API key — [Meta Model API Dashboard](https://api.meta.ai/dashboard/keys) |
| `gemini-openai` | Gemini (OpenAI mode) | AI SDK OpenAI-compatible (`https://generativelanguage.googleapis.com/v1beta/openai`) | `gemini-4-argon`, `gemini-3.8-flash`, `gemini-3.6-flash`, `gemini-3.5-flash`, `gemini-3.5-flash-lite`, `gemini-3.1-flash-lite`, `gemma-4-31b-it`, `gemma-4-26b-a4b-it` | Gemini API key — [Google AI Studio](https://aistudio.google.com/app/apikey) |
| `custom` | Custom (OpenAI-compatible) | AI SDK OpenAI-compatible (user-entered base URL) | Custom model field required | Custom API key (Enforces `https://`, except local `http://localhost:*` or `http://127.0.0.1:*`) |

### Thinking and Grounding Rules

- **Thinking Dropdown:** Low / Medium / High / Provider Default (send nothing). Default is Medium in UI.
  - Gemini 3/4 & Gemma 4 (`gemini-4-argon`, `gemini-3.8-flash`, `gemini-3.6-flash`, `gemini-3.5-flash`, `gemini-3.5-flash-lite`, `gemini-3.1-flash-lite`, `gemma-4-31b-it`, `gemma-4-26b-a4b-it`): maps to `thinkingLevel: "low" | "medium" | "high"`. Never sends `MINIMAL` or token budgets (both return HTTP 400 on Gemini 4).
  - Gemini 2.5: maps to token budgets `1024 | 8192 | 24576`.
  - OpenAI-compatible track: maps to `reasoning_effort: "low" | "medium" | "high"`.
  - Muse Spark: Thinking dropdown is **disabled** with the note: *"Muse Spark manages its own reasoning."*
  - Cost Notice: Settings displays *"Higher thinking may increase cost and time to first token."*
- **Google Search Grounding (Gemini Native ONLY):**
  - Grounding toggle is enabled **only** when Provider = Gemini Native, using `@ai-sdk/google`'s native search tool.
  - For all other providers, the toggle is disabled with the exact text: *"Grounding requires Gemini Native, not an OpenAI provider."*
  - When grounding sources are returned, they appear in a dedicated **Grounding Sources** panel under the diff card. Citations are never auto-injected into user files unless proposed by the LLM.

### Vibe Chat Tools & Safety Rules

- **Allowed Tools (Strictly Limited to 4):**
  1. `readContentFile`: read frontmatter and Markdown body of a content file.
  2. `readTemplateFile`: read MiniJinja code of a template file.
  3. `readThemeCss`: read theme CSS variables from `assets/theme.css`.
  4. `proposeEdit`: propose an edit to content, template, or theme CSS.
- **Explicit User Approval Gate:** `toolApproval: { proposeEdit: 'user-approval' }`. The model cannot apply changes directly. Proposed edits render as a unified diff card in the chat. Clicking **Accept & Save** writes the file via normal Tauri save commands (with template validation & revert backup intact) and updates the open editor. Clicking **Reject** discards the proposal and continues the conversation.
- **No-Key Rule:** When no API key is configured for the active provider, the composer and send button are disabled with the exact text: `"Configure an LLM API key in Settings to use Vibe."`

### Error Handling & Test Connection

- **Test Connection Button:** Minimal 1-token test call ("Reply with exactly: OK", 30-second timeout) with cost notice: *"A test call may incur a small provider charge."*
- **Provider Error Surfaces Verbatim:**
  - **401 / Invalid Key:** `"Invalid API key for <provider>. Check the key in Settings."`
  - **402 / Billing (Muse):** `"Billing is not configured for Muse. Update the payment method in the Meta Model API dashboard, then retry."`
  - **404 / Model Not Found:** `"Model <id> was not found for <provider>. Check the model ID or pick another model."`
  - **400 / Thinking:** `"Thinking setting is not supported for this model/provider combination."`
  - **429 / Rate Limit:** `"Rate limited by <provider>. Wait and retry."`
  - **Timeout / Network:** `"Could not reach <provider>. Check your connection and the base URL, then retry."`

### Settings Migration

Existing `settings.json` files from legacy single-endpoint Pico builds automatically migrate to the `openai` provider entry on first launch, preserving base URL, model, and key without loss.

### Verification Status: Live vs. Error-Path

- **Automated Tests:** 19 Rust unit/integration tests (`cargo test`) pass cleanly (28.64s).
- **Backend Type Check:** `cargo check` clean (0.19s, 0 warnings).
- **Standalone Binary Build:** `pnpm run build:bin` builds optimized release binary `target/release/pico` (30.60s).
- **Frontend Build:** `pnpm build` (`tsc --noEmit && vite build`) passes with 0 errors.
- **Testing Mode Note:** In the automated verification environment without live paid commercial API keys, LLM endpoints were validated via offline mock HTTP servers, schema contracts, and error-path unit tests. Pico never fakes LLM responses.

---

## Deferred (Non-Goals in MVP)

- Streaming directly into files without user diff review (all edits require Accept/Reject)
- Autonomous multi-file rewrites or vibe-generated models/plugins
- Server-side LLM proxy or Node runtime
- Usage billing dashboards or hardcoded pricing tables
- Real-time multi-user collaboration or CMS server backends

---

## Stack

| Layer | Technology |
| --- | --- |
| Desktop Shell & IPC | **Rust + Tauri v2** (`tauri`, `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-shell`, `tauri-plugin-http`) |
| Templating Engine | **MiniJinja v2** (`loader`, `fuel`, `builtins`, `AutoEscape::Html`) |
| Markdown Rendering | **pulldown-cmark** (GFM tables, strikethrough, task lists, code language classes) |
| Image Processing | **image** crate (`FilterType::Lanczos3` WebP hero max 1200px & card thumb max 600px) |
| Preview Server & Watcher | **Axum** + `tower-http::services::ServeDir` + **notify** (300ms debounced live reload) |
| WYSIWYG Editor | **TipTap v3** (ported from CaffeineMD, `@tiptap/react`, `@tiptap/starter-kit`, lowlight) |
| Conversational AI | **Vercel AI SDK v7** (`ai`, `@ai-sdk/react`, `@ai-sdk/google`, `@ai-sdk/openai`) |
| Frontend Bundler | **React 19 + TypeScript + Vite**, managed exclusively with **pnpm** |

---

## Run & Build Commands

```bash
# 1. Install frontend dependencies (pnpm exclusively)
pnpm install

# 2. Run backend test suite
cargo test

# 3. Type-check and build React frontend bundle
pnpm build

# 4. Check backend compilation
cargo check

# 5. Build standalone desktop release executable:
pnpm run build:bin
# -> Outputs optimized standalone executable to: target/release/pico

# 6. Launch Pico in desktop dev mode:
pnpm tauri dev
```

---

## License

**(k) Kopimi** — Copying is encouraged. You are free to copy, modify, remix, and distribute this work for any purpose. All rights unreserved. See [LICENSE](file:///home/oloty/Dev/Pico/LICENSE).
