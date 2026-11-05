import React from 'react';
import type { Editor } from '@tiptap/react';
import { Table as TableIcon, Columns, Rows } from 'lucide-react';

interface TableControlsProps {
  editor: Editor;
}

export function TableControls({ editor }: TableControlsProps) {
  const isTableActive = editor.isActive('table');

  return (
    <>
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
    </>
  );
}
