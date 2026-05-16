[file-search-feature-map]

- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration asks Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Feature 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment has already been captured in FEATURE_MAP.md from Oracle session `main-menu-feature-map`.



- `source context expansion` with guessed `[[file search]]` refs failed because those section names do not exist; this is not a blocker because the prompt has no actual user refs to expand.
- `source checks` passed after the first FEATURE_MAP.md write.

- Bundle contains targeted snippets from process docs, file-search skill, actions/protocol skills, removed-docs verification/builtins/surfaces/automation, file_search modules, dedicated file search renderers/layout/list/preview/setup-key, root_file_search, attachment_portal, root_unified_result_actions, file action handlers, file path action builders, stdin simulateKey routing, and focused file-search/root-file tests.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out any high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Return only markdown suitable for appending to FEATURE_MAP.md after feature 001.

## 002 Dedicated File Search / Root Files / Directory Browse / File Actions

### Boundaries

### State Map

### Keystroke Matrix

### Visual/Protocol Matrix

### File Action Matrix

### Portal / Attachment Matrix

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.

- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
