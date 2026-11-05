import React, { useState, useEffect } from 'react';
import { Layers, Trash2, Check, AlertCircle } from 'lucide-react';
import { picoApi, confirmAction } from '../../api';
import { slugify } from '../../lib/slug';
import { ModelSidebar } from './models/ModelSidebar';
import { ModelFieldTable } from './models/ModelFieldTable';
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
    const cleanName = slugify(name);
    if (!cleanName) return;
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
      <ModelSidebar
        models={models}
        activeModelName={activeModelName}
        onSelectModel={setActiveModelName}
        onCreateModel={handleCreateModel}
      />

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
                  Single Item Template
                </label>
                <input
                  type="text"
                  value={activeModel.template || ''}
                  onChange={(e) => setActiveModel({ ...activeModel, template: e.target.value })}
                  placeholder="blog.jinja"
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
            <ModelFieldTable
              fields={activeModel.fields}
              fieldTypes={FIELD_TYPES}
              onAddField={handleAddField}
              onUpdateField={handleUpdateField}
              onDeleteField={handleDeleteField}
              onMoveField={handleMoveField}
            />
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
