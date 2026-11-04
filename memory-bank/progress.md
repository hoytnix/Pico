# Progress & Roadmap: Pico

## Build & Test Status
- **Rust Backend (`cargo test`):** 19 passed; 0 failed (28.37s).
- **Frontend Bundle (`pnpm build`):** Builds cleanly (`tsc --noEmit && vite build`), generating Tailwind v4 CSS bundle.
- **Release Executable (`pnpm run build:bin`):** Compiles standalone release binary to `target/release/pico` (optimized 64-bit ELF on Linux, with embedded starter templates) in 30.50s.
- **Backend Type Check (`cargo check`):** Clean, 0 warnings (0.18s).

## Feature Matrix & Completed Modules

| Module / Requirement | Status | Notes |
| --- | --- | --- |
| CI Release Workflow Fixes | Completed | 1) Removed `version: 12` from `release.yml` for `pnpm/action-setup`. 2) Added `permissions: contents: write`. 3) Tag-gated publishing (`if: startsWith(github.ref, 'refs/tags/')`) so releases are only published when a new tag occurs. |
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
