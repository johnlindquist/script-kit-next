[file-search-feature-map]

Project briefing:
- Repo: /Users/johnlindquist/dev/script-kit-gpui, Rust GPUI desktop app for Script Kit.
- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration asks Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Feature 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment has already been captured in FEATURE_MAP.md from Oracle session `main-menu-feature-map`.
- Repo rules: run lat search/lat expand before work; update lat.md only when functionality/architecture/tests/behavior change; run lat check after every task before final response. This current task is docs-artifact generation only, not behavior change.
- Required repo context included in bundle: CLAUDE.md/AGENTS.md process context, owning file-search/actions/protocol skills, and relevant lat.md pages including verification, builtins, surfaces, automation.

Goal for this iteration:
Map FEATURE 002: Dedicated File Search / Root Files / Directory Browse / File Actions.

Scope to cover exhaustively in terse language:
- Dedicated File Search surface: entry paths from `~`, `/`, root continuation rows, file actions, portals, attachment flows, triggerBuiltin if visible, mini/full behavior if visible.
- Root file search states that bridge into dedicated file search: recent files, global file query, directory browse, child fragment, source-filter `f:`/`files:` states, loading/cached/empty/capped/exhausted states.
- Directory navigation: enter directory, parent/browse parent, Search Inside Folder, hidden file behavior, path display/home-prefix shortening, direct child listing vs recursive/global search.
- File row actions: open, reveal, copy path/name, Quick Look, Browse Parent Folder, Search Inside Folder, Attach to AI, open in Quick Terminal/editor/Finder where visible; file-vs-directory action differences.
- Every keystroke/state transition visible in the bundle: character input, arrows, Enter, Tab, Shift+Tab, Cmd+K, Escape, Cmd+Y, Cmd+Shift+F, Cmd+Shift+C, drag, action shortcuts, stdin simulateKey parity where visible.
- Visual states and protocol/automation receipts relevant to this feature: surfaceContract, semantic surface, file_search state fields, mainWindowPreflight/root file rows, actionsDialog receipts, preview pane, drag-out rows, mutation refresh receipts, portal receipts.

Current evidence:
- `lat expand` with guessed `[[file search]]` refs failed because those section names do not exist; this is not a blocker because the prompt has no actual user refs to expand.
- `lat search` returned relevant sections: `lat.md/verification#Verification#Root Unified Search Result Actions`, `lat.md/builtins#Built-ins#Root Unified Search Result Actions`, and `lat.md/verification#Verification#Root Recent File Seed Pool`.
- `lat check` passed after the first FEATURE_MAP.md write.

Bundle map:
- Bundle contains targeted snippets from process docs, file-search skill, actions/protocol skills, lat.md verification/builtins/surfaces/automation, file_search modules, dedicated file search renderers/layout/list/preview/setup-key, root_file_search, attachment_portal, root_unified_result_actions, file action handlers, file path action builders, stdin simulateKey routing, and focused file-search/root-file tests.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out any high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Deliverable format:
Return only markdown suitable for appending to FEATURE_MAP.md after feature 001.
Use this exact structure:

## 002 Dedicated File Search / Root Files / Directory Browse / File Actions

### Boundaries
- Owns: ...
- Adjacent: ...
- Source anchors: ...
- Verification anchors: ...

### State Map
Use terse bullet rows. Each row should be: `State | Entry | Input/Keystroke | Visible/Protocol State | Exit/Next | Notes`.

### Keystroke Matrix
Use terse bullet rows. Each row should be: `Key | Preconditions | Handler/Owner | Result | Receipt/Test`.

### Visual/Protocol Matrix
Use terse bullet rows. Each row should be: `Surface | Visible State | Protocol/Automation Receipt | Failure Mode | Proof`.

### File Action Matrix
Use terse bullet rows. Each row should be: `Intent | Entry | Handler/Action ID | File-vs-Directory Rule | Visible State | Proof`.

### Portal / Attachment Matrix
Use terse bullet rows. Each row should be: `Intent | Entry | Context/Portal State | Exit/Return | Failure Mode | Proof`.

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.
- Include `NEEDS_NEXT_PASS:` bullets for known omissions requiring another Oracle pass or more context.
- End with `Suggested next feature: 003 ACP Chat / Agent Catalog / Context Composer / Attachment Portals` unless the bundle strongly suggests a better next slice.

Style constraints:
- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Output boundary:
Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
