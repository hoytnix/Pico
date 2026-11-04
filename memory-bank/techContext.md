# Technical Context: Pico

## Technology Stack

### Backend (Rust 2021)
- **Tauri v2:** `tauri` (v2.12), `tauri-build` (v2.7), `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-shell`, `tauri-plugin-http`.
- **Templating:** `minijinja` (v2.24) with `loader`, `fuel` (50,000 steps limit), `builtins`, and `AutoEscape::Html`.
- **Markdown:** `pulldown-cmark` (v0.12) with GFM tables, strikethrough, task lists, and syntax-highlight classes.
- **Image Optimization:** `image` (v0.25) with `Lanczos3` resizing to WebP (1200px hero, 600px thumb).
- **Preview Server & Watcher:** `axum` (v0.7), `tower-http` (v0.5), `tokio` (v1), `notify` (v6 with 300ms debounce).
- **Diff & Vibe:** `similar` (v2.7), `reqwest` (v0.12 with rustls).
- **Asset Embedding:** `include_dir` (v0.7) for embedding all starter packs into the binary.
- **Trash / Cleanup:** `trash` (v5) for native OS trash integration with fallback to file removal.

### Frontend (TypeScript + Vite)
- **TypeScript:** 5.9
- **Bundler:** Vite 6.4
- **WYSIWYG Editor:** `@milkdown/crepe` (7.23), `@milkdown/kit` (7.23).
- **Google GenAI SDK:** `@google/genai` (2.32) for native Track 1 Gemini calls, thinking level mapping, and Google Search Grounding.
- **Package Manager:** `pnpm` exclusively (v12.15).

## Build & Distribution Scripts

| Script | Purpose |
| --- | --- |
| `pnpm dev` | Starts Vite dev server for frontend development |
| `pnpm build` | Compiles TypeScript and builds Vite bundle to `dist/` |
| `pnpm run build:bin` | Builds optimized standalone release executable to `target/release/pico` |
| `pnpm run build:app` | Builds full OS installer packages via Tauri |
| `pnpm tauri dev` | Launches Pico in Tauri desktop dev mode |
| `cargo test` | Runs Rust unit and integration test suite |
| `cargo check` | Type-checks backend compilation |

## Security & Capabilities
Tauri capabilities in `src-tauri/capabilities/default.json` enforce deny-by-default:
- Scoped file dialogs (`dialog:allow-open`, `dialog:allow-save`, `dialog:allow-ask`).
- Shell open restricted strictly to localhost URLs (`^https?://(127\.0\.0\.1|localhost)(:[0-9]+)?(/.*)?$`).
- Network requests scoped strictly to LLM provider endpoints and GitHub API:
  - `https://generativelanguage.googleapis.com/*`
  - `https://api.meta.ai/*`
  - `https://api.openai.com/*`
  - `https://openrouter.ai/*`
  - `https://api.github.com/*`
