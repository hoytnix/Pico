# Active Context: Pico

## Current Focus
Fixed GitHub Actions CI/CD release workflow (`.github/workflows/release.yml`) permission failure (`403 Resource not accessible by integration`) by granting `permissions: contents: write` and adding dynamic release tag resolution for pull requests, branch pushes, and version tags.

## Recent State Changes
1. **CI Release Workflow Permission & Tag Resolution Fix:**
   - Identified root cause in run `41891752066`: `tauri-apps/tauri-action@v0` failed with `##[error]Resource not accessible by integration - https://docs.github.com/rest/releases/releases#create-a-release`.
   - Workflow lacked `permissions: contents: write`, defaulting `GITHUB_TOKEN` to read-only.
   - Pushes to `main` passed `tagName: ${{ github.ref_name }}` (evaluating to `"main"`), attempting to create an invalid `"main"` tag release.
   - Added `permissions: contents: write` at the top level of `.github/workflows/release.yml`.
   - Added step `Determine release tag` with portable bash script across Linux, macOS, and Windows:
     - On `pull_request`: sets `tag=` (empty string) and `name=Pico (PR Build)`, causing `tauri-action` to build bundles without attempting to draft releases.
     - On tag push (`refs/tags/v*`): sets `tag=${{ github.ref_name }}` and `name=Pico ${{ github.ref_name }}`.
     - On branch push / manual dispatch: sets `tag=v__VERSION__` and `name=Pico v__VERSION__`, allowing `tauri-action` to replace `__VERSION__` with the semantic version from `tauri.conf.json`.
   - Pushed commit `0b4aca5` to `origin/main`.
2. **CI Release Workflow pnpm Fix:**
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
