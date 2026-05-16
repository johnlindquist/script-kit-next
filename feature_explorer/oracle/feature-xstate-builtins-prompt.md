[feature-xstate-builtins]

- The feature atlas lives in `feature-map/features/*.md`; the explorer turns those chapters into wireframe/runtime XState models.
- Authored machine examples for Features 001 through 004 already exist in `feature_explorer/src/state/authoredFeatureMachines.ts`.
- Any local implementation that changes behavior, architecture, tests, or contracts must update `removed-docs/` and pass `source checks`.

Design a paste-ready authored machine for Feature 005, `Built-in Filterable Surfaces`, using the existing `AuthoredFeatureMachineConfig` schema and local style.

- `npm run build` in `feature_explorer/` passes.
- `source checks` passes.
- Generated explorer coverage reports 42 index rows, 42 raw Oracle sessions, and 42 feature chapters.
- Authored runtime coverage currently includes Features 001 through 004.

- Included Feature 005 distilled chapter.
- Included current explorer runtime/authored-machine code.
- Included owning skills for built-in filterable surfaces, protocol automation, agentic testing, and quality gates.

1. A state inventory for Feature 005.
2. An event/transition inventory.
3. A TypeScript object literal compatible with the current authored-machine file.
4. Any minimal type/schema edits needed.
5. Static/build and later state-first verification guidance.
6. Ambiguities that should remain visible instead of invented.

- canonical triggerBuiltin route resolution and route planning;
- shared filter input, visible-row projection, selection, wheel/scroll, Escape, Enter, and Cmd+K contracts;
- Clipboard History preview/portal/action states;
- App Launcher scan/filter/launch states;
- Window Switcher preload/filter/focus states;
- Browser Tabs preload/filter/activate states;
- Emoji Picker search/category/grid/paste states;
- Process Manager active/empty/filter/refresh/stop states;
- root passive source boundaries and collector-warning/error states.

Return your answer as text in this response only. Do not create, attach, export,
or offer any downloadable file. Do not create local project artifacts yourself.
The local agent will write any needed files, plans, notes, goals, commits, or
verification logs using local tools.
