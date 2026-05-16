[feature-xstate-agent-context]

- The repo uses `removed-docs/` as the architecture/test knowledge graph. Any local implementation that changes behavior, architecture, or verification contracts must update `removed-docs/` and pass `source checks`.
- The feature atlas lives in `feature-map/features/*.md`, with raw Oracle harvests under `feature-map/raw-oracle/*/answer.md`.
- The Feature Explorer reads the atlas and builds executable XState runtime models. It already has authored machines for Features 001 and 002 in `feature_explorer/src/state/authoredFeatureMachines.ts`, and falls back to derived table parsing for all other features.

Create an implementation-ready authored XState representation for Feature 003, `Agent Chat Context Composer`, that can be added to `feature_explorer/src/state/authoredFeatureMachines.ts` using the existing `AuthoredFeatureMachineConfig` schema.

- The local agent has already added authored machines for Features 001 and 002.
- `npm run build` in `feature_explorer/` passes.
- `source checks` passes.
- Generated explorer coverage reports 41 index rows, 41 raw Oracle sessions, and 41 feature chapters.
- The next local implementation should add Feature 003 without rewriting the runtime schema unless a schema limitation is unavoidable.

- Included Feature 003 chapter and its raw Oracle harvest.
- Included current explorer state/runtime code, including the authored machines for 001/002 as the local style target.
- Included owning skills for ACP composer/chat, MCP context resources, protocol automation, and agentic testing.

Return a concrete authored-machine design for Feature 003 that a local agent can paste/adapt into `feature_explorer/src/state/authoredFeatureMachines.ts`.

1. A compact state inventory with state ids, labels, status, wireframe hints, and why each state exists.
2. An event inventory with transition targets, especially slash command, inline mention, attachment token, portal return, focused target, submit, cancel, and history/session flows.
3. A TypeScript object literal compatible with the existing `AuthoredFeatureMachineConfig` type and style. Prefer direct code that matches the existing file over pseudocode.
4. Notes on whether the current schema is sufficient; if not, propose the smallest schema addition and explain why it is worth it.
6. Any unresolved ambiguity that should remain visible in the explorer instead of being invented.

- Respect the `removed-docs/` update rule and required `source checks`.
- Do not ask for downloadable files or create project artifacts.
- Keep the output focused on real implementation progress for this iteration, not a vague plan.

Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.
