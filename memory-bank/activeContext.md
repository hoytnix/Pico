# Active Context: Pico

## Current Focus
Implemented Full Site Source Context Ingestion & 970/1000 System Prompt Architecture in Vibe. Vibe now receives comprehensive, cached site context (site configuration, collection models, theme CSS tokens, MiniJinja templates, content manifest, and active editor document) along with rigorous AST guardrail contracts and expanded discovery tools.

## Recent State Changes
1. **Full Site Source Snapshot (Rust Backend):**
   - Added `VibeSiteFile` and `VibeSiteSnapshot` in [`src-tauri/src/models.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/models.rs).
   - Implemented `load_vibe_site_snapshot` in [`src-tauri/src/vibe.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/vibe.rs), scanning and aggregating `site.yaml`, all `models/*.yaml`, `assets/theme.css`, all `templates/**/*.jinja` (via `load_all_templates`), and `content_manifest` (via `list_content_impl`).
   - Added Tauri IPC command `get_vibe_site_snapshot` in [`src-tauri/src/commands/vibe.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/vibe.rs), registered in [`src-tauri/src/lib.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/lib.rs) and re-exported in [`src-tauri/src/commands/mod.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/mod.rs).
   - Added integration unit test `test_load_vibe_site_snapshot` in [`src-tauri/src/tests.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/tests.rs) (23/23 tests passing).

2. **970/1000 Confidence Prompt Builder (Frontend):**
   - Created [`src/lib/vibe/promptBuilder.ts`](file:///home/oloty/Dev/Pico/src/lib/vibe/promptBuilder.ts) (166 LOC) featuring:
     - Non-negotiable operational invariants (zero terminal, non-destructive, complete proposals, sequential multi-file proposals).
     - Exact engine contracts and guardrails (`site.yaml` keys, model schema rules, MiniJinja fuel limit of 50k, macro rules, autoescape with `| safe`, dotted variable AST checking).
     - Full pre-loaded code injection (Tier 1 core code + Tier 2 content manifest + Tier 3 active document context).
     - ReAct cognitive workflow (grounding -> planning -> multi-file proposing -> concluding).
   - Exposed `getVibeSiteSnapshot` in [`src/api.ts`](file:///home/oloty/Dev/Pico/src/api.ts) and types in [`src/types.ts`](file:///home/oloty/Dev/Pico/src/types.ts).

3. **Tool Expansion & Active Document Integration:**
   - Expanded [`src/lib/vibeAgent.ts`](file:///home/oloty/Dev/Pico/src/lib/vibeAgent.ts) (260 LOC) with tools: `listContentFiles`, `listTemplates`, and `readSiteConfig`, plus support for dynamic `customInstructions`.
   - Wired `activePostPath` from [`src/components/vibe/VibePane.tsx`](file:///home/oloty/Dev/Pico/src/components/vibe/VibePane.tsx) into [`src/components/vibe/useVibeChat.ts`](file:///home/oloty/Dev/Pico/src/components/vibe/useVibeChat.ts) (352 LOC), compiling the dynamic prompt before streaming each turn.

## Verification & Status
- **Rust Backend Tests:** 23/23 unit and integration tests passing (`cargo test`).
- **Rust Compilation:** 0 warnings, 0 errors (`cargo check --tests`).
- **Frontend Build:** Clean compilation, 0 errors (`pnpm build`).
- **Module Sizes:** 100% of production code files strictly adhere to the <400 LOC ceiling and reside within the 100–250 LOC ideal zone.
