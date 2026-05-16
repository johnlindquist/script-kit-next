[mcp-sdk-protocol-map]

- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration asks Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Features 001-003 have already been captured in FEATURE_MAP.md from Oracle sessions `main-menu-feature-map`, `file-search-feature-map`, and `acp-context-feature-map`.


- Every state/scenario/visual/protocol state visible in the bundle. This slice is not primarily a UI feature, so include protocol-visible and resource-visible states instead of forcing visual-only rows.

- `source context expansion` on this feature name returned no bracket refs to expand.
- `source checks` passed after feature 003 was appended.

- Bundle contains targeted snippets from process docs, protocol/MCP/SDK skills, removed-docs protocol/automation/ai-context/scripting, protocol modules, stdin commands, runtime stdin dispatchers, MCP resources/server/protocol/tool files, computer-use MCP tools, SDK file, agentic index, protocol/wait/batch/MCP resource drift/SDK automation tests, and trigger-builtin/source-audit tests.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Return only markdown suitable for appending to FEATURE_MAP.md after feature 003.

## 004 MCP Resources / SDK / Protocol Automation

### Boundaries

### Protocol State Map

### Automation Query Matrix

### MCP Resource Matrix

### MCP Tool Matrix

### SDK / Agentic Matrix

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.

- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
