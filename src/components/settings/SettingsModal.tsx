import React, { useState, useEffect } from 'react';
import {
  X,
  Key,
  Globe,
  Sliders,
  CheckCircle2,
  AlertCircle,
  Save,
} from 'lucide-react';
import { picoApi } from '../../api';
import { normalizeGoogleModelId } from '../../lib/llmProviders';
import { LlmSettingsTab } from './tabs/LlmSettingsTab';
import { SiteSettingsTab } from './tabs/SiteSettingsTab';
import type {
  LlmSettingsView,
  ProviderInfo,
  SaveLlmSettingsInput,
  SiteConfig,
  TestConnectionResult,
} from '../../types';

interface SettingsModalProps {
  isOpen: boolean;
  onClose?: () => void;
  onSettingsSaved?: () => void;
  isInline?: boolean;
}

export function SettingsModal({ isOpen, onClose, onSettingsSaved, isInline = false }: SettingsModalProps) {
  const [activeTab, setActiveTab] = useState<'llm' | 'site'>('llm');

  // LLM settings
  const [providersRegistry, setProvidersRegistry] = useState<ProviderInfo[]>([]);
  const [llmSettings, setLlmSettings] = useState<LlmSettingsView | null>(null);
  const [selectedProviderId, setSelectedProviderId] = useState<string>('gemini-native');
  const [apiKeyInput, setApiKeyInput] = useState<string>('');
  const [customModelInput, setCustomModelInput] = useState<string>('');
  const [customBaseUrlInput, setCustomBaseUrlInput] = useState<string>('');
  const [selectedModel, setSelectedModel] = useState<string>('gemini-4-argon');
  const [selectedThinking, setSelectedThinking] = useState<string>('medium');
  const [groundingEnabled, setGroundingEnabled] = useState<boolean>(false);
  const [githubTokenInput, setGithubTokenInput] = useState<string>('');

  // Test connection state
  const [testingConnection, setTestingConnection] = useState(false);
  const [testResult, setTestResult] = useState<TestConnectionResult | null>(null);

  // Site.yaml config
  const [siteConfig, setSiteConfig] = useState<SiteConfig | null>(null);

  // Status
  const [saveStatus, setSaveStatus] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen && !isInline) return;

    async function loadData() {
      try {
        const [reg, settings, site] = await Promise.all([
          picoApi.listLlmProviders(),
          picoApi.getLlmSettings(),
          picoApi.getSiteConfig().catch(() => null),
        ]);
        setProvidersRegistry(reg);
        setLlmSettings(settings);
        setSelectedProviderId(settings.current_provider);

        const currentProv = settings.providers[settings.current_provider];
        if (currentProv) {
          setSelectedModel(currentProv.model || 'gemini-4-argon');
          setCustomModelInput(currentProv.custom_model || '');
          setCustomBaseUrlInput(currentProv.base_url || '');
          setSelectedThinking(currentProv.thinking || 'medium');
          setGroundingEnabled(Boolean(currentProv.grounding));
        }

        setGithubTokenInput(settings.github_token || '');
        if (site) setSiteConfig(site);
      } catch (err: any) {
        console.warn('Failed to load settings:', err);
      }
    }

    loadData();
  }, [isOpen]);

  // When changing provider dropdown
  const handleSelectProvider = (provId: string) => {
    setSelectedProviderId(provId);
    setTestResult(null);
    setApiKeyInput('');

    if (llmSettings?.providers[provId]) {
      const p = llmSettings.providers[provId];
      setSelectedModel(p.model || '');
      setCustomModelInput(p.custom_model || '');
      setCustomBaseUrlInput(p.base_url || '');
      setSelectedThinking(p.thinking || 'medium');
      setGroundingEnabled(Boolean(p.grounding));
    } else {
      const reg = providersRegistry.find((r) => r.id === provId);
      if (reg) {
        setSelectedModel(reg.default_models[0] || '');
        setCustomModelInput('');
        setCustomBaseUrlInput(reg.base_url || '');
        setSelectedThinking('medium');
        setGroundingEnabled(false);
      }
    }
  };

  const currentRegistryInfo = providersRegistry.find((r) => r.id === selectedProviderId);
  const currentProviderView = llmSettings?.providers[selectedProviderId];
  const rawModel = customModelInput.trim() || selectedModel;
  const effectiveModel = normalizeGoogleModelId(selectedProviderId, rawModel);
  const isGeminiNative = selectedProviderId === 'gemini-native';

  const handleTestConnection = async () => {
    setTestingConnection(true);
    setTestResult(null);
    setSaveError(null);
    try {
      const res = await picoApi.testLlmConnection(selectedProviderId, effectiveModel);
      setTestResult(res);
    } catch (err: any) {
      setTestResult({
        success: false,
        model: effectiveModel,
        elapsed_ms: 0,
        error: err.message || String(err),
      });
    } finally {
      setTestingConnection(false);
    }
  };

  const handleSaveAllSettings = async () => {
    setSaveStatus('Saving…');
    setSaveError(null);
    try {
      // Validate Custom Base URL if custom provider
      if (selectedProviderId === 'custom' && customBaseUrlInput.trim()) {
        const url = customBaseUrlInput.trim();
        const isLocal =
          url.startsWith('http://localhost') || url.startsWith('http://127.0.0.1');
        if (!url.startsWith('https://') && !isLocal) {
          throw new Error('Custom Base URL must use HTTPS (except for local http://localhost or http://127.0.0.1).');
        }
      }

      const input: SaveLlmSettingsInput = {
        current_provider: selectedProviderId,
        provider_updates: [
          {
            id: selectedProviderId,
            base_url: customBaseUrlInput.trim() || undefined,
            model: selectedModel,
            custom_model: customModelInput.trim(),
            thinking: selectedThinking,
            grounding: isGeminiNative ? groundingEnabled : false,
            api_key: apiKeyInput.trim() || undefined,
          },
        ],
        github_token: githubTokenInput.trim() || undefined,
      };

      await picoApi.saveLlmSettings(input);

      if (siteConfig) {
        await picoApi.saveSiteConfig(siteConfig);
      }

      setSaveStatus('Settings saved successfully!');
      setTimeout(() => setSaveStatus(null), 3000);
      onSettingsSaved?.();

      // Refresh loaded settings
      const updated = await picoApi.getLlmSettings();
      setLlmSettings(updated);
      setApiKeyInput('');
    } catch (err: any) {
      setSaveStatus(null);
      setSaveError(err.message || String(err));
    }
  };

  if (!isOpen && !isInline) return null;

  const content = (
    <div className={`bg-neutral-900 border border-neutral-800 rounded-2xl w-full flex flex-col shadow-2xl overflow-hidden text-neutral-100 ${isInline ? 'h-full max-h-none rounded-none border-none' : 'max-w-2xl max-h-[85vh]'}`}>
      {/* Modal Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-neutral-800 bg-neutral-900/90 shrink-0">
        <div className="flex items-center gap-2">
          <Sliders className="w-5 h-5 text-indigo-400" />
          <h2 className="text-base font-bold text-neutral-100">Settings</h2>
        </div>
        {!isInline && onClose && (
          <button
            type="button"
            onClick={onClose}
            className="p-1 rounded-lg hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        )}
      </div>

      {/* Tab Switcher */}
      <div className="flex border-b border-neutral-800 px-6 bg-neutral-950/60 text-xs font-medium shrink-0">
        <button
          type="button"
          onClick={() => setActiveTab('llm')}
          className={`py-3 px-4 border-b-2 -mb-px transition-colors cursor-pointer flex items-center gap-2 ${
            activeTab === 'llm'
              ? 'border-indigo-500 text-indigo-300 font-semibold'
              : 'border-transparent text-neutral-400 hover:text-neutral-200'
          }`}
        >
          <Key className="w-4 h-4" />
          <span>AI & LLM Providers</span>
        </button>
        <button
          type="button"
          onClick={() => setActiveTab('site')}
          className={`py-3 px-4 border-b-2 -mb-px transition-colors cursor-pointer flex items-center gap-2 ${
            activeTab === 'site'
              ? 'border-indigo-500 text-indigo-300 font-semibold'
              : 'border-transparent text-neutral-400 hover:text-neutral-200'
          }`}
        >
          <Globe className="w-4 h-4" />
          <span>Site Configuration (site.yaml)</span>
        </button>
      </div>

      {/* Modal Body */}
      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        {saveError && (
          <div className="p-3 bg-red-950/70 border border-red-800 rounded-lg text-xs text-red-200 flex items-start gap-2">
            <AlertCircle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
            <div>{saveError}</div>
          </div>
        )}

        {saveStatus && (
          <div className="p-3 bg-emerald-950/70 border border-emerald-800 rounded-lg text-xs text-emerald-200 flex items-center gap-2">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <div>{saveStatus}</div>
          </div>
        )}

        {activeTab === 'llm' && (
          <LlmSettingsTab
            selectedProviderId={selectedProviderId}
            onSelectProvider={handleSelectProvider}
            customBaseUrlInput={customBaseUrlInput}
            onCustomBaseUrlChange={setCustomBaseUrlInput}
            selectedModel={selectedModel}
            onSelectModel={setSelectedModel}
            customModelInput={customModelInput}
            onCustomModelChange={setCustomModelInput}
            currentRegistryInfo={currentRegistryInfo}
            selectedThinking={selectedThinking}
            onSelectThinking={setSelectedThinking}
            groundingEnabled={groundingEnabled}
            onToggleGrounding={setGroundingEnabled}
            apiKeyInput={apiKeyInput}
            onApiKeyChange={setApiKeyInput}
            providerKeyStatus={currentProviderView?.key_status}
            isTesting={testingConnection}
            testResult={testResult}
            onTestConnection={handleTestConnection}
            githubTokenInput={githubTokenInput}
            onGithubTokenChange={setGithubTokenInput}
          />
        )}

        {activeTab === 'site' && siteConfig && (
          <SiteSettingsTab
            siteConfig={siteConfig}
            onChange={setSiteConfig}
          />
        )}
      </div>

      {/* Modal Footer */}
      <div className="flex items-center justify-between px-6 py-4 border-t border-neutral-800 bg-neutral-950/80 shrink-0">
        {!isInline && onClose ? (
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 text-xs font-medium transition-colors cursor-pointer"
          >
            Cancel
          </button>
        ) : (
          <div />
        )}

        <button
          type="button"
          onClick={handleSaveAllSettings}
          className="px-5 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-colors cursor-pointer flex items-center gap-1.5 shadow-sm"
        >
          <Save className="w-4 h-4" /> Save Settings
        </button>
      </div>
    </div>
  );

  if (isInline) {
    return content;
  }

  return (
    <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-xs flex items-center justify-center p-4">
      {content}
    </div>
  );
}
