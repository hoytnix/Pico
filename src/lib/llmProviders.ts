import { createGoogle } from '@ai-sdk/google';
import { createOpenAI } from '@ai-sdk/openai';

export interface VibeAgentConfig {
  provider: string;
  model: string;
  apiKey: string;
  baseUrl?: string;
  thinking?: string;
  grounding?: boolean;
}

/**
 * Human-readable display labels for known Google / Gemini / Gemma model presets.
 */
export const MODEL_DISPLAY_LABELS: Record<string, string> = {
  'gemini-4-argon': 'gemini-4-argon (Gemini 4 Argon)',
  'gemini-3.8-flash': 'gemini-3.8-flash (Gemini 3.8 Flash)',
  'gemini-3.6-flash': 'gemini-3.6-flash (Gemini 3.6 Flash)',
  'gemini-3.5-flash': 'gemini-3.5-flash (Gemini 3.5 Flash)',
  'gemini-3.5-flash-lite': 'gemini-3.5-flash-lite (Gemini 3.5 Flash-Lite)',
  'gemini-3.1-flash-lite': 'gemini-3.1-flash-lite (Gemini 3.1 Flash-Lite)',
  'gemma-4-31b-it': 'gemma-4-31b-it (Gemma 4 31B)',
  'gemma-4-26b-a4b-it': 'gemma-4-26b-a4b-it (Gemma 4 26B)',
};

/**
 * Normalizes user-typed or legacy model aliases to canonical IDs.
 */
export function normalizeGoogleModelId(provider: string, model: string): string {
  const trimmed = model.trim();
  const lower = trimmed.toLowerCase();

  if (provider === 'gemini-native' || provider === 'gemini-openai') {
    switch (lower) {
      case 'gemma-4-26b':
      case 'gemma 4 26b':
      case 'gemma-4-26b-it':
      case 'gemma-4-26b-a4b':
        return 'gemma-4-26b-a4b-it';
      case 'gemma-4-31b':
      case 'gemma 4 31b':
        return 'gemma-4-31b-it';
      case 'gemini-3.6':
      case 'gemini 3.6':
      case 'gemini 3.6 flash':
        return 'gemini-3.6-flash';
      case 'gemini-3.1':
      case 'gemini 3.1':
      case 'gemini 3.1 flash-lite':
      case 'gemini-3.1-flash':
        return 'gemini-3.1-flash-lite';
      default:
        return trimmed;
    }
  } else if (provider === 'openrouter') {
    switch (lower) {
      case 'google/gemma-4-26b':
      case 'google/gemma-4-26b-it':
      case 'google/gemma-4-26b-a4b':
        return 'google/gemma-4-26b-a4b-it';
      case 'google/gemma-4-31b':
        return 'google/gemma-4-31b-it';
      case 'google/gemini-3.6':
      case 'google/gemini 3.6':
        return 'google/gemini-3.6-flash';
      case 'google/gemini-3.1':
      case 'google/gemini 3.1':
        return 'google/gemini-3.1-flash-lite';
      default:
        return trimmed;
    }
  }

  return trimmed;
}

/**
 * Converts provider error payloads and network failures into clear, actionable messages.
 */
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

/**
 * Builds AI SDK providerOptions for thinking, reasoning budgets, and search grounding.
 */
export function buildProviderOptions(config: VibeAgentConfig): Record<string, any> {
  const { provider, thinking } = config;
  const model = normalizeGoogleModelId(provider, config.model);

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

/**
 * Creates and initializes the corresponding Vercel AI SDK language model instance.
 */
export function createVibeModel(config: VibeAgentConfig) {
  const { provider, apiKey, baseUrl } = config;
  const model = normalizeGoogleModelId(provider, config.model);

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
