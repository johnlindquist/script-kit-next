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

### 2. Launch via Named Pipe
```bash
PIPE=$(mktemp -u)
mkfifo "$PIPE"
export SCRIPT_KIT_AI_LOG=1
./target/debug/script-kit-gpui < "$PIPE" > /tmp/sk-test.log 2>&1 &
APP_PID=$!
exec 3>"$PIPE"
sleep 3
```
The `exec 3>"$PIPE"` keeps the write end open. Without it, the pipe closes after one write and the app exits.

### 3. Show the Window
```bash
echo '{"type":"show"}' >&3
sleep 1.5
```
The app starts hidden. Always send `show` first.

### 4. Interact
Send commands via fd 3. Common commands:
```bash
# Set filter text
echo '{"type":"setFilter","text":"search term"}' >&3

# Discover visible elements (returns semantic IDs)
echo '{"type":"getElements","requestId":"e1"}' >&3

# Select element by semantic ID (from getElements response)
echo '{"type":"batch","requestId":"b1","commands":[{"type":"selectBySemanticId","semanticId":"choice:0:apple","submit":true}]}' >&3

# Trigger a built-in view
echo '{"type":"triggerBuiltin","name":"clipboard"}' >&3
echo '{"type":"triggerBuiltin","name":"tab-ai"}' >&3
echo '{"type":"triggerBuiltin","name":"emoji"}' >&3
echo '{"type":"triggerBuiltin","name":"apps"}' >&3
echo '{"type":"triggerBuiltin","name":"file-search"}' >&3

# Simulate keys (dispatches to current view)
echo '{"type":"simulateKey","key":"enter","modifiers":[]}' >&3
echo '{"type":"simulateKey","key":"escape","modifiers":[]}' >&3
echo '{"type":"simulateKey","key":"k","modifiers":["cmd"]}' >&3
echo '{"type":"simulateKey","key":"w","modifiers":["cmd"]}' >&3

# Type individual characters (for views with text input)
echo '{"type":"simulateKey","key":"h","modifiers":[]}' >&3
```

### 5. Capture Screenshots
```bash
mkdir -p test-screenshots
echo '{"type":"captureWindow","title":"","path":"'"$(pwd)"'/test-screenshots/step-01.png"}' >&3
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
exec 3>&-
rm -f "$PIPE"
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true
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

## Verification Recipes

See [references/recipes.md](references/recipes.md) for named verification patterns.

## Key Gotchas

- `simulateKey` does NOT go through GPUI's `intercept_keystrokes()`. Use `triggerBuiltin` for Tab AI, not `simulateKey` Tab.
- `AcpChatView` accepts single-char `simulateKey` for typing, `enter` for submit, `w`+cmd for close.
- The app window auto-hides when focus is lost. If captures fail with "Window not found", the window was dismissed.
- `captureWindow` filters out windows under 100x100 (tray icons).
- Always unset API keys if you need the setup card: `unset ANTHROPIC_API_KEY`.
