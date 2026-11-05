# Active Context: Pico

## Current Focus
Completed Phase 2 (Core Engine Modularization) and Phase 3 (Frontend Component Decoupling) from the Refactoring Roadmap: decomposed monolithic backend build pipeline and Tauri commands into clean sub-crates and modules, and decoupled complex frontend components into focused subcomponents within the 100–250 LOC ideal zone.

## Recent State Changes
1. **Refactoring Roadmap Phase 2 (Tasks 1 & 4 - Core Engine Modularization):**
   - **Task 1: Deconstructed 1,484-Line Build Pipeline into Sub-Crate (`src-tauri/src/builder/`):**
     - [`assets.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/assets.rs) (103 LOC): directory copying and WebP variants generation.
     - [`content_scanner.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/content_scanner.rs) (237 LOC): markdown scanning, frontmatter parsing, draft filtering, and attachment discovery.
     - [`feeds.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/feeds.rs) (101 LOC): sitemap.xml, robots.txt, RSS, and Atom feeds generation.
     - [`models_loader.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/models_loader.rs) (128 LOC): `site.yaml` loading, model validation, and template discovery.
     - [`pagination.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/pagination.rs) (31 LOC): pagination chunking and tag grouping.
     - [`post.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/post.rs) (156 LOC): post rendering, template variable mapping, and image variant derivation.
     - [`render.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/render.rs) (454 LOC): single/collection/tag/home/404 rendering and live-reload script injection.
     - [`pipeline.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/pipeline.rs) (379 LOC): 10-step atomic swap orchestrator.
     - [`mod.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/mod.rs) (19 LOC): backward-compatible re-exports.
   - **Task 4: Decomposed 2,163-Line `src-tauri/src/lib.rs` into Modular Commands & Services:**
     - Reduced [`src-tauri/src/lib.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/lib.rs) from 2,163 LOC down to 87 LOC (AppState, command registration, and app launch).
     - Extracted [`starters.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/starters.rs) (85 LOC): embedded starter pack asset loading and directory extraction.
     - Extracted [`storage.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/storage.rs) (112 LOC): settings persistence and recent site history.
     - Extracted [`tests.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/tests.rs) (668 LOC): Rust unit and integration test suite.
     - Created [`commands/`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/) sub-module:
       - [`site.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/site.rs) (258 LOC): site open, create, export, build, preview, and GitHub Pages publishing.
       - [`content.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/content.rs) (325 LOC): post CRUD, listing, image saving, and size checking.
       - [`models.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/models.rs) (142 LOC): model listing, schema saving, and deletion.
       - [`templates.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/templates.rs) (214 LOC): template CRUD, validation, context inspection, and revert backup.
       - [`settings.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/settings.rs) (99 LOC): app/LLM settings and frontend log bridge.
       - [`vibe.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/vibe.rs) (44 LOC): context retrieval and proposal evaluation.
       - [`mod.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/mod.rs) (38 LOC): shared state guards and re-exports.

2. **Refactoring Roadmap Phase 3 (Tasks 6 & 9 - Frontend Component Decoupling):**
   - **Task 6: Decoupled `VibePane.tsx` (reduced from 607 LOC down to 297 LOC):**
     - Created [`src/components/vibe/types.ts`](file:///home/oloty/Dev/Pico/src/components/vibe/types.ts) (26 LOC) defining `ProposalItem`, `Message`, and `ProposalCallbacks`.
     - Created [`src/lib/vibe/proposalApplier.ts`](file:///home/oloty/Dev/Pico/src/lib/vibe/proposalApplier.ts) (28 LOC) handling disk persistence and IPC updates for accepted proposals across content, templates, themes, and models.
     - Created [`src/components/vibe/VibeHeader.tsx`](file:///home/oloty/Dev/Pico/src/components/vibe/VibeHeader.tsx) (44 LOC) for header branding, active provider/model badge, and history reset.
     - Created [`src/components/vibe/VibeComposer.tsx`](file:///home/oloty/Dev/Pico/src/components/vibe/VibeComposer.tsx) (87 LOC) for chat prompt textarea, keyboard shortcuts, and stream stop/send controls.
     - Created [`src/components/vibe/MessageList.tsx`](file:///home/oloty/Dev/Pico/src/components/vibe/MessageList.tsx) (177 LOC) for conversation bubbles, avatars, thinking indicators, sequential diff card list, and batch proposal controls.
   - **Task 9: Decomposed `ContentEditor.tsx` (reduced from 409 LOC down to 217 LOC):**
     - Created [`src/components/code/content/PostListSidebar.tsx`](file:///home/oloty/Dev/Pico/src/components/code/content/PostListSidebar.tsx) (114 LOC) for collection tabs, post filtering, new post creation, draft badges, and post deletion.
     - Created [`src/components/code/content/FrontmatterForm.tsx`](file:///home/oloty/Dev/Pico/src/components/code/content/FrontmatterForm.tsx) (89 LOC) for post title, live save status (saving/saved/unsaved), draft toggle, date, slug, and tag array inputs.

3. **Phase 1 & 3 Quick Wins & SSOT Modules (Tasks 2, 8, 5, 3):**
   - **Task 2:** Purged dead legacy `vibe_edit` backend Tauri command and uncalled LLM functions.
   - **Task 8:** Created [`src/lib/slug.ts`](file:///home/oloty/Dev/Pico/src/lib/slug.ts) canonical utility.
   - **Task 5:** Created [`src/lib/llmProviders.ts`](file:///home/oloty/Dev/Pico/src/lib/llmProviders.ts) SSOT module.
   - **Task 3:** Decomposed `SettingsModal.tsx` into `LlmSettingsTab.tsx` and `SiteSettingsTab.tsx`.

## Verification & Status
- **Rust Backend Tests:** 22/22 unit and integration tests passing (`cargo test`).
- **Rust Compilation:** 0 warnings, 0 errors (`cargo check --tests`).
- **Frontend Build:** Clean compilation, 0 errors (`pnpm build`).
- **Module Sizes:** All refactored modules strictly adhere to the <400 LOC hard ceiling and reside within or near the 100–250 LOC ideal zone.
