# 037 Storybook, Design Explorer, and Visual Verification Oracle Prompt

Produce a complete operator-grade feature atlas for humans and AI agents.

Prefer clarity, explicit examples, and workflow detail over compression. Explain every user workflow, UI state, shortcut, action, state transition, automation receipt, data boundary, edge case, failure mode, and verification path. Do not summarize away behavior.

Ground claims in the attached repo context. Prefer concrete file/function/test/script references. Mark uncertain claims as inference. Do not write code, create downloadable artifacts, or ask the user to open files. Return text only.

## Feature Scope

Feature id: `037-storybook-design-visual-verification`

Feature name: Storybook, Design Explorer, and Visual Verification

Cover these Script Kit GPUI capabilities:

- Storybook as the visual state lab: story registry, catalog roles, browser, story selection, comparable stories, child preview windows, popup preview windows, lifecycle registry, and theme synchronization.
- Canonical state stories versus adoptable variations versus archived design experiments.
- Main-menu Storybook production adoption contracts, compare-mode contracts, representation quality metadata, catalog JSON, diagnostics, and audit reports.
- Design Gallery built-in surface, triggerBuiltin route, selection footer, state/elements receipts, and design-picker visual matrix.
- Agentic visual verification: surface navigator, image-library manifests, strict-window screenshot capture, attached-popup crop proof, verify-shot content audit, blank/black screenshot rejection, fresh-per-case sweeps, and all-active matrices.
- Storybook/design relationship to the standalone `feature_explorer` XState mockup: Storybook proves visual states; feature_explorer explores feature/state contracts.

Explicitly distinguish:

- Storybook visual state coverage versus production behavior proof.
- Live-surface and presenter-fixture stories versus old PNG/runtime fixture experiments.
- Design Gallery as a built-in product surface versus Storybook browser as a developer lab.
- Screenshot proof versus state-first receipts.
- Visual proof infrastructure failures versus product visual regressions.

## Required Output Shape

```markdown
## 037 Storybook, Design Explorer, and Visual Verification

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

## Context Notes

This repository requires `lat check` after atlas updates. The maintained atlas preserves the complete Oracle answer under `feature-map/raw-oracle/<feature-id>/answer.md` and distills a readable chapter under `feature-map/features/<feature-id>.md`.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
