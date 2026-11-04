import React, { useState } from 'react';
import {
  FolderOpen,
  Plus,
  Clock,
  Sparkles,
  BookOpen,
  Briefcase,
  Compass,
  FileText,
  X,
} from 'lucide-react';
import { picoApi, pickFolderDialog } from '../../api';
import type { RecentSite, SiteInfo } from '../../types';

interface SitePickerModalProps {
  isOpen: boolean;
  recentSites: RecentSite[];
  onSiteOpened: (site: SiteInfo) => void;
  onClose?: () => void;
  allowClose?: boolean;
  isInline?: boolean;
}

const STARTERS = [
  {
    id: 'blog',
    title: 'Blog',
    desc: 'Articles, tag feeds, RSS feed & clean typography.',
    icon: BookOpen,
  },
  {
    id: 'portfolio',
    title: 'Portfolio',
    desc: 'Showcase creative work, case studies & project galleries.',
    icon: Briefcase,
  },
  {
    id: 'landing',
    title: 'Landing Page',
    desc: 'High-conversion product page with features & lead capture.',
    icon: Compass,
  },
  {
    id: 'docs',
    title: 'Documentation',
    desc: 'Multi-section technical guides, search index & code snippets.',
    icon: FileText,
  },
];

export function SitePickerModal({
  isOpen,
  recentSites,
  onSiteOpened,
  onClose,
  allowClose = false,
  isInline = false,
}: SitePickerModalProps) {
  const [isCreatingNew, setIsCreatingNew] = useState(false);
  const [selectedStarter, setSelectedStarter] = useState('blog');
  const [siteTitle, setSiteTitle] = useState('My Awesome Site');
  const [authorName, setAuthorName] = useState('Pico Creator');
  const [loading, setLoading] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleOpenExistingFolder = async () => {
    setErrorMsg(null);
    try {
      const folder = await pickFolderDialog('Select Pico Site Folder');
      if (!folder) return;
      setLoading(true);
      const info = await picoApi.openSite(folder);
      onSiteOpened(info);
    } catch (err: any) {
      setErrorMsg(err.message || String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleOpenRecent = async (path: string) => {
    setErrorMsg(null);
    setLoading(true);
    try {
      const info = await picoApi.openSite(path);
      onSiteOpened(info);
    } catch (err: any) {
      setErrorMsg(err.message || String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleCreateSite = async () => {
    setErrorMsg(null);
    try {
      const targetFolder = await pickFolderDialog('Select Target Folder for New Site');
      if (!targetFolder) return;
      setLoading(true);
      const info = await picoApi.createSite(
        targetFolder,
        selectedStarter,
        siteTitle.trim() || 'My Pico Site',
        authorName.trim() || 'Author'
      );
      onSiteOpened(info);
    } catch (err: any) {
      setErrorMsg(err.message || String(err));
    } finally {
      setLoading(false);
    }
  };

  const modalBody = (
    <div className={`bg-neutral-900 border border-neutral-800 rounded-2xl w-full max-w-xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden text-neutral-100 ${isInline ? 'my-auto' : ''}`}>
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-neutral-800 bg-neutral-900/90 shrink-0">
        <div className="flex items-center gap-2">
          <Sparkles className="w-5 h-5 text-indigo-400" />
          <h2 className="text-base font-bold text-neutral-100">
            {isCreatingNew ? 'Create New Static Site' : 'Open or Create Site'}
          </h2>
        </div>
        {!isInline && allowClose && onClose && (
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded-lg hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0 overflow-y-auto p-6 space-y-6">
          {errorMsg && (
            <div className="p-3 bg-red-950/70 border border-red-800 rounded-lg text-xs text-red-200">
              {errorMsg}
            </div>
          )}

          {!isCreatingNew ? (
            <>
              {/* Main Action Buttons */}
              <div className="grid grid-cols-2 gap-4">
                <button
                  type="button"
                  onClick={handleOpenExistingFolder}
                  disabled={loading}
                  className="p-5 rounded-xl bg-neutral-950 border border-neutral-800 hover:border-indigo-500/80 hover:bg-neutral-800/50 transition-all text-left group cursor-pointer"
                >
                  <FolderOpen className="w-6 h-6 text-indigo-400 mb-2 group-hover:scale-105 transition-transform" />
                  <div className="text-sm font-semibold text-neutral-200">Open Existing Folder</div>
                  <div className="text-xs text-neutral-400 mt-1">
                    Select any folder containing a Pico site layout.
                  </div>
                </button>

                <button
                  type="button"
                  onClick={() => setIsCreatingNew(true)}
                  disabled={loading}
                  className="p-5 rounded-xl bg-neutral-950 border border-neutral-800 hover:border-indigo-500/80 hover:bg-neutral-800/50 transition-all text-left group cursor-pointer"
                >
                  <Plus className="w-6 h-6 text-emerald-400 mb-2 group-hover:scale-105 transition-transform" />
                  <div className="text-sm font-semibold text-neutral-200">Create New Site</div>
                  <div className="text-xs text-neutral-400 mt-1">
                    Start from Blog, Portfolio, Landing, or Docs starters.
                  </div>
                </button>
              </div>

              {/* Recent Sites List */}
              {recentSites && recentSites.length > 0 && (
                <div>
                  <div className="text-xs font-semibold text-neutral-400 uppercase tracking-wider mb-2 flex items-center gap-1.5">
                    <Clock className="w-3.5 h-3.5 text-neutral-500" />
                    <span>Recent Sites</span>
                  </div>
                  <div className="space-y-1.5">
                    {recentSites.map((site) => (
                      <div
                        key={site.path}
                        onClick={() => handleOpenRecent(site.path)}
                        className="p-3 rounded-lg bg-neutral-950 border border-neutral-800 hover:border-neutral-700 hover:bg-neutral-800/40 cursor-pointer transition-colors flex items-center justify-between"
                      >
                        <div className="min-w-0 pr-3">
                          <div className="text-xs font-semibold text-neutral-200 truncate">
                            {site.title || 'Untitled Site'}
                          </div>
                          <div className="text-[11px] font-mono text-neutral-400 truncate mt-0.5">
                            {site.path}
                          </div>
                        </div>
                        <span className="text-[10px] text-neutral-500 shrink-0">
                          {site.last_opened ? new Date(site.last_opened).toLocaleDateString() : ''}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </>
          ) : (
            <div className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-neutral-300 mb-1.5">
                  Select Starter Template
                </label>
                <div className="grid grid-cols-2 gap-3">
                  {STARTERS.map((s) => {
                    const Icon = s.icon;
                    const isSelected = selectedStarter === s.id;
                    return (
                      <div
                        key={s.id}
                        onClick={() => setSelectedStarter(s.id)}
                        className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
                          isSelected
                            ? 'bg-indigo-600/20 border-indigo-500 text-indigo-300'
                            : 'bg-neutral-950 border-neutral-800 text-neutral-300 hover:border-neutral-700'
                        }`}
                      >
                        <Icon className="w-5 h-5 mb-1.5 text-indigo-400" />
                        <div className="text-xs font-semibold">{s.title}</div>
                        <div className="text-[11px] text-neutral-400 mt-0.5 leading-normal">
                          {s.desc}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-semibold text-neutral-300 mb-1">
                    Site Title
                  </label>
                  <input
                    type="text"
                    value={siteTitle}
                    onChange={(e) => setSiteTitle(e.target.value)}
                    className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2 text-xs text-neutral-100 outline-none focus:border-indigo-500"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-neutral-300 mb-1">
                    Author Name
                  </label>
                  <input
                    type="text"
                    value={authorName}
                    onChange={(e) => setAuthorName(e.target.value)}
                    className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2 text-xs text-neutral-100 outline-none focus:border-indigo-500"
                  />
                </div>
              </div>

              <div className="flex items-center justify-between pt-3 border-t border-neutral-800">
                <button
                  type="button"
                  onClick={() => setIsCreatingNew(false)}
                  className="px-4 py-2 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 text-xs font-medium cursor-pointer"
                >
                  Back
                </button>
                <button
                  type="button"
                  onClick={handleCreateSite}
                  disabled={loading}
                  className="px-5 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold cursor-pointer shadow-sm disabled:opacity-50"
                >
                  {loading ? 'Creating Site…' : 'Choose Folder & Create Site'}
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    );

  if (isInline) {
    return modalBody;
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/75 backdrop-blur-xs flex items-center justify-center p-4">
      {modalBody}
    </div>
  );
}
