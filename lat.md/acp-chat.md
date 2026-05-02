# ACP Chat

Agent Chat is the user-facing chat surface for ACP-compatible agents. Internal code and protocol contracts still use `acp` names, but command labels and setup copy say Agent Chat.

## Entry paths

Agent Chat opens through `open_tab_ai_acp_with_entry_intent(...)` and related launcher flows. If a detached Agent Chat window is already open, the app focuses that window instead of opening a second one.

Plain `Tab` from the launcher also routes into Agent Chat. If the detached chat window already exists and the launcher has text, the app submits that text into the detached thread; otherwise it opens Agent Chat with only the current launcher input.

Selecting a skill from the main menu opens Agent Chat with the same staged state as accepting that skill from the slash picker: `/{skill} ` remains in the composer, the skill file is attached as pending context, and nothing auto-submits. The launcher path is [[src/app_impl/tab_ai_mode/mod.rs#ScriptListApp#open_acp_with_selected_skill]], and the ACP staging owner is [[src/ai/acp/view.rs#AcpChatView#stage_selected_plugin_skill_from_main_menu]].

Launcher `Tab`, launcher-style `Cmd+Enter`, and shared actions handoff now seed one ACP return-origin contract before opening ACP. `src/app_impl/tab_ai_mode/mod.rs` routes the launcher-style entry-intent openings through one preserving-return helper so close restores the exact originating launcher surface and shared filter focus, while detached-window reuse or save-offer short-circuits leave the prior return origin untouched.

When launcher ACP owns the shared actions dialog, closing that dialog restores focus through the embedded `AcpChatView` handle. The launcher host uses an ACP-specific restore target instead of the generic chat-prompt target so the composer regains caret ownership immediately after `Escape`, backdrop close, or `Cmd+K` toggle-close.

Large launcher pastes bypass Script List filtering when the clipboard payload is obviously document-sized. Cmd+V from the main menu routes straight into ACP and stages the clipboard as a `TextBlock` attachment so the launcher input does not fill with a giant multi-line query.

Some adjacent flows still route to `QuickTerminalView` instead of Agent Chat when the task needs a PTY-backed harness surface. That boundary matters because `QuickTerminalView` is the verification-oriented terminal wrapper, not the chat UI.

## New-script session contract

ACP primes `/new-script` in the slash picker on fresh opens and gates the footer Run button on a validated `SCRIPT_READY` receipt.

When ACP opens without an auto-submit intent, explicit empty-composer handoff, or focused context part, the composer primes the slash picker with `/new-script` pre-selected via `AcpChatView::prime_slash_entry()`. This surfaces the active skill affordance without auto-accepting it. Priming is skipped when the launcher trigger was plain `Tab` with empty input or an explicit `/` or `@` from the main menu — those paths already chose the composer or picker state. The policy is pinned by `tests/tab_ai_routing.rs` so plain Tab keeps `suppress_focused_part=true` through ACP open and that flag disables slash priming.

The ACP observer scans assistant messages for a `SCRIPT_READY path=<path> validated=true` receipt via `parse_script_ready_receipt()` in `src/ai/acp/view.rs`. The receipt must come from the `/new-script` skill's validation loop (Bun build + SK_VERIFY). When a validated receipt exists, the footer shows `Run` and dispatches `execute_script_by_path` for that exact path. Without a receipt, the footer hides `Run` in ACP views to prevent executing the wrong target.

The `SCRIPT_READY` receipt marker is defined as `SCRIPT_READY_RECEIPT_MARKER` in `src/ai/harness/mod.rs` and must appear in `kit-init/skills/new-script/SKILL.md`. The harness marker struct tracks its presence via `includes_script_ready_receipt`.

Both the native footer (mini mode) and the GPUI footer (full mode in `src/render_script_list/mod.rs`) route Run clicks through `dispatch_main_window_footer_action` so the ACP-aware path is always active regardless of footer mode.

During deferred context capture, `AcpChatView.context_capture_pending` drives the footer loading dot to `Streaming` so the UI never looks idle while screenshots or desktop context are being assembled.

## Footer activity indicator

The ACP footer dot reflects live agent activity, not only visible assistant text chunks.

`AcpChatView::footer_dot_status` is the single status mapper for the GPUI footer and the native main-window footer. Context capture, streaming turns, tool calls, plan updates, and permission waits all use an active pulsing dot because they keep the ACP thread in a non-idle user-visible state.

The embedded ACP observer in [[src/app_impl/tab_ai_mode/mod.rs#ScriptListApp#wire_embedded_acp_footer_callbacks]] notifies the parent `ScriptListApp` when the visible ACP footer status or model label changes. The parent owns the native footer, so this repaint is required for the dot to begin and stop pulsing during tool-only or thought-only activity without rebuilding the native dot on every token.

The native footer pulse deliberately animates opacity/color only on a stable AppKit layer at a slow breathing cadence of about 2.2 seconds per full autoreversed loop. The dot view is reconciled by identifier instead of rebuilt on every footer refresh, because removing the layer also restarts CoreAnimation and can make the pulse appear static. The animation setup is idempotent, uses `f64` duration values for `CABasicAnimation::setDuration:`, does not scale the dot, and the active dot is independent of whether the model label is currently available.

Agent Chat footers expose ACP actions, not launcher AI entry. [[src/app_impl/ui_window.rs#ScriptListApp#acp_footer_buttons]] omits the global `⌘↵ AI` button, and external ACP footer hosts mirror that in [[src/ai/acp/view.rs#AcpChatView#render_external_host_footer_from_snapshot]].

Plain Escape in Agent Chat cancels a streaming turn before it can close the surface. [[src/ai/acp/view.rs#AcpChatView#cancel_streaming_from_escape]] routes `AcpThreadStatus::Streaming` to [[src/ai/acp/thread.rs#AcpThread#cancel_streaming]] so the active dot and composer return to idle without hiding the chat.

Embedded host interceptors in [[src/app_impl/startup.rs]] and [[src/app_impl/startup_new_actions.rs]] call the same helper before their idle Escape return-to-menu path, because focused child routing is not guaranteed for every Escape key event.

Automation `simulateKey` routes in [[src/main_entry/runtime_stdin.rs]] and [[src/main_entry/app_run_setup.rs]] also call the helper before closing, so agentic tests exercise the same cancellation contract as the user-facing surface.

## Detached window behavior

The detached ACP window lives in `src/ai/acp/chat_window.rs` and carries a live thread when opened from an existing conversation.

`open_chat_window_with_thread(...)` transfers a live `AcpThread` into that window, stores the handle for later focus/close operations, and registers a stable automation ID for runtime targeting.

The detached window wires the same core footer actions as the embedded view: toggle actions, close, and history. `Cmd+W` closes the detached popup directly from `AcpChatView`; the main panel handles the equivalent close gesture through the app-level interceptor.

Detached ACP also keeps the thread alive for reuse. When the window is already open, ACP entry focuses it rather than creating another copy of the chat surface.

That reuse path must restore the `AcpChatView` focus handle, not just activate the popup shell. Current-app generation handoffs can return to an already-open detached chat, and the composer loses keyboard focus until click if the view focus is not re-applied.

Detached ACP is now explicit about attachment portal ownership. It only advertises `AcpHistory`, opens that history popup inside the detached window, and logs unsupported portal requests instead of exposing dead file or clipboard entry points that cannot return to the detached host.

If detached history popup opening fails after ACP has staged a portal session, the detached host now cancels that staged session instead of leaving ACP stuck in a half-open portal state.

When detached ACP attaches or dismisses a local history popup after reopening an inline `@history` mention, it now goes back through the staged portal session instead of bypassing it. Accept replaces the original token through the shared exact-replacement contract, and cancel restores the staged composer text and caret snapshot.

### Hide path asymmetry — hotkey detaches, stdin hides

Hotkey toggle and stdin `hide` deliberately diverge when `AcpChatView` is active: hotkey detaches the chat to a popup; stdin hides the main window.

The hotkey path (in `src/main_entry/app_run_setup.rs` around the `hotkey_detach_acp_requested` tracing event) detects `AppView::AcpChatView` and calls `open_chat_window_with_thread` to spawn the detached popup, then switches the main panel back to `ScriptList` while keeping it visible — the user's mental model is "toggle my launcher, not my AI chat."

The stdin `ExternalCommand::Hide` path in all three dispatchers (`src/main_entry/runtime_stdin_match_core.rs`, `src/main_entry/runtime_stdin.rs`, `src/main_entry/app_run_setup.rs`) does NOT branch on `AcpChatView` — it unconditionally calls `view.reset_to_script_list(ctx)`, re-keys `update_automation_semantic_surface("main", Some("scriptList"))`, and hides the main window. A programmatic `{"type":"hide"}` is "make this window go away," not "spawn a new popup as a side effect" — folding detach into stdin hide would break any automation script that chains `triggerBuiltin tab-ai` → `hide` because the hide would suddenly create a user-visible popup from code.

The asymmetry is pinned by `tests/stdin_hide_no_acp_detach_branch_contract.rs`: the stdin Hide arm in each dispatcher must contain the reset-and-rekey pair AND must NOT contain any hotkey-detach sentinels (`open_chat_window_with_thread`, `hotkey_detach_acp_requested`, or an `AppView::AcpChatView` branch). The companion `tests/hide_rpc_surface_reset_contract.rs` pins the positive half (reset + re-key present), so the two contracts together lock down both what the stdin hide MUST do and what it MUST NOT do.

### Automation registry parity — detached popup shares `acpChat` surface tag

The detached popup is intentionally registered with the SAME `semanticSurface = "acpChat"` as the Main-hosted `AcpChatView`; the `kind` field discriminates.

The single production registration site is inside `open_chat_window_with_thread` in `src/ai/acp/chat_window.rs` — one `upsert_automation_window(AutomationWindowInfo { kind: AutomationWindowKind::AcpDetached, semantic_surface: Some("acpChat".to_string()), .. })` call. The parity with the Main-hosted side (see `AppView::surface_contract()` in `src/main_sections/app_view_state.rs`, which maps `AppView::AcpChatView` → `"acpChat"`) lets a consumer match on `semanticSurface == "acpChat"` to find ACP chat regardless of attachment; `kind == "acpDetached"` is how the consumer targets the popup specifically. Numerous automation tests (`tests/automation/detached_acp_targeting.rs`, `tests/automation/window_targeting/mod.rs`, `tests/acp_targeted_reads.rs`, `tests/automation/actions_dialog_targeting.rs`) hardcode the `Some("acpChat")` expectation on `AcpDetached` fixtures, so renaming the detached tag to e.g. `"acpDetached"` is a conscious API divergence, not a drive-by clarification.

Pinned by `tests/detached_acp_popup_registry_surface_contract.rs`: two source-level tests assert that `src/ai/acp/chat_window.rs` contains exactly one `upsert_automation_window` call, that its body references both `AutomationWindowKind::AcpDetached` and `semantic_surface: Some("acpChat".to_string())`, and that no second site inside the file could fork the parity invariant.

### Embedded AI subview — addressable via `kind:"ai"` in the automation registry

When `AcpChatView` is the active main-panel subview, the automation registry holds a parallel entry with `kind: Ai` so `listAutomationWindows` enumerates the embedded AI surface as a first-class logical window.

Before this, the embedded surface was only observable via `listAutomationWindows.windows[0].semanticSurface == "acpChat"` on main — automation could not target it by kind. The shape is fixed: `id="ai"`, `kind=Ai`, `parent_window_id="main"`, `parent_kind=Main`, `semantic_surface="acpChat"`, `visible=true`, `focused=false`. The `focused: false` is load-bearing — the embedded subview does not own OS focus, main does, and setting `focused: true` here would de-focus main via [[src/windows/automation_registry.rs#upsert_automation_window]]'s "if focused, clear all others" invariant.

The single production helper is [[src/windows/automation_registry.rs#ensure_embedded_ai_window]], called from six sites in `src/app_impl/tab_ai_mode/mod.rs`: four `ensure_embedded_ai_window(true)` calls, one after each `self.current_view = AppView::AcpChatView { … }` assignment (reuse path, setup-error path, not-ready path, full-launch path), and two `ensure_embedded_ai_window(false)` calls inside the ACP close paths. Each entry-site upsert immediately re-keys main through [[src/windows/automation_registry.rs#update_automation_semantic_surface]] using [[src/main_sections/app_view_state.rs#semantic_surface_for_main_view]], so `listAutomationWindows` reports both `{id:"main", semanticSurface:"acpChat"}` and the child `{id:"ai", semanticSurface:"acpChat"}` after direct ACP entry paths such as dictation transcript delivery. `close_acp_chat_to_script_list` handles forced ScriptList returns; `close_tab_ai_harness_terminal_impl` handles normal return-origin closes from Escape, Cmd+W, and native main-window close. Pinned by `tests/embedded_ai_window_tab_ai_mode_sites_contract.rs` (5 tests: exact entry/exit counts, each entry follows an AcpChatView assignment, each entry re-keys main, and both exits live inside ACP close bodies).

The hide path needs the same teardown pairing. Four hide dispatchers (`src/main_sections/window_visibility.rs::hide_main_window_helper`, the three stdin `ExternalCommand::Hide` arms in `src/main_entry/runtime_stdin.rs`, `src/main_entry/runtime_stdin_match_core.rs`, and `src/main_entry/app_run_setup.rs`) each call `reset_to_script_list` + `update_automation_semantic_surface("main", Some("scriptList"))` + `ensure_embedded_ai_window(false)` in lock-step. Without the third call, `listAutomationWindows` post-hide leaves a stale `{id:"ai", visible:true, semanticSurface:"acpChat"}` entry that disagrees with its parent main entry on both visibility and semantic surface — the Pass #20 attacker anomaly `attacker-hide-path-embedded-ai-registry-stale`. Pinned by `tests/hide_path_embedded_ai_registry_teardown_contract.rs` (3 tests across all 4 sites: teardown exists, teardown follows `reset_to_script_list`, teardown is adjacent to the `scriptList` re-key — the two sibling registry writes stay lexically co-located so a refactor can't move one and forget the other).

`reset_to_script_list` is also an ACP teardown boundary. If the current view is Agent Chat, [[src/app_impl/registries_state.rs#ScriptListApp#reset_to_script_list]] calls [[src/ai/acp/view.rs#AcpChatView#prepare_for_host_hide]] before dropping the view, re-keys main back to `scriptList`, removes the embedded `kind:"ai"` registry entry, and emits `EmbeddedClosed`. This covers generic reset paths after launcher-triggered `/` or `@` opens so inline dropdown popups cannot survive into the next ScriptList render. Pinned by `reset_to_script_list_runs_embedded_acp_teardown` in `src/ai/acp/tests.rs`.

The same four hide dispatchers carry a sibling teardown for the `actions-dialog` popup. When `Cmd+K` has opened the shared actions popup and the user then hides main (stdin `{type:"hide"}`, Escape on the launcher, Cmd+W, etc.), the hide path must clear BOTH the automation registry entry AND the `ACTIONS_WINDOW` static `Mutex<Option<WindowHandle<ActionsWindow>>>` at [[src/actions/window.rs#close_actions_window]]. The sole production helper that does both is `crate::actions::close_actions_window(cx)` — its first line is `remove_automation_window("actions-dialog")` and its body takes+consumes the static `Option`.

Pass #23 (commit `a1349de4d`, Run 9) discovered the bare registry lie: post-hide `listAutomationWindows` reported `{id:"actions-dialog", kind:"actionsDialog", visible:true, parentWindowId:"main", semanticSurface:"actionsDialog"}` while `inspectAutomationWindow("actions-dialog")` failed with `No OS window matched automation target actions-dialog strongly enough`. Fix: added `crate::windows::remove_automation_window("actions-dialog")` to all four hide sites. That closed the registry-lie falsifier but — as Pass #29 (`cmd-k-on-unfocused-clipboard-pops-overlay-not-actions`) later uncovered — left a deeper bug alive: the `ACTIONS_WINDOW` static still held a stale `Some(handle)` across hide, so a later `simulateKey cmd+k` on an unfocused built-in (e.g. clipboardHistory visible after `triggerBuiltin`+`show`) read `is_actions_window_open()=true` and routed through the CLOSE branch of `toggle_clipboard_actions` at [[src/render_builtins/actions.rs#ScriptListApp#toggle_clipboard_actions]] line 333, popping whichever overlay was on top of the FocusCoordinator stack instead of opening the actions dialog. Pass #29 upgraded the fix at all four sites to `crate::actions::close_actions_window(cx)` — strictly stronger (it calls `remove_automation_window("actions-dialog")` internally, AND clears the static). Filed as `attacker-hide-path-actions-dialog-registry-stale` (Pass #23) + `cmd-k-on-unfocused-clipboard-pops-overlay-not-actions` (Pass #28 Reproduce / Pass #29 Fix). Pinned by `tests/hide_path_actions_dialog_registry_teardown_contract.rs` (4 tests across all 4 sites: every site calls `close_actions_window`, no site retains the legacy bare-registry call, the call follows its paired `ensure_embedded_ai_window(false)` in lock-step, and the gap between the two sibling teardowns stays under 1100 bytes with no function boundary between them). That dispatcher-side contract is backed by `tests/close_actions_window_first_line_registry_clear_contract.rs` (Pass #30, 4 tests): the `close_actions_window` signature appears exactly once at source level, the `remove_automation_window("actions-dialog")` call sits as the first real statement of the body and strictly precedes the `ACTIONS_WINDOW.get(` static read, both `ACTIONS_WINDOW.get(` and `guard.take(` remain in the body (the static clear stays present), and the `Unregister from automation registry before destroying` anchor comment sits above the registry call — closing the hole where a future silent-cleanup refactor could drop the first-line registry clear (arguing it's redundant given the static take), leave the dispatcher-side contract vacuously compliant (they still call `close_actions_window(cx)`), and silently re-enter the Pass #23 stale-registry regression shape.

The same four hide dispatchers also carry a sibling teardown for the `confirm-popup` registry entry. When a confirm dialog (script deletion via `Cmd+Shift+Delete`, Notes rename/create, etc.) is open and the user then hides main, AppKit tears down the OS child window with main but the automation registry entry persists — `listAutomationWindows` post-hide reports a phantom `{id:"confirm-popup", kind:"promptPopup", visible:true, parentWindowId:"main"}` entry. Root cause is structurally identical to the `actions-dialog` sibling above: the only production removal path is [[src/confirm/window.rs#close_confirm_window]] (line 385 calls `remove_automation_window("confirm-popup")`), and a grep of `src/main_sections` and `src/main_entry` for `close_confirm_window` returns zero hits — no hide dispatcher invokes it. The `lifecycle_reset` path has no confirm-popup branch at all (a stronger class than the `actions-dialog` sibling, which at least had a lifecycle_reset branch bypassed only by stdin). The Pass #25 fix adds `crate::windows::remove_automation_window("confirm-popup")` to all four hide dispatcher sites, immediately after the Pass #23 `actions-dialog` teardown so the three sibling registry writes (embedded-AI, actions-dialog, confirm-popup) stay lexically co-located in each dispatcher body. Filed as `attacker-hide-path-confirm-popup-registry-stale`. Pinned by `tests/hide_path_confirm_popup_registry_teardown_contract.rs` (3 tests across all 4 sites chaining the Pass #21 + Pass #23 + Pass #25 lock-steps into a 3-call adjacency — every site calls the removal, the confirm-popup teardown follows its paired `actions-dialog` teardown in lock-step, and the gap stays under 900 bytes with no function boundary between them, so an extracted `reset_main_and_teardown_children` helper that omits confirm-popup fails loudly on all four sites).

### getAcpState `{target:{kind:"ai"}}` routes to main's collector

`getAcpState` with an `{kind:"ai"}` target routes to `AcpReadTarget::Main { info: Some(resolved) }` — the embedded AI is a subview of main, so its ACP state is main's ACP state.

The routing lives in [[src/prompt_handler/mod.rs#resolve_acp_read_target]] as an explicit `AutomationWindowKind::Ai =>` arm added alongside the existing `Main` and `AcpDetached` arms. The arm must NOT route to `AcpReadTarget::Detached` — that variant calls `get_detached_acp_view_entity()`, which returns `None` when the ACP chat is attached rather than popped-out, so routing embedded AI through Detached would hit the no-entity error branch in normal operation. The arm emits a distinct `automation.acp_target.embedded_ai_routed_to_main` trace line so ops can tell from the log that a `{kind:"ai"}` request was served via main rather than being rejected as `target_unsupported`.

Pinned by `tests/source_audits/embedded_ai_acp_read_target.rs`: three source-level tests that assert (1) the `Ai` match arm exists, (2) it routes to `AcpReadTarget::Main { info: Some(resolved) }` and NOT `Detached`, (3) it emits the distinctive trace line so future refactors can't silently collapse the Ai arm into the generic Main fallthrough.

The downstream `AcpResolvedTarget.windowKind` receipt field carries the **resolved** `AutomationWindowKind` (not the read-target variant) through the authoritative `AutomationWindowKind::as_camel_case(self) -> &'static str` helper in [[src/protocol/types/automation_window.rs#AutomationWindowKind]]. So a `{target:{id:"ai"}}` or `{target:{kind:"ai"}}` request routed to `AcpReadTarget::Main { info: Some(resolved) }` reports `windowKind:"ai"` — agentic callers can tell from the receipt alone that they reached the embedded AI subview vs the ambient `scriptList` main surface. Pinned by `tests/acp_resolved_target_window_kind_contract.rs` (3 tests: helper exists, serde-vs-helper lock-step for all 7 variants, `build_acp_resolved_target` body reads `info.kind.as_camel_case()` and carries no `"main".to_string()` or `"acpDetached".to_string()` literal drift).

### Close cleanup — both paths drain the registry pair

Detached ACP close has two entry paths; both must drain the runtime-handle and automation-registry pair before `window.remove_window()` to avoid the Pass #29 stale-entry leak.

Path 1 (user-titlebar close / `Cmd+W`): GPUI fires the `on_window_should_close` callback registered inside `open_chat_window_with_thread` in `src/ai/acp/chat_window.rs`. That callback acquires the `CHAT_WINDOW.slot` mutex, takes the state, and if an `automation_id` is present calls `remove_runtime_window_handle(id)` followed by `remove_automation_window(id)` as an adjacent pair.

Path 2 (external TriggerAction or other automation caller): `close_chat_window(cx)` `take()`s the `CHAT_WINDOW.slot` FIRST, then calls `window.remove_window()`. The on_window_should_close callback still fires during the OS teardown, but by then `slot.take()` returns `None` so its registry-cleanup branch is unreachable. Pass #29 moved the same adjacent pair into `close_chat_window` itself so this path also drains the registry — without that duplication, external closes leaked registry entries forever.

Pinned by `tests/detached_acp_close_cleanup_contract.rs`: two source-level tests. `close_chat_window_cleans_up_registry_before_remove_window` asserts the adjacent pair exists inside `close_chat_window`, is gated by `if let Some(ref id) = state.automation_id`, appears before `window.remove_window();`, and that the `save_window_from_gpui(WindowRole::AcpChat, ...)` persistence call remains alongside it. `both_close_paths_carry_the_cleanup_pair` scans the whole file for adjacent cleanup pairs, asserts there are at least two (one per path), and asserts the count of `remove_automation_window(id)` calls equals the adjacent-pair count so neither half can be orphaned.

### Concurrent close safety — take-from-mutex pattern

Every CHAT_WINDOW cleanup site follows the same race-safe shape: acquire the `CHAT_WINDOW` mutex, call `g.take()` to extract the state atomically, then run cleanup on the taken state.

The shape matters because three close sites can race against the same state: (1) the placeholder `on_window_should_close` in `chat_window_options`, (2) the thread-carrying `on_window_should_close` in `open_chat_window_with_thread`, and (3) the external-caller `close_chat_window` helper. All three lock the single `CHAT_WINDOW: OnceLock<Mutex<Option<ChatWindowState>>>` static, so rust-level mutual exclusion is guaranteed. The exactly-once FUNCTIONAL guarantee — that `remove_runtime_window_handle` / `remove_automation_window` are not double-run on the same id — comes from the `.take()` pattern: whichever lock-holder runs first receives `Some(state)` and does the work; any subsequent lock-holder observes `None` and becomes a no-op. A refactor that replaced `.take()` with `.as_ref().cloned()` or `.clone()` would break this: both paths could observe the same state and double-run the registry-cleanup pair, corrupting the automation registry.

Pinned by `tests/detached_acp_concurrent_close_safety_contract.rs`: three source-level tests. `every_close_site_takes_state_out_of_the_mutex` asserts `g.take()` appears at least 3 times in the file (one per close site). `close_sites_use_the_same_chat_window_mutex` asserts the `CHAT_WINDOW` static is declared exactly once and initialized at every close site (≥3 `get_or_init` occurrences). `no_close_site_uses_non_take_extraction` forbids the clone-out-of-mutex patterns (`g.as_ref().cloned()`, ` g.clone()`, `(*g).clone()`) that would silently break exactly-once cleanup.

### Detach path re-keys main's automation surface to scriptList

The detach flow must re-key main's `AutomationWindowInfo.semanticSurface` from `"acpChat"` to `"scriptList"` in lockstep with the view flip, mirroring the hide-path re-key.

`close_acp_chat_to_script_list` in `src/app_impl/tab_ai_mode/mod.rs` is the single entry point for the ACP-chat-detach-to-script-list transition. It sets `self.current_view = AppView::ScriptList`, then calls `crate::windows::update_automation_semantic_surface("main", Some("scriptList".to_string()))` before emitting the `acp_chat_restored_to_script_list` tracing event. Without this call, `listAutomationWindows` reports `semanticSurface:"acpChat"` on main after detach even though `getState.promptType` is `"none"` — the registry tag only re-keys on the next `hide` or subview flip. Pass #49 surfaced this drift live; Pass #50 fixed it by mirroring the hide-path pattern at `src/main_sections/window_visibility.rs:397`, which calls the same helper after `reset_to_script_list`.

Pinned by `tests/detach_path_main_surface_rekey_contract.rs`: three source-level tests. `close_acp_chat_to_script_list_rekeys_main_surface_to_scriptlist` asserts the helper call is present inside the function body. `rekey_call_appears_before_acp_chat_restored_tracing_event` asserts ordering so any downstream observer of the trace event sees a consistent registry snapshot. `rekey_call_appears_after_current_view_is_set_to_scriptlist` asserts the re-key follows the view flip so the tag-view pair is never observed in an inconsistent intermediate state.

### Reattach re-keys main's automation surface via the triggerBuiltin choke point

Re-entering ACP through `triggerBuiltin tab-ai` after detach+close re-keys main's `semanticSurface` from `"scriptList"` to `"acpChat"` via a single unconditional post-match call shared by every builtin trigger in every stdin dispatcher.

The call lives at the tail of `ExternalCommand::TriggerBuiltin` in all three dispatcher files (`src/main_entry/runtime_stdin.rs`, `src/main_entry/runtime_stdin_match_core.rs`, `src/main_entry/app_run_setup.rs`) as `crate::windows::update_automation_semantic_surface("main", semantic_surface_for_main_view(&view.current_view))`. It runs AFTER the inner trigger match has flipped `view.current_view`, reads the post-flip view, and writes the matching surface tag. That same call is what Pass #44's direct-subview-to-subview chain, Pass #42's dispatcher sweep, and Pass #51's reattach round trip all depend on; without it, every `triggerBuiltin` would flip the view but leave the registry stale. Pass #51 confirmed at runtime that the reattach-side re-key (scriptList → acpChat) works cleanly across a full detach+close+reattach cycle.

Pinned by `tests/trigger_builtin_post_match_surface_rekey_contract.rs`: three source-level tests. `every_dispatcher_has_trigger_builtin_post_match_rekey` asserts each dispatcher contains the exact canonical call shape with the dynamic `semantic_surface_for_main_view(&view.current_view)` argument. `every_dispatcher_has_hide_path_script_list_rekey` asserts each dispatcher also contains the sibling hide-path call with hardcoded `Some("scriptList".to_string())`, symmetry with the tear-down side. `post_match_rekey_appears_after_inner_match_closing_brace` asserts structural ordering — the re-key must follow the `match name.to_lowercase()` body (verified via the canonical `Unknown built-in:` catch-all marker between them) — so a refactor that floated the call above the match would be caught before it shipped.

### Reattach preserves embedded view identity

The "Return to Panel" action closes the detached window and reuses the cached embedded `AcpChatView`, preserving the thread, message history, and view identity across the detach/reattach round trip.

On detach, `close_acp_chat_to_script_list` writes the live embedded entity into `self.embedded_acp_chat` before switching the current view to ScriptList. The detached window and the main embedded view share the same `AcpThread` entity, so messages sent in the detached session land in the same thread the embedded view references.

[[src/app_impl/tab_ai_mode/mod.rs#ScriptListApp#reattach_embedded_acp_from_detached]] is the single entry point for the reattach flow: it calls `try_reuse_embedded_acp_view(None, cx)` first (emitting `acp_reattach_embedded_reused`), falling back to a fresh launch only when the cache is empty (emitting `acp_reattach_embedded_cache_miss_fresh_launch`). Routing through this helper — rather than straight to `open_tab_ai_acp_with_entry_intent(None, ...)` — is what prevents the reuse gate from silently dropping the cached thread.

### Dictation delivery to the composer

Dictation sessions that target the AI chat composer deliver the transcript without auto-submit so the "nothing else changed" half of the round-trip contract holds: the text lands in the input, and the user still owns the submit gesture.

[[src/app_execute/builtin_execution.rs#ScriptListApp#handle_dictation_transcript]] records history BEFORE the delivery `match` so a delivery failure never silently drops the captured audio, then routes to `ai::set_ai_input(&mut **cx, &transcript, false)` for the `DictationTarget::AiChatComposer` arm (the `false` is the no-auto-submit flag). The `DictationTarget::TabAiHarness` arm seeds ScriptList/MainFilter as the ACP return origin, opens Agent Chat with the transcript as the entry intent, and suppresses focused launcher context so the dictated prompt is submitted as the first turn without attaching the currently selected ScriptList row.

The global dictation hotkey and legacy hidden `builtin/dictation-to-app` command route through `builtin/dictation-to-ai`, forcing `DictationTarget::TabAiHarness`. On completion, that target reveals main as Agent Chat, focuses the composer, submits the transcript, and ignores any remembered Notes or detached-ACP return focus. If a detached Agent Chat popup is open, dictation closes it before opening embedded Agent Chat so the orchestrator can focus the main composer.

The dictation reveal path depends on the window orchestrator bridge applying `FocusMain(ChatComposer)` after the AppKit reveal command runs. The bridge maps that token to the dedicated ACP focus target so the embedded composer receives keyboard focus on the next render instead of leaving focus on the main panel shell.

Because the ACP composer owns the focused GPUI handle, embedded ACP handles `Cmd+K`, Escape, and `Cmd+W` locally and invokes host callbacks instead of relying on bubbling back to the launcher interceptor. Detached ACP keeps its separate detached-actions path.

Embedded ACP close gestures deliberately split by intent. Plain Escape cancels streaming first; only idle Escape calls the close callback (`close_tab_ai_harness_terminal_with_window`) and returns to the seeded origin without hiding main. Cmd+W calls the host-window close callback, which runs the same lifecycle close first, then hide/resets main. Native main-window close follows the same close-then-hide order and syncs `SurfaceClosedBySystem(Main)`, ensuring the active ACP thread is prepared for host hide and the embedded `kind:"ai"` automation entry is removed.

If transcription returns no text, the ACP-targeted dictation session aborts quietly instead of finishing. `WindowEvent::AbortDictation` is target-aware for `TabAiHarness`, closing the overlay without restoring the launcher as `ScriptList` when no transcript exists to seed ACP.

`WindowEvent::FinishDictation` fires inside the `TabAiHarness` arm so the orchestrator triggers `RevealMain` immediately; the post-match cleanup guards against a second dispatch with `if !matches!(target, ...::TabAiHarness)`. The orchestrator then treats that finish as terminal: it does not run the generic aux-surface return-focus block, so Notes or detached ACP cannot steal keyboard focus after the ACP composer is shown. Removing the guard would double-fire orchestrator events for tab-ai dictation.

#### pushDictationResult stdin RPC

A named stdin hook lets automation inject synthetic transcripts through real dictation delivery.

`ExternalCommand::PushDictationResult { transcript, target, request_id }` routes through [[src/app_execute/builtin_execution.rs#ScriptListApp#deliver_stdin_dictation_result]], which resolves the explicit loose target label, then the active dictation session target, then the current UI-derived fallback before calling [[src/app_execute/builtin_execution.rs#ScriptListApp#handle_dictation_transcript]].

When a capture session is active, the hook stops it before injecting the synthetic result so agentic tests do not leave the microphone pipeline running. The loose `target: Option<String>` shape accepts aliases such as `acp` and `acpChat` without binding the stdin protocol to enum serde. Dispatcher receipts log request id, requested target, resolved delivery target, and transcript length only; transcript contents are never logged. Pinned at source level in `tests/push_dictation_result_stub_contract.rs`.

### Host isolation between Notes and the main launcher

`NotesApp` and `ScriptListApp` each hold their own `embedded_acp_chat: Option<Entity<AcpChatView>>` so a Notes-hosted ACP surface never inherits or mutates the main launcher's cached view (and vice versa). Host swaps cannot smuggle view state across.

Fresh-view spawn is the single point of construction: [[src/ai/acp/hosted.rs#spawn_hosted_view]] always runs `cx.new(|cx| AcpChatView::new(thread, cx))`, and [[src/ai/acp/view.rs#AcpChatView#new]] initializes `pending_portal_session: None` in every arm, so a newly-spawned Notes host starts clean even if the main-host view had a portal staged moments before. [[src/notes/window/acp_host.rs#NotesApp#open_or_focus_embedded_acp]] emits the `notes_acp_surface_opened` tracing event — the audit-visible stand-in for an `acp_host=notes` receipt while `AcpState` carries no `host` field.

`prepare_for_host_hide` clears the ephemeral popup state (attach menu, model selector, permission options, mention session, history menu, setup agent picker) but deliberately leaves `pending_portal_session` alone, matching [[tests/acp-portal-contract#Host transitions#Host hide keeps the staged session]] — the staged portal contract must survive host hides so reattach can deliver the token. It also clears a bare `@` or `/` composer trigger so the thread-change observer cannot re-fire on a later notify (agent preflight, model discovery, etc.) and pop the mention/slash picker back open over the newly-visible main menu.

## App-owned surface placement machine

A single [[src/ai/acp/surface_state.rs#AcpSurfaceState]] enum collapses the "where does ACP live right now" cross-product into one explicit state reduced by a tiny event machine.

The three states are `Hidden`, `Embedded`, and `AttachmentPortal { kind }`. The four events are `EmbeddedOpened`, `EmbeddedClosed`, `PortalOpened { kind }`, and `PortalClosed`. The pure reducer [[src/ai/acp/surface_state.rs#reduce_acp_surface]] lives in the `ai` module so it can be exhaustively unit-tested without a running app; the only mutator is [[src/app_impl/acp_surface_transitions.rs#ScriptListApp#transition_acp_surface]]. Every real transition emits one `acp_surface_transition` tracing event under `target = "script_kit::acp"` so operators can correlate placement drift with launcher-entry bugs.

Before the machine, "is ACP on-screen right now?" was inferred from `current_view == AcpChatView` + `embedded_acp_chat` + `attachment_portal_return_view.is_some()` + `active_attachment_portal_kind` — a 4-field conjunction that drifted under refactor.

The detached popup placement is deliberately NOT a state variant. The detached lifecycle lives in [[src/ai/acp/chat_window.rs]] and the app observes it externally via `is_chat_window_open()`. Merging it here would change invariants the portal flow relies on.

### Contract: only the mutator writes the field

Every mutation goes through [[src/app_impl/acp_surface_transitions.rs#ScriptListApp#transition_acp_surface]] — `self.acp_surface_state = ...` appears exactly once in the tree, inside the mutator.

Pinned by the `acp_surface_state_raw_writes_only_in_mutator` source-audit test in `src/app_impl/tests.rs`, which asserts neither `tab_ai_mode.rs` nor `attachment_portal.rs` nor `startup.rs` contains `self.acp_surface_state =`.

The four embedded-ACP open paths in [[src/app_impl/tab_ai_mode/mod.rs]] (fresh launch, reuse, setup card, not-ready) all fire `EmbeddedOpened`; `close_acp_chat_to_script_list` and the harness-terminal closing-chat branch both fire `EmbeddedClosed`. [[src/app_impl/attachment_portal.rs#ScriptListApp#open_attachment_portal]] fires `PortalOpened { kind }`; both close paths (`close_attachment_portal_with_part` and `cancel_attachment_portal`) fire `PortalClosed` first, then optionally `EmbeddedClosed` when the return view is NOT `AppView::AcpChatView`. The `acp_embedded_open_sites_fire_transition` and `acp_attachment_portal_fires_portal_transitions` source-audit tests pin these call shapes.

### Why it matters

`blocks_launcher_ai_entry()` is the shared predicate the two launcher guards can call to refuse routing into ACP while an attachment portal is on-screen.

Before the machine, `try_route_plain_tab_to_acp_context_capture` and `try_route_global_cmd_enter_to_acp_context_capture` each inferred portal ownership from a different field shape and drifted under refactor. The single enum makes "is ACP currently on-screen?" a 3-way match, not a 4-field conjunction.

Debug builds also run [[src/app_impl/acp_surface_transitions.rs#ScriptListApp#debug_assert_acp_surface_consistent]] after transitions: `Embedded` must agree with `AppView::AcpChatView`, `Hidden` must not observe `AcpChatView`, and `AttachmentPortal` must not observe `AcpChatView` as the current view (the portal host view is some launcher-style surface). Release builds pay no cost.

This is PR1 of Oracle-Session `acp-chat-state-machine-audit`. Overlay state (picker / history / permission), thread turn status, and a formal portal-contract machine are explicitly out of scope — they stay where they are until PR2.

## Context staging

ACP entry stages context in the current codebase, not just in old compatibility helpers. The launch path captures a UI snapshot, resolves desktop context, seeds the apply-back route, and then switches to ACP before deferred capture finishes.

The staged context can start from different inputs:

- a focused target chip, via `open_tab_ai_acp_with_explicit_target(...)`
- a selected plugin skill, via `open_acp_with_selected_skill(...)`
- the Ask Anything minimal desktop context resource
- an explicit ambient capture label for launcher-driven AI commands

For focused-target launches, the thread gets an inline token immediately and marks context bootstrap ready without waiting for deferred capture. For Ask Anything launches, ACP first stages the minimal context resource and then fills in the rest after the first paint.

Ask Anything screenshot capture stays path-based during ACP bootstrap. The staged text context includes the absolute screenshot path, while explicit screenshot attachments are the only ACP path that upgrades into image blocks.

If the launch is not ready yet, ACP renders an inline setup card instead of a broken chat surface. That setup path is part of the current `AcpChatView` contract.

### Agent setup copy

The setup card and docs point users to the Agent Catalog and `config.ts`, not direct-provider API-key setup commands.

The user-facing setup action is `Open Agent Catalog`. README configuration examples use the `ai` block (`selectedAcpAgentId`, model preferences, and profiles), while stable internal keys may keep `acp` because they back the ACP protocol. Legacy direct-provider API keys remain documented only as non-Agent Chat compatibility settings. Pinned by `tests/source_audits/execution_helpers.rs#agent_chat_setup_copy_points_to_catalog_and_config_ts_not_direct_provider_keys`.

### Screenshot identity threading

Tab AI screenshots thread identity through a file path, not a protocol-level id. The filename encodes three axes (UTC timestamp, PID, monotonic sequence) and the path survives into the ACP context text.

[[src/ai/harness/screenshot_files.rs#build_tab_ai_screenshot_filename]] formats the name with `%Y%m%dT%H%M%S%.3fZ` (millisecond precision), `std::process::id()`, and a `TAB_AI_SCREENSHOT_SEQUENCE: AtomicU64` bumped via `fetch_add(1, Ordering::Relaxed)`, all under the stable `tab-ai-screenshot-` prefix.

Both capture helpers ([[src/ai/harness/screenshot_files.rs#capture_tab_ai_focused_window_screenshot_file]] and [[src/ai/harness/screenshot_files.rs#capture_tab_ai_screen_screenshot_file]]) route through the same builder so focused-window and full-screen captures share one identity format; any divergence would fork identity and break cleanup. The capture result is a [[src/ai/harness/screenshot_files.rs#TabAiScreenshotFile]] tuple — `path`, `width`, `height`, `title`, `used_fallback` — where `path` is the primary identity axis and the rest are corroborating metadata consumers use to render and caption the image deterministically.

The path is threaded into `TabAiContextBlob.screenshot_path: Option<String>` in [[src/ai/tab_context.rs]], then [[src/ai/harness/mod.rs#build_tab_ai_harness_context_block]] emits a literal `screenshot path: <path>` line when the field is `Some`. [[src/ai/acp/context.rs#build_tab_ai_acp_context_blocks]] wraps the harness text as a SINGLE `ContentBlock::Text` — no parallel image block is inserted (the `screenshot_path_stays_in_text_context_without_image_block` regression test in the same file pins this). A second identity channel would drift from the text path and break end-to-end identity matching.

### Screen capture FFI flag invariant

The screen-capture path in `src/platform/ai_commands.rs` must pass the real `kCGWindowImageNominalResolution = 1 << 4` bit to `CGWindowListCreateImageFromArray` to actually get 1x output.

Earlier code declared that constant as `1 << 9`, which is not a defined `CGWindowImageOption` bit. CoreGraphics silently ignored it, so `capture_screen_excluding_self` returned retina-resolution pixels even though the surrounding comment claimed 1x. The vendored reference crate at `oracle/macos-screenshot-kit/src/platform/macos/ffi.rs` pins the correct values (`kCGWindowImageBestResolution = 1 << 3`, `kCGWindowImageNominalResolution = 1 << 4`); any future refactor that inlines or re-declares this constant must match.

`cgimage_to_rgba` also uses `checked_mul` when computing `bytes_per_row` and the total buffer size. An unchecked `width * 4` followed by `height * bytes_per_row` could wrap on very large captures and hand `CGBitmapContextCreate` a buffer smaller than its declared row/height, which would be out-of-bounds writes in FFI land. Overflow returns an `Err` before any allocation.

## ACP composer

`src/ai/acp/view.rs` owns the composer, message rendering, inline mention parsing, slash picker, history popup, and portal callbacks.

Plain Up on an empty idle/error composer recalls the latest user-authored turn via [[src/ai/acp/thread.rs#AcpThread#recall_last_user_message]], matching Zed-style prompt history behavior without intercepting cursor movement in non-empty input.

Cmd+0 resets Agent Chat font sizing through [[src/ai/acp/view.rs#AcpChatView#reset_agent_chat_zoom]], which restores the current theme's UI and mono font sizes to defaults through the shared theme sync/persist path.

### Markdown code fences

ACP message bodies render fenced code through the shared markdown block renderer so long lines scroll horizontally instead of clipping or wrapping.

`AcpChatView` calls [[src/prompts/markdown/api.rs#render_markdown_with_scope]] for each role. Fenced code bodies are built by [[src/prompts/markdown/code_table.rs#build_code_block_element]], which keeps the header fixed and puts only the code lines on an `overflow_x_scroll` surface with non-wrapping spans.

The current implementation supports inline mention sessions, slash-command sessions, and the context preview / portal flow that replaces stale wiki-era “dead token” language.

When the inline picker is open, plain `Enter` now accepts the focused row the same way plain `Tab` does. Embedded ACP also keeps a main-window interceptor fallback so picker acceptance still works when the launcher input layer sees the key first.

Pointer handling follows the same contract for `/` slash commands and `@` mentions. Parent-surface clicks dismiss the picker, row clicks first focus the row, and a second click or native double-click accepts it like Enter and dismisses the menu.

Outside-click dismissal records the exact active trigger/query so unchanged composer text cannot immediately recreate the popup on the next refresh. The suppression clears when the input or cursor no longer matches that dismissed trigger.

Mouse focus updates the existing popup row state in place. It must not resync the popup window from inside the popup click handler, because that can leave a replacement native popup visible after double-click submit.

ACP close paths must always clear the shared picker before hiding or removing the chat view. This prevents detached slash or mention popups from surviving after the embedded or detached chat surface closes.

When the caret lands on an existing inline `@mention`, ACP now prefers the focused mention preview over the generic `@` picker. The mention can reopen its portal with either `Cmd+.` or `Cmd+Shift+O` without flashing the picker empty state.

The composer’s footer callbacks are host-driven. The view exposes hooks for toggle actions, close, and history so embedded ACP and detached ACP can share the same UI logic without borrowing the view at the wrong time.

Attachment portal entry is now transactional on the ACP side. Opening a portal stages one session with the portal kind, seeded query, and original replace range, keeps the composer text unchanged while the portal is active, replaces that exact range on attach, and clears the staged session on cancel.

Portal staging is also host-aware. `open_picker_portal(...)` now refuses to stage a session when no host callback is registered, which closes the dead-session hole for detached or unsupported ACP hosts.

That same transactional contract now covers local `AcpHistory` popup acceptance too. Detached ACP and Notes-hosted ACP attach history through the staged portal session when one exists, so the preview copy, inserted token, and cancel restore path all stay aligned with the main-window portal flow.

If that file-backed history attach fails after ACP has staged a portal session, ACP cancels the staged session and restores the prior composer snapshot instead of falling through into the non-portal history resume path.

The mention picker also includes provider-backed items such as `@dictation`. That mention stays hidden until the `kit://dictation` resource is available, then routes into the dictation-history attachment portal so ACP can browse and attach a specific saved transcript instead of only staging the latest provider snapshot.

Selecting the built-in `Dictation` picker row from an in-progress query such as `@di` opens dictation history with an empty portal search. The typed mention query is only a picker lookup hint there, not an initial transcript filter.

Attached dictation-history selections round-trip as `kit://dictation-history?id=...` parts. ACP converts those parts back into inline `@dictation:<entry-id>` tokens so the portal can be reopened from the composer and the selected transcript still resolves on submit.

Browser history is also available as a portal-backed mention source. Typing `@browser-history` opens a searchable history browser that reads recent visits from supported local browser databases and attaches the selected visit as a structured context target instead of only the current tab URL.

The browser-history portal also shares the generic attachment-portal routing contract. `BrowserHistoryView` must restore launcher focus like the other portal views and must participate in prompt-state reporting so ACP and automation callers see the active filter and selection correctly.

Large clipboard text pastes collapse into inline `@text:"Pasted text #n +..."`
tokens instead of flooding the composer. ACP keeps the full pasted payload in a
`TextBlock` context part, renders the token as a compact pill in the composer,
and lets backspace/delete remove the whole token atomically like other inline
context mentions. Those pasted-text pills share the composer caret metrics so
the insertion point stays vertically aligned beside the pill. Their focused
preview hint stays explicit about replacing that token, but it remains
preview-only instead of reopening a search portal.

Clipboard images follow the same compact-input path. ACP writes the pasted
image to a temp PNG attachment, inserts a stable `@img:pasteN` alias token,
renders it as a `Pasted image #n` pill, and routes main-menu image pastes
straight into ACP instead of leaving the launcher without a text query. Those
synthetic image tokens also stay preview-only when focused so ACP does not
misrepresent them as file-portal reentry points.

Accepting any slash command — default, plugin skill, or Claude Code skill — inserts a literal `/{slash-name} ` token into the composer instead of expanding the skill body inline. For plugin and Claude skills, the skill body is attached as an `AiContextPart::SkillFile` via `thread.add_context_part(...)` so it still reaches the agent at submit time via the shared `build_staged_skill_prompt` resolver. This keeps the visible composer compact and uniform across command sources.

Composer tokens are colored with the theme accent to distinguish them from regular text. Attached `@mention` tokens are highlighted via `attached_inline_mention_highlight_ranges()` and the leading `/slash-name` token is highlighted via `leading_slash_highlight_range()`; both share `theme.colors.accent.selected`.

## Preconfigured profiles

`config.ts` can author Agent Chat profiles under `ai.profiles`, each with a label, optional agent/model override, and a `systemPromptSlug`.

The selected profile is stored on `ai.activeProfileId` and forwarded to the active agent at `session/new` time. Profiles are additive: omitting `selectedAcpAgentId`/`selectedModelId` defers to the current global selection. `systemPromptSlug` resolves the authored system prompt before it is appended through the agent's system-prompt hook (e.g. `--append-system-prompt` for the Claude Code harness). Switching profile from the ACP actions menu reuses the existing agent-switching relaunch path.

## Agent switching

Switching agents from the ACP actions menu relaunches ACP with the newly selected agent while preserving the existing draft input and pending inline context.

That relaunch must suppress any fresh focused-chip staging from the launcher surface that originally opened ACP. Otherwise the reopen path can inject a new inline `@mention` from the current Script List selection even though the user only asked to change agents.

The embedded actions popup must preserve the route-backed ACP dialog after construction. [[src/app_impl/actions_toggle.rs#ScriptListApp#toggle_actions]] skips generic scriptlet and Power Syntax rebuild hooks for ACP so `Change Agent` and `Change Model` cannot be replaced by global launcher actions.

### Config reload isolation during streaming

A config edit during an in-flight ACP stream must not interrupt the running agent subprocess, but a subsequent fresh read must still pick up the change. Two separate caches split these responsibilities.

Agent-side: `CACHED_AGENT_CONFIG: OnceLock<AcpAgentConfig>` in [[src/ai/acp/config.rs#claude_code_agent_config_cached]] is one-shot per-process. `prewarm_agent_config` populates it at startup off the main thread so the first ACP open does not block on bun transpile. The hot path short-circuits on the cached value, so re-entry during a streaming turn never spawns bun and never re-reads `config.ts`. There is deliberately no invalidation path — no `take()`, no `replace()`, no clear fn — because any mid-stream flip could yield args that drift away from what the already-spawned subprocess was launched with.

Script Kit-side: [[src/config/loader.rs#load_config]] holds no process-global `OnceLock<Config>`. Every call fingerprints `config.ts` (len + `modified_ms`) and consults a fingerprint-keyed disk cache (`try_load_cached_config`). An edit changes the fingerprint, invalidates the cache, and re-runs bun on the next read; an unchanged file serves instantly from disk. This is the half that lets the next ACP open / next read see the updated value without restarting the app.

### Agent selector catalog

The agent selector must expose starter ACP agents even before their binaries are installed.

`load_acp_agent_configs` merges the starter catalog into the loaded `~/.scriptkit/acp/agents.json` view, so OpenCode, Gemini CLI, and Codex are available to the selector as ready or install-needed entries. Fresh setup seeds Codex as `npx @zed-industries/codex-acp`; [[src/ai/acp/config.rs#normalize_well_known_agent_config]] also rewrites older `codex-acp` entries to that adapter path when the global adapter binary is absent.

Codex install readiness is based on the actual launch shape: [[src/ai/acp/config.rs#install_state_for_agent]] requires both the Codex CLI and an adapter path (`npx` or `codex-acp`) before reporting the Codex starter as ready.

Explicit agent switches use explicit preflight resolution. If the chosen agent is not ready, ACP shows that agent's setup blocker instead of falling back to another ready agent and persisting the fallback as the user's selection.

## Model selection

The model list shown in the actions menu is agent-driven. Each agent advertises its own models through the ACP `session/new` response, so the picker reflects whatever the launched agent currently supports instead of a stale hardcoded table.

When a session is created, the ACP client reads `NewSessionResponse.models` (an `Option<SessionModelState>`) and, if present, emits `AcpEvent::ModelsAvailable { current_model_id, models }` on the thread event channel before setting the session model. The thread reducer replaces `available_models` with the agent's live list. The user's persisted selection is preserved when it is still in the new list; otherwise the thread falls back to the agent's declared `current_model_id`, or the first entry.

`default_claude_code_models()` in `src/ai/acp/config.rs` remains as a bootstrap fallback. It seeds the picker before the first session is created and serves as the final list for older agents that do not advertise models. Do not rely on it as the source of truth for the current model catalog — the agent's advertisement wins once available.

### Preflight on thread open and actions-dialog open

`AcpRuntime::prepare_session` dispatches an `AcpCommand::PrepareSession` that runs `session/new` without sending a prompt, emits `ModelsAvailable` (or `SetupRequired` on auth failure), and drops the event sender.

The handler shares the same `ensure_session_and_announce_models` helper as the prompt-turn path, so binding bookkeeping and auth-failure handling stay identical across both entry points.

`AcpThread::refresh_models` wraps that call, spawns a listener, and funnels the events through the existing `apply_event` reducer so no new state-handling paths are required. It is invoked in two places:

- `AcpChatView::new` — runs when the view switches into `AcpChatSession::Live`, so by the time the user opens the actions dialog the agent's advertisement has typically landed in `thread.available_models`.
- `app_impl/actions_toggle.rs` — runs every time the user opens the ACP actions dialog. This is the deliberate "call `session/new` when Change Model is invoked" path so the picker reflects whatever the agent currently exposes (including models released after the previous open).

Subsequent calls to `PrepareSession` for the same `ui_thread_id` reuse the cached `AcpSessionBinding` and do not re-emit `ModelsAvailable`, so idle-opening the actions dialog many times in a row costs one `session/new` round-trip, not N. Relaunching the agent creates a fresh session and re-emits the advertisement.

### Hot prewarm before first submit

ACP keeps one default launch hidden and initialized so the first compatible chat open can submit without paying subprocess, initialize, or first `session/new` latency.

[[src/app_impl/tab_ai_mode/mod.rs#ScriptListApp#warm_acp_chat_on_startup]] runs after config prewarm and uses the host-neutral bootstrap to create a never-shown `AcpChatView` with default launch requirements. Because `AcpChatView::new` calls `AcpThread::refresh_models`, the worker queues `PrepareSession` immediately; the runtime then spawns, initializes, and creates an ACP session while the user is still outside chat.

The hidden view lives in `ScriptListApp::prewarmed_acp_chat`, separate from `embedded_acp_chat` so it cannot be confused with a previously visible conversation. [[src/app_impl/tab_ai_mode/acp_launch.rs#ScriptListApp#open_tab_ai_acp_view_from_request_impl]] calls [[src/app_impl/tab_ai_mode/mod.rs#ScriptListApp#take_prewarmed_acp_chat_for_launch]] before `AcpConnection::spawn_with_approval`; consumption only succeeds for default requirements, no retry request, and a matching selected agent. Capability-specific launches and agent-switch retries still create a fresh runtime.

## Boundary with `QuickTerminalView`

`QuickTerminalView` is a separate surface with different semantics.

- it is PTY-backed
- it is used for harness or verification-oriented flows
- `Tab` and `Shift+Tab` inside the terminal belong to the PTY, not to ACP focus navigation
- `Cmd+Enter` apply-back behavior is terminal-specific and uses the harness route logic
- it stays inside the compact main-window height when opened from the launcher, while SDK-spawned `TermPrompt` views still use the full terminal height

Agent Chat should not be described as that terminal surface. The current code treats them as related AI entry paths, not the same product surface.

### Quick Terminal launcher height contract

When the user opens Quick Terminal from the main launcher (the `>` sigil → `UtilityCommandType::QuickTerminal`), the panel must NOT grow to the SDK terminal height. The contract is enforced in three places that have to stay aligned:

- [[src/window_resize/mod.rs#quick_terminal_content_height]] returns `MINI_MAIN_WINDOW_MAX_HEIGHT - layout::FOOTER_HEIGHT` (≈410px). This is the *terminal-grid-only* height — used as the initial PTY resize hint via `TermPrompt::with_height`.
- [[src/window_resize/mod.rs#quick_terminal_panel_height]] returns the full `MINI_MAIN_WINDOW_MAX_HEIGHT` (440px). This is the *render wrapper* height — used as `.h()` on the container that holds both the terminal entity and the footer hint strip. Using the smaller content height here leaves a `FOOTER_HEIGHT`-sized gap below the footer.
- [[src/app_execute/utility_views.rs#ScriptListApp#open_quick_terminal]] passes `quick_terminal_content_height()` to `TermPrompt::with_height` and intentionally does NOT call `resize_to_view_sync(ViewType::TermPrompt, ...)`. Calling that resize would expand the NSPanel to `layout::MAX_HEIGHT` (700px), which is the bug the contract prevents.
- The render path at `src/render_prompts/term.rs` `render_term_prompt` branches on `is_quick_terminal`: Quick Terminal uses `quick_terminal_panel_height()` for the root wrapper height, all other terminal views (SDK-spawned `TermPrompt`, fallback "Run in Terminal" via [[src/app_execute/utility_views.rs#ScriptListApp#open_terminal_with_command]]) keep `layout::MAX_HEIGHT`.

The `ViewType::EditorPrompt | ViewType::TermPrompt => max_height` branch in `src/window_resize/mod.rs` is intentionally untouched — pinned by `src/window_resize/tests.rs` and used by the SDK terminal path.

### Quick Terminal warm PTY pool

Quick Terminal keeps one idle PTY ready so the next launcher open can attach without shell startup latency.

The warm pool lives in `ScriptListApp` state, not in a global singleton. It owns at most one `TerminalHandle`, tracks whether a refill is already in flight, and records a creation timestamp so stale handles can be rejected before use.

Lifecycle is startup → take → refill. `ScriptListApp::new` schedules [[src/app_impl/quick_terminal_warm.rs#ScriptListApp#warm_quick_terminal_pty]] after main-window initialization. [[src/app_execute/utility_views.rs#ScriptListApp#open_quick_terminal]] first calls [[src/app_impl/quick_terminal_warm.rs#ScriptListApp#take_quick_terminal_warm_pty]], then wraps a valid handle with [[src/term_prompt/mod.rs#TermPrompt#with_existing_terminal]] or falls back to `TermPrompt::with_height` for a cold spawn. After a successful open, it asks the pool to refill in the background.

The liveness contract fails open. A missing slot, an inflight refill, a spawn failure, a PTY older than 10 minutes, or a dead child process all make the current open cold-spawn a terminal instead of blocking the user. Invalid warm handles are killed and discarded; shutdown clears and kills the idle slot.

#### Quick Terminal path actions

Path-bearing rows can open Quick Terminal at their on-disk location while preserving the warm PTY path.

The canonical action is built by [[src/actions/builders/file_path.rs#open_in_quick_terminal_action]] and dispatches through the normal actions dialog as `file:open_in_quick_terminal`. Dispatch resolves symlinks lazily, uses the directory directly or a file's parent, and passes that cwd to [[src/app_execute/utility_views.rs#ScriptListApp#open_quick_terminal]]. The Quick Terminal entry point still takes the warm PTY first; when a cwd is present it writes a quoted `cd <dir>` into the attached PTY before yielding focus.

#### Quick Terminal theme adaptation

Quick Terminal must use the active Script Kit theme for terminal foreground, background, ANSI palette, cursor, and selection colors.

Cold PTYs are created through [[src/terminal/alacritty/handle_creation.rs#TerminalHandle#new_with_theme]] and [[src/terminal/alacritty/handle_creation.rs#TerminalHandle#with_command_and_theme]], so the Alacritty theme adapter starts from the current `Theme`. Warm PTYs are prewarmed with the current theme in [[src/app_impl/quick_terminal_warm.rs#ScriptListApp#warm_quick_terminal_pty]], then rethemed again when attached by [[src/term_prompt/mod.rs#TermPrompt#with_existing_terminal]] so a cached PTY cannot keep an obsolete light/dark palette.

Runtime theme changes route through [[src/app_impl/theme_focus.rs#ScriptListApp#sync_open_terminal_theme]], including theme chooser previews and restores, so open terminal cells resolve named/default colors against the latest light or dark theme before render.

### Quick Terminal edge inset

Quick Terminal gets a 6px gutter on all four sides so terminal cells don't touch the panel border.

The inset routes through the existing `effective_padding()` helper so render, resize/cols-rows math, and pixel→cell mouse hit-testing all see the same value — diverging would mismatch the rendered grid against mouse coordinates.

- `QUICK_TERMINAL_INSET_PX` constant in `src/render_prompts/term.rs` is the inset value (6.0).
- `TermPrompt::edge_to_edge_inset_px` is the per-instance pixel inset, defaulting to 0.0 for SDK terminals.
- [[src/term_prompt/mod.rs#TermPrompt#effective_padding]] returns `(inset, inset, inset)` when `edge_to_edge` is true, else falls back to `config.get_padding()`.
- [[src/term_prompt/mod.rs#TermPrompt#effective_padding_bottom]] returns `0.0` in `edge_to_edge` mode so cells run flush with the footer hint strip, else falls back to `config.get_padding().top` for symmetric SDK terminal padding. Render's `.pb()` and `resize_if_needed`'s `padding_bottom` argument MUST both source from this helper — if they diverge, the visible grid no longer matches the cell-count math.
- The `entity.update` block in `render_term_prompt` sets both `edge_to_edge = is_quick_terminal` and `edge_to_edge_inset_px = if is_quick_terminal { QUICK_TERMINAL_INSET_PX } else { 0.0 }` together. The fields must stay paired — flipping `edge_to_edge` without setting the inset is a regression.

### Quick Terminal native footer

Quick Terminal renders the same native AppKit main-window footer as the launcher list, ACP chat, and other launcher views.

The chrome (blur, divider, height, typography, padding, right-alignment) is shared via the native footer renderer; only the button list is scoped to actions meaningful in the terminal surface.

- `src/app_impl/ui_window.rs` `main_window_footer_surface` registers `AppView::QuickTerminalView { .. } => Some("quick_terminal")`. Without this arm, `main_window_uses_native_footer()` returns false and the GPUI hint strip leaks back in.
- SDK-spawned `AppView::TermPrompt` intentionally does not register a native footer surface. It keeps the GPUI terminal hint strip, while only `QuickTerminalView` swaps that strip for a native-footer spacer.
- `src/app_impl/ui_window.rs` `quick_terminal_footer_buttons()` returns `[Close]` always, plus `[Apply, Close]` when `tab_ai_harness_apply_back_route` AND `tab_ai_harness_return_view` are both `Some(_)`. Run / AI / Actions are intentionally NOT in this list — they are main-menu affordances; visual parity comes from sharing the native footer renderer, not from copying the main-menu button row.
- `src/app_impl/ui_window.rs` `quick_terminal_can_apply_back()` is the single predicate for both native Apply button visibility and Cmd+Enter apply-back handling. If Apply is hidden, Cmd+Enter falls through to the terminal instead of invoking an invisible action.
- `main_window_footer_buttons_for_current_view` checks Quick Terminal first, then ACP, then falls through to the standard set.
- `dispatch_main_window_footer_action` already maps `FooterAction::Apply` to `apply_tab_ai_result_from_terminal` and `FooterAction::Close` to `close_tab_ai_harness_terminal_with_window` for `AppView::QuickTerminalView`. Don't add new dispatch arms — adding the surface entry was sufficient.
- `src/render_prompts/term.rs` `render_term_prompt` branches on `is_quick_terminal`: Quick Terminal hands `render_native_main_window_footer_spacer()` to `main_window_footer_slot`; all other terminal views still call `render_terminal_prompt_hint_strip(None, None)` for their GPUI hint row.

### Quick Terminal stray `%` suppression

Quick Terminal disables zsh's `PROMPT_SP`/`PROMPT_CR` options so the first terminal row is the real prompt, not a marker glyph and not a blank wrapped row.

Zsh's `PROMPT_SP` option emits `PROMPT_EOL_MARK` + enough spaces to fill the line + `\r` whenever it suspects a partial line. The default mark is `%` (the original visible glyph), and the spaces+CR alone wrap to a blank row above the real prompt — so suppressing only the visible glyph still leaves a blank row.

Two layers cooperate:

- [[src/terminal/pty/lifecycle.rs#PtyManager#unix_spawn_env_allowlist]] sets `PROMPT_EOL_MARK=""` unconditionally. This hides the marker glyph for any zsh we can't reach via ZDOTDIR, and is harmlessly ignored by bash/fish.
- For zsh shells specifically, the allowlist also adds `ZDOTDIR=~/.scriptkit/quick-terminal-zsh/`. That directory is a Script-Kit-owned shim with two files: `.zshenv` forwards to `~/.zshenv`; `.zshrc` sources the user's real `~/.zshrc` and THEN runs `unsetopt PROMPT_SP` / `unsetopt PROMPT_CR` so the disable wins regardless of what the user's config does. The shim is created/refreshed on every spawn via [[src/terminal/pty/lifecycle.rs#PtyManager#ensure_zsh_quick_terminal_shim]]; `write_if_changed` keeps the operation idempotent.
- bash/fish/sh users get `PROMPT_EOL_MARK=""` only — `ZDOTDIR` is added only when `detect_shell()` ends with `zsh`.
- The fix applies uniformly to warm-pool spawns AND cold spawns because both routes go through `PtyManager::with_size → spawn_internal`.
- DO NOT switch to attach-time `\x1bc` / `\x1b[2J\x1b[H` clears — those bytes go through the PTY writer, can be misinterpreted as shell input depending on timing, and wipe useful startup output.

## Automation state snapshots

ACP automation receipts are assembled by named snapshot builders so state-first tests can audit each state part without reading one monolithic function.

`AcpChatView::collect_acp_state_snapshot` delegates setup, live thread state, picker state, input layout, context summary, and runtime setup-card inclusion to named helpers. This keeps `getAcpState` and `getAcpTestProbe` schema behavior stable while making future additions explicit.

## Telemetry

End-to-end turn telemetry is emitted under `acp_*` event/message names rather than a symmetric `start`/`chunk`/`end` trio. Every turn has an observable submit edge, a per-kind chunk fanout, and a termination edge.

The submit edge is [[src/ai/acp/thread.rs#AcpThread#prepare_turn_blocks_with_receipt]] emitting `event = "acp_submit_resolved_context_parts"` with `target: "script_kit::tab_ai"` and `attempted` / `resolved` / `failures` counts. This is the structural analogue of a `turn_start` event.

Stream chunks fan out by kind through [[src/ai/acp/handlers.rs]]: `acp_agent_thought`, `acp_tool_call`, `acp_tool_call_update`, `acp_plan_received`, `acp_mode_change`, `acp_commands_update`, `acp_usage_update`, and the `acp_session_update_unhandled` catch-all. There is no single `acp_stream_chunk` name — the per-kind discrimination is deliberate so tool-call activity can be correlated against model output.

The termination edge is dual. The modern streaming path emits `"acp_turn_completed"` with `stop_reason = ?prompt_response.stop_reason` in [[src/ai/acp/client.rs]]; the legacy `handle_stream_prompt` path emits `"acp_prompt_completed"`. Both must remain — renaming one silently halves turn-termination visibility. The `stop_reason` field is the structural analogue of a `turn_end` event's expected fields, distinguishing normal completion from cancellation or tool-request-stop.

User cancellation is a real ACP `session/cancel` notification, not only a local UI stop. [[src/ai/acp/thread.rs#AcpThread#cancel_streaming]] enqueues an out-of-band cancel through [[src/ai/acp/client.rs#AcpRuntime#cancel_turn]], and the worker emits `acp_session_cancel_requested` before waiting for the agent's cancelled stop reason.

## Current code references

These are the live files that define the ACP surface today.

- `src/app_impl/tab_ai_mode/mod.rs` for ACP entry routing, detached-window reuse, and context staging
- `src/app_impl/tab_ai_mode/source_classification.rs` for source-type classification and apply-back hints used by Tab AI compatibility paths
- `src/ai/acp/chat_window.rs` for detached window lifecycle and action wiring
- `src/ai/acp/view.rs` for the ACP composer, inline mentions, history, and portal behavior
- `src/ai/acp/client.rs` for the ACP event loop and `AcpEvent::ModelsAvailable` emission on `session/new`
- `src/ai/acp/thread.rs` for `AcpThread::apply_agent_models` and the thread reducer
- `src/ai/acp/config.rs` for `AcpModelEntry` and the `default_claude_code_models()` bootstrap fallback
- `src/ai/window/context_picker/mod.rs` for mention hint visibility and provider-backed `@dictation` entries
- `src/browser_history.rs` for local browser-history loading and ranking across supported browsers
- `src/dictation/history.rs` for the persisted transcript feed that hydrates the dictation provider
- `src/main_sections/app_view_state.rs` for the `AppView` routing enum

## Stale claims corrected

These are the old wiki claims this page no longer repeats.

- Agent Chat is not the only AI surface in the app; `QuickTerminalView` still exists for PTY-backed harness flows.
- Plain `Tab` does not always create a fresh ACP surface; if a detached chat window exists, the app focuses that window and may submit into it.
- Context staging is not just a single compatibility helper anymore; it now includes focused targets, explicit skill handoffs, and deferred desktop capture.
- Detached ACP is not just an embedded panel detail; it is a separate popup window with its own focus, close, and history behavior.
