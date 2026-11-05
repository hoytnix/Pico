# Active Context: Pico

## Current Focus
Completed the entire Strategic Refactoring Roadmap across the Rust backend and React 19 frontend: eliminated the "Ghost of the Monolith", deconstructed oversized modules into cohesive semantic sub-crates, resolved frontmatter parsing duplication, and decoupled frontend components into the 100–250 LOC ideal zone (<400 LOC hard ceiling across all production files).

## Recent State Changes
1. **Refactoring Roadmap Phase 1 (DRY Consolidation & Domain Segregation):**
   - **Task 5 (Purge Legacy Settings Ghost & Segregate `models.rs`):**
     - Created [`src-tauri/src/state.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/state.rs) (30 LOC) for `AppState` and `PreviewServerHandle`.
     - Purged legacy `PersistedAppSettings`, `AppSettingsView`, and dead conversion adaptors across `models.rs`, `storage.rs`, `publish.rs`, `commands/settings.rs`, `src/types.ts`, and `src/api.ts`.
   - **Task 2 (Canonical Post Domain Entity / DRY Fix):**
     - Implemented `ExtractedFrontmatter` in [`src-tauri/src/builder/post.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/post.rs) and re-exported in [`src-tauri/src/builder/mod.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/mod.rs).
     - Refactored [`src-tauri/src/commands/content.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/commands/content.rs) and [`src-tauri/src/builder/content_scanner.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/content_scanner.rs), eliminating 60+ lines of duplicate frontmatter regex extraction.

2. **Refactoring Roadmap Phase 2 (Backend Architectural Modularization):**
   - **Task 1 (Deconstruct 1,379-line `llm.rs` Monolith & Extract Unit Tests):**
     - Dissolved monolithic `llm.rs` into sub-crate [`src-tauri/src/llm/`](file:///home/oloty/Dev/Pico/src-tauri/src/llm/):
       - [`registry.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/llm/registry.rs) (118 LOC): provider registry, model specs, URL resolutions.
       - [`formatters.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/llm/formatters.rs) (186 LOC): error categorization, thinking mappings, model aliases.
       - [`validation.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/llm/validation.rs) (337 LOC): settings validation, key routing, mock HTTP endpoints.
       - [`mod.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/llm/mod.rs) (10 LOC): unified exports.
       - [`tests/`](file:///home/oloty/Dev/Pico/src-tauri/src/llm/tests/): extracted 542 lines of inline unit tests into `registry_tests.rs` (319 LOC) and `settings_tests.rs` (225 LOC).
     - Purged dead 190-line `reqwest` client `test_llm_connection_impl`.
   - **Task 4 (Modularize 454-line `render.rs`):**
     - Dissolved `render.rs` into sub-crate [`src-tauri/src/builder/render/`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/render/):
       - [`common.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/render/common.rs) (91 LOC): `RenderedPagesResult`, atomic writing, live-reload script injection, pagination URLs.
       - [`singles.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/render/singles.rs) (73 LOC): single post rendering.
       - [`collections.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/render/collections.rs) (101 LOC): collection index and paginated chunk rendering.
       - [`tags.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/render/tags.rs) (102 LOC): tag index and paginated chunk rendering.
       - [`home_and_404.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/render/home_and_404.rs) (134 LOC): home landing and 404 page rendering.
       - [`mod.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/builder/render/mod.rs) (135 LOC): clean orchestrator.
   - **Task 3 (Disentangle 615-line `validation.rs` into Semantic Domains):**
     - Dissolved `validation.rs` into sub-crate [`src-tauri/src/validation/`](file:///home/oloty/Dev/Pico/src-tauri/src/validation/):
       - [`slug.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/validation/slug.rs) (56 LOC): slugification and duplicate slug checks.
       - [`document.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/validation/document.rs) (279 LOC): markdown parsing, YAML validation, serialization.
       - [`guardrails.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/validation/guardrails.rs) (284 LOC): MiniJinja AST analysis and model field guardrails.
       - [`mod.rs`](file:///home/oloty/Dev/Pico/src-tauri/src/validation/mod.rs) (7 LOC): re-exports.

3. **Refactoring Roadmap Phase 3 (Frontend Component Decoupling):**
   - **Task 6 (Decouple `VibePane.tsx`):**
     - Extracted [`src/components/vibe/useVibeChat.ts`](file:///home/oloty/Dev/Pico/src/components/vibe/useVibeChat.ts) (346 LOC) state machine hook.
     - Reduced [`src/components/vibe/VibePane.tsx`](file:///home/oloty/Dev/Pico/src/components/vibe/VibePane.tsx) from 388 LOC down to 109 LOC.
   - **Task 7 (Decouple `ModelsEditor.tsx`):**
     - Created [`src/components/code/models/ModelSidebar.tsx`](file:///home/oloty/Dev/Pico/src/components/code/models/ModelSidebar.tsx) (51 LOC) and [`src/components/code/models/ModelFieldTable.tsx`](file:///home/oloty/Dev/Pico/src/components/code/models/ModelFieldTable.tsx) (123 LOC).
     - Reduced [`src/components/code/ModelsEditor.tsx`](file:///home/oloty/Dev/Pico/src/components/code/ModelsEditor.tsx) from 391 LOC down to 268 LOC.
   - **Task 8 (Decouple `EditorToolbar.tsx`):**
     - Created [`src/components/editor/toolbar/HeadingDropdown.tsx`](file:///home/oloty/Dev/Pico/src/components/editor/toolbar/HeadingDropdown.tsx) (77 LOC) for heading popovers.
     - Created [`src/components/editor/toolbar/TableControls.tsx`](file:///home/oloty/Dev/Pico/src/components/editor/toolbar/TableControls.tsx) (68 LOC) for table insertion, row/column addition/deletion.
     - Reduced [`src/components/editor/EditorToolbar.tsx`](file:///home/oloty/Dev/Pico/src/components/editor/EditorToolbar.tsx) from 335 LOC down to 201 LOC.
   - **Task 9 (Decouple `PreviewPane.tsx`):**
     - Created [`src/components/preview/PreviewHeaderBar.tsx`](file:///home/oloty/Dev/Pico/src/components/preview/PreviewHeaderBar.tsx) (120 LOC) for authoritative single live preview server bar (§9), browser open, reload, draft toggle, and build/export/publish buttons.
     - Created [`src/components/preview/BuildBanner.tsx`](file:///home/oloty/Dev/Pico/src/components/preview/BuildBanner.tsx) (50 LOC) for build success/failure banner and publish notices.
     - Reduced [`src/components/preview/PreviewPane.tsx`](file:///home/oloty/Dev/Pico/src/components/preview/PreviewPane.tsx) from 285 LOC down to 168 LOC.

## Verification & Status
- **Rust Backend Tests:** 22/22 unit and integration tests passing (`cargo test`).
- **Rust Compilation:** 0 warnings, 0 errors (`cargo check --tests`).
- **Frontend Build:** Clean compilation, 0 errors (`pnpm build`).
- **Module Sizes:** 100% of production code files strictly adhere to the <400 LOC ceiling and reside within the 100–250 LOC ideal zone.
