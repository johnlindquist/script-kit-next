# Oracle Feature Atlas Loop Prompt

Use this prompt for each new feature-map Oracle pass. It is intentionally expansive because the atlas is meant to teach humans and implementation agents how every interaction works.

## Local Prep

1. Pick a stable feature id such as `012-window-resizing`.
2. Identify owner skills and adjacent skills.
3. Run `source context expansion "<feature prompt>"`.
4. Run `source search "<feature concepts>"` and read the relevant sections.
   - `AGENTS.md`
   - `CLAUDE.md`
   - owning `.agents/skills/<skill>/SKILL.md`
   - adjacent skills for crossed boundaries
   - relevant `removed-docs/` pages
   - `removed-docs`
   - focused source files, tests, and agentic scripts

## Prompt

```text
Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code or create downloadable artifacts.


## <feature id> <feature name>

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

## Required Saves


- `feature-map/raw-oracle/<feature-id>/prompt.md`
- `feature-map/raw-oracle/<feature-id>/bundle-map.md`
- `feature-map/raw-oracle/<feature-id>/answer.md`
- `feature-map/raw-oracle/<feature-id>/output.log`
- `feature-map/raw-oracle/<feature-id>/session.json`


- `feature-map/features/<feature-id>.md`
- `feature-map/index.md`
- `feature-map/receipts/oracle-sessions.md`

Run `source checks` before calling the loop done.
