[feature-xstate-composer-lite]

- The feature atlas lives in `feature-map/features/*.md`; the explorer turns those chapters into wireframe/runtime XState models.
- Authored machine examples for Features 001 and 002 already exist in `feature_explorer/src/state/authoredFeatureMachines.ts`.
- Any local implementation that changes behavior, architecture, tests, or contracts must update `removed-docs/` and pass `source checks`.

Design a paste-ready authored machine for Feature 003, `Agent Chat Context Composer`, using the existing `AuthoredFeatureMachineConfig` schema and local style.

- A larger raw-harvest bundle for the same feature may be too large for ChatGPT display. This smaller bundle intentionally uses the distilled Feature 003 chapter plus current explorer/runtime code and owner skills.
- Do not require a broad schema rewrite. If the existing enums are too launcher/file-search-specific for ACP, propose only the smallest enum additions needed to make ACP wireframes honest.

1. A state inventory for Feature 003.
2. An event/transition inventory.
3. A TypeScript object literal compatible with the current authored-machine file.
4. Any minimal type/schema edits needed.
5. Static/build and later state-first verification guidance.
6. Ambiguities that should remain visible instead of invented.

- embedded, detached, and setup-required ACP entries;
- composer idle/draft state;
- slash and mention popup;
- context token/pasted-token staging;
- portal staged, active, accepted, cancelled, and refused paths;
- focused portal reopen;
- agent/model switch with draft preservation;
- streaming cancel before close;
- actions/history popup surfaces.

Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.
