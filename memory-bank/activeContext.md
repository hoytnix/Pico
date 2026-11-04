# Active Context: Pico

## Current Focus
Mobile AI Studio Layout redesign is fully implemented, verified across viewports and build targets, and synchronized with the Memory Bank.

## Recent State Changes
1. **Mobile AI Studio Layout Redesign (M84 Prompt):**
   - **Narrow Viewports (`< 768px`)**:
     - Fixed bottom tab bar within thumb reach (64–72px height, 48px touch targets, `safe-area-inset-bottom` padding, pill highlight for active tab).
     - Exactly three tabs: **Chat**, **Preview**, and **Files**.
     - **Chat Tab**: Vibe conversation area with target badge and provider/model indicator in header; unified diff with **Accept & Save** and **Reject & Discard**; sticky composer above the bottom bar; strict no-key rule displaying exact text: `"Configure an LLM API key in Settings to use Vibe."`.
     - **Preview Tab**: Live Axum preview server with 300ms debounced live reload and `DRAFT` badges; **Build Site** and **Export** buttons placed directly in the mobile preview header so they are never hidden behind a tab switch.
     - **Files Tab**: Comprehensive file browser listing Content posts, Templates (`templates/*.jinja`, `macros/*.jinja`), Models (in-progress visual schema editor intact), Theme (`assets/theme.css` with Vibe routing to Chat), Site Configuration (`site.yaml`), and Media/Assets (`assets/`). Settings gear icon in Files header opens LLM and site settings without adding a fourth tab.
   - **Zero-Remount State Preservation**:
     - Persistent 3-pane shell architecture (`#pane-chat`, `#pane-preview`, `#pane-files`) using CSS visibility toggling (`.pane-active`).
     - Milkdown Crepe WYSIWYG editor content and unsaved-changes state, chat draft input, pending Vibe diffs, preview scroll position, and file selection all survive tab switches.
   - **App Shell & Internal Scrolling**:
     - Viewport fill using `100dvh` (with `100vh` fallback) and `overflow: hidden` on shell root.
     - Independent internal scrolling (`overflow-y: auto`) on each pane with bottom padding equal to tab bar height plus safe-area inset; zero document-level scrollbars.
   - **Wide Viewports (`≥ 768px`)**:
     - Desktop top navigation (Content, Models, Templates, Theme, Preview, Build & Export, Settings) and sidebar layouts remain intact and unmodified. Bottom tab bar is hidden.
2. **Dual-Track LLM Architecture (M84 Prompt):**
   - **Track 1 — Gemini Native (Frontend TS via `@google/genai`):** Native Gemini integration, call-time key retrieval, thinking level/budget mapping, and Google Search Grounding with web sources display.
   - **Track 2 — OpenAI-Compatible Providers (Rust Backend):** High-throughput client POSTing to `{base_url}/chat/completions` for OpenAI, OpenRouter, Meta Muse Spark, Gemini OpenAI mode, and Custom HTTPS.
3. **Build & Test Verification:**
   - `cargo test`: All 19 unit and integration tests pass cleanly (28.42s).
   - `pnpm build`: `tsc --noEmit && vite build` completes with 0 errors.
   - `pnpm run build:bin`: Compiles standalone release binary (`target/release/pico`, 31.5MB).
   - `README.md` updated with "Mobile AI Studio Layout" section and three-tab mapping table.

## Active Decisions & Considerations
- Scope strictly confined to layout and navigation: backend, build pipeline, MiniJinja rendering, Milkdown Crepe editing, model validation, and provider logic remain intact.
- Zero silent substitutions: all design and UX constraints satisfied as specified.
- The app remains completely offline-first; network calls are strictly restricted to the user's selected LLM provider and optional GitHub Pages deployment.
