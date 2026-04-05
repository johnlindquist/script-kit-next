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

# Legacy fd 3 cleanup (single-shell only)
# exec 3>&-
# rm -f "$PIPE"
# kill $APP_PID 2>/dev/null || true
# wait $APP_PID 2>/dev/null || true
```

### 8. Report
- **PASS**: build succeeded + expected screenshots match + expected log output
- **FAIL**: describe what went wrong with evidence (screenshot, log line)

## Timing Guidelines

| Action | Sleep after |
|--------|------------|
| App startup | 3s |
| `show` window | 1.5s |
| `setFilter` | 1s |
| `triggerBuiltin` (opens new view) | 3-5s |
| `simulateKey` (view transition) | 1.5s |
| `simulateKey` (text input) | 0.1s |
| `captureWindow` | 1s |
| ACP context bootstrap | 5-8s |
| ACP response streaming | 10-20s |

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
| `--acp-context-ready` | Context bootstrap complete |

**Exit codes:** 0 = pass, 1 = assertion failure, 2 = infrastructure error.

**Rule:** The recipe must fail when ACP state contradicts expected picker/caret
outcome, even if the screenshot capture itself succeeds. State receipt is the
primary proof; screenshot is secondary visual confirmation.

## Recipe Orchestrator (index.ts)

For common multi-step flows, use the thin wrapper:

```bash
# Check all prerequisites
bun scripts/agentic/index.ts preflight --session default

# Open ACP and verify ready state
bun scripts/agentic/index.ts acp-open --session default

# Full ACP picker accept golden path via Enter
bun scripts/agentic/index.ts acp-enter-accept --session default

# Full ACP picker accept golden path via Tab
bun scripts/agentic/index.ts acp-tab-accept --session default
```

The wrapper returns per-step JSON receipts from the underlying tools. It does
**not** replace the lower-level commands — use them directly when you need
finer control or different assertion combinations.

## ACP Golden Path (Critical)

The **mandatory** verification flow for any ACP interaction testing:

```
1. session start       → session alive
2. show                → window visible
3. triggerBuiltin tab-ai → ACP opens
4. wait / sleep        → context bootstraps
5. focus window        → frontmost confirmed
6. native type (macos-input.ts) → open picker
7. native Enter or Tab → accept picker item
8. getAcpState         → state receipt (MUST come before screenshot)
9. captureWindow       → screenshot captured
10. verify-shot.ts     → assertions pass on state + visual
11. grep telemetry     → acp_picker_item_accepted / acp_picker_tab_accept in logs
```

**State receipt before screenshot is non-negotiable.** If the state says the
picker is still open but the screenshot looks fine, the test must FAIL.

## Verification Recipes

See [references/recipes.md](references/recipes.md) for named verification patterns.

## Key Gotchas

- `simulateKey` does NOT go through GPUI's `intercept_keystrokes()`. Use `triggerBuiltin` for Tab AI, not `simulateKey` Tab.
- `AcpChatView` accepts single-char `simulateKey` for typing, `enter` for submit, `w`+cmd for close.
- The app window auto-hides when focus is lost. If captures fail with "Window not found", the window was dismissed.
- `captureWindow` filters out windows under 100x100 (tray icons).
- Always unset API keys if you need the setup card: `unset ANTHROPIC_API_KEY`.
- For ACP picker testing, use **native macOS input** (`macos-input.ts`) instead of `simulateKey` — synthetic keys bypass GPUI's native key interception and do not faithfully exercise picker selection behavior.
- Use `getAcpState` to verify picker acceptance, cursor landing, and input content — do not rely solely on screenshots for ACP state verification.
