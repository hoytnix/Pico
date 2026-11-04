# Active Context: Pico

## Current Focus
Rebuilt Pico's frontend in React + CaffeineMD TipTap editor + AI SDK v7 Vibe multi-turn conversation, meeting all M84 one-shot prompt criteria without regressions.

## Recent State Changes
1. **Frontend Stack Migration:**
   - Completely replaced vanilla TypeScript with **React 19 + TypeScript 5.9 + Vite 6.4**, packaged exclusively with `pnpm`.
   - Purged `@milkdown/crepe`, `@milkdown/kit`, `@google/genai`, and vanilla mount code from `package.json`, styles, and `src/`.
2. **CaffeineMD TipTap Editor Integration:**
   - Ported TipTap v3 editor implementation directly from `../CaffeineMD` (`@tiptap/react`, `@tiptap/starter-kit`, `@tiptap/pm`, `lowlight`, `highlight.js`).
   - Integrated CaffeineMD markdown parser and serializer (`markdownToEditorHtml`, `editorHtmlToMarkdown`, `markdownTable`, `markdownInline`, `codeLanguageDetector`).
   - Separated frontmatter completely into the Post Properties form, editing only the Markdown body in TipTap.
   - Connected 300ms debounced auto-save to `picoApi.savePost` and external update handling via `editor.commands.setContent(..., { emitUpdate: false })` on Vibe Accept.
   - Preserved CaffeineMD desktop dark theme and toolbar controls (headings popover, bold/italic/underline/strike, code blocks, lists, task checkboxes, GFM tables, image insertion, undo/redo).
3. **AI SDK v7 Vibe Multi-Turn Conversation:**
   - Replaced one-shot prompt-to-diff with full conversational assistant powered by Vercel AI SDK v7 (`ai` Core, `@ai-sdk/react`, `@ai-sdk/google`, `@ai-sdk/openai`).
   - Frontend-direct architecture in webview with call-time key retrieval via `get_llm_key`. No Node server at runtime.
   - Integrated `ToolLoopAgent` with 4 strictly allowed tools (`readContentFile`, `readTemplateFile`, `readThemeCss`, `proposeEdit`).
   - Enforced tool approval gate (`toolApproval: { proposeEdit: 'user-approval' }`): assistant proposals render as unified diff cards with Accept/Reject buttons.
   - Accept saves via normal save path (`savePost`, `saveTemplate`, `saveTheme`) and updates the active editor.
   - Multi-turn conversation history, streaming responses, stop/regenerate, and no-key rule (`"Configure an LLM API key in Settings to use Vibe."`).
4. **Desktop Layout & Navigation:**
   - Desktop-only app. Bottom bar is the ONLY tab location containing exactly **Vibe**, **Preview**, and **Code** tabs. Topnav has zero tab buttons.
   - Zero-remount persistent shell preserves chat history, composer drafts, pending diffs, editor content, and preview scroll across tab switches.
   - Single authoritative Live Preview Server bar transitions empty → starting → live URL without duplicates.
5. **Provider, Thinking & Grounding Controls:**
   - Provider presets: `gemini-native`, `openai`, `openrouter`, `meta`, `gemini-openai`, `custom`.
   - Thinking mapping: Gemini 3/4 (levels LOW/MEDIUM/HIGH, never MINIMAL/budget), Gemini 2.5 (budgets 1024/8192/24576), OpenAI (reasoning_effort), Muse Spark (disabled with note).
   - Google Search Grounding: enabled ONLY on Gemini Native with sources panel under diff; disabled on others with exact text.
   - Named error formatting for 401, 402, 404, 400, 429, timeout.
6. **Build Verification:**
   - `cargo test`: all 19 tests pass cleanly (28.64s).
   - `pnpm build`: TypeScript and Vite bundle compiles with 0 errors.
   - `cargo check`: backend compiles clean (0.19s).
   - `pnpm run build:bin`: compiled standalone release binary `target/release/pico` (30.60s).

## Active Decisions & Considerations
- Pico is a desktop application; navigation lives exclusively in the bottom bar with persistent pane mounting.
- Frontmatter remains strictly in the visual Post Properties form; TipTap edits Markdown body only.
- Vibe never auto-applies edits: every proposal goes through the unified diff card and explicit Accept/Reject gate.
