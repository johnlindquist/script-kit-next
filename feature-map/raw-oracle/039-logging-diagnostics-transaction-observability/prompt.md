# 039 Logging, Diagnostics, and Transaction Observability

Produce a complete operator-grade feature atlas for Script Kit GPUI feature 039, Logging, Diagnostics, and Transaction Observability.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Cover the observability surface as a product/operator feature:

- `./dev.sh`, compact AI logs, JSONL logs, and log-driven iteration.
- Safe user-value previews, byte caps, and rate-limited untrusted-value logging.
- Stable debug trace markers such as `DO_IN_TRACE`, `SCROLL_TRACE`, and input-history trace targets.
- Protocol transaction traces, transaction replay, bounded retention, malformed JSONL recovery, and MCP trace resources.
- AI preflight audit logs, correlation-level dedupe, compaction, and schema drift handling.
- Protocol stats/reporting and source-audit logging contracts.
- Verification recipes for logging safety, trace propagation, transaction trace persistence, and runtime receipt proof.

Use this output shape:

```markdown
## 039 Logging, Diagnostics, and Transaction Observability

### Executive Summary
### What Users Can Do
### Core Concepts
### Entry Points
### User Workflows
### Interaction Matrix
| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
### State Machine
### Visual And Focus States
### Keystrokes And Commands
### Actions And Menus
### Automation And Protocol Surface
### Data, Storage, And Privacy Boundaries
### Error, Empty, Loading, And Disabled States
### Code Ownership
### Invariants And Regression Risks
### Verification Recipes
### Agent Notes
### Related Features
### Open Questions And Gaps
```
