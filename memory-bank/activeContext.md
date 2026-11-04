# Active Context: Pico

## Current Focus
The visual **Model Editor** tab and collection management suite are complete, fully verified, and synchronized with the Memory Bank.

## Recent State Changes
1. **Model Editor & Management (§3.2, §4.2):**
   - Added a dedicated **Models** tab in the top navbar (`Content | Models | Templates | Theme | Preview | Build & Export | Settings`).
   - Implemented a visual collection model editor (`models/*.yaml`) allowing users to create, inspect, edit, and delete site models without touching raw YAML.
   - Built an interactive **Frontmatter Fields Schema** table supporting custom field names, display labels, field type dropdowns (`string`, `date`, `tags`, `image`, `bool`), required flags, up/down field reordering, and field deletion.
   - Guarded core fields (`title` and `date`) to prevent accidental deletion and guarantee sorting, RSS/Atom feed generation, and sitemap compatibility.
   - Added Tauri commands `list_models`, `save_model`, and `delete_model` with OS trash fallback (`trash` crate).
   - Real-time synchronization: Saving or deleting models updates `site.collections` immediately across the studio, updating collection chips and frontmatter forms in the Content editor.
2. **Release Executable & Distribution Pipeline:**
   - Standalone release binary built to `target/release/pico` with embedded starter templates via `include_dir`.
   - Multi-platform GitHub Actions release workflow configured for Linux, macOS, and Windows.
3. **License:** Set to `(k) Kopimi` across repository root, `package.json`, `Cargo.toml`, and `README.md`.
4. **Test Suite:** Expanded to 12 unit and integration tests (including `test_save_and_delete_model_impl`); all 12 tests pass cleanly.

## Active Decisions & Considerations
- The app is completely offline-first, local-first, and self-contained.
- Non-goals outlined in §9 (Pagefind, AVIF, plugins, real-time collaboration, user accounts) remain deferred per specification.
