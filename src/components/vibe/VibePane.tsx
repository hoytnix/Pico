import React, { useState, useRef, useEffect } from 'react';
import { AlertCircle } from 'lucide-react';
import { picoApi } from '../../api';
import { picoLog } from '../../lib/logger';
import {
  createVibeAgent,
  formatLlmError,
  type ProposeEditResult,
} from '../../lib/vibeAgent';
import { applyAcceptedProposal } from '../../lib/vibe/proposalApplier';
import { VibeHeader } from './VibeHeader';
import { VibeComposer } from './VibeComposer';
import { MessageList } from './MessageList';
import type { Message, ProposalItem } from './types';
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
  activePostPath,
  onPostUpdatedFromVibe,
  onTemplateUpdatedFromVibe,
  onThemeUpdatedFromVibe,
  onModelUpdatedFromVibe,
  onOpenSettings,
}: VibePaneProps) {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: 'welcome',
      role: 'assistant',
      content:
        'Hello! I am Vibe, your AI web designer and content editor. You can ask me to write or edit blog posts, modify MiniJinja templates, update collection models, or adjust theme colors. Any changes I suggest will be shown as a diff for you to review and Accept or Reject.',
    },
  ]);
  const [inputPrompt, setInputPrompt] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [activeError, setActiveError] = useState<string | null>(null);
  const [processingProposalIds, setProcessingProposalIds] = useState<Record<string, boolean>>({});

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  const currentProviderId = llmSettings?.current_provider || 'gemini-native';
  const providerView = llmSettings?.providers[currentProviderId];
  const hasKey = providerView?.key_status === 'SET';
  const effectiveModel = providerView?.effective_model || 'gemini-4-argon';

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, isStreaming]);

  const handleSendMessage = async (textToSend?: string) => {
    const prompt = (textToSend || inputPrompt).trim();
    if (!prompt || isStreaming) return;

    if (!hasKey) {
      setActiveError('Configure an LLM API key in Settings to use Vibe.');
      return;
    }

    setActiveError(null);
    setInputPrompt('');

    const userMsgId = `user-${Date.now()}`;
    const assistantMsgId = `assistant-${Date.now()}`;

    const newMessages: Message[] = [
      ...messages,
      { id: userMsgId, role: 'user', content: prompt },
      { id: assistantMsgId, role: 'assistant', content: '' },
    ];
    setMessages(newMessages);
    setIsStreaming(true);

    let apiKey = '';
    try {
      apiKey = await picoApi.getLlmKey(currentProviderId);
    } catch {
      setActiveError(`Invalid API key for ${currentProviderId}. Check the key in Settings.`);
      setIsStreaming(false);
      return;
    }

    if (!apiKey.trim()) {
      setActiveError('Configure an LLM API key in Settings to use Vibe.');
      setIsStreaming(false);
      return;
    }

    const abortController = new AbortController();
    abortControllerRef.current = abortController;

    try {
      const turnProposals: ProposalItem[] = [];

      const config = {
        provider: currentProviderId,
        model: effectiveModel,
        apiKey: apiKey.trim(),
        baseUrl: providerView?.base_url,
        thinking: providerView?.thinking,
        grounding: providerView?.grounding,
      };

      let streamError: unknown = null;

      const agent = createVibeAgent(
        config,
        (proposal) => {
          const propId =
            proposal.id ||
            `prop-${Date.now()}-${turnProposals.length + 1}-${Math.random().toString(36).slice(2, 6)}`;
          const proposalItem: ProposalItem = {
            ...proposal,
            id: propId,
            status: 'pending',
          };
          turnProposals.push(proposalItem);

          setMessages((prev) =>
            prev.map((m) => {
              if (m.id !== assistantMsgId) return m;
              const existingProposals = m.proposals || [];
              return {
                ...m,
                proposals: [...existingProposals, proposalItem],
                proposal,
                proposalStatus: 'pending',
              };
            })
          );
        },
        (err) => {
          streamError = err;
        }
      );

      // Build model messages from history (multi-turn)
      const agentPrompt = newMessages
        .filter((m) => m.id !== assistantMsgId)
        .map((m) => ({
          role: m.role,
          content: m.content,
        }));

      picoLog.info('vibe', `Streaming Vibe response: provider=${currentProviderId}, model=${effectiveModel}`);
      const streamResult = await agent.stream({
        prompt: agentPrompt,
        abortSignal: abortController.signal,
      });

      let accumulatedText = '';
      for await (const chunk of streamResult.textStream) {
        accumulatedText += chunk;
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantMsgId
              ? {
                  ...m,
                  content: accumulatedText,
                }
              : m
          )
        );
      }

      if (streamError) {
        const formatted = formatLlmError(streamError, currentProviderId, effectiveModel);
        setActiveError(formatted);
        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantMsgId
              ? {
                  ...m,
                  error: formatted,
                  content: accumulatedText || 'An error occurred while contacting the AI model. Check the error message below.',
                }
              : m
          )
        );
      } else if (!accumulatedText.trim()) {
        const count = turnProposals.length;
        let fallbackText = '';
        if (count === 1) {
          const first = turnProposals[0];
          fallbackText =
            first.explanation ||
            `I have prepared a proposed edit for ${first.path}. Please review the diff below.`;
        } else if (count > 1) {
          fallbackText = `I have prepared proposed edits for ${count} files. Please review each diff below sequentially.`;
        } else {
          fallbackText =
            'I was unable to propose any changes for this request. Please try rephrasing or inspect the file directly.';
        }

        setMessages((prev) =>
          prev.map((m) =>
            m.id === assistantMsgId
              ? {
                  ...m,
                  content: fallbackText,
                }
              : m
          )
        );
      }

      setIsStreaming(false);
      abortControllerRef.current = null;
    } catch (err: any) {
      if (abortController.signal.aborted) {
        setIsStreaming(false);
        return;
      }
      const formatted = formatLlmError(err, currentProviderId, effectiveModel);
      setActiveError(formatted);
      setMessages((prev) =>
        prev.map((m) =>
          m.id === assistantMsgId
            ? {
                ...m,
                error: formatted,
              }
            : m
        )
      );
      setIsStreaming(false);
      abortControllerRef.current = null;
    }
  };

  const handleStopStreaming = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
    setIsStreaming(false);
  };

  const handleAcceptProposal = async (
    msgId: string,
    propId: string,
    proposal: ProposeEditResult
  ) => {
    setProcessingProposalIds((prev) => ({ ...prev, [propId]: true }));
    try {
      await applyAcceptedProposal(proposal, {
        onPostUpdatedFromVibe,
        onTemplateUpdatedFromVibe,
        onThemeUpdatedFromVibe,
        onModelUpdatedFromVibe,
      });

      setMessages((prev) =>
        prev.map((m) => {
          if (m.id !== msgId) return m;
          const updatedProposals = m.proposals?.map((p) =>
            p.id === propId ? { ...p, status: 'accepted' as const } : p
          );
          const hasPending = updatedProposals?.some((p) => p.status === 'pending');
          return {
            ...m,
            proposals: updatedProposals,
            proposalStatus: hasPending ? 'pending' : 'accepted',
          };
        })
      );
    } catch (err: any) {
      setActiveError(`Failed to save accepted edit: ${err.message || String(err)}`);
    } finally {
      setProcessingProposalIds((prev) => ({ ...prev, [propId]: false }));
    }
  };

  const handleRejectProposal = (msgId: string, propId: string) => {
    setMessages((prev) =>
      prev.map((m) => {
        if (m.id !== msgId) return m;
        const updatedProposals = m.proposals?.map((p) =>
          p.id === propId ? { ...p, status: 'rejected' as const } : p
        );
        const hasPending = updatedProposals?.some((p) => p.status === 'pending');
        return {
          ...m,
          proposals: updatedProposals,
          proposalStatus: hasPending ? 'pending' : 'rejected',
        };
      })
    );
  };

  const handleAcceptAll = async (msgId: string, proposals: ProposalItem[]) => {
    const pendingToAccept = proposals.filter((p) => p.status === 'pending' && !p.validationError);
    if (pendingToAccept.length === 0) return;

    for (const prop of pendingToAccept) {
      await handleAcceptProposal(msgId, prop.id, prop);
    }
  };

  const handleRejectAll = (msgId: string, proposals: ProposalItem[]) => {
    const pendingToReject = proposals.filter((p) => p.status === 'pending');
    for (const prop of pendingToReject) {
      handleRejectProposal(msgId, prop.id);
    }
  };

  const handleClearHistory = () => {
    setMessages([
      {
        id: 'welcome',
        role: 'assistant',
        content:
          'Chat history reset. How can I assist you with your content, templates, or theme?',
      },
    ]);
    setActiveError(null);
  };

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
