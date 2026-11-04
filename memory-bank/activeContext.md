# Active Context: Pico

## Current Focus
Duplicate Live Preview Server bug fix and mobile tab naming correction (Vibe / Preview / Code) are fully implemented, verified across narrow and wide viewports, and synchronized with the Memory Bank.

## Recent State Changes
1. **Duplicate Live Preview Server Bug Fix (M84 Prompt):**
   - **Root Cause Resolution**:
     - Removed duplicate preview mounting: `#pane-preview` (from persistent 3-pane shell) is now the single, authoritative mount point for live preview across all viewport widths. Duplicate `renderPreviewPane()` in `renderActiveTabPane()` was completely eliminated.
     - Resolved CSS visibility conflict: `.studio-pane` on desktop (`≥ 768px`) now uses `.pane-active` visibility toggling rather than defaulting to `display: flex`, preventing `#pane-preview` and `#pane-code` from rendering simultaneously stacked in the DOM.
     - Fixed transient state handling: The single Live Preview Server control bar renders "Starting preview server…" in the URL input while the server is initializing, and transitions smoothly in-place to the live URL (`http://127.0.0.1:<port>/`) once ready. It never leaves a stale empty bar or renders a second bar.
   - **Tab Naming Correction (§2)**:
     - Mobile / narrow tab bar is strictly standardized to:
       1. **Vibe** (formerly Chat): Conversation area, prompt box, proposed edit, diff with Accept/Reject.
       2. **Preview**: Live site preview with exactly ONE Live Preview Server control bar.
       3. **Code** (formerly Files): Editing / file side (content posts, templates, models, assets, and site.yaml).
     - Wide viewport top nav (Content, Models, Templates, Theme, Preview, Build & Export, Settings) remains intact and unmodified.
   - **State & Server Preservation**:
     - Switching between tabs on narrow viewports (375px) or desktop preserves Vibe conversation draft, pending Vibe diff, Code editor unsaved changes, Milkdown Crepe WYSIWYG editor state, and preview scroll position.
     - Exactly one preview server/port is active per site. Switching sites properly stops polling and resets preview state.
     - Grepping built site output in `out/` confirms 0 Pico chrome leakage.
2. **Mobile AI Studio Layout Redesign (M84 Prompt):**
   - Fixed bottom tab bar within thumb reach (64–72px height, 48px touch targets, `safe-area-inset-bottom` padding, pill highlight for active tab).
   - Zero-remount 3-pane architecture (`#pane-vibe`, `#pane-preview`, `#pane-code`).
   - App shell with `100dvh` and independent internal scrolling per pane.
3. **Build & Test Verification:**
   - `cargo test`: All 19 unit and integration tests pass cleanly (28.30s).
   - `pnpm build`: `tsc --noEmit && vite build` completes with 0 errors.
   - `pnpm run build:bin`: Compiles standalone release binary (`target/release/pico`, 31.5MB).
   - Bundled `.deb` and `.rpm` packages built in `target/release/bundle/`.
   - `README.md` and `/memory-bank/` updated with root cause, resolution details, and tab naming corrections.

## Active Decisions & Considerations
- Scope strictly confined to bug fix and tab naming correction (§2): build pipeline, MiniJinja rendering, Milkdown Crepe editing, model validation, and provider logic remain intact.
- Zero CSS-only workarounds: duplicate render was removed at its source, not hidden with `display: none` while keeping multiple components mounted.
- The app remains completely offline-first; network calls are strictly restricted to the user's selected LLM provider and optional GitHub Pages deployment.
