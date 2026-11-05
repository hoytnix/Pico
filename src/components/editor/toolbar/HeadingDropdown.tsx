import React, { useState, useRef, useEffect } from 'react';
import type { Editor } from '@tiptap/react';
import { Heading1, Heading2, Heading3, ChevronDown } from 'lucide-react';

interface HeadingDropdownProps {
  editor: Editor;
}

export function HeadingDropdown({ editor }: HeadingDropdownProps) {
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

  const currentHeadingLabel = () => {
    if (editor.isActive('heading', { level: 1 })) return 'H1';
    if (editor.isActive('heading', { level: 2 })) return 'H2';
    if (editor.isActive('heading', { level: 3 })) return 'H3';
    return 'Text';
  };

  return (
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
  );
}
