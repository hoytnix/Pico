import React from 'react';
import { Bot, User } from 'lucide-react';
import { DiffCard } from './DiffCard';
import type { Message, ProposalItem } from './types';
import type { ProposeEditResult } from '../../lib/vibeAgent';

interface MessageListProps {
  messages: Message[];
  isStreaming: boolean;
  processingProposalIds: Record<string, boolean>;
  onAcceptProposal: (msgId: string, propId: string, proposal: ProposeEditResult) => void;
  onRejectProposal: (msgId: string, propId: string) => void;
  onAcceptAll: (msgId: string, proposals: ProposalItem[]) => void;
  onRejectAll: (msgId: string, proposals: ProposalItem[]) => void;
  messagesEndRef: React.RefObject<HTMLDivElement | null>;
}

export function MessageList({
  messages,
  isStreaming,
  processingProposalIds,
  onAcceptProposal,
  onRejectProposal,
  onAcceptAll,
  onRejectAll,
  messagesEndRef,
}: MessageListProps) {
  return (
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
            className={`rounded-2xl px-4 py-3 text-sm leading-relaxed shadow-sm ${
              msg.role === 'user'
                ? 'bg-indigo-600 text-white rounded-tr-none max-w-2xl'
                : 'bg-neutral-900 border border-neutral-800 text-neutral-200 rounded-tl-none w-full max-w-3xl'
            }`}
          >
            {msg.content ? (
              <div className="whitespace-pre-wrap break-words">{msg.content}</div>
            ) : isStreaming &&
              msg.role === 'assistant' &&
              (!msg.proposals || msg.proposals.length === 0) &&
              !msg.proposal ? (
              <div className="flex items-center gap-1.5 text-xs text-neutral-400 py-1">
                <span className="w-2 h-2 rounded-full bg-indigo-400 animate-pulse" />
                <span>Thinking & drafting response...</span>
              </div>
            ) : null}

            {/* Proposed Edit Diff Cards (Rendered sequentially) */}
            {msg.proposals && msg.proposals.length > 0 ? (
              <div className="mt-3 space-y-3">
                {/* Batch actions if more than 1 pending proposal */}
                {msg.proposals.filter((p) => p.status === 'pending').length > 1 && (
                  <div className="flex items-center justify-between px-3 py-2 bg-neutral-950/80 border border-neutral-800 rounded-lg text-xs">
                    <span className="text-neutral-400 font-medium">
                      {msg.proposals.filter((p) => p.status === 'pending').length} pending changes
                    </span>
                    <div className="flex items-center gap-2">
                      <button
                        type="button"
                        onClick={() => onRejectAll(msg.id, msg.proposals!)}
                        className="px-2.5 py-1 text-xs rounded bg-neutral-800 hover:bg-neutral-700 text-neutral-300 transition-colors cursor-pointer"
                      >
                        Reject All
                      </button>
                      <button
                        type="button"
                        onClick={() => onAcceptAll(msg.id, msg.proposals!)}
                        className="px-2.5 py-1 text-xs rounded bg-emerald-600 hover:bg-emerald-500 text-white font-medium transition-colors cursor-pointer"
                      >
                        Accept All
                      </button>
                    </div>
                  </div>
                )}

                {msg.proposals.map((prop, idx) => (
                  <div key={prop.id || idx} className="space-y-1">
                    {msg.proposals!.length > 1 && (
                      <div className="flex items-center justify-between text-[11px] font-mono text-neutral-400 px-1 pt-1">
                        <span className="font-semibold text-neutral-300">
                          Diff {idx + 1} of {msg.proposals!.length}
                        </span>
                        <span className="text-neutral-500 truncate max-w-xs">{prop.path}</span>
                      </div>
                    )}
                    <DiffCard
                      targetType={prop.targetType}
                      targetPath={prop.path}
                      diff={prop.diff}
                      explanation={prop.explanation}
                      validationError={prop.validationError}
                      sources={prop.sources}
                      status={prop.status}
                      isProcessing={Boolean(processingProposalIds[prop.id])}
                      onAccept={() => onAcceptProposal(msg.id, prop.id, prop)}
                      onReject={() => onRejectProposal(msg.id, prop.id)}
                    />
                  </div>
                ))}
              </div>
            ) : msg.proposal ? (
              <DiffCard
                targetType={msg.proposal.targetType}
                targetPath={msg.proposal.path}
                diff={msg.proposal.diff}
                explanation={msg.proposal.explanation}
                validationError={msg.proposal.validationError}
                sources={msg.proposal.sources}
                status={msg.proposalStatus || 'pending'}
                isProcessing={Boolean(processingProposalIds['legacy'])}
                onAccept={() => onAcceptProposal(msg.id, 'legacy', msg.proposal!)}
                onReject={() => onRejectProposal(msg.id, 'legacy')}
              />
            ) : null}

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
  );
}
