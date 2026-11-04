import React, { useState } from 'react';
import { Settings, FileText, Layers, FileCode, Palette } from 'lucide-react';
import { ContentEditor } from './ContentEditor';
import { ModelsEditor } from './ModelsEditor';
import { TemplatesEditor } from './TemplatesEditor';
import { ThemeEditor } from './ThemeEditor';
import type { SiteInfo } from '../../types';

interface CodePaneProps {
  currentSite: SiteInfo | null;
  activePostPath: string | null;
  setActivePostPath: (path: string | null) => void;
  onOpenSettings: () => void;
  onPostSaved?: () => void;
  onModelSaved?: () => void;
  onTemplateSaved?: () => void;
  onThemeSaved?: () => void;
  tiptapEditorRef?: React.MutableRefObject<any>;
}

export type CodeSubView = 'content' | 'models' | 'templates' | 'theme';

export function CodePane({
  currentSite,
  activePostPath,
  setActivePostPath,
  onOpenSettings,
  onPostSaved,
  onModelSaved,
  onTemplateSaved,
  onThemeSaved,
  tiptapEditorRef,
}: CodePaneProps) {
  const [activeSubView, setActiveSubView] = useState<CodeSubView>('content');

  return (
    <div className="code-pane flex flex-col h-full bg-neutral-950 text-neutral-100 overflow-hidden">
      {/* Code Header: Sub-view tabs on left, Settings gear on right (§5.1, §9) */}
      <div className="code-header flex items-center justify-between px-6 py-2 bg-neutral-900 border-b border-neutral-800">
        <div className="flex items-center gap-1 bg-neutral-950 p-1 rounded-lg border border-neutral-800">
          <button
            type="button"
            onClick={() => setActiveSubView('content')}
            className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-colors cursor-pointer ${
              activeSubView === 'content'
                ? 'bg-indigo-600 text-white shadow-xs'
                : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800'
            }`}
          >
            <FileText className="w-3.5 h-3.5" />
            <span>Content</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveSubView('models')}
            className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-colors cursor-pointer ${
              activeSubView === 'models'
                ? 'bg-indigo-600 text-white shadow-xs'
                : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800'
            }`}
          >
            <Layers className="w-3.5 h-3.5" />
            <span>Models</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveSubView('templates')}
            className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-colors cursor-pointer ${
              activeSubView === 'templates'
                ? 'bg-indigo-600 text-white shadow-xs'
                : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800'
            }`}
          >
            <FileCode className="w-3.5 h-3.5" />
            <span>Templates</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveSubView('theme')}
            className={`flex items-center gap-1.5 px-3 py-1 rounded-md text-xs font-medium transition-colors cursor-pointer ${
              activeSubView === 'theme'
                ? 'bg-indigo-600 text-white shadow-xs'
                : 'text-neutral-400 hover:text-neutral-200 hover:bg-neutral-800'
            }`}
          >
            <Palette className="w-3.5 h-3.5" />
            <span>Theme</span>
          </button>
        </div>

        {/* Settings gear on right (§5.1, §9) */}
        <button
          type="button"
          onClick={onOpenSettings}
          className="p-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 hover:text-white transition-colors cursor-pointer flex items-center gap-1.5 text-xs font-medium border border-neutral-700"
          title="Open Settings"
        >
          <Settings className="w-4 h-4" />
          <span>Settings</span>
        </button>
      </div>

      {/* Sub-view Content */}
      <div className="flex-1 overflow-hidden relative">
        <div className={`h-full ${activeSubView === 'content' ? 'block' : 'hidden'}`}>
          <ContentEditor
            currentSite={currentSite}
            activePostPath={activePostPath}
            setActivePostPath={setActivePostPath}
            onPostSaved={onPostSaved}
            tiptapEditorRef={tiptapEditorRef}
          />
        </div>

        <div className={`h-full ${activeSubView === 'models' ? 'block' : 'hidden'}`}>
          <ModelsEditor currentSite={currentSite} onModelSaved={onModelSaved} />
        </div>

        <div className={`h-full ${activeSubView === 'templates' ? 'block' : 'hidden'}`}>
          <TemplatesEditor currentSite={currentSite} onTemplateSaved={onTemplateSaved} />
        </div>

        <div className={`h-full ${activeSubView === 'theme' ? 'block' : 'hidden'}`}>
          <ThemeEditor currentSite={currentSite} onThemeSaved={onThemeSaved} />
        </div>
      </div>
    </div>
  );
}
