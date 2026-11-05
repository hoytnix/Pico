import type { ProposeEditResult } from '../../lib/vibeAgent';

export interface ProposalItem extends ProposeEditResult {
  id: string;
  status: 'pending' | 'accepted' | 'rejected';
}

export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  proposals?: ProposalItem[];
  proposal?: ProposeEditResult;
  proposalStatus?: 'pending' | 'accepted' | 'rejected';
  reasoning?: string;
  error?: string;
}

export interface ProposalCallbacks {
  onPostUpdatedFromVibe?: (path: string, newMarkdown: string) => void;
  onTemplateUpdatedFromVibe?: (path: string, newContent: string) => void;
  onThemeUpdatedFromVibe?: (newContent: string) => void;
  onModelUpdatedFromVibe?: (path: string) => void;
}
