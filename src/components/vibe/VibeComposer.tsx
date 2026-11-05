import React from 'react';
import { Send, Square } from 'lucide-react';

interface VibeComposerProps {
  inputPrompt: string;
  setInputPrompt: (val: string) => void;
  hasKey: boolean;
  isStreaming: boolean;
  onSend: (text?: string) => void;
  onStopStreaming: () => void;
}

export function VibeComposer({
  inputPrompt,
  setInputPrompt,
  hasKey,
  isStreaming,
  onSend,
  onStopStreaming,
}: VibeComposerProps) {
  return (
    <div className="px-6 py-4 bg-neutral-900 border-t border-neutral-800">
      <form
        onSubmit={(e) => {
          e.preventDefault();
          onSend();
        }}
        className="relative flex items-center bg-neutral-950 border border-neutral-700 rounded-xl overflow-hidden focus-within:border-indigo-500 transition-colors shadow-inner"
      >
        <textarea
          value={inputPrompt}
          onChange={(e) => setInputPrompt(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              onSend();
            }
          }}
          disabled={!hasKey || isStreaming}
          placeholder={
            !hasKey
              ? 'Configure an LLM API key in Settings to use Vibe.'
              : 'Ask Vibe to edit content, customize templates, or refine styles...'
          }
          rows={2}
          className="w-full bg-transparent px-4 py-3 text-sm text-neutral-100 placeholder-neutral-500 resize-none outline-none disabled:cursor-not-allowed disabled:opacity-60"
        />

        <div className="flex items-center gap-2 pr-3">
          {isStreaming ? (
            <button
              type="button"
              onClick={onStopStreaming}
              className="p-2 rounded-lg bg-red-600 hover:bg-red-500 text-white transition-colors cursor-pointer"
              title="Stop generation"
            >
              <Square className="w-4 h-4 fill-current" />
            </button>
          ) : (
            <button
              type="submit"
              disabled={!hasKey || !inputPrompt.trim()}
              className="p-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-40 disabled:cursor-not-allowed transition-colors cursor-pointer"
              title="Send instruction"
            >
              <Send className="w-4 h-4" />
            </button>
          )}
        </div>
      </form>

      <div className="flex items-center justify-between mt-2 text-[11px] text-neutral-500">
        <span>
          Vibe proposals never apply automatically. You review diffs and explicitly Accept or Reject.
        </span>
        <span>Shift+Enter for newline</span>
      </div>
    </div>
  );
}
