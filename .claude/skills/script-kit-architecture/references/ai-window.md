# AI Window

Separate floating BYOK chat window. SQLite `~/.scriptkit/db/ai-chats.sqlite`. Theme synced from Script Kit theme.

## Files

- `window.rs` - Main view
- `storage.rs` - SQLite persistence
- `model.rs` - `Chat`, `Message`, `ChatId`, roles
- `providers.rs` - Anthropic/OpenAI implementations
- `config.rs` - Environment detection

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

- stdin: `{"type":"openAi"}`
- log filter: `grep -i 'ai|chat|PANEL'`

## Open Methods

- Hotkey: `Cmd+Shift+Space` (configurable `aiHotkey`)
- Tray menu
- Stdin

## Single-Instance Pattern

Global `OnceLock<Mutex<Option<WindowHandle<Root>>>>`.
