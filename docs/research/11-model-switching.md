# AI Chat Model Switching + Configuration Research

Research date: 2026-02-01

## Existing product patterns

### OpenAI ChatGPT model selector
- Paid ChatGPT plans can switch models using a model selector, with a dropdown at the top of the page. ([OAI-1])
- ChatGPT sometimes routes individual messages to a different model and shows a small annotation like "Used GPT-5" under the response; the model picker still shows the user-selected default. Routing is per-message. ([OAI-2])

### Anthropic Claude model selector
- Claude shows the currently selected model under the text input; clicking the model name opens the selector. ([ANTH-1])
- Switching models within an existing chat opens a new chat. ([ANTH-1])

### Anthropic Claude Code model configuration (CLI)
- Claude Code supports model aliases like `default`, `sonnet`, `opus`, and `haiku`, and a full model name can be used instead. ([ANTH-2])
- Model selection priority order: in-session command, startup flag, environment variable, settings file. ([ANTH-2])

### Vercel AI Gateway + AI SDK
- AI Gateway provides a unified API to switch models/providers without rewriting app code; it also supports provider routing and model fallbacks. ([VERCEL-1])
- Models are identified using a `creator/model-name` format and can be set per-request or globally. ([VERCEL-1])
- Vercel positions AI Gateway as a single endpoint that centralizes access, reduces provider/key management overhead, and offers intelligent failovers. ([VERCEL-2])
- The AI SDK AI Gateway provider highlights: multi-provider access, easy switching, automatic authentication on Vercel, and observability via the Vercel dashboard. ([AISDK-1])

## UX patterns distilled
- Always-visible model picker: either in the top header (ChatGPT) or under the composer (Claude).
- Keep a clear "current model" indicator near the input.
- When system routing happens, add a per-message annotation that shows the actual model used.
- Consider defaulting model switches to a new chat (Claude) to avoid cross-model context confusion.

## API key management considerations
- Support a "single gateway key" mode for multi-provider access (aligns with AI Gateway's one-endpoint and reduced key management story). ([VERCEL-2])
- Support provider-specific keys when bypassing a gateway; show key status inline in the model picker.
- Use a layered config priority similar to Claude Code: session override > app launch flag > environment variable > settings file. ([ANTH-2])
- Provide "Bring Your Own Key" UX with lightweight validation and clear error states per provider.

## Provider switching suggestions for Script Kit AI chat window
- Model picker layout: top-right or under-composer pill; clicking opens a searchable panel.
- Group models by provider; add a quick "Auto" / "Recommended" group at the top.
- Show a small badge on each assistant message when the actual model differs from the default (mirrors "Used GPT-5"). ([OAI-2])
- Switching model in an active chat should default to a new chat, but offer a "Continue in same chat" option for advanced users. ([ANTH-1])
- Add fallback rules: on provider/model error, auto-fallback to the next configured provider with an inline notice.
- Surface capability hints (speed, cost, context, tools) in the selector for faster tradeoffs.

## Configuration model suggestions
- Store selection state per conversation: `default_model`, `actual_model_used`, `provider`, `fallback_chain`.
- Expose a structured config for defaults and aliases (provider + model id + label + capabilities).
- Allow per-message override for power users (e.g., a slash command).

## Sources
- [OAI-1] OpenAI Help Center: "What is the ChatGPT model selector?" https://help.openai.com/en/articles/7864572-what-is-the-chatgpt-plus-model-selector%3F
- [OAI-2] OpenAI Help Center: "Why you may see \"Used GPT-5\" in ChatGPT" https://help.openai.com/en/articles/12454167
- [ANTH-1] Anthropic Help Center: "How can I change the model version that I'm chatting with?" https://support.anthropic.com/en/articles/8664678-how-can-i-change-the-model-version-that-i-m-chatting-with
- [ANTH-2] Anthropic Docs: "Claude Code model configuration" https://docs.anthropic.com/en/docs/claude-code/model-config
- [VERCEL-1] Vercel Docs: "Models & Providers" (AI Gateway) https://vercel.com/docs/ai-gateway/models-and-providers
- [VERCEL-2] Vercel: "AI Gateway" landing page https://vercel.com/ai-gateway
- [AISDK-1] Vercel AI SDK Docs: "AI Gateway Provider" https://v5.ai-sdk.dev/providers/ai-sdk-providers/ai-gateway
