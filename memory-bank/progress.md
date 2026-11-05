# Progress & Roadmap: Pico

## Build & Test Status
- **Rust Backend (`cargo test`):** 21 passed; 0 failed (28.73s).
- **Frontend Bundle (`pnpm build`):** Builds cleanly (`tsc --noEmit && vite build`), generating Tailwind v4 CSS bundle.
- **Release Executable (`pnpm run build:bin`):** Compiles standalone release binary to `target/release/pico` (optimized 64-bit ELF on Linux, with embedded starter templates) in 44.67s.
- **Backend Type Check (`cargo check`):** Clean, 0 warnings.

## Feature Matrix & Completed Modules

| Module / Requirement | Status | Notes |
| --- | --- | --- |
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
