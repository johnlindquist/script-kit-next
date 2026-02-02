# Vercel AI SDK + AI Gateway Research

## Scope
- Streaming patterns (text streams vs UI message/data streams)
- Multi-provider support via Vercel AI Gateway
- Edge runtime constraints relevant to streaming
- Integration plan for Script Kit GPUI AI chat

## Streaming Patterns (AI SDK)

### 1) Text stream protocol (plain text)
- `streamText` provides a `textStream` you can iterate over to stream deltas in real time.
- To return a plain text stream from an API route, use `streamText(...).toTextStreamResponse()`.
- On the client, `useChat` supports text streams by setting `streamProtocol: 'text'` and using `TextStreamChatTransport`.
- Tradeoff: text streams only carry text (no tool calls, usage metadata, or finish reasons).

### 2) UI message / data stream protocol (SSE)
- The UI message stream protocol uses SSE; it is the **default** for `useChat`.
- A custom backend must set the header `x-vercel-ai-ui-message-stream: v1`.
- On the backend, use `streamText(...).toUIMessageStreamResponse()` to emit the protocol.
- Stream parts include message start, text deltas, reasoning, tool inputs/results, custom `data-*` parts, and a `[DONE]` terminator.
- This is the better fit when you need tool calls, message metadata, or non-text payloads.

### 3) Consuming UI message streams outside React
- The AI SDK provides `readUIMessageStream` for non-UI clients (terminal UIs, custom processors).
- This is a reference for how to process UI message stream chunks into incremental message state.

## Multi-Provider Support / Vercel AI Gateway

### Unified model access
- AI Gateway exposes a unified API to switch providers/models without rewriting the app.
- Model IDs follow `creator/model-name` (e.g., `openai/gpt-5.2`, `xai/grok-code-fast-1`).
- You can configure provider routing and model fallbacks for reliability.

### AI SDK integration options
- **Model string**: pass `model: 'creator/model-name'` to AI SDK functions; AI SDK uses AI Gateway automatically.
- **Provider instance**: use `gateway('creator/model')` or `createGateway({ apiKey, baseURL })` for custom base URL or API key (commonly imported from `@ai-sdk/gateway`; `gateway` is also available from `ai` in newer AI SDK versions).
- Default Gateway base URL: `https://ai-gateway.vercel.sh/v1/ai`
- Default API key env var: `AI_GATEWAY_API_KEY`

### Authentication & operations
- When deployed on Vercel, AI Gateway supports automatic OIDC-based auth (no API keys required).
- For local development, use `vercel env pull` or `vercel dev` for token management.
- Model discovery is available via the public models endpoint: `https://ai-gateway.vercel.sh/v1/models`

## Edge Runtime Considerations (Vercel)
- Edge Functions **must begin streaming within 25 seconds** to keep the connection alive.
- Edge Functions can **continue streaming for up to 300 seconds**.
- Edge runtime is V8-based and exposes only a subset of Web APIs.
- Dynamic code execution (`eval`, `new Function`, etc.) is not allowed.
- Implication: keep server code lightweight and avoid Node-only or eval-based dependencies.

## Integration Plan: Script Kit GPUI AI Chat via AI Gateway

### Recommended architecture
1. **Backend route/service** (Next.js API route, Bun server, or small Node service)
2. **AI SDK + AI Gateway** in the backend
3. **Streaming response** to GPUI client

### Step 1: Choose streaming protocol
- **Use UI message/data stream** if we want tool calls, reasoning, usage, or structured `data-*` parts.
- **Use text stream** if we only need token-by-token text and want minimal parsing.

### Step 2: Backend implementation (TypeScript sketch)
```ts
import { streamText } from 'ai';
import { gateway } from 'ai'; // or: import { gateway } from '@ai-sdk/gateway'

export async function POST(req: Request) {
  const { messages } = await req.json();

  const result = streamText({
    model: gateway('openai/gpt-5'), // or 'openai/gpt-5' as plain string
    messages,
  });

  // UI message/data stream (recommended for tool calls, metadata)
  return result.toUIMessageStreamResponse();

  // Or: text stream
  // return result.toTextStreamResponse();
}
```

### Step 3: GPUI client parsing
- **UI message stream**: implement an SSE parser that handles `start`, `text-start`, `text-delta`, `text-end`, `tool-*`, `data-*`, and `[DONE]`.
- **Text stream**: append UTF-8 chunks to the active assistant message.

### Step 4: Reliability + provider routing
- Use AI Gateway’s provider routing and model fallback settings to improve availability.
- Keep a model list locally, but consider periodic refresh via `/v1/models` for dynamic discovery.

### Step 5: Auth & deployment
- In Vercel deployments, prefer OIDC tokens (automatic).
- For non-Vercel environments, use `AI_GATEWAY_API_KEY`.

## Open Decisions
- Do we need tool-call support in the GPUI AI chat? If yes, choose UI message/data streams.
- Do we want to expose provider selection to users (AI Gateway model list), or keep a curated set?
- Where should the AI Gateway API key live (server-only service vs per-user BYOK)?

## Sources
- https://ai-sdk.dev/docs/ai-sdk-ui/stream-protocol
- https://ai-sdk.dev/docs/reference/ai-sdk-ui/use-chat
- https://ai-sdk.dev/docs/ai-sdk-ui/chatbot
- https://ai-sdk.dev/docs/ai-sdk-ui/reading-ui-message-streams
- https://ai-sdk.dev/docs/reference/ai-sdk-core/stream-text
- https://ai-sdk.dev/providers/ai-sdk-providers/ai-gateway
- https://vercel.com/docs/ai-gateway/models-and-providers
- https://vercel.com/docs/functions/runtimes/edge
