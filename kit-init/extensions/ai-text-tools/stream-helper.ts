/**
 * Shared AI streaming helper for text tools extension.
 * Handles streaming responses from Anthropic or OpenAI APIs.
 * Supports both single prompts and full conversation context.
 */

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface StreamOptions {
  /** System prompt for the AI */
  systemPrompt: string;
  /** Single user prompt (for initial request) */
  userPrompt?: string;
  /** Full message history (for follow-ups) */
  messages?: Message[];
  /** Called for each chunk of streamed text */
  onChunk: (chunk: string) => void;
  /** Called on error with user-friendly message */
  onError: (error: string) => void;
  /** Called when stream completes */
  onComplete: () => void;
}

/** Get the configured AI provider and key */
export function getAiProvider(): { provider: 'anthropic' | 'openai' | null; key: string | null } {
  const anthropicKey = process.env.SCRIPT_KIT_ANTHROPIC_API_KEY || process.env.ANTHROPIC_API_KEY;
  const openaiKey = process.env.SCRIPT_KIT_OPENAI_API_KEY || process.env.OPENAI_API_KEY;

  if (anthropicKey) return { provider: 'anthropic', key: anthropicKey };
  if (openaiKey) return { provider: 'openai', key: openaiKey };
  return { provider: null, key: null };
}

/** Get model name for display */
export function getModelName(): string {
  const { provider } = getAiProvider();
  if (provider === 'anthropic') return 'Claude 3.5 Sonnet';
  if (provider === 'openai') return 'GPT-4o';
  return 'No AI configured';
}

export async function streamAiResponse(options: StreamOptions): Promise<void> {
  const { systemPrompt, userPrompt, messages, onChunk, onError, onComplete } = options;

  const { provider, key } = getAiProvider();

  if (!provider || !key) {
    onError('No AI API key configured. Set SCRIPT_KIT_ANTHROPIC_API_KEY or SCRIPT_KIT_OPENAI_API_KEY in your environment.');
    return;
  }

  // Build messages array
  const chatMessages: Message[] = messages || [{ role: 'user', content: userPrompt || '' }];

  try {
    if (provider === 'anthropic') {
      await streamAnthropic(key, systemPrompt, chatMessages, onChunk);
    } else {
      await streamOpenAI(key, systemPrompt, chatMessages, onChunk);
    }
    onComplete();
  } catch (error: any) {
    const msg = error.message || 'Failed to get AI response';
    onError(msg);
  }
}

async function streamAnthropic(
  apiKey: string,
  systemPrompt: string,
  messages: Message[],
  onChunk: (chunk: string) => void
): Promise<void> {
  // Filter out system messages for Anthropic (uses separate system param)
  const userMessages = messages.filter(m => m.role !== 'system');

  const response = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': apiKey,
      'anthropic-version': '2023-06-01',
    },
    body: JSON.stringify({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 4096,
      system: systemPrompt,
      messages: userMessages.map(m => ({ role: m.role, content: m.content })),
      stream: true,
    }),
  });

  if (!response.ok) {
    const errorBody = await response.text().catch(() => '');
    let errorMsg = `Anthropic API error (${response.status})`;
    try {
      const parsed = JSON.parse(errorBody);
      if (parsed.error?.message) errorMsg = parsed.error.message;
    } catch {}
    throw new Error(errorMsg);
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error('No response body');

  const decoder = new TextDecoder();
  let buffer = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';

    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = line.slice(6);
        if (data === '[DONE]') continue;
        try {
          const parsed = JSON.parse(data);
          if (parsed.type === 'content_block_delta' && parsed.delta?.text) {
            onChunk(parsed.delta.text);
          }
        } catch {}
      }
    }
  }
}

async function streamOpenAI(
  apiKey: string,
  systemPrompt: string,
  messages: Message[],
  onChunk: (chunk: string) => void
): Promise<void> {
  // Add system message at the beginning for OpenAI
  const allMessages = [
    { role: 'system' as const, content: systemPrompt },
    ...messages.map(m => ({ role: m.role, content: m.content })),
  ];

  const response = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${apiKey}`,
    },
    body: JSON.stringify({
      model: 'gpt-4o',
      max_tokens: 4096,
      messages: allMessages,
      stream: true,
    }),
  });

  if (!response.ok) {
    const errorBody = await response.text().catch(() => '');
    let errorMsg = `OpenAI API error (${response.status})`;
    try {
      const parsed = JSON.parse(errorBody);
      if (parsed.error?.message) errorMsg = parsed.error.message;
    } catch {}
    throw new Error(errorMsg);
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error('No response body');

  const decoder = new TextDecoder();
  let buffer = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';

    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = line.slice(6);
        if (data === '[DONE]') continue;
        try {
          const parsed = JSON.parse(data);
          const chunk = parsed.choices?.[0]?.delta?.content;
          if (chunk) onChunk(chunk);
        } catch {}
      }
    }
  }
}
