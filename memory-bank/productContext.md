# Product Context: Pico

## Problem Domain
Traditional static site generators (Hugo, Jekyll, Astro, Zola) offer speed, security, and low hosting costs, but impose heavy technical friction on non-technical users:
1. Command-line installations, terminal build commands, and terminal error stack traces.
2. Direct editing of raw YAML frontmatter, where indentation mistakes break builds.
3. Complex Git workflows required simply to publish or preview changes.
4. Broken templates causing silent failures or hard panics without clear line references.

## Solution & UX Objectives
Pico bridges the gap between static site generator performance and CMS visual comfort:
- **Visual Frontmatter Form:** Frontmatter is presented as a clean GUI form generated from collection models (`models/*.yaml`). Raw YAML frontmatter is never visible inside the editor body.
- **Milkdown Crepe WYSIWYG:** Writers get formatting toolbars, slash commands (`/`), task lists, tables, and CodeMirror code blocks without leaving Markdown semantics.
- **Local Asset Management:** Images pasted or picked from disk are copied to the post's directory (`content/<collection>/<slug>/`) or `assets/` and linked relatively — never base64 encoded into Markdown. Warnings are shown for files >2MB while preserving originals.
- **Fail-Safe Templates:** MiniJinja template editing validates variables against models before saving. Unknown variables or macro errors reject saves with friendly messages naming file, line, and variable. Previous versions are backed up for one-click Revert.
- **Instant Preview & Atomic Builds:** Embedded Axum localhost server provides preview with 300ms debounced live reload. Builds compile to `out.tmp/` first and atomically swap to `out/` only when error-free.

## User Flows
1. **Site Management:** Open folder via dialog or create new site from starter (Blog, Portfolio, Landing, Docs).
2. **Content Authoring:** Browse collections sorted by date descending, toggle drafts, write in Crepe, auto-save after 2 seconds idle.
3. **Template Customization:** Inspect available model variables in a side panel, click to insert, save with syntax and guardrail validation, revert bad edits in one click.
4. **Theme Customization:** Edit CSS variables in `assets/theme.css` without risking `skeleton.jinja` HTML structure.
5. **Vibe AI Editing:** Prompt an LLM to refine content, templates, or theme CSS. Review unified diffs and explicitly Accept or Reject.
6. **Publishing:** Export static files to any folder or push directly to GitHub Pages.
