[acp-context-feature-map]

- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration asks Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Feature 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment is captured from Oracle session `main-menu-feature-map`.
- Feature 002 Dedicated File Search / Root Files / Directory Browse / File Actions is captured from Oracle session `file-search-feature-map`.



- `source context expansion` on this feature name returned no bracket refs to expand.
- `source checks` passed after feature 002 was appended.

- Bundle contains targeted snippets from process docs, ACP chat/context/MCP skills, removed-docs acp-chat/ai-context/acp portal contract, ACP catalog/chat_window/composer/config/context/history/model/picker/popup/portal/surface/thread/view files, context mentions, context snapshot, attachment portal, tab_ai_mode entry/staging/launch files, and focused ACP/context tests.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Return only markdown suitable for appending to FEATURE_MAP.md after feature 002.

## 003 ACP Chat / Agent Catalog / Context Composer / Attachment Portals

### Boundaries

### State Map

### Keystroke Matrix

### Visual/Protocol Matrix

### Context / Portal Matrix

### Agent / Model Matrix

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.

- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
