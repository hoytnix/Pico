import React from 'react';
import { Sparkles, Trash2 } from 'lucide-react';

interface VibeHeaderProps {
  providerId: string;
  model: string;
  onClearHistory: () => void;
}

export function VibeHeader({ providerId, model, onClearHistory }: VibeHeaderProps) {
  return (
    <div className="flex items-center justify-between px-6 py-3 bg-neutral-900 border-b border-neutral-800">
      <div className="flex items-center gap-2.5">
        <div className="p-1.5 rounded-lg bg-indigo-600/20 text-indigo-400 border border-indigo-500/30">
          <Sparkles className="w-4 h-4" />
        </div>
        <div>
          <div className="text-sm font-semibold text-neutral-100 flex items-center gap-2">
            <span>Vibe AI Chat</span>
            <span className="text-[11px] font-mono font-normal px-2 py-0.5 rounded-full bg-neutral-800 text-neutral-300 border border-neutral-700">
              {providerId} / {model}
            </span>
          </div>
          <div className="text-xs text-neutral-400">
            Desktop AI pair programmer for Markdown, MiniJinja templates & theme CSS
          </div>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={onClearHistory}
          className="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors cursor-pointer text-xs flex items-center gap-1"
          title="Clear conversation"
        >
          <Trash2 className="w-3.5 h-3.5" /> Clear
        </button>
      </div>
    </div>
  );
}
