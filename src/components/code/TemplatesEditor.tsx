import React, { useState, useEffect, useRef } from 'react';
import {
  FileCode,
  RotateCcw,
  Check,
  AlertTriangle,
  Info,
  ChevronRight,
  Sparkles,
} from 'lucide-react';
import { picoApi, confirmAction } from '../../api';
import type { SiteInfo, TemplateContextInfo, VariableDoc } from '../../types';

interface TemplatesEditorProps {
  currentSite: SiteInfo | null;
  onTemplateSaved?: () => void;
}

export function TemplatesEditor({ currentSite, onTemplateSaved }: TemplatesEditorProps) {
  const [templates, setTemplates] = useState<string[]>([]);
  const [activeTemplatePath, setActiveTemplatePath] = useState<string | null>(null);
  const [templateContent, setTemplateContent] = useState<string>('');
  const [contextInfo, setContextInfo] = useState<TemplateContextInfo | null>(null);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [validationError, setValidationError] = useState<string | null>(null);
  const [revertNotice, setRevertNotice] = useState<string | null>(null);

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const loadTemplates = async () => {
    try {
      const list = await picoApi.listTemplates();
      setTemplates(list);
      if (list.length > 0 && !activeTemplatePath) {
        setActiveTemplatePath(list[0]);
      }
    } catch (err: any) {
      console.warn('Failed to list templates:', err);
    }
  };

  useEffect(() => {
    loadTemplates();
  }, [currentSite]);

  useEffect(() => {
    let active = true;
    if (!activeTemplatePath) {
      setTemplateContent('');
      setContextInfo(null);
      return;
    }

    async function loadTemplate() {
      try {
        const content = await picoApi.readTemplate(activeTemplatePath!);
        const ctx = await picoApi.getTemplateContext(activeTemplatePath!);
        if (active) {
          setTemplateContent(content);
          setContextInfo(ctx);
          setValidationError(null);
          setRevertNotice(null);
          setSaveStatus('idle');
        }
      } catch (err: any) {
        console.warn('Failed to read template:', err);
      }
    }

    loadTemplate();

    return () => {
      active = false;
    };
  }, [activeTemplatePath]);

  const handleSave = async () => {
    if (!activeTemplatePath) return;
    setSaveStatus('saving');
    setValidationError(null);
    setRevertNotice(null);
    try {
      await picoApi.saveTemplate(activeTemplatePath, templateContent);
      setSaveStatus('saved');
      setTimeout(() => setSaveStatus('idle'), 2000);
      onTemplateSaved?.();

      // Refresh context info to update has_revert flag
      const ctx = await picoApi.getTemplateContext(activeTemplatePath);
      setContextInfo(ctx);
    } catch (err: any) {
      setSaveStatus('error');
      setValidationError(err.message || String(err));
    }
  };

  const handleRevert = async () => {
    if (!activeTemplatePath) return;
    const confirmed = await confirmAction(
      `Revert ${activeTemplatePath} to its previous version? Your current changes will be overwritten.`
    );
    if (!confirmed) return;

    try {
      const restored = await picoApi.revertTemplate(activeTemplatePath);
      setTemplateContent(restored);
      setValidationError(null);
      setRevertNotice('Template successfully reverted to previous version.');
      setTimeout(() => setRevertNotice(null), 3500);
      onTemplateSaved?.();

      const ctx = await picoApi.getTemplateContext(activeTemplatePath);
      setContextInfo(ctx);
    } catch (err: any) {
      alert(`Failed to revert template: ${err.message || String(err)}`);
    }
  };

  const handleInsertVariable = (varName: string) => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const insertText = `{{ ${varName} }}`;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const newContent =
      templateContent.substring(0, start) + insertText + templateContent.substring(end);

    setTemplateContent(newContent);

    setTimeout(() => {
      textarea.focus();
      textarea.setSelectionRange(start + insertText.length, start + insertText.length);
    }, 50);
  };

  return (
    <div className="templates-editor flex h-full bg-neutral-950 text-neutral-100 overflow-hidden">
      {/* Sidebar: Templates List */}
      <div className="w-64 border-r border-neutral-800 bg-neutral-900/60 flex flex-col shrink-0">
        <div className="p-3 border-b border-neutral-800">
          <span className="text-xs font-semibold text-neutral-300 uppercase tracking-wider">
            MiniJinja Templates
          </span>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {templates.map((tpl) => (
            <div
              key={tpl}
              onClick={() => setActiveTemplatePath(tpl)}
              className={`flex items-center gap-2 p-2.5 rounded-lg text-xs cursor-pointer transition-colors ${
                activeTemplatePath === tpl
                  ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/40'
                  : 'hover:bg-neutral-800/80 text-neutral-300'
              }`}
            >
              <FileCode className="w-4 h-4 text-indigo-400 shrink-0" />
              <div className="flex-1 truncate font-mono text-[11px]">{tpl}</div>
            </div>
          ))}
        </div>
      </div>

      {/* Main Code Editing Area */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {activeTemplatePath ? (
          <>
            {/* Header Toolbar */}
            <div className="px-5 py-3 border-b border-neutral-800 bg-neutral-900/50 flex items-center justify-between">
              <div className="flex items-center gap-2.5">
                <FileCode className="w-4 h-4 text-indigo-400" />
                <span className="font-mono text-xs text-neutral-200">{activeTemplatePath}</span>
                {contextInfo?.is_skeleton && (
                  <span className="text-[10px] px-1.5 py-0.2 rounded bg-neutral-800 text-neutral-400 border border-neutral-700">
                    Base Layout
                  </span>
                )}
                {contextInfo?.is_macro && (
                  <span className="text-[10px] px-1.5 py-0.2 rounded bg-indigo-950 text-indigo-300 border border-indigo-800">
                    Macro
                  </span>
                )}
              </div>

              <div className="flex items-center gap-2">
                {contextInfo?.has_revert && (
                  <button
                    type="button"
                    onClick={handleRevert}
                    className="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 border border-neutral-700 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5"
                    title="Restore previous version"
                  >
                    <RotateCcw className="w-3.5 h-3.5 text-amber-400" /> Revert
                  </button>
                )}
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
                    <span>Save Template</span>
                  )}
                </button>
              </div>
            </div>

            {/* Validation Error Banner */}
            {validationError && (
              <div className="px-5 py-2.5 bg-red-950/70 border-b border-red-800 text-xs text-red-200 flex items-start gap-2 shadow-sm">
                <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
                <div>
                  <strong>MiniJinja Guardrail Violation:</strong>
                  <p className="mt-0.5">{validationError}</p>
                </div>
              </div>
            )}

            {/* Revert Success Notice */}
            {revertNotice && (
              <div className="px-5 py-2 bg-emerald-950/60 border-b border-emerald-800 text-xs text-emerald-300 flex items-center gap-2">
                <Check className="w-4 h-4 text-emerald-400" />
                <span>{revertNotice}</span>
              </div>
            )}

            {/* Editor Code Area */}
            <div className="flex-1 p-4 bg-neutral-950 flex flex-col overflow-hidden">
              <textarea
                ref={textareaRef}
                value={templateContent}
                onChange={(e) => setTemplateContent(e.target.value)}
                className="w-full flex-1 bg-neutral-900/80 border border-neutral-800 rounded-xl p-4 font-mono text-xs text-neutral-200 leading-relaxed outline-none focus:border-indigo-500 resize-none"
                spellCheck={false}
              />
            </div>
          </>
        ) : (
          <div className="flex-1 flex flex-col items-center justify-center text-neutral-500 text-sm">
            <FileCode className="w-10 h-10 mb-2 opacity-40" />
            <p>Select a template from the sidebar to edit.</p>
          </div>
        )}
      </div>

      {/* Right Sidebar: Variable Inspector */}
      {contextInfo && contextInfo.variables && contextInfo.variables.length > 0 && (
        <div className="w-72 border-l border-neutral-800 bg-neutral-900/40 flex flex-col shrink-0">
          <div className="p-3 border-b border-neutral-800 flex items-center gap-1.5">
            <Sparkles className="w-3.5 h-3.5 text-indigo-400" />
            <span className="text-xs font-semibold text-neutral-300 uppercase tracking-wider">
              Available Variables
            </span>
          </div>

          <div className="flex-1 overflow-y-auto p-3 space-y-2">
            <div className="text-[11px] text-neutral-400 leading-relaxed">
              Click a variable to insert it at cursor position:
            </div>

            {contextInfo.variables.map((v, idx) => (
              <div
                key={idx}
                onClick={() => handleInsertVariable(v.name)}
                className="p-2 rounded-lg bg-neutral-900 border border-neutral-800 hover:border-indigo-500/60 cursor-pointer transition-colors group"
              >
                <div className="font-mono text-xs text-indigo-300 group-hover:text-indigo-200 font-medium">
                  {`{{ ${v.name} }}`}
                </div>
                <div className="text-[11px] text-neutral-400 mt-1 leading-normal">
                  {v.description || v.kind}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
