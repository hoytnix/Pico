import { picoApi } from '../../api';
import type { ProposeEditResult } from '../vibeAgent';
import type { ProposalCallbacks } from '../../components/vibe/types';

/**
 * Persists an accepted Vibe proposal to disk through Tauri IPC commands
 * and invokes appropriate UI synchronization callbacks.
 */
export async function applyAcceptedProposal(
  proposal: ProposeEditResult,
  callbacks?: ProposalCallbacks
): Promise<void> {
  if (proposal.targetType === 'content') {
    const doc = await picoApi.readPost(proposal.path);
    await picoApi.savePost(proposal.path, doc.frontmatter, proposal.proposed);
    callbacks?.onPostUpdatedFromVibe?.(proposal.path, proposal.proposed);
  } else if (proposal.targetType === 'template') {
    await picoApi.saveTemplate(proposal.path, proposal.proposed);
    callbacks?.onTemplateUpdatedFromVibe?.(proposal.path, proposal.proposed);
  } else if (proposal.targetType === 'theme') {
    await picoApi.saveTheme(proposal.proposed);
    callbacks?.onThemeUpdatedFromVibe?.(proposal.proposed);
  } else if (proposal.targetType === 'model') {
    await picoApi.saveModelYaml(proposal.path, proposal.proposed);
    callbacks?.onModelUpdatedFromVibe?.(proposal.path);
  }
}
