import { useState, useRef, useEffect } from 'react';
import { picoApi } from '../../api';
import { picoLog } from '../../lib/logger';
import {
  createVibeAgent,
  formatLlmError,
  type ProposeEditResult,
} from '../../lib/vibeAgent';
import { applyAcceptedProposal } from '../../lib/vibe/proposalApplier';
import type { Message, ProposalItem } from './types';
import type { LlmSettingsView, SiteInfo } from '../../types';

interface UseVibeChatOptions {
  currentSite: SiteInfo | null;
  llmSettings: LlmSettingsView | null;
  onPostUpdatedFromVibe?: (path: string, newMarkdown: string) => void;
  onTemplateUpdatedFromVibe?: (path: string, newContent: string) => void;
  onThemeUpdatedFromVibe?: (newContent: string) => void;
  onModelUpdatedFromVibe?: (path: string) => void;
}

export function useVibeChat({
  currentSite,
  llmSettings,
  onPostUpdatedFromVibe,
  onTemplateUpdatedFromVibe,
  onThemeUpdatedFromVibe,
  onModelUpdatedFromVibe,
}: UseVibeChatOptions) {
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

  return {
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
  };
}
