import { ToolLoopAgent, tool } from 'ai';
import { createGoogle } from '@ai-sdk/google';
import { createOpenAI } from '@ai-sdk/openai';
import { z } from 'zod';
import { picoApi } from '../api';
import { picoLog } from './logger';
import type { GroundingSource } from '../types';

export interface VibeAgentConfig {
  provider: string;
  model: string;
  apiKey: string;
  baseUrl?: string;
  thinking?: string;
  grounding?: boolean;
}

export function formatLlmError(err: unknown, provider: string, model: string): string {
  const msg = err instanceof Error ? err.message : String(err);
  const lower = msg.toLowerCase();

  // 401 / invalid key
  if (
    lower.includes('401') ||
    lower.includes('invalid api key') ||
    lower.includes('incorrect api key') ||
    lower.includes('unauthenticated') ||
    lower.includes('unauthorized') ||
    lower.includes('invalid_api_key')
  ) {
    return `Invalid API key for ${provider}. Check the key in Settings. (${msg})`;
  }

  // 402 / billing (Muse / Meta)
  if (
    lower.includes('402') ||
    lower.includes('billing_not_configured') ||
    lower.includes('billing not configured') ||
    (lower.includes('billing') && (lower.includes('payment') || lower.includes('quota')))
  ) {
    return `Billing is not configured for Muse. Update the payment method in the Meta Model API dashboard, then retry. (${msg})`;
  }

  // 404 / model not found
  if (
    lower.includes('404') ||
    lower.includes('model not found') ||
    lower.includes('model_not_found') ||
    lower.includes('does not exist')
  ) {
    return `Model ${model} was not found for ${provider}. Check the model ID or pick another model. (${msg})`;
  }

  // 400 / thinking
  if (
    lower.includes('400') &&
    (lower.includes('thinking') || lower.includes('budget') || lower.includes('reasoning') || lower.includes('minimal'))
  ) {
    return `Thinking setting is not supported for this model/provider combination. (${msg})`;
  }

  // 429 / rate limit
  if (
    lower.includes('429') ||
    lower.includes('rate limit') ||
    lower.includes('rate_limit_exceeded') ||
    lower.includes('resource_exhausted')
  ) {
    return `Rate limited by ${provider}. Wait and retry. (${msg})`;
  }

  // Network / timeout / webkit load failed
  if (
    lower.includes('timeout') ||
    lower.includes('timed out') ||
    lower.includes('network') ||
    lower.includes('fetch failed') ||
    lower.includes('load failed') ||
    lower.includes('connection refused') ||
    lower.includes('econnrefused')
  ) {
    return `Could not reach ${provider}. Check your connection and the base URL, then retry. (${msg})`;
  }

  return `${provider} error (${model}): ${msg}`;
}

export function buildProviderOptions(config: VibeAgentConfig): Record<string, any> {
  const { provider, model, thinking } = config;

  if (provider === 'gemini-native') {
    const lowerModel = model.toLowerCase();
    if (thinking && thinking !== 'default') {
      if (lowerModel.includes('2.5')) {
        const budget = thinking === 'low' ? 1024 : thinking === 'high' ? 24576 : 8192;
        return {
          google: {
            thinkingConfig: { thinkingBudget: budget },
          },
        };
      } else {
        const level = thinking === 'low' ? 'low' : thinking === 'high' ? 'high' : 'medium';
        return {
          google: {
            thinkingConfig: { thinkingLevel: level },
          },
        };
      }
    }
    return {};
  }

  if (provider !== 'meta' && thinking && thinking !== 'default') {
    return {
      openai: {
        reasoningEffort: thinking,
      },
    };
  }

  return {};
}

export function createVibeModel(config: VibeAgentConfig) {
  const { provider, model, apiKey, baseUrl } = config;

  if (provider === 'gemini-native') {
    const google = createGoogle({ apiKey });
    return google(model);
  }

  // OpenAI-compatible providers
  let resolvedBaseUrl = 'https://api.openai.com/v1';
  let headers: Record<string, string> | undefined = undefined;

  if (provider === 'openrouter') {
    resolvedBaseUrl = 'https://openrouter.ai/api/v1';
    headers = {
      'HTTP-Referer': 'https://pico.local',
      'X-Title': 'Pico',
    };
  } else if (provider === 'meta') {
    resolvedBaseUrl = 'https://api.meta.ai/v1';
  } else if (provider === 'gemini-openai') {
    resolvedBaseUrl = 'https://generativelanguage.googleapis.com/v1beta/openai';
  } else if (provider === 'custom') {
    resolvedBaseUrl = baseUrl || 'http://127.0.0.1:11434/v1';
  }

  const openai = createOpenAI({
    apiKey,
    baseURL: resolvedBaseUrl,
    headers,
  });

  return openai(model);
}

export interface ProposeEditResult {
  targetType: 'content' | 'template' | 'theme';
  path: string;
  diff: string;
  proposed: string;
  validationError?: string | null;
  sources?: GroundingSource[] | null;
  explanation: string;
}

export function createVibeAgent(
  config: VibeAgentConfig,
  onProposalCreated?: (result: ProposeEditResult) => void,
  onError?: (error: unknown) => void
) {
  const modelInstance = createVibeModel(config);

  const tools: Record<string, any> = {
    readContentFile: tool({
      description: 'Read the current frontmatter and markdown body of a content file in the site.',
      inputSchema: z.object({
        path: z.string().describe('Relative path to the content file, e.g. content/blog/first-post.md'),
      }),
      execute: async ({ path }) => {
        try {
          const doc = await picoApi.readPost(path);
          return {
            path,
            frontmatter: doc.frontmatter,
            markdown: doc.markdown,
          };
        } catch (err: any) {
          return { error: `Failed to read content file ${path}: ${err.message || String(err)}` };
        }
      },
    }),

    readTemplateFile: tool({
      description: 'Read the current MiniJinja code of a template in the site.',
      inputSchema: z.object({
        path: z.string().describe('Relative path to the template, e.g. templates/index.jinja or templates/skeleton.jinja'),
      }),
      execute: async ({ path }) => {
        try {
          const content = await picoApi.readTemplate(path);
          return { path, content };
        } catch (err: any) {
          return { error: `Failed to read template ${path}: ${err.message || String(err)}` };
        }
      },
    }),

    readThemeCss: tool({
      description: 'Read the CSS variables in assets/theme.css of the site.',
      inputSchema: z.object({}),
      execute: async () => {
        try {
          const content = await picoApi.readTheme();
          return { path: 'assets/theme.css', content };
        } catch (err: any) {
          return { error: `Failed to read theme CSS: ${err.message || String(err)}` };
        }
      },
    }),

    proposeEdit: tool({
      description:
        'Propose an edit to a content file (markdown body), template file (Jinja), or theme CSS. The edit will be presented to the user as a diff for review. After calling proposeEdit, explain what changes were made in your response.',
      inputSchema: z.object({
        targetType: z
          .enum(['content', 'template', 'theme'])
          .describe('The kind of target file being edited'),
        path: z
          .string()
          .describe('The file path, e.g. content/blog/first-post.md, templates/index.jinja, or assets/theme.css'),
        proposedContent: z
          .string()
          .describe('The complete updated content (full markdown body for content, full template code for template, full css for theme)'),
        explanation: z.string().optional().describe('Short explanation of what was changed and why'),
      }),
      execute: async ({ targetType, path, proposedContent, explanation }) => {
        try {
          let cleanPath = path.trim().replace(/\\/g, '/').replace(/^\.?\//, '');
          if (targetType === 'theme') {
            cleanPath = 'assets/theme.css';
          }
          const target = { kind: targetType, path: cleanPath };
          const evalResult = await picoApi.evaluateVibeProposal(target, proposedContent);

          const result: ProposeEditResult = {
            targetType,
            path: cleanPath,
            diff: evalResult.diff,
            proposed: evalResult.proposed,
            validationError: evalResult.validation_error,
            sources: evalResult.sources,
            explanation: explanation || 'Proposed modification',
          };

          if (onProposalCreated) {
            onProposalCreated(result);
          }

          return {
            status: 'proposal-created',
            path: cleanPath,
            explanation: result.explanation,
            message: 'The proposed edit has been created and presented to the user as a diff for review and approval.',
          };
        } catch (err: any) {
          return { error: `Failed to evaluate proposal for ${path}: ${err.message || String(err)}` };
        }
      },
    }),
  };

  // Gemini Native Google Search grounding tool if requested (§7)
  if (config.provider === 'gemini-native' && config.grounding) {
    const google = createGoogle({ apiKey: config.apiKey });
    tools.googleSearch = google.tools.googleSearch({});
  }

  const instructions = `You are Vibe, an expert AI web designer and content editor in Pico (a local-first desktop static site generator).
You help users refine markdown content, customize MiniJinja templates, and tweak theme CSS variables.
Safety rules:
- NEVER output raw shell commands or ask the user to use a terminal. Pico is completely terminal-free.
- To make a change, inspect the file first using readContentFile, readTemplateFile, or readThemeCss, then call proposeEdit with the complete proposed new text.
- If the user asks to change theme colors, fonts, or styling of their app or site, inspect assets/theme.css using readThemeCss and propose the changes using proposeEdit.
- Do NOT fabricate file paths or hallucinate models.
- When editing MiniJinja templates, adhere to model variables and never invent unknown variables or top-level macro HTML.
- When proposing an edit, the proposal will be presented to the user as a unified diff with Accept / Reject buttons. Explain your reasoning clearly.`;

  return new ToolLoopAgent({
    model: modelInstance,
    instructions,
    tools,
    providerOptions: buildProviderOptions(config),
    onError: (event: any) => {
      const err = event?.error ?? event;
      picoLog.error('vibe', 'ToolLoopAgent error:', err);
      if (onError) {
        onError(err);
      }
    },
  });
}
