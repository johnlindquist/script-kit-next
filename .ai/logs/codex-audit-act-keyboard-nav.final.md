# Keyboard Navigation Audit: Actions UI

## Scope
- `src/actions/dialog/part_01.rs`
- `src/actions/window/part_01.rs`
- `src/actions/window/part_02.rs`

## Checklist Findings

1. Up/down navigation with both key variants (`up`/`arrowup`, `down`/`arrowdown`)
- **Status: PASS**
- `actions_window_key_intent()` uses `is_key_up()` / `is_key_down()` in `src/actions/window/part_01.rs:108` and `src/actions/window/part_01.rs:111`.
- The shared key helpers accept both variants in `src/ui_foundation/part_001.rs:12` and `src/ui_foundation/part_001.rs:19`.
- Navigation updates selection via `move_up()` / `move_down()` and skips section headers (`src/actions/window/part_01.rs:233`, `src/actions/window/part_01.rs:238`; selection logic in `src/actions/dialog/part_02/part_03.rs:8` and `src/actions/dialog/part_02/part_03.rs:30`).

2. Enter/escape with both variants
- **Status: PASS**
- `actions_window_key_intent()` uses `is_key_enter()` / `is_key_escape()` (`src/actions/window/part_01.rs:126`, `src/actions/window/part_01.rs:129`).
- The helpers map `enter` + `return`, and `escape` + `esc` (`src/ui_foundation/part_001.rs:40`, `src/ui_foundation/part_001.rs:47`).
- Alias behavior is directly unit-tested (`src/actions/window/part_02.rs:7`).

3. Tab navigation (if applicable)
- **Status: FAIL / Not implemented**
- No `tab` / `shift+tab` branch exists in `actions_window_key_intent()` (`src/actions/window/part_01.rs:104`).
- Dialog render explicitly has no local `on_key_down` and relies on parent routing (`src/actions/dialog/part_04/body_part_03.rs:217`).
- Result: Tab is ignored in this UI path.

4. Page up/down
- **Status: PASS**
- `pageup` and `pagedown` are mapped (`src/actions/window/part_01.rs:120`, `src/actions/window/part_01.rs:123`).
- Behavior jumps by `ACTIONS_WINDOW_PAGE_JUMP` and re-coerces to selectable rows (`src/actions/window/part_01.rs:58`, `src/actions/window/part_01.rs:259`, `src/actions/window/part_01.rs:276`).
- Covered by unit test (`src/actions/window/part_02.rs:27`, `src/actions/window/part_02.rs:31`).

5. Home/End keys
- **Status: PASS**
- `home` and `end` intents are mapped (`src/actions/window/part_01.rs:114`, `src/actions/window/part_01.rs:117`).
- Handlers move to first/last selectable row and reveal selection (`src/actions/window/part_01.rs:241`, `src/actions/window/part_01.rs:250`).
- Covered by unit test (`src/actions/window/part_02.rs:19`, `src/actions/window/part_02.rs:23`).

6. Search input focus management
- **Status: PARTIAL (implicit, not explicit focus-managed input)**
- Search field is rendered as styled text/cursor visuals, not a native/focusable text input (`src/actions/dialog/part_04/body_part_01.rs:12`, `src/actions/dialog/part_04/body_part_01.rs:69`, `src/actions/dialog/part_04/body_part_03.rs:157`).
- Typed characters/backspace are captured at window key handler level and forwarded (`src/actions/window/part_01.rs:340`, `src/actions/window/part_01.rs:329`) into dialog search state (`src/actions/dialog/part_02/part_02.rs:402`, `src/actions/dialog/part_02/part_02.rs:409`).
- No separate focus target for search vs. list; effectively "always type-to-filter" while action UI is active.

7. Focus trapping within dialog
- **Status: FAIL / Not implemented**
- Actions popup window is intentionally opened with `focus: false` so parent window keeps focus (`src/actions/window/part_02.rs:228`, `src/actions/window/part_02.rs:230`).
- Dialog can skip focus tracking (`src/actions/dialog/part_02/part_01.rs:302`) and only conditionally calls `.track_focus(...)` (`src/actions/dialog/part_04/body_part_03.rs:213`).
- There is no focus-cycle or trap logic for keyboard traversal inside the dialog.

## Test Coverage Notes
- Present: alias/jump key intent test for `return`, `esc`, `home`, `end`, `pageup`, `pagedown` in `src/actions/window/part_02.rs:7`.
- Missing in scoped files: explicit tests for Tab handling/focus trap and for up/down alias handling at the `actions_window_key_intent()` level.

## Overall Risk Summary
- Core directional and activation navigation is implemented and mostly tested.
- Accessibility/keyboard UX gaps remain around Tab behavior and true focus containment in dialog mode.
