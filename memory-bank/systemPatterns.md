# System Patterns: Pico

## System Architecture

```mermaid
flowchart TD
    subgraph Frontend ["Frontend (Vanilla TS + Vite + Milkdown Crepe)"]
        UI["Studio Shell (Tab Router & Modals)"]
        Form["Model Form (Frontmatter)"]
        Crepe["Milkdown Crepe Vanilla Hook"]
        TplEditor["Code Editor & Variable Inspector"]
        DiffModal["Vibe Unified Diff Review"]
    end

    subgraph TauriBridge ["Tauri v2 IPC (Capabilities Scoped)"]
        IPC["Tauri Commands Handler"]
    end

    subgraph Backend ["Rust Backend (pico_lib)"]
        Pipeline["10-Step Build Pipeline"]
        Guardrails["MiniJinja Validator & Guardrails"]
        PreviewSrv["Axum Server + notify Watcher"]
        VibeEngine["Vibe LLM Client & Diff Engine"]
        Storage["Settings & Embedded Starters (include_dir)"]
    end

    subgraph FileSystem ["Disk Storage (User Site Layout)"]
        SiteYAML["site.yaml"]
        Models["models/*.yaml"]
        Templates["templates/*.jinja"]
        Content["content/**/*.md"]
        Assets["assets/"]
        OutDir["out/ (Atomic Swap)"]
    end

    UI --> IPC
    Form --> IPC
    Crepe --> IPC
    TplEditor --> IPC
    DiffModal --> IPC

    IPC --> Pipeline
    IPC --> Guardrails
    IPC --> PreviewSrv
    IPC --> VibeEngine
    IPC --> Storage

    Pipeline --> FileSystem
    Guardrails --> Templates
    PreviewSrv --> OutDir
```

## State Machine & Lifecycles

### 1. Editor Lifecycle (`useCrepeEditorEffect`)
- Mounted without React/Vue wrappers into a vanilla DOM node.
- Debounced auto-save triggers 2 seconds after the user stops typing.
- On save, frontmatter is validated against the model schema, formatted into YAML headers, concatenated with clean Markdown body, and saved to disk.
- Unsaved changes flag prompts the user before switching sites or closing the application.

### 2. Template Guardrail Pattern
- Dotted variables inside `{{ ... }}` and `{% ... %}` (e.g. `post.author_bio`, `site.unknown`) are checked against allowed model fields and `site.yaml` properties.
- In `macros/*.jinja`, `varargs` and `kwargs` are prohibited; top-level HTML outside `{% macro %}` is rejected.
- On save, the previous version is backed up in memory (`AppState.template_backups`) and on disk (`.pico_history/templates/`).
- One-click Revert restores the exact previous version without requiring Git.

### 3. Atomic Build Pipeline Pattern (§6)
- Compilation always builds into `out.tmp/`.
- If any validation or render errors occur, `out.tmp/` is purged and errors are reported to the GUI with file and line. The previous working `out/` folder is left untouched.
- On success, `out.tmp/` is atomically renamed or swapped into `out/`.

### 4. Zero-Dependency Embedded Starters Pattern
- `static EMBEDDED_STARTERS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../starters")`.
- `create_site_impl` checks disk directory first; if absent (standalone distributed binary), it extracts from embedded memory into the target directory.
