# Oracle Prompt: Feature XState 001-005

Produce implementation-ready XState definitions for Script Kit GPUI feature-map chapters 001-005.

The local goal is to make `feature_explorer/` a visual XState-driven wireframe/mockup of the real app, not just a document browser. The app currently parses `feature-map/features/*.md`, derives state rows, events, and conservative transitions, and renders them with XState actors. We need richer authored machine definitions that preserve user workflows and can be checked into the repo as generated or maintained data.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.

For each feature chapter in this bundle:

- Build an organized XState-compatible machine definition.
- Include stable `id`, `initial`, `states`, `on`, nested states where useful, guards, actions, invoked service placeholders, and tags.
- Include scenario metadata that the explorer can render as a wireframe: entry points, visible regions, selected item, input/filter text, active popup/portal, footer owner, errors/loading/empty states, and expected proof/receipt.
- Include event names that are stable and human-readable.
- Preserve cross-feature handoffs such as main menu to file search, ACP portals, MCP/protocol calls, and built-in filterable surfaces.
- Separate the canonical machine from notes where the current chapter lacks enough precision.
- Identify which parts should be implemented now in `feature_explorer` versus which should wait for later slices.

Use this output shape:

```markdown
## Feature XState 001-005

### Shared Schema
```ts
// TypeScript interfaces local agent should implement.
```

### Machines
```ts
// One object per feature, usable as data for createMachine().
```

### Explorer Integration Plan

### Coverage And Weak Spots

### Verification Plan
```
