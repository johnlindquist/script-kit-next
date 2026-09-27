# Script Kit Feature Contracts

This document is generated from the `scriptkit-core-qa-50` web-choice submission and is the durable map from user-facing features to the QA proof we expect to keep stable while refactoring, fixing bugs, and adding features.

## Scope Rules

- Submission: `.hitl-choice/submissions/7526023d-9cdf-4ef2-8bd9-c6cad5938d46.json`.
- Active lock-in scope: 49 selected QA candidates.
- One selected candidate is omitted from this contract per product-scope feedback.
- Experimental: Mini Mode is documented as WIP only; it is not treated as a stable user contract yet.
- Terminology: user-facing docs say `Agent Chat`; the goose engine is an implementation detail. Stable anchors below keep their original ids.
- Main input cwd behavior: `>` must not select cwd; Tab is the cwd trigger.
- Cargo verification must use `./scripts/agentic/agent-cargo.sh`, not bare `cargo`.

## QA Lock-In Status

| Priority | Count | Meaning |
| --- | ---: | --- |
| P0 | 15 | Core daily workflows that should get first DevTools receipts. |
| P1 | 19 | Important support and recovery surfaces. |
| P2 | 15 | Lower-risk but still user-visible contracts. |

## Standard Proof Pattern

Every feature below should be locked with the smallest proof set that can fail for the contract:

1. A Script Kit DevTools runtime receipt when the behavior is visible or interactive.
2. A focused source or integration test through `./scripts/agentic/agent-cargo.sh`.
3. A documented blocker when a DevTools primitive is missing, including the exact missing primitive.
4. A `FEATURES.md` update when a contract changes intentionally.

For search, the owned launcher recipe is the shared executable inventory, not a claim that every live provider or native desktop effect is covered. Receipts must account for every declared case/schedule, preserve exact target and completed-frame identities, distinguish semantic/production-consumer/GPUI/framebuffer evidence, and fail on missing capability or invalid cleanup. A sampled or blocked case is not a pass. Builds use immutable `ArtifactReference` outputs; no mtime-selected binaries, desktop activation, or native-input fallback is permitted for the owned recipe.

## Memory Layer Surfaces (Brain Time)

New surfaces from the Brain Time work (`.notes/brain-time.md`). These are NOT
part of the `scriptkit-core-qa-50` QA lock-in submission — the Feature Matrix
below stays pinned to the 49 selected candidates. These entries graduate into
the matrix only through a future QA lock-in selection.

### Day Page opens in the main launcher window

- Stable anchor: `day-page-opens-in-main-launcher-window`
- Surface: Day Page / Main Launcher
- Status: Active (Brain Time T9), pending QA lock-in selection
- User contract: Tap-while-open toggles Script List ↔ Day Page in the same main window (no resize, no window swap). Today's `brain/days/YYYY-MM-DD.md` binds on entry; the shared notes editor (`day-page-editor`) persists edits through the brain substrate. Cmd+P uses the same Notes search container/result language as the Notes view, with selected rows opening in the current editor by default. Esc dismisses the main window.
- Regression risk: Gesture routing or `AppView` morph refactors can break in-place toggle, day rollover binding, or editor save semantics.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --lib day_page`
  - `bun scripts/agentic/notes-day-page-switcher-probe.ts`
  - `bun scripts/devtools/inspect.ts --session brain-time --start --show --main --surface DayPage --limit 200`
- Pass evidence: Day-page unit tests pass; DevTools inspect shows `day-page-editor` and stable main-window bounds across launcher ↔ Day Page toggle.
- Fail evidence: Wrong view after toggle, missing editor semantic id, day file not created, or unit tests fail.

### Main hotkey gesture grammar morphs surfaces in place

- Stable anchor: `main-hotkey-gesture-grammar-morphs-surfaces`
- Surface: Main Launcher / Day Page / Agent Chat
- Status: Active (Brain Time T2/T8), pending QA lock-in selection
- User contract: Key-down shows the launcher immediately. Tap-while-open toggles launcher ↔ Day Page with query carry-over. Double-tap opens Agent Chat. Hold (~250ms) opens Day Page. Tap never dismisses — Esc is the only dismiss.
- Regression risk: Hotkey delivery or classifier wiring can regress instant show, retire tap-to-dismiss, or route gestures to the wrong surface.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --lib gesture`
  - `scripts/agentic/main-hotkey-gesture-probe.ts` (runtime receipt when available)
- Pass evidence: Gesture classifier unit tests pass; runtime probe shows stable window id across morphs and carry-over text in Day Page editor after toggle.
- Fail evidence: Delayed first paint, tap hides window, wrong surface after double-tap/hold, or classifier tests fail.

### Script Kit Brain substrate persists memory as markdown files

- Stable anchor: `script-kit-brain-substrate-markdown-canonical`
- Surface: Script Kit Brain / Storage
- Status: Active (Brain Time T1/T5/T7), pending QA lock-in selection
- User contract: User memory lives under `~/.scriptkit/brain/{days,fragments,notes,trash}` as plain markdown. Day-page appends are timestamped and append-only. Fragments split at the word threshold with excerpt + link on the day page. `brain.sqlite` and `notes.sqlite` are derived indexes rebuildable from files alone.
- Regression risk: Path construction outside `src/brain/substrate/`, partial writes, or indexer source drift can break the files-canonical contract.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --lib brain::substrate`
  - `./scripts/agentic/agent-cargo.sh test notes`
  - `./scripts/agentic/agent-cargo.sh test --lib brain`
- Pass evidence: Substrate unit tests pass; notes rebuild-from-files test passes; brain indexer includes day pages and fragments.
- Fail evidence: Content only in sqlite, failed atomic writes, missing rebuild parity, or tests fail.

### Clipboard sediment keeps brain content without popup UI

- Stable anchor: `clipboard-sediment-no-popup`
- Surface: Clipboard History / Day Page
- Status: Active (Brain Time T4/T10/T12/T14), pending QA lock-in selection
- User contract: Secrets are rejected before storage. URLs auto-keep to today's day page. Non-URLs promote on re-copy. Copy tracking must not open a post-copy popup or HUD. Day Page renders fragment excerpt cards and kept-URL links.
- Regression risk: Monitor ordering, sediment tiers, or post-copy UI regressions can leak secrets, skip keeps, or reintroduce surprise UI.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --lib clipboard_history`
  - `bun scripts/agentic/clipboard-sediment-no-popup-probe.ts` (runtime no-popup receipt when available)
- Pass evidence: Rejection and sediment unit tests pass; runtime probe shows brain insertion with no post-copy UI target.
- Fail evidence: Rejected content stored, duplicate URL lines same day, copied content opens post-copy UI, or tests fail.

## Feature Matrix

### Main launcher cold start renders the script list

- Stable anchor: `main-launcher-cold-start-renders-the-script-list`
- Choice id: `main-launcher-cold-start-surface-contract`
- Surface: Main Launcher
- Status: Active QA lock-in candidate
- User contract: A visible focused main window reports promptType/surfaceContract for ScriptList, nonzero visible choices, stable input ownership, and usable semantic elements.
- Regression risk: Startup, route, or render refactors can leave the launcher blank, unfocused, uninspectable, or on the wrong surface while still compiling.
- Proof commands:
  - `bun scripts/devtools/inspect.ts --session core-qa --start --show --main --surface ScriptList --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test smoke_main_menu -- --nocapture`
- Pass evidence: DevTools inspect receipt has classification ok, windowKind main, surface ScriptList, visible/focused true, and semantic element count greater than zero; smoke_main_menu passes.
- Fail evidence: Inspect returns blocked/error, no semantic elements, wrong surface, hidden/unfocused window, or the smoke test fails.

### Search selects and submits a script from the main launcher

- Stable anchor: `search-selects-and-submits-a-script-from-the-main-launcher`
- Choice id: `main-launcher-search-select-submit-script`
- Surface: Main Launcher / Script Execution
- Status: Active QA lock-in candidate
- User contract: Filtering commits rows for the current query/scope revision. Automatic selection follows the first eligible result; deliberate selection anchors identity, even when choosing the already-first row. Preview, footer, actions, and guarded submit use that same subject. Stale queries, removed rows, and stale semantic targets are refused before effects.
- Regression risk: Filtering caches, selection ownership, submit diagnostics, or script routing can drift and cause the wrong script to launch or no launch at all.
- Proof commands:
  - `bun scripts/devtools/devtools.ts build-ops act app-build --features owned-ui-evaluation --artifact-out .test-output/search-contract.reference.json`
  - `bun scripts/devtools/devtools.ts design run --artifact .test-output/search-contract.reference.json --out .test-output/search-contract-proof --scenario launcher-ranking-provider`
  - `./scripts/agentic/agent-cargo.sh test --locked -p sk-protocol --lib search_contract::tests`
- Pass evidence: The complete bounded selection inventory passes through real source completion, grouping, input dispatch, and natural completed GPUI frames with matching retained pixels. Headers and reserved slots remain inert; calculator navigation, preflight and copy completion agree. Dispatch acceptance, confirmed completion and refusal remain distinct; external effects remain truthfully refused in owned evaluation.
- Recorded bounded proof (2026-09-01): All 781 runnable version-2 search schedules passed, with one declared structural exclusion and complete owned cleanup. Receipt: `.test-output/search-contract-final-v20-full-20260901/receipt.json`. This does not qualify real external script/app execution.
- Fail evidence: Unintended second-row selection, stale selected/preview/footer subject, incorrect focus or viewport movement, stale-target acceptance, absent scheduled paint, blank/mismatched readback, uncovered cases, or invalid cleanup. Hidden proof does not claim a real external script or app was launched.

### Root built-in dispatch opens the requested built-in

- Stable anchor: `root-built-in-dispatch-opens-the-requested-built-in`
- Choice id: `trigger-builtin-from-root-is-stable`
- Surface: Main Launcher / Built-ins
- Status: Active QA lock-in candidate
- User contract: triggerBuiltin resolves canonical built-in ids case-insensitively, transitions to the expected built-in surface, and does not leave stale root state behind.
- Regression risk: Builtin registry, route, or deprecation work can silently break root commands or dispatch the wrong surface.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-clipboard","builtinId":"builtin/clipboard-history"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface ClipboardHistory --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test trigger_builtin_route_golden -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test trigger_builtin_dispatch_view_agnostic_contract -- --nocapture`
- Pass evidence: RPC result is ok, inspect lands on the requested built-in surface, and trigger builtin route/dispatch contracts pass.
- Fail evidence: Unknown/legacy id error, wrong surface, stale ScriptList still active, or route/dispatch contracts fail.

### Cmd+K actions target the current row

- Stable anchor: `cmd-k-actions-target-the-current-row`
- Choice id: `cmd-k-actions-popup-targets-current-row`
- Surface: Actions / Cmd+K in place
- Status: Active QA lock-in candidate
- User contract: ⌘K puts the highlighted row in the line above the input ("Acting on · …") and the list under the same input becomes that row's actions, led by a "Say what to do with it, or pick one" header. The row's own action comes first, so ⌘K then ↵ does what the row does; "Ask the model about it" is next. Typing filters the actions, ↵ runs the highlighted one on the staged row, and Esc lets go and highlights the row again. No window opens.
- Regression risk: Staging refactors can act on a different row than the line names, change what ↵ does after ⌘K, open a separate surface, or lose the highlighted row on Esc.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test properties`
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui enter_button_property_tests::staging`
  - `./scripts/agentic/agent-cargo.sh test --test main_window_actions_key_intent_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test actions_dialog_enter_routing_contract -- --nocapture`
  - `bash scripts/agentic/one-window/run.sh` (chapters `m2`, `p2-b` and `p2-k-*`)
- Pass evidence: Property and Enter-button tests pass; frames show the line reading "Acting on" with the row's name, the staged header leading the actions, ↵ naming the highlighted action, zero popups and the input at the same y before, during and after.
- Fail evidence: A popup window opens, the line names a different row than the actions act on, ↵ after ⌘K runs something other than the row's own action, Esc leaves a different row highlighted, or tests fail.

### Escape and focus loss preserve the right state

- Stable anchor: `escape-and-focus-loss-preserve-the-right-state`
- Choice id: `escape-go-back-hide-preserves-state`
- Surface: Main Launcher / Window Focus
- Status: Active QA lock-in candidate
- User contract: Escape lets go of what's staged, then goes back one view (a chat, a note, a form, an answer, an actions list), then clears the input, and only then hides the main window. Focus loss hides the one window without recording anything or losing the view it was on.
- Regression risk: Hide/reset refactors can skip a step of the Esc ladder, clear state too aggressively, or trap the user on a stale view.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test replay`
  - `./scripts/agentic/agent-cargo.sh test --test main_focus_loss_preserve_state_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test hide_rpc_surface_reset_contract -- --nocapture`
- Pass evidence: The replayed prototype traces agree on every Esc step, and the focus-loss and hide contracts pass.
- Fail evidence: Escape hides the window while something is staged or a view is open, leaves stale view or filter state, or contracts fail.

### Agent Chat accepts a prompt and exposes response lifecycle

- Stable anchor: `agent-chat-accepts-a-prompt-and-exposes-response-lifecycle`
- Choice id: `agent-chat-submit-stream-lifecycle`
- Surface: Agent Chat / Agent Chat
- Status: Active QA lock-in candidate
- User contract: The composer accepts text, Cmd+Enter/Enter submit is allowed only with proof intent, message count or awaiting-first-assistant state changes, and Agent Chat state remains targetable.
- Regression risk: Chat routing, submit gates, or streaming state can drift and make prompt submission unsafe or unobservable.
- Proof commands:
  - `bun scripts/devtools/inspect.ts --session core-qa --start --show --main --surface AgentChat --limit 200`
  - `bun scripts/devtools/act.ts set-input --text 'Core QA hello' --main --strict --surface AgentChat`
  - `bun scripts/devtools/act.ts key --key Enter --modifiers cmd --allow-submit --submit-intent agent-chat-route --allow-submit-reason core-qa-agent-chat-submit --main --strict --surface AgentChat`
  - `./scripts/agentic/agent-cargo.sh test --test agent_chat_runtime_seam_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test tab_ai_harness_submission -- --nocapture`
- Pass evidence: Receipts show composer text, guarded submit dispatch, Agent Chat/chat state transition, and chat harness/source contracts pass.
- Fail evidence: Submit blocked without clear gate, wrong target, no state transition, transcript unavailable, or contracts fail.

### Agent Chat context selector accepts @ items with receipts

- Stable anchor: `agent-chat-with-pi-backend-context-selector-accepts-items-with-receipts`
- Choice id: `agent_chat-context-selector-acceptance-receipts`
- Surface: Agent Chat / Context Selector
- Status: Active QA lock-in candidate
- User contract: Agent Chat state and test probe receipts show selector open, selected item, acceptedViaKey, cursorAfter, contextChipCount, and final input layout after acceptance.
- Regression risk: Selector keyboard routing and context insertion are easy to break during composer or context refactors.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"resetAgentChatTestProbe","requestId":"core-qa-reset-probe","target":{"type":"main"}}' --expect agent_chatTestProbeResult --timeout 8000`
  - `bun scripts/devtools/act.ts set-input --text '@' --main --strict --surface AgentChat`
  - `bun scripts/devtools/act.ts key --key Tab --main --strict --surface AgentChat`
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"getAgentChatTestProbe","requestId":"core-qa-probe","tail":4,"target":{"type":"main"}}' --expect agent_chatTestProbeResult --timeout 8000`
- Pass evidence: Probe receipt contains keyRoutes and acceptedItems with acceptedViaKey tab/enter, cursorAfter, and context state.
- Fail evidence: Selector never opens, acceptedItems stays empty, cursor/chip count is wrong, event route is propagated incorrectly, or contract fails.

### Agent Chat opens under the main input and past chats reopen in place

- Stable anchor: `detached-agent-chat-with-pi-backend-targets-reattaches-and-cleans-up`
- Choice id: `agent_chat-detached-targeting-and-close-cleanup`
- Surface: Agent Chat / Past chats
- Status: Active QA lock-in candidate
- User contract: Every way into the model's chat (the launcher, ⌘↵, a script error, the SDK, stdin, the tray, dictation, a note) opens the one chat under the main input in the main window. What the chat is about waits in the line above the input and goes with the first turn. Past chats are rows under the main input that filter as you type and reopen in place; deleting one waits 8 s in the line and ⌘Z stops it.
- Regression risk: Chat entry points can drift back to a separate window, lose what was staged before the first turn, or reopen the wrong conversation.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test work_replay`
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui one_window::past_chats`
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui enter_button_property_tests::chat`
  - `./scripts/agentic/agent-cargo.sh test --test chat_surface_behavior_contract -- --nocapture`
  - `bash scripts/agentic/one-window/run.sh` (chapter `p2-w-chat`)
- Pass evidence: Chat replay and unit tests pass; the chat chapter's frames show the chat under the same input, past chats as rows under a header, zero popups and the input at the same y.
- Fail evidence: A second window appears, the staged subject is missing from the first turn, a past chat opens a different conversation, a delete happens before the wait runs out, or tests fail.

### Notes open under the main input and keep their text

- Stable anchor: `notes-edit-state-survives-main-window-activity`
- Choice id: `notes-edit-persist-main-hide-isolation`
- Surface: Notes (view in the main window)
- Status: Active QA lock-in candidate
- User contract: Notes open as a view under the main input, today's note by default. Lines keep their markup apart from their words, talking or typing to today's note adds the words, and Esc goes back to the list without losing the note's text.
- Regression risk: View refactors can reopen Notes in a window of its own, drop unsaved text on Esc, or move the input when the note grows.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui one_window::notes`
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui enter_button_property_tests::notes`
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test talk talking_to_todays_note_adds_the_words`
  - `./scripts/agentic/agent-cargo.sh test --test mcp_notes_tools -- --nocapture`
  - `bash scripts/agentic/one-window/run.sh` (chapter `p2-c-notes`)
- Pass evidence: Note tests pass; the notes chapter's frames show note lines under the same input, zero popups and the input at the same y through opening, typing and Esc.
- Fail evidence: A Notes window opens, words are lost after Esc, the input moves, or tests fail.

### Dictation fixture delivery records a safe target receipt

- Stable anchor: `dictation-fixture-delivery-records-a-safe-target-receipt`
- Choice id: `dictation-fixture-delivery-target-receipts`
- Surface: Dictation
- Status: Active QA lock-in candidate
- User contract: pushDictationResult advances lastDelivery generation, records target/destination/insertionRange, stores transcript fingerprint/length, and never returns raw transcript content.
- Regression risk: Dictation delivery touches cross-window text insertion and privacy-sensitive receipts, both easy to regress.
- Proof commands:
  - `bun scripts/devtools/dictation.ts deliver-fixture --session core-qa --start --show --target mainWindowFilter --fixture-id short-phrase`
  - `bun scripts/devtools/dictation.ts deliver-fixture --session core-qa --target notesEditor --fixture-id punctuation`
  - `./scripts/agentic/agent-cargo.sh test --test push_dictation_result_stub_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test dictation_lifecycle_contract -- --nocapture`
- Pass evidence: Dictation receipts classify ok or a specific missing primitive, include redacted fingerprint/length and target match, and contracts pass.
- Fail evidence: Raw transcript leaks, generation does not advance, target mismatches, insertion range missing without explicit blocker, or contracts fail.

### Clipboard history search, preview, pin, and delete work

- Stable anchor: `clipboard-history-search-preview-pin-and-delete-work`
- Choice id: `clipboard-history-search-preview-actions`
- Surface: Clipboard History
- Status: Active QA lock-in candidate
- User contract: Clipboard History exposes filter-aware elements, preview type metadata, pin/delete/bulk-delete receipts, and stable empty-state behavior.
- Regression risk: Clipboard is high-frequency and prone to stale filters, wrong preview types, and accidental destructive action drift.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-clip-open","builtinId":"builtin/clipboard-history"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface ClipboardHistory --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test clipboard_history_getelements_filter_aware_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test clipboard_history_preview_type_filters_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test clipboard_bulk_delete_contract -- --nocapture`
- Pass evidence: Inspect shows ClipboardHistory elements/preview state and clipboard contracts pass for filter-aware rows, preview types, and bulk delete safety.
- Fail evidence: Surface cannot open, elements ignore filters, preview type mismatches row type, destructive receipt missing, or contracts fail.

### File Search handles paths, search, selection, and verbs

- Stable anchor: `file-search-handles-paths-search-selection-and-verbs`
- Choice id: `file-search-path-search-select-verbs`
- Surface: File Search
- Status: Active QA lock-in candidate
- User contract: File Search resolves tilde/start paths, refreshes after mutations, exposes selectable file rows, and actions/drag verbs carry the exact selected path.
- Regression risk: Filesystem surfaces commonly regress around path expansion, stale rows after mutation, and wrong-path actions.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-file-open","builtinId":"builtin/file-search"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface FileSearch --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test file_search_tilde_entry -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test file_search_drag_and_verbs -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test file_search_mutation_refresh -- --nocapture`
- Pass evidence: Inspect shows FileSearch rows/preview and source contracts pass for tilde entry, selected-path verbs, drag metadata, and mutation refresh.
- Fail evidence: Wrong path expansion, no rows/preview, selected action path mismatch, stale mutation state, or contract failure.

### Stdin batch transactions produce replay-safe traces

- Stable anchor: `stdin-batch-transactions-produce-replay-safe-traces`
- Choice id: `stdin-batch-transaction-trace-idempotent`
- Surface: Stdin Protocol / Transactions
- Status: Active QA lock-in candidate
- User contract: Batch commands return ordered results, failedAt is correct, trace includes before/after snapshots and command_fingerprint, and same requestId with same payload is idempotent.
- Regression risk: Automation protocol drift breaks every DevTools/runtime receipt and makes QA locks unreliable.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"batch","requestId":"core-qa-batch-script","target":{"type":"main"},"commands":[{"type":"setInput","text":"script"},{"type":"waitFor","condition":"choicesRendered","timeout":5000}],"trace":"on"}' --expect batchResult --timeout 8000`
  - `./scripts/agentic/agent-cargo.sh test --test protocol_batch -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test tx_trace_replay_idempotency_contract -- --nocapture`
- Pass evidence: batchResult success has per-command results and trace, replay returns same trace or conflict only for changed payload, and protocol tests pass.
- Fail evidence: Missing trace, wrong command order, non-idempotent replay, requestId conflict mishandled, or tests fail.

### Automation targets the intended window, not the focused accident

- Stable anchor: `automation-targets-the-intended-window-not-the-focused-accident`
- Choice id: `automation-window-targeting-strict-all-core-windows`
- Surface: Automation / Window Targeting
- Status: Active QA lock-in candidate
- User contract: listAutomationWindows and inspectAutomationWindow resolve explicit targets, include window id/kind/bounds/osWindowId when available, and never silently fall back to the wrong focused window. The app has one window, so every Script Kit surface resolves to the main window.
- Owned search target contract: The caller's observed identity remains authoritative across client calls and deferred dispatch. A refreshed observation never silently replaces an explicit stale expectation. Pointer actions bind to the completed geometry frame; window retirement invalidates old actions, provider attachments, and retained frames.
- Regression risk: Any runtime QA suite depends on strict targeting; target drift makes receipts false positives.
- Proof commands:
  - `bun scripts/devtools/surfaces.ts`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --strict --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test automation_window_targeting -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test screenshot_identity_threading_contract -- --nocapture`
- Pass evidence: Receipts show exact target identity for the main window and tests pass for target resolution and screenshot identity.
- Fail evidence: Target resolves to the wrong window, id/kind missing, screenshot identity mismatches, or contracts fail.

### Permission preflight guides the user without surprise prompts

- Stable anchor: `permission-preflight-guides-the-user-without-surprise-prompts`
- Choice id: `permissions-preflight-guides-user-without-surprise-prompts`
- Surface: Permissions / Preflight
- Status: Active QA lock-in candidate
- User contract: Permission state is visible in getState/mainWindowPreflight or setup snapshots, actions are explicit, and source audits confirm no hidden prompt path bypasses the permission guide.
- Regression risk: Permission regressions are high-risk because they block core workflows and can surprise users with native prompts.
- Proof commands:
  - `bun scripts/devtools/inspect.ts --session core-qa --start --show --main --surface ScriptList --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test permission_guide_assistant_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test platform_accessibility_contract -- --nocapture`
- Pass evidence: Inspect receipt exposes preflight/permission state when relevant and source contracts pass for permission guide, AI preflight guard, and accessibility wiring.
- Fail evidence: Missing permission has no surfaced guidance, hidden prompt path appears, preflight is stale, or contracts fail.

### Main input sigils project the right catalogs

- Stable anchor: `main-input-sigils-project-the-right-catalogs`
- Choice id: `spine-sigils-project-main-input-catalogs`
- Surface: Main Input / Spine
- Status: Active QA lock-in candidate
- User contract: Each sigil parses to the expected segment kind, renders the expected catalog rows, and accepts insertion/resolution without corrupting free-text tail input.
- Regression risk: Prompt-builder grammar and row projection can drift subtly and make common shortcuts unusable.
- Feedback lock: `>` must not select cwd. Tab is the cwd trigger.
- Proof commands:
  - `bun scripts/devtools/act.ts set-input --text '@file:' --session core-qa --main --strict --surface ScriptList`
  - `bun scripts/devtools/elements.ts snapshot --session core-qa --main --strict --surface ScriptList --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --lib spine -- --nocapture`
- Pass evidence: Runtime elements show the projected catalog for the active sigil and spine unit tests pass for parse/projection/list rows.
- Fail evidence: Wrong catalog, broken highlighting/span range, lost tail text, or spine tests fail.

### Root source filters browse unified launcher sources

- Stable anchor: `root-source-filters-browse-unified-launcher-sources`
- Choice id: `root-unified-source-filters-browse-and-actions`
- Surface: Main Launcher / Unified Sources
- Status: Active QA lock-in candidate
- User contract: Source filter chips or syntax update visible rows, source metadata, action policies, and passive snapshots without mixing rows from inactive query/scope revisions. Implicit global Files is cache-only; explicit Files/directory and other admitted publishers commit through the same interaction-preserving boundary. Empty, unavailable, failed, cancelled, and stale outcomes remain distinct.
- Regression risk: Unified source refactors can bleed actions and rows between source domains, causing wrong launches or stale previews.
- Proof commands:
  - `bun scripts/devtools/devtools.ts design run --artifact .test-output/search-contract.reference.json --out .test-output/search-source-proof --scenario launcher-ranking-provider`
  - `./scripts/agentic/agent-cargo.sh test --locked -p sk-protocol --lib filter_coalescer::tests`
  - `./scripts/agentic/agent-cargo.sh test --locked -p sk-protocol --lib search_contract::tests`
- Pass evidence: Controlled source runs exercise production publication across query supersession, ABA, compatible work reuse, scope changes, completion order, row removal/content replacement, and empty/error states. The receipt accounts for every declared provider/schedule and distinguishes cache mutation from visible row publication.
- Recorded bounded proof (2026-09-01): The same complete version-2 receipt covers all 20 controlled root providers, including both publisher orders, same-turn/cohort comparisons, scope fences, and distinct terminal outcomes; no schedule was reduced or blocked. See `scripts/devtools/README.md` for artifact and proof limits.
- Fail evidence: A retired run changes current rows, one source releases another's worker, missing/error data becomes success, stale input executes, rows/actions cross source boundaries, or required cases lack qualified proof.

### App Launcher searches apps and reports activation feedback

- Stable anchor: `app-launcher-searches-apps-and-reports-activation-feedback`
- Choice id: `app-launcher-search-activation-feedback`
- Surface: App Launcher
- Status: Active QA lock-in candidate
- User contract: App rows are visible and selectable, activation attempts produce feedback, and empty-state behavior is explicit when no apps match.
- Regression risk: App launch is a core user path that can regress through filtering, action feedback, or empty-state changes.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-app-launcher","builtinId":"builtin/app-launcher"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface AppLauncher --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test app_launcher_visible_rows_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test app_launcher_activation_feedback_contract -- --nocapture`
- Pass evidence: AppLauncher inspect shows rows or empty state and contracts pass for visible rows and activation feedback.
- Fail evidence: Surface fails to open, rows lack selectable app metadata, activation gives no feedback, or contracts fail.

### Current App Commands shows safe commands for the frontmost app

- Stable anchor: `current-app-commands-shows-safe-commands-for-the-frontmost-app`
- Choice id: `current-app-commands-visible-actions`
- Surface: Current App Commands
- Status: Active QA lock-in candidate
- User contract: Rows include frontmost-app command metadata, visible/empty states are explicit, and action sources match current-app command audits.
- Regression risk: Frontmost app observation and command action wiring can drift, making commands target the wrong application.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-current-app","builtinId":"builtin/current-app-commands"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface CurrentAppCommands --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test current_app_commands_visible_rows_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test current_app_commands_surface_contract -- --nocapture`
- Pass evidence: Inspect receipt shows CurrentAppCommands rows/empty state and source contracts pass.
- Fail evidence: Wrong app/source metadata, invisible rows without empty state, unsafe actions, or test failures.

### Window Switcher lists windows and handles empty state

- Stable anchor: `window-switcher-lists-windows-and-handles-empty-state`
- Choice id: `window-switcher-list-empty-and-trigger`
- Surface: Window Switcher
- Status: Active QA lock-in candidate
- User contract: Window rows include app/title/window id metadata, selection action targets the same window id, and empty state is explicit.
- Regression risk: Window switching can silently target stale ids or disappear behind platform-specific observation changes.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-window-switcher","builtinId":"builtin/window-switcher"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface WindowSwitcher --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test window_switcher_triggerbuiltin_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test window_switcher_empty_state_contract -- --nocapture`
- Pass evidence: WindowSwitcher inspect shows row metadata or empty state and trigger/empty contracts pass.
- Fail evidence: Rows lack ids, selection targets wrong window, empty state missing, or contracts fail.

### Browser tabs and history search are visible and fail safe

- Stable anchor: `browser-tabs-and-history-search-are-visible-and-fail-safe`
- Choice id: `browser-tabs-history-search-and-empty-states`
- Surface: Browser Tabs / Browser History
- Status: Active QA lock-in candidate
- User contract: Browser built-ins expose filter-aware rows with source metadata, open actions target the selected item, and empty states are explicit.
- Regression risk: Browser providers vary by environment; surfaces must not look broken when data is unavailable.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-browser-tabs","builtinId":"builtin/browser-tabs"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface BrowserTabs --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test collect_elements_browser_tabs_arm_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test browser_tabs_empty_state_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test browser_history_empty_state_contract -- --nocapture`
- Pass evidence: Inspect and contracts show rows or clear empty state for tabs/history with correct source semantics.
- Fail evidence: Provider absence yields blank UI, actions target wrong URL/tab, filter-aware elements break, or contracts fail.

### Actions list keyboard navigation keeps a valid selection

- Stable anchor: `actions-dialog-keyboard-navigation-keeps-a-valid-selection`
- Choice id: `actions-dialog-keyboard-selection-invariants`
- Surface: Actions list (in place)
- Status: Active QA lock-in candidate
- User contract: In the actions list ⌘K puts under the input, arrow navigation skips headers, the highlight never leaves selectable rows, ↵ runs the highlighted action according to its policy, and Esc lets go without changing the list underneath.
- Regression risk: Keyboard handling regressions make actions feel broken and can run the wrong action.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --test actions_dialog_arrow_nav_skips_section_headers_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test actions_dialog_selection_clamps_to_item_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test actions_dialog_route_stack_contract -- --nocapture`
  - `bash scripts/agentic/one-window/run.sh` (chapters `p2-b` and `p2-k-*`)
- Pass evidence: Navigation and clamping contracts pass; frames show the yellow tile on a selectable action row after every arrow key.
- Fail evidence: The highlight lands on a header or goes out of bounds, ↵ runs the wrong action, Esc changes the list underneath, or tests fail.

### Undoable actions run at once and irreversible ones wait in the line

- Stable anchor: `actions-lifecycle-preserves-parent-or-closes-only-the-correct-host`
- Choice id: `actions-lifecycle-parent-child-close-rules`
- Surface: Actions / Undo and hold in the line
- Status: Active QA lock-in candidate
- User contract: An action that can be undone runs at once and the line above the input says what ran with ⌘Z to take it back. One that can't be undone waits in the line ("Runs in 8s · …"), pauses while you type, runs when the wait ends, and ⌘Z stops it. Nothing asks in a dialog, and nothing shows a toast or HUD.
- Regression risk: Action lifecycle changes can bring back confirm dialogs or toasts, run an irreversible action at once, or commit a held action twice.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test properties`
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test replay`
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui enter_button_property_tests::ledger`
  - `bash scripts/agentic/one-window/run.sh` (chapter `p2-a`)
- Pass evidence: The held-message properties (one commit per hold, ⌘Z stops it, typing pauses it) and trace replay pass; frames show the countdown in the line, zero popups, and the input at the same y.
- Fail evidence: A dialog, toast or HUD appears, an irreversible action runs before its wait, ⌘Z fails to stop it, a hold commits twice, or tests fail.

### Mini mode toggles, sizes, and reveals filtered rows

- Stable anchor: `mini-mode-toggles-sizes-and-reveals-filtered-rows`
- Choice id: `mini-mode-toggle-size-filter-reveal`
- Surface: Mini Mode
- Status: Experimental/WIP documentation only
- User contract: Mini window reports correct semantic surface, sizing constraints, focused input, filtered-row reveal, and clean reset on close/toggle.
- Regression risk: Mini mode touches alternate chrome and input routing, making it easy to regress during launcher changes.
- Feedback lock: keep this as experimental/WIP documentation until Mini Mode behavior is intentionally stabilized.
- Proof commands:
  - `bun scripts/devtools/surface.ts inspect --session core-qa --start --show --surface Mini --main`
  - `./scripts/agentic/agent-cargo.sh test --test mini_mode_toggle_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test mini_sizing_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test mini_filter_reveal_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test mini_close_reset_contract -- --nocapture`
- Pass evidence: Surface receipt shows mini state and contracts pass for toggle, sizing, filtering, and close reset.
- Fail evidence: Wrong surface/chrome, focus lost, filtered row hidden, close leaves stale state, or tests fail.

### Focused-text inline agent captures, applies, and copies safely

- Stable anchor: `focused-text-inline-agent-captures-applies-and-copies-safely`
- Choice id: `focused-text-inline-agent-apply-copy-receipts`
- Surface: Inline Agent / Focused Text
- Status: Active QA lock-in candidate
- User contract: Focused text snapshot, submitted prompt lock, output state, replace/append/copy action receipt, and privacy flags are present and target the captured session id.
- Regression risk: Focused text mutation is user-sensitive and can corrupt external app text or leak context if receipts drift.
- Proof commands:
  - `bun scripts/devtools/inspect.ts --session core-qa --start --show --main --surface InlineAgent --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test focused_text_prompt_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test focused_text_agent_chat_action_receipt_contract -- --nocapture`
- Pass evidence: Inspect/protocol receipts expose focusedText state and action receipts; source contracts pass for prompt, actions, and protocol wiring.
- Fail evidence: Session id missing, mutation targets wrong app, private text leaked, copy/replace receipt absent, or tests fail.

### Context selector and Add to AI attach selected sources

- Stable anchor: `context-selector-and-add-to-ai-attach-selected-sources`
- Choice id: `context-selector-add-to-ai-end-to-end`
- Surface: Context Selector / Add to AI
- Status: Active QA lock-in candidate
- User contract: Context part resolution produces source-specific attachments, composer state records chips, preflight blocks unavailable sources, and submission carries the expected context parts.
- Regression risk: Context handoff connects many sources; drift can silently drop or mislabel what the user asked AI to use.
- Proof commands:
  - `bun scripts/devtools/inspect.ts --session core-qa --start --show --main --surface AgentChat --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test context_contract_end_to_end -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test context_part_submission_flow -- --nocapture`
- Pass evidence: Receipts/state show attached context chips and context contracts pass end-to-end.
- Fail evidence: Context source missing/mislabeled, unavailable source bypasses preflight, chips lost before submit, or contracts fail.

### Agent Chat history can resume and export conversations

- Stable anchor: `agent-chat-history-can-resume-and-export-conversations`
- Choice id: `agent_chat-history-resume-export`
- Surface: Agent Chat / Past chats
- Status: Active QA lock-in candidate
- User contract: Past chats are rows under the main input, each saying when it was in words. Filtering finds them, ↵ reopens the selected conversation in place with your turns, the model's and its tools, and export emits one canonical path with transcript metadata.
- Regression risk: History and export refactors can duplicate export paths, resume the wrong thread, or lose message metadata.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui one_window::past_chats`
  - `./scripts/agentic/agent-cargo.sh test --test agent_chat_existing_chat_mutation_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test agent_chat_conversation_export_single_path_contract -- --nocapture`
- Pass evidence: Past-chat tests pass for dates and saved turns, and contracts pass for resume, mutation and single-path export.
- Fail evidence: Wrong conversation resumed, saved turns lost, duplicate export routes, or tests fail.

### Agent Chat setup and onboarding offer recoverable actions

- Stable anchor: `agent-chat-with-pi-backend-setup-and-onboarding-offer-recoverable-actions`
- Choice id: `agent_chat-setup-onboarding-agent-picker`
- Surface: Agent Chat / Setup
- Status: Active QA lock-in candidate
- User contract: Agent Chat setup state exposes reasonCode/title/body/primaryAction, setup actions return updated state, and agent picker open/select/close states are observable.
- Regression risk: Setup regressions can strand users when agents are missing or auth/capabilities change.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"getAgentChatState","requestId":"core-qa-agent-chat-pi-backend-setup","target":{"type":"main"}}' --expect agent_chatStateResult --timeout 8000`
  - `./scripts/agentic/agent-cargo.sh test --test agent_chat_onboarding -- --nocapture`
- Pass evidence: Agent Chat state receipt has setup details when applicable and onboarding/preflight/thread-affinity contracts pass.
- Fail evidence: Setup state absent for blocked condition, actions do nothing, picker selection stale, or tests fail.

### Agent Chat streaming can cancel and survive config reloads

- Stable anchor: `agent-chat-with-pi-backend-streaming-can-cancel-and-survive-config-reloads`
- Choice id: `agent_chat-streaming-cancel-config-reload`
- Surface: Agent Chat / Streaming
- Status: Active QA lock-in candidate
- User contract: Streaming status transitions to canceled/idle, subscriptions are cleaned up, config reload does not duplicate providers, and transcript remains consistent.
- Regression risk: Streaming is asynchronous; cancel and config changes commonly leak tasks or double-send events.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"aiGetStreamingStatus","requestId":"core-qa-streaming"}' --expect aiStreamingStatusResult --timeout 8000`
  - `./scripts/agentic/agent-cargo.sh test --test agent_chat_cancel_midstream_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test config_reload_during_streaming_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test agent_chat_live_subscription_contract -- --nocapture`
- Pass evidence: RPC/contract output shows streaming status and cleanup invariants pass for cancel, config reload, and live subscriptions.
- Fail evidence: Cancel ignored, duplicate subscription events, transcript corruption, provider reload drift, or tests fail.

### Asking about a note opens the one chat with the note staged

- Stable anchor: `notes-agent-chat-with-pi-backend-actions-match-chat-behavior-without-leaking-host-state`
- Choice id: `notes-agent_chat-actions-history-parity`
- Surface: Notes / Agent Chat
- Status: Active QA lock-in candidate
- User contract: Asking the model about a note opens the one chat under the main input with the note waiting in the line above it; the note's words go with the first turn and the note itself is unchanged. There is no second chat host inside Notes.
- Regression risk: Note-to-chat handoff can lose the note as context, send it twice, or open a chat host of its own.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui enter_button_property_tests::chat`
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui one_window::chat_entry`
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test talk talking_to_the_model_opens_its_chat_with_the_words`
- Pass evidence: Chat-entry tests show the note staged in the line and sent once with the first turn.
- Fail evidence: The note is missing from the first turn, sent twice, or a separate chat host appears, or tests fail.

### Dictation talks in the main window and reads typing or a request

- Stable anchor: `dictation-setup-history-and-overlay-lifecycle-are-coherent`
- Choice id: `dictation-setup-history-overlay-lifecycle`
- Surface: Dictation (in the main window)
- Status: Active QA lock-in candidate
- User contract: Holding the voice key talks in the main window; the line says "Typing into · <app>" or what the words will do. Words that read as writing paste at the cursor of the frontmost app and the window goes; words that read as a request become a sentence in the input. The reading changes at most once while you talk, and letting go acts only when it is sure. Setup, stop and recovery stay in the same window.
- Regression risk: Dictation has async states that can desync from the window: a reading can flip back and forth, act while unsure, or bring back an overlay.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test talk`
  - `./scripts/agentic/agent-cargo.sh test -p sk-one-window --test properties voice`
  - `./scripts/agentic/agent-cargo.sh test --test dictation_setup_nux_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test dictation_lifecycle_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test dictation_stop_liveness_contract -- --nocapture`
  - `bash scripts/agentic/one-window/run.sh` (chapter `p2-e`)
- Pass evidence: Talk tests and voice properties pass; frames show the voice rows under the same input, zero popups and the input at the same y.
- Fail evidence: An overlay or popup appears, the reading changes more than once, letting go acts while unsure, stop never completes, or tests fail.

### Settings exposes visible rows, empty state, and permission routes

- Stable anchor: `settings-exposes-visible-rows-empty-state-and-permission-routes`
- Choice id: `settings-visible-empty-state-and-permission-routes`
- Surface: Settings
- Status: Active QA lock-in candidate
- User contract: Settings visible rows, empty state, permission guide entries, and config fingerprints are present and stable under filtering.
- Regression risk: Settings is the recovery surface for many failures; hidden rows or stale config can block users.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-settings","builtinId":"builtin/settings"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface Settings --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test settings_visible_rows_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test settings_empty_state_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test get_config_fingerprint_contract -- --nocapture`
- Pass evidence: Settings inspect shows rows or empty state and contracts pass for visible rows, empty state, and config fingerprint.
- Fail evidence: Rows disappear, search yields blank without empty state, config fingerprint stale, or tests fail.

### Process Manager lists processes and handles triggers safely

- Stable anchor: `process-manager-lists-processes-and-handles-triggers-safely`
- Choice id: `process-manager-list-empty-trigger-actions`
- Surface: Process Manager
- Status: Active QA lock-in candidate
- User contract: Process rows expose safe metadata, actions are gated, and empty/feedback states are explicit.
- Regression risk: Process management can become dangerous if actions target stale rows or feedback is missing.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-process-manager","builtinId":"builtin/process-manager"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface ProcessManager --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test process_manager_visible_rows_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test process_manager_empty_state_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test trigger_builtin_process_manager_contract -- --nocapture`
- Pass evidence: Inspect shows ProcessManager row/empty state and contracts pass for visible rows, trigger dispatch, and empty state.
- Fail evidence: Unsafe action target, no feedback, blank surface, or contract failure.

### Theme chooser controls persist and keep keyboard routing correct

- Stable anchor: `theme-chooser-controls-persist-and-keep-keyboard-routing-correct`
- Choice id: `theme-chooser-controls-persist-and-propagate-keys`
- Surface: Theme Chooser
- Status: Active QA lock-in candidate
- User contract: Theme control changes produce state receipts, persist selected design/theme, and key propagation remains scoped to the theme surface.
- Regression risk: Theme UI refactors can break persistence or route keys to the launcher accidentally.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-theme","builtinId":"builtin/theme-chooser"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/act.ts set-theme-control --control theme --value light --session core-qa --main --strict --surface ThemeChooser`
  - `./scripts/agentic/agent-cargo.sh test --test theme_chooser_key_propagation_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test theme_contrast_audit -- --nocapture`
- Pass evidence: Theme control receipt succeeds, selected theme/design state is persisted, and key/contrast contracts pass.
- Fail evidence: Control mutation unsupported without blocker, keys leak to parent, persistence missing, or tests fail.

### SDK Reference and Script Templates expose stable text states

- Stable anchor: `sdk-reference-and-script-templates-expose-stable-text-states`
- Choice id: `sdk-reference-and-script-templates-text-states`
- Surface: SDK Reference / Script Templates
- Status: Active QA lock-in candidate
- User contract: Developer help built-ins expose visible rows, preview/text state, and stable trigger dispatch.
- Regression risk: Help surfaces are less destructive but regressions make users think APIs/templates are missing.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-sdk-ref","builtinId":"builtin/sdk-reference"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface SdkReference --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test sdk_reference_text_state_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test script_templates_text_state_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test template_prompt_parity_contract -- --nocapture`
- Pass evidence: Inspect shows help/template text state and contracts pass.
- Fail evidence: Wrong surface, missing preview/text state, template parity drift, or tests fail.

### Emoji Picker filters, navigates, and handles empty results

- Stable anchor: `emoji-picker-filters-navigates-and-handles-empty-results`
- Choice id: `emoji-picker-empty-arrow-and-selection`
- Surface: Emoji Picker
- Status: Active QA lock-in candidate
- User contract: Emoji rows filter correctly, arrow up/down selection is stable, and no-result state is visible.
- Regression risk: Emoji picker drift is low risk but exposes shared list/input behavior used elsewhere.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-emoji","builtinId":"builtin/emoji-picker"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface EmojiPicker --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test emoji_picker_arrow_up_down_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test emoji_picker_empty_state_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test emoji_picker_state_choice_count_asymmetry_contract -- --nocapture`
- Pass evidence: EmojiPicker inspect shows rows/empty state and arrow/choice-count contracts pass.
- Fail evidence: Selection jumps, empty state missing, choice count lies, or tests fail.

### Design catalog stays unique and designs persist

- Stable anchor: `design-gallery-and-picker-browse-persist-and-resize`
- Choice id: `design-gallery-picker-browse-persist-resize`
- Surface: Designs / Theme
- Status: Active QA lock-in candidate
- User contract: Design ids in the catalog are unique, the designs config schema matches what the app reads, and legacy design variants migrate instead of lingering.
- Regression risk: Design surfaces share list/layout code and can reveal drift in noncritical appearance flows.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --test design_catalog_uniqueness -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test designs_config_schema_parity -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test legacy_design_variant_migration -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test no_legacy_design_variant -- --nocapture`
- Pass evidence: Catalog, schema and migration contracts pass.
- Fail evidence: Duplicate design ids, schema drift, a legacy variant left unmigrated, or tests fail.

### Quick Terminal opens warm and closes cleanly

- Stable anchor: `quick-terminal-opens-warm-and-closes-cleanly`
- Choice id: `quick-terminal-open-warm-lifecycle`
- Surface: Quick Terminal
- Status: Active QA lock-in candidate
- User contract: Quick Terminal warm state is observable, target surface is correct, and close resets only terminal-specific state.
- Regression risk: Terminal warm/start paths can leak tasks or steal focus from the main window.
- Proof commands:
  - `bun scripts/devtools/act.ts set-input --text '> echo core-qa' --session core-qa --main --strict --surface ScriptList`
  - `./scripts/agentic/agent-cargo.sh test --test quick_terminal_contracts -- --nocapture`
- Pass evidence: Runtime command records mode-exit input and quick terminal contracts pass.
- Fail evidence: Terminal does not open, focus remains stolen, warm lifecycle leaks, or contract fails.

### Path prompt handles tilde and filesystem edge cases

- Stable anchor: `path-prompt-handles-tilde-and-filesystem-edge-cases`
- Choice id: `path-prompt-tilde-filesystem-edges`
- Surface: Path Prompt
- Status: Active QA lock-in candidate
- User contract: Path prompt expands tilde, clamps to valid filesystem state, exposes selected path state, and reports invalid paths safely.
- Regression risk: Path prompts are common in scripts and can break through filesystem/path normalization changes.
- Proof commands:
  - `bun scripts/devtools/act.ts set-input --text '~/' --session core-qa --main --strict --surface ScriptList`
  - `./scripts/agentic/agent-cargo.sh test --test path_prompt_filesystem_edges_contract -- --nocapture`
- Pass evidence: Path-related state/contract tests pass and runtime input path handling stays stable.
- Fail evidence: Tilde not expanded, invalid path panics, selected path missing, or contracts fail.

### Drop prompt accepts native file drops safely

- Stable anchor: `drop-prompt-accepts-native-file-drops-safely`
- Choice id: `drop-prompt-native-file-drop`
- Surface: Drop Prompt
- Status: Active QA lock-in candidate
- User contract: Drop state records files or rejection reason, and native drop routing does not leak into unrelated surfaces.
- Regression risk: Drag/drop is platform-sensitive and easy to break without regular proof.
- Proof commands:
  - `bun scripts/devtools/inspect.ts --session core-qa --start --show --main --surface DropPrompt --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test drop_prompt_native_drop_contract -- --nocapture`
- Pass evidence: Inspect/drop state is present when prompt active and native drop contract passes.
- Fail evidence: Drop ignored, files attached to wrong surface, rejection silent, or contract fails.

### Shortcut view captures keys and shows errors clearly

- Stable anchor: `shortcut-recorder-captures-keys-and-shows-errors-clearly`
- Choice id: `shortcut-recorder-captures-errors-and-popup-state`
- Surface: Shortcut view (in the main window)
- Status: Active QA lock-in candidate
- User contract: Giving something a shortcut (a saved run, a script, the SDK's hotkey()) opens the shortcut view under the main input with a "Press the keys for …" header. It records normalized keycaps, names the saved script, and shows validation errors without corrupting existing shortcut config. Esc goes back.
- Regression risk: Keyboard shortcut changes can break recording, display, or conflict validation, or bring back a popup recorder.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui one_window::shortcut_view`
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui one_window::learns`
  - `./scripts/agentic/agent-cargo.sh test --test shortcut_error_messages -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test shortcut_keycap_unification_contract -- --nocapture`
  - `bash scripts/agentic/one-window/run.sh` (chapters `p2-e` and `p2-z-hotkey`)
- Pass evidence: Shortcut tests pass for capture, naming, error messages and keycap normalization; frames show the shortcut view under the same input with zero popups.
- Fail evidence: Shortcut not recorded, invalid shortcut silently accepted, a popup opens, or tests fail.

### About and Help render stable contracts, and asks happen in the line

- Stable anchor: `about-help-and-confirm-surfaces-render-stable-contracts`
- Choice id: `about-help-confirm-surfaces-contracts`
- Surface: About / Help
- Status: Active QA lock-in candidate
- User contract: About/Help surfaces expose expected text/actions under the main input and keep focus/keyboard contracts stable. Nothing asks in a Confirm dialog: SDK `confirm()` stages its question in the line and its answers become rows, and a destructive one holds for 8 s with ⌘Z to stop it.
- Regression risk: Small shared surfaces catch regressions in prompt chrome, buttons, and focus policies.
- Proof commands:
  - `bash scripts/agentic/session.sh rpc core-qa '{"type":"triggerBuiltin","requestId":"core-qa-about","builtinId":"builtin/about"}' --expect triggerBuiltinResult --timeout 8000`
  - `bun scripts/devtools/inspect.ts --session core-qa --main --surface About --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test about_surface_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test help_info_surface_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --bin script-kit-gpui enter_button_property_tests::ledger`
- Pass evidence: Inspect shows informational surface content and contracts pass for About and Help; ledger tests pass for asks and holds in the line.
- Fail evidence: Missing content/actions, wrong focus, a confirm dialog appears, or tests fail.

### DevTools inspect, elements, layout, and measure produce trustworthy receipts

- Stable anchor: `devtools-inspect-elements-layout-and-measure-produce-trustworthy-receipts`
- Choice id: `devtools-inspect-elements-layout-measure-truth`
- Surface: Script Kit DevTools
- Status: Active QA lock-in candidate
- User contract: inspect/elements/layout/measure agree on target identity and report missing primitives explicitly instead of false success.
- Regression risk: The QA program depends on DevTools receipts; false positives or inconsistent target identity invalidate later locked tests.
- Proof commands:
  - `bun scripts/devtools/inspect.ts --session core-qa --start --show --main --strict --surface ScriptList --limit 200 > /tmp/core-qa-inspect.json`
  - `bun scripts/devtools/elements.ts snapshot --session core-qa --main --strict --surface ScriptList --limit 200`
  - `bun scripts/devtools/layout.ts measure --session core-qa --main --strict --surface ScriptList --include nodes,regions,scroll,anchors,resize,overlaps --limit 200`
  - `bun scripts/devtools/measure.ts --inspect /tmp/core-qa-inspect.json --surface main`
  - `./scripts/agentic/agent-cargo.sh test --test devtools_inspect_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test devtools_truth_execution_receipts_contract -- --nocapture`
- Pass evidence: Receipts have consistent target identity and explicit capability/missingFields; DevTools contracts pass.
- Fail evidence: Primitive returns invalid JSON, target mismatch, false ok despite missing data, or tests fail.

### DevTools act, events, coverage, and sessions form replayable proofs

- Stable anchor: `devtools-act-events-coverage-and-sessions-form-replayable-proofs`
- Choice id: `devtools-act-events-coverage-session-lifecycle`
- Surface: Script Kit DevTools / Sessions
- Status: Active QA lock-in candidate
- User contract: events record captures child command output, act receipts include lifecycle and safety gates, coverage reports known surfaces/missing primitives, and session cleanup is reliable.
- Regression risk: Without stable recording and coverage, future QA choices cannot be audited or replayed.
- Proof commands:
  - `bun scripts/devtools/events.ts record --session core-qa --start --show --output /tmp/core-qa-events.json -- bun scripts/devtools/act.ts set-input --text core-qa --main --strict --surface ScriptList`
  - `bun scripts/devtools/coverage.ts --surface main`
  - `./scripts/agentic/agent-cargo.sh test --test devtools_act_lifecycle_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test devtools_coverage_contract -- --nocapture`
- Pass evidence: Events receipt embeds child receipt, coverage reports surface status, and DevTools lifecycle/session contracts pass.
- Fail evidence: Recorder drops child output, safety gates absent, coverage stale, session cleanup fails, or contracts fail.

### MCP resources and tools expose Script Kit state consistently

- Stable anchor: `mcp-resources-and-tools-expose-script-kit-state-consistently`
- Choice id: `mcp-resources-and-tools-contracts`
- Surface: MCP Resources / Tools
- Status: Active QA lock-in candidate
- User contract: MCP resource URIs and tools return stable schemas, observation-only computer resources stay non-mutating, and SDK reference aligns with runtime docs.
- Regression risk: MCP drift breaks agent integrations and creates mismatches between UI and programmatic surfaces.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --test mcp_scripts_tools -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test mcp_notes_tools -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test mcp_clipboard_tools -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test mcp_config_tools -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test mcp_resource_drift -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test mcp_resources_sdk_reference -- --nocapture`
- Pass evidence: MCP tests pass with stable schemas and no mutation in observation-only resources.
- Fail evidence: Schema drift, missing resource, unsafe mutation, SDK mismatch, or test failure.

### Protocol versioning, deprecations, and parse recovery are stable

- Stable anchor: `protocol-versioning-deprecations-and-parse-recovery-are-stable`
- Choice id: `protocol-version-deprecation-and-error-recovery`
- Surface: Stdin Protocol / Compatibility
- Status: Active QA lock-in candidate
- User contract: Known messages parse, unknown/invalid lines are skipped gracefully, unsupported protocolVersion increments stats, and deprecated fields warn or error by configured version.
- Regression risk: Protocol compatibility work can accidentally break older SDKs or crash on future/invalid messages.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --lib protocol -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test stdin_parse_error_recovery_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test stdin_protocol_version_dispatch_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test protocol_stats_report_contract -- --nocapture`
- Pass evidence: Protocol and stdin tests pass for parse recovery, version dispatch, deprecation, and stats reporting.
- Fail evidence: Unknown/future message crashes, unsupported version not counted, deprecation policy ignored, or tests fail.

### Automation screenshots carry strict identity and nonblank proof

- Stable anchor: `automation-screenshots-carry-strict-identity-and-nonblank-proof`
- Choice id: `automation-screenshots-screenshot-identity`
- Surface: Automation / Screenshots
- Status: Active QA lock-in candidate
- User contract: captureScreenshot/inspectAutomationWindow returns screenshot identity, target bounds, optional osWindowId, nonblank content, and semantic elements that match the target.
- Regression risk: Visual proofs can become misleading if screenshots target the wrong window or lose identity metadata.
- Proof commands:
  - `bun scripts/devtools/inspect.ts --session core-qa --start --show --main --strict --hi-dpi --surface ScriptList --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test automation_screenshots -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test state_result_screenshot_identity_contract -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test verify_shot_strict_window_contract -- --nocapture`
- Pass evidence: Inspect receipt has screenshot dimensions/identity and screenshot tests pass for strict target/nonblank proof.
- Fail evidence: Blank screenshot, wrong window, missing identity/bounds, semantic mismatch, or tests fail.

### Startup stays event-driven and avoids blocking regressions

- Stable anchor: `startup-stays-event-driven-and-avoids-blocking-regressions`
- Choice id: `startup-performance-event-driven-no-blocking-regression`
- Surface: Startup / Performance
- Status: Active QA lock-in candidate
- User contract: Startup contracts confirm event-driven initialization, new-action startup state, and no known blocking regressions in shared startup paths.
- Regression risk: Performance regressions may not break functional tests but can make the core launcher feel dead.
- Proof commands:
  - `./scripts/agentic/agent-cargo.sh test --test startup_perf_event_driven -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test main_window_preflight -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test app_state_domain_structs_contract -- --nocapture`
- Pass evidence: Startup/performance source contracts pass and report no blocking startup path regression.
- Fail evidence: Startup state blocks, preflight stale, app state domain drift, or tests fail.

### Plugin skills are discoverable and launchable

- Stable anchor: `plugin-skills-are-discoverable-and-launchable`
- Choice id: `plugin-skill-inventory-launch-and-search`
- Surface: Plugin Skills
- Status: Active QA lock-in candidate
- User contract: Plugin skill inventory exposes expected rows/source metadata, search finds skills, and launch dispatch uses the plugin runtime owner.
- Regression risk: Skills extend core workflows; inventory or launch drift makes installed skills disappear or run under the wrong owner.
- Proof commands:
  - `bun scripts/devtools/act.ts set-input --text 'type:skill' --session core-qa --main --strict --surface ScriptList`
  - `bun scripts/devtools/elements.ts snapshot --session core-qa --main --strict --surface ScriptList --limit 200`
  - `./scripts/agentic/agent-cargo.sh test --test plugin_inventory -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test plugin_skill_search -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test plugin_skill_launch -- --nocapture`
  - `./scripts/agentic/agent-cargo.sh test --test plugin_skill_main_menu -- --nocapture`
- Pass evidence: Runtime filter exposes skill rows when available and plugin inventory/search/launch/main-menu contracts pass.
- Fail evidence: Skills missing from inventory, wrong runtime owner, launch route stale, or tests fail.

## Submission Feedback Applied

- `spine-sigils-project-main-input-catalogs`: `>` no longer selects cwd; Tab is the cwd trigger.
- One selected candidate was removed from active scope per product-scope feedback.
- `mini-mode-toggle-size-filter-reveal`: documented as experimental/WIP only.
- Overall: user-facing feature language says `Agent Chat`, not the engine name.
