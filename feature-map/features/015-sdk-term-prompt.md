# 015 SDK TermPrompt / term() / Terminal Actions / Full-height Terminal

This chapter maps the SDK-spawned terminal prompt surface created by `term()` scripts.

Raw Oracle reference: [answer](../raw-oracle/015-sdk-term-prompt/answer.md), [prompt](../raw-oracle/015-sdk-term-prompt/prompt.md), [bundle map](../raw-oracle/015-sdk-term-prompt/bundle-map.md), [full log](../raw-oracle/015-sdk-term-prompt/output.log), [session metadata](../raw-oracle/015-sdk-term-prompt/session.json).

## Executive Summary

SDK `term()` is the Script Kit SDK terminal prompt. TypeScript/Bun scripts call `globalThis.term(command?: string, actions?: Action[]): Promise<string>` from `scripts/kit-sdk.ts`; Rust routes that request through the prompt handler and renders it as `AppView::TermPrompt { id, entity }`.

SDK TermPrompt shares the `TermPrompt` implementation with Quick Terminal for PTY/Alacritty rendering, input, theme adaptation, terminal actions, scrollback, and output capture. It does not share Quick Terminal route identity, compact sizing, native footer ownership, warm pool, or apply-back behavior.

| Surface | Route | Sizing | Footer | Primary behavior |
|---|---|---|---|---|
| SDK TermPrompt | `AppView::TermPrompt { id, entity }` | Full `ViewType::TermPrompt` height. | GPUI terminal hint strip / prompt-owned footer behavior. | SDK `term()` command or interactive shell, terminal actions, output string return. |
| Quick Terminal | `AppView::QuickTerminalView` | Compact launcher height. | Native `quick_terminal` surface. | Launcher utility terminal, warm PTY pool, cwd handoff, optional Tab AI apply-back. |
| ACP Chat | `AppView::AcpChatView` | Chat layout. | Chat/composer-owned. | Agent threads, composer, slash/mention/context flow. |

## What Users Can Do

| User capability | SDK/API entry | Result |
|---|---|---|
| Run a shell command. | `await term("ls")` | Opens a full-height terminal prompt, runs the command, and resolves with captured terminal output. |
| Print and capture output. | `await term('echo "hello"')` | Output appears in the terminal and is returned as a string when the prompt closes/submits. |
| Render ANSI output. | `await term('echo -e "\\x1b[31mRed\\x1b[0m"')` | Terminal adapter renders ANSI/control sequences in the terminal grid. |
| Capture multi-line output. | `await term('for i in 1 2 3; do echo "Line $i"; done')` | Multi-line terminal output is captured for the promise result. |
| Open an interactive shell. | `await term()` | Opens a terminal with no command; user interacts until close/submit. |
| Use SDK-provided actions. | `await term(command, actions)` | Cmd+Shift+K opens SDK actions instead of built-in terminal commands. |
| Use built-in terminal commands. | `await term(command)` with no SDK actions. | Cmd+Shift+K opens built-in terminal actions such as clear, reset, find/search, and scroll commands. |
| Clear terminal. | Terminal focused, Cmd+K. | Runs the terminal clear action. |
| Toggle terminal actions. | Terminal focused, Cmd+Shift+K. | Opens/closes `ActionsDialogHost::TermPrompt`. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| SDK `term()` | TypeScript SDK global for terminal prompts. | Sends a terminal prompt request with prompt id, optional command, and optional serialized actions. |
| `AppView::TermPrompt` | Rust route for SDK-spawned terminal prompts. | Full-height prompt child content, not Quick Terminal and not ACP. |
| `TermPrompt::with_height` | Terminal prompt constructor. | Creates themed command or interactive terminal handles and receives full terminal content height. |
| `TerminalHandle::with_command_and_theme` | Command terminal creation. | Runs a command in a themed PTY/Alacritty terminal. |
| `TerminalHandle::new_with_theme` | Interactive shell terminal creation. | Opens a themed shell when no command is supplied. |
| `TermPromptActionsMode` | Renderer decision for terminal actions. | SDK actions win when present; otherwise built-in terminal commands are shown. |
| `ActionsDialogHost::TermPrompt` | Actions host for terminal prompt actions. | Keeps terminal action routing and focus restore separate from launcher and ACP actions. |
| Terminal GPUI hint strip | SDK terminal footer/hint behavior. | SDK TermPrompt must not register a native `quick_terminal` footer surface. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| SDK `term(command?)`. | Script calls the global SDK helper. | Sends terminal prompt request and waits for a string result. |
| SDK `term(command?, actions)`. | Script passes SDK actions with the terminal. | Serializes action handlers and uses SDK actions mode. |
| Rust prompt handler. | Receives terminal prompt request. | Computes terminal content height, creates `TermPrompt`, installs `AppView::TermPrompt`, and focuses terminal. |
| Terminal renderer. | `AppView::TermPrompt` renders. | Draws terminal grid, hint strip/footer slot, and terminal action behavior. |
| Actions dialog. | Cmd+Shift+K or terminal action toggle. | Opens `ActionsDialogHost::TermPrompt`. |
| Automation element collection. | `getElements` or layout collection. | Collects SDK terminal elements with `term` semantic prefix and `term-prompt` surface identity. |

## User Workflows

### Run A Command And Receive Output

A script calls:

```ts
const output = await term('echo "SDK_TERM_TEST_OUTPUT_12345"')
```

The SDK creates a prompt id, sends a terminal request, and Rust creates a full-height `AppView::TermPrompt`. `TermPrompt` starts the command through `TerminalHandle::with_command_and_theme`. Output appears in the terminal grid and is returned as a string when the prompt closes/submits.

`tests/sdk/test-term.ts` covers `ls`, echo output, ANSI color output, exit-zero command behavior, a non-zero intermediate command followed by output, multi-line output, and no-command interactive terminal behavior.

### Open An Interactive Shell

A script calls:

```ts
const output = await term()
```

The prompt handler calls `TermPrompt::with_height` with no command, which opens an interactive shell through `TerminalHandle::new_with_theme`. The user types into the terminal until closing/submitting. The result is the captured terminal session output.

### Use SDK Actions

A script calls `term(command, actions)`. The SDK serializes the actions and stores handlers. The renderer sees SDK actions and sets `TermPromptActionsMode::SdkActions`. Cmd+Shift+K opens the actions dialog with script-provided actions, and selecting an action routes back through the SDK action handler.

### Use Built-in Terminal Commands

When no SDK actions are present, `term_prompt_actions_mode(false)` resolves to built-in terminal commands. The terminal actions dialog includes terminal operations such as find/search, scroll to top, scroll to bottom, clear, reset, and action toggle.

### Clear Terminal

Cmd+K matches `is_term_prompt_clear_shortcut(has_cmd, has_shift, key)` when the command/platform modifier is down, Shift is not down, and the key is `K`. The renderer executes the `clear` terminal action. If the actions popup is open, the clear path closes it afterward.

### Toggle Terminal Actions

Cmd+Shift+K matches `is_term_prompt_actions_toggle_shortcut`. The renderer calls `toggle_term_prompt_actions(actions_mode, cx, window)`, and the actions dialog is hosted as `ActionsDialogHost::TermPrompt`.

### Automate The SDK Terminal Surface

Automation should identify SDK TermPrompt with the `term` semantic prefix and `term-prompt` surface identity. Quick Terminal uses `quick-terminal`, so agents must not reuse Quick Terminal semantic ids when targeting SDK `term()`.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open SDK terminal with command. | `await term("ls")`. | SDK prompt route pending. | Script call. | `globalThis.term` -> prompt handler -> `TermPrompt::with_height`. | Full-height `AppView::TermPrompt` opens and runs command. | `tests/sdk/test-term.ts`; `src/prompt_handler/mod.rs`. |
| Open interactive shell. | `await term()`. | No command supplied. | Script call. | `TermPrompt::with_height` -> `TerminalHandle::new_with_theme`. | Shell opens in terminal prompt. | `tests/sdk/test-term.ts`; `src/term_prompt/mod.rs`. |
| Run command with active theme. | SDK command terminal. | Terminal spawning. | Script call. | `TerminalHandle::with_command_and_theme`. | Command PTY uses active Script Kit theme. | `tests/quick_terminal_contracts.rs`. |
| Keep full terminal height. | SDK TermPrompt. | Prompt view. | Render/resize. | `ViewType::TermPrompt => max_height`; prompt route uses `MAX_HEIGHT - FOOTER_HEIGHT` content height. | SDK terminal does not collapse to Quick Terminal size. | `src/window_resize/mod.rs`; `tests/source_audits/resize_presentation_contract.rs`. |
| Avoid native Quick Terminal footer. | SDK TermPrompt. | Footer render. | N/A. | No native footer surface for `AppView::TermPrompt`; non-quick terminal uses terminal hint strip. | SDK terminal keeps GPUI terminal footer/hints. | `tests/quick_terminal_contracts.rs`; `tests/main_window_footer_surface_owner_contract.rs`. |
| Clear terminal. | TermPrompt focused. | Terminal active or actions open. | Cmd+K. | `is_term_prompt_clear_shortcut` -> `execute_term_prompt_action_by_id("clear")`. | Terminal clear action executes. | Renderer/action tests. |
| Toggle actions. | TermPrompt focused. | Terminal active. | Cmd+Shift+K. | `is_term_prompt_actions_toggle_shortcut` -> `toggle_term_prompt_actions`. | Actions dialog opens/closes. | Renderer/action tests. |
| Use SDK actions. | `term(command, actions)`. | SDK actions present. | Cmd+Shift+K, select action. | `term_prompt_actions_mode(true) => SdkActions`. | SDK-provided action list owns palette. | `src/render_prompts/term.rs`. |
| Use terminal commands. | `term(command)` with no actions. | No SDK actions. | Cmd+Shift+K. | `term_prompt_actions_mode(false) => TerminalCommands`. | Built-in terminal commands show. | `src/render_prompts/term.rs`. |
| Restore focus after actions. | Actions dialog closes. | Dialog host TermPrompt. | Close/backdrop/activation. | `ActionsDialogHost::TermPrompt => FocusRequest::term_prompt()`. | Terminal focus restored. | `src/app_impl/actions_dialog.rs`. |
| Identify terminal in automation. | Agent state/elements call. | `AppView::TermPrompt`. | Protocol call. | `collect_term_prompt_elements(term, "term", limit)`. | Elements have SDK terminal semantic identity. | `src/app_layout/collect_elements.rs`. |
| Distinguish Quick Terminal. | Launcher `>` / utility route. | `QuickTerminalView`. | Utility command. | `AppView::QuickTerminalView`, semantic prefix `quick-terminal`. | Compact terminal, native footer, separate identity. | [014 Quick Terminal](./014-quick-terminal-pty.md). |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| Idle script. | Script has not called `term()`. | No terminal prompt exists. | SDK continues normal script execution. |
| SDK request created. | `globalThis.term` called. | Creates prompt id, optional serialized actions, terminal request. | Promise waits for terminal result. |
| Prompt handler receives request. | App receives terminal prompt message. | Builds terminal submit callback and computes content height. | Content height is `MAX_HEIGHT - FOOTER_HEIGHT`. |
| Terminal entity creation. | `TermPrompt::with_height`. | Command creates `with_command_and_theme`; no command creates `new_with_theme`. | Initial grid starts around 80x24 and resizes to render size. |
| View installed. | Entity creation succeeds. | `current_view = AppView::TermPrompt { id, entity }`; focus goes to terminal. | `focused_input = None`; `pending_focus = TermPrompt`. |
| Rendered and focused. | GPUI renders current view. | `render_term_prompt` draws terminal and hint/footer strip. | Actions mode depends on SDK actions presence. |
| Interactive/running. | User/script terminal session active. | Terminal input/output/scroll/actions operate. | Output accumulates for return value. |
| Actions open. | Cmd+Shift+K or action toggle. | `ActionsDialogHost::TermPrompt`; terminal keys suppressed while dialog owns focus. | Focus restores to terminal on close. |
| Close/submit. | User closes or terminal submits. | Submit callback resolves SDK promise with terminal output string. | View transitions out of `AppView::TermPrompt`. |
| Stale async guard. | Deferred resize/focus fires after view changes. | Expected id check blocks stale mutation. | Prevents old terminal prompt work affecting newer views. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| SDK terminal open. | Full-height terminal prompt. | `FocusTarget::TermPrompt`. | `AppView::TermPrompt`; prompt type `term`; surface `term-prompt`. |
| Command running. | Terminal grid shows command output. | Terminal PTY. | Terminal elements under `term` prefix. |
| Interactive shell. | Shell prompt in full terminal. | Terminal PTY. | `term-prompt` surface identity. |
| SDK actions mode. | Actions dialog with script-provided actions. | Actions dialog. | `ActionsDialogHost::TermPrompt`; SDK actions present. |
| Built-in terminal commands mode. | Actions dialog with terminal commands. | Actions dialog. | `TermPromptActionsMode::TerminalCommands`. |
| Quick Terminal open. | Compact terminal utility. | Terminal PTY. | `QuickTerminalView`; prefix `quick-terminal`; native footer `quick_terminal`. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Printable text | TermPrompt focused, actions closed. | Expected PTY input; exact dispatch path should be verified in full renderer/input source. |
| Enter | TermPrompt focused, actions closed. | Expected shell/PTY Enter. |
| Tab / Shift+Tab | TermPrompt focused, actions closed. | Expected terminal keystrokes unless an overlay owns focus; exact SDK physical/protocol parity needs expanded source proof. |
| Ctrl chords | TermPrompt focused. | Expected shell control sequences. Automated Ctrl+C coverage is limited; manual verification recommended. |
| Escape | Actions dialog open. | Expected overlay dismissal. |
| Escape | Terminal focused, actions closed. | Exact SDK TermPrompt behavior needs expanded source confirmation. |
| Cmd+C | Terminal selection/copy path. | Expected terminal copy/selection behavior; verify exact implementation before changing. |
| Cmd+V | Terminal focused. | Expected paste into PTY; clipboard text enters local shell. |
| Cmd+K | TermPrompt focused. | Clear terminal via `TERM_PROMPT_CLEAR_ACTION_ID = "clear"`. |
| Cmd+Shift+K | TermPrompt focused. | Toggle terminal actions via `TERM_PROMPT_ACTIONS_TOGGLE_ACTION_ID`. |
| Cmd+W | SDK TermPrompt. | Do not assume Quick Terminal state-first close; use normal prompt/window close semantics unless source proves otherwise. |

## Actions And Menus

| Actions mode | When active | What appears | Owner |
|---|---|---|---|
| SDK actions | `term(command, actions)` supplies actions. | Script-provided actions. | SDK handlers and `ActionsDialogHost::TermPrompt`. |
| Terminal commands | No SDK actions supplied. | Find/search, scroll top/bottom, clear, reset, toggle actions. | Built-in terminal action builder. |

Built-in terminal actions are sorted/categorized in `src/app_impl/actions_toggle.rs`. Clear uses Cmd+K and appears in the Session category. Toggle Actions uses Cmd+Shift+K and opens/closes the terminal actions palette.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| Current prompt metadata. | `AppView::TermPrompt` maps to prompt type `term` with current prompt id where available. |
| `getElements`. | SDK terminal elements are collected through `collect_term_prompt_elements(term, "term", limit)`. |
| Surface identity. | SDK terminal final surface identity is `term-prompt`. |
| Quick Terminal distinction. | Quick Terminal uses `quick-terminal` semantic prefix and `QuickTerminalView`. |
| Surface kind. | `AppView::TermPrompt` is `SurfaceKind::PromptChildContent`, not launcher list or ACP chat. |
| Footer ownership matrix. | SDK TermPrompt must remain non-native-footer-owned; Quick Terminal owns native `quick_terminal`. |
| Actions host. | Terminal actions use `ActionsDialogHost::TermPrompt`. |

## Data, Storage, And Privacy Boundaries

- `term()` returns a terminal output string, not ordinary prompt text.
- Terminal output can include command output, shell prompts, ANSI output, multi-line text, and local command data.
- Commands run in a local PTY subprocess and can access local files, environment, shell config, and cwd according to the terminal process setup.
- Clipboard paste injects clipboard text into the PTY/local shell.
- Clipboard copy can place terminal selection/output on the clipboard.
- SDK actions serialize action metadata and handler ids across the SDK/app boundary.
- Built-in terminal commands do not require SDK action handler storage.
- PTY subprocess cleanup belongs to terminal lifecycle code; verify cleanup before changing close semantics.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Terminal creation failure. | `TermPrompt::with_height` returns an error; prompt handler must not install a broken terminal view. Exact error UI needs source expansion. |
| Command with no output. | Empty string output can be valid and should not be treated as failure. |
| Non-zero command. | Intermediate `false; echo ...` path is tested as successful output; final exit-code rejection/resolve behavior needs separate proof. |
| ANSI output. | Terminal should render ANSI sequences; return-string normalization needs source expansion. |
| Interactive no-command. | Valid terminal session; output may be zero or non-zero depending on shell/user activity. |
| Actions dialog open. | `term.suppress_keys = show_actions` prevents terminal key handling from fighting the dialog. |
| Stale deferred resize/focus. | Expected id guard prevents stale async work from touching a newer prompt. |

## Code Ownership

| Area | Files |
|---|---|
| SDK API | `scripts/kit-sdk.ts` |
| Prompt route/current prompt metadata | `src/prompt_handler/mod.rs` |
| Terminal prompt model | `src/term_prompt/mod.rs` |
| Terminal renderer/actions shortcuts | `src/render_prompts/term.rs` |
| Built-in terminal actions | `src/app_impl/actions_toggle.rs` |
| Actions host/focus restore | `src/app_impl/actions_dialog.rs` |
| Element collection | `src/app_layout/collect_elements.rs` |
| Surface kind/footer ownership | `src/main_sections/app_view_state.rs` |
| Sizing | `src/window_resize/mod.rs` |
| Terminal/PTY lifecycle | `src/terminal/`, `src/terminal/pty/`, `src/terminal/alacritty/` |
| SDK tests | `tests/sdk/test-term.ts` |
| Footer/Quick Terminal separation tests | `tests/quick_terminal_contracts.rs`, `tests/main_window_footer_surface_owner_contract.rs` |
| Resize/source audit | `tests/source_audits/resize_presentation_contract.rs` |

## Invariants And Regression Risks

- Do not collapse SDK TermPrompt into Quick Terminal.
- Do not apply compact Quick Terminal height to SDK TermPrompt.
- `ViewType::TermPrompt` must remain full-height.
- Prompt route content height should remain `MAX_HEIGHT - FOOTER_HEIGHT`.
- Do not register SDK TermPrompt as a native footer surface.
- SDK TermPrompt must keep the GPUI terminal hint strip.
- Do not give SDK TermPrompt Quick Terminal apply-back behavior.
- Cmd+K must remain Clear.
- Cmd+Shift+K must remain Toggle Actions.
- SDK actions must override built-in terminal commands only when present.
- Actions host must remain `ActionsDialogHost::TermPrompt`.
- SDK TermPrompt semantic prefix must remain `term`; Quick Terminal prefix must remain `quick-terminal`.
- Terminal creation must pass the active theme.
- Deferred resize/focus must stay id-guarded.
- ACP Chat must not be treated as a terminal.

## Verification Recipes

Targeted checks recommended by Oracle:

```bash
cargo test quick_terminal_native_footer_does_not_capture_sdk_term_prompt_footer
cargo test quick_terminal_keyboard_and_footer_close_share_state_first_close
cargo test --test main_window_footer_surface_owner_contract
cargo test --test resize_presentation_contract
cargo test term_prompt_actions_mode
cargo test test_term_prompt_clear_shortcut_matches_cmd_k_without_shift
cargo test test_term_prompt_actions_toggle_shortcut_matches_cmd_shift_k
cargo test test_terminal_actions_for_dialog_shows_cmd_k_for_clear_terminal
cargo test test_terminal_actions_for_dialog_adds_cmd_shift_k_toggle_shortcut
bun tests/sdk/test-term.ts
lat check
```

Manual/runtime proof checklist:

1. Run `await term("echo hello")`; assert full-height terminal opens, output appears, close resolves output.
2. Run `await term()`; assert interactive shell opens and basic typing/Enter reaches the shell.
3. Run ANSI output command; assert terminal renders color and output is captured.
4. Run `term(command, actions)`; assert Cmd+Shift+K shows SDK actions.
5. Run `term(command)` without actions; assert Cmd+Shift+K shows built-in terminal commands and Cmd+K clears.
6. Open Quick Terminal separately; assert compact height, native footer, and `quick-terminal` automation identity, not SDK `term-prompt`.
7. Call `getState`/`getElements` while SDK TermPrompt is open; assert `term`/`term-prompt` identity.

## Agent Notes

Treat SDK TermPrompt as a prompt runtime surface, not a launcher utility surface. Shared `TermPrompt` changes may affect both SDK TermPrompt and Quick Terminal; route identity, footer, sizing, apply-back, warm PTY, and Quick Terminal path behavior must remain separated.

When changing footer code, run the footer owner contract tests. The likely regression is accidentally registering `AppView::TermPrompt` as native-footer-owned or making SDK TermPrompt render Quick Terminal native footer chrome.

When changing window sizing, keep `ViewType::TermPrompt => max_height`. Quick Terminal compact sizing is intentionally outside the generic TermPrompt branch.

When changing actions, preserve `ActionsDialogHost::TermPrompt`, Cmd+K Clear, and Cmd+Shift+K Toggle Actions.

When changing automation, keep `term` and `quick-terminal` semantic prefixes distinct.

## Related Features

- [014 Quick Terminal PTY](./014-quick-terminal-pty.md) shares terminal implementation but owns compact launcher terminal, warm pool, native footer, cwd handoff, and apply-back.
- [004 MCP / SDK / Protocol Automation](./004-mcp-sdk-protocol.md) owns broader SDK/protocol proof surfaces.
- ACP Chat is adjacent only as a separate AI/chat surface; it should not inherit terminal behavior.
- Editor Prompt is adjacent through full-height prompt child sizing.
- Terminal/PTY lifecycle owns local shell/process behavior, theme adaptation, scrollback, copy/paste, and terminal rendering.

## Open Questions And Gaps

- Exact SDK JSON/protocol field names for `term()` need a wider source pass through `scripts/kit-sdk.ts` and protocol handlers.
- Returned output normalization is not fully established: ANSI preservation, stripping, and rendered/plain-text buffer behavior need source/runtime proof.
- Final exit-code semantics are not fully proven. The visible non-zero test uses `false; echo ...`, so the shell continues.
- Printable key, Tab, Shift+Tab, Ctrl, Escape, Cmd+C, and Cmd+V dispatch details need expanded renderer/input source before stronger claims.
- Terminal lifecycle cleanup on close should be confirmed in `src/terminal/pty/lifecycle.rs` before changing close semantics.
- Copy/paste and selection edge behavior should be verified in full terminal renderer/input and Alacritty adapter source.
