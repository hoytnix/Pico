# Active Context: Pico

## Current Focus
Configured GitHub Actions release workflow (`.github/workflows/release.yml`) so that releases are published exclusively when a new git tag occurs. Branch pushes, pull requests, and manual triggers continue to run compilation, tests, and standalone executable uploads without publishing releases.

## Recent State Changes
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
