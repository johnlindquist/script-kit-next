# 014 Quick Terminal PTY / TermPrompt / Warm Pool / Apply-back

This chapter maps the PTY-backed Quick Terminal surface, its terminal child, warm pool, native footer, and apply-back lifecycle.

Raw Oracle reference: [answer](../raw-oracle/014-quick-terminal-pty/answer.md), [prompt](../raw-oracle/014-quick-terminal-pty/prompt.md), [bundle map](../raw-oracle/014-quick-terminal-pty/bundle-map.md), [full log](../raw-oracle/014-quick-terminal-pty/output.log), [session metadata](../raw-oracle/014-quick-terminal-pty/session.json).

## Executive Summary

Quick Terminal is the app's compact PTY-backed terminal surface. It is represented as `AppView::QuickTerminalView`, contains a `TermPrompt` entity, owns terminal input/focus semantics, and is used by the launcher Quick Terminal route, path-based "Open in Quick Terminal", and verification-oriented Tab AI harness flows.

Quick Terminal is not ACP Chat and it is not the SDK `term()` prompt. The important boundaries are:

| Surface | Route | Sizing | Footer | Primary behavior |
|---|---|---|---|---|
| Quick Terminal | `AppView::QuickTerminalView` | Compact launcher terminal height. | Native `quick_terminal` surface. | PTY shell, warm pool, state-first close, optional apply-back. |
| SDK TermPrompt | `AppView::TermPrompt { id, entity }` | Full terminal prompt height. | Prompt-owned; no `quick_terminal` native surface. | SDK `term()` / prompt handler terminal. |
| ACP Chat | `AppView::AcpChatView` | Chat layout. | Chat/composer-owned. | Agent threads, composer, slash/mention/context flow. |

The load-bearing contracts are compact launcher sizing, warm PTY reuse that fails open to cold spawn, terminal-scoped native footer buttons, and apply-back visibility gated by both an apply route and a return view.

## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Open plain Quick Terminal. | Launcher `>` handoff or built-in Quick Terminal route. | Opens `QuickTerminalView`, focuses `TermPrompt`, uses warm PTY if available, and stays compact. |
| Open at a file or directory. | `file:open_in_quick_terminal` action. | Opens Quick Terminal, resolves cwd to directory or file parent, and writes quoted `cd <dir>` to the PTY. |
| Type into a real PTY. | Printable keys, Ctrl chords, terminal special keys, Tab, Shift+Tab, Escape. | Bytes are forwarded to the PTY, not ACP navigation, launcher filter traversal, or global cancel. |
| Close the wrapper. | Cmd+W, protocol `simulateKey` Cmd+W, or native footer Close. | Closes state-first and clears harness/apply-back terminal state. |
| Apply terminal output back. | Native Apply or Cmd+Enter when Apply is visible. | Reads selected terminal text when possible, falls back to clipboard only if needed, and applies to the saved route. |
| Use terminal actions. | Terminal action shortcut/actions dialog. | Opens `ActionsDialogHost::TermPrompt` actions, distinct from launcher actions/footer buttons. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| `AppView::QuickTerminalView` | Top-level launcher utility route that stores a terminal entity. | Owns Quick Terminal lifecycle, native footer surface, compact height, and terminal focus. |
| `TermPrompt` | Terminal renderer/input component used by multiple terminal surfaces. | Quick Terminal contains one, but SDK `term()` also uses one through a different route. |
| `TerminalHandle` / PTY manager | Alacritty/PTY-backed runtime handle. | Cold terminals are themed on creation; attached warm terminals are rethemed. |
| Warm PTY pool | One-slot idle terminal cache on `ScriptListApp`. | Tracks handle, inflight state, and creation time; stale/dead/missing/inflight states cold-spawn. |
| Native footer surface | `AppView::native_footer_surface()` returns `Some("quick_terminal")`. | Applies only to Quick Terminal; SDK `TermPrompt` deliberately has no native `quick_terminal` footer. |
| Apply-back state | Harness route state for sending terminal output back to origin. | `quick_terminal_can_apply_back()` is true only when apply route and return view both exist. |
| Zsh prompt suppression | Spawn-time shell environment/shim behavior. | Uses `PROMPT_EOL_MARK=""` and zsh-only `ZDOTDIR` shim; do not use attach-time clear bytes. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| Launcher `>` special entry. | Main ScriptList filter contains exactly `>`. | Calls `open_quick_terminal(None, cx)`. |
| Built-in Quick Terminal. | Built-in utility command / triggerBuiltin route. | Calls the same Quick Terminal opener without cwd. |
| File/path action. | A file, directory, or path row exposes `file:open_in_quick_terminal`. | Resolves cwd and calls `open_quick_terminal(Some(cwd), cx)`. |
| Tab AI verification harness. | Verification flow needs PTY-backed execution instead of ACP chat. | Saves return view/focus, seeds apply-back route, opens Quick Terminal, then submits/captures terminal output. |
| SDK `term()`. | Script prompt handler creates a terminal prompt. | Creates `AppView::TermPrompt { id, entity }`, not Quick Terminal. |
| Fallback "run in terminal". | Utility fallback command terminal path. | Uses a terminal prompt with full terminal resize while sharing some QuickTerminalView wrapper behavior; do not confuse with compact launcher Quick Terminal. |
| Automation. | Stdin protocol / agentic scripts. | Can trigger Quick Terminal, inspect state/footer ownership, and select `footer:native:close`. |

## User Workflows

### Open From Launcher

The user types `>` or invokes the built-in Quick Terminal command. `open_quick_terminal(None, cx)` attempts to take a fresh, live warm PTY. If one is available, it attaches through `TermPrompt::with_existing_terminal`; otherwise it cold-spawns through `TermPrompt::with_height`.

The app sets `current_view = AppView::QuickTerminalView`, clears ordinary editable input focus, sets `pending_focus = Some(FocusTarget::TermPrompt)`, refills the warm pool, and notifies GPUI. The main window stays at compact Quick Terminal height rather than SDK terminal height.

### Open From File Or Directory

The user opens actions on a file/path row and chooses "Open in Quick Terminal". The action resolves symlinks lazily, uses the directory itself or the file's parent as cwd, opens Quick Terminal, and writes a quoted `cd <dir>\r` into the PTY.

If the PTY is dead before the cwd write, the app logs the dead-PTY condition. If writing fails, it logs the write failure. It should not silently pretend the shell cwd was applied.

### Type And Navigate In The PTY

Once Quick Terminal is focused, ordinary terminal keys belong to the PTY. Printable text, Ctrl chords, special keys, Tab, Shift+Tab, Enter, arrow sequences, and Escape are terminal input.

Tab writes `b"\t"` and Shift+Tab writes `b"\x1b[Z"` while stopping GPUI propagation. Escape is terminal input and must not close Quick Terminal.

### Close The Wrapper

Cmd+W, protocol `simulateKey` Cmd+W, and native footer Close converge on the Quick Terminal state-first close path. The window hides before ScriptList return content becomes visible, and terminal harness/apply-back state is cleared.

### Apply Back To Origin

When a harness route has both `tab_ai_harness_apply_back_route` and `tab_ai_harness_return_view`, the footer shows Apply and Close. Native Apply and Cmd+Enter share `quick_terminal_can_apply_back()`.

Apply reads terminal selection directly when available. Clipboard priming is only a fallback when there is no selection. If the route is missing, the implementation polls for a bounded period and cancels if the user leaves Quick Terminal or the entity disappears.

### Use Terminal Actions

Terminal actions are scoped to `ActionsDialogHost::TermPrompt`. They are separate from ScriptList actions, ACP popups, and launcher footer actions. The Quick Terminal footer itself shows only Close or Apply + Close.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open plain Quick Terminal. | Launcher `>` or built-in route. | ScriptList -> QuickTerminalView. | Type `>` / select command. | `open_quick_terminal(None)` -> warm attach or cold spawn. | Compact PTY terminal focused. | `tests/quick_terminal_contracts.rs`; feature 013 trigger tests. |
| Open at cwd. | File/path action. | File/path row actions. | Select `file:open_in_quick_terminal`. | Path execution -> `open_quick_terminal(Some(cwd))`. | Terminal opens and receives quoted `cd`. | Path action source audit plus runtime cwd proof. |
| Type text. | Quick Terminal. | Terminal focused. | Printable chars. | `TermPrompt` input path. | Bytes go to PTY. | Terminal input contract tests. |
| Use shell Ctrl chords. | Quick Terminal. | Terminal focused. | Ctrl+A through Ctrl+Z and bracket variants. | `TermPrompt::ctrl_key_to_byte`. | Control byte reaches PTY. | `tests/quick_terminal_contracts.rs`. |
| Use completion. | Quick Terminal. | Terminal focused. | Tab. | Quick Terminal key interceptor. | Writes `b"\t"` and stops propagation. | Tab contract tests. |
| Use backtab. | Quick Terminal. | Terminal focused. | Shift+Tab. | Special-key encoding. | Writes `b"\x1b[Z"`. | Shift+Tab contract tests. |
| Use shell/TUI Escape. | Quick Terminal. | Terminal focused. | Escape. | Quick Terminal terminal input path. | ESC reaches PTY; wrapper stays open. | Escape contract and runtime proof. |
| Apply harness output. | Harness Quick Terminal. | Apply visible. | Cmd+Enter or Apply. | `quick_terminal_can_apply_back()` -> `apply_tab_ai_result_from_terminal`. | Selected/fallback text applies to origin. | Tab AI apply-back tests. |
| Press Cmd+Enter without Apply. | Plain Quick Terminal or incomplete harness state. | Close-only footer. | Cmd+Enter. | Predicate false. | Falls through; no invisible apply. | Apply visibility contract. |
| Close wrapper. | Quick Terminal. | Terminal visible. | Cmd+W. | Quick Terminal key handler. | State-first close. | `tests/quick_terminal_contracts.rs`. |
| Close through protocol. | Automation target main. | Terminal visible. | `simulateKey` Cmd+W. | `runtime_stdin_match_simulate_key.rs`. | State-first close. | Protocol close contract. |
| Close through footer. | Quick Terminal. | Native footer active. | `footer:native:close`. | Footer dispatch -> state-first close. | Window hidden, terminal state cleared. | `scripts/agentic/footer-ownership-matrix.ts`; runtime receipt. |
| Open terminal actions. | Terminal surface. | Terminal focused. | Terminal action shortcut. | `toggle_term_prompt_actions` / `ActionsDialogHost::TermPrompt`. | Terminal actions dialog opens. | Terminal action source tests. |
| Clear terminal. | Terminal surface. | Terminal focused. | Clear shortcut. | `is_term_prompt_clear_shortcut` branch. | Terminal clear action runs. | Needs local source confirmation for exact plain Cmd+K chord. |
| Scroll terminal. | Quick Terminal wrapper. | Terminal content overflows. | Trackpad/mouse wheel. | Wrapper forwards to `TermPrompt::handle_external_scroll_wheel`. | Terminal scrollback moves. | Renderer source / runtime scroll proof. |
| Verify footer owner. | Automation. | Quick Terminal visible. | Footer ownership script. | `native_footer_surface()`. | Surface `quick_terminal`, owner `native`. | `scripts/agentic/footer-ownership-matrix.ts`. |

## State Machine

| State | Trigger | Transition | Notes |
|---|---|---|---|
| ScriptList idle. | Startup or return. | Warm pool may be empty, inflight, or ready. | Prewarm must not change `current_view`. |
| Warm prewarming. | App startup or post-open refill. | Creates one 80x24 themed terminal in background. | No visible UI. |
| Warm ready. | Prewarm succeeds. | Stores handle and creation timestamp. | TTL is 600 seconds. |
| Open request. | Launcher/built-in/path/harness route. | Calls `take_quick_terminal_warm_pty(cx)`. | Valid warm handle attaches; otherwise cold spawn. |
| Warm rejected. | Missing, inflight, stale, dead, or spawn-failed handle. | Kill invalid handle, schedule refill, return `None`. | User still gets cold terminal. |
| Active Quick Terminal. | Terminal created or attached. | `QuickTerminalView`, `focused_input = None`, `pending_focus = TermPrompt`. | PTY owns terminal input. |
| Active with apply-back. | Apply route and return view exist. | Footer resolves to Apply + Close; Cmd+Enter applies. | Predicate is `quick_terminal_can_apply_back()`. |
| Close restore-origin. | Harness close with restore disposition. | Clears apply state and restores saved origin where appropriate. | Used by harness-specific return behavior. |
| Close state-first. | Cmd+W, simulated Cmd+W, footer Close. | Hides/closes main window before return rendering. | Required for user-visible close contract. |
| Drop/shutdown. | App drop or idle clear. | Kills idle warm handle. | Warm PTY is process-local, not durable storage. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Launcher before route. | ScriptList filter and rows. | Launcher filter/list. | `AppView::ScriptList`. |
| Plain Quick Terminal. | Compact terminal panel with native footer Close. | `FocusTarget::TermPrompt`. | `QuickTerminalView`, prompt id `quick-terminal`, native surface `quick_terminal`. |
| Harness Quick Terminal. | Compact terminal panel with Apply + Close. | `FocusTarget::TermPrompt`. | Apply route and return view present. |
| SDK `term()`. | Full terminal prompt surface. | `FocusTarget::TermPrompt`. | `AppView::TermPrompt`; no native `quick_terminal` footer. |
| Terminal actions. | Actions dialog for terminal actions. | Actions dialog. | `ActionsDialogHost::TermPrompt`. |
| Closing. | Window becomes invisible before launcher content returns. | No terminal focus after close. | `windowVisible:false` before ScriptList receipt. |

## Keystrokes And Commands

| Input | Scope | Behavior |
|---|---|---|
| Printable chars | Quick Terminal | Forward to PTY. |
| Ctrl chords | Quick Terminal | Encoded as terminal control bytes. |
| Tab | Quick Terminal | Writes `b"\t"` to PTY and stops focus traversal. |
| Shift+Tab | Quick Terminal | Writes `b"\x1b[Z"` to PTY and stops focus traversal. |
| Escape | Quick Terminal | Forwarded to PTY; does not close. |
| Cmd+W | Quick Terminal | State-first wrapper close. |
| Cmd+Enter | Quick Terminal | Applies only when `quick_terminal_can_apply_back()` is true; otherwise falls through. |
| Cmd+K / clear shortcut | Terminal surface | Source shows a terminal clear shortcut branch; exact plain Cmd+K behavior needs local confirmation before stronger claims. |
| Terminal actions shortcut | Terminal surface | Opens/toggles `ActionsDialogHost::TermPrompt`; visible tests pin Cmd+Shift+K behavior. |
| Native Apply | Quick Terminal footer | Runs `apply_tab_ai_result_from_terminal` when Apply is visible. |
| Native Close | Quick Terminal footer | Runs state-first close through `footer:native:close`. |

## Actions And Menus

Quick Terminal has three distinct action/menu surfaces:

| Surface | Owner | Behavior |
|---|---|---|
| Native footer | Main-window native footer. | Shows Close only, or Apply + Close for apply-back harness state. |
| Terminal actions dialog | `ActionsDialogHost::TermPrompt`. | Terminal-specific actions such as clear/action commands. |
| File/path row actions | File/path action builder and execution path. | Offers `file:open_in_quick_terminal` before terminal is open. |

Launcher Run, AI, and Actions footer buttons are not copied into Quick Terminal. ACP slash/mention popups are also unrelated; Quick Terminal is terminal-owned after route handoff.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| `getState` after open. | Current view/prompt type identifies Quick Terminal, not SDK term or ACP chat. |
| `getElements` / footer receipts. | Native footer exposes `footer:native:close`; Apply appears only in apply-back state. |
| `simulateKey` Cmd+W. | Closes Quick Terminal state-first. |
| `simulateKey` Escape. | Must not close Quick Terminal. |
| `selectBySemanticId("footer:native:close", submit=true)`. | Dispatches native Close and returns a close receipt. |
| Footer ownership matrix. | SDK terminal is prompt-owned with no native surface; Quick Terminal is native-owned with `quick_terminal`. |
| Close ordering proof. | `windowVisible:false` is observed before ScriptList content returns. |

## Data, Storage, And Privacy Boundaries

- Terminal content is PTY content, not ordinary prompt text.
- Context extraction should treat `TermPrompt` and `QuickTerminalView` as terminal content rather than editable user input.
- PTY spawn environment is allowlisted: terminal vars plus selected `HOME`, `USER`, `PATH`, `SHELL`, `TMPDIR`, and `LANG`.
- `PROMPT_EOL_MARK=""` is part of spawn-time terminal behavior.
- Zsh-only `ZDOTDIR` points to `~/.scriptkit/quick-terminal-zsh/`, whose shim forwards user zsh config before disabling prompt marker options.
- Apply-back prefers direct terminal selection. Clipboard use is fallback only.
- Warm PTY state is process-local and killed on app drop or clear; it is not durable storage.
- Path actions can log local path/cwd fragments and should be treated as path-bearing diagnostics.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Warm missing or inflight. | Cold-spawn immediately; do not block the user. |
| Warm stale or dead. | Kill/discard invalid handle, schedule refill, cold-spawn. |
| Warm spawn failure. | Log failure and leave open path able to cold-spawn. |
| Terminal creation failure. | Log error and show a toast rather than switching to a broken surface. |
| Path cwd resolution failure. | Follow error path; do not silently open in the wrong directory. |
| PTY dead before cwd write. | Log dead PTY and skip fake cwd success. |
| Cwd input write failure. | Log write failure. |
| Apply route missing. | Bound polling, cancel if terminal closes/leaves, show route-aware failure if it never arrives. |
| Apply not available. | Hide Apply; Cmd+Enter must not invoke invisible apply. |
| Escape pressed. | Send to PTY; not an error, empty state, or close state. |

## Code Ownership

| Area | Files |
|---|---|
| Quick Terminal openers and cwd writes | `src/app_execute/utility_views.rs` |
| Warm PTY pool | `src/app_impl/quick_terminal_warm.rs` |
| Surface identity and native footer surface | `src/main_sections/app_view_state.rs` |
| Terminal rendering/key interception/scroll forwarding | `src/render_prompts/term.rs` |
| Terminal creation/input/actions/theme | `src/term_prompt/mod.rs` |
| PTY lifecycle and zsh shim | `src/terminal/pty/lifecycle.rs`, `src/terminal/pty/io_ops.rs` |
| Native footer buttons and dispatch | `src/app_impl/ui_window.rs` |
| Harness route/apply-back/close lifecycle | `src/app_impl/tab_ai_mode/mod.rs` |
| Protocol close handling | `src/main_entry/runtime_stdin_match_simulate_key.rs`, `src/main_entry/app_run_setup.rs` |
| File/path action routing | `src/actions/builders/file_path.rs`, `src/app_impl/execution_paths.rs` |
| Tests and scripts | `tests/quick_terminal_contracts.rs`, `tests/tab_ai_routing.rs`, `tests/tab_ai_harness_submission.rs`, `tests/tab_ai_input_coverage.rs`, `tests/main_window_footer_surface_owner_contract.rs`, `tests/sdk/test-term.ts`, `scripts/agentic/footer-ownership-matrix.ts` |

## Invariants And Regression Risks

- Quick Terminal is not ACP Chat.
- SDK `TermPrompt` must not inherit Quick Terminal's native `quick_terminal` footer.
- Launcher Quick Terminal must stay compact and avoid SDK `ViewType::TermPrompt` resize behavior.
- Warm PTY pool must fail open to cold spawn.
- Warm attach must retheme the terminal.
- Tab and Shift+Tab must go to the PTY, not focus traversal.
- Escape must go to the PTY, not wrapper close.
- Cmd+W physical, simulated Cmd+W, and native footer Close must converge on state-first close.
- Apply visibility and Cmd+Enter must share `quick_terminal_can_apply_back()`.
- Close must clear apply-back route/capture state.
- Footer buttons must stay terminal-scoped: Close or Apply + Close.
- Zsh prompt suppression must happen at spawn time, not through attach-time clear bytes.
- Terminal edge inset, render padding, resize math, and mouse hit testing must remain aligned.
- The Oracle pass flagged stale documentation risk around footer Close naming and footer spacer/hint-strip wording; source/tests should be treated as authoritative before editing `lat.md/`.

## Verification Recipes

Targeted source and agentic checks:

```bash
cargo test --test quick_terminal_contracts -- --nocapture
cargo test --test tab_ai_routing quick_terminal -- --nocapture
cargo test --test main_window_footer_surface_owner_contract -- --nocapture
bun scripts/agentic/footer-ownership-matrix.ts
lat check
```

Runtime proof checklist:

1. Trigger Quick Terminal from the launcher and assert `QuickTerminalView`, terminal focus, compact height, and native footer `quick_terminal`.
2. Trigger from a file action for a directory, a file parent, and a path with spaces/apostrophes; assert quoted cwd handoff.
3. Send Tab, Shift+Tab, Escape, printable text, and Ctrl chords; assert terminal ownership and no ACP/global focus movement.
4. Close via physical Cmd+W, protocol `simulateKey` Cmd+W, and `selectBySemanticId("footer:native:close")`; assert `windowVisible:false` before ScriptList content returns.
5. Start a harness Quick Terminal with apply-back route and return view; assert Apply + Close, Cmd+Enter apply, route cleanup on close.
6. Open plain Quick Terminal; assert Close-only footer and Cmd+Enter fallthrough.
7. Switch themes across cold and warm opens; assert terminal theme updates.
8. Spawn zsh Quick Terminal cold and warm; assert no prompt marker/blank-row artifact and no attach-time clear bytes.

## Agent Notes

Do not collapse Quick Terminal, SDK `term()`, and ACP Chat into one model. They share some terminal or harness primitives but have separate route identity, sizing, footer ownership, and input semantics.

When changing close behavior, prove all three close paths: physical Cmd+W, simulated Cmd+W, and native footer Close. When changing footer behavior, prove both visibility and dispatch. When changing terminal input, assume Tab, Shift+Tab, and Escape regressions will be immediately user-visible in shells and TUIs.

## Related Features

- [013 ScriptList Special Entry Triggers](./013-scriptlist-special-entry-triggers.md) owns the `>` launcher handoff into Quick Terminal.
- SDK `term()` prompt behavior is adjacent but should be documented as a separate feature because it has different route identity, height, and footer ownership.
- ACP Chat / Tab AI is adjacent where verification harness routing chooses Quick Terminal instead of chat.
- File Search / root source actions are adjacent because path rows can open Quick Terminal at a cwd.
- Theme/window/footer contracts are adjacent because Quick Terminal depends on compact sizing, native footer ownership, and runtime terminal theme propagation.

## Open Questions And Gaps

- Confirm the exact plain Cmd+K behavior inside Quick Terminal from local source before documenting it as the clear chord. The Oracle snapshot saw the clear branch and Cmd+Shift+K terminal-action test, but not enough to make a precise plain-Cmd+K claim.
- Reconcile stale documentation around footer Close naming: Oracle saw docs naming `close_tab_ai_harness_terminal_with_window` while source/tests pointed at `close_quick_terminal_main_window_state_first`.
- Reconcile footer spacer versus terminal hint-strip wording before writing this into `lat.md/`.
- Add explicit runtime receipt examples for Close-only and Apply + Close footer states once the current `getState`/footer schema is inspected.
- Name the fallback "run in terminal" route separately in future docs so compact launcher Quick Terminal constraints are not applied to full-height command terminal behavior.
