---
name: script-kit-agent-workflow
description: Autonomous fix-verify workflow for Script Kit GPUI. Use when fixing bugs, making changes, or completing tasks. Covers the fast smoke-test-first loop, logging modes, and session completion protocol.
---

# Script Kit Agent Workflow

Default workflow for code changes. Do NOT ask users to test. Do NOT skip verification.

## Quick-Start Checklist (Do First)

1. Read CLAUDE.md before changing code
2. Check `.hive/issues.jsonl` for tasks/context
3. TDD: write failing test → implement → refactor
4. Update bead status when starting/completing work
5. Include `correlation_id` in all log entries/spans
6. UI changes: test via stdin JSON protocol (never CLI args)
7. Before every commit, run only the checks the user asked for or the task actually needs

## The Fix-Verify Loop

```
1. EXPLORE: Understand the problem
   - Use Task tool with explore agent for codebase search
   - Read relevant files, identify root cause before coding

2. FIX: Make the code change
   - Keep changes minimal and focused

3. BUILD: Compile the smallest valid target
   cargo build

4. LAUNCH: Run the app with logging
   echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

5. CHECK LOGS: Verify the fix
   grep -i "keyword" ~/.scriptkit/logs/script-kit-gpui.jsonl

6. VISUAL VERIFY (if UI change):
   - Prefer `agentic-testing` against the real runtime surface
   - Use `make smoke-main-menu` for main window/footer work
   - Save PNG to ./test-screenshots/
   - READ the PNG file to actually verify

7. RUN TARGETED TESTS: only when they are relevant to the touched area
```

## Log Modes

| Mode | Command | Use Case |
|------|---------|----------|
| Compact AI logs | `SCRIPT_KIT_AI_LOG=1` | Default for AI agents |
| Full debug logs | `RUST_LOG=debug` | Deep debugging |
| Specific module | `RUST_LOG=script_kit::theme=debug` | Target one module |

## Anti-Patterns (Never Do These)

- "The user should test this manually"
- "I made the change, it should work"
- Making a fix without checking logs
- Capturing screenshot but not reading the PNG
- Skipping `cargo check` before launch
- Not using `SCRIPT_KIT_AI_LOG=1` or `RUST_LOG=debug`
- Leaving a `script-kit-gpui` verification process/session running after testing

## Success Indicators

- Explored codebase before fixing
- Made targeted fix based on understanding
- Build succeeds for the touched surface
- Launched app and checked relevant logs
- Logs confirm the change took effect
- Any targeted checks requested by the task pass
- (If visual) Screenshot captured AND read
- Any local verification session/process explicitly stopped before handoff

## Session Completion ("Landing the Plane")

Work is done when the requested change is implemented and verified.

1. File issues for remaining work
2. If code changed: run the smallest verification that proves the change
3. Update issue status
4. Commit only if the user asks
5. Push only if the user asks
6. Clean up local verification processes/sessions
7. Hand off context for next session

Rules:
- Never skip verification
- Never push unless the user asked for it
- Never leave verification processes running

## References

- [Scenario Testing](references/scenario-testing.md) - Automated scenario-based testing
- [Dev Workflow](references/dev-workflow.md) - Hot reload and debugging
