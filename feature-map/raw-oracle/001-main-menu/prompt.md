[main-menu-feature-map]

- The user wants a 100% exhaustive FEATURE_MAP.md built iteratively. Each iteration must ask Oracle to map one feature's user stories/states/keystrokes/scenarios/visual states tersely, then the local agent writes Oracle's response into FEATURE_MAP.md, then continues to the next feature.


- Main menu empty/default state, search/filter entry, grouped/fuzzy results, fallbacks, selected row behavior, info/preview affordances if visible in code/docs.

- `source context expansion` on the user request returned no bracket refs to expand.
- FEATURE_MAP.md does not exist yet in the repo root, so this should be an initial append-ready section.

- Bundle contains targeted snippets from process docs, repo skills, removed-docs docs, main menu/search/rendering, menu_syntax, filter input change/update paths, actions dialog/toggle, root unified result actions, shortcut recorder, runtime simulateKey, hotkeys, config-cli/config-schema, and source audit tests.
- Bundle may omit unrelated app features and some full source bodies due token limits; stay within the scoped feature and call out any high-confidence gaps as `NEEDS_NEXT_PASS` items rather than inventing.

Return only markdown suitable for direct insertion into FEATURE_MAP.md.

# FEATURE_MAP.md

## Coverage Index
- [x] 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment
- [ ] 002 <suggest next feature slice>
- [ ] 003 <suggest next feature slice>

## 001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment

### Boundaries

### State Map

### Keystroke Matrix

### Visual/Protocol Matrix

### Shortcut Assignment / config.ts Matrix

### Exhaustiveness Notes
- Include every explicit scenario/state you can infer from the bundle.

- Be terse, but do not drop edge cases.
- Prefer exact product terms and source/function names from the bundle.
- Do not write explanatory prose outside the markdown artifact.
- Do not mention that you are Oracle.

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
