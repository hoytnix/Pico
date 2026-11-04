import React, { useState, useRef, useEffect } from 'react';
import type { Editor } from '@tiptap/react';
import {
  Bold,
  Italic,
  Underline as UnderlineIcon,
  Strikethrough,
  Code,
  Heading1,
  Heading2,
  Heading3,
  List,
  ListOrdered,
  CheckSquare,
  Quote,
  Table as TableIcon,
  Columns,
  Rows,
  Trash2,
  Minus,
  Image as ImageIcon,
  Undo,
  Redo,
  ChevronDown,
} from 'lucide-react';

interface EditorToolbarProps {
  editor: Editor | null;
  onPickImage?: () => void;
}

export function EditorToolbar({ editor, onPickImage }: EditorToolbarProps) {
  const [headingOpen, setHeadingOpen] = useState(false);
  const headingMenuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (headingMenuRef.current && !headingMenuRef.current.contains(e.target as Node)) {
        setHeadingOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  if (!editor) return null;

  const currentHeadingLabel = () => {
    if (editor.isActive('heading', { level: 1 })) return 'H1';
    if (editor.isActive('heading', { level: 2 })) return 'H2';
    if (editor.isActive('heading', { level: 3 })) return 'H3';
    return 'Text';
  };

  const isTableActive = editor.isActive('table');

  return (
    <div className="editor-toolbar flex items-center flex-wrap gap-1 p-2 bg-neutral-900 border-b border-neutral-800 text-neutral-300 select-none">
      {/* Heading Selector Popover */}
      <div className="relative" ref={headingMenuRef}>
        <button
          type="button"
          onClick={() => setHeadingOpen(!headingOpen)}
          className="flex items-center gap-1 px-2.5 py-1 rounded bg-neutral-800 hover:bg-neutral-700 text-xs font-medium cursor-pointer"
          title="Headings"
        >
          <span>{currentHeadingLabel()}</span>
          <ChevronDown className="w-3.5 h-3.5 opacity-70" />
        </button>
        {headingOpen && (
          <div className="absolute top-full left-0 mt-1 w-32 bg-neutral-900 border border-neutral-700 rounded-md shadow-xl py-1 z-50">
            <button
              type="button"
              onClick={() => {
                editor.chain().focus().setParagraph().run();
                setHeadingOpen(false);
              }}
              className="w-full text-left px-3 py-1.5 text-xs hover:bg-neutral-800 flex items-center gap-2"
            >
              Paragraph
            </button>
            <button
              type="button"
              onClick={() => {
                editor.chain().focus().toggleHeading({ level: 1 }).run();
                setHeadingOpen(false);
              }}
              className="w-full text-left px-3 py-1.5 text-xs hover:bg-neutral-800 flex items-center gap-2 font-bold"
            >
              <Heading1 className="w-3.5 h-3.5 text-indigo-400" /> Heading 1
            </button>
            <button
              type="button"
              onClick={() => {
                editor.chain().focus().toggleHeading({ level: 2 }).run();
                setHeadingOpen(false);
              }}
              className="w-full text-left px-3 py-1.5 text-xs hover:bg-neutral-800 flex items-center gap-2 font-semibold"
            >
              <Heading2 className="w-3.5 h-3.5 text-indigo-400" /> Heading 2
            </button>
            <button
              type="button"
              onClick={() => {
                editor.chain().focus().toggleHeading({ level: 3 }).run();
                setHeadingOpen(false);
              }}
              className="w-full text-left px-3 py-1.5 text-xs hover:bg-neutral-800 flex items-center gap-2 font-medium"
            >
              <Heading3 className="w-3.5 h-3.5 text-indigo-400" /> Heading 3
            </button>
          </div>
        )}
      </div>

      <div className="h-4 w-px bg-neutral-700 mx-1" />

      {/* Inline styles */}
      <button
        type="button"
        onClick={() => editor.chain().focus().toggleBold().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('bold') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Bold (Ctrl+B)"
      >
        <Bold className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().toggleItalic().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('italic') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Italic (Ctrl+I)"
      >
        <Italic className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().toggleUnderline().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('underline') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Underline (Ctrl+U)"
      >
        <UnderlineIcon className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().toggleStrike().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('strike') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Strikethrough"
      >
        <Strikethrough className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().toggleCode().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('code') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Inline Code"
      >
        <Code className="w-4 h-4" />
      </button>

      <div className="h-4 w-px bg-neutral-700 mx-1" />

      {/* Lists */}
      <button
        type="button"
        onClick={() => editor.chain().focus().toggleBulletList().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('bulletList') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Bullet List"
      >
        <List className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().toggleOrderedList().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('orderedList') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Numbered List"
      >
        <ListOrdered className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().toggleTaskList().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('taskList') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Task List"
      >
        <CheckSquare className="w-4 h-4" />
      </button>

      <div className="h-4 w-px bg-neutral-700 mx-1" />

      {/* Blocks */}
      <button
        type="button"
        onClick={() => editor.chain().focus().toggleBlockquote().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('blockquote') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Blockquote"
      >
        <Quote className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().toggleCodeBlock().run()}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          editor.isActive('codeBlock') ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title="Code Block"
      >
        <Code className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().setHorizontalRule().run()}
        className="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 transition-colors cursor-pointer"
        title="Horizontal Line"
      >
        <Minus className="w-4 h-4" />
      </button>

      {/* Table */}
      <button
        type="button"
        onClick={() => {
          if (!isTableActive) {
            editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
          } else {
            editor.chain().focus().deleteTable().run();
          }
        }}
        className={`p-1.5 rounded hover:bg-neutral-800 transition-colors cursor-pointer ${
          isTableActive ? 'bg-indigo-600 text-white' : 'text-neutral-400'
        }`}
        title={isTableActive ? 'Delete Table' : 'Insert Table (3x3)'}
      >
        <TableIcon className="w-4 h-4" />
      </button>

      {isTableActive && (
        <div className="flex items-center gap-1 bg-neutral-800 px-1.5 py-0.5 rounded">
          <button
            type="button"
            onClick={() => editor.chain().focus().addColumnAfter().run()}
            className="p-1 rounded hover:bg-neutral-700 text-neutral-300 text-xs flex items-center gap-0.5"
            title="Add Column"
          >
            <Columns className="w-3 h-3 text-indigo-400" /> +
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().deleteColumn().run()}
            className="p-1 rounded hover:bg-neutral-700 text-neutral-300 text-xs flex items-center gap-0.5"
            title="Delete Column"
          >
            <Columns className="w-3 h-3 text-red-400" /> -
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().addRowAfter().run()}
            className="p-1 rounded hover:bg-neutral-700 text-neutral-300 text-xs flex items-center gap-0.5"
            title="Add Row"
          >
            <Rows className="w-3 h-3 text-indigo-400" /> +
          </button>
          <button
            type="button"
            onClick={() => editor.chain().focus().deleteRow().run()}
            className="p-1 rounded hover:bg-neutral-700 text-neutral-300 text-xs flex items-center gap-0.5"
            title="Delete Row"
          >
            <Rows className="w-3 h-3 text-red-400" /> -
          </button>
        </div>
      )}

      {/* Image picker */}
      {onPickImage && (
        <button
          type="button"
          onClick={onPickImage}
          className="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 transition-colors cursor-pointer"
          title="Insert Image from disk"
        >
          <ImageIcon className="w-4 h-4" />
        </button>
      )}

      <div className="flex-1" />

      {/* Undo / Redo */}
      <button
        type="button"
        onClick={() => editor.chain().focus().undo().run()}
        disabled={!editor.can().undo()}
        className="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 disabled:opacity-30 disabled:cursor-not-allowed transition-colors cursor-pointer"
        title="Undo (Ctrl+Z)"
      >
        <Undo className="w-4 h-4" />
      </button>

      <button
        type="button"
        onClick={() => editor.chain().focus().redo().run()}
        disabled={!editor.can().redo()}
        className="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 disabled:opacity-30 disabled:cursor-not-allowed transition-colors cursor-pointer"
        title="Redo (Ctrl+Y)"
      >
        <Redo className="w-4 h-4" />
      </button>
    </div>
  );
}
