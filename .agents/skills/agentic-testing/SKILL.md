---
name: agentic-testing
description: >-
  State-first runtime proof for UI, protocol, behavior, ACP, actions dialogs, surface changes, screenshots only when needed, and cleanup of launched sessions.
---

# Agentic Testing

This canonical repo-local skill owns runtime proof and session cleanup for Script Kit GPUI. It combines the current `.agents/skills` routing policy with the full operational recipe previously kept under `.codex/skills` / `.claude/skills`.

Verify code changes with the fastest proof that can establish correctness. Reuse warm sessions, prefer state receipts and exact targets, and escalate to screenshots or OS-level focus only when lower-cost proof cannot answer the question.

## Canonical Ownership

Use this `.agents/skills/agentic-testing` file as the source of truth for Codex routing and runtime proof. Legacy `.codex/skills/agentic-testing` and `.claude/skills/agentic-testing` files are compatibility snapshots only; mine them only when auditing history.

Primary paths and concepts:

- `scripts/agentic/`, `.test-output/`, `.test-screenshots/`
- Runtime proof, exact automation targets, screenshots, native input escalation, and cleanup
- State-first receipts from `getState`, `getElements`, `inspectAutomationWindow`, `waitFor`, `batch`, ACP state/probe APIs, and recipe receipts

## First Reads

Start with these sources before editing or proving behavior:

- `lat.md/automation.md`
- `lat.md/verification.md`
- `.agents/subagents/agentic-testing-reader.md` for broad or high-risk investigation.

## Workflow

1. Run or review the required `lat search` / `lat expand` context from `AGENTS.md`.
2. Identify the behavior owner before editing shared files. Path ownership is a hint; the user-visible behavior and documented contract decide the owner.
3. Check adjacent-skill boundaries before changing shared code.
4. Make the narrowest change that preserves the domain invariant.
5. Verify with the smallest proof that can fail if the behavior regresses.
6. Report changed files, proof tier, exact commands or receipts, adjacent skills consulted, cleanup status, and remaining risk.

Do not use this skill as the primary owner for test authoring or product ownership decisions; load `$testing-quality-gates`, `$protocol-automation`, `$lat-md`, or the relevant domain skill when those surfaces own the behavior.

## When to Use

- After implementing any UI, protocol, or behavior change
- For routine UI and behavior work, this is the default smoke test
- When Oracle's autonomous verification says "Run the agentic-testing skill"
- Before marking a task as complete
- Especially after changes to: prompts, views, keyboard handlers, ACP chat, actions dialog

## Seconds-First Default

Most verification runs should complete in seconds, not minutes. Default to the smallest proof tier that answers the question without stealing focus or blocking the user.

1. No-runtime proof: docs, skills, source audits, or focused tests only. Do not launch the app if runtime evidence is unnecessary.
2. State-first runtime proof: reuse a warm session and prove behavior with `getElements`, `getState`, `waitFor`, `batch`, and exact automation targets. This is the default for routing, selection, focus, popup ownership, and protocol bugs.
3. Visual proof: capture one screenshot only when layout, styling, visibility, animation, or real-shell composition is part of the acceptance criteria.
4. Native input and focus enforcement: use only when protocol-level and GPUI-level paths cannot exercise the real bug.

Rules:
- If your plan starts with cold start -> show -> screenshot -> log scrape, stop and look for a state-first proof.
- Reuse an existing healthy session before starting a new one.
- Avoid stealing OS focus unless the bug specifically involves native focus or the proof requires real keyboard or mouse delivery.
- Prefer exact target threading and targeted receipts over reading generic global state.
- If a non-visual proof is taking longer than about 10 seconds, redesign the proof before continuing.

## Safety Rules (MANDATORY)

- NEVER delete files, directories, or data
- NEVER modify databases, user data, or production state
- NEVER run destructive commands (rm -rf, DROP, git push --force, git reset --hard)
- NEVER send requests to external services, APIs, or webhooks
- NEVER modify files outside the project directory
- NEVER commit, push, or modify git history
- ALL verification is read-only: build, launch, screenshot, grep, read logs
- Temp pipes/logs may live under `/tmp` via the session wrapper. Runtime `captureWindow` screenshots must go in project `.test-screenshots/` / `test-screenshots/` or `~/.scriptkit/screenshots`
- The app runs locally only — never connect to production
- Every verification run MUST stop every Script Kit process/session it started before reporting results

## Surface Identity Rules (MANDATORY)

- Always verify the real user-facing surface through its real runtime entry path first.
- For Script Kit UI, prefer stdin JSON commands, built-in routing, and real app windows over ad hoc component harnesses.
- Never treat an isolated GPUI entity, temporary debug window, story, off-screen render, or synthetic wrapper as proof of a real product surface unless the user explicitly asks for component-level verification.
- Before trusting a screenshot, confirm the captured surface matches the intended product surface:
  - same entry path
  - same window/shell
  - same wrapper/root chrome
  - same footer, sizing, and layout structure
- If the screenshot does not clearly match the real surface, stop and re-route verification to the real surface instead of iterating on the fake one.
- For ACP specifically, `AcpChatView` in isolation is not sufficient proof. Default to the real ACP entry path (`triggerBuiltin tab-ai`, detached chat window routing, or another production runtime path) before using any synthetic ACP harness.

## Visual Diagnostics

Visual proof must connect what the user sees to structured layout and semantic receipts. Use visible text, layout measurement, and screenshot-to-semantics diagnostics before trusting a screenshot as UX proof.

- Use `screenshot-semantics-visual-consistency-stress` for pass-now visual consistency. It checks strict capture identity, non-blank content audit, `getState`, `getElements`, selected row, focus receipt, footer actions, popup crop bounds, and semantic visible text labels.
- `visibleTextMode:"semanticElements"` means the harness found visible text from automation element labels. It is not OCR and not clipping proof.
- For visible text, require text bounds, rendered text bounds, measured width, available width, glyph/container bounds, overlap pairs, and truncation metadata. Clipped or ellipsized text is acceptable only when the receipt proves intentional truncation plus tooltip or accessible full text.
- For layout measurement, require rem/font/scale metrics, window/content/container/scroll/input/footer bounds, footer/input ownership, and before/after layout-shift fingerprints for filtering or resizing.
- For screenshot-to-semantics checks, require the screenshot crop target to match the exact automation window and semantic surface, then cross-check selected row, focus ring, footer actions, and visible text against `getElements` receipts.
- Do not treat pixels alone as proof. A PNG can show that something rendered, but it does not prove the selected row, focus target, visible text, or footer actions are the correct semantic objects unless the receipt ties pixels back to the same target window.
- Do not claim text fits from a screenshot alone. Use `visible-text-clipping-overlap-stress` for clipping and overlap audits; it must fail closed until app-side text measurement receipts expose text bounds, measured width, available width, clipping state, truncation intent, tooltip or accessible full text, and overlap pairs.
- Do not claim rem/layout correctness from window bounds alone. Use `layout-measurement-regression-stress`; it must fail closed until app-side layout receipts expose rem size, scale factor, content/container/scroll metrics, footer/input ownership, and layout-shift samples.

## Hard Interaction Boundaries

When a user flow spans stacked modals, cross-surface export, or app restart recovery, require one receipt that proves ownership boundaries before sending input.

- For a modal stack, prove the topmost owner before each Escape, Cmd-W, or Enter action, then prove the child closed or executed without mutating the parent selection/focus unless that parent was the target.
- For cross-surface export provenance, prove the payload origin surface, generation, selected semantic id, redacted preview, destination identity, stale-source rejection, and cleanup. Clipboard or drag side effects alone are not proof.
- Restart/recovery recipes must gate every promoted target with a session epoch. If the epoch changes, the harness must refuse native, batch, and GPUI input before delivery, then re-resolve the exact target.
- For stale-target recovery, prove stale window targets are rejected, exact targets are re-resolved after restart or id churn, no stale input is delivered, and session cleanup ran. Never retry by kind without an identity receipt.
- For menu syntax ambiguity, prove tolerant diagnostics, skipped malformed fragments, selected command identity, and no accidental execution before submitting any command.
- For IME composition, prove composition start/update/commit boundaries, no premature submit/actions, and final committed text semantics. Plain key events are not enough.
- For selected-text fallback, prove permission denial/staleness, redaction, fallback source, and safe action disablement. Never trust stale frontmost-app context or raw selected text logs.
- For display migration visual bounds, prove source/target display identity, scale/rem metrics, focus/selection preservation, visible text bounds, screenshot-to-semantics alignment, wrong-display capture rejection, stale migration rejection, and no popup/main clobbering.
- For native picker or external app return, prove origin surface identity, handoff request id, picker/external window identity, restored focus/selection/cursor, stale or foreign window event rejection, and no submit or selection mutation during handoff.
- For drag cancellation, prove drag session identity, scoped payload fingerprint, redacted preview, hover/drop target cleanup, origin focus/selection restoration, no clipboard/file/attachment/prompt side effects, stale drag rejection, and foreign drop rejection.
- For runtime appearance churn while focused input is active, prove surface/window identity, focus, text, visible text, cursor/selection, rem/font/scale/layout metrics, theme and renderer token generations, stale token repaint rejection, and wrong-surface mutation rejection.
- For power resume recovery, prove pre-sleep target identity, post-wake target generation, stale target refusal before native/batch/GPUI/screenshot delivery, exact re-resolution, fresh state/elements/screenshot receipts, focus/selection preservation, and cleanup.
- For menu/tray/notification interruption, prove active modal/prompt identity, interruption identity, wrong-surface action rejection, topmost modal preservation, no focus steal, no selection/input/cursor mutation, no prompt submit, and focus restoration.
- For streaming progress cancellation, prove stream run identity, monotonic progress samples, visible progress text, cancellation request/ack ordering, stale post-cancel chunk rejection, no stale repaint, screenshot-to-state revalidation, focus/cursor restoration, no accidental submit, and cleanup.
- For dictation/media permission readiness churn, prove passive microphone/model setup, permission and model readiness generations, churn event ordering, target identity, transcript generation identity, wrong-target delivery rejection, no auto-submit, no System Settings or TCC mutation, focus/cursor preservation, and cleanup.
- For animation frame capture determinism, prove app animation frame ids, capture sequence ids, state/elements/screenshot receipts per sampled frame, visible text/layout fingerprints, motion occlusion pairs, stable frame ordering, stale-frame rejection, wrong-window rejection, blank-frame rejection, and cleanup.
- For accessibility tree semantic parity, prove visible controls, automation elements, and AX nodes share roles, labels, focus order, tab order, disabled states, safe keyboard activation semantics, hit targets, screenshot-to-semantics alignment, stale-tree rejection, wrong-window rejection, and cleanup.
- For RTL/bidirectional/emoji text rendering, prove direction runs, bidi levels, grapheme clusters, emoji ZWJ and combining mark sequences, cursor visual positions, selection rectangles, visible/rendered text bounds, truncation intent, search/filter semantics, stale layout rejection, wrong-surface mutation rejection, no accidental submit, and cleanup.
- For high-volume virtualized list stability, prove fixture identity, row identity, virtualization generations, selected-row reanchor, selection reanchor, scroll anchor preservation, rapid filter ordering, stale result rejection, duplicate-key rejection, blank-row rejection, footer-safe selection, screenshot-to-semantics consistency, and cleanup.
- For input-device modality transitions, prove hover/focus/selection affordances, pointer hover, keyboard focus, selection, trackpad/wheel scroll anchors, shortcut ownership, activation ownership, stale modality event rejection, wrong-surface input rejection, no accidental submit, screenshot-to-state revalidation, and cleanup.
- For multi-context attachment dedupe/provenance, prove file, screenshot, selected-text, MCP resource, script resource, and clipboard snippet origins across ACP Composer and Notes, with attachment provenance, destination generations, dedupe keys, provenance fingerprints, redacted preview, remove/reorder receipts, stale provenance rejection, duplicate-id rejection, no privacy leaks, and cleanup.
- For visual contrast readable state checks, prove active inactive disabled focused error loading states across themes, scale factors, and surfaces with theme token fingerprints, rem/scale metrics, text/color/bounds receipts, contrast ratios, non-color state cue coverage, screenshot-to-state revalidation, stale theme token rejection, wrong-surface rejection, and cleanup.
- For empty/error/retry state UX, prove empty, loading, error, retry, and recovered states with visible text, semantic retry identity, footer-safe actions, stable selection, and no stale error after recovery.
- For form validation and inline error recovery, prove invalid submit prevention, focus first invalid field, preserve user input, inline error identity, clear errors on valid edits, final submit recovery, prevent accidental submit, and no cross-field error leakage.
- For navigation/back-stack history, prove transition generations, route stack depth, actions discoverability, disabled/no-op affordances, Escape/back/Cmd-K close behavior, and return-to-origin restore selection, filter, scroll, footer, and focus without stale surface state.
- For long text wrapping/resizing UX stress, prove fixture identity, width mode, resize generation, full text, visible text, text/rendered/element bounds, available width, measured width, wrap line count, truncation intent, tooltip or accessible full text, overlap pairs, footer/input collision, focus and selection preservation, stale resize rejection, wrong-surface rejection, and cleanup.
- For actions/command discoverability no-op UX, prove actionable, disabled, and no-op row identities with labels, sections, disabled reasons, no-op reasons, keyboard selectability, skipped-row explanations, activation-prevention receipts, no host mutation, no accidental execution, focus restoration, stale action rejection, and cleanup.
- For dense list/detail preview readability, prove selected row identity, preview source identity, preview title/body bounds, metadata chip readability, footer action readability, filter generations, selection generations, resize generations, stale-preview rejection, row reanchor, focus preservation, no column/footer overlap, and cleanup.
- For transient toast/notification feedback, prove queue generation, bridge generation, visible text, duplicate collapse, autohide/dismiss ordering, bounds/overlap, footer/input non-blocking, stale rejection, and no action execution from toast UI.
- For destructive confirmation, prove dry-run-only fixture identity, confirm prompt identity, focused button, Enter/Escape resolution, no mutation before confirm, no mutation after cancel, no real system command request, stale/wrong-surface rejection, and parent focus/selection/filter restoration.
- For loading skeleton/progress restoration, prove request/result generations, skeleton rows, progress text/percent monotonicity, activation blocking while loading, stale loading/progress/result rejection, skeleton cleanup after results, and selection/focus/filter/scroll restoration.
- For icon/image fallback redaction, prove requested image source kind, redacted source fingerprint, fallback icon kind, fallback reason, image load generation, no raw path/URL/content leakage, stale image rejection, accessible label preservation, and cleanup.
- For footer/status persistence, prove owner, native footer surface id, rendered buttons, shortcut labels, status generation, persistence across filter/selection/actions transitions, duplicate-footer rejection, stale-status rejection, wrong-surface rejection, and cleanup.
- For keyboard hint label parity, prove footer, row accessory, tooltip, action catalog, normalized shortcut tokens, platform glyphs, disabled-state parity, activation owner, no accidental execution, stale-hint rejection, wrong-surface rejection, and cleanup.
- For row state parity without native pointer input, prove selected, focused, hovered, and selected-hovered row states through semantic row ids, state/elements receipts, modality receipts, tokenized fill/focus/text/icon states, stale-row rejection, wrong-surface rejection, no accidental execution, and cleanup.
- For quiet chrome/card nesting regressions, prove shell/content/row/popup/footer chrome layers, border/fill/shadow tokens, card depth, inset/gap/radius, duplicate-border rejection, opaque-fill rejection, stale-token rejection, wrong-surface rejection, and cleanup.
- For scroll shadows, sticky headers, and density drift, prove scroll position, viewport/content bounds, sticky header bounds/z-index, scroll shadow opacity tokens, row/header/input/footer heights, rem/scale metrics, footer-safe viewport, selected-row visibility, stale-scroll rejection, wrong-surface rejection, and cleanup.
- For popup focus/keycap visual semantics, prove popup owner identity, focused button/keycap parity, normalized shortcut glyphs, danger semantics on labels rather than keycaps, parent focus/selection preservation, stale focus rejection, wrong-surface rejection, no accidental execution, AFK-safe flags, and cleanup.
- For reduced-motion animation disable behavior, prove fixture-only reduced-motion policy, animation/transition generations, stable opacity/transform/frame receipts, disabled shimmer/spinner/pulse motion, focus/selection/cursor preservation, stale motion rejection, wrong-surface rejection, no System Settings or TCC mutation, AFK-safe flags, and cleanup.
- For command search highlighting/accessory badges, prove query/search generations, highlighted ranges, command row identity, accessory badge order/kinds/tooltips, disabled/no-op/loading reasons, action-catalog parity, stale highlight/badge rejection, wrong-host rejection, no accidental execution, AFK-safe flags, and cleanup.

## The Pattern

Every verification follows the same core loop:

### 1. Build Only What the Change Can Break
```bash
cargo build 2>&1 | tail -5
```
Only rebuild when the touched files can invalidate the binary or helper you need to exercise.

- Docs, skills, notes, or source-audit-only changes: skip build.
- Bun or shell harness changes with no Rust protocol changes: reuse the current debug binary if it already exists and a healthy session can start.
- Rust or runtime changes: run `cargo build`.

If you do build, it must complete with `Finished`. If it fails, fix the build error first.

### 2. Reuse or Start a Session
```bash
# First look for a healthy reusable session
bun scripts/agentic/session-state.ts --list
bash scripts/agentic/session.sh status default

# Start or resume a named session — works from any shell
# session.sh waits for the APP_READY log marker instead of sleeping
SESSION_JSON="$(bash scripts/agentic/session.sh start default 2>/dev/null)"
APP_PID="$(printf '%s' "$SESSION_JSON" | jq -r '.pid')"
PIPE="$(printf '%s' "$SESSION_JSON" | jq -r '.pipe')"
LOG="$(printf '%s' "$SESSION_JSON" | jq -r '.log')"
READY="$(printf '%s' "$SESSION_JSON" | jq -r '.ready // false')"
READY_WAIT_MS="$(printf '%s' "$SESSION_JSON" | jq -r '.readyWaitMs // 0')"

# Fallback only if readiness marker was not observed.
if [ "$READY" != "true" ]; then
  sleep 0.5
fi
```
The session wrapper manages the named pipe, forwarder process, and PID tracking.
Sessions are reusable across shells — no `exec 3>` / fd 3 trick required.
`session.sh start` means the app is stdin-ready, not necessarily capture-ready.
Prefer resume over cold start. A warm session plus state-only receipts should be the default path.

**Session commands:**
```bash
bash scripts/agentic/session.sh start [NAME]    # Create or resume (default: "default")
bash scripts/agentic/session.sh send NAME CMD    # Send JSON command
bash scripts/agentic/session.sh status [NAME]    # Check session state (JSON)
bash scripts/agentic/session.sh stop [NAME]      # Stop and clean up
bun scripts/agentic/session-state.ts --session NAME  # Detailed state report
bun scripts/agentic/session-state.ts --list          # List all sessions
```

All commands emit stable JSON envelopes on stdout (`schemaVersion`, `status`, payload).
Diagnostics go to stderr.

**`start` is idempotent** — re-running it resumes an existing healthy session.

**Alternative (legacy, single-shell only):**
```bash
PIPE=$(mktemp -u)
mkfifo "$PIPE"
export SCRIPT_KIT_AI_LOG=1
./target/debug/script-kit-gpui < "$PIPE" > /tmp/sk-test.log 2>&1 &
APP_PID=$!
exec 3>"$PIPE"
sleep 3
```

### 3. Show the Window Only When Needed
```bash
# Session-based (any shell)
bash scripts/agentic/session.sh send default '{"type":"show"}'
sleep 1.5
```
The app starts hidden. State-only proofs should usually skip this step entirely.

Show the window only for screenshots, native input, or other proofs that require the real visible surface.

### 4. Interact
Send commands via the session. Common commands:
```bash
S="bash scripts/agentic/session.sh send default"

# Set filter text
$S '{"type":"setFilter","text":"search term"}'

# Read current state without touching focus
bash scripts/agentic/session.sh rpc default '{"type":"getState","requestId":"s1"}' --expect stateResult

# Discover visible elements (returns semantic IDs)
bash scripts/agentic/session.sh rpc default '{"type":"getElements","requestId":"e1"}' --expect elementsResult

# Discover an attached popup or detached surface directly by target
bash scripts/agentic/session.sh rpc default '{"type":"getElements","requestId":"e2","target":{"type":"kind","kind":"actionsDialog","index":0}}' --expect elementsResult

# Select element by semantic ID (from getElements response)
bash scripts/agentic/session.sh rpc default '{"type":"batch","requestId":"b1","commands":[{"type":"selectBySemanticId","semanticId":"choice:0:apple","submit":true}]}' --expect batchResult

# When supported, mutate popup state directly instead of typing through native focus
bash scripts/agentic/session.sh rpc default '{"type":"batch","requestId":"b2","target":{"type":"kind","kind":"actionsDialog","index":0},"commands":[{"type":"setInput","text":"alias"}]}' --expect batchResult

# Trigger a built-in view
$S '{"type":"triggerBuiltin","name":"clipboard"}'
$S '{"type":"triggerBuiltin","name":"tab-ai"}'
$S '{"type":"triggerBuiltin","name":"emoji"}'
$S '{"type":"triggerBuiltin","name":"apps"}'
$S '{"type":"triggerBuiltin","name":"file-search"}'

# Simulate keys (dispatches to current view; not suitable for interceptor bugs)
$S '{"type":"simulateKey","key":"enter","modifiers":[]}'
$S '{"type":"simulateKey","key":"escape","modifiers":[]}'
$S '{"type":"simulateKey","key":"k","modifiers":["cmd"]}'
$S '{"type":"simulateKey","key":"w","modifiers":["cmd"]}'

# Prefer GPUI event dispatch over simulateKey when you need the real key pipeline
bash scripts/agentic/session.sh rpc default '{"type":"simulateGpuiEvent","requestId":"g1","target":{"type":"main"},"event":{"type":"keyDown","key":"down","modifiers":[]}}' --expect simulateGpuiEventResult

# Type individual characters (for views with text input)
$S '{"type":"simulateKey","key":"h","modifiers":[]}'

# Query ACP state (returns input, cursor, picker, accepted item, thread status)
bash scripts/agentic/session.sh rpc default '{"type":"getAcpState","requestId":"acp1"}' --expect acpStateResult
```

### 5. Capture Screenshots
```bash
mkdir -p .test-screenshots
bash scripts/agentic/session.sh send default '{"type":"captureWindow","title":"","path":"'"$(pwd)"'/.test-screenshots/step-01.png"}'
sleep 1
```
- `title` is substring match. `""` matches any window.
- For embedded ACP in the main Script Kit window, use `title: ""` or the resolver-driven `verify-shot.ts` / `window.ts` flow. Do not assume the title contains `ACP Chat`.
- Path must be absolute — use `$(pwd)/` prefix.
- Runtime `captureWindow` does not allow arbitrary `/tmp/*.png` output paths.
- Always `sleep 1` after capture for file write.
- The screenshot must come from the real runtime surface you are verifying, not a synthetic component window.
- **Read the PNG** to visually verify. Never assume correctness without checking.

### 6. Read Logs
```bash
grep -i "keyword" /tmp/sk-test.log | head -20
```
Log format: `TIMESTAMP|LEVEL|CATEGORY|cid=CORRELATION_ID message`

### 7. Cleanup
```bash
# Session-based (preferred)
bash scripts/agentic/session.sh stop default

# Verify the session is actually gone before reporting success
bash scripts/agentic/session.sh status default

# Legacy fd 3 cleanup (single-shell only)
# exec 3>&-
# rm -f "$PIPE"
# kill $APP_PID 2>/dev/null || true
# wait $APP_PID 2>/dev/null || true
```

Cleanup is mandatory, even after failures or interrupted runs.

- Do not report PASS or FAIL until the session you started has been stopped.
- If you launched Script Kit via `session.sh`, run `session.sh stop NAME` and verify the session is no longer alive.
- If you launched Script Kit directly, kill that specific PID and `wait` for it.
- Do not leave orphan `script-kit-gpui` processes behind from agentic testing.

### 8. Report
- **PASS**: build succeeded + expected screenshots match + expected log output + cleanup confirmed
- **FAIL**: describe what went wrong with evidence (screenshot, log line), then still clean up the launched process/session

## Timing Guidelines

| Action | Wait strategy |
|--------|--------------|
| App startup | `session.sh start` readiness wait; fallback 0.5s only if `ready=false` |
| Warm session reuse | Prefer 0-1s `status` / resume over creating a fresh process |
| State-only proof | Aim for 3-10s total; no screenshot or OS focus |
| `show` window | 0.3s macOS focus-settling delay |
| `setFilter` | 1s sleep or waitFor stateMatch |
| `triggerBuiltin` (opens new view) | waitFor appropriate condition |
| `simulateKey` (view transition) | 1.5s sleep |
| `simulateKey` (text input) | 0.1s sleep |
| `captureWindow` | 1s sleep (file write) |
| ACP context bootstrap | `waitFor(acpReady, timeout=8000)` |
| ACP picker open | `waitFor(acpPickerOpen, timeout=3000)` |
| ACP picker accept | `waitFor(acpItemAccepted, timeout=3000)` |
| ACP response streaming | 10-20s or waitFor(acpStatus) |

**Rule:** Use `waitFor` for all ACP state transitions. Only use fixed sleeps
for macOS focus-settling (0.3s) and file I/O (1s screenshot write).

**Rule:** Do not add a fixed `sleep 3` after `session.sh start`. The session
wrapper is responsible for readiness. Only use the 0.5s fallback when `ready=false`.

**Rule:** If a non-visual proof is trending beyond this budget, stop and redesign around `getElements`, `getState`, `waitFor`, `batch`, exact targets, or session reuse before escalating.

## Session Management

Use `scripts/agentic/session.sh` instead of hand-rolling `mkfifo` + `exec 3>` in ad hoc shells.

**Why:** The `exec 3>"$PIPE"` pattern ties the pipe to a single shell process. When a coding agent
spawns a new shell (e.g., follow-up verification step), fd 3 does not exist and the session is lost.
The session wrapper uses a background forwarder process so any shell can send commands via
`session.sh send`.

**Rules:**
- Always use `session.sh start` instead of manual `mkfifo` + `exec 3>` for new verification flows
- Use `session.sh send` for fire-and-forget stdin commands like `show`, `triggerBuiltin`, `setFilter`, and `captureWindow`
- Use `session.sh rpc` for protocol requests that expect a typed response like `getAcpState`, `getElements`, `waitFor`, `batch`, and `inspectAutomationWindow`
- Check session health with `session.sh status` or `session-state.ts` before sending commands
- Stop sessions with `session.sh stop` when done — do not leave orphan processes
- Treat cleanup as part of the test itself: a run is incomplete until the session is stopped and verified dead

## Field Notes

These are practical lessons from real ACP verification runs in this repo.

- If `session.sh start` reports a dead session even though the log reached `STARTUP_READY`, inspect the log before assuming the app crashed. In some debug runs the wrapper/forwarder dies while `script-kit-gpui` is still healthy. When that happens, switch to the legacy single-shell FIFO fallback so you can keep stdin open yourself.
- `window.ts` and `macos-input.ts --ensure-focus` may fail against the debug binary because the process name is `script-kit-gpui`, not the bundled `Script Kit` app identity. If focus/capture helpers cannot find the app, use `System Events` targeting `process "script-kit-gpui"` directly.
- For debug-only window capture, direct region screenshots via `screencapture -R<x,y,w,h>` can be more reliable than the bundle-oriented window resolver. Use runtime automation bounds or `System Events` window position/size to compute the region.
- ACP pasted-text verification needs two deletes when the cursor is immediately after a newly inserted token: the first backspace removes the trailing space, the second removes the token atomically. Query `getAcpState` after each step so you do not misread a correct first delete as a failure.

## Screenshot Assertion (verify-shot.ts)

Use `verify-shot.ts` for automated screenshot + state verification. It enforces
the correct ACP verification order: **state receipt first, screenshot second**.

```bash
# Basic: capture screenshot with ACP state assertions
bun scripts/agentic/verify-shot.ts --session default \
  --label step-name \
  --acp-status idle \
  --acp-picker-closed \
  --acp-context-ready

# Assert picker is open after typing @
bun scripts/agentic/verify-shot.ts --session default \
  --label picker-open \
  --acp-picker-open

# Assert item was accepted after Enter/Tab
bun scripts/agentic/verify-shot.ts --session default \
  --label item-accepted \
  --acp-picker-closed \
  --acp-item-accepted

# State-only (skip screenshot)
bun scripts/agentic/verify-shot.ts --session default \
  --label quick-check \
  --skip-screenshot \
  --acp-input-contains "@context"

# Screenshot-only (skip state query)
bun scripts/agentic/verify-shot.ts --session default \
  --label visual-check \
  --skip-state
```

**Available assertions:**
| Flag | Checks |
|------|--------|
| `--acp-status STATUS` | ACP status equals value (idle, streaming, etc.) |
| `--acp-picker-open` | Picker overlay is visible |
| `--acp-picker-closed` | Picker overlay is closed |
| `--acp-input-contains STR` | Input text contains substring |
| `--acp-input-match STR` | Input text matches exactly |
| `--acp-cursor-at N` | Cursor at character index N |
| `--acp-item-accepted` | A picker item was accepted (lastAcceptedItem non-null) |
| `--acp-accepted-label STR` | lastAcceptedItem.label equals STR |
| `--acp-accepted-trigger STR` | lastAcceptedItem.trigger equals STR (@ or /) |
| `--acp-accepted-via KEY` | Probe confirms acceptance via enter or tab |
| `--acp-cursor-after-accepted N` | Probe confirms cursor landed at index N after acceptance |
| `--acp-context-ready` | Context bootstrap complete |
| `--acp-no-selection` | No text selection active (hasSelection is false) |
| `--acp-has-selection` | Text selection is active (hasSelection is true) |
| `--acp-no-permission` | No pending permission (hasPendingPermission is false) |
| `--acp-has-permission` | Pending permission present (hasPendingPermission is true) |
| `--acp-visible-start N` | inputLayout.visibleStart equals N (first visible char index) |
| `--acp-visible-end N` | inputLayout.visibleEnd equals N (last visible char index) |
| `--acp-cursor-in-window N` | inputLayout.cursorInWindow equals N (cursor position in viewport) |

**Proof bundle fields:** The receipt includes stable top-level fields for machine consumption:
`state` (ACP snapshot), `probe` (test probe snapshot), `screenshot` (path + capture metadata),
`captureTarget` (requested vs actual window ID for identity proof),
`visionCrops` (structured image check entries). These are the canonical fields for automated parsing.

**Capture identity threading:** Detached ACP screenshots use the inspected
native `osWindowId`, not the automation window ID. When `--target-json` is
present, `verify-shot.ts` auto-lifts `inspection.osWindowId` into the screenshot
step. An explicit `--capture-window-id` is only an override and must match the
inspected `osWindowId`. The receipt exposes `captureTarget.requestedWindowId`,
`captureTarget.actualWindowId`, `captureRouting`, `requestedAutomationWindowId`,
and `inspectionOsWindowId`.

**Exit codes:** 0 = pass, 1 = assertion failure, 2 = infrastructure error.

### Canonical input-stability proof

Use visible-text-window assertions to verify single-line input rendering and cursor tracking
without a screenshot:

```bash
bun scripts/agentic/verify-shot.ts --session default \
  --label input-stability \
  --skip-screenshot \
  --acp-visible-start 12 \
  --acp-visible-end 52 \
  --acp-cursor-in-window 39
```

This proves the cursor is within the visible window and the viewport bounds are stable,
which catches scroll jumps, layout shifts, and cursor-out-of-view regressions.

**Strict capture:** When ACP assertions are present, `verify-shot.ts` requires
`window.ts` quartz capture with frontmost confirmation and the exact inspected
native window ID. If focus drifts, the inspected `osWindowId` is missing, or the
captured `windowId` differs from the requested ID, the run fails instead of
silently falling back to a full-screen screenshot.

**Rule:** The recipe must fail when ACP state contradicts expected picker/caret
outcome, even if the screenshot capture itself succeeds. State receipt is the
primary proof; screenshot is secondary visual confirmation.

## Recipe Orchestrator (index.ts) — Preferred ACP Verification

**Always prefer the canonical CLI over ad hoc shell sequences.** The orchestrator
encodes the correct verification order, focus enforcement, probe resets, and
checkpoint strategy so agents do not need to reconstruct these from scratch.

## Default Surface Proof (Preferred)

Use `surface-proof` as the default seconds-first proof command for an already-open product surface. For main-hosted surfaces, enter through the real runtime command and keep the proof state-first.

```bash
bash scripts/agentic/session.sh start default
bash scripts/agentic/session.sh send default '{"type":"triggerBuiltin","name":"clipboard"}' --await-parse
bun scripts/agentic/index.ts surface-proof --session default --kind main
bash scripts/agentic/session.sh stop default
bash scripts/agentic/session.sh status default

# Advanced exact-target proofs when a popup or detached surface already exists:
bun scripts/agentic/index.ts surface-proof --session default --kind promptPopup --index 0
bun scripts/agentic/index.ts surface-proof --session default --kind acpDetached --index 0
```

This path reuses a warm session, promotes the target through `automation-window.ts inspect`, samples `getState` and `getElements`, returns a machine-readable proof bundle, and does not call `show`, native input, or screenshot capture unless the proof explicitly needs that escalation.

Sample output shape:

```json
{
  "schemaVersion": 1,
  "recipe": "surface-proof",
  "status": "pass",
  "summary": "State-first main proof succeeded for main",
  "proofBundle": {
    "schemaVersion": 2,
    "scenario": "main-window-exact-id",
    "surfaceClass": "main",
    "resolvedTarget": {
      "windowId": "main",
      "windowKind": "Main"
    },
    "targetIdentity": { "stable": true },
    "usage": {
      "stateFirst": true,
      "usedGetState": true,
      "usedGetElements": true,
      "usedScreenshot": false,
      "usedNativeInput": false,
      "usedShow": false,
      "usedFixedSleepMs": 0
    },
    "capabilities": {
      "state": true,
      "elements": true,
      "nativeInputRequired": false,
      "screenshotRequired": false
    },
    "state": { "type": "stateResult" },
    "elements": { "type": "elementsResult" },
    "warnings": []
  }
}
```

### Canonical ACP proof commands

```bash
# Full ACP picker accept — choose key with --key enter|tab
bun scripts/agentic/index.ts acp-accept --session default --key enter
bun scripts/agentic/index.ts acp-accept --session default --key tab --vision

# Target a specific ACP window (detached/popup) — resolve exact identity first
RESOLVED="$(bun scripts/agentic/automation-window.ts resolve --session default --kind acpDetached --index 0)"
TARGET="$(printf '%s' "$RESOLVED" | jq -c '.targetJson')"
SURFACE_ID="$(printf '%s' "$RESOLVED" | jq -r '.surfaceId')"
bun scripts/agentic/index.ts acp-accept --session default --key enter \
  --target-json "$TARGET" --surface "$SURFACE_ID" --vision
```

### Target threading (non-negotiable for multi-window ACP)

When verifying a detached or popup ACP window, resolve **one target** once and
reuse it for every RPC and native input step in the entire run.

**Canonical rule:**
1. Discover the surface (e.g., `bun scripts/agentic/window.ts list`).
2. Pick one `--target-json` object (e.g., `{"type":"kind","kind":"acpDetached","index":0}`).
3. Pass that same target to every ACP RPC: `getAcpState`, `getAcpTestProbe`,
   `resetAcpTestProbe`, `waitFor`, and `batch`.
4. Pass the matching `--surface` value to native input so focus and proof stay
   on the same window.
5. **Never mix focused-window ACP RPCs with surface-targeted native input in the
   same verification run.** This causes cross-window false proof where you drive
   one ACP surface and verify another.

The `--target-json` flag threads through `index.ts` → `verify-shot.ts` → every
RPC command, and the `--surface` flag threads through `index.ts` → `macos-input.ts`
→ `window.ts` for focus enforcement.

When `--target-json` is omitted, RPCs default to the main ACP view (existing behavior).

What `acp-accept` guarantees:
- Resets ACP test probe before native interaction (no stale accepted items)
- Uses `macos-input.ts --ensure-focus` for native typing and acceptance
- Uses **state-only** checks for ACP-ready and picker-open (no intermediate screenshots)
- Waits for `acpAcceptedViaKey` (key-specific proof, not generic `acpItemAccepted`)
- Keeps exactly **one final screenshot** as visual proof
- Emits vision crops only when `--vision` is requested
- When `--vision` is used, surfaces the full proof bundle (with `state`, `probe`, `screenshot`, `visionCrops`) as `proofBundle` in the recipe receipt

### Other recipes

```bash
# Check all prerequisites
bun scripts/agentic/index.ts preflight --session default

# Open ACP and verify ready state (state-only, no screenshot)
bun scripts/agentic/index.ts acp-open --session default

# Compatibility aliases (same as --key enter / --key tab)
bun scripts/agentic/index.ts acp-enter-accept --session default
bun scripts/agentic/index.ts acp-tab-accept --session default

# Hard-scenario recipes
bun scripts/agentic/index.ts acp-detached-target-threading-stress \
  --session default --kind acpDetached --index 0 --min-targets 2 --key enter --vision --json
bun scripts/agentic/index.ts acp-prompt-popup-parity \
  --session default --families mention,model-selector,local-history --json
bun scripts/agentic/index.ts notes-acp-delayed-action-origin-stress \
  --session default --drift generation --json
bun scripts/agentic/index.ts file-portal-origin-roundtrip \
  --session default --origin acp --portal file-search --selection file --query AGENTS.md --json
bun scripts/agentic/index.ts permission-privacy-preflight \
  --session default --kinds accessibility,screen-recording,microphone --json
bun scripts/agentic/index.ts shortcut-recorder-focus-capture \
  --session default --surface shortcuts --action test-agentic-shortcut --chord cmd+shift+7 --sandbox-config --json
bun scripts/agentic/index.ts template-prompt-automation-parity-stress \
  --session default --template 'Hello {{name}}' --field name --value Ada --forced-value forced-template-result --json
bun scripts/agentic/index.ts current-app-commands-frontmost-stress \
  --session default --alias 'Do in Current Command' --query 'close tab' --json
bun scripts/agentic/index.ts actions-captured-subject-frame-stress \
  --session default --source root-file --action quick-look --mutation filter-selection-cache-frame --json
bun scripts/agentic/index.ts drop-prompt-native-drop-privacy-stress \
  --session default --file-name agentic-drop.txt --size 12 --json
bun scripts/agentic/index.ts path-prompt-filesystem-edge-stress \
  --session default --json
bun scripts/agentic/index.ts screenshot-identity-acp-context-stress \
  --session default --source tab-ai-screenshot --json
bun scripts/agentic/index.ts clipboard-history-portal-range-stress \
  --session default --portal-id 'kit://clipboard-history?id=agentic' --range composer:0..0 --json
bun scripts/agentic/index.ts browser-tabs-cache-identity-stress \
  --session default --source browser-tabs --json
bun scripts/agentic/index.ts scroll-selection-reanchor-stress \
  --session default --kinds clipboard,browser-history,current-app-commands,file-search --json
bun scripts/agentic/index.ts accessibility-tree-semantic-parity-stress \
  --session default --surfaces main,actionsDialog,promptPopup --json
bun scripts/agentic/index.ts rtl-bidi-emoji-text-rendering-stress \
  --session default --surface acp-composer --text 'abc שלום 👩🏽‍💻 é مرحبا 123' --json
bun scripts/agentic/index.ts high-volume-virtualized-list-stability-stress \
  --session default --surface clipboard-history --fixture-count 5000 --filter-cycles 8 --scroll-cycles 12 --json
bun scripts/agentic/index.ts input-modality-transition-ownership-stress \
  --session default --surface main --interleave pointer-hover,keyboard-nav,trackpad-scroll,wheel-scroll,shortcut --cycles 8 --json
bun scripts/agentic/index.ts multi-context-attachment-dedupe-provenance-stress \
  --session default --origins file,screenshot,selected-text,mcp-resource,clipboard-snippet --destinations acp-composer,notes --reorder-cycles 3 --json
bun scripts/agentic/index.ts visual-contrast-readable-state-stress \
  --session default --surfaces main,actionsDialog,promptPopup,acp-composer,notes --themes light,dark --scale-factors 1,1.25,1.5 --states active,inactive,disabled,focused,error,loading --json
bun scripts/agentic/index.ts empty-error-retry-state-ux-stress \
  --session default --surfaces main,clipboard-history,emoji-picker,file-search --query 'agentic-loop-eighteen-no-results-zzzz' --json
bun scripts/agentic/index.ts form-validation-inline-recovery-stress \
  --session default --surface fields-prompt --fields email,required-text,number --invalid email:not-an-email,required-text:,number:not-a-number --valid email:ada@example.com,required-text:Ada,number:42 --json
bun scripts/agentic/index.ts navigation-back-stack-history-stress \
  --session default --origin main --surfaces clipboard-history,emoji-picker,file-search,actionsDialog --transitions triggerBuiltin,cmd-k,escape,back --json
```

### State-only vs screenshot checkpoints

| Checkpoint | Screenshot? | Probe? | Why |
|------------|-------------|--------|-----|
| ACP ready | No | No | `waitFor(acpReady)` is sufficient proof; screenshot is waste |
| Picker open | No | No | `waitFor(acpPickerOpen)` is sufficient proof |
| **Final accepted** | **Yes** | **Yes** | The only checkpoint that needs visual + probe evidence |

**Rule:** Intermediate checkpoints use state-only verification (`--skip-screenshot --skip-probe`).
Only the final acceptance step captures a screenshot and queries the probe.

### Receipt shape

Each recipe returns a machine-readable JSON receipt:
```json
{
  "schemaVersion": 1,
  "recipe": "acp-enter-accept",
  "status": "pass",
  "steps": [
    { "name": "acp-open", "status": "pass" },
    { "name": "reset-probe", "status": "pass" },
    { "name": "type-at-trigger", "status": "pass" },
    { "name": "wait-accepted-via-key", "status": "pass" },
    { "name": "verify-accepted", "status": "pass" }
  ]
}
```

When `--vision` is used, a `proofBundle` field is added containing the verify-shot receipt
with `state`, `probe`, `screenshot`, and `visionCrops` for direct machine consumption.

The wrapper does **not** replace the lower-level commands — use `session.sh`,
`macos-input.ts`, `window.ts`, and `verify-shot.ts` directly when you need
finer control.

## ACP Golden Path (Critical)

The **mandatory** verification flow for any ACP interaction testing.
**Prefer the canonical CLI** (`bun scripts/agentic/index.ts acp-accept`) over
reconstructing the manual steps below.

### Canonical (one command, fully non-interactive)

```bash
bash scripts/agentic/session.sh start default
bun scripts/agentic/index.ts acp-accept --session default --key enter --vision
# The recipe returns a machine-readable JSON receipt with proofBundle.
# Parse proofBundle.state, proofBundle.probe, proofBundle.screenshot, proofBundle.visionCrops
# to verify ACP behavior programmatically, then read the written PNG for final visual confirmation.
bash scripts/agentic/session.sh stop default
```

### Exact detached ACP proof (preferred)

The `scenario` recipe resolves one exact detached ACP target once, reuses
the exact `targetJson` for every subsequent step, and emits a structured
proof bundle recording `windowId`, `surfaceId`, and ordered step receipts.

```bash
bash scripts/agentic/session.sh start default
bun scripts/agentic/index.ts scenario \
  --session default \
  --scenario detached-acp-exact-id \
  --index 0
bash scripts/agentic/session.sh stop default
```

The proof bundle is the regression substrate — every step records the exact
`target` used, the full request/response pair, and a timestamp. Exit code 0
means all steps succeeded; exit code 1 means some steps produced warnings.

### Canonical with target threading (detached/popup ACP)

For finer-grained control (e.g., picker acceptance flows), resolve one exact
ACP target once and reuse both the target and surfaceId for the full run.
**Do not use loose family-level `--surface acp`** — use the exact surfaceId
from the resolver.

```bash
bash scripts/agentic/session.sh start default

# Resolve exact target and surface identity once
RESOLVED="$(bun scripts/agentic/automation-window.ts resolve --session default --kind acpDetached --index 0)"
TARGET="$(printf '%s' "$RESOLVED" | jq -c '.targetJson')"
SURFACE_ID="$(printf '%s' "$RESOLVED" | jq -r '.surfaceId')"

bun scripts/agentic/index.ts acp-accept --session default --key enter \
  --target-json "$TARGET" --surface "$SURFACE_ID" --vision
INSPECTED="$(bun scripts/agentic/automation-window.ts inspect --session default --id "$(printf '%s' "$RESOLVED" | jq -r '.automationWindowId')")"
WINDOW_ID="$(printf '%s' "$INSPECTED" | jq -r '.osWindowId')"

bun scripts/agentic/index.ts acp-accept --session default --key enter \
  --target-json "$TARGET" --surface "$SURFACE_ID" --vision
bun scripts/agentic/verify-shot.ts --session default --label detached-proof \
  --target-json "$TARGET" --capture-window-id "$WINDOW_ID"
# Confirm proofBundle.state.resolvedTarget.windowKind == "acpDetached"
# Confirm captureTarget.requestedWindowId == captureTarget.actualWindowId
bash scripts/agentic/session.sh stop default
```

The `--vision` flag makes the recipe return a `proofBundle` containing all
machine-readable proof. The golden path is complete when the exit code is 0
and the `proofBundle.state` and `proofBundle.probe` fields confirm the expected
ACP state. Screenshot files are still written for archival but are not the
primary verification mechanism.

### Manual (when you need finer control)

```
1. session start                               → session alive
2. show                                        → window visible
3. triggerBuiltin tab-ai                       → ACP opens
4. waitFor(acpReady, timeout=8000)             → context bootstrapped (deterministic)
5. focus window                                → frontmost confirmed
6. native type @ (macos-input.ts --ensure-focus) → open picker
7. waitFor(acpPickerOpen, timeout=3000)        → picker open (deterministic)
8. native Enter or Tab (macos-input.ts --ensure-focus) → accept picker item
9. waitFor(acpAcceptedViaKey, timeout=3000)    → key-specific acceptance (deterministic)
10. verify-shot.ts with --acp-accepted-via     → state + probe + screenshot proof
```

**Key tools in the golden path:**
| Tool | Role |
|------|------|
| `session.sh` | Cross-shell session management, RPC, lifecycle |
| `macos-input.ts` | Native macOS keyboard/mouse with `--ensure-focus` |
| `window.ts` | Window discovery, focus, activation, quartz capture |
| `verify-shot.ts` | State + probe + screenshot bundle with strict capture |
| `automation-window.ts` | Exact ACP target-to-surface resolver |
| `scenario.ts` | Replayable proof-bundle scenarios for cross-window regression |
| `index.ts` | Orchestrator that composes all of the above correctly |

**waitFor replaces fixed sleeps.** Each `waitFor` polls at 25ms intervals
and returns a `waitForResult` receipt with `success`, `elapsed`, and an
optional `trace` (included automatically on failure when `trace: "onFailure"`).

**State receipt before screenshot is non-negotiable.** If the state says the
picker is still open but the screenshot looks fine, the test must FAIL.

**Any remaining sleeps** in the recipes are brief macOS focus-settling delays
(~300ms) with explicit comments. They are not proof of ACP state.

## Verification Recipes

See [references/recipes.md](references/recipes.md) for named verification patterns.

Other hard-scenario recipes:

```bash
bun scripts/agentic/index.ts long-text-wrap-resize-surface-stress --session default --surfaces main,clipboard-history,emoji-picker,file-search,actionsDialog --widths mini,narrow,full --fixtures long-name,long-path,long-description,multiline-snippet --json
bun scripts/agentic/index.ts actions-command-discoverability-noop-stress --session default --hosts main,clipboard-history,emoji-picker,file-search,app-launcher --states actionable,disabled,no-op --json
bun scripts/agentic/index.ts dense-list-detail-preview-readability-stress --session default --surfaces file-search,sdk-reference,script-template-catalog --query agentic-loop-nineteen-preview --filter-cycles 4 --selection-cycles 8 --resize-cycles 3 --json
bun scripts/agentic/index.ts toast-notification-queue-lifecycle-stress --session default --surface main --fixtures success,duplicate,persistent,dismiss,autohide --cycles 3 --json
bun scripts/agentic/index.ts destructive-confirm-modal-safety-stress --session default --host main --fixture agentic-destructive-dry-run --paths cancel,confirm,stale-confirm --dry-run-only --json
bun scripts/agentic/index.ts loading-skeleton-progress-restoration-stress --session default --surfaces sdk-reference,script-template-catalog --fixture delayed-local --cycles 4 --json
bun scripts/agentic/index.ts icon-image-fallback-redaction-stress --session default --surfaces app-launcher,file-search,clipboard-history --fixtures missing-file,corrupt-png,private-local-path,data-uri-redacted --json
bun scripts/agentic/index.ts footer-status-persistence-stress --session default --surfaces main,clipboard-history,emoji-picker,file-search,actionsDialog --transitions filter,selection,cmd-k,escape,clear-filter --json
bun scripts/agentic/index.ts keyboard-hint-label-parity-stress --session default --surfaces main,clipboard-history,emoji-picker,file-search,actionsDialog,menuSyntaxTriggerPopup --families footer,row-accessory,tooltip,action-catalog --json
bun scripts/agentic/index.ts row-state-parity-without-pointer-stress --session default --surfaces main,clipboard-history,emoji-picker,file-search,actionsDialog --states selected,focused,hovered,selected-hovered --json
bun scripts/agentic/index.ts quiet-chrome-card-nesting-stress --session default --surfaces main,clipboard-history,emoji-picker,file-search,actionsDialog,promptPopup --chrome quiet --json
bun scripts/agentic/index.ts scroll-shadow-sticky-header-density-stress --session default --surfaces clipboard-history,emoji-picker,file-search,app-launcher,actionsDialog --scroll-positions top,middle,bottom --density compact,default --json
bun scripts/agentic/index.ts popup-focus-keycap-visual-semantics-stress --session default --surfaces actionsDialog,menuSyntaxTriggerPopup,confirmPrompt --json
bun scripts/agentic/index.ts reduced-motion-animation-disable-stress --session default --surfaces main,actionsDialog,menuSyntaxTriggerPopup --fixture reduced-motion --json
bun scripts/agentic/index.ts command-search-highlighting-accessory-badges-stress --session default --hosts main,actionsDialog,app-launcher,menuSyntaxTriggerPopup --query agentic-loop-twenty-three --json
```

## Adjacent Skills

Use adjacent skills when the work crosses boundaries:

- `$testing-quality-gates` for choosing narrow build/test gates.
- `$lat-md` for `lat.md/` section, wiki-link, or code-ref changes.
- `$protocol-automation` when stdin JSON, receipts, target identity, `waitFor`, or `batch` are the behavior owner.
- The domain skill for the active surface, such as `$acp-chat-core`, `$actions-popups`, `$keyboard-focus-routing`, `$launcher-surface-contracts`, or `$window-resizing`.

## Migration Notes

This skill intentionally absorbs the long-form `.codex/skills/agentic-testing` and `.claude/skills/agentic-testing` recipes so future agents do not have to choose between duplicate `agentic-testing` definitions. Keep future updates here first, then update `lat.md/agent-skills.md` when the canonical routing or proof contract changes.

## Key Gotchas

- `simulateKey` does NOT go through GPUI's `intercept_keystrokes()`. Use `triggerBuiltin` for ACP Chat entry, not `simulateKey` Tab.
- `AcpChatView` accepts single-char `simulateKey` for typing, `enter` for submit, `w`+cmd for close.
- Attached popups like `ActionsDialog` and `PromptPopup` can expose targeted state snapshots even when they do not expose an independent GPUI key handle. Read state from the popup target first; only escalate to parent-window or native input when you must drive real key delivery.
- `simulateGpuiEvent` is better than `simulateKey` for interceptor bugs, but `handle_unavailable` means the target has no usable runtime handle for that path. Treat that as a proof-design problem, not a cue to spam retries.
- The app window auto-hides when focus is lost. If captures fail with "Window not found", the window was dismissed.
- `captureWindow` filters out windows under 100x100 (tray icons).
- Always unset API keys if you need the setup card: `unset ANTHROPIC_API_KEY`.
- For ACP picker testing, use **native macOS input** (`macos-input.ts --ensure-focus`) instead of `simulateKey` — synthetic keys bypass GPUI's native key interception and do not faithfully exercise picker selection behavior.
- Use `getAcpState` to verify picker acceptance, cursor landing, and input content — do not rely solely on screenshots for ACP state verification.
- Use `waitFor` commands via `session.sh rpc` for deterministic ACP state transitions — do not use fixed sleeps as proof of ACP state.
