---
name: script-kit-architecture
description: Script Kit GPUI codebase architecture. Use when navigating the codebase, understanding module structure, or tracing data flow. Covers repository structure, SDK deployment, configuration, and secondary windows.
---

# Script Kit Architecture

Overview of the Script Kit GPUI codebase structure.

## Repository Structure (Key Modules)

`src/`
- `main.rs` — app entry, window setup, include!() hub for app_impl/main_sections/render_prompts/etc.
- `lib.rs` — crate exports
- `main_sections/` — app state, render dispatch, window visibility (include!() into main.rs)
- `app_impl/` — app logic: shortcuts, input handling, startup (include!() into main.rs)
- `app_execute/` — builtin execution, script execution (include!() into main.rs)
- `render_prompts/` — outer prompt wrappers: arg, div, editor, form, etc. (include!() into main.rs)
- `prompts/` — inner prompt entities: chat, env, select, term_prompt (proper module)
- `components/` — reusable UI components: text_input, alias_input, shortcut_recorder (proper module)
- `protocol/` — stdin/stdout JSON protocol, message types (proper module)
- `theme/` — theme system, colors, opacity constants, HexColorExt (proper module)
- `ai/` — AI chat window, providers, streaming (proper module)
- `notes/` — Notes window module (proper module)
- `platform/` — macOS ObjC interop, panel config, cursor, visibility (proper module with include!())
- `scripts/` — script loading, execution (proper module)
- `builtins/` — built-in command definitions (proper module)
- `hotkeys/` — global hotkey registration (proper module)
- `watcher/` — file system watchers (proper module)
- `ui_foundation/` — key helpers (is_key_*), layout constants, hex color utils

Logs: `~/.scriptkit/logs/script-kit-gpui.jsonl`

## SDK Deployment Architecture

SDK source: `scripts/kit-sdk.ts`

Two-tier deployment:
1. **Build time (dev):** `build.rs` copies `scripts/kit-sdk.ts` to `~/.scriptkit/sdk/`
2. **Compile time:** `executor.rs` embeds via `include_str!("../scripts/kit-sdk.ts")`
3. **Runtime:** `ensure_sdk_extracted()` writes embedded SDK to `~/.scriptkit/sdk/kit-sdk.ts`
4. **Execution:** `bun run --preload ~/.scriptkit/sdk/kit-sdk.ts <script>`

Tests import `../../scripts/kit-sdk` (repo path). Production uses runtime-extracted SDK.

tsconfig mapping:
```json
{ "compilerOptions": { "paths": { "@scriptkit/sdk": ["./sdk/kit-sdk.ts"] } } }
```

## User Configuration (`~/.scriptkit/config.ts`)

```ts
import type { Config } from "@scriptkit/sdk";
export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  padding: { top: 8, left: 12, right: 12 },
  editorFontSize: 16,
  terminalFontSize: 14,
  uiScale: 1.0,
  builtIns: { clipboardHistory: true, appLauncher: true },
  bun_path: "/opt/homebrew/bin/bun",
  editor: "code"
} satisfies Config;
```

Rust helpers (use these; they handle defaults):
- `config.get_editor_font_size()` (default 14)
- `config.get_terminal_font_size()` (default 14)
- `config.get_padding()` (top 8, left/right 12)
- `config.get_ui_scale()` (default 1.0)
- `config.get_builtins()` (clipboardHistory/appLauncher default true)
- `config.get_editor()` (default `"code"`)

Font sizing:
- Editor: `line_height = font_size * 1.43`
- Terminal: `line_height = font_size * 1.3`

## References

- [System Diagrams](references/diagrams.md) - Architecture, state machine, execution flow diagrams
- [Notes Window](references/notes-window.md) - Notes module details
- [AI Window](references/ai-window.md) - AI chat module details
- [Gotchas](references/gotchas.md) - Common issues and fixes
- [Vibrancy](references/vibrancy.md) - macOS vibrancy/blur patterns
