# ACP Chat

Primary and only AI chat surface. ACP Chat can render in-panel or in a detached window. SQLite `~/.scriptkit/db/ai-chats.sqlite` remains the conversation store and theme stays synced from Script Kit.

## Files

- `src/ai/acp/view.rs` - Main ACP chat view
- `src/ai/acp/chat_window.rs` - Detached ACP window lifecycle
- `src/ai/tab_context.rs` - Compatibility-named ACP context types
- `src/ai/acp/config.rs` - ACP agent configuration
- `src/ai/acp/client.rs` - ACP client runtime

## Features

- Streaming responses
- Markdown rendering
- Model picker
- Chat history sidebar
- Multi-provider support
- BYOK (Bring Your Own Key)

## API Keys (via environment)

- `SCRIPT_KIT_ANTHROPIC_API_KEY`
- `SCRIPT_KIT_OPENAI_API_KEY`
- `SCRIPT_KIT_VERCEL_API_KEY` (for Vercel AI Gateway)

Set in shell profile or system environment.

## Testing

- stdin: `{"type":"openAi"}` (legacy compatibility command that opens ACP Chat)
- log filter: `grep -i 'ai|chat|PANEL'`

## Open Methods

- Hotkey: `Cmd+Shift+Space` (configurable `aiHotkey`)
- Tray menu
- Stdin

## Single-Instance Pattern

Global detached-window handle is coordinated through `src/ai/acp/chat_window.rs`.
