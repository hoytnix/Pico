import React, { useState, useEffect } from 'react';
import { Palette, Check, AlertCircle } from 'lucide-react';
import { picoApi } from '../../api';
import type { SiteInfo } from '../../types';

interface ThemeEditorProps {
  currentSite: SiteInfo | null;
  onThemeSaved?: () => void;
}

export function ThemeEditor({ currentSite, onThemeSaved }: ThemeEditorProps) {
  const [themeCss, setThemeCss] = useState('');
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadTheme = async () => {
    try {
      const content = await picoApi.readTheme();
      setThemeCss(content);
      setErrorMessage(null);
    } catch (err: any) {
      console.warn('Failed to read theme:', err);
      setErrorMessage(err.message || String(err));
    }
  };

  useEffect(() => {
    loadTheme();
  }, [currentSite]);

  const handleSave = async () => {
    setSaveStatus('saving');
    setErrorMessage(null);
    try {
      await picoApi.saveTheme(themeCss);
      setSaveStatus('saved');
      setTimeout(() => setSaveStatus('idle'), 2000);
      onThemeSaved?.();
    } catch (err: any) {
      setSaveStatus('error');
      setErrorMessage(err.message || String(err));
    }
  };

  return (
    <div className="theme-editor flex flex-col h-full bg-neutral-950 text-neutral-100 p-6 overflow-hidden">
      <div className="flex items-center justify-between pb-4 border-b border-neutral-800">
        <div className="flex items-center gap-2.5">
          <Palette className="w-5 h-5 text-indigo-400" />
          <div>
            <h2 className="text-sm font-semibold text-neutral-200">Theme CSS Customizer</h2>
            <p className="text-xs text-neutral-400">
              Edit CSS variables in <code className="text-indigo-300 font-mono">assets/theme.css</code> without altering HTML structure.
            </p>
          </div>
        </div>

        <button
          type="button"
          onClick={handleSave}
          disabled={saveStatus === 'saving'}
          className="px-4 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5 shadow-sm"
        >
          {saveStatus === 'saved' ? (
            <>
              <Check className="w-3.5 h-3.5" /> Saved
            </>
          ) : (
            <span>Save Theme</span>
          )}
        </button>
      </div>

      {errorMessage && (
        <div className="mt-3 p-3 bg-red-950/60 border border-red-800 rounded-lg text-xs text-red-300 flex items-center gap-2">
          <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
          <span>{errorMessage}</span>
        </div>
      )}

      <div className="flex-1 mt-4 flex flex-col overflow-hidden">
        <textarea
          value={themeCss}
          onChange={(e) => setThemeCss(e.target.value)}
          className="w-full flex-1 bg-neutral-900 border border-neutral-800 rounded-xl p-4 font-mono text-xs text-neutral-200 leading-relaxed outline-none focus:border-indigo-500 resize-none"
          spellCheck={false}
          placeholder=":root { --primary-color: #6366f1; ... }"
        />
      </div>
    </div>
  );
}
