import React from 'react';
import { AlertCircle } from 'lucide-react';
import { VibeHeader } from './VibeHeader';
import { VibeComposer } from './VibeComposer';
import { MessageList } from './MessageList';
import { useVibeChat } from './useVibeChat';
import type { ProposalItem } from './types';
import type { LlmSettingsView, SiteInfo } from '../../types';

export type { ProposalItem };

interface VibePaneProps {
  currentSite: SiteInfo | null;
  llmSettings: LlmSettingsView | null;
  activePostPath: string | null;
  onPostUpdatedFromVibe?: (path: string, newMarkdown: string) => void;
  onTemplateUpdatedFromVibe?: (path: string, newContent: string) => void;
  onThemeUpdatedFromVibe?: (newContent: string) => void;
  onModelUpdatedFromVibe?: (path: string) => void;
  onOpenSettings?: () => void;
}

export function VibePane({
  currentSite,
  llmSettings,
  onPostUpdatedFromVibe,
  onTemplateUpdatedFromVibe,
  onThemeUpdatedFromVibe,
  onModelUpdatedFromVibe,
  onOpenSettings,
}: VibePaneProps) {
  const {
    messages,
    inputPrompt,
    setInputPrompt,
    isStreaming,
    activeError,
    processingProposalIds,
    currentProviderId,
    effectiveModel,
    hasKey,
    messagesEndRef,
    handleSendMessage,
    handleStopStreaming,
    handleAcceptProposal,
    handleRejectProposal,
    handleAcceptAll,
    handleRejectAll,
    handleClearHistory,
  } = useVibeChat({
    currentSite,
    llmSettings,
    onPostUpdatedFromVibe,
    onTemplateUpdatedFromVibe,
    onThemeUpdatedFromVibe,
    onModelUpdatedFromVibe,
  });

  return (
    <div className="vibe-pane flex flex-col h-full bg-neutral-950 text-neutral-100 select-text">
      {/* Top Header */}
      <VibeHeader
        providerId={currentProviderId}
        model={effectiveModel}
        onClearHistory={handleClearHistory}
      />

      {/* Global Error Notice */}
      {activeError && (
        <div className="mx-6 mt-3 px-4 py-2.5 bg-red-950/80 border border-red-800 rounded-lg text-xs text-red-200 flex items-start justify-between gap-3 shadow-md">
          <div className="flex items-start gap-2">
            <AlertCircle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
            <div>{activeError}</div>
          </div>
          {activeError.includes('Settings') && onOpenSettings && (
            <button
              type="button"
              onClick={onOpenSettings}
              className="underline text-indigo-300 hover:text-indigo-200 font-medium shrink-0 cursor-pointer"
            >
              Open Settings
            </button>
          )}
        </div>
      )}

      {/* Conversation Message List */}
      <MessageList
        messages={messages}
        isStreaming={isStreaming}
        processingProposalIds={processingProposalIds}
        onAcceptProposal={handleAcceptProposal}
        onRejectProposal={handleRejectProposal}
        onAcceptAll={handleAcceptAll}
        onRejectAll={handleRejectAll}
        messagesEndRef={messagesEndRef}
      />

      {/* Composer Bar */}
      <VibeComposer
        inputPrompt={inputPrompt}
        setInputPrompt={setInputPrompt}
        hasKey={hasKey}
        isStreaming={isStreaming}
        onSend={handleSendMessage}
        onStopStreaming={handleStopStreaming}
      />
    </div>
  );
}
