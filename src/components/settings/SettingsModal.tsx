import React, { useState, useEffect } from 'react';
import {
  X,
  Key,
  Globe,
  Sliders,
  CheckCircle2,
  AlertCircle,
  HelpCircle,
  Lock,
  ExternalLink,
  Save,
  Play,
  RotateCw,
} from 'lucide-react';
import { picoApi } from '../../api';
import { normalizeGoogleModelId } from '../../lib/vibeAgent';
import type {
  LlmSettingsView,
  ProviderInfo,
  ProviderView,
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

const MODEL_DISPLAY_LABELS: Record<string, string> = {
  'gemini-4-argon': 'gemini-4-argon (Gemini 4 Argon)',
  'gemini-3.8-flash': 'gemini-3.8-flash (Gemini 3.8 Flash)',
  'gemini-3.6-flash': 'gemini-3.6-flash (Gemini 3.6 Flash)',
  'gemini-3.5-flash': 'gemini-3.5-flash (Gemini 3.5 Flash)',
  'gemini-3.5-flash-lite': 'gemini-3.5-flash-lite (Gemini 3.5 Flash-Lite)',
  'gemini-3.1-flash-lite': 'gemini-3.1-flash-lite (Gemini 3.1 Flash-Lite)',
  'gemma-4-31b-it': 'gemma-4-31b-it (Gemma 4 31B)',
  'gemma-4-26b-a4b-it': 'gemma-4-26b-a4b-it (Gemma 4 26B)',
};

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
  const isMuseSpark = selectedProviderId === 'meta';

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
            <div className="space-y-5 text-xs">
              {/* Provider Selection (§6) */}
              <div>
                <label className="block text-xs font-semibold text-neutral-300 mb-1.5">
                  LLM Provider
                </label>
                <select
                  value={selectedProviderId}
                  onChange={(e) => handleSelectProvider(e.target.value)}
                  className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs text-neutral-100 outline-none focus:border-indigo-500 [&>option]:bg-neutral-950 [&>option]:text-neutral-100"
                >
                  <option value="gemini-native">Gemini (Native) — @ai-sdk/google</option>
                  <option value="openai">OpenAI — AI SDK OpenAI provider</option>
                  <option value="openrouter">OpenRouter — OpenAI-compatible provider</option>
                  <option value="meta">Muse Spark — AI SDK OpenAI-compatible provider</option>
                  <option value="gemini-openai">Gemini (OpenAI mode) — OpenAI-compatible endpoint</option>
                  <option value="custom">Custom (OpenAI-compatible) — Local or custom HTTPS endpoint</option>
                </select>
              </div>

              {/* Custom Base URL (if custom provider) */}
              {selectedProviderId === 'custom' && (
                <div>
                  <label className="block text-xs font-semibold text-neutral-300 mb-1.5">
                    Custom Base URL (HTTPS or local http://localhost)
                  </label>
                  <input
                    type="text"
                    value={customBaseUrlInput}
                    onChange={(e) => setCustomBaseUrlInput(e.target.value)}
                    placeholder="https://your-api.com/v1 or http://127.0.0.1:11434/v1"
                    className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
                  />
                </div>
              )}

              {/* Model Dropdown + Custom Model field */}
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-neutral-300 mb-1.5">
                    Preset Model
                  </label>
                  {currentRegistryInfo && currentRegistryInfo.default_models.length > 0 ? (
                    <select
                      value={selectedModel}
                      onChange={(e) => setSelectedModel(e.target.value)}
                      className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs text-neutral-100 outline-none focus:border-indigo-500 font-mono [&>option]:bg-neutral-950 [&>option]:text-neutral-100"
                    >
                      {currentRegistryInfo.default_models.map((m) => (
                        <option key={m} value={m}>
                          {MODEL_DISPLAY_LABELS[m] || m}
                        </option>
                      ))}
                    </select>
                  ) : (
                    <div className="p-2.5 bg-neutral-950 border border-neutral-800 rounded-lg text-neutral-500 italic">
                      No presets (custom model required)
                    </div>
                  )}
                </div>

                <div>
                  <label className="block text-xs font-semibold text-neutral-300 mb-1.5">
                    Custom Model (Overrides preset)
                  </label>
                  <input
                    type="text"
                    value={customModelInput}
                    onChange={(e) => setCustomModelInput(e.target.value)}
                    placeholder="e.g. meta-llama/llama-3.3-70b"
                    className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
                  />
                </div>
              </div>

              {/* Effective Model Plaintext indicator */}
              <div className="px-3 py-2 bg-neutral-950/80 rounded-lg border border-neutral-800 flex items-center justify-between">
                <span className="text-neutral-400">Effective Model ID:</span>
                <span className="font-mono text-indigo-300 font-semibold">{effectiveModel}</span>
              </div>

              {/* API Key Input & Status */}
              <div>
                <div className="flex items-center justify-between mb-1.5">
                  <label className="text-xs font-semibold text-neutral-300">
                    API Key for {currentRegistryInfo?.label || selectedProviderId}
                  </label>
                  <span
                    className={`text-[10px] px-2 py-0.5 rounded font-mono font-medium ${
                      currentProviderView?.key_status === 'SET'
                        ? 'bg-emerald-950 text-emerald-400 border border-emerald-800'
                        : 'bg-red-950 text-red-400 border border-red-800'
                    }`}
                  >
                    {currentProviderView?.key_status === 'SET' ? 'KEY CONFIGURED (SET)' : 'NOT SET'}
                  </span>
                </div>
                <input
                  type="password"
                  value={apiKeyInput}
                  onChange={(e) => setApiKeyInput(e.target.value)}
                  placeholder={
                    currentProviderView?.key_status === 'SET'
                      ? '•••••••••••••••• (Enter new key to replace)'
                      : 'Enter API key here…'
                  }
                  className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
                />
                <div className="mt-1 text-[11px] text-neutral-500">
                  Stored securely in <code>~/.config/pico/settings.json</code>. Never bundled or logged.
                </div>
              </div>

              {/* Thinking Dropdown (§7) */}
              <div>
                <label className="block text-xs font-semibold text-neutral-300 mb-1.5">
                  Thinking / Reasoning Effort
                </label>
                <select
                  value={selectedThinking}
                  onChange={(e) => setSelectedThinking(e.target.value)}
                  disabled={isMuseSpark}
                  className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs text-neutral-100 outline-none focus:border-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed [&>option]:bg-neutral-950 [&>option]:text-neutral-100"
                >
                  <option value="default">Provider Default (send nothing)</option>
                  <option value="low">Low</option>
                  <option value="medium">Medium</option>
                  <option value="high">High</option>
                </select>
                <div className="mt-1 text-[11px] text-neutral-500">
                  {isMuseSpark ? (
                    <span className="text-amber-400 font-medium">
                      Muse Spark manages its own reasoning.
                    </span>
                  ) : (
                    <span>Higher thinking may increase cost and time to first token.</span>
                  )}
                </div>
              </div>

              {/* Grounding Toggle (§7) */}
              <div>
                <div className="flex items-center justify-between p-3 bg-neutral-950/80 rounded-lg border border-neutral-800">
                  <div>
                    <div className="text-xs font-semibold text-neutral-200">
                      Google Search Grounding
                    </div>
                    <div className="text-[11px] text-neutral-400 mt-0.5">
                      Ground AI edits in live Google Search results with real-time sources.
                    </div>
                  </div>
                  <input
                    type="checkbox"
                    checked={groundingEnabled && isGeminiNative}
                    onChange={(e) => setGroundingEnabled(e.target.checked)}
                    disabled={!isGeminiNative}
                    className="w-4 h-4 rounded bg-neutral-800 border-neutral-700 text-indigo-600 focus:ring-0 disabled:opacity-40 cursor-pointer disabled:cursor-not-allowed"
                  />
                </div>
                {!isGeminiNative && (
                  <div className="mt-1 text-[11px] text-amber-400 font-medium">
                    Grounding requires Gemini Native, not an OpenAI provider.
                  </div>
                )}
              </div>

              {/* Test Connection Button (§6, §10) */}
              <div className="p-4 bg-neutral-950 rounded-xl border border-neutral-800 space-y-3">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-xs font-semibold text-neutral-200">Test Connection</div>
                    <div className="text-[11px] text-neutral-500">
                      Sends a minimal test prompt ("OK") to verify provider credentials and endpoint.
                    </div>
                  </div>
                  <button
                    type="button"
                    onClick={handleTestConnection}
                    disabled={testingConnection}
                    className="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-200 border border-neutral-700 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5 disabled:opacity-50"
                  >
                    {testingConnection ? (
                      <>
                        <RotateCw className="w-3.5 h-3.5 animate-spin" /> Testing…
                      </>
                    ) : (
                      <>
                        <Play className="w-3.5 h-3.5 text-emerald-400" /> Test Connection
                      </>
                    )}
                  </button>
                </div>

                <div className="text-[10px] text-neutral-500 italic">
                  Note: A test call may incur a small provider charge.
                </div>

                {testResult && (
                  <div
                    className={`p-3 rounded-lg text-xs border ${
                      testResult.success
                        ? 'bg-emerald-950/40 border-emerald-800 text-emerald-300'
                        : 'bg-red-950/40 border-red-800 text-red-300'
                    }`}
                  >
                    {testResult.success ? (
                      <div className="flex items-center gap-2">
                        <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                        <span>
                          Success! Model: <strong>{testResult.model}</strong> ({testResult.elapsed_ms}ms)
                        </span>
                      </div>
                    ) : (
                      <div className="flex items-start gap-2">
                        <AlertCircle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
                        <div>
                          <strong>Connection Test Failed:</strong>
                          <p className="mt-0.5">{testResult.error}</p>
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>

              {/* GitHub Token */}
              <div>
                <label className="block text-xs font-semibold text-neutral-300 mb-1">
                  GitHub Personal Access Token (Optional, for GitHub Pages publishing)
                </label>
                <input
                  type="password"
                  value={githubTokenInput}
                  onChange={(e) => setGithubTokenInput(e.target.value)}
                  placeholder="ghp_••••••••••••••••"
                  className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
                />
              </div>
            </div>
          )}

          {activeTab === 'site' && siteConfig && (
            <div className="space-y-4 text-xs">
              <div>
                <label className="block text-xs font-semibold text-neutral-300 mb-1">
                  Site Title
                </label>
                <input
                  type="text"
                  value={siteConfig.title || ''}
                  onChange={(e) => setSiteConfig({ ...siteConfig, title: e.target.value })}
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
                  onChange={(e) => setSiteConfig({ ...siteConfig, base_url: e.target.value })}
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
                  onChange={(e) => setSiteConfig({ ...siteConfig, description: e.target.value })}
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
                  onChange={(e) => setSiteConfig({ ...siteConfig, author: e.target.value })}
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
                  onChange={(e) => setSiteConfig({ ...siteConfig, comments_repo: e.target.value })}
                  className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
                />
              </div>
            </div>
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
