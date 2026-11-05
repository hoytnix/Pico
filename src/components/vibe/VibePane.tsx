import React, { useState, useRef, useEffect } from 'react';
import {
  Send,
  Square,
  Sparkles,
  Bot,
  User,
  AlertCircle,
  RotateCcw,
  Trash2,
  Cpu,
} from 'lucide-react';
import { picoApi } from '../../api';
import { picoLog } from '../../lib/logger';
import {
  createVibeAgent,
  buildProviderOptions,
  formatLlmError,
  type ProposeEditResult,
} from '../../lib/vibeAgent';
import { DiffCard } from './DiffCard';
import type { LlmSettingsView, SiteInfo } from '../../types';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  proposal?: ProposeEditResult;
  proposalStatus?: 'pending' | 'accepted' | 'rejected';
  reasoning?: string;
  error?: string;
}

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
      let pendingProposal: ProposeEditResult | undefined = undefined;

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
          pendingProposal = proposal;
          setMessages((prev) =>
            prev.map((m) =>
              m.id === assistantMsgId
                ? {
                    ...m,
                    proposal,
                    proposalStatus: 'pending',
                  }
                : m
            )
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

      // Stream assistant response
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
        const capturedProposal = pendingProposal as ProposeEditResult | undefined;
        const fallbackText = capturedProposal
          ? capturedProposal.explanation || `I have prepared a proposed edit for ${capturedProposal.path}. Please review the diff below.`
          : 'I was unable to propose any changes for this request. Please try rephrasing or inspect the file directly.';

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

  const handleAcceptProposal = async (msgId: string, proposal: ProposeEditResult) => {
    try {
      if (proposal.targetType === 'content') {
        const doc = await picoApi.readPost(proposal.path);
        await picoApi.savePost(proposal.path, doc.frontmatter, proposal.proposed);
        if (onPostUpdatedFromVibe) {
          onPostUpdatedFromVibe(proposal.path, proposal.proposed);
        }
      } else if (proposal.targetType === 'template') {
        await picoApi.saveTemplate(proposal.path, proposal.proposed);
        if (onTemplateUpdatedFromVibe) {
          onTemplateUpdatedFromVibe(proposal.path, proposal.proposed);
        }
      } else if (proposal.targetType === 'theme') {
        await picoApi.saveTheme(proposal.proposed);
        if (onThemeUpdatedFromVibe) {
          onThemeUpdatedFromVibe(proposal.proposed);
        }
      } else if (proposal.targetType === 'model') {
        await picoApi.saveModelYaml(proposal.path, proposal.proposed);
        if (onModelUpdatedFromVibe) {
          onModelUpdatedFromVibe(proposal.path);
        }
      }

      setMessages((prev) =>
        prev.map((m) =>
          m.id === msgId
            ? {
                ...m,
                proposalStatus: 'accepted',
              }
            : m
        )
      );
    } catch (err: any) {
      setActiveError(`Failed to save accepted edit: ${err.message || String(err)}`);
    }
  };

  const handleRejectProposal = (msgId: string) => {
    setMessages((prev) =>
      prev.map((m) =>
        m.id === msgId
          ? {
              ...m,
              proposalStatus: 'rejected',
            }
          : m
      )
    );
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
      <div className="flex items-center justify-between px-6 py-3 bg-neutral-900 border-b border-neutral-800">
        <div className="flex items-center gap-2.5">
          <div className="p-1.5 rounded-lg bg-indigo-600/20 text-indigo-400 border border-indigo-500/30">
            <Sparkles className="w-4 h-4" />
          </div>
          <div>
            <div className="text-sm font-semibold text-neutral-100 flex items-center gap-2">
              <span>Vibe AI Chat</span>
              <span className="text-[11px] font-mono font-normal px-2 py-0.5 rounded-full bg-neutral-800 text-neutral-300 border border-neutral-700">
                {currentProviderId} / {effectiveModel}
              </span>
            </div>
            <div className="text-xs text-neutral-400">
              Desktop AI pair programmer for Markdown, MiniJinja templates & theme CSS
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handleClearHistory}
            className="p-1.5 rounded hover:bg-neutral-800 text-neutral-400 hover:text-neutral-200 transition-colors cursor-pointer text-xs flex items-center gap-1"
            title="Clear conversation"
          >
            <Trash2 className="w-3.5 h-3.5" /> Clear
          </button>
        </div>
      </div>

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
      <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex gap-3 max-w-4xl ${
              msg.role === 'user' ? 'ml-auto flex-row-reverse' : 'mr-auto'
            }`}
          >
            {/* Avatar */}
            <div
              className={`w-7 h-7 rounded-full flex items-center justify-center shrink-0 text-xs ${
                msg.role === 'user'
                  ? 'bg-indigo-600 text-white'
                  : 'bg-neutral-800 text-indigo-400 border border-neutral-700'
              }`}
            >
              {msg.role === 'user' ? <User className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
            </div>

            {/* Bubble */}
            <div
              className={`rounded-2xl px-4 py-3 text-sm leading-relaxed max-w-2xl shadow-sm ${
                msg.role === 'user'
                  ? 'bg-indigo-600 text-white rounded-tr-none'
                  : 'bg-neutral-900 border border-neutral-800 text-neutral-200 rounded-tl-none'
              }`}
            >
              {msg.content ? (
                <div className="whitespace-pre-wrap break-words">{msg.content}</div>
              ) : isStreaming && msg.role === 'assistant' && !msg.proposal ? (
                <div className="flex items-center gap-1.5 text-xs text-neutral-400 py-1">
                  <span className="w-2 h-2 rounded-full bg-indigo-400 animate-pulse" />
                  <span>Thinking & drafting response...</span>
                </div>
              ) : null}

              {/* Proposed Edit Diff Card */}
              {msg.proposal && (
                <DiffCard
                  targetType={msg.proposal.targetType}
                  targetPath={msg.proposal.path}
                  diff={msg.proposal.diff}
                  explanation={msg.proposal.explanation}
                  validationError={msg.proposal.validationError}
                  sources={msg.proposal.sources}
                  status={msg.proposalStatus || 'pending'}
                  onAccept={() => handleAcceptProposal(msg.id, msg.proposal!)}
                  onReject={() => handleRejectProposal(msg.id)}
                />
              )}

              {msg.error && (
                <div className="mt-2 text-xs text-red-400 border-t border-neutral-800 pt-1.5">
                  {msg.error}
                </div>
              )}
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Composer Bar */}
      <div className="px-6 py-4 bg-neutral-900 border-t border-neutral-800">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            handleSendMessage();
          }}
          className="relative flex items-center bg-neutral-950 border border-neutral-700 rounded-xl overflow-hidden focus-within:border-indigo-500 transition-colors shadow-inner"
        >
          <textarea
            value={inputPrompt}
            onChange={(e) => setInputPrompt(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                handleSendMessage();
              }
            }}
            disabled={!hasKey || isStreaming}
            placeholder={
              !hasKey
                ? 'Configure an LLM API key in Settings to use Vibe.'
                : 'Ask Vibe to edit content, customize templates, or refine styles...'
            }
            rows={2}
            className="w-full bg-transparent px-4 py-3 text-sm text-neutral-100 placeholder-neutral-500 resize-none outline-none disabled:cursor-not-allowed disabled:opacity-60"
          />

          <div className="flex items-center gap-2 pr-3">
            {isStreaming ? (
              <button
                type="button"
                onClick={handleStopStreaming}
                className="p-2 rounded-lg bg-red-600 hover:bg-red-500 text-white transition-colors cursor-pointer"
                title="Stop generation"
              >
                <Square className="w-4 h-4 fill-current" />
              </button>
            ) : (
              <button
                type="submit"
                disabled={!hasKey || !inputPrompt.trim()}
                className="p-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-40 disabled:cursor-not-allowed transition-colors cursor-pointer"
                title="Send instruction"
              >
                <Send className="w-4 h-4" />
              </button>
            )}
          </div>
        </form>

        <div className="flex items-center justify-between mt-2 text-[11px] text-neutral-500">
          <span>
            Vibe proposals never apply automatically. You review diffs and explicitly Accept or
            Reject.
          </span>
          <span>Shift+Enter for newline</span>
        </div>
      </div>
    </div>
  );
}
