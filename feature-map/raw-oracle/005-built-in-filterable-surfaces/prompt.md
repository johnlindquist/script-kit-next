[builtin-filterable-map]

- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration asks Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Features 001-004 have already been captured in FEATURE_MAP.md from Oracle sessions `main-menu-feature-map`, `file-search-feature-map`, `acp-context-feature-map`, and `mcp-sdk-protocol-map`.



- `source context expansion` on this feature name returned no bracket refs to expand.
- `source checks` passed after feature 004 was appended.

- Bundle contains targeted snippets from process docs, built-in/actions/protocol skills, removed-docs builtins/surfaces/automation, builtins trigger registry/resolve/catalog, render_builtins files for clipboard/app launcher/window switcher/browser tabs/emoji/process manager/common, clipboard history/app launcher/emoji/process manager modules, and focused tests/source audits for row projection, counts, triggerBuiltin, root browser/clipboard sources, and emoji behavior.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Return only markdown suitable for appending to FEATURE_MAP.md after feature 004.

## 005 Built-in Filterable Surfaces / Clipboard / App Launcher / Window Switcher / Browser Tabs / Emoji / Process Manager

### Boundaries

### Shared Surface Matrix

### Keystroke Matrix

### Surface-Specific State Map

### Actions Matrix

### Visual/Protocol Matrix

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.

- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
