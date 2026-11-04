import React, { useEffect, useRef, useMemo } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import { getEditorExtensions } from './config/editorExtensions';
import { EditorToolbar } from './EditorToolbar';
import { markdownToEditorHtml, editorHtmlToMarkdown } from '../../lib/markdown/serializer';

interface TiptapEditorProps {
  markdown: string;
  postPath: string;
  onUpdateMarkdown: (newMarkdown: string) => void;
  onPickImage?: () => void;
  setEditorInstance?: (editor: any) => void;
}

export function TiptapEditor({
  markdown,
  postPath,
  onUpdateMarkdown,
  onPickImage,
  setEditorInstance,
}: TiptapEditorProps) {
  const loadedPathRef = useRef<string | null>(null);
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const onUpdateRef = useRef(onUpdateMarkdown);
  onUpdateRef.current = onUpdateMarkdown;

  const extensions = useMemo(() => getEditorExtensions(), []);

  const initialContent = useMemo(() => {
    return markdownToEditorHtml(markdown || '');
  }, []); // Only initial load

  const editor = useEditor({
    immediatelyRender: false,
    content: initialContent,
    extensions,
    editorProps: {
      attributes: {
        class:
          'tiptap-prosemirror ProseMirror focus:outline-none min-h-[400px] p-6 text-[15px] leading-[1.8] text-neutral-200',
      },
    },
    onUpdate: ({ editor: currentEditor }) => {
      if (!currentEditor || currentEditor.isDestroyed) return;
      if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
      saveTimeoutRef.current = setTimeout(() => {
        if (!currentEditor || currentEditor.isDestroyed) return;
        try {
          const html = currentEditor.getHTML();
          const md = editorHtmlToMarkdown(html);
          onUpdateRef.current(md);
        } catch (err) {
          console.warn('Auto-save conversion skipped:', err);
        }
      }, 300);
    },
  });

  // Provide editor instance to parent if needed (for external setContent, e.g. Vibe Accept)
  useEffect(() => {
    if (setEditorInstance) {
      setEditorInstance(editor);
    }
    return () => {
      if (setEditorInstance) setEditorInstance(null);
    };
  }, [editor, setEditorInstance]);

  // Synchronize external markdown updates (e.g. switching post or Vibe Accept)
  // CaffeineMD pattern: guarded against re-injecting when focused to prevent Enter-key cursor jumping
  useEffect(() => {
    if (!editor || editor.isDestroyed) return;

    const isFileSwitch = loadedPathRef.current !== postPath;

    if (isFileSwitch) {
      const targetHtml = markdownToEditorHtml(markdown || '');
      editor.commands.setContent(targetHtml, { emitUpdate: false });
      loadedPathRef.current = postPath;
      return;
    }

    // If currently focused and typing, do not overwrite editor state from reactive local saves
    if (editor.isFocused) {
      return;
    }

    // External change (e.g. Vibe proposal accepted or external revert)
    const targetHtml = markdownToEditorHtml(markdown || '');
    const currentHtml = editor.getHTML();
    if (currentHtml === '<p></p>' || Math.abs(currentHtml.length - targetHtml.length) > 10) {
      editor.commands.setContent(targetHtml, { emitUpdate: false });
    }
  }, [editor, markdown, postPath]);

  return (
    <div className="tiptap-editor-shell flex flex-col h-full bg-neutral-950 border border-neutral-800 rounded-lg overflow-hidden shadow-inner">
      <EditorToolbar editor={editor} onPickImage={onPickImage} />
      <div className="tiptap-scroll-container flex-1 overflow-y-auto">
        <EditorContent editor={editor} />
      </div>
    </div>
  );
}
