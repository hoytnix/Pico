import React from 'react';
import { Check, Clock } from 'lucide-react';

interface FrontmatterFormProps {
  frontmatter: Record<string, unknown>;
  saveStatus: 'saved' | 'saving' | 'unsaved';
  onChange: (key: string, value: any) => void;
}

export function FrontmatterForm({
  frontmatter,
  saveStatus,
  onChange,
}: FrontmatterFormProps) {
  return (
    <div className="p-4 border-b border-neutral-800 bg-neutral-900/40">
      <div className="flex items-center justify-between mb-3">
        <input
          type="text"
          value={(frontmatter.title as string) || ''}
          onChange={(e) => onChange('title', e.target.value)}
          placeholder="Post Title"
          className="text-lg font-bold text-neutral-100 bg-transparent outline-none flex-1 mr-4 focus:border-b focus:border-indigo-500"
        />
        <div className="flex items-center gap-3 text-xs">
          {saveStatus === 'saving' && (
            <span className="text-amber-400 flex items-center gap-1">
              <Clock className="w-3.5 h-3.5 animate-spin" /> Saving…
            </span>
          )}
          {saveStatus === 'saved' && (
            <span className="text-emerald-400 flex items-center gap-1">
              <Check className="w-3.5 h-3.5" /> Saved
            </span>
          )}
          {saveStatus === 'unsaved' && (
            <span className="text-amber-400 flex items-center gap-1">
              Unsaved changes
            </span>
          )}
          <label className="flex items-center gap-1.5 cursor-pointer text-neutral-300">
            <input
              type="checkbox"
              checked={Boolean(frontmatter.draft)}
              onChange={(e) => onChange('draft', e.target.checked)}
              className="rounded bg-neutral-800 border-neutral-700 text-indigo-600 focus:ring-0"
            />
            <span>Draft</span>
          </label>
        </div>
      </div>

      {/* Secondary Properties Row */}
      <div className="grid grid-cols-3 gap-3 text-xs">
        <div>
          <label className="block text-[11px] text-neutral-400 mb-1">Date</label>
          <input
            type="date"
            value={String(frontmatter.date || '')}
            onChange={(e) => onChange('date', e.target.value)}
            className="w-full bg-neutral-950 border border-neutral-700 rounded px-2.5 py-1 text-neutral-200 outline-none focus:border-indigo-500"
          />
        </div>
        <div>
          <label className="block text-[11px] text-neutral-400 mb-1">Slug</label>
          <input
            type="text"
            value={(frontmatter.slug as string) || ''}
            onChange={(e) => onChange('slug', e.target.value)}
            className="w-full bg-neutral-950 border border-neutral-700 rounded px-2.5 py-1 text-neutral-200 font-mono outline-none focus:border-indigo-500"
          />
        </div>
        <div>
          <label className="block text-[11px] text-neutral-400 mb-1">Tags (comma-separated)</label>
          <input
            type="text"
            value={
              Array.isArray(frontmatter.tags)
                ? frontmatter.tags.join(', ')
                : String(frontmatter.tags || '')
            }
            onChange={(e) => {
              const tagsArray = e.target.value
                .split(',')
                .map((t) => t.trim())
                .filter(Boolean);
              onChange('tags', tagsArray);
            }}
            className="w-full bg-neutral-950 border border-neutral-700 rounded px-2.5 py-1 text-neutral-200 outline-none focus:border-indigo-500"
          />
        </div>
      </div>
    </div>
  );
}
