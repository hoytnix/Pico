import React, { useState, useEffect, useRef } from 'react';
import { picoApi, pickFolderDialog } from '../../api';
import type { BuildResult, SiteInfo } from '../../types';
import { PreviewHeaderBar } from './PreviewHeaderBar';
import { BuildBanner } from './BuildBanner';

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

  const handleReload = () => {
    if (iframeRef.current && previewUrl) {
      iframeRef.current.src = previewUrl;
    }
  };

  const handleBuild = async () => {
    setIsBuilding(true);
    setBuildResult(null);
    try {
      const res = await picoApi.buildSite(includeDrafts);
      setBuildResult(res);
      handleReload();
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
      <PreviewHeaderBar
        serverStatus={serverStatus}
        previewUrl={previewUrl}
        serverError={serverError}
        includeDrafts={includeDrafts}
        onToggleDrafts={setIncludeDrafts}
        isBuilding={isBuilding}
        onBuild={handleBuild}
        onExport={handleExport}
        isPublishing={isPublishing}
        onPublish={handlePublish}
        onReload={handleReload}
      />

      <BuildBanner buildResult={buildResult} publishMessage={publishMessage} />

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
