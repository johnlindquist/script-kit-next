# Getting Started with Script Kit GPUI

Script Kit GPUI turns macOS into a scriptable command center: run TypeScript scripts, search built-in utilities, chat with agents, dictate text, and expose desktop context over MCP — all from one launcher.

## Prerequisites

- **macOS** (Linux/Windows support planned)
- **Rust** (1.70+) — https://rustup.rs
- **Bun** — https://bun.sh
- Optional: `cargo-watch` for hot reload while developing the app:

```bash
cargo install cargo-watch
```

## 1. Build and Run

```bash
git clone https://github.com/johnlindquist/script-kit-gpui.git
cd script-kit-gpui
cargo build --release
./target/release/script-kit-gpui
```

For contributor work, use the hot-reload loop instead:

```bash
./dev.sh
```

Script Kit stores user data under `~/.scriptkit/`. Personal scripts live in the default plugin:

```bash
mkdir -p ~/.scriptkit/plugins/main/scripts
```

## 2. The Main Hotkey

The default launcher hotkey is `⌘;` (Cmd+Semicolon). To customize it, create `~/.scriptkit/config.ts`:

```typescript
export default {
  hotkey: {
    modifiers: ["meta"],   // "meta", "ctrl", "alt", "shift"
    key: "Semicolon",      // key codes: "KeyK", "Digit0", "Semicolon", ...
  },
};
```

The main hotkey understands more than one gesture:

| Gesture | Result |
| --- | --- |
| Tap | Toggle the launcher |
| Hold (~250 ms) | Open the Day Page (today's diary/notes surface) |
| Double-tap | Open Agent Chat |

## 3. Your First Script

Create `~/.scriptkit/plugins/main/scripts/hello.ts`:

```typescript
import "@scriptkit/sdk";

export const metadata = {
  name: "Hello World",
  description: "My first script",
};

const name = await arg("What's your name?");
await div(`<h1 class="text-4xl p-8">Hello, ${name}!</h1>`);
```

Press the hotkey, type `hello`, and press `Enter`. Script Kit picks up new files automatically; if a script doesn't appear, run **Reload Scripts** from the actions menu (`⌘K`).

## 4. The Core Workflow

1. **Hotkey** — press `⌘;` from anywhere.
2. **Search** — type a script name, built-in, app, or task. Fuzzy matching applies.
3. **Run** — press `Enter` on the selected row.

Along the way:

- `⌘K` opens the **actions menu** for the selected row (edit, logs, copy path, and more).
- `⌘I` toggles an info panel about the selected item.
- `Tab` on typed text asks AI about it (**Quick AI**); `Tab` on an empty input opens the working-directory picker.
- If nothing matches, fallback rows appear: **Do in Current App**, **Search Files**, **Ask AI**, open-as-URL/file, and an inline calculator for math input.

## 5. Use Bun Packages

Utilities aren't bundled — install what you need:

```bash
cd ~/.scriptkit
bun add zod lodash-es date-fns
```

Then `import` them normally inside scripts. See [SDK Scripting](./sdk-scripting.md) for the full prompt API and metadata reference.

## 6. Pick Your Next Guide

- Big picture and hidden features: [Feature Tour](./feature-tour.md)
- Every launcher input mode and sigil: [Main Menu Input](./main-menu-input.md)
- Writing real scripts: [SDK Scripting](./sdk-scripting.md)
- Agents, context, and MCP: [MCP and Agent Context](./mcp-and-agent-context.md)
- Voice input: [Dictation](./dictation.md)
- Safe window observation: [Computer Use](./computer-use.md)

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| Launcher doesn't open | Confirm the app is running and your `hotkey` config saved to `~/.scriptkit/config.ts`. |
| Script doesn't appear | Save it under `~/.scriptkit/plugins/main/scripts/` and check the file extension (`.ts`, `.js`, `.tsx`, `.jsx`). |
| Imports fail at runtime | Install dependencies in `~/.scriptkit` with `bun add <package>`. |
| MCP/agent examples fail | Start Script Kit first — the live discovery file `~/.scriptkit/server.json` only exists while the app runs. |
| Dictation unavailable | Search **Dictation Setup** in the launcher and check model, microphone permission, and device readiness. |
