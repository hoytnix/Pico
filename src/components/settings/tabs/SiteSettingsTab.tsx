import React from 'react';
import type { SiteConfig } from '../../../types';

interface SiteSettingsTabProps {
  siteConfig: SiteConfig;
  onChange: (config: SiteConfig) => void;
}

export function SiteSettingsTab({ siteConfig, onChange }: SiteSettingsTabProps) {
  return (
    <div className="space-y-4 text-xs">
      <div>
        <label className="block text-xs font-semibold text-neutral-300 mb-1">
          Site Title
        </label>
        <input
          type="text"
          value={siteConfig.title || ''}
          onChange={(e) => onChange({ ...siteConfig, title: e.target.value })}
          className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs text-neutral-100 outline-none focus:border-indigo-500"
        />
      </div>

      <div>
        <label className="block text-xs font-semibold text-neutral-300 mb-1">
          Base URL
        </label>
        <input
          type="text"
          value={siteConfig.base_url || ''}
          onChange={(e) => onChange({ ...siteConfig, base_url: e.target.value })}
          placeholder="https://mysite.com"
          className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
        />
      </div>

      <div>
        <label className="block text-xs font-semibold text-neutral-300 mb-1">
          Site Description
        </label>
        <textarea
          value={siteConfig.description || ''}
          onChange={(e) => onChange({ ...siteConfig, description: e.target.value })}
          rows={2}
          className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs text-neutral-100 outline-none focus:border-indigo-500 resize-none"
        />
      </div>

      <div>
        <label className="block text-xs font-semibold text-neutral-300 mb-1">
          Author Name
        </label>
        <input
          type="text"
          value={siteConfig.author || ''}
          onChange={(e) => onChange({ ...siteConfig, author: e.target.value })}
          className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs text-neutral-100 outline-none focus:border-indigo-500"
        />
      </div>

      <div>
        <label className="block text-xs font-semibold text-neutral-300 mb-1">
          Comments GitHub Repo (e.g. user/my-site-comments)
        </label>
        <input
          type="text"
          value={siteConfig.comments_repo || ''}
          onChange={(e) => onChange({ ...siteConfig, comments_repo: e.target.value })}
          className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
        />
      </div>
    </div>
  );
}
