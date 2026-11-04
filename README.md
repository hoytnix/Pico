# Pico — Desktop Static Site Generator

**Pico** is a local-first, visual desktop static site generator designed for writers, designers, and makers who do not want to use a terminal, git, or command-line build tools. Open or create a site folder, edit Markdown content in a Milkdown Crepe WYSIWYG editor, customize MiniJinja templates with model-aware guardrails and one-click revert, preview with live reload, and build or export a complete static site — all from a desktop GUI.

---

## Deviations from prompt

None. Every locked decision in §2 is implemented as specified:
- **App name:** Pico
- **Stack:** Rust + Tauri v2 + MiniJinja + Milkdown (Crepe) + Markdown documents with YAML frontmatter on disk.
- **Frontend:** Vanilla TypeScript + Vite (no React/Vue/Svelte framework). Milkdown Crepe is mounted directly as a vanilla instance in a `useEffect`-style lifecycle hook (`useCrepeEditorEffect` in `src/crepeHook.ts`).
- **Package manager:** `pnpm` exclusively.
- **Templating engine:** MiniJinja (`minijinja` v2) with runtime loading, HTML autoescaping, fuel limits, and model-based variable guardrails.
- **Editor:** Milkdown with Crepe built-in (`Frame` theme, light + dark following system), floating toolbar, slash `/` menu, block drag handles, tables, task lists, CodeMirror code blocks with language picker & syntax highlighting, and non-base64 local file image uploads.

---

## Stack

| Layer | Technology |
| --- | --- |
| Desktop Shell & IPC | **Rust + Tauri v2** (`tauri`, `tauri-plugin-dialog`, `tauri-plugin-fs`, `tauri-plugin-shell`, `tauri-plugin-http`) |
| Templating Engine | **MiniJinja v2** (`loader`, `fuel`, `builtins`, `AutoEscape::Html`) |
| Markdown Rendering | **pulldown-cmark** (GFM tables, strikethrough, task lists, fenced code language classes) |
| Image Processing | **image** crate (`FilterType::Lanczos3` WebP hero max 1200px & card thumb max 600px) |
| Preview Server & Watcher | **Axum** + `tower-http::services::ServeDir` + **notify** (300ms debounced live reload) |
| WYSIWYG Editor | **Milkdown Crepe** (`@milkdown/crepe`, `@milkdown/kit`, `Frame` light/dark theme) |
| Frontend Bundler | **Vanilla TypeScript + Vite**, managed exclusively with **pnpm** |

---

## Run & Verification Instructions

```bash
# 1. Install frontend dependencies (pnpm exclusively)
pnpm install

# 2. Run the Rust test suite (frontmatter validation, slugify, pagination, tag grouping,
#    WebP image variants, sitemap/RSS, MiniJinja macro & unknown-variable guardrails, E2E build)
cargo test

# 3. Build the TypeScript + Vite frontend bundle
pnpm build

# 4. Type-check / compile the Tauri Rust backend
cargo check

# 5. Launch Pico in desktop development mode
pnpm tauri dev

# 6. Compile to standalone release executable (for the current machine):
pnpm run build:bin
# -> Outputs optimized standalone executable to: target/release/pico (or pico.exe on Windows)

# 7. Compile and build full OS installer/package bundles (deb/AppImage, dmg/app, msi/exe):
pnpm run build:app
```

---

## Multi-Platform Executable Distribution (Mac, Windows, Linux)

Pico includes an automated GitHub Actions release workflow in [`.github/workflows/release.yml`](file:///.github/workflows/release.yml):
- **Linux (`ubuntu-22.04`)**: Compiles `x86_64-unknown-linux-gnu` standalone binary + `.deb` & `.AppImage`.
- **macOS (`macos-latest`)**: Compiles Apple Silicon / Intel universal `.dmg`, `.app` bundle, and standalone binary.
- **Windows (`windows-latest`)**: Compiles `x86_64-pc-windows-msvc` `.msi`, NSIS setup `.exe`, and standalone `pico.exe`.
- **Zero-Dependency Starters**: All 4 starter templates (Blog, Portfolio, Landing, Docs) are embedded directly into the compiled executable via `include_dir`, so the distributed binary runs completely self-contained even when executed outside the repository folder.

Pushing a version tag (e.g. `git tag v0.1.0 && git push origin v0.1.0`) triggers the matrix build and automatically uploads executable release assets to GitHub Releases.

---

## User Site Layout

Every Pico site on disk follows this exact directory structure:

```text
my-site/
  site.yaml
  models/
    blog.yaml
    project.yaml
  templates/
    skeleton.jinja
    blog.jinja
    blog-list.jinja
    project.jinja
    project-list.jinja
    tag.jinja
    home.jinja
    404.jinja
    macros/
      cards.jinja
  content/
    blog/*.md
    projects/*.md
  assets/
    theme.css
    highlight.css
    highlight.js
  out/
```

- **Markdown is the source of truth on disk:** Posts live in `content/<collection>/*.md` with YAML frontmatter at the top and Markdown body below.
- **Frontmatter GUI Form:** Generated dynamically from `models/<collection>.yaml`. Frontmatter is never placed inside the Milkdown editor body.
- **Attachments & Images:** Dropping, pasting, or picking an image copies the file into `content/<collection>/<slug>/` (or `assets/` for site-wide assets) and inserts a relative Markdown link — never base64. Images larger than 2 MB display a friendly warning and are resized into WebP variants at build time.

---

## Starter Templates Included (`starters/`)

1. **`starters/blog`** — Latest posts home page, paginated blog index (10 posts/page), single post template, tag index pages, RSS 2.0 + Atom feeds, and client-side search.
2. **`starters/portfolio`** — Featured projects showcase, project detail template with automatic attachment image gallery, and notes/about pages.
3. **`starters/landing`** — Single-page product hero, feature cards, and conditional contact form (rendered only when `site.form_endpoint` is set in `site.yaml`).
4. **`starters/docs`** — Documentation layout with sidebar navigation ordered from content, documentation detail pages, and instant `search.json` search bar in `skeleton.jinja`.

---

## How Preview, Build, Export, and Vibe Work

### Preview
- Clicking **Preview** runs the build pipeline with `include_drafts = true` (rendering draft posts with a visible `DRAFT` badge), starts an in-process Axum + `tower-http::ServeDir` static server on a dynamic `127.0.0.1:<port>`, and opens the live site inside Pico (with a copyable localhost URL and "Open in Browser" button).
- A background `notify` watcher monitors `content/`, `templates/`, `assets/`, and `site.yaml` (debounced to 300ms), automatically rebuilding and triggering live reload.

### Build (10-Step Pipeline)
1. Walks `content/**/*.md`, parses YAML frontmatter + Markdown body, and validates fields against `models/<collection>.yaml` (reporting missing required fields, unknown fields, or type mismatches with file + line + plain-language reason).
2. Filters out `draft: true` posts (unless `include_drafts` is enabled), auto-slugifies empty slugs from titles, and checks slug uniqueness per collection.
3. Renders Markdown bodies to HTML with `pulldown-cmark` (tables, strikethrough, task lists, fenced code `language-<lang>` classes for `highlight.js`).
4. Sorts collections by `date` descending then `title` ascending, chunks into pages of 10, and groups/paginates tag indexes.
5. Renders with MiniJinja (`AutoEscape::Html`, fuel limit `50_000`) in exact order: single pages (`/blog/<slug>/index.html`), collection list pages (`/blog/`, `/blog/page/2/`), tag pages (`/tags/<tag>/`), home page (`home.jinja`), and `404.html`.
6. Copies `assets/` → `out/assets/` unchanged and copies page attachments → `out/<collection>/<slug>/`.
7. Generates WebP image variants using the `image` crate (`FilterType::Lanczos3`): hero max 1200px wide (`<stem>-hero.webp`) and card thumbnail max 600px wide (`<stem>-thumb.webp`) written only to `out/`, never back into source. SVG, animated GIF, and AVIF files are copied unchanged.
8. Generates `sitemap.xml`, `robots.txt`, RSS 2.0 (`feed.xml`), Atom (`atom.xml`), and `search.json`.
9. Bundles offline `highlight.js` + `highlight.css` in `skeleton.jinja` so fenced code blocks are syntax-highlighted on the published site even completely offline.
10. Builds into `out.tmp/` first and atomically swaps into `out/` only if there are zero build errors, preserving any previous good build on error.

### Export & GitHub Pages Publish
- **Export:** Copies `out/` to any destination folder picked by the user. Guaranteed to work 100% offline without git or a GitHub account.
- **Publish to GitHub Pages:** Best-effort helper that checks for `git` and a GitHub `origin` remote, pushes `out/` to the `gh-pages` branch, and optionally configures Pages via the GitHub API if a token is set in Settings. If any prerequisite is missing, it displays friendly instructions without failing the build.

### Vibe Editing
- Available on the **Content Editor**, **Template Editor**, and **Theme Panel**.
- Uses an OpenAI-compatible chat completions endpoint and API key configured in **Settings**. When no API key is configured, the Vibe button is disabled with the exact message: `"Configure an LLM API key in Settings to use Vibe."`
- Never auto-applies changes: displays a unified diff (`old` vs. `proposed`) with **Accept & Save** and **Reject & Discard** buttons. Template and Theme Vibe outputs pass through the same MiniJinja and model variable guardrails as manual edits.

---

## Deferred (Non-Goals in MVP, §9)

The following items are intentionally deferred outside MVP scope per §9:
- Plugins / extensions system
- i18n / multi-language routing
- Pagefind binary indexing (MVP uses `search.json` client-side search)
- AVIF image conversion and responsive `srcset` generation (MVP generates 1200px hero & 600px thumb WebP variants)
- API-only GitHub Pages artifact deployment
- Real-time multi-user collaboration, custom comments/form backend servers, user accounts, or analytics/telemetry

---

## License

**(k) Kopimi** — Copying is encouraged. You are free to copy, modify, remix, and distribute this work for any purpose. All rights unreserved. See [LICENSE](file:///home/oloty/Dev/Pico/LICENSE).

