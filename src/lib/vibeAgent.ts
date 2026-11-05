import { ToolLoopAgent, tool } from 'ai';
import { createGoogle } from '@ai-sdk/google';
import { z } from 'zod';
import { picoApi } from '../api';
import { picoLog } from './logger';
import { normalizeModelPath, normalizeRelativePath } from './slug';
import {
  type VibeAgentConfig,
  formatLlmError,
  normalizeGoogleModelId,
  buildProviderOptions,
  createVibeModel,
} from './llmProviders';
import type { GroundingSource } from '../types';

export {
  type VibeAgentConfig,
  formatLlmError,
  normalizeGoogleModelId,
  buildProviderOptions,
  createVibeModel,
};

export interface ProposeEditResult {
  id?: string;
  targetType: 'content' | 'template' | 'theme' | 'model';
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

    readModelFile: tool({
      description: 'Read the YAML schema definition of a collection model in the site.',
      inputSchema: z.object({
        path: z.string().describe('Relative path or name of the model YAML file, e.g. models/blog.yaml or blog.yaml'),
      }),
      execute: async ({ path }) => {
        try {
          const cleanPath = normalizeModelPath(path);
          const ctx = await picoApi.getVibeContext({ kind: 'model', path: cleanPath });
          return { path: cleanPath, content: ctx.original_content };
        } catch (err: any) {
          return { error: `Failed to read model file ${path}: ${err.message || String(err)}` };
        }
      },
    }),

    listModels: tool({
      description: 'List all existing collection models in the site with their fields and templates.',
      inputSchema: z.object({}),
      execute: async () => {
        try {
          const models = await picoApi.listModels();
          return {
            models: models.map((m) => ({
              name: m.name,
              label: m.label,
              template: m.template,
              list_template: m.list_template,
              path: `models/${m.name}.yaml`,
              fields: m.fields.map((f) => ({
                name: f.name,
                label: f.label,
                type: f.type,
                required: f.required,
              })),
            })),
          };
        } catch (err: any) {
          return { error: `Failed to list models: ${err.message || String(err)}` };
        }
      },
    }),

    proposeEdit: tool({
      description:
        'Propose an edit to a content file (markdown body), template file (Jinja), theme CSS, or collection model YAML. The edit will be presented to the user as a diff for review. After calling proposeEdit, explain what changes were made in your response.',
      inputSchema: z.object({
        targetType: z
          .enum(['content', 'template', 'theme', 'model'])
          .describe('The kind of target file being edited'),
        path: z
          .string()
          .describe('The file path, e.g. content/blog/first-post.md, templates/index.jinja, assets/theme.css, or models/blog.yaml'),
        proposedContent: z
          .string()
          .describe('The complete updated content (full markdown body for content, full template code for template, full css for theme, full yaml for model)'),
        explanation: z.string().optional().describe('Short explanation of what was changed and why'),
      }),
      execute: async ({ targetType, path, proposedContent, explanation }) => {
        try {
          let cleanPath = normalizeRelativePath(path);
          if (targetType === 'theme') {
            cleanPath = 'assets/theme.css';
          } else if (targetType === 'model') {
            cleanPath = normalizeModelPath(cleanPath);
          }
          const target = { kind: targetType, path: cleanPath };
          const evalResult = await picoApi.evaluateVibeProposal(target, proposedContent);

          const result: ProposeEditResult = {
            id: `proposal-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
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
            id: result.id,
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
You help users refine markdown content, customize MiniJinja templates, edit collection models (YAML schema definitions), and tweak theme CSS variables.
Safety rules:
- NEVER output raw shell commands or ask the user to use a terminal. Pico is completely terminal-free.
- To inspect files or schema, use readContentFile, readTemplateFile, readThemeCss, readModelFile, or listModels.
- To make a change, inspect the file first using the appropriate read tool, then call proposeEdit with the complete proposed new text.
- If the user asks for changes across multiple files, call proposeEdit sequentially for each affected file. Each proposal will be presented to the user as a distinct diff card for review.
- If the user asks to change theme colors, fonts, or styling of their app or site, inspect assets/theme.css using readThemeCss and propose the changes using proposeEdit.
- If the user asks to view, create, or modify content models / collections (e.g. adding fields to blog, creating a portfolio or team model, modifying model schema), inspect the models using readModelFile or listModels, then propose the updated YAML definition via proposeEdit with targetType 'model' and path 'models/<name>.yaml'.
- Collection models live in models/<collection>.yaml. Valid field types in Pico are: 'string', 'date', 'tags', 'image', 'bool'. Models must include 'title' and 'date' fields.
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
