---
name: agentic-testing
description: Autonomously verify code changes by building, launching Script Kit GPUI, sending stdin JSON commands, capturing screenshots, and reading logs. Run after implementing changes to confirm correctness.
---

# Agentic Testing

Verify code changes by observing the running app. Build, start via named pipe, interact via stdin JSON, capture screenshots, read logs.

## When to Use

- After implementing any UI, protocol, or behavior change
- When Oracle's autonomous verification says "Run the agentic-testing skill"
- Before marking a task as complete
- Especially after changes to: prompts, views, keyboard handlers, ACP chat, actions dialog

## Safety Rules (MANDATORY)

- NEVER delete files, directories, or data
- NEVER modify databases, user data, or production state
- NEVER run destructive commands (rm -rf, DROP, git push --force, git reset --hard)
- NEVER send requests to external services, APIs, or webhooks
- NEVER modify files outside the project directory
- NEVER commit, push, or modify git history
- ALL verification is read-only: build, launch, screenshot, grep, read logs
- Temp files (pipes, screenshots) go in project `test-screenshots/` or `/tmp`
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

## The Pattern

Every verification follows the same core loop:

### 1. Build
```bash
cargo build 2>&1 | tail -5
```
Must complete with `Finished`. If it fails, fix the build error first.

### 2. Start a Session (Preferred)
```bash
# Start or resume a named session — works from any shell
eval "$(bash scripts/agentic/session.sh start default 2>/dev/null | jq -r '@sh "APP_PID=\(.pid) PIPE=\(.pipe) LOG=\(.log)"')"
sleep 3
```
The session wrapper manages the named pipe, forwarder process, and PID tracking.
Sessions are reusable across shells — no `exec 3>` / fd 3 trick required.

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

### 3. Show the Window
```bash
# Session-based (any shell)
bash scripts/agentic/session.sh send default '{"type":"show"}'
sleep 1.5
```
The app starts hidden. Always send `show` first.

### 4. Interact
Send commands via the session. Common commands:
```bash
S="bash scripts/agentic/session.sh send default"

# Set filter text
$S '{"type":"setFilter","text":"search term"}'

# Discover visible elements (returns semantic IDs)
$S '{"type":"getElements","requestId":"e1"}'

# Select element by semantic ID (from getElements response)
$S '{"type":"batch","requestId":"b1","commands":[{"type":"selectBySemanticId","semanticId":"choice:0:apple","submit":true}]}'

# Trigger a built-in view
$S '{"type":"triggerBuiltin","name":"clipboard"}'
$S '{"type":"triggerBuiltin","name":"tab-ai"}'
$S '{"type":"triggerBuiltin","name":"emoji"}'
$S '{"type":"triggerBuiltin","name":"apps"}'
$S '{"type":"triggerBuiltin","name":"file-search"}'

# Simulate keys (dispatches to current view)
$S '{"type":"simulateKey","key":"enter","modifiers":[]}'
$S '{"type":"simulateKey","key":"escape","modifiers":[]}'
$S '{"type":"simulateKey","key":"k","modifiers":["cmd"]}'
$S '{"type":"simulateKey","key":"w","modifiers":["cmd"]}'

# Type individual characters (for views with text input)
$S '{"type":"simulateKey","key":"h","modifiers":[]}'

# Query ACP state (returns input, cursor, picker, accepted item, thread status)
$S '{"type":"getAcpState","requestId":"acp1"}'
```

### 5. Capture Screenshots
```bash
mkdir -p test-screenshots
bash scripts/agentic/session.sh send default '{"type":"captureWindow","title":"","path":"'"$(pwd)"'/test-screenshots/step-01.png"}'
sleep 1
```
- `title` is substring match. `""` matches any window.
- Path must be absolute — use `$(pwd)/` prefix.
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
| App startup | 3s sleep |
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

## Session Management

Use `scripts/agentic/session.sh` instead of hand-rolling `mkfifo` + `exec 3>` in ad hoc shells.

**Why:** The `exec 3>"$PIPE"` pattern ties the pipe to a single shell process. When a coding agent
spawns a new shell (e.g., follow-up verification step), fd 3 does not exist and the session is lost.
The session wrapper uses a background forwarder process so any shell can send commands via
`session.sh send`.

**Rules:**
- Always use `session.sh start` instead of manual `mkfifo` + `exec 3>` for new verification flows
- Use `session.sh send` for all protocol commands — do not assume fd 3 survives across steps
- Check session health with `session.sh status` or `session-state.ts` before sending commands
- Stop sessions with `session.sh stop` when done — do not leave orphan processes
- Treat cleanup as part of the test itself: a run is incomplete until the session is stopped and verified dead

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
| `--acp-item-accepted` | A picker item was accepted |
| `--acp-accepted-via KEY` | Probe confirms acceptance via enter or tab |
| `--acp-context-ready` | Context bootstrap complete |

**Exit codes:** 0 = pass, 1 = assertion failure, 2 = infrastructure error.

**Strict capture:** When ACP assertions are present, `verify-shot.ts` requires
`window.ts` quartz capture with frontmost confirmation. If focus drifts, the
capture fails instead of silently falling back to a full-screen screenshot.

**Rule:** The recipe must fail when ACP state contradicts expected picker/caret
outcome, even if the screenshot capture itself succeeds. State receipt is the
primary proof; screenshot is secondary visual confirmation.

## Recipe Orchestrator (index.ts) — Preferred ACP Verification

**Always prefer the canonical CLI over ad hoc shell sequences.** The orchestrator
encodes the correct verification order, focus enforcement, probe resets, and
checkpoint strategy so agents do not need to reconstruct these from scratch.

### Canonical ACP proof commands

```bash
# Full ACP picker accept — choose key with --key enter|tab
bun scripts/agentic/index.ts acp-accept --session default --key enter
bun scripts/agentic/index.ts acp-accept --session default --key tab --vision
```

What `acp-accept` guarantees:
- Resets ACP test probe before native interaction (no stale accepted items)
- Uses `macos-input.ts --ensure-focus` for native typing and acceptance
- Uses **state-only** checks for ACP-ready and picker-open (no intermediate screenshots)
- Waits for `acpAcceptedViaKey` (key-specific proof, not generic `acpItemAccepted`)
- Keeps exactly **one final screenshot** as visual proof
- Emits vision crops only when `--vision` is requested

### Other recipes

```bash
# Check all prerequisites
bun scripts/agentic/index.ts preflight --session default

# Open ACP and verify ready state (state-only, no screenshot)
bun scripts/agentic/index.ts acp-open --session default

# Compatibility aliases (same as --key enter / --key tab)
bun scripts/agentic/index.ts acp-enter-accept --session default
bun scripts/agentic/index.ts acp-tab-accept --session default
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

The wrapper does **not** replace the lower-level commands — use `session.sh`,
`macos-input.ts`, `window.ts`, and `verify-shot.ts` directly when you need
finer control.

## ACP Golden Path (Critical)

The **mandatory** verification flow for any ACP interaction testing.
**Prefer the canonical CLI** (`bun scripts/agentic/index.ts acp-accept`) over
reconstructing the manual steps below.

### Canonical (one command)

```bash
bash scripts/agentic/session.sh start default
sleep 3
bun scripts/agentic/index.ts acp-accept --session default --key enter
# Read the final screenshot PNG to confirm visually
bash scripts/agentic/session.sh stop default
```

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

## Key Gotchas

- `simulateKey` does NOT go through GPUI's `intercept_keystrokes()`. Use `triggerBuiltin` for Tab AI, not `simulateKey` Tab.
- `AcpChatView` accepts single-char `simulateKey` for typing, `enter` for submit, `w`+cmd for close.
- The app window auto-hides when focus is lost. If captures fail with "Window not found", the window was dismissed.
- `captureWindow` filters out windows under 100x100 (tray icons).
- Always unset API keys if you need the setup card: `unset ANTHROPIC_API_KEY`.
- For ACP picker testing, use **native macOS input** (`macos-input.ts --ensure-focus`) instead of `simulateKey` — synthetic keys bypass GPUI's native key interception and do not faithfully exercise picker selection behavior.
- Use `getAcpState` to verify picker acceptance, cursor landing, and input content — do not rely solely on screenshots for ACP state verification.
- Use `waitFor` commands via `session.sh rpc` for deterministic ACP state transitions — do not use fixed sleeps as proof of ACP state.
