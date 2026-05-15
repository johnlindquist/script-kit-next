# Feature Tour

Script Kit GPUI looks like a launcher, but it is also a script runtime, AI context hub, local MCP server, dictation system, notes app, terminal surface, and automation harness.

## The Mental Model

Think of Script Kit as four layers that work together:

1. **Launcher** — search scripts, built-ins, apps, files, tabs, notes, and utilities.
2. **SDK** — build custom TypeScript prompts and automations with Bun.
3. **Agent Chat + MCP** — give agents structured context, resources, and tools.
4. **Native macOS capabilities** — dictation, computer-use observation, windows, menus, clipboard, notes, and notifications.

## Built-ins Worth Trying

| Search for | What it opens |
| --- | --- |
| `clipboard` | Clipboard History with previews, OCR metadata, and actions |
| `files` | File Search and Recent Files |
| `tabs` | Browser Tabs from supported running browsers |
| `history` | Browser History or Agent Chat history, depending on context |
| `notes` | Floating Markdown notes and note search |
| `emoji` | Emoji picker |
| `apps` | App launcher |
| `process` | Script process manager |
| `terminal` | Quick Terminal |
| `theme` | Theme chooser |
| `settings` | Settings, microphone selection, dictation setup, window reset |
| `sdk` | SDK Reference generated from the live API catalog |
| `template` | New Script from Template |
| `current app` | Frontmost app menu search and current-app automation |

Suggested default rows include Agent Chat, Do in Current App, New Script, Clipboard History, Open Notes, Search Files, Search Browser Tabs, Quick Terminal, and SDK Reference.

## Agent Chat Is Built In

Press `Tab` from the launcher to open Agent Chat. It can stage the active launcher surface as context, reuse a detached chat window, attach files and resources, and hand off focused targets from surfaces like file search or current-app commands.

Useful context commands include:

| Command | Context |
| --- | --- |
| `/context` | Minimal desktop context |
| `/context-full` | Full desktop context |
| `/selection` | Selected text |
| `/browser` | Browser URL |
| `/window` | Focused window info |

Agent Chat setup goes through the Agent Catalog and `~/.scriptkit/config.ts`; direct-provider API key commands are legacy and are not the primary Agent Chat setup path.

## MCP Is a First-Class Surface

Script Kit runs its own local HTTP MCP server while the app is active. It exposes resources such as:

- `kit://context`
- `kit://context/schema`
- `kit://sdk-reference`
- `kit://script-templates`
- `kit://scripts`
- `kit://scriptlets`
- `kit://clipboard-history`
- `kit://dictation`
- `kit://dictation-history`
- `kit://focused-item`
- `kit://state`

Scripts and Agent Chat can also use configured external MCP servers through the SDK's `mcp.*` helpers.

See [MCP and Agent Context](./mcp-and-agent-context.md).

## Dictation Is More Than Recording Audio

Dictation can target active Script Kit surfaces and feeds a persistent history:

- open **Dictation Setup** to check model and microphone readiness
- use **Select Microphone** to choose the input device
- use **Start Dictation Here** for the active surface
- use **Start Dictation to AI** for Agent Chat quick-submit
- use **Open Dictation History** to search previous transcripts
- attach saved dictation to Agent Chat as `kit://dictation-history?id=...`

See [Dictation](./dictation.md).

## Computer Use Is Observation-First

The SDK exposes `computer.listNativeWindows()` and `computer.captureNativeWindow()` so scripts can inspect and capture exact native macOS windows. These helpers are deliberately observation/capture-only: they do not focus windows, send input, move windows, or resize anything.

See [Computer Use](./computer-use.md).

## The SDK Is Small but Deep

The SDK focuses on UI prompts and integration points:

- prompts: `arg`, `div`, `editor`, `fields`, `form`, `path`, `drop`, `hotkey`, `term`
- feedback: `hud`, `notify`
- automation: `getState`, `getElements`, `waitFor`, `batch`
- MCP client helpers: `mcp.listServers`, `mcp.listTools`, `mcp.discover`, `mcp.call`
- computer-use helpers: `computer.listNativeWindows`, `computer.captureNativeWindow`
- filesystem/clipboard/process helpers listed in the in-app SDK Reference

Because scripts run through Bun, you can bring your own packages with `bun add` inside `~/.scriptkit`.

See [SDK Scripting](./sdk-scripting.md).

## Current-App Automation

Current-app commands inspect the frontmost app's menu structure and let you search actions such as "close tab" or "export." If a direct menu command is not enough, Script Kit can hand the captured current-app context to Agent Chat or script generation.

This is useful for turning repetitive GUI workflows into reusable commands without manually spelunking an app's menus.

## Notes, Terminal, and Utilities

Script Kit includes a floating notes window with Markdown behavior, Quick Terminal for PTY-backed flows, process management for running scripts, settings surfaces, theme controls, window reset tools, and macOS system actions.

## Discovery Tips

- Search broadly: `ai`, `sdk`, `template`, `context`, `dictation`, `history`, `current app`, `terminal`.
- Learn the launcher grammar in [Main Menu Input](./main-menu-input.md): `~` for mini file search, `/` and `@` for Agent Chat pickers, `:` for filters, source heads like `files:`, capture with `;todo`, commands with `!` / `>head`, and Quick Terminal with bare `>`.
- Open **SDK Reference** before guessing an API name.
- Use **New Script from Template** when starting a new script.
- Use Agent Chat when the task needs context from the current surface.
- Use MCP resources when an external agent needs structured read-only state.
