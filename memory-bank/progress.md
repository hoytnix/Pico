# Progress & Roadmap: Pico

## Build & Test Status
- **Rust Backend (`cargo test`):** 23 passed; 0 failed (28.36s).
- **Frontend Bundle (`pnpm build`):** Builds cleanly (`tsc --noEmit && vite build`), generating Tailwind v4 CSS bundle in 4.51s.
- **Backend Type Check (`cargo check --tests`):** Clean, 0 warnings, 0 errors in 0.18s.

## Feature Matrix & Completed Modules

| Module / Requirement | Status | Notes |
| --- | --- | --- |
| Vibe Full Site Context & 970/1000 Prompt Architecture | Completed | **Injected complete site context into Vibe agent prompt with prompt caching optimizations:**<br>1) Added `VibeSiteFile`, `VibeSiteSnapshot`, and `load_vibe_site_snapshot` in backend, returning `site.yaml`, all `models/*.yaml`, `assets/theme.css`, all `templates/**/*.jinja`, and content manifest.<br>2) Created `src/lib/vibe/promptBuilder.ts` (166 LOC) with 970/1000 prompt architecture: zero terminal invariants, engine guardrails (fuel limit 50k, macro rules, autoescape `| safe`, AST dotted variable rules), ReAct cognitive workflow, and cached pre-loaded site source code.<br>3) Added `listContentFiles`, `listTemplates`, and `readSiteConfig` discovery tools to `vibeAgent.ts` (260 LOC).<br>4) Wired `activePostPath` and prompt loading into `useVibeChat.ts` and `VibePane.tsx`.<br>5) Added backend integration unit test `test_load_vibe_site_snapshot` (23/23 tests passing). |
| Refactoring Roadmap (All 9 Tasks) | Completed | **100% of tasks completed across backend and frontend:**<br>1) Dissolved 1,379-line `llm.rs` into `src-tauri/src/llm/` (`registry.rs`, `formatters.rs`, `validation.rs`, and test suite).<br>2) Abstracted `ExtractedFrontmatter` in `post.rs` eliminating duplication across `content.rs` and `content_scanner.rs`.<br>3) Disentangled 615-line `validation.rs` into `slug.rs`, `document.rs`, and `guardrails.rs`.<br>4) Modularized 454-line `render.rs` into `common.rs`, `singles.rs`, `collections.rs`, `tags.rs`, `home_and_404.rs`.<br>5) Purged legacy settings ghost and extracted `state.rs`.<br>6) Extracted `useVibeChat.ts` (346 LOC), reducing `VibePane.tsx` from 388 to 109 LOC.<br>7) Decoupled `ModelsEditor.tsx` via `ModelSidebar.tsx` and `ModelFieldTable.tsx`.<br>8) Decoupled `EditorToolbar.tsx` via `HeadingDropdown.tsx` and `TableControls.tsx` (reduced to 201 LOC).<br>9) Decoupled `PreviewPane.tsx` via `PreviewHeaderBar.tsx` and `BuildBanner.tsx` (reduced to 168 LOC).<br>**Zero production files exceed 400 LOC.** |
| Version Bump `v0.1.2` | Completed | Bumped version to `0.1.2` across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. Tagged `v0.1.2` and pushed to origin. |
| Sequential Multi-File Diffs in Vibe | Completed | Fixed proposal clobbering where subsequent edits overwrote the single proposal slot. Diff cards now display sequentially in order (`Diff X of Y`) inside the message bubble, with independent Accept & Save / Reject controls per card, batch Accept All / Reject All actions, individual loading indicators, and agent prompt guidance. |
| Google Models (Gemma 4 & Gemini 3.6 / 3.1) | Completed | Added support for Gemma 4 26B (`gemma-4-26b-a4b-it`), Gemma 4 31B (`gemma-4-31b-it`), Gemini 3.6 Flash (`gemini-3.6-flash`), and Gemini 3.1 Flash-Lite (`gemini-3.1-flash-lite`). Expanded `default_models` in `gemini-native` and `gemini-openai`, added alias normalization (`normalize_google_model_id` in backend and `normalizeGoogleModelId` in frontend), updated thinking controls & tests, and added friendly labels in Settings. |
| Vibe Model Editing Support | Completed | Added full support for Vibe to work with collection models (`models/*.yaml`). Added `readModelFile` and `listModels` tools, `proposeEdit` with `targetType: 'model'`, YAML schema and field type validation guardrails, unified diff preview, `save_model_yaml` backend command, and auto-refresh of site collections upon acceptance. |
| Version Bump `v0.1.1` | Completed | Bumped version to `0.1.1` across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. Tagged `v0.1.1` and pushed to origin. |
| Vibe Stream Error & Log Flooding Fix | Completed | 1) Hooked `onError` into `ToolLoopAgent` and `VibePane.tsx` to capture stream failures (e.g. 503 high demand / 429 quota) instead of swallowing them. 2) Replaced false "completed the requested operation" fallback text with honest error and status messaging. 3) Normalized theme CSS path handling. 4) Suppressed `get_preview_revision` 600ms polling logs in `apiInvoke`. |
| Verbose File Logging (`~/.pico.log`) | Completed | Implemented unified file logger in `src-tauri/src/logger.rs` and frontend client in `src/lib/logger.ts`. Writes millisecond-timestamped logs to `~/.pico.log`, capturing Rust panics, all Tauri commands, build steps, preview watcher events, git actions, and frontend console/window errors. |
| Vibe Proposal Loop Fix | Completed | Resolved issue where `toolApproval: 'user-approval'` halted `ToolLoopAgent` before `execute()` could run, causing blank responses. `proposeEdit` now generates diffs during execution and returns status to the loop so the model streams its explanation, while keeping file saves gated behind manual user Accept/Reject. |
| Memory Bank Git Commit Rule | Completed | Added Step 4 and strict guardrail to `GEMINI.md` to ensure memory bank updates are committed to Git immediately after modification. |
| Gemini Native CSP Fix | Completed | Added `https://generativelanguage.googleapis.com`, `https://api.meta.ai`, and `https://openrouter.ai` to CSP `connect-src` in `tauri.conf.json` to eliminate WebKit "Load failed" fetch blocking. Also mapped `load failed` in `formatLlmError`. |
| Preset Model Selector Update | Completed | Removed `gemini-3-flash-preview` and `gemini-2.5-flash`; replaced with `gemini-3.8-flash`, `gemini-3.5-flash`, and `gemini-3.5-flash-lite`. |
| Top Navbar Settings Button | Completed | Removed Settings from the Code sub-navigation. Added a gear-only icon button to the top navbar to the right of "Switch Site" to open the Settings modal. |
| Dropdown Visibility Fix | Completed | Fixed white-on-white text in select dropdowns by specifying `color-scheme: dark;`, solid dark option backgrounds, and option text colors in `styles.css` and components. |
| CI Release Workflow Fixes | Completed | 1) Removed `version: 12` from `release.yml` for `pnpm/action-setup`. 2) Added `permissions: contents: write`. 3) Tag-gated publishing (`if: startsWith(github.ref, 'refs/tags/')`). 4) Tag `v0.1.0` created and pushed to origin to trigger release publishing. |
| TailwindCSS v4 Setup (§2) | Completed | `tailwindcss` + `@tailwindcss/vite` via `pnpm`, Vite plugin registered, `@import "tailwindcss";` in `src/styles.css`. No v3 config files. |
| Single Visible Pane Shell (§3) | Completed | Root shell `flex h-screen flex-col overflow-hidden`. Exactly one pane visible at a time via `block` / `hidden`. Zero pane stacking. |
| State Preservation (§3) | Completed | Components remain mounted with `hidden`; chat drafts, pending diffs, editor content, and preview scroll survive all tab switches. |
| Bottom Bar Dock (§3) | Completed | Direct `shrink-0 w-full` flex child at the bottom of the shell. Always visible, houses strictly Vibe, Preview, Code tabs. No floating buttons. |
| Code Pane Sub-Navigation (§3) | Completed | Content, Models, Templates, Theme, Settings live strictly inside Code pane as sub-navigation. Zero topbar tab buttons. |
| No-Site State (§3) | Completed | Displays only Open / Create Site + Recent Sites in main area when no site is open. Empty states and Vibe composer hidden. |
| TipTap & ProseMirror Isolation (§3) | Completed | Scoped base styles in `editor.css` ensure typography and editor formatting remain intact under Tailwind preflight. |
| CaffeineMD TipTap Editor | Completed | Reused and adapted CaffeineMD TipTap v3 editor, custom markdown serializer, dark theme, and toolbars. |
| Vibe Multi-Turn Conversation | Completed | AI SDK v7 (`ai` Core, `@ai-sdk/react`, `@ai-sdk/google`, `@ai-sdk/openai`), `ToolLoopAgent`, streaming replies, multi-turn history. |
| Tool Approval & Diff Card | Completed | `toolApproval: { proposeEdit: 'user-approval' }`. Propose edit returns diff card with Accept/Reject gate; saves via normal path. |
| Allowed Tools | Completed | Exactly 4 tools: `readContentFile`, `readTemplateFile`, `readThemeCss`, `proposeEdit`. |
| LLM Providers Dropdown | Completed | `gemini-native`, `openai`, `openrouter`, `meta`, `gemini-openai`, `custom`. Masked keys in `~/.config/pico/settings.json`. |
| Thinking Controls | Completed | Exact mapping for Gemini 3/4 (levels), Gemini 2.5 (budgets), OpenAI (reasoning_effort), Muse Spark (disabled with note). |
| Google Search Grounding | Completed | Enabled only for Gemini Native with Sources panel; disabled on others with exact required text. |
| Single Preview Server Bar | Completed | Exactly ONE live preview server bar in Preview, transitioning empty → starting → live URL without duplicates. |
| Build Pipeline & Export | Completed | 10-step atomic build pipeline, sitemap, robots, RSS/Atom, search.json, WebP variants, folder export, GitHub Pages. |
| Starters Embedded | Completed | Blog, Portfolio, Landing, and Docs embedded in binary with `include_dir`. |

## Known Issues
- None. All acceptance criteria and verification tests pass.

## Deferred (Non-Goals)
- Streaming directly into files without user diff review
- Autonomous multi-file rewrites or vibe-generated models/plugins
- Server-side LLM proxy or Node runtime
- Usage billing dashboards or hardcoded pricing tables
