import React from 'react';
import { Plus, ArrowUp, ArrowDown, Trash2 } from 'lucide-react';
import type { ModelField } from '../../../types';

interface ModelFieldTableProps {
  fields: ModelField[];
  fieldTypes: string[];
  onAddField: () => void;
  onUpdateField: (index: number, updates: Partial<ModelField>) => void;
  onDeleteField: (index: number) => void;
  onMoveField: (index: number, direction: 'up' | 'down') => void;
}

export function ModelFieldTable({
  fields,
  fieldTypes,
  onAddField,
  onUpdateField,
  onDeleteField,
  onMoveField,
}: ModelFieldTableProps) {
  return (
    <div>
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold text-neutral-200">Frontmatter Fields</h3>
        <button
          type="button"
          onClick={onAddField}
          className="px-2.5 py-1 rounded bg-neutral-800 hover:bg-neutral-700 text-neutral-200 border border-neutral-700 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1"
        >
          <Plus className="w-3.5 h-3.5 text-indigo-400" /> Add Field
        </button>
      </div>

      <div className="border border-neutral-800 rounded-xl overflow-hidden bg-neutral-900/30">
        <table className="w-full text-left text-xs border-collapse">
          <thead className="bg-neutral-900/80 border-b border-neutral-800 text-neutral-400">
            <tr>
              <th className="py-2.5 px-3 font-medium">Field Name</th>
              <th className="py-2.5 px-3 font-medium">Label</th>
              <th className="py-2.5 px-3 font-medium">Type</th>
              <th className="py-2.5 px-3 font-medium text-center">Required</th>
              <th className="py-2.5 px-3 font-medium text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-neutral-800">
            {fields.map((field, idx) => (
              <tr key={idx} className="hover:bg-neutral-800/40 transition-colors">
                <td className="py-2 px-3">
                  <input
                    type="text"
                    value={field.name}
                    onChange={(e) => onUpdateField(idx, { name: e.target.value })}
                    className="bg-neutral-950 border border-neutral-700 rounded px-2 py-1 text-xs font-mono text-neutral-200 outline-none w-32 focus:border-indigo-500"
                  />
                </td>
                <td className="py-2 px-3">
                  <input
                    type="text"
                    value={field.label}
                    onChange={(e) => onUpdateField(idx, { label: e.target.value })}
                    className="bg-neutral-950 border border-neutral-700 rounded px-2 py-1 text-xs text-neutral-200 outline-none w-36 focus:border-indigo-500"
                  />
                </td>
                <td className="py-2 px-3">
                  <select
                    value={field.type}
                    onChange={(e) => onUpdateField(idx, { type: e.target.value })}
                    className="bg-neutral-950 border border-neutral-700 rounded px-2 py-1 text-xs text-neutral-200 outline-none focus:border-indigo-500 [&>option]:bg-neutral-950 [&>option]:text-neutral-100"
                  >
                    {fieldTypes.map((t) => (
                      <option key={t} value={t}>
                        {t}
                      </option>
                    ))}
                  </select>
                </td>
                <td className="py-2 px-3 text-center">
                  <input
                    type="checkbox"
                    checked={field.required}
                    onChange={(e) => onUpdateField(idx, { required: e.target.checked })}
                    className="rounded bg-neutral-800 border-neutral-700 text-indigo-600 focus:ring-0 cursor-pointer"
                  />
                </td>
                <td className="py-2 px-3 text-right">
                  <div className="flex items-center justify-end gap-1">
                    <button
                      type="button"
                      onClick={() => onMoveField(idx, 'up')}
                      disabled={idx === 0}
                      className="p-1 rounded hover:bg-neutral-700 text-neutral-400 hover:text-neutral-200 disabled:opacity-30 cursor-pointer"
                      title="Move up"
                    >
                      <ArrowUp className="w-3.5 h-3.5" />
                    </button>
                    <button
                      type="button"
                      onClick={() => onMoveField(idx, 'down')}
                      disabled={idx === fields.length - 1}
                      className="p-1 rounded hover:bg-neutral-700 text-neutral-400 hover:text-neutral-200 disabled:opacity-30 cursor-pointer"
                      title="Move down"
                    >
                      <ArrowDown className="w-3.5 h-3.5" />
                    </button>
                    <button
                      type="button"
                      onClick={() => onDeleteField(idx)}
                      className="p-1 rounded hover:bg-red-900/60 text-neutral-500 hover:text-red-400 cursor-pointer ml-1"
                      title="Delete field"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
