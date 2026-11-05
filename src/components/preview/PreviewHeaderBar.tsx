import React from 'react';
import {
  RotateCw,
  ExternalLink,
  PackageCheck,
  UploadCloud,
  AlertCircle,
  FolderUp,
} from 'lucide-react';
import { openUrlInBrowser } from '../../api';

interface PreviewHeaderBarProps {
  serverStatus: 'idle' | 'starting' | 'live' | 'error';
  previewUrl: string | null;
  serverError: string | null;
  includeDrafts: boolean;
  onToggleDrafts: (checked: boolean) => void;
  isBuilding: boolean;
  onBuild: () => void;
  onExport: () => void;
  isPublishing: boolean;
  onPublish: () => void;
  onReload: () => void;
}

export function PreviewHeaderBar({
  serverStatus,
  previewUrl,
  serverError,
  includeDrafts,
  onToggleDrafts,
  isBuilding,
  onBuild,
  onExport,
  isPublishing,
  onPublish,
  onReload,
}: PreviewHeaderBarProps) {
  return (
    <div className="preview-header flex items-center justify-between px-6 py-2.5 bg-neutral-900 border-b border-neutral-800 flex-wrap gap-3">
      {/* Authoritative Single Live Preview Server bar (§9) */}
      <div className="flex items-center gap-2.5 text-xs">
        <span className="font-semibold text-neutral-300">Live Preview Server:</span>
        {serverStatus === 'starting' && (
          <span className="text-amber-400 flex items-center gap-1.5 animate-pulse">
            <span className="w-2 h-2 rounded-full bg-amber-400" />
            Starting preview server…
          </span>
        )}
        {serverStatus === 'live' && previewUrl && (
          <div className="flex items-center gap-2">
            <span className="text-emerald-400 font-mono font-medium flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-full bg-emerald-400" />
              {previewUrl}
            </span>
            <button
              type="button"
              onClick={() => openUrlInBrowser(previewUrl)}
              className="p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors cursor-pointer"
              title="Open preview in system browser"
            >
              <ExternalLink className="w-3.5 h-3.5" />
            </button>
            <button
              type="button"
              onClick={onReload}
              className="p-1 rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors cursor-pointer"
              title="Reload preview"
            >
              <RotateCw className="w-3.5 h-3.5" />
            </button>
          </div>
        )}
        {serverStatus === 'error' && (
          <span className="text-red-400 flex items-center gap-1">
            <AlertCircle className="w-3.5 h-3.5" />
            {serverError || 'Failed to start server'}
          </span>
        )}
      </div>

      {/* Build & Export Controls */}
      <div className="flex items-center gap-2.5">
        <label className="flex items-center gap-1.5 text-xs text-neutral-300 cursor-pointer select-none">
          <input
            type="checkbox"
            checked={includeDrafts}
            onChange={(e) => onToggleDrafts(e.target.checked)}
            className="rounded border-neutral-700 bg-neutral-800 text-indigo-600 focus:ring-0"
          />
          <span>Drafts</span>
        </label>

        <button
          type="button"
          onClick={onBuild}
          disabled={isBuilding}
          className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors disabled:opacity-50 cursor-pointer flex items-center gap-1.5 shadow-sm"
        >
          <PackageCheck className="w-3.5 h-3.5" />
          <span>{isBuilding ? 'Building…' : 'Build Site'}</span>
        </button>

        <button
          type="button"
          onClick={onExport}
          className="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-200 border border-neutral-700 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5"
        >
          <FolderUp className="w-3.5 h-3.5" />
          <span>Export…</span>
        </button>

        <button
          type="button"
          onClick={onPublish}
          disabled={isPublishing}
          className="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-200 border border-neutral-700 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5"
          title="Publish to GitHub Pages"
        >
          <UploadCloud className="w-3.5 h-3.5" />
          <span>{isPublishing ? 'Publishing…' : 'GitHub Pages'}</span>
        </button>
      </div>
    </div>
  );
}
