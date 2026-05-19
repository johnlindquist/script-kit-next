# Getting Started with Script Kit GPUI

Script Kit GPUI turns macOS into a scriptable command center: run TypeScript, open built-in utilities, chat with agents, dictate text, inspect UI state, and expose context over MCP.

## What You Will Build

By the end of this guide you will have:

- a local Script Kit GPUI build
- a global hotkey
- a first TypeScript script
- a map of the built-ins worth trying next

## Prerequisites

- macOS
- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- Optional: `cargo-watch` for hot reload while developing the app

```bash
cargo install cargo-watch
```

## 1. Build and Run

Clone the repo and start the app:

```bash
git clone https://github.com/johnlindquist/script-kit-gpui.git
cd script-kit-gpui
cargo build --release
./target/release/script-kit-gpui
```

For contributor/dev work, run the hot-reload loop instead:

```bash
./dev.sh
```

Script Kit writes user data under `~/.scriptkit/`. Personal scripts live in the default plugin:

```bash
mkdir -p ~/.scriptkit/plugins/main/scripts
mkdir -p ~/.scriptkit/plugins/main/scriptlets
```

## 2. Add a Hotkey

Create `~/.scriptkit/config.ts`:

```ts
export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },
};
```

Restart Script Kit, then press `Cmd+;` to open the launcher.

## 3. Create Your First Script

Create `~/.scriptkit/plugins/main/scripts/hello.ts`:

```ts
import "@scriptkit/sdk";

export const metadata = {
  name: "Hello World",
  description: "Ask for a name and render a friendly response",
};

const name = await arg("What's your name?");
const safeName = name.replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);

await div(`
  <div class="p-8">
    <h1 class="text-4xl font-bold">Hello, ${safeName}!</h1>
    <p class="mt-4 opacity-80">This UI came from a TypeScript script.</p>
  </div>
`);
```

Open the launcher, type `hello`, press `Enter`, and run the prompt.

## 4. Learn the Launcher

The launcher searches scripts and built-ins together. Try these queries:

| Type | What to try |
| --- | --- |
| `new script` | Create a blank script |
| `template` | Browse starter templates |
| `sdk` | Open the in-app SDK Reference |
| `clipboard` | Browse clipboard history |
| `files` | Search or browse files |
| `tabs` | Search open browser tabs |
| `notes` | Open the Markdown notes window |
| `terminal` | Open Quick Terminal |
| `dictation` | Open dictation setup/history/actions |
| `current app` | Search or automate the frontmost app's menu commands |

The first surprising idea: Script Kit is not only "run a script." It is also a searchable UI for system utilities, browser tabs, notes, local context, script templates, and Agent Chat.

For the full launcher grammar, including `~`, `/`, `@`, `:`, source heads such as `files:`, capture syntax such as `;todo`, and command syntax, see [Main Menu Input](./main-menu-input.md).

## 5. Open Agent Chat

Press `Tab` from the launcher to open Agent Chat with current context staged. If the launcher input contains text, plain `Tab` can submit that text as the first Agent Chat turn.

In Agent Chat, context can come from:

- the current launcher surface
- selected text
- browser/window state
- files
- notes
- clipboard history
- dictation history
- MCP resources such as `kit://context`

See [MCP and Agent Context](./mcp-and-agent-context.md) for the full model.

## 6. Use the SDK Reference

Search `sdk` and open **SDK Reference**. It is generated from the same data as the `kit://sdk-reference` MCP resource, so the in-app docs and agent-facing docs stay aligned.

Useful APIs to look up first:

- `arg`, `div`, `editor`, `fields`, `form`, `path`, `drop`, `hotkey`, `term`
- `getState`, `getElements`, `waitFor`, `batch`
- `mcp.listServers`, `mcp.listTools`, `mcp.call`
- `computer.listNativeWindows`, `computer.captureNativeWindow`
- `hud` and `notify`

## 7. Use the Command Line

Script Kit refreshes an app-managed command shim at `~/.scriptkit/bin/scriptkit` while setting up the workspace. Use it directly, add `~/.scriptkit/bin` to `PATH`, or install a shorter shell command:

```bash
~/.scriptkit/bin/scriptkit --help
~/.scriptkit/bin/scriptkit install-command ~/.local/bin/scriptkit
scriptkit mcp tools
```

The MCP subcommands require Script Kit to be running because they read the live server discovery file at `~/.scriptkit/server.json`.

## 8. Pick Your Next Guide

- Want the big picture? Read [Feature Tour](./feature-tour.md).
- Want every launcher input mode? Read [Main Menu Input](./main-menu-input.md).
- Want to write real scripts? Read [SDK Scripting](./sdk-scripting.md).
- Want AI tools and context? Read [MCP and Agent Context](./mcp-and-agent-context.md).
- Want voice input? Read [Dictation](./dictation.md).
- Want safe screen/window observation? Read [Computer Use](./computer-use.md).

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| Launcher does not open | Confirm the app is running and the `hotkey` config was saved to `~/.scriptkit/config.ts`. |
| Script does not appear | Save it under `~/.scriptkit/plugins/main/scripts/` and make sure it has a `.ts`, `.js`, `.tsx`, or `.jsx` extension. |
| Script runs but imports fail | Install dependencies in `~/.scriptkit` with `bun add <package>`. |
| Agent/MCP examples fail | Start Script Kit first. The live MCP server discovery file is created at `~/.scriptkit/server.json` while the app is running. |
| Dictation is unavailable | Open **Dictation Setup** from the launcher and check model, microphone permission, and input device readiness. |
