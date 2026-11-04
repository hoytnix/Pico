import React, { useState, useEffect, useRef } from 'react';
import {
  Sparkles,
  Eye,
  Code2,
  FolderOpen,
} from 'lucide-react';
import { picoApi } from './api';
import { markdownToEditorHtml } from './lib/markdown/serializer';
import { VibePane } from './components/vibe/VibePane';
import { PreviewPane } from './components/preview/PreviewPane';
import { CodePane } from './components/code/CodePane';
import { SettingsModal } from './components/settings/SettingsModal';
import { SitePickerModal } from './components/site/SitePickerModal';
import type { LlmSettingsView, RecentSite, SiteInfo } from './types';

export type MainTab = 'vibe' | 'preview' | 'code';

export function App() {
  const [currentSite, setCurrentSite] = useState<SiteInfo | null>(null);
  const [recentSites, setRecentSites] = useState<RecentSite[]>([]);
  const [llmSettings, setLlmSettings] = useState<LlmSettingsView | null>(null);

  // Tab order: (1) Vibe, (2) Preview, (3) Code
  const [activeTab, setActiveTab] = useState<MainTab>('code');

  // Modals
  const [isSitePickerOpen, setIsSitePickerOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  // Shared active editing state across tabs
  const [activePostPath, setActivePostPath] = useState<string | null>(null);
  const tiptapEditorRef = useRef<any>(null);

  // Initial load
  useEffect(() => {
    async function initApp() {
      try {
        const [recent, llm] = await Promise.all([
          picoApi.getRecentSites(),
          picoApi.getLlmSettings(),
        ]);
        setRecentSites(recent);
        setLlmSettings(llm);

        // If there is a most recent site, open it automatically
        if (recent.length > 0 && recent[0].path) {
          try {
            const site = await picoApi.openSite(recent[0].path);
            setCurrentSite(site);
          } catch {
            // Keep currentSite as null to show no-site view
          }
        }
      } catch (err) {
        console.warn('Initialization failed:', err);
      }
    }

    initApp();
  }, []);

  const refreshLlmSettings = async () => {
    try {
      const llm = await picoApi.getLlmSettings();
      setLlmSettings(llm);
    } catch (err) {
      console.warn('Failed to refresh LLM settings:', err);
    }
  };

  const handleSiteOpened = (site: SiteInfo) => {
    setCurrentSite(site);
    setIsSitePickerOpen(false);
    picoApi.getRecentSites().then(setRecentSites).catch(() => {});
  };

  // Callback when Vibe accepts an edit to update the editor if open
  const handlePostUpdatedFromVibe = (path: string, newMarkdown: string) => {
    if (activePostPath === path && tiptapEditorRef.current) {
      const html = markdownToEditorHtml(newMarkdown);
      tiptapEditorRef.current.commands.setContent(html, { emitUpdate: false });
    }
  };

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-neutral-950 text-neutral-100 font-sans select-none">
      {/* Header: shrink-0, app title / Switch Site only, NO tab buttons */}
      <header className="shrink-0 flex items-center justify-between px-5 py-2.5 bg-neutral-900 border-b border-neutral-800">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <span className="font-black text-sm tracking-tight text-white bg-indigo-600 px-2 py-0.5 rounded-md">
              Pico
            </span>
            <span className="text-xs text-neutral-400 font-medium">
              Desktop Static Studio
            </span>
          </div>

          {currentSite && (
            <div className="flex items-center gap-2 border-l border-neutral-800 pl-3">
              <span className="text-xs font-semibold text-neutral-200">
                {currentSite.config?.title || 'Untitled Site'}
              </span>
              <span className="text-[11px] font-mono text-neutral-500 truncate max-w-xs">
                {currentSite.path}
              </span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setIsSitePickerOpen(true)}
            className="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-200 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5 border border-neutral-700"
            title="Open or Switch Site"
          >
            <FolderOpen className="w-3.5 h-3.5 text-indigo-400" />
            <span>Switch Site</span>
          </button>
        </div>
      </header>

      {/* Main: flex-1 min-h-0 overflow-auto (or overflow-hidden with internal scrolling per pane) */}
      <main className="flex-1 min-h-0 relative overflow-hidden bg-neutral-950">
        {/* No-site state: When no site folder is open, show ONLY Open / Create Site + Recent Sites in main area */}
        <div
          className={`h-full w-full overflow-y-auto p-6 flex items-center justify-center ${
            !currentSite ? 'block' : 'hidden'
          }`}
        >
          <SitePickerModal
            isOpen={!currentSite}
            isInline={true}
            recentSites={recentSites}
            onSiteOpened={handleSiteOpened}
          />
        </div>

        {/* Vibe pane: active ? "block" : "hidden" */}
        <div
          className={`h-full w-full ${
            currentSite && activeTab === 'vibe' ? 'block' : 'hidden'
          }`}
        >
          <VibePane
            currentSite={currentSite}
            llmSettings={llmSettings}
            activePostPath={activePostPath}
            onPostUpdatedFromVibe={handlePostUpdatedFromVibe}
            onOpenSettings={() => {
              setActiveTab('code');
            }}
          />
        </div>

        {/* Preview pane: active ? "block" : "hidden" */}
        <div
          className={`h-full w-full ${
            currentSite && activeTab === 'preview' ? 'block' : 'hidden'
          }`}
        >
          <PreviewPane currentSite={currentSite} />
        </div>

        {/* Code pane: active ? "block" : "hidden" */}
        <div
          className={`h-full w-full ${
            currentSite && activeTab === 'code' ? 'block' : 'hidden'
          }`}
        >
          <CodePane
            currentSite={currentSite}
            activePostPath={activePostPath}
            setActivePostPath={setActivePostPath}
            onOpenSettings={() => setIsSettingsOpen(true)}
            onLlmSettingsChanged={refreshLlmSettings}
            tiptapEditorRef={tiptapEditorRef}
          />
        </div>
      </main>

      {/* Bottom bar: shrink-0, full width, always visible, Vibe / Preview / Code only */}
      <nav
        role="tablist"
        aria-label="Studio Primary Navigation"
        className="shrink-0 w-full h-[52px] bg-neutral-900 border-t border-neutral-800 flex items-center justify-center gap-3 px-6 select-none shadow-2xl z-20"
      >
        {/* Tab 1: Vibe */}
        <button
          type="button"
          role="tab"
          id="tab-vibe"
          aria-selected={currentSite ? activeTab === 'vibe' : false}
          aria-controls="pane-vibe"
          disabled={!currentSite}
          onClick={() => currentSite && setActiveTab('vibe')}
          className={`h-9 px-6 rounded-lg text-xs font-semibold flex items-center gap-2 transition-all ${
            !currentSite
              ? 'opacity-40 cursor-not-allowed text-neutral-500'
              : activeTab === 'vibe'
              ? 'bg-indigo-600 text-white shadow-md cursor-pointer'
              : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800 cursor-pointer'
          }`}
          title={!currentSite ? 'Open a site to use Vibe' : 'Vibe AI Assistant'}
        >
          <Sparkles className="w-4 h-4" />
          <span>Vibe</span>
        </button>

        {/* Tab 2: Preview */}
        <button
          type="button"
          role="tab"
          id="tab-preview"
          aria-selected={currentSite ? activeTab === 'preview' : false}
          aria-controls="pane-preview"
          disabled={!currentSite}
          onClick={() => currentSite && setActiveTab('preview')}
          className={`h-9 px-6 rounded-lg text-xs font-semibold flex items-center gap-2 transition-all ${
            !currentSite
              ? 'opacity-40 cursor-not-allowed text-neutral-500'
              : activeTab === 'preview'
              ? 'bg-indigo-600 text-white shadow-md cursor-pointer'
              : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800 cursor-pointer'
          }`}
          title={!currentSite ? 'Open a site to preview' : 'Live Preview & Publish'}
        >
          <Eye className="w-4 h-4" />
          <span>Preview</span>
        </button>

        {/* Tab 3: Code */}
        <button
          type="button"
          role="tab"
          id="tab-code"
          aria-selected={currentSite ? activeTab === 'code' : false}
          aria-controls="pane-code"
          disabled={!currentSite}
          onClick={() => currentSite && setActiveTab('code')}
          className={`h-9 px-6 rounded-lg text-xs font-semibold flex items-center gap-2 transition-all ${
            !currentSite
              ? 'opacity-40 cursor-not-allowed text-neutral-500'
              : activeTab === 'code'
              ? 'bg-indigo-600 text-white shadow-md cursor-pointer'
              : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800 cursor-pointer'
          }`}
          title={!currentSite ? 'Open a site to edit code' : 'Code & Content Editor'}
        >
          <Code2 className="w-4 h-4" />
          <span>Code</span>
        </button>
      </nav>

      {/* Settings Modal (Overlay for external triggers) */}
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        onSettingsSaved={refreshLlmSettings}
      />

      {/* Site Picker Modal (Overlay when switching sites from header) */}
      {currentSite && isSitePickerOpen && (
        <SitePickerModal
          isOpen={isSitePickerOpen}
          isInline={false}
          recentSites={recentSites}
          onSiteOpened={handleSiteOpened}
          onClose={() => setIsSitePickerOpen(false)}
          allowClose={true}
        />
      )}
    </div>
  );
}
