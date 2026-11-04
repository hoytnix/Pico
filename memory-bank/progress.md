# Progress & Roadmap: Pico

## Build & Test Status
- **Rust Backend (`cargo test`):** 19 passed; 0 failed (28.64s).
- **Frontend Bundle (`pnpm build`):** Builds without errors (`tsc --noEmit && vite build`).
- **Release Executable (`pnpm run build:bin`):** Compiles standalone binary to `target/release/pico` (optimized 64-bit ELF on Linux, with embedded starter templates).
- **Backend Type Check (`cargo check`):** Clean, 0 warnings (0.19s).

## Feature Matrix & Completed Modules

| Module / Requirement | Status | Notes |
| --- | --- | --- |
| React + Vite Frontend (§3) | Completed | Clean React 19 + TypeScript + Vite architecture using pnpm exclusively. |
| CaffeineMD TipTap Editor (§2, §4) | Completed | Reused and adapted CaffeineMD TipTap v3 editor, custom markdown serializer, dark theme, and toolbars. Milkdown/Crepe completely removed. |
| Vibe Multi-Turn Conversation (§5) | Completed | AI SDK v7 (`ai` Core, `@ai-sdk/react`, `@ai-sdk/google`, `@ai-sdk/openai`), `ToolLoopAgent`, streaming replies, multi-turn history. |
| Tool Approval & Diff Card (§5) | Completed | `toolApproval: { proposeEdit: 'user-approval' }`. Propose edit returns diff card with Accept/Reject gate; saves via normal path and updates editor. |
| Allowed Tools (§5.2) | Completed | Exactly 4 tools: `readContentFile`, `readTemplateFile`, `readThemeCss`, `proposeEdit`. No shell, no arbitrary fs. |
| LLM Providers Dropdown (§6) | Completed | `gemini-native`, `openai`, `openrouter`, `meta`, `gemini-openai`, `custom`. Masked keys in `~/.config/pico/settings.json`. |
| Thinking Controls (§7) | Completed | Exact mapping for Gemini 3/4 (levels), Gemini 2.5 (budgets), OpenAI (reasoning_effort), Muse Spark (disabled with note). |
| Google Search Grounding (§7) | Completed | Enabled only for Gemini Native with Sources panel; disabled on others with exact required text. |
| Test Connection (§6, §10) | Completed | Minimal 1-token call with 30s timeout, charge notice, and exact error formatting for 401, 402, 404, 400, 429, timeout. |
| Desktop Layout & Navigation (§9) | Completed | Desktop-only app. Bottom bar is the ONLY tab location (Vibe, Preview, Code). Zero tab buttons in topnav. |
| Single Preview Server Bar (§9) | Completed | Exactly ONE live preview server bar in Preview, transitioning empty → starting → live URL without duplicates. |
| Zero-Remount State (§5.1, §9) | Completed | Tab switching preserves chat history, drafts, pending diffs, editor content, and preview scroll. |
| Models & Templates Editors (§3) | Completed | Visual models schema editor and MiniJinja template editor with variable inspector and one-click revert preserved intact. |
| Build Pipeline & Export (§3) | Completed | 10-step atomic build pipeline, sitemap, robots, RSS/Atom, search.json, WebP variants, folder export, GitHub Pages. |
| Starters Embedded (§3) | Completed | Blog, Portfolio, Landing, and Docs embedded in binary with `include_dir`. |
| Security Checks (§11) | Completed | No tracked keys, no keys in `out/` or built JS, zero `milkdown`/`crepe` in `src/` or `package.json`. |

## Known Issues
- None. All acceptance criteria and verification tests pass.

## Deferred (Non-Goals)
- Streaming directly into files without user diff review
- Autonomous multi-file rewrites or vibe-generated models/plugins
- Server-side LLM proxy or Node runtime
- Usage billing dashboards or hardcoded pricing tables
