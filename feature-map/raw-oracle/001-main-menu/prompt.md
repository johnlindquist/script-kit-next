[main-menu-feature-map]

Project briefing:
- Repo: /Users/johnlindquist/dev/script-kit-gpui, Rust GPUI desktop app for Script Kit.
- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration must ask Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, then the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.
- Repo rules: run lat search/lat expand before work; update lat.md only when functionality/architecture/tests/behavior change; run lat check after every task before final response. This current task is docs-artifact generation only, not behavior change.
- Required repo context included in bundle: CLAUDE.md/AGENTS.md process context, owning skills for main menu/search/actions/keyboard/protocol, and relevant lat.md pages including verification, builtins, surfaces, automation, shortcuts.

Goal for this iteration:
Map FEATURE 001: Main Menu / Script List / Menu Syntax / Actions and Shortcut Assignment states.

Scope to cover exhaustively in terse language:
- Main menu empty/default state, search/filter entry, grouped/fuzzy results, fallbacks, selected row behavior, info/preview affordances if visible in code/docs.
- Special trigger character and token states in the main menu: include examples such as `f:`, `@`, `has:`, `has:shortcut`, partial-vs-terminal query states, menu-syntax popup open/closed behavior, chips/decorations, stale decoration risks, trigger picker rows and keyboard handling.
- Every keystroke/state transition visible in the bundle for this slice: character input, arrow/nav, Enter, Tab, Cmd+Enter, Cmd+K, Escape, backspace/delete where applicable, physical vs stdin simulateKey parity where documented.
- Actions menu from the main menu / selected result: open, close, toggle, focus restore, popup/attached popup state, selected action execution, action rows/sections, shortcut glyphs.
- Shortcut assignment/update/remove flow for scripts/builtins/agents/scriptlets where visible: Add Shortcut, Update Shortcut, Remove Shortcut, shortcut recorder state, config.ts writes through scripts/config-cli.ts, hotkey registration/refresh behavior, config-backed command shortcuts as source of truth.
- Visual states and protocol/automation receipts relevant to this feature: surfaceContract, promptPopup/actionsDialog windows, filterInputDecorations, getState/getElements surfaces, verification gates from lat.md.

Current evidence:
- `lat expand` on the user request returned no bracket refs to expand.
- `lat search` found relevant sections: `lat.md/verification#Verification#Main menu and footer`, `lat.md/builtins#Built-ins#Key Files`, and `lat.md/acp-chat#ACP Chat#Entry paths`.
- FEATURE_MAP.md does not exist yet in the repo root, so this should be an initial append-ready section.

Bundle map:
- Bundle contains targeted snippets from process docs, repo skills, lat.md docs, main menu/search/rendering, menu_syntax, filter input change/update paths, actions dialog/toggle, root unified result actions, shortcut recorder, runtime simulateKey, hotkeys, config-cli/config-schema, and source audit tests.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out any high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Deliverable format:
Return only markdown suitable for direct insertion into FEATURE_MAP.md.
Use this exact structure:

# FEATURE_MAP.md

## Coverage Index
- [x] 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment
- [ ] 002 <suggest next feature slice>
- [ ] 003 <suggest next feature slice>

## 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment

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

### Shortcut Assignment / config.ts Matrix
Use terse bullet rows. Each row should be: `Intent | Entry | Recorder/Config Path | Runtime Refresh | Visible State | Proof`.

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.
- Include `NEEDS_NEXT_PASS:` bullets for any known omissions that require another Oracle pass or more context.

Style constraints:
- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Output boundary:
Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
