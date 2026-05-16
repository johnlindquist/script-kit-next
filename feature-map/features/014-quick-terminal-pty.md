# 014 Quick Terminal PTY / TermPrompt / Warm Pool / Apply-back

This chapter maps the PTY-backed Quick Terminal surface, its terminal child, warm pool, native footer, and apply-back lifecycle.


## Executive Summary



| Surface | Route | Sizing | Footer | Primary behavior |
|---|---|---|---|---|

The load-bearing contracts are compact launcher sizing, warm PTY reuse that fails open to cold spawn, terminal-scoped native footer buttons, and apply-back visibility gated by both an apply route and a return view.

## What Users Can Do

| User capability | Entry | Result |
|---|---|---|
| Open plain Quick Terminal. | Launcher `>` handoff or built-in Quick Terminal route. | Opens `QuickTerminalView`, focuses `TermPrompt`, uses warm PTY if available, and stays compact. |
| Type into a real PTY. | Printable keys, Ctrl chords, terminal special keys, Tab, Shift+Tab, Escape. | Bytes are forwarded to the PTY, not ACP navigation, launcher filter traversal, or global cancel. |
| Close the wrapper. | Cmd+W, protocol `simulateKey` Cmd+W, or native footer Close. | Closes state-first and clears harness/apply-back terminal state. |
| Apply terminal output back. | Native Apply or Cmd+Enter when Apply is visible. | Reads selected terminal text when possible, falls back to clipboard only if needed, and applies to the saved route. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| `TermPrompt` | Terminal renderer/input component used by multiple terminal surfaces. | Quick Terminal contains one, but SDK `term()` also uses one through a different route. |
| `TerminalHandle` / PTY manager | Alacritty/PTY-backed runtime handle. | Cold terminals are themed on creation; attached warm terminals are rethemed. |
| Warm PTY pool | One-slot idle terminal cache on `ScriptListApp`. | Tracks handle, inflight state, and creation time; stale/dead/missing/inflight states cold-spawn. |
| Apply-back state | Harness route state for sending terminal output back to origin. | `quick_terminal_can_apply_back()` is true only when apply route and return view both exist. |
| Zsh prompt suppression | Spawn-time shell environment/shim behavior. | Uses `PROMPT_EOL_MARK=""` and zsh-only `ZDOTDIR` shim; do not use attach-time clear bytes. |

## Entry Points

| Entry | Context | Result |
|---|---|---|
| Launcher `>` special entry. | Main ScriptList filter contains exactly `>`. | Calls `open_quick_terminal(None, cx)`. |
| Built-in Quick Terminal. | Built-in utility command / triggerBuiltin route. | Calls the same Quick Terminal opener without cwd. |
| Tab AI verification harness. | Verification flow needs PTY-backed execution instead of ACP chat. | Saves return view/focus, seeds apply-back route, opens Quick Terminal, then submits/captures terminal output. |
| Fallback "run in terminal". | Utility fallback command terminal path. | Uses a terminal prompt with full terminal resize while sharing some QuickTerminalView wrapper behavior; do not confuse with compact launcher Quick Terminal. |

## User Workflows

### Open From Launcher



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


## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open plain Quick Terminal. | Launcher `>` or built-in route. | ScriptList -> QuickTerminalView. | Type `>` / select command. | `open_quick_terminal(None)` -> warm attach or cold spawn. | Compact PTY terminal focused. | `tests/quick_terminal_contracts.rs`; feature 013 trigger tests. |
| Type text. | Quick Terminal. | Terminal focused. | Printable chars. | `TermPrompt` input path. | Bytes go to PTY. | Terminal input contract tests. |
| Use completion. | Quick Terminal. | Terminal focused. | Tab. | Quick Terminal key interceptor. | Writes `b"\t"` and stops propagation. | Tab contract tests. |
| Use backtab. | Quick Terminal. | Terminal focused. | Shift+Tab. | Special-key encoding. | Writes `b"\x1b[Z"`. | Shift+Tab contract tests. |
| Use shell/TUI Escape. | Quick Terminal. | Terminal focused. | Escape. | Quick Terminal terminal input path. | ESC reaches PTY; wrapper stays open. | Escape contract and runtime proof. |
| Apply harness output. | Harness Quick Terminal. | Apply visible. | Cmd+Enter or Apply. | `quick_terminal_can_apply_back()` -> `apply_tab_ai_result_from_terminal`. | Selected/fallback text applies to origin. | Tab AI apply-back tests. |
| Press Cmd+Enter without Apply. | Plain Quick Terminal or incomplete harness state. | Close-only footer. | Cmd+Enter. | Predicate false. | Falls through; no invisible apply. | Apply visibility contract. |
| Close wrapper. | Quick Terminal. | Terminal visible. | Cmd+W. | Quick Terminal key handler. | State-first close. | `tests/quick_terminal_contracts.rs`. |
| Close through protocol. | Automation target main. | Terminal visible. | `simulateKey` Cmd+W. | `runtime_stdin_match_simulate_key.rs`. | State-first close. | Protocol close contract. |
| Clear terminal. | Terminal surface. | Terminal focused. | Clear shortcut. | `is_term_prompt_clear_shortcut` branch. | Terminal clear action runs. | Needs local source confirmation for exact plain Cmd+K chord. |
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
| Native Apply | Quick Terminal footer | Runs `apply_tab_ai_result_from_terminal` when Apply is visible. |

## Actions And Menus


| Surface | Owner | Behavior |
|---|---|---|
| Native footer | Main-window native footer. | Shows Close only, or Apply + Close for apply-back harness state. |

Launcher Run, AI, and Actions footer buttons are not copied into Quick Terminal. ACP slash/mention popups are also unrelated; Quick Terminal is terminal-owned after route handoff.

## Automation And Protocol Surface

| Automation target | Assertion |
|---|---|
| `getState` after open. | Current view/prompt type identifies Quick Terminal, not SDK term or ACP chat. |
| `simulateKey` Cmd+W. | Closes Quick Terminal state-first. |
| `simulateKey` Escape. | Must not close Quick Terminal. |
| Footer ownership matrix. | SDK terminal is prompt-owned with no native surface; Quick Terminal is native-owned with `quick_terminal`. |

## Data, Storage, And Privacy Boundaries

- Terminal content is PTY content, not ordinary prompt text.
- Context extraction should treat `TermPrompt` and `QuickTerminalView` as terminal content rather than editable user input.
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
- Warm PTY pool must fail open to cold spawn.
- Warm attach must retheme the terminal.
- Tab and Shift+Tab must go to the PTY, not focus traversal.
- Escape must go to the PTY, not wrapper close.
- Cmd+W physical, simulated Cmd+W, and native footer Close must converge on state-first close.
- Apply visibility and Cmd+Enter must share `quick_terminal_can_apply_back()`.
- Close must clear apply-back route/capture state.
- Zsh prompt suppression must happen at spawn time, not through attach-time clear bytes.
- Terminal edge inset, render padding, resize math, and mouse hit testing must remain aligned.
- The Oracle pass flagged stale documentation risk around footer Close naming and footer spacer/hint-strip wording; source/tests should be treated as authoritative before editing `removed-docs/`.

## Verification Recipes


```bash
cargo test --test quick_terminal_contracts -- --nocapture
cargo test --test tab_ai_routing quick_terminal -- --nocapture
cargo test --test main_window_footer_surface_owner_contract -- --nocapture
bun scripts/agentic/footer-ownership-matrix.ts
source checks
```


1. Trigger Quick Terminal from the launcher and assert `QuickTerminalView`, terminal focus, compact height, and native footer `quick_terminal`.
2. Trigger from a file action for a directory, a file parent, and a path with spaces/apostrophes; assert quoted cwd handoff.
3. Send Tab, Shift+Tab, Escape, printable text, and Ctrl chords; assert terminal ownership and no ACP/global focus movement.
5. Start a harness Quick Terminal with apply-back route and return view; assert Apply + Close, Cmd+Enter apply, route cleanup on close.
6. Open plain Quick Terminal; assert Close-only footer and Cmd+Enter fallthrough.
7. Switch themes across cold and warm opens; assert terminal theme updates.
8. Spawn zsh Quick Terminal cold and warm; assert no prompt marker/blank-row artifact and no attach-time clear bytes.

## Agent Notes

Do not collapse Quick Terminal, SDK `term()`, and ACP Chat into one model. They share some terminal or harness primitives but have separate route identity, sizing, footer ownership, and input semantics.


## Related Features

- [013 ScriptList Special Entry Triggers](./013-scriptlist-special-entry-triggers.md) owns the `>` launcher handoff into Quick Terminal.
- SDK `term()` prompt behavior is adjacent but should be documented as a separate feature because it has different route identity, height, and footer ownership.
- ACP Chat / Tab AI is adjacent where verification harness routing chooses Quick Terminal instead of chat.
- File Search / root source actions are adjacent because path rows can open Quick Terminal at a cwd.
- Theme/window/footer contracts are adjacent because Quick Terminal depends on compact sizing, native footer ownership, and runtime terminal theme propagation.

## Open Questions And Gaps

- Confirm the exact plain Cmd+K behavior inside Quick Terminal from local source before documenting it as the clear chord. The Oracle snapshot saw the clear branch and Cmd+Shift+K terminal-action test, but not enough to make a precise plain-Cmd+K claim.
- Reconcile footer spacer versus terminal hint-strip wording before writing this into `removed-docs/`.
- Add explicit runtime receipt examples for Close-only and Apply + Close footer states once the current `getState`/footer schema is inspected.
- Name the fallback "run in terminal" route separately in future docs so compact launcher Quick Terminal constraints are not applied to full-height command terminal behavior.
