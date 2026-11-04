import { type AnyExtension } from '@tiptap/core';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import TaskList from '@tiptap/extension-task-list';
import TaskItem from '@tiptap/extension-task-item';
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight';
import { Table } from '@tiptap/extension-table';
import { TableRow } from '@tiptap/extension-table-row';
import { TableCell } from '@tiptap/extension-table-cell';
import { TableHeader } from '@tiptap/extension-table-header';
import Image from '@tiptap/extension-image';
import Underline from '@tiptap/extension-underline';
import Link from '@tiptap/extension-link';
import { createLowlight, all } from 'lowlight';
import { SlashCommands } from '../extensions/SlashCommands';

export const lowlight = createLowlight(all);

export interface EditorCallbacks {
  onSlashMenuStart?: (props: any) => void;
  onSlashMenuUpdate?: (props: any) => void;
  onSlashMenuKeyDown?: (props: any) => boolean;
  onSlashMenuExit?: () => void;
}

export function getEditorExtensions(callbacks?: EditorCallbacks): AnyExtension[] {
  const extensions: AnyExtension[] = [
    StarterKit.configure({
      codeBlock: false,
      heading: { levels: [1, 2, 3, 4, 5, 6] },
      paragraph: {},
      hardBreak: { keepMarks: true },
    }),
    CodeBlockLowlight.configure({
      lowlight,
      defaultLanguage: 'plaintext',
    }),
    Placeholder.configure({
      placeholder: 'Write your story, notes, or documentation...',
    }),
    Underline,
    Link.configure({
      openOnClick: false,
      autolink: true,
      defaultProtocol: 'https',
      HTMLAttributes: {
        rel: 'noopener noreferrer',
        target: null,
      },
    }),
    TaskList,
    TaskItem.configure({ nested: true }),
    Table.configure({ resizable: true }),
    TableRow,
    TableHeader,
    TableCell,
    Image.configure({
      inline: true,
      allowBase64: true,
      HTMLAttributes: {
        class: 'editor-image rounded-md max-w-full my-2',
      },
    }),
  ];

  if (callbacks?.onSlashMenuStart) {
    extensions.push(
      SlashCommands.configure({
        suggestion: {
          render: () => ({
            onStart: callbacks.onSlashMenuStart,
            onUpdate: callbacks.onSlashMenuUpdate,
            onKeyDown: callbacks.onSlashMenuKeyDown,
            onExit: callbacks.onSlashMenuExit,
          }),
        },
      })
    );
  }

  return extensions;
}
