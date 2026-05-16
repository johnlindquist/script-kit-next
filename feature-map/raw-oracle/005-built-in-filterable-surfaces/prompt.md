[builtin-filterable-map]

Project briefing:
- Repo: /Users/johnlindquist/dev/script-kit-gpui, Rust GPUI desktop app for Script Kit.
- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration asks Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Features 001-004 have already been captured in FEATURE_MAP.md from Oracle sessions `main-menu-feature-map`, `file-search-feature-map`, `acp-context-feature-map`, and `mcp-sdk-protocol-map`.
- Repo rules: run lat search/lat expand before work; update lat.md only when functionality/architecture/tests/behavior change; run lat check after every task before final response. This current task is docs-artifact generation only, not behavior change.
- Required repo context included in bundle: CLAUDE.md/AGENTS.md process context, owning built-in/actions/protocol skills, relevant lat.md pages including builtins, surfaces, automation.

Goal for this iteration:
Map FEATURE 005: Built-in Filterable Surfaces / Clipboard / App Launcher / Window Switcher / Browser Tabs / Emoji / Process Manager.

Scope to cover exhaustively in terse language:
- Shared built-in filterable surface contract: triggerBuiltin dispatch, AppView variants, surfaceContract, getState counts, getElements rows, actionsDialog host, footer/focus/navigation/filter patterns.
- Clipboard History: list/filter, text/image/file rows, preview, paste behavior, paste sequential if visible, actions, Quick Look/open with/reveal/copy/share/attach if visible, hidden/security/exclusion states visible in bundle.
- App Launcher: scan/cache/icon states, launch action, vendor folder/app bundle handling if visible, visible rows/filtering/actions.
- Window Switcher: visible rows, activation, filtering, actions if visible, triggerBuiltin route and receipts.
- Browser Tabs: enumeration/fuzzy ranking/activation if visible, getElements arm, root source relation if visible.
- Emoji Picker: category/search, Enter/row-click paste behavior, ArrowUp/ArrowDown, count asymmetry, clipboard paste pattern, actions if visible.
- Process Manager: visible rows, activity/process metadata, kill/cleanup actions if visible, deterministic test provider if visible.
- Every keystroke/state transition visible in the bundle: character input, arrows, Enter, Cmd+K, Escape, click, row action shortcuts, triggerBuiltin, filter clear, paste/activate behavior where visible.
- Visual/protocol states: surfaceContract, semanticSurface, visibleChoiceCount/choiceCount, getElements row ids/roles/warnings, activePopupContract/actionsDialog, matrix proof states.

Current evidence:
- `lat expand` on this feature name returned no bracket refs to expand.
- `lat search` returned relevant sections: `lat.md/surfaces#Surfaces#Current Surface Families`, `lat.md/automation#Automation#Filterable Surface Matrix`, `lat.md/automation#Automation#Surface Navigator`, and `lat.md/builtins#Built-ins#Emoji picker activation pastes like clipboard history`.
- `lat check` passed after feature 004 was appended.

Bundle map:
- Bundle contains targeted snippets from process docs, built-in/actions/protocol skills, lat.md builtins/surfaces/automation, builtins trigger registry/resolve/catalog, render_builtins files for clipboard/app launcher/window switcher/browser tabs/emoji/process manager/common, clipboard history/app launcher/emoji/process manager modules, and focused tests/source audits for row projection, counts, triggerBuiltin, root browser/clipboard sources, and emoji behavior.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Deliverable format:
Return only markdown suitable for appending to FEATURE_MAP.md after feature 004.
Use this exact structure:

## 005 Built-in Filterable Surfaces / Clipboard / App Launcher / Window Switcher / Browser Tabs / Emoji / Process Manager

### Boundaries
- Owns: ...
- Adjacent: ...
- Source anchors: ...
- Verification anchors: ...

### Shared Surface Matrix
Use terse bullet rows. Each row should be: `Surface | Entry/Trigger | State/Filter | Selection/Action | Protocol Receipt | Proof`.

### Keystroke Matrix
Use terse bullet rows. Each row should be: `Key | Preconditions | Handler/Owner | Result | Receipt/Test`.

### Surface-Specific State Map
Use terse bullet rows. Each row should be: `Surface | State | Entry | Visible/Protocol State | Exit/Next | Notes`.

### Actions Matrix
Use terse bullet rows. Each row should be: `Surface | Intent | Entry | Action/Handler | Visible State | Proof`.

### Visual/Protocol Matrix
Use terse bullet rows. Each row should be: `Surface | Visible State | Protocol/Automation Receipt | Failure Mode | Proof`.

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.
- Include `NEEDS_NEXT_PASS:` bullets for known omissions requiring another Oracle pass or more context.
- End with `Suggested next feature: 006 Notes Window / Notes Browse / Notes-hosted ACP` unless the bundle strongly suggests a better next slice.

Style constraints:
- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Output boundary:
Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
