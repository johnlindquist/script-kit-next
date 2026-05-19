# Mention token ACP DevTools verification — 2026-05-18

## Classification: **fixed** (live)

## Isolated session

- `SESSION=mention-verify-cli-1779164290`
- `SCRIPT_KIT_SESSION_DIR=/tmp/sk-agentic-sessions` (flat layout; do **not** nest `/${SESSION}` twice)
- Binary: `target/debug/script-kit-gpui` (promoted from `target-agent/dt-mention-build/`)

## DevTools proof

```bash
bun scripts/devtools/acp-mention.ts verify --session "$SESSION" --file CLAUDE.md
```

**Result:**

| Check | Value |
| --- | --- |
| `classification` | `fixed` |
| `inputText` | `@file:CLAUDE.md ` |
| Extension prefixes (`@md:`, `@ts:`) | None |
| Batch path | `target: { type: "id", id: "ai" }` → detached ACP entity |

## Batch flow (two-level picker)

1. `setInput` `@`
2. `waitFor` `acpPickerOpen`
3. `selectByValue` `@file` + submit
4. `waitFor` `acpPickerOpen`
5. `selectByValue` `CLAUDE.md` + submit
6. `waitFor` `acpItemAccepted`

## Unit tests (prior run)

- `context_mentions` lib tests: **82/82** pass
- `check --lib`: clean (after Ai routing fix)

## Receipt

Full JSON: `.agent-reports/mention-token-acp-verify-20260518.json`
