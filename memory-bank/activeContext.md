# Active Context: Pico

## Current Focus
Bumped version from `0.1.1` to `0.1.2`, tagged release `v0.1.2`, and pushed to remote to trigger the CI release pipeline.

## Recent State Changes
1. **Version Bump to `0.1.2` & Release Tag `v0.1.2`:**
   - Updated package version to `0.1.2` across [package.json](file:///home/oloty/Dev/Pico/package.json), [Cargo.toml](file:///home/oloty/Dev/Pico/src-tauri/Cargo.toml), and [tauri.conf.json](file:///home/oloty/Dev/Pico/src-tauri/tauri.conf.json).
   - Recompiled backend with `cargo check` and rebuilt frontend bundle with `pnpm build`.
   - Created git tag `v0.1.2` and pushed commits and tag to `origin/main` to trigger the CI release workflow.
2. **Sequential Multi-File Diff Rendering in Vibe AI (`src/components/vibe/VibePane.tsx` & `src/lib/vibeAgent.ts`):**
   - **Root Cause:** When an instruction required edits to more than one file, the agent executed `proposeEdit` for each file. Previously, `Message` and `pendingProposal` stored only a single proposal reference. Each new proposal immediately overwrote the previous one, causing earlier diffs to appear briefly and disappear before the user could review or accept them.
   - **Multi-Proposal State:** Added `ProposalItem` interface extending `ProposeEditResult` with unique `id` and independent `status` (`'pending' | 'accepted' | 'rejected'`). Stored proposals as an array (`proposals?: ProposalItem[]`) on `Message`, appending each proposal emitted during the turn.
   - **Sequential UI & Independent Approval:** Rendered all diff cards sequentially in order (`Diff X of Y`) inside the assistant message bubble. Each diff card features dedicated `Accept & Save` and `Reject` buttons that update the specific proposal without altering or discarding others.
   - **Batch Action Bar:** When more than one proposal is pending, an action bar displays with pending count and `Accept All` / `Reject All` buttons for rapid review.
   - **Per-Proposal Processing State:** Added `processingProposalIds` state to disable buttons during disk save on a per-card basis.
   - **Vibe Instructions & Tool IDs:** Added unique proposal IDs in `vibeAgent.ts` `proposeEdit` tool results and instructed Vibe to invoke `proposeEdit` sequentially for each affected file when addressing multi-file prompts.
2. **Google Models Support (Gemma 4 & Gemini 3.6 / 3.1):**
   - **Backend Provider Registry (`src-tauri/src/llm.rs`):**
     - Expanded `default_models` in `get_provider_registry()` for both `gemini-native` and `gemini-openai` to include `gemini-3.6-flash`, `gemini-3.1-flash-lite`, `gemma-4-31b-it`, and `gemma-4-26b-a4b-it`.
     - Implemented `normalize_google_model_id` to normalize model aliases (e.g. `gemma-4-26b`, `gemma 4 26b`, `gemma-4-26b-it` -> `gemma-4-26b-a4b-it`; `gemma-4-31b` -> `gemma-4-31b-it`; `gemini-3.6` -> `gemini-3.6-flash`; `gemini-3.1` -> `gemini-3.1-flash-lite`; and `google/` OpenRouter prefixed variants).
     - Applied model normalization in `test_llm_connection_impl` and `execute_backend_vibe_turn`.
     - Added comprehensive unit tests in `test_normalize_google_model_id` and updated `test_provider_registry_ids_urls_and_order` and `test_thinking_mapping_table_exact_spec` verifying thinking level mapping for Gemma 4 and Gemini 3.6 / 3.1.
   - **Frontend Vibe Agent & Normalization (`src/lib/vibeAgent.ts`):**
     - Implemented `normalizeGoogleModelId` helper to normalize model IDs in frontend calls.
     - Updated `createVibeModel` and `buildProviderOptions` to normalize models before initializing `@ai-sdk/google` or OpenAI-compatible providers, ensuring thinkingLevel is passed for all Gemini 3/4 and Gemma 4 models.
   - **UI & Settings Modal (`src/components/settings/SettingsModal.tsx`):**
     - Added `MODEL_DISPLAY_LABELS` to present human-readable model labels in the preset dropdown (e.g. `gemini-3.6-flash (Gemini 3.6 Flash)`, `gemma-4-26b-a4b-it (Gemma 4 26B)`).
     - Normalized `effectiveModel` so test connections and saves always use canonical model IDs even if shorthand was typed in the custom model field.
   - **Documentation (`README.md`):**
     - Updated providers table and thinking mode notes in `README.md` with the new Google models.
2. **Vibe Collection Model Support (`models/*.yaml`):**
   - **Backend (`src-tauri/src/vibe.rs` & `models.rs`):**
     - Updated `VibeTarget` kind to include `"model"`.
     - Added `"model"` system prompt instructions in `build_vibe_system_prompt`, explaining model schema structure, valid field types (`string`, `date`, `tags`, `image`, `bool`), and existing collection models in the site.
     - Implemented model file reading in `read_vibe_target_content` for `models/<name>.yaml` (handling non-existent models gracefully as empty for new proposals).
     - Added model proposal validation in `evaluate_vibe_proposal`: validates proposed YAML as a valid `ModelDef`, checks for required names/labels, validates field types against allowed types, and computes unified diffs.
     - Added unit tests in `src-tauri/src/lib.rs` verifying valid model proposals generate unified diffs without validation errors, and invalid YAML / unsupported field types are flagged with descriptive guardrail violations.
   - **Tauri IPC Command & API (`src-tauri/src/lib.rs` & `src/api.ts`):**
     - Added `save_model_yaml` Tauri command to validate and persist model YAML to `models/<slug>.yaml`, creating the collection content directory `content/<slug>` automatically. Registered in `tauri::generate_handler!`.
     - Added `saveModelYaml` to `picoApi` in [api.ts](file:///home/oloty/Dev/Pico/src/api.ts).
   - **Vibe Agent & Tools (`src/lib/vibeAgent.ts`):**
     - Added `readModelFile` tool to read YAML schema of any model in `models/`.
     - Added `listModels` tool so Vibe can discover all models, their fields, and templates in one call.
     - Updated `proposeEdit` schema to accept `targetType: 'model'`, with path normalization for `models/<name>.yaml`.
     - Updated agent instructions guiding Vibe on how to inspect and propose modifications or additions to collection models.
   - **UI & Diff Approval (`src/components/vibe/VibePane.tsx`, `DiffCard.tsx`, `src/App.tsx`):**
     - Updated `handleAcceptProposal` in `VibePane.tsx` to handle `proposal.targetType === 'model'` by invoking `picoApi.saveModelYaml`.
     - Added `onModelUpdatedFromVibe` callback wired to `App.tsx` to reload `currentSite` via `openSite`, keeping models synchronized in the Models sub-navigation, post editor, and content lists.
     - Updated `DiffCard.tsx` guardrail error header to generic "Guardrail Violation" for template and model guardrails.
     - Updated Vibe welcome message in `VibePane.tsx` to announce model editing capabilities.
2. **Version Bump to `0.1.1` & Release Tag `v0.1.1`:**
   - Updated package version to `0.1.1` in [package.json](file:///home/oloty/Dev/Pico/package.json), [Cargo.toml](file:///home/oloty/Dev/Pico/src-tauri/Cargo.toml), and [tauri.conf.json](file:///home/oloty/Dev/Pico/src-tauri/tauri.conf.json).
   - Rebuilt frontend and backend metadata.
   - Created git tag `v0.1.1` and pushed commits and tag to `origin/main` to trigger CI release workflow.
2. **Vibe Stream Error Handling & False Completion Fix:**
   - Root cause: In AI SDK v7 (`ai` Core), `streamText` suppresses stream errors so they do not crash streaming iterators, routing them instead to lifecycle callbacks. When an error occurred during subsequent steps of a tool loop (such as Google API 503 high demand or 429 quota exhaustion on Gemini 3.5 Flash Lite), `streamResult.textStream` terminated cleanly with 0 deltas without throwing in `for await`.
   - In `VibePane.tsx`, when `accumulatedText` was empty and `pendingProposal` was undefined, a hardcoded fallback previously emitted `'I have completed the requested operation.'`, falsely assuring the user that changes were made when in reality the model had failed or aborted.
   - Wired `onError` into `createVibeAgent` in [vibeAgent.ts](file:///home/oloty/Dev/Pico/src/lib/vibeAgent.ts) and captured `streamError` in [VibePane.tsx](file:///home/oloty/Dev/Pico/src/components/vibe/VibePane.tsx) to surface formatted LLM error cards to the user immediately.
   - Replaced misleading completion fallback with an honest message: `"I was unable to propose any changes for this request. Please try rephrasing or inspect the file directly."`
   - Normalized theme paths in `vibeAgent.ts` and `src-tauri/src/vibe.rs` to tolerate leading slashes and relative variants (`/assets/theme.css`, `./assets/theme.css`).
   - Added system instruction guiding Vibe to inspect `assets/theme.css` via `readThemeCss` whenever users ask to modify app or site colors/styles.
2. **Suppression of `get_preview_revision` Log Flooding:**
   - Identified that `PreviewPane.tsx` polls `getPreviewRevision` every 600ms, resulting in two log lines (`[invoke -> get_preview_revision]` and `[invoke <- get_preview_revision OK]`) being written every second to `~/.pico.log`.
   - Updated `apiInvoke` in [api.ts](file:///home/oloty/Dev/Pico/src/api.ts) to suppress debug and trace logging for `get_preview_revision`.
3. **Verbose Unified File Logging to `~/.pico.log`:**
   - Implemented thread-safe `PicoFileLogger` in [logger.rs](file:///home/oloty/Dev/Pico/src-tauri/src/logger.rs) implementing the standard `log::Log` facade, writing millisecond-precision timestamped entries directly to `~/.pico.log` with immediate flushing on every write.
   - Installed panic hook in Rust to capture and log critical panics with source file, line number, and payload before termination.
   - Added `log_frontend` Tauri command to channel frontend logs into the same unified log file.
   - Implemented [logger.ts](file:///home/oloty/Dev/Pico/src/lib/logger.ts) in frontend: intercepts `console.log`, `info`, `warn`, `error`, `debug`, hooks `window.onerror` and `window.onunhandledrejection`, and exports `picoLog`.
   - Wrapped all Tauri API invocations in [api.ts](file:///home/oloty/Dev/Pico/src/api.ts) with `apiInvoke` to log command entry, argument keys, success, and detailed error messages.
   - Added verbose logging across [lib.rs](file:///home/oloty/Dev/Pico/src-tauri/src/lib.rs) commands, [builder.rs](file:///home/oloty/Dev/Pico/src-tauri/src/builder.rs) 10-step build pipeline, [preview.rs](file:///home/oloty/Dev/Pico/src-tauri/src/preview.rs) watcher/server events, and [publish.rs](file:///home/oloty/Dev/Pico/src-tauri/src/publish.rs) git operations.
2. **Vibe Agent Empty Response & Tool Loop Fix:**
   - Identified that AI SDK v7's `toolApproval: { proposeEdit: 'user-approval' }` configuration on `ToolLoopAgent` paused the tool loop before `proposeEdit.execute()` could run, waiting indefinitely for client approval tokens before proceeding. Because the loop was frozen waiting for tool results, the agent never called `execute()` (leaving the proposal diff card ungenerated) and never streamed an explanation, resulting in a blank response.
   - Refactored `proposeEdit` in [vibeAgent.ts](file:///home/oloty/Dev/Pico/src/lib/vibeAgent.ts): the tool execution itself runs `evaluateVibeProposal`, generates the diff card via `onProposalCreated`, and returns confirmation to the model. The model is then prompted to explain the changes in its streaming response. Disk saves remain strictly gated behind user action ("Accept & Save" / "Reject").
   - Added fallback summary text in [VibePane.tsx](file:///home/oloty/Dev/Pico/src/components/vibe/VibePane.tsx) so that even if a model halts after emitting a tool call without generating text, the user receives a helpful message instead of a blank card.
2. **Added Git Commit Protocol on Memory Bank Updates to GEMINI.md:**
   - Updated [GEMINI.md](file:///home/oloty/Dev/Pico/GEMINI.md) operational rules and guardrails to mandate committing to Git every time `/memory-bank/` is updated.
2. **CSP `connect-src` Fix for Gemini Native & Third-Party LLMs:**
   - Identified root cause of WebKit generic `TypeError: Load failed` when calling Gemini Native from frontend: [tauri.conf.json](file:///home/oloty/Dev/Pico/src-tauri/tauri.conf.json) CSP `connect-src` only permitted `https://api.openai.com` and `https://api.github.com`. WebKit/WebKit2GTK silently blocked requests to `https://generativelanguage.googleapis.com` as CSP violations.
   - Updated `connect-src` in `tauri.conf.json` to include all supported LLM endpoints: `https://generativelanguage.googleapis.com`, `https://api.meta.ai`, `https://openrouter.ai`, alongside `https://api.openai.com` and `https://api.github.com`.
   - Updated [vibeAgent.ts](file:///home/oloty/Dev/Pico/src/lib/vibeAgent.ts) `formatLlmError` to recognize WebKit's generic `Load failed` string and format it as a user-friendly connectivity error.
2. **Preset Model Selector Updates:**
   - Removed deprecated `gemini-3-flash-preview` and `gemini-2.5-flash` presets.
   - Added new presets for Gemini providers (`gemini-native` and `gemini-openai`): `gemini-3.8-flash`, `gemini-3.5-flash`, and `gemini-3.5-flash-lite`.
   - Updated backend provider registry in [llm.rs](file:///home/oloty/Dev/Pico/src-tauri/src/llm.rs) and verified thinking mapping & unit tests.
2. **Settings Relocated to Top Navbar:**
   - Removed Settings sub-tab from [CodePane.tsx](file:///home/oloty/Dev/Pico/src/components/code/CodePane.tsx) (sub-navigation is now: Content, Models, Templates, Theme).
   - Added a dedicated gear icon button (`<Settings className="w-4 h-4" />`) to the top navbar in [App.tsx](file:///home/oloty/Dev/Pico/src/App.tsx) immediately to the right of the "Switch Site" button with no text label.
   - Wired the gear button to toggle the Settings modal dialog.
   - Updated Vibe pane settings trigger callback to directly open the modal dialog.
2. **Dropdown Menu White-on-White Contrast Fix:**
   - Identified that HTML `<select>` dropdown popups and `<option>` elements on Linux/WebKit defaulted to white backgrounds with white text when inheriting app styles.
   - Enforced `color-scheme: dark;` on `html`, `body`, `#app`, and `select` in `src/styles.css`.
   - Explicitly styled `select option` with dark background (`#0a0a0a`) and light text (`#f5f5f5`).
   - Added `[&>option]:bg-neutral-950 [&>option]:text-neutral-100` utility classes to dropdowns across [SettingsModal.tsx](file:///home/oloty/Dev/Pico/src/components/settings/SettingsModal.tsx) and [ModelsEditor.tsx](file:///home/oloty/Dev/Pico/src/components/code/ModelsEditor.tsx).
2. **Pushed Tag `v0.1.0` to Origin:**
   - Created annotated/lightweight git tag `v0.1.0` on `main` (`5c464c3`).
   - Pushed tag `v0.1.0` to `github.com:hoytnix/Pico.git`.
   - Triggers `tauri-apps/tauri-action@v0` to build and publish release bundles under `v0.1.0`.
1. **Tag-Gated Publishing in CI Release Workflow:**
   - Modified `.github/workflows/release.yml` so that `tags: ["*"]` triggers for any release tag.
   - Replaced dynamic tag step with direct step gate `if: startsWith(github.ref, 'refs/tags/')` on `tauri-apps/tauri-action@v0`.
   - On branch pushes (`main`) and PRs, `cargo test`, `pnpm run build:frontend`, and standalone binary executable builds run as CI, while the publishing step is skipped.
   - On new tags, `tauri-action` runs with `tagName: ${{ github.ref_name }}` and `releaseDraft: true`, publishing all OS installer packages to GitHub Releases.
   - Pushed commit `bcbcc28` to `origin/main`.
2. **CI Release Workflow Permissions Fix:**
   - Added `permissions: contents: write` to resolve `403 Resource not accessible by integration`.
3. **CI Release Workflow pnpm Fix:**
   - Identified clash where `.github/workflows/release.yml` specified `version: 12` while `package.json` pinned `packageManager: pnpm@12.15.0`.
   - Removed `version: 12` from `.github/workflows/release.yml` leaving `package.json` as single source of truth.
   - Pushed commit `8450eab29db21755b72ec31af7878aff05331271`.
3. **TailwindCSS v4 Integration (Step 0):**
   - Installed `tailwindcss` and `@tailwindcss/vite` via `pnpm` exclusively.
   - Configured Vite plugin in `vite.config.ts` alongside `@vitejs/plugin-react`.
   - Added `@import "tailwindcss";` to `src/styles.css`.
   - Purged legacy 1,158-line vanilla `styles.css` layout rules that caused vertical stacking.
   - Added scoped ProseMirror typography base rules to `src/styles/editor.css` to prevent preflight conflicts.
2. **Desktop Shell & Pane Visibility:**
   - Root shell structured as `flex h-screen flex-col overflow-hidden`.
   - Header is `shrink-0` with app title and "Switch Site" button only — zero tab buttons.
   - Main area is `flex-1 min-h-0 relative overflow-hidden bg-neutral-950`.
   - Fixed exactly one pane visible at a time using Tailwind `block` / `hidden`. All panes remain mounted to preserve drafts, chat, diffs, unsaved editor content, and scroll state across switches.
3. **Dedicated Full-Width Bottom Bar:**
   - Bottom bar is a direct `shrink-0 w-full h-[52px]` flex child at the bottom of the root shell, always visible.
   - Houses strictly 3 primary tabs in order: **Vibe**, **Preview**, **Code**.
   - Removed all `fixed bottom-0` hacks and bottom padding offsets (`pb-[52px]`).
4. **Code Pane Sub-Navigation:**
   - Content, Models, Templates, Theme, and Settings are strictly sub-navigation inside the Code pane.
   - Zero top-level topbar buttons.
   - Settings is integrated directly into Code sub-navigation with inline editing support and external modal trigger capability.
5. **No-Site State:**
   - When no site is open (`currentSite === null`), main area displays only Open / Create Site + Recent Sites.
   - Content, Models, Templates, Theme, and Vibe composers are hidden. Vibe tab in bottom dock is disabled with `opacity-40 cursor-not-allowed`.
6. **Build Verification:**
   - `pnpm build`: TypeScript and Vite bundle compiles cleanly with Tailwind v4 utilities.
   - `cargo test`: all 19 tests pass cleanly.
   - `cargo check`: backend compiles clean.
   - `pnpm run build:bin`: compiled standalone release binary `target/release/pico`.

## Active Decisions & Considerations
- Tailwind CSS v4 is used exclusively for the Pico desktop studio shell, never injected into `out/` or previewed user sites.
- Pane toggling relies on Tailwind `block` and `hidden` to guarantee zero unmounting and zero state loss.
