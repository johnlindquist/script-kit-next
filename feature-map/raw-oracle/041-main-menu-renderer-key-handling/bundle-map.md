# Bundle Map

Feature atlas pass for `041-main-menu-renderer-key-handling`, focused on the main ScriptList renderer and key routing gaps identified by `001-main-menu`.

## Oracle Session

- Slug: `main-menu-renderer-keys`
- Model: `gpt-5.5-pro`
- Engine: browser
- Thinking time: extended
- Bundle: `/Users/johnlindquist/.oracle/bundles/main-menu-renderer-keys.txt`
- Bundle size: 191,656 bytes
- Packx estimate: 24 files, 40,560 exact tokens, 122 matches, 16 context windows

## Lat Context

```bash
lat expand "041 Main Menu Renderer Key Handling: Cmd+Enter non-file Tab action shortcut execution popup-first ordering ScriptList render_script_list keyboard actions"
lat search "Main Menu Renderer Key Handling Cmd+Enter non-file Tab action shortcut execution popup-first ordering ScriptList render_script_list keyboard actions"
```

Top relevant results included `lat.md/builtins#Built-ins#Favorites Cmd+K opens a six-row actions menu`, `lat.md/menu-syntax#Menu Syntax`, and keyboard behavior sections for adjacent surfaces.

## Owner Skills

- `.agents/skills/main-menu-search-selection/SKILL.md`
- `.agents/skills/keyboard-focus-routing/SKILL.md`
- `.agents/skills/actions-popups/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`

## Source And Test Context

The bundle was generated with context windows around key-routing symbols and strings, including:

- `src/render_script_list/mod.rs`
- `src/app_impl/actions_dialog.rs`
- `src/app_impl/actions_toggle.rs`
- `src/app_impl/root_unified_result_actions.rs`
- `src/app_impl/shortcut_recorder.rs`
- `src/app_actions/handle_action/shortcuts.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `tests/source_audits/root_unified_source_actions_contract.rs`
- `tests/source_audits/shortcut_config_source.rs`
- `tests/file_search_tilde_entry.rs`

The bundle also includes `AGENTS.md`, `CLAUDE.md`, relevant `lat.md` pages, and existing chapters `001`, `011`, `013`, and `022` so Oracle can distinguish already-covered behavior from the focused renderer-key gap.
