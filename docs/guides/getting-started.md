# Getting Started with Script Kit GPUI

Script Kit GPUI is one window you tell what should happen: run TypeScript scripts, search built-in utilities, say a sentence and let it do the steps, chat with agents, dictate text, and expose desktop context over MCP — all under one input that never moves.

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
| Tap | Open the launcher; a tap while it's open hides it |
| Hold (~250 ms) | Open the Day Page (today's diary/notes surface) |
| Double-tap | Open Agent Chat under the same input; a double-tap while it's open closes it |

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

Press the hotkey, type `hello`, and press `Enter`. The prompt opens under the same input. Script Kit picks up new files automatically; if a script doesn't appear, run the **Reload Scripts** command.

## 4. The Core Workflow

1. **Hotkey** — press `⌘;` from anywhere.
2. **Say it** — type a script name, built-in, app, or a sentence saying what should happen. Fuzzy matching applies, and Jev lists up to three readings of a sentence under a "Jev · ⌘1–3" header.
3. **Run** — press `Enter`. The ↵ label always names exactly what it will run.

Along the way:

- `⌘K` stages the selected row in the line above the input, and the list becomes its actions (edit, logs, copy path, and more). `Esc` lets go.
- `⌘I` toggles an info panel about the selected item.
- `Tab` on typed text hands the words to an agent (Tab again takes them back); `Tab` on an empty input opens the working-directory picker.
- Nothing asks "Are you sure?": something that can't be undone waits 8 s in the line ("Runs in 8s") and `⌘Z` stops it; something that can be undone just happens, and `⌘Z` takes it back.
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
| Snippets don't expand, or text is copied instead of pasted | Search **Set Up Permissions** and turn on Accessibility. See [Permissions](./permissions.md). |
