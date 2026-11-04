import React, { useState, useEffect } from 'react';
import {
  Layers,
  Plus,
  Trash2,
  ArrowUp,
  ArrowDown,
  Check,
  AlertCircle,
} from 'lucide-react';
import { picoApi, confirmAction } from '../../api';
import type { ModelDef, ModelField, SiteInfo } from '../../types';

interface ModelsEditorProps {
  currentSite: SiteInfo | null;
  onModelSaved?: () => void;
}

const FIELD_TYPES = ['string', 'date', 'tags', 'image', 'bool'];

export function ModelsEditor({ currentSite, onModelSaved }: ModelsEditorProps) {
  const [models, setModels] = useState<ModelDef[]>([]);
  const [activeModelName, setActiveModelName] = useState<string | null>(null);
  const [activeModel, setActiveModel] = useState<ModelDef | null>(null);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadModels = async () => {
    try {
      const list = await picoApi.listModels();
      setModels(list);
      if (list.length > 0 && !activeModelName) {
        setActiveModelName(list[0].name);
        setActiveModel(JSON.parse(JSON.stringify(list[0])));
      }
    } catch (err: any) {
      console.warn('Failed to load models:', err);
    }
  };

  useEffect(() => {
    loadModels();
  }, [currentSite]);

  useEffect(() => {
    if (!activeModelName) {
      setActiveModel(null);
      return;
    }
    const found = models.find((m) => m.name === activeModelName);
    if (found) {
      setActiveModel(JSON.parse(JSON.stringify(found)));
    }
  }, [activeModelName, models]);

  const handleAddField = () => {
    if (!activeModel) return;
    const newField: ModelField = {
      name: `field_${activeModel.fields.length + 1}`,
      label: `Field ${activeModel.fields.length + 1}`,
      type: 'string',
      required: false,
    };
    setActiveModel({
      ...activeModel,
      fields: [...activeModel.fields, newField],
    });
  };

  const handleUpdateField = (index: number, updates: Partial<ModelField>) => {
    if (!activeModel) return;
    const updatedFields = [...activeModel.fields];
    updatedFields[index] = { ...updatedFields[index], ...updates };
    setActiveModel({ ...activeModel, fields: updatedFields });
  };

  const handleDeleteField = (index: number) => {
    if (!activeModel) return;
    const updatedFields = activeModel.fields.filter((_, idx) => idx !== index);
    setActiveModel({ ...activeModel, fields: updatedFields });
  };

  const handleMoveField = (index: number, direction: 'up' | 'down') => {
    if (!activeModel) return;
    const targetIdx = direction === 'up' ? index - 1 : index + 1;
    if (targetIdx < 0 || targetIdx >= activeModel.fields.length) return;
    const fields = [...activeModel.fields];
    const temp = fields[index];
    fields[index] = fields[targetIdx];
    fields[targetIdx] = temp;
    setActiveModel({ ...activeModel, fields });
  };

  const handleSaveModel = async () => {
    if (!activeModel) return;
    setSaveStatus('saving');
    setErrorMessage(null);
    try {
      await picoApi.saveModel(activeModel);
      setSaveStatus('saved');
      setTimeout(() => setSaveStatus('idle'), 2000);
      onModelSaved?.();
      await loadModels();
    } catch (err: any) {
      setSaveStatus('error');
      setErrorMessage(err.message || String(err));
    }
  };

  const handleDeleteModel = async () => {
    if (!activeModel) return;
    const confirmed = await confirmAction(`Are you sure you want to delete model "${activeModel.label || activeModel.name}"?`);
    if (!confirmed) return;
    try {
      await picoApi.deleteModel(activeModel.name);
      setActiveModelName(null);
      await loadModels();
      onModelSaved?.();
    } catch (err: any) {
      alert(`Failed to delete model: ${err.message || String(err)}`);
    }
  };

  const handleCreateModel = () => {
    const name = window.prompt('Enter model name (e.g. docs, portfolio):');
    if (!name || !name.trim()) return;
    const cleanName = name.trim().toLowerCase().replace(/[^a-z0-9_-]/g, '');
    const newModel: ModelDef = {
      name: cleanName,
      label: name.trim(),
      template: `${cleanName}.jinja`,
      list_template: `${cleanName}_list.jinja`,
      fields: [
        { name: 'title', label: 'Title', type: 'string', required: true },
        { name: 'date', label: 'Date', type: 'date', required: true },
        { name: 'slug', label: 'Slug', type: 'string', required: true },
        { name: 'draft', label: 'Draft', type: 'bool', required: false },
        { name: 'tags', label: 'Tags', type: 'tags', required: false },
      ],
    };
    picoApi
      .saveModel(newModel)
      .then(async () => {
        await loadModels();
        setActiveModelName(cleanName);
        onModelSaved?.();
      })
      .catch((err) => alert(`Failed to create model: ${err}`));
  };

  return (
    <div className="models-editor flex h-full bg-neutral-950 text-neutral-100 overflow-hidden">
      {/* Sidebar: Models list */}
      <div className="w-60 border-r border-neutral-800 bg-neutral-900/60 flex flex-col shrink-0">
        <div className="p-3 border-b border-neutral-800 flex items-center justify-between">
          <span className="text-xs font-semibold text-neutral-300 uppercase tracking-wider">
            Models
          </span>
          <button
            type="button"
            onClick={handleCreateModel}
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
              onClick={() => setActiveModelName(m.name)}
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

      {/* Main Model Schema View */}
      <div className="flex-1 flex flex-col p-6 overflow-y-auto min-w-0">
        {activeModel ? (
          <div className="max-w-4xl space-y-6">
            <div className="flex items-center justify-between pb-4 border-b border-neutral-800">
              <div>
                <h2 className="text-lg font-bold text-neutral-100 flex items-center gap-2">
                  <span>{activeModel.label || activeModel.name}</span>
                  <span className="text-xs font-mono font-normal text-neutral-400">
                    ({activeModel.name}.yaml)
                  </span>
                </h2>
                <div className="text-xs text-neutral-400 mt-1">
                  Configure fields and template bindings for this collection.
                </div>
              </div>

              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={handleDeleteModel}
                  className="px-3 py-1.5 rounded-lg bg-red-950/60 hover:bg-red-900/80 text-red-300 border border-red-800/60 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5"
                >
                  <Trash2 className="w-3.5 h-3.5" /> Delete Model
                </button>
                <button
                  type="button"
                  onClick={handleSaveModel}
                  disabled={saveStatus === 'saving'}
                  className="px-4 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5 shadow-sm"
                >
                  {saveStatus === 'saved' ? (
                    <>
                      <Check className="w-3.5 h-3.5" /> Saved
                    </>
                  ) : (
                    <span>Save Model</span>
                  )}
                </button>
              </div>
            </div>

            {errorMessage && (
              <div className="p-3 bg-red-950/60 border border-red-800 rounded-lg text-xs text-red-300 flex items-center gap-2">
                <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
                <span>{errorMessage}</span>
              </div>
            )}

            {/* Model Metadata */}
            <div className="grid grid-cols-3 gap-4 bg-neutral-900/40 p-4 rounded-xl border border-neutral-800">
              <div>
                <label className="block text-xs font-medium text-neutral-300 mb-1">
                  Display Label
                </label>
                <input
                  type="text"
                  value={activeModel.label || ''}
                  onChange={(e) => setActiveModel({ ...activeModel, label: e.target.value })}
                  className="w-full bg-neutral-950 border border-neutral-700 rounded-md px-3 py-1.5 text-xs text-neutral-200 outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-neutral-300 mb-1">
                  Single Post Template
                </label>
                <input
                  type="text"
                  value={activeModel.template || ''}
                  onChange={(e) => setActiveModel({ ...activeModel, template: e.target.value })}
                  placeholder="post.jinja"
                  className="w-full bg-neutral-950 border border-neutral-700 rounded-md px-3 py-1.5 text-xs text-neutral-200 font-mono outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-neutral-300 mb-1">
                  List Template
                </label>
                <input
                  type="text"
                  value={activeModel.list_template || ''}
                  onChange={(e) =>
                    setActiveModel({ ...activeModel, list_template: e.target.value })
                  }
                  placeholder="blog.jinja"
                  className="w-full bg-neutral-950 border border-neutral-700 rounded-md px-3 py-1.5 text-xs text-neutral-200 font-mono outline-none focus:border-indigo-500"
                />
              </div>
            </div>

            {/* Fields Schema Table */}
            <div>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-neutral-200">Frontmatter Fields</h3>
                <button
                  type="button"
                  onClick={handleAddField}
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
                    {activeModel.fields.map((field, idx) => (
                      <tr key={idx} className="hover:bg-neutral-800/40 transition-colors">
                        <td className="py-2 px-3">
                          <input
                            type="text"
                            value={field.name}
                            onChange={(e) => handleUpdateField(idx, { name: e.target.value })}
                            className="bg-neutral-950 border border-neutral-700 rounded px-2 py-1 text-xs font-mono text-neutral-200 outline-none w-32 focus:border-indigo-500"
                          />
                        </td>
                        <td className="py-2 px-3">
                          <input
                            type="text"
                            value={field.label}
                            onChange={(e) => handleUpdateField(idx, { label: e.target.value })}
                            className="bg-neutral-950 border border-neutral-700 rounded px-2 py-1 text-xs text-neutral-200 outline-none w-36 focus:border-indigo-500"
                          />
                        </td>
                        <td className="py-2 px-3">
                          <select
                            value={field.type}
                            onChange={(e) => handleUpdateField(idx, { type: e.target.value })}
                            className="bg-neutral-950 border border-neutral-700 rounded px-2 py-1 text-xs text-neutral-200 outline-none focus:border-indigo-500"
                          >
                            {FIELD_TYPES.map((t) => (
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
                            onChange={(e) => handleUpdateField(idx, { required: e.target.checked })}
                            className="rounded bg-neutral-800 border-neutral-700 text-indigo-600 focus:ring-0 cursor-pointer"
                          />
                        </td>
                        <td className="py-2 px-3 text-right">
                          <div className="flex items-center justify-end gap-1">
                            <button
                              type="button"
                              onClick={() => handleMoveField(idx, 'up')}
                              disabled={idx === 0}
                              className="p-1 rounded hover:bg-neutral-700 text-neutral-400 hover:text-neutral-200 disabled:opacity-30 cursor-pointer"
                              title="Move up"
                            >
                              <ArrowUp className="w-3.5 h-3.5" />
                            </button>
                            <button
                              type="button"
                              onClick={() => handleMoveField(idx, 'down')}
                              disabled={idx === activeModel.fields.length - 1}
                              className="p-1 rounded hover:bg-neutral-700 text-neutral-400 hover:text-neutral-200 disabled:opacity-30 cursor-pointer"
                              title="Move down"
                            >
                              <ArrowDown className="w-3.5 h-3.5" />
                            </button>
                            <button
                              type="button"
                              onClick={() => handleDeleteField(idx)}
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
          </div>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-neutral-500 text-sm">
            <Layers className="w-10 h-10 mb-2 opacity-40" />
            <p>Select a model from the sidebar to inspect its fields schema.</p>
          </div>
        )}
      </div>
    </div>
  );
}
