# Project Brief: Pico

## Executive Summary
**Pico** is a local-first desktop static site generator designed for writers, creators, and designers who are uncomfortable with terminals, git, or command-line build processes. It provides a visual studio where users can open or create site folders, write in a Milkdown Crepe WYSIWYG editor, customize MiniJinja templates with model-driven guardrails, preview with live reload, and build or export complete static sites.

## Core Goals & Foundation
- **Zero Terminal Dependency:** All editing, image asset management, templating, previewing, building, and exporting are driven 100% from the GUI.
- **Markdown Source of Truth:** Content files reside on disk as Markdown with YAML frontmatter (`content/<collection>/*.md`). The editor is a visual view over Markdown on disk, not a proprietary database.
- **Offline-First:** All core operations function entirely offline without phoning home or requiring cloud connections. External networking is strictly opt-in (LLM API for Vibe and GitHub Pages publishing).
- **Embedded Starters:** The application binary embeds all 4 complete starter templates (Blog, Portfolio, Landing, Docs) using `include_dir` for zero-dependency distribution.
- **License:** (k) Kopimi — Copying is encouraged; all rights unreserved.

## Locked Architecture Decisions
- **Stack:** Rust + Tauri v2 + MiniJinja v2 + Milkdown Crepe + Markdown files on disk. No Python runtime, no Node server at runtime, no Electron.
- **Frontend:** Vanilla TypeScript + Vite, packaged exclusively with `pnpm`. No React/Vue/Svelte wrappers for Milkdown Crepe.
- **Templating Engine:** MiniJinja with runtime loading, HTML autoescaping, fuel limit safety, and model-aware guardrails.
- **Editor:** Milkdown with Crepe built-in (Frame theme following system light/dark mode).
