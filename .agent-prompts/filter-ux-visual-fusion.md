# Fusion request: Script Kit GPUI filter picker rows exist semantically but do not paint

## User report

The user attached `/Users/johnlindquist/.codex/attachments/75ddbdf3-25db-41a8-9e6c-4c439237bcab/image-1.png`.
It shows the Script Kit main window with visible input `type:` and a blank body where the type picker rows should appear.

User explicitly requested:

- Verify every state is working/broken with screenshots from `$script-kit-devtools`.
- Ask `$fusion` how to fix.
- Implement the fix.
- Verify with screenshots in `$script-kit-devtools` again.

## Current repo state

- cwd: `/Users/johnlindquist/dev/script-kit-gpui`
- latest commit: `f36ed6efb Fix advanced filter picker head value UX`
- worktree before this investigation: clean except untracked `.herenow/`.
- New red artifacts from this investigation:
  - `.artifacts/filter-ux-devsh-red/report.json`
  - `.artifacts/filter-ux-devsh-red/01-colon.png`
  - `.artifacts/filter-ux-devsh-red/02-colon-t.png`
  - `.artifacts/filter-ux-devsh-red/03-colon-ty.png`
  - `.artifacts/filter-ux-devsh-red/04-accept-type-from-colon-ty.png`
  - `.artifacts/filter-ux-devsh-red/05-type-direct.png`
  - `.artifacts/filter-ux-devsh-red/06-type-s.png`
  - `.artifacts/filter-ux-devsh-red/07-type-scr.png`
  - `.artifacts/filter-ux-devsh-red/08-type-script-git.png`
  - `.artifacts/filter-ux-devsh-red/09-escape-from-type-after.png`
- DevTools session: `/tmp/sk-driver-sessions/filter-ux-devsh-red-89328-1-mqn0c4qp`
- Binary verified: `target/debug/script-kit-gpui`, i.e. the `./dev.sh` target/debug path.
- Real profile was used: `sandboxHome=false`, matching the user screenshot path.

## Critical red observation

The semantics say the picker is correct, but the screenshots show rows are not painted.

Example: `.artifacts/filter-ux-devsh-red/05-type-direct.png` visually shows only `type:` in the input and a blank body. This matches the user attachment.

But `.artifacts/filter-ux-devsh-red/report.json` says for `type:`:

```json
{
  "inputValue": "type:",
  "selectedValue": "type:script",
  "choiceCount": 8,
  "visibleChoiceCount": 8,
  "tokens": [
    "type:script",
    "type:scriptlet",
    "type:skill",
    "type:builtin",
    "type:app",
    "type:window",
    "type:agent",
    "type:issue"
  ],
  "fallbackRowsVisible": false,
  "visibleResultKeys": ["menu-syntax-trigger:qualifier-head:type"]
}
```

So the prior grammar/state fix is not enough. This is now a visual render/layout/paint bug:
`getState` and `getElements` expose rows, but `captureScreenshot` shows a blank list area.

## State matrix summary

All semantic checks passed, but the visual screenshots for picker-row states appear blank under the input:

- `:`: semantic rows are filter heads.
- `:t`: semantic rows are `type:`, `tag:`.
- `:ty`: semantic row is `type:`.
- Accept `type:` from `:ty`: semantic input is `type:`, rows are all type values.
- Direct `type:`: semantic rows are all type values, screenshot blank.
- `type:s`: semantic rows are script/scriptlet/skill, screenshot blank.
- `type:scr`: semantic rows are script/scriptlet, screenshot blank.
- `type:script git`: no picker rows, expected empty/no-results state.
- Escape from `type:` clears input in one press.

## Relevant source owners

- `src/app_impl/menu_syntax_trigger_picker_main_list.rs`
  - `menu_syntax_trigger_picker_owns_main_keyboard`
  - main-list trigger picker ownership/apply path.
- `src/app_impl/menu_syntax_trigger_picker.rs`
  - `trigger_picker_row_to_main_list_row`
  - converts trigger picker rows into main-list search rows.
- `src/app_impl/filtering_cache.rs`
  - `build_menu_syntax_trigger_picker_main_list_results`
  - `filtered_results`
  - `get_filtered_results_cached`
  - contains gates that return empty while trigger picker owns the list.
- `src/render_script_list/mod.rs`
  - `render_script_list`
  - likely visual renderer that should paint main-list rows.
- `src/main_sections/render_impl.rs`
  - main window render shell.
- `src/components/unified_list_item/**`
  - shared row renderer if main-list rows are being converted but not painted.

## Source snippets

`filtered_results()` and `get_filtered_results_cached()` currently return empty whenever `menu_syntax_trigger_picker_state.owns_main_list()` is true:

```rust
if self.menu_syntax_object_selector_state.owns_main_list()
    || self.menu_syntax_trigger_picker_state.owns_main_list()
    || crate::menu_syntax::active_filter_head_owns_main_list(filter_text)
    || self.menu_syntax_mode.capture_composer_owns_input_for(filter_text)
    || self.menu_syntax_mode.command_owns_input_for(filter_text)
{
    return Vec::new();
}
```

`getState.mainWindowPreflight.visibleResults` still exposes `menu-syntax-trigger:*` keys, and `getElements` still exposes rows with `role=menu-syntax-trigger-row`, so some automation path sees the rows.

Hypothesis: render_script_list is still using `filtered_results()` / grouped results for painting, while DevTools state/elements use a different preflight/main-list path that includes trigger picker rows. If true, the visual renderer is blank because the cache gate correctly suppresses fallback rows but fails to substitute the trigger picker main-list rows into the rendered list.

## Ask

Please diagnose the most likely render-path mismatch and recommend a minimal fix.

Specifically answer:

1. Which render function/path probably paints the blank body even though DevTools state exposes rows?
2. Should the fix make `render_script_list` consume the same menu-syntax trigger picker main-list rows used by `getState`/`getElements`, or should the cache gates be changed?
3. Which source files/functions should be edited?
4. Which existing tests should be updated or added? Avoid source-audit tests unless no behavior test/probe can express it.
5. What screenshot-based DevTools green proof should pass after the fix?

Constraints:

- Keep grammar behavior from commit `f36ed6efb`; do not regress semantic rows.
- Use `./scripts/agentic/agent-cargo.sh` for Rust commands.
- Verify against `target/debug/script-kit-gpui` / `./dev.sh` path, not only a named artifact.
- Preserve unrelated dirty/untracked work (`.herenow/`).
- The required final proof must include screenshots for every state, not only state JSON.
