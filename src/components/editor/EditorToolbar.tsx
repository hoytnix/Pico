import React from 'react';
import type { Editor } from '@tiptap/react';
import {
  Bold,
  Italic,
  Underline as UnderlineIcon,
  Strikethrough,
  Code,
  List,
  ListOrdered,
  CheckSquare,
  Quote,
  Minus,
  Image as ImageIcon,
  Undo,
  Redo,
} from 'lucide-react';
import { HeadingDropdown } from './toolbar/HeadingDropdown';
import { TableControls } from './toolbar/TableControls';

interface EditorToolbarProps {
  editor: Editor | null;
  onPickImage?: () => void;
}

export function EditorToolbar({ editor, onPickImage }: EditorToolbarProps) {
  if (!editor) return null;

  return (
    <div className="editor-toolbar flex items-center flex-wrap gap-1 p-2 bg-neutral-900 border-b border-neutral-800 text-neutral-300 select-none">
      {/* Heading Selector */}
      <HeadingDropdown editor={editor} />

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

      {/* Table Controls */}
      <TableControls editor={editor} />

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
