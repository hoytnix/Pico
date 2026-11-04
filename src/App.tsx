import React, { useState, useEffect, useRef } from 'react';
import {
  Sparkles,
  Eye,
  Code2,
  FolderOpen,
  Globe,
  Settings,
  HelpCircle,
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

  // Tab order: (1) Vibe, (2) Preview, (3) Code (§5.1, §9)
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
            setIsSitePickerOpen(true);
          }
        } else {
          setIsSitePickerOpen(true);
        }
      } catch (err) {
        console.warn('Initialization failed:', err);
        setIsSitePickerOpen(true);
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

  // Callback when Vibe accepts an edit to update the editor if open (§4)
  const handlePostUpdatedFromVibe = (path: string, newMarkdown: string) => {
    if (activePostPath === path && tiptapEditorRef.current) {
      const html = markdownToEditorHtml(newMarkdown);
      tiptapEditorRef.current.commands.setContent(html, { emitUpdate: false });
    }
  };

  return (
    <div className="pico-desktop-app flex flex-col h-screen w-screen bg-neutral-950 text-neutral-100 overflow-hidden font-sans select-none">
      {/* Topnav Bar — NO Tab buttons per §5.1 & §9 */}
      <header className="pico-topbar flex items-center justify-between px-5 py-2.5 bg-neutral-900 border-b border-neutral-800 shrink-0">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <span className="font-black text-sm tracking-tight text-white bg-indigo-600 px-2 py-0.5 rounded-md">
              Pico
            </span>
            <span className="text-xs text-neutral-400 font-medium hidden sm:inline">
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

      {/* Main Content Area (Zero-Remount Persistent Panes §5.1, §9) */}
      <main className="flex-1 relative overflow-hidden pb-[52px]">
        {/* Pane 1: Vibe */}
        <div
          className={`h-full w-full absolute inset-0 ${
            activeTab === 'vibe' ? 'visible z-10' : 'invisible -z-10'
          }`}
          style={{ display: activeTab === 'vibe' ? 'block' : 'none' }}
        >
          <VibePane
            currentSite={currentSite}
            llmSettings={llmSettings}
            activePostPath={activePostPath}
            onPostUpdatedFromVibe={handlePostUpdatedFromVibe}
            onOpenSettings={() => setIsSettingsOpen(true)}
          />
        </div>

        {/* Pane 2: Preview */}
        <div
          className={`h-full w-full absolute inset-0 ${
            activeTab === 'preview' ? 'visible z-10' : 'invisible -z-10'
          }`}
          style={{ display: activeTab === 'preview' ? 'block' : 'none' }}
        >
          <PreviewPane currentSite={currentSite} />
        </div>

        {/* Pane 3: Code */}
        <div
          className={`h-full w-full absolute inset-0 ${
            activeTab === 'code' ? 'visible z-10' : 'invisible -z-10'
          }`}
          style={{ display: activeTab === 'code' ? 'block' : 'none' }}
        >
          <CodePane
            currentSite={currentSite}
            activePostPath={activePostPath}
            setActivePostPath={setActivePostPath}
            onOpenSettings={() => setIsSettingsOpen(true)}
            tiptapEditorRef={tiptapEditorRef}
          />
        </div>
      </main>

      {/* Bottom Bar — THE ONLY tab location, sized for desktop (§5.1, §9) */}
      <nav
        role="tablist"
        aria-label="Studio Primary Navigation"
        className="pico-bottom-bar fixed bottom-0 left-0 right-0 h-[52px] bg-neutral-900 border-t border-neutral-800 flex items-center justify-center gap-2 px-6 z-40 select-none shadow-2xl"
      >
        {/* Tab 1: Vibe */}
        <button
          type="button"
          role="tab"
          id="tab-vibe"
          aria-selected={activeTab === 'vibe'}
          aria-controls="pane-vibe"
          onClick={() => setActiveTab('vibe')}
          className={`h-9 px-6 rounded-lg text-xs font-semibold flex items-center gap-2 transition-all cursor-pointer ${
            activeTab === 'vibe'
              ? 'bg-indigo-600 text-white shadow-md'
              : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800'
          }`}
        >
          <Sparkles className="w-4 h-4" />
          <span>Vibe</span>
        </button>

        {/* Tab 2: Preview */}
        <button
          type="button"
          role="tab"
          id="tab-preview"
          aria-selected={activeTab === 'preview'}
          aria-controls="pane-preview"
          onClick={() => setActiveTab('preview')}
          className={`h-9 px-6 rounded-lg text-xs font-semibold flex items-center gap-2 transition-all cursor-pointer ${
            activeTab === 'preview'
              ? 'bg-indigo-600 text-white shadow-md'
              : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800'
          }`}
        >
          <Eye className="w-4 h-4" />
          <span>Preview</span>
        </button>

        {/* Tab 3: Code */}
        <button
          type="button"
          role="tab"
          id="tab-code"
          aria-selected={activeTab === 'code'}
          aria-controls="pane-code"
          onClick={() => setActiveTab('code')}
          className={`h-9 px-6 rounded-lg text-xs font-semibold flex items-center gap-2 transition-all cursor-pointer ${
            activeTab === 'code'
              ? 'bg-indigo-600 text-white shadow-md'
              : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800'
          }`}
        >
          <Code2 className="w-4 h-4" />
          <span>Code</span>
        </button>
      </nav>

      {/* Settings Modal */}
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        onSettingsSaved={refreshLlmSettings}
      />

      {/* Site Picker Modal */}
      <SitePickerModal
        isOpen={isSitePickerOpen}
        recentSites={recentSites}
        onSiteOpened={handleSiteOpened}
        onClose={() => setIsSitePickerOpen(false)}
        allowClose={Boolean(currentSite)}
      />
    </div>
  );
}
