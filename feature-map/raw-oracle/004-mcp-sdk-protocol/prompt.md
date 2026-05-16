[mcp-sdk-protocol-map]

Project briefing:
- Repo: /Users/johnlindquist/dev/script-kit-gpui, Rust GPUI desktop app for Script Kit.
- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration asks Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Features 001-003 have already been captured in FEATURE_MAP.md from Oracle sessions `main-menu-feature-map`, `file-search-feature-map`, and `acp-context-feature-map`.
- Repo rules: run lat search/lat expand before work; update lat.md only when functionality/architecture/tests/behavior change; run lat check after every task before final response. This current task is docs-artifact generation only, not behavior change.
- Required repo context included in bundle: CLAUDE.md/AGENTS.md process context, owning protocol/MCP/SDK skills, relevant lat.md pages including protocol, automation, ai-context, scripting.

Goal for this iteration:
Map FEATURE 004: MCP Resources / SDK / Protocol Automation.

Scope to cover exhaustively in terse language:
- Stdin JSON protocol: version envelope, deprecation registry, ingress observation, parse/error recovery, requestId behavior, fire-and-forget show/hide/simulateKey, setFilter/setInput, triggerBuiltin, hide/reset, simulateKey routing, ExternalCommand families visible in bundle.
- Query/automation APIs: getState, getElements, getLayoutInfo, captureScreenshot, getAcpState/getAcpTestProbe if visible, waitFor, batch, transaction executor/traces, target identity, automation windows, surface contracts, active popup/footer receipts.
- MCP resources: kit://state, scripts://, scriptlets://, kit://scripts, kit://scriptlets, kit://sdk-reference, kit://context, kit://context/schema, clipboard/focused/git/process/system/dictation/calendar/notifications/stdin-commands/trigger-builtins/diagnostics/transaction resources; resource drift audits and schema/versioning.
- MCP tools/server: app-owned HTTP MCP server, script-derived tools, static tools, computer/* observation/capture/menu/tray/screen/permission tools visible in bundle; read-only boundaries and exact action exclusions.
- SDK/scriptability: scripts/kit-sdk.ts automation helpers, SDK reference resource, triggerBuiltin literals, computer namespace if visible, script execution metadata boundaries visible in bundle.
- Agentic proof surface: scripts/agentic index/session/surface navigator where visible; getState/getElements/waitFor/batch proof patterns and cleanup.
- Every state/scenario/visual/protocol state visible in the bundle. This slice is not primarily a UI feature, so include protocol-visible and resource-visible states instead of forcing visual-only rows.

Current evidence:
- `lat expand` on this feature name returned no bracket refs to expand.
- `lat search` returned relevant sections: `lat.md/protocol#Protocol`, `lat.md/protocol#Protocol#MCP resources`, `lat.md/ai-context#AI Context and MCP#MCP context resources`, and `lat.md/automation#Automation#Key Facts`.
- `lat check` passed after feature 003 was appended.

Bundle map:
- Bundle contains targeted snippets from process docs, protocol/MCP/SDK skills, lat.md protocol/automation/ai-context/scripting, protocol modules, stdin commands, runtime stdin dispatchers, MCP resources/server/protocol/tool files, computer-use MCP tools, SDK file, agentic index, protocol/wait/batch/MCP resource drift/SDK automation tests, and trigger-builtin/source-audit tests.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Deliverable format:
Return only markdown suitable for appending to FEATURE_MAP.md after feature 003.
Use this exact structure:

## 004 MCP Resources / SDK / Protocol Automation

### Boundaries
- Owns: ...
- Adjacent: ...
- Source anchors: ...
- Verification anchors: ...

### Protocol State Map
Use terse bullet rows. Each row should be: `State/Command | Entry | Input Shape | Response/Side Effect | Follow-up Receipt | Notes`.

### Automation Query Matrix
Use terse bullet rows. Each row should be: `Query | Target/Preconditions | Result Shape | Failure/Diagnostic State | Proof`.

### MCP Resource Matrix
Use terse bullet rows. Each row should be: `Resource | Entry/URI | Payload Shape | Freshness/Source of Truth | Drift Risk | Proof`.

### MCP Tool Matrix
Use terse bullet rows. Each row should be: `Tool Family | Input Shape | Result/Observation | Explicit Non-Goals | Proof`.

### SDK / Agentic Matrix
Use terse bullet rows. Each row should be: `Intent | SDK/Script Entry | Runtime Route | Receipt/State | Failure Mode | Proof`.

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.
- Include `NEEDS_NEXT_PASS:` bullets for known omissions requiring another Oracle pass or more context.
- End with `Suggested next feature: 005 Built-in Filterable Surfaces / Clipboard / App Launcher / Window Switcher / Browser Tabs / Emoji / Process Manager` unless the bundle strongly suggests a better next slice.

Style constraints:
- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Output boundary:
Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
