# State Management Improvements Audit

## Scope
- Reviewed `src/**/*.rs` with focus on `ScriptListApp` state, `AppView` state, prompt/view transitions, focus handling, filter/search state, list selection state, and externally exposed app state.
- Files inspected in depth: `src/main.rs`, `src/app_impl.rs`, `src/app_navigation.rs`, `src/prompt_handler.rs`, `src/focus_coordinator.rs`, `src/window_state.rs`, `src/mcp_kit_tools.rs`, `src/mcp_resources.rs`, `src/mcp_server.rs`.

## Executive Summary
- State is functional but overly distributed. The current design relies on multiple parallel state systems (especially for focus and filtering) plus duplicated reset logic.
- Main risk class: state drift. Multiple code paths mutate the same conceptual state with different mechanisms.
- Highest-value fix: define single sources of truth for focus and active query, then route transitions through a small set of reducer-style methods.

## Findings (Prioritized)

### 1) Dual focus state machines are active at once (high)
Evidence:
- Legacy fields remain primary in many paths: `focused_input`, `pending_focus` (`src/main.rs:1350`, `src/main.rs:1453`).
- New coordinator exists in parallel (`src/main.rs:1456`, initialized in `src/app_impl.rs:405`).
- Prompt transitions still write legacy fields directly (`src/prompt_handler.rs:73`, `src/prompt_handler.rs:75`, `src/prompt_handler.rs:171`, `src/prompt_handler.rs:172`, `src/prompt_handler.rs:191`, `src/prompt_handler.rs:192`, `src/prompt_handler.rs:1476`, `src/prompt_handler.rs:1477`, `src/prompt_handler.rs:1764`, `src/prompt_handler.rs:1765`).
- Non-prompt paths also set legacy focus directly (`src/main.rs:481`, `src/main.rs:482`, `src/app_impl.rs:189`, `src/app_impl.rs:215`, `src/app_impl.rs:6593`, `src/app_impl.rs:6959`).
- Coordinator sync is lossy for chat cursor semantics (`src/app_impl.rs:1618` maps `ChatPrompt` cursor owner to `FocusedInput::None`).

Impact:
- Focus restoration behavior depends on the path taken (coordinator path vs direct legacy writes).
- Cursor blink ownership can drift from actual focused element.
- Overlay push/pop restore can infer the wrong prior focus when coordinator cursor owner is stale.

Recommendation:
- Make `FocusCoordinator` the only mutable focus state.
- Replace direct `focused_input`/`pending_focus` writes with one API, e.g. `request_focus(FocusRequest)`.
- Keep legacy fields as derived read-only compatibility fields until removed.
- Add a single transition log for each focus request/apply with `correlation_id`.

### 2) Query/filter state is duplicated across app + view + input component (high)
Evidence:
- Global filter state cluster: `filter_text`, `computed_filter_text`, `gpui_input_state`, `pending_filter_sync`, `suppress_filter_events` (`src/main.rs:1262`, `src/main.rs:1264`, `src/main.rs:1277`, `src/main.rs:1275`, `src/main.rs:1364`).
- Built-in views also keep independent query/filter fields inside `AppView` (`src/main.rs:856`, `src/main.rs:863`, `src/main.rs:869`, `src/main.rs:874`, `src/main.rs:893`, `src/main.rs:898`).
- Synchronization is manual and branch-heavy in handlers (`src/app_impl.rs:2727`, `src/app_impl.rs:2754`, `src/app_impl.rs:2767`, `src/app_impl.rs:2780`, `src/app_impl.rs:2793`, `src/app_impl.rs:2806`).
- Programmatic sync path is separate (`src/app_impl.rs:3295`) and can lag behind mutations.

Impact:
- Easy to introduce drift between displayed input value, active view query, and search cache keys.
- Adds repetitive branching and reset logic for every view that uses search.

Recommendation:
- Introduce a single `ActiveQueryState` owned by app state:
  - `MainMenu { live, computed }`
  - `ViewScoped { view: BuiltinViewKind, query }`
- Provide helpers `read_active_query()`, `set_active_query()`, `clear_active_query()` used by all views.
- Make input component value fully derived from active query except during transient IME/edit operations.

### 3) Selection validation mutates unrelated fallback state (high)
Evidence:
- `validate_selection_bounds()` clears fallback state even when called for generic selection correction (`src/app_navigation.rs:508`, `src/app_navigation.rs:509`, `src/app_navigation.rs:510`, and again `src/app_navigation.rs:519`, `src/app_navigation.rs:520`).
- `set_filter_text_immediate()` contains special-case code to rebuild fallback immediately because validation already cleared it (`src/app_impl.rs:3271`).

Impact:
- Hidden side effects: callers expecting only selection correction also lose fallback mode/cache.
- Increases ordering sensitivity and makes fallback behavior fragile.

Recommendation:
- Split responsibilities:
  - `validate_selection_bounds()` should only touch selection/hover/scroll fields.
  - Add explicit fallback state transition methods (`enter_fallback_mode`, `exit_fallback_mode`, `recompute_fallback_mode`).
- Ensure fallback recompute is triggered at query transition points only.

### 4) State reset flows are duplicated and diverging (high)
Evidence:
- Reset-like logic appears in multiple methods with overlapping but non-identical behavior:
  - `go_back_or_close()` main-menu return path (`src/app_impl.rs:6557` onward),
  - `reset_to_script_list()` full reset (`src/app_impl.rs:6901` onward),
  - `ensure_selection_at_first_item()` partial reset (`src/app_impl.rs:7038` onward),
  - show-window helper path also mutates selection/focus (`src/main.rs:473`, `src/main.rs:481`, `src/main.rs:482`).

Impact:
- Behavior differences are path-dependent (placeholder reset, channels cleared, focus restore, cache invalidation, scroll reset).
- Regressions are likely when one reset path is updated but others are not.

Recommendation:
- Create one transition reducer for returning to script list:
  - `transition_to_script_list(reason, options)` where options are explicit (full teardown vs preserve session, force scroll top, etc.).
- Make all current call sites invoke this reducer.
- Log `from_view`, `to_view`, and transition flags with `correlation_id`.

### 5) Actions dialog close/open paths still mix coordinator and manual focus (medium)
Evidence:
- Some close paths call `pop_focus_overlay()` then direct focus helpers (`src/app_impl.rs:3612`, `src/app_impl.rs:3615`, `src/app_impl.rs:3748`, `src/app_impl.rs:3749`, `src/app_impl.rs:3921`, `src/app_impl.rs:3922`).
- Other paths centralize closure and immediate pending focus application (`src/app_impl.rs:4675` through `src/app_impl.rs:4708`).

Impact:
- Inconsistent focus restore timing and cursor ownership after closing actions.
- Additional opportunities for coordinator state to drift from actual focused element.

Recommendation:
- Route all actions open/close paths through one shared close/open helper.
- Close helper should: clear dialog state, close window, pop overlay, apply restored focus exactly once.

### 6) `GetState` snapshot can diverge from on-screen list state (medium)
Evidence:
- `PromptMessage::GetState` ScriptList branch uses `filtered_results()` (`src/prompt_handler.rs:679`, `src/prompt_handler.rs:681`).
- `filtered_results()` uses `fuzzy_search_unified` on scripts/scriptlets only (`src/app_impl.rs:2068`), while displayed main list uses grouped results with built-ins/apps and `computed_filter_text` (`src/app_impl.rs:2137`, `src/app_impl.rs:2144`, `src/app_impl.rs:2190`).

Impact:
- API/automation consumers can receive a state snapshot that does not match what users currently see.
- Selected item/value may be reported from a different result set than the rendered list.

Recommendation:
- Add a dedicated `collect_script_list_state_snapshot()` that reads from the same grouped cache path as render.
- Use one snapshot object for GetState and future `kit://state` integration.

### 7) External “app state” surfaces are disconnected from runtime state (medium)
Evidence:
- `kit/state` tool returns `AppState::default()` unless caller injects state (`src/mcp_kit_tools.rs:101`, `src/mcp_kit_tools.rs:103`).
- HTTP MCP server always passes `None` for app_state context (`src/mcp_server.rs:425`).
- `kit://state` resource falls back to default when app_state absent (`src/mcp_resources.rs:178`, `src/mcp_resources.rs:179`).

Impact:
- MCP clients receive stale/default state despite resource/tool claiming current app state.
- Introduces parallel app-state DTOs (`AppState` vs `AppStateResource`) with no canonical producer.

Recommendation:
- Define one runtime state snapshot DTO and adapter methods for tool/resource responses.
- Feed MCP handler with live snapshot captured from `ScriptListApp`.
- Keep default fallback only for startup/no-app contexts and mark clearly in response.

## Simplification Plan (Suggested Order)

1. **Focus unification**
- Add `FocusState` adapter around `FocusCoordinator`.
- Replace direct writes to `focused_input`/`pending_focus` at prompt transitions and show/reset paths.
- Convert legacy fields to derived compatibility layer.

2. **Query state unification**
- Introduce `ActiveQueryState`.
- Refactor filter handlers and view clear handlers to use shared query helpers.

3. **Selection/fallback separation**
- Make `validate_selection_bounds()` side-effect free beyond selection state.
- Add explicit fallback transition methods.

4. **Single reset reducer**
- Implement `transition_to_script_list(reason, options)`.
- Migrate all current reset/go-back/show-window paths.

5. **State snapshot unification**
- Build one `AppSnapshot` from runtime state.
- Reuse for `GetState`, MCP tool/resource state endpoints.

## Test Gaps To Add
- Focus restore invariants:
  - actions open/close from ScriptList, ArgPrompt, ChatPrompt restores expected target/cursor owner.
- Fallback invariants:
  - selection validation never clears fallback unless explicit exit path called.
- Snapshot consistency:
  - `GetState` selected/visible counts match grouped list actually rendered.
- Transition consistency:
  - all return-to-script-list paths yield same canonical state (filter, selection, focus, placeholder, overlays).

## Notes
- This report is analysis-only and intentionally does not include code changes.
