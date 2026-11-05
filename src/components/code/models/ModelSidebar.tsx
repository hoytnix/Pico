import React from 'react';
import { Layers, Plus } from 'lucide-react';
import type { ModelDef } from '../../../types';

interface ModelSidebarProps {
  models: ModelDef[];
  activeModelName: string | null;
  onSelectModel: (name: string) => void;
  onCreateModel: () => void;
}

export function ModelSidebar({
  models,
  activeModelName,
  onSelectModel,
  onCreateModel,
}: ModelSidebarProps) {
  return (
    <div className="w-60 border-r border-neutral-800 bg-neutral-900/60 flex flex-col shrink-0">
      <div className="p-3 border-b border-neutral-800 flex items-center justify-between">
        <span className="text-xs font-semibold text-neutral-300 uppercase tracking-wider">
          Models
        </span>
        <button
          type="button"
          onClick={onCreateModel}
          className="p-1 rounded bg-indigo-600 hover:bg-indigo-500 text-white transition-colors cursor-pointer"
          title="Create New Model"
        >
          <Plus className="w-3.5 h-3.5" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {models.map((m) => (
          <div
            key={m.name}
            onClick={() => onSelectModel(m.name)}
            className={`flex items-center gap-2 p-2.5 rounded-lg text-xs cursor-pointer transition-colors ${
              activeModelName === m.name
                ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/40'
                : 'hover:bg-neutral-800/80 text-neutral-300'
            }`}
          >
            <Layers className="w-4 h-4 text-indigo-400 shrink-0" />
            <div className="flex-1 truncate font-medium">{m.label || m.name}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
