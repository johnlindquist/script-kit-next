# Bundle Map

Feature atlas pass for `042-menu-syntax-power-capture`, focused on ScriptList menu syntax, capture composition, command invocation, and popup lifecycle.

## Oracle Session

- Primary slug: `menu-syntax-capture-atlas`
- Failed attempt slug: `menu-syntax-power-capture`
- Model: `gpt-5.5-pro`
- Engine: browser
- Thinking time: extended
- Bundle: `/Users/johnlindquist/.oracle/bundles/menu-syntax-power-capture.txt`
- Bundle size used by successful retry: 225,367 bytes
- Successful retry token estimate: about 61,222 input tokens
- Earlier focused bundle size: 178,437 bytes, 49,635 exact tokens, 132 matches, 29 context windows

The first browser attempt failed before submission because Oracle could not find the Thinking chip button. Its raw log and metadata are preserved as `output-failed-attempt.log` and `session-failed-attempt.json`.

## Lat Context

```bash
lat expand "042 Menu Syntax Power Commands and Capture Composer: semicolon plus colon todo calendar command heads trigger popup capture body composer qualifier insertion legacy trigger boundary"
lat search "menu syntax capture composer semicolon plus colon todo calendar command heads trigger popup qualifier insertion legacy trigger boundary"
```

Top relevant sections included:

- `lat.md/menu-syntax#Menu Syntax#MCAL Power Syntax Fragments`
- `lat.md/menu-syntax#Menu Syntax#Composable Natural Language Capture`
- `lat.md/menu-syntax#Menu Syntax#Command Invocation`
- `lat.md/menu-syntax#Menu Syntax#Power Syntax Grammar`
- `lat.md/menu-syntax#Menu Syntax#Demo Variety Pack`

The prompt restates the relevant `lat.md/menu-syntax.md` obligations because the final bundle is source-window focused to fit Oracle context limits.

## Owner Skills

- `.agents/skills/main-menu-search-selection/SKILL.md`
- `.agents/skills/actions-popups/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`

## Source And Test Context

The bundle was generated with context windows around menu-syntax symbols and strings, including:

- `src/menu_syntax/parse.rs`
- `src/menu_syntax/trigger_picker.rs`
- `src/menu_syntax/trigger_picker_keys.rs`
- `src/menu_syntax/capture.rs`
- `src/menu_syntax/capture_schema.rs`
- `src/menu_syntax/command.rs`
- `src/menu_syntax/filter.rs`
- `src/menu_syntax/metadata.rs`
- `src/menu_syntax/payload.rs`
- `src/menu_syntax/execute.rs`
- `src/app_impl/menu_syntax_trigger_popup.rs`
- `src/app_execute/menu_syntax_execution.rs`
- `src/scripts/grouping.rs`
- `tests/file_search_tilde_entry.rs`
- `tests/menu_syntax_source_filters.rs`

It also includes `AGENTS.md`, `CLAUDE.md`, owner skills, and existing chapters `001` and `013` so Oracle can distinguish this focused menu-syntax slice from broader launcher and special-entry behavior.
