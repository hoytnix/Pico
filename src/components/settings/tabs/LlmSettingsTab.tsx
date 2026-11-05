import React from 'react';
import {
  Lock,
  ExternalLink,
  Play,
  RotateCw,
  CheckCircle2,
  AlertCircle,
  HelpCircle,
} from 'lucide-react';
import { MODEL_DISPLAY_LABELS } from '../../../lib/llmProviders';
import type { ProviderInfo, TestConnectionResult } from '../../../types';

interface LlmSettingsTabProps {
  selectedProviderId: string;
  onSelectProvider: (id: string) => void;
  customBaseUrlInput: string;
  onCustomBaseUrlChange: (val: string) => void;
  selectedModel: string;
  onSelectModel: (val: string) => void;
  customModelInput: string;
  onCustomModelChange: (val: string) => void;
  currentRegistryInfo?: ProviderInfo;
  selectedThinking: string;
  onSelectThinking: (val: string) => void;
  groundingEnabled: boolean;
  onToggleGrounding: (val: boolean) => void;
  apiKeyInput: string;
  onApiKeyChange: (val: string) => void;
  providerKeyStatus?: string;
  isTesting: boolean;
  testResult: TestConnectionResult | null;
  onTestConnection: () => void;
  githubTokenInput: string;
  onGithubTokenChange: (val: string) => void;
}

export function LlmSettingsTab({
  selectedProviderId,
  onSelectProvider,
  customBaseUrlInput,
  onCustomBaseUrlChange,
  selectedModel,
  onSelectModel,
  customModelInput,
  onCustomModelChange,
  currentRegistryInfo,
  selectedThinking,
  onSelectThinking,
  groundingEnabled,
  onToggleGrounding,
  apiKeyInput,
  onApiKeyChange,
  providerKeyStatus,
  isTesting,
  testResult,
  onTestConnection,
  githubTokenInput,
  onGithubTokenChange,
}: LlmSettingsTabProps) {
  return (
    <div className="space-y-5 text-xs">
      {/* Provider Selection */}
      <div>
        <label className="block text-xs font-semibold text-neutral-300 mb-1.5">
          LLM Provider
        </label>
        <select
          value={selectedProviderId}
          onChange={(e) => onSelectProvider(e.target.value)}
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
            onChange={(e) => onCustomBaseUrlChange(e.target.value)}
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
              onChange={(e) => onSelectModel(e.target.value)}
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
            Custom Model ID (Overrides preset)
          </label>
          <input
            type="text"
            value={customModelInput}
            onChange={(e) => onCustomModelChange(e.target.value)}
            placeholder="e.g. gpt-4o, claude-3-5-sonnet, llama-3"
            className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
          />
        </div>
      </div>

      {/* Thinking / Reasoning Controls */}
      <div>
        <div className="flex items-center justify-between mb-1.5">
          <label className="block text-xs font-semibold text-neutral-300">
            Thinking / Reasoning Effort
          </label>
          {selectedProviderId === 'meta' && (
            <span className="text-[11px] text-amber-400 italic">
              Thinking is not supported on Muse Spark
            </span>
          )}
        </div>
        <div className="grid grid-cols-4 gap-2">
          {(['default', 'low', 'medium', 'high'] as const).map((lvl) => (
            <button
              key={lvl}
              type="button"
              disabled={selectedProviderId === 'meta'}
              onClick={() => onSelectThinking(lvl)}
              className={`p-2 rounded-lg border text-xs font-medium capitalize transition-all cursor-pointer ${
                selectedThinking === lvl
                  ? 'bg-indigo-950 border-indigo-500 text-indigo-300'
                  : 'bg-neutral-950 border-neutral-800 text-neutral-400 hover:border-neutral-700'
              } ${selectedProviderId === 'meta' ? 'opacity-40 cursor-not-allowed' : ''}`}
            >
              {lvl}
            </button>
          ))}
        </div>
      </div>

      {/* Google Search Grounding toggle */}
      <div className="p-3 bg-neutral-950/70 border border-neutral-800 rounded-lg flex items-center justify-between">
        <div>
          <div className="font-semibold text-neutral-200">Google Search Grounding</div>
          <div className="text-[11px] text-neutral-400">
            {selectedProviderId === 'gemini-native'
              ? 'Enables real-time Google Search retrieval for facts and citations in Vibe.'
              : 'Grounding via Google Search is supported only on Gemini Native.'}
          </div>
        </div>
        <label className="relative inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            checked={groundingEnabled}
            disabled={selectedProviderId !== 'gemini-native'}
            onChange={(e) => onToggleGrounding(e.target.checked)}
            className="sr-only peer"
          />
          <div
            className={`w-9 h-5 bg-neutral-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all ${
              selectedProviderId === 'gemini-native'
                ? 'peer-checked:bg-indigo-600'
                : 'opacity-40 cursor-not-allowed'
            }`}
          />
        </label>
      </div>

      {/* API Key Input */}
      <div>
        <div className="flex items-center justify-between mb-1.5">
          <label className="block text-xs font-semibold text-neutral-300">
            API Key
          </label>
          {providerKeyStatus === 'SET' && (
            <span className="text-[11px] text-emerald-400 flex items-center gap-1 font-mono">
              <Lock className="w-3 h-3" /> Key Stored Securely
            </span>
          )}
        </div>
        <input
          type="password"
          value={apiKeyInput}
          onChange={(e) => onApiKeyChange(e.target.value)}
          placeholder={providerKeyStatus === 'SET' ? '•••••••••••••••• (Leave blank to keep current key)' : 'Enter your API key...'}
          className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
        />
        <p className="mt-1 text-[11px] text-neutral-500">
          Keys are stored locally on your machine in <code className="text-neutral-400">~/.config/pico/settings.json</code> and never transmitted to our servers.
        </p>
      </div>

      {/* Test Connection Button & Result Box */}
      <div className="space-y-3 pt-2 border-t border-neutral-800">
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={onTestConnection}
            disabled={isTesting}
            className="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 text-neutral-200 rounded-lg text-xs font-medium flex items-center gap-2 transition-colors disabled:opacity-50 cursor-pointer"
          >
            {isTesting ? (
              <RotateCw className="w-3.5 h-3.5 animate-spin text-indigo-400" />
            ) : (
              <Play className="w-3.5 h-3.5 text-indigo-400" />
            )}
            <span>Test Connection</span>
          </button>
          <span className="text-[11px] text-neutral-500">
            Sends a minimal test prompt to verify credentials and endpoint reachability.
          </span>
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
          onChange={(e) => onGithubTokenChange(e.target.value)}
          placeholder="ghp_••••••••••••••••"
          className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-2.5 text-xs font-mono text-neutral-100 outline-none focus:border-indigo-500"
        />
      </div>
    </div>
  );
}
