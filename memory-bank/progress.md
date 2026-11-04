# Progress & Roadmap: Pico

## Build & Test Status
- **Rust Backend (`cargo test`):** 12 passed; 0 failed.
- **Frontend Bundle (`pnpm build`):** Builds without errors (`tsc --noEmit && vite build`).
- **Release Executable (`pnpm run build:bin`):** Compiles standalone binary to `target/release/pico` (30MB, dynamically linked ELF 64-bit on Linux, with embedded starter templates).
- **Backend Type Check (`cargo check`):** Clean, 0 warnings.

## Feature Matrix & Completed Modules

| Module / Requirement | Status | Notes |
| --- | --- | --- |
| Site Picker / New Site (§3.1) | Completed | Open folder dialog, create new site with Blog/Portfolio/Landing/Docs starters, local recent sites tracking. |
| Content List Sidebar (§3.2) | Completed | Model collection tabs, posts sorted date desc then title, draft badges, delete to OS trash. |
| Content Editor (§3.3) | Completed | Model-generated frontmatter form, Milkdown Crepe WYSIWYG, relative image copy, 2s debounced auto-save. |
| Model Editor (Visual Models) | Completed | Dedicated navbar tab, models sidebar, field schema table (add/reorder/delete fields, type picker), save/delete models with trash. |
| Template Editor (§3.4) | Completed | Variable inspector, MiniJinja model guardrails, kwargs/varargs rejection, one-click revert. |
| Live Preview (§3.5) | Completed | Axum localhost server, notify file watcher (300ms debounce), live reload script, draft badges. |
| Build Pipeline (§3.6, §6) | Completed | 10-step sequence, Lanczos3 WebP variants, sitemap, robots, RSS/Atom, search.json, atomic out.tmp swap. |
| Export & Publish (§3.6) | Completed | Export out/ to user-selected folder, best-effort GitHub Pages branch push with graceful error reporting. |
| Vibe AI Editing (§7) | Completed | Natural language editing for content, templates, and theme CSS; unified diff modal with Accept/Reject. |
| Starter Templates (§8) | Completed | 4 complete starters with sample posts, models, skeleton, macros, and offline syntax highlighter. |
| Embedded Executable Packaging | Completed | `include_dir` starter fallback, `pnpm run build:bin`, and `.github/workflows/release.yml`. |
| License | Completed | (k) Kopimi across repository root, package.json, Cargo.toml, and README.md. |

## Known Issues
- None. All acceptance criteria and verification tests pass.

## Deferred (Non-Goals in MVP, §9)
- Plugins / extensions system
- i18n / multi-language routing
- Pagefind binary integration
- AVIF image conversion and responsive `srcset` generation
- API-only GitHub Pages artifact deployment
- Real-time multi-user collaboration or CMS server backends
