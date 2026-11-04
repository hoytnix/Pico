import React, { useState, useEffect, useRef } from 'react';
import {
  Play,
  RotateCw,
  ExternalLink,
  PackageCheck,
  UploadCloud,
  CheckCircle2,
  AlertCircle,
  FolderUp,
} from 'lucide-react';
import { picoApi, openUrlInBrowser, pickFolderDialog } from '../../api';
import type { BuildResult, SiteInfo } from '../../types';

interface PreviewPaneProps {
  currentSite: SiteInfo | null;
}

export function PreviewPane({ currentSite }: PreviewPaneProps) {
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [serverStatus, setServerStatus] = useState<'idle' | 'starting' | 'live' | 'error'>('idle');
  const [serverError, setServerError] = useState<string | null>(null);
  const [isBuilding, setIsBuilding] = useState(false);
  const [includeDrafts, setIncludeDrafts] = useState(true);
  const [buildResult, setBuildResult] = useState<BuildResult | null>(null);
  const [publishMessage, setPublishMessage] = useState<string | null>(null);
  const [isPublishing, setIsPublishing] = useState(false);

  const iframeRef = useRef<HTMLIFrameElement>(null);
  const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const lastRevRef = useRef<number>(0);

  // Start preview server once site is opened
  useEffect(() => {
    let active = true;

    async function initPreview() {
      if (!currentSite) return;
      setServerStatus('starting');
      setServerError(null);
      try {
        const url = await picoApi.startPreview();
        if (active) {
          setPreviewUrl(url);
          setServerStatus('live');
        }
      } catch (err: any) {
        if (active) {
          setServerError(err.message || String(err));
          setServerStatus('error');
        }
      }
    }

    initPreview();

    return () => {
      active = false;
    };
  }, [currentSite?.path]);

  // Poll for revision updates to trigger iframe reload
  useEffect(() => {
    if (serverStatus !== 'live' || !previewUrl) return;

    if (pollIntervalRef.current) clearInterval(pollIntervalRef.current);

    pollIntervalRef.current = setInterval(async () => {
      try {
        const rev = await picoApi.getPreviewRevision();
        if (rev > lastRevRef.current) {
          lastRevRef.current = rev;
          if (iframeRef.current) {
            iframeRef.current.src = previewUrl;
          }
        }
      } catch {
        // Ignore polling errors
      }
    }, 600);

    return () => {
      if (pollIntervalRef.current) clearInterval(pollIntervalRef.current);
    };
  }, [serverStatus, previewUrl]);

  const handleBuild = async () => {
    setIsBuilding(true);
    setBuildResult(null);
    try {
      const res = await picoApi.buildSite(includeDrafts);
      setBuildResult(res);
      if (iframeRef.current && previewUrl) {
        iframeRef.current.src = previewUrl;
      }
    } catch (err: any) {
      setBuildResult({
        success: false,
        out_dir: '',
        errors: [{ file: 'build', line: 1, message: err.message || String(err) }],
        pages_built: 0,
        images_generated: 0,
        log: [err.message || String(err)],
      });
    } finally {
      setIsBuilding(false);
    }
  };

  const handleExport = async () => {
    const dest = await pickFolderDialog('Select Export Destination Folder');
    if (!dest) return;
    try {
      await picoApi.exportSite(dest);
      alert(`Site successfully exported to:\n${dest}`);
    } catch (err: any) {
      alert(`Export failed: ${err.message || String(err)}`);
    }
  };

  const handlePublish = async () => {
    setIsPublishing(true);
    setPublishMessage(null);
    try {
      const res = await picoApi.publishGithubPages();
      if (res.success) {
        setPublishMessage(`Published to GitHub Pages: ${res.url || res.message}`);
      } else {
        setPublishMessage(`Publish notice: ${res.message}`);
      }
    } catch (err: any) {
      setPublishMessage(`Publish failed: ${err.message || String(err)}`);
    } finally {
      setIsPublishing(false);
    }
  };

  return (
    <div className="preview-pane flex flex-col h-full bg-neutral-950 text-neutral-100">
      {/* Header — Build/Export & EXACTLY ONE Live Preview Server bar (§9) */}
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
                onClick={() => {
                  if (iframeRef.current && previewUrl) {
                    iframeRef.current.src = previewUrl;
                  }
                }}
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
              onChange={(e) => setIncludeDrafts(e.target.checked)}
              className="rounded border-neutral-700 bg-neutral-800 text-indigo-600 focus:ring-0"
            />
            <span>Drafts</span>
          </label>

          <button
            type="button"
            onClick={handleBuild}
            disabled={isBuilding}
            className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors disabled:opacity-50 cursor-pointer flex items-center gap-1.5 shadow-sm"
          >
            <PackageCheck className="w-3.5 h-3.5" />
            <span>{isBuilding ? 'Building…' : 'Build Site'}</span>
          </button>

          <button
            type="button"
            onClick={handleExport}
            className="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-200 border border-neutral-700 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5"
          >
            <FolderUp className="w-3.5 h-3.5" />
            <span>Export…</span>
          </button>

          <button
            type="button"
            onClick={handlePublish}
            disabled={isPublishing}
            className="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-200 border border-neutral-700 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5"
            title="Publish to GitHub Pages"
          >
            <UploadCloud className="w-3.5 h-3.5" />
            <span>{isPublishing ? 'Publishing…' : 'GitHub Pages'}</span>
          </button>
        </div>
      </div>

      {/* Build / Publish Status Banner */}
      {buildResult && (
        <div
          className={`px-6 py-2 text-xs border-b flex items-center justify-between ${
            buildResult.success
              ? 'bg-emerald-950/40 border-emerald-800 text-emerald-300'
              : 'bg-red-950/40 border-red-800 text-red-300'
          }`}
        >
          <div className="flex items-center gap-2">
            {buildResult.success ? (
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            ) : (
              <AlertCircle className="w-4 h-4 text-red-400" />
            )}
            <span>
              {buildResult.success
                ? `Build succeeded: ${buildResult.pages_built} pages built, ${buildResult.images_generated} images optimized.`
                : `Build failed with ${buildResult.errors.length} error(s).`}
            </span>
          </div>
          {buildResult.errors.length > 0 && (
            <div className="text-right">
              {buildResult.errors[0].file}:{buildResult.errors[0].line} —{' '}
              {buildResult.errors[0].message}
            </div>
          )}
        </div>
      )}

      {publishMessage && (
        <div className="px-6 py-2 bg-indigo-950/40 border-b border-indigo-800 text-indigo-300 text-xs flex items-center gap-2">
          <UploadCloud className="w-3.5 h-3.5 text-indigo-400" />
          <span>{publishMessage}</span>
        </div>
      )}

      {/* Live Preview Iframe */}
      <div className="flex-1 bg-white relative overflow-hidden">
        {previewUrl ? (
          <iframe
            ref={iframeRef}
            src={previewUrl}
            title="Pico Live Site Preview"
            className="w-full h-full border-none"
          />
        ) : (
          <div className="flex items-center justify-center h-full bg-neutral-950 text-neutral-500 text-sm">
            Waiting for preview server to start…
          </div>
        )}
      </div>
    </div>
  );
}
