---
name: script-kit-ai-testing
description: Testing AI chat features in Script Kit GPUI. Use when testing AI chat window, inline chat prompt, AI streaming, model selection, or iterating on AI UI designs. Covers both the separate AI window and the inline ChatPrompt, stdin commands, mock data, and visual verification.
---

# Script Kit AI Testing

Two distinct AI interfaces exist. Know which you're testing.

## Two AI Interfaces

### 1. AI Window (Separate Floating Window)
- Full chat app: sidebar with chat history, message bubbles, model dropdown, search, Cmd+K command bar
- Source: `src/ai/window/`
- Opens via `openAi` stdin command or "Ask AI" built-in

### 2. Inline Chat Prompt (ChatPrompt in Main Window)
- Input at TOP, full-width message containers (not bubbles), conversation turns
- Source: `src/prompts/chat/`
- Opens via Tab key with filter text, or SDK `chat()` call

## Stdin Commands for AI Testing

### AI Window Commands
```json
{"type":"openAi"}                                          // Open empty AI window
{"type":"openAiWithMockData"}                              // Open with sample conversations
{"type":"showAiCommandBar"}                                // Open Cmd+K actions menu
{"type":"setAiInput","text":"Hello AI","submit":true}      // Set input text & optionally submit
{"type":"setAiSearch","text":"python"}                      // Filter chat list sidebar
{"type":"simulateAiKey","key":"enter"}                     // Simulate keypress in AI window
{"type":"captureWindow","title":"Script Kit AI","path":".test-screenshots/ai.png"}  // Screenshot
```

### Inline Chat Prompt Commands
```json
{"type":"run","path":"/abs/path/to/chat-test.ts"}          // Run a chat() test script
```

## Quick Start: AI Window Visual Test

```bash
cargo build && echo '{"type":"openAiWithMockData"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

Then capture and verify:
```bash
# In a separate terminal or via named pipe
echo '{"type":"captureWindow","title":"Script Kit AI","path":".test-screenshots/ai-window.png"}'
```

Always **read the PNG file** after capture to verify visually.

## Quick Start: Inline Chat Prompt Test

```ts
// tests/smoke/test-chat-inline.ts
import '../../scripts/kit-sdk';

function log(test: string, status: string, extra: any = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

const name = "chat-inline";
log(name, "running");
const start = Date.now();
try {
  await chat({
    placeholder: "Ask anything...",
    messages: [{ role: 'user', content: 'Hello' }],
    useBuiltinAi: true,
  });
  log(name, "pass", { duration_ms: Date.now() - start });
} catch (e) {
  log(name, "fail", { error: String(e), duration_ms: Date.now() - start });
}
process.exit(0);
```

Run:
```bash
cargo build && echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-chat-inline.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

## Mock Data Mode

`openAiWithMockData` inserts sample conversations into the AI window database so you can visually test the sidebar, chat list, and message rendering without needing real API keys.

Use this for:
- Sidebar layout iteration
- Chat history rendering
- Search/filter behavior
- Visual regression checks

## API Key Configuration for Testing

Set environment variables to enable providers:
```bash
SCRIPT_KIT_ANTHROPIC_API_KEY=sk-...   # Anthropic (default provider)
SCRIPT_KIT_OPENAI_API_KEY=sk-...      # OpenAI
SCRIPT_KIT_CLAUDE_CODE_ENABLED=1       # Enable Claude Code provider
```

Without API keys, the inline chat shows a **setup card** instead of the chat input. Test this state with:
```bash
unset SCRIPT_KIT_ANTHROPIC_API_KEY SCRIPT_KIT_OPENAI_API_KEY
cargo build && echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-ai-setup-card.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

## Visual Iteration Workflow

1. Make UI change in `src/ai/window/` or `src/prompts/chat/`
2. `cargo build`
3. Open with mock data: `echo '{"type":"openAiWithMockData"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
4. Capture screenshot: `{"type":"captureWindow","title":"Script Kit AI","path":".test-screenshots/ai-iteration.png"}`
5. **Read the PNG** to verify
6. Repeat until satisfied

For inline chat, use the test script pattern above with `captureScreenshot()` from the SDK.

## Named Pipe Pattern (Interactive Iteration)

For sending multiple commands to a running instance:

```bash
mkfifo /tmp/skpipe
cargo build && cat /tmp/skpipe | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 &

# Send commands one at a time
echo '{"type":"openAiWithMockData"}' > /tmp/skpipe
sleep 1
echo '{"type":"captureWindow","title":"Script Kit AI","path":".test-screenshots/ai-1.png"}' > /tmp/skpipe
sleep 0.5
echo '{"type":"setAiInput","text":"test message"}' > /tmp/skpipe
sleep 0.5
echo '{"type":"captureWindow","title":"Script Kit AI","path":".test-screenshots/ai-2.png"}' > /tmp/skpipe

rm /tmp/skpipe
```

## Protocol Messages Reference

### AI Window Protocol (`src/protocol/message/variants/ai.rs`)

| Message | Direction | Purpose |
|---------|-----------|---------|
| `aiStartChat` | SDK→App | Start new conversation |
| `aiSendMessage` | SDK→App | Send message & get AI response |
| `aiAppendMessage` | SDK→App | Add message without triggering response |
| `aiGetConversation` | SDK→App | Get messages from a chat |
| `aiListChats` | SDK→App | List all chats |
| `aiDeleteChat` | SDK→App | Delete a chat |
| `aiSubscribe` | SDK→App | Subscribe to streaming events |
| `aiStreamChunk` | App→SDK | Streaming token chunk |
| `aiStreamComplete` | App→SDK | Stream finished |
| `aiError` | App→SDK | Error event |

### Inline Chat Protocol (`src/protocol/message/variants/prompts_media.rs`)

| Message | Direction | Purpose |
|---------|-----------|---------|
| `Chat` | SDK→App | Create chat prompt UI |
| `ChatMessage` | SDK→App | Add message to conversation |
| `ChatStreamStart` | SDK→App | Begin streaming response |
| `ChatStreamChunk` | SDK→App | Stream token |
| `ChatStreamComplete` | SDK→App | End streaming |
| `ChatSubmit` | App→SDK | User submitted a message |
| `ChatSetError` | SDK→App | Show error on message |
| `ChatClear` | SDK→App | Clear all messages |

## Existing Test Files

| Test | What It Covers |
|------|---------------|
| `tests/smoke/test-chat-simple.ts` | Basic `chat()` without callbacks |
| `tests/smoke/test-chat-builtin-ai.ts` | Built-in AI mode (`useBuiltinAi: true`) |
| `tests/smoke/test-inline-ai-chat.ts` | Tab key triggers ChatPrompt |
| `tests/smoke/test-ai-window-visual-suite.ts` | Visual screenshots of AI window |
| `tests/smoke/test-ai-setup-card.ts` | Setup card when no API keys |
| `tests/smoke/test-ai-actions-selection.ts` | Actions menu in chat |
| `tests/smoke/test-ai-actions-menu.ts` | Command bar (Cmd+K) |
| `tests/smoke/test-ai-dropdown.ts` | Model selection dropdown |
| `tests/smoke/test-ai-window-sidebar.ts` | Chat list sidebar |
| `tests/smoke/test-chat-callbacks.ts` | SDK streaming callbacks |
| `tests/smoke/test-chat-edge-cases.ts` | Edge case handling |

## Key Source Files

| File | Purpose |
|------|---------|
| `src/ai/window/` | AI chat window (all rendering, state, streaming) |
| `src/ai/window/window_api.rs` | Public API: `open_ai_window`, `set_ai_input`, etc. |
| `src/ai/window/init.rs` | Window initialization, placeholder "Ask anything..." |
| `src/ai/window/command_bar.rs` | Cmd+K command bar |
| `src/ai/config.rs` | API key detection, provider config |
| `src/ai/model.rs` | Chat, Message, ChatId data models |
| `src/ai/storage.rs` | SQLite persistence (`~/.scriptkit/ai-chats.db`) |
| `src/ai/providers.rs` | Provider trait (OpenAI, Anthropic, etc.) |
| `src/ai/session.rs` | Claude Code CLI session manager |
| `src/prompts/chat/` | Inline ChatPrompt (all rendering, state, streaming) |
| `src/prompts/chat/prompt.rs` | ChatPrompt struct & lifecycle |
| `src/prompts/chat/render_input.rs` | Input field rendering |
| `src/prompts/chat/render_turns.rs` | Message turn rendering |
| `src/prompts/chat/streaming.rs` | Built-in AI streaming logic |
| `src/prompts/chat/render_setup.rs` | API key setup card |
| `src/app_impl/prompt_ai.rs` | Inline chat prompt wiring & script generation |

## Testing Checklist

- [ ] AI window opens: `{"type":"openAi"}`
- [ ] AI window with mock data: `{"type":"openAiWithMockData"}`
- [ ] Sidebar shows chat list with dates (Today, This Week)
- [ ] Search filters chat list: `{"type":"setAiSearch","text":"..."}`
- [ ] Input field shows placeholder "Ask anything..."
- [ ] Input accepts text: `{"type":"setAiInput","text":"..."}`
- [ ] Submit triggers streaming: `{"type":"setAiInput","text":"...","submit":true}`
- [ ] Model dropdown shows available providers
- [ ] Command bar opens: `{"type":"showAiCommandBar"}`
- [ ] Inline ChatPrompt via `chat({ useBuiltinAi: true })`
- [ ] Setup card when no API keys configured
- [ ] Error messages display correctly (ChatSetError)
- [ ] Screenshots capture AI window: `{"type":"captureWindow","title":"Script Kit AI",...}`

## Anti-Patterns

- Running scripts via CLI args (must use stdin JSON)
- Testing AI streaming without `SCRIPT_KIT_AI_LOG=1` (log output explodes)
- Forgetting to read the PNG after `captureWindow`
- Testing with real API keys when mock data suffices (slow, costs money)
- Confusing AI window (`src/ai/window/`) with inline ChatPrompt (`src/prompts/chat/`)
- Not setting `useBuiltinAi: true` when testing built-in AI mode
