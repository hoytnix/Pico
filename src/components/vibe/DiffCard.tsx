import React from 'react';
import { Check, X, FileText, ExternalLink, AlertTriangle } from 'lucide-react';
import type { GroundingSource } from '../../types';

interface DiffCardProps {
  targetType: string;
  targetPath: string;
  diff: string;
  explanation?: string;
  validationError?: string | null;
  sources?: GroundingSource[] | null;
  status: 'pending' | 'accepted' | 'rejected';
  onAccept?: () => void;
  onReject?: () => void;
  isProcessing?: boolean;
}

export function DiffCard({
  targetType,
  targetPath,
  diff,
  explanation,
  validationError,
  sources,
  status,
  onAccept,
  onReject,
  isProcessing = false,
}: DiffCardProps) {
  const diffLines = (diff || '').split('\n');

  return (
    <div className="diff-card my-3 border border-neutral-700 bg-neutral-900/90 rounded-lg overflow-hidden shadow-lg text-sm">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 bg-neutral-800/80 border-b border-neutral-700">
        <div className="flex items-center gap-2">
          <FileText className="w-4 h-4 text-indigo-400" />
          <span className="font-mono text-xs text-neutral-200">{targetPath}</span>
          <span className="text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded bg-indigo-950/80 text-indigo-300 border border-indigo-700/50">
            {targetType}
          </span>
        </div>
        <div className="text-xs">
          {status === 'pending' && (
            <span className="text-amber-400 font-medium flex items-center gap-1">
              Approval Requested
            </span>
          )}
          {status === 'accepted' && (
            <span className="text-emerald-400 font-medium flex items-center gap-1">
              <Check className="w-3.5 h-3.5" /> Accepted & Saved
            </span>
          )}
          {status === 'rejected' && (
            <span className="text-red-400 font-medium flex items-center gap-1">
              <X className="w-3.5 h-3.5" /> Rejected
            </span>
          )}
        </div>
      </div>

      {/* Explanation */}
      {explanation && (
        <div className="px-3 py-2 text-xs text-neutral-300 bg-neutral-900/60 border-b border-neutral-800">
          {explanation}
        </div>
      )}

      {/* Validation Error banner */}
      {validationError && (
        <div className="px-3 py-2 text-xs bg-red-950/60 text-red-300 border-b border-red-800 flex items-start gap-2">
          <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
          <div>
            <strong>Guardrail Violation:</strong>
            <p className="mt-0.5">{validationError}</p>
          </div>
        </div>
      )}

      {/* Diff View */}
      <div className="p-3 font-mono text-xs max-h-72 overflow-y-auto bg-neutral-950 select-text leading-5">
        {diffLines.length === 0 || !diff.trim() ? (
          <div className="text-neutral-500 italic">No textual difference detected.</div>
        ) : (
          diffLines.map((line, idx) => {
            const isAdded = line.startsWith('+') && !line.startsWith('+++');
            const isRemoved = line.startsWith('-') && !line.startsWith('---');
            const isHeader = line.startsWith('@@') || line.startsWith('---') || line.startsWith('+++');

            let className = 'text-neutral-400';
            let bgClass = '';
            if (isAdded) {
              className = 'text-emerald-300';
              bgClass = 'bg-emerald-950/40 -mx-3 px-3';
            } else if (isRemoved) {
              className = 'text-red-300';
              bgClass = 'bg-red-950/40 -mx-3 px-3';
            } else if (isHeader) {
              className = 'text-sky-400 font-semibold';
            }

            return (
              <div key={idx} className={`${className} ${bgClass} whitespace-pre-wrap break-all`}>
                {line || ' '}
              </div>
            );
          })
        )}
      </div>

      {/* Sources Panel (for Gemini Native Grounding) */}
      {sources && sources.length > 0 && (
        <div className="px-3 py-2 bg-neutral-900/90 border-t border-neutral-800">
          <div className="text-[11px] font-semibold text-neutral-400 uppercase tracking-wider mb-1.5 flex items-center gap-1.5">
            <span>Grounding Sources</span>
            <span className="text-[10px] bg-neutral-800 text-neutral-300 px-1.5 rounded-full">
              {sources.length}
            </span>
          </div>
          <div className="space-y-1">
            {sources.map((s, idx) => (
              <a
                key={idx}
                href={s.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-indigo-400 hover:text-indigo-300 flex items-center gap-1 truncate transition-colors"
                title={s.title || s.url}
              >
                <ExternalLink className="w-3 h-3 shrink-0" />
                <span className="truncate">{s.title || s.url}</span>
              </a>
            ))}
          </div>
        </div>
      )}

      {/* Action Footer (Accept / Reject) */}
      {status === 'pending' && (
        <div className="flex items-center justify-end gap-2 px-3 py-2.5 bg-neutral-800/80 border-t border-neutral-700">
          <button
            type="button"
            onClick={onReject}
            disabled={isProcessing}
            className="px-3 py-1.5 text-xs font-medium rounded-md bg-neutral-700 hover:bg-neutral-600 text-neutral-200 transition-colors disabled:opacity-50 cursor-pointer flex items-center gap-1"
          >
            <X className="w-3.5 h-3.5" /> Reject
          </button>
          <button
            type="button"
            onClick={onAccept}
            disabled={isProcessing || Boolean(validationError)}
            className="px-3 py-1.5 text-xs font-medium rounded-md bg-emerald-600 hover:bg-emerald-500 text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center gap-1 shadow-sm"
          >
            <Check className="w-3.5 h-3.5" /> Accept & Save
          </button>
        </div>
      )}
    </div>
  );
}
