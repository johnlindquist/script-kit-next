# 038 Agent Skills and AI Context Catalog

Produce a complete operator-grade feature atlas for Script Kit GPUI feature 038, Agent Skills and AI Context Catalog.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.

Cover the agent-facing skill/context surface as a product feature:

- Repo-local `.agents/skills` and `.agents/subagents` routing topology.
- AI context snapshots and capture profiles.
- `kit://context`, `kit://context/schema`, and adjacent MCP resources.
- `AiContextPart` variants and submit-time resolution receipts.
- Context picker, context preview metadata, attachment portal return, and focused-target handoff.
- Skill file/resource staging, script/skill search, and agent-facing catalog boundaries.
- Verification recipes for context schemas, MCP resources, composer parts, and portal flows.

Use this output shape:

```markdown
## 038 Agent Skills and AI Context Catalog

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
