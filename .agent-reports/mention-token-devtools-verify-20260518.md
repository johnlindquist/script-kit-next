# Mention token canonical `@file:` — DevTools verification

**Date:** 2026-05-18  
**Session(s):** `mention-token-dt-20260518-210329` (partial live proof), `mention-token-dt-20260518-210453` (batch probe)

## Compile / unit gates

| Check | Result |
| --- | --- |
| `./scripts/agentic/agent-cargo.sh check --lib` | **PASS** |
| `./scripts/agentic/agent-cargo.sh test --lib context_mentions` | **PASS** (82/82) |
| Stale docstring fix in `src/ai/acp/view.rs` | Comment-only; no rebuild risk |

## DevTools investigation (`script-kit-devtools`)

### `investigate.ts --surface acp-chat-ai`

- Classification: investigation plan emitted; missing primitives noted (`devtools.acp.inspect`, `devtools.composer.inspect`).
- Owners: `acp-context-composer`, `acp-chat-core`.

### `inspect.ts` (AI target `ai`)

- **Target resolution:** `Script Kit AI` (`automationId: ai`, `targetKind: Ai`).
- **Classification:** `blocked-by-missing-primitive` — empty `getElements` / `getState` surface fields when AI window is not the protocol-inspect host (same gap as prior handoff).
- **Primitive stack:** `listAutomationWindows`, `inspectAutomationWindow`, `getState`, `getElements`, `getLayoutInfo` all returned envelopes; semantic tree empty.

### `getAcpState` (target `ai`) — **usable**

After `openAi`:

- Reads composer via embedded-AI → main routing (`automation.acp_target.embedded_ai_routed_to_main`).
- **`set-input "@"` on `--main`** opens picker:
  - `inputText: "@"`
  - `picker.open: true`, `itemCount: 24`, `selectedLabel: "@file"` (category row).

### `act.ts` gaps (documented blockers)

| Action | Target | Result |
| --- | --- | --- |
| `set-input` | `ai` | **FAIL** — batch rejects `Ai` target |
| `set-input` | `main` | **Works** for ACP input/picker state |
| `key down/enter` | `main` | Keys do not advance picker selection (`selectedLabel` stays `@file`) |
| `batch` + `simulateKey` | `main` | **FAIL** — `simulateKey` not a batch subcommand |

### Live acceptance proof

**Not green.** Picker opens and shows file category, but DevTools cannot yet drive row accept + read back `@file:<name.ext>` in `inputText` without native/ACP-specific picker navigation primitives.

**Not reproduced** the old bug (`@md:`, `@ts:`) in any live `inputText` sample — no extension-prefix tokens observed.

## Verdict

| Layer | Status | Notes |
| --- | --- | --- |
| Formatter fix (`part_to_inline_token`) | **GREEN** | 82 unit tests |
| Build | **GREEN** | `check --lib` |
| DevTools live picker → accept → token | **YELLOW** (`blocked-by-missing-primitive` / keyboard routing) | `getAcpState` proves picker opens; accept path needs `devtools.composer.inspect` or working `simulateKey`/batch picker navigation on `main` |

## Recommended next primitives

1. Map `devtools.act` / `batch` `setInput` to `targetKind: Ai` (alias to main ACP composer).
2. Route `act key` / `simulateKey` with `target: main` to ACP picker when `picker.open` (not ScriptList).
3. Add `batch` step or `waitFor` + `getAcpState` assertion helper for `@file:` token shape after `AcpItemAccepted`.
