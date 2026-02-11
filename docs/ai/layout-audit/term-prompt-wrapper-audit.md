# Term Prompt Wrapper Audit

## Scope
- File audited: `src/render_prompts/term.rs`
- Comparison targets: `src/render_prompts/editor.rs`, `src/render_prompts/other.rs`, `src/render_prompts/div.rs`, `src/components/prompt_layout_shell.rs`
- Focus: terminal prompt layout (edge-to-edge vs rounded, explicit height, footer behavior, actions overlay/backdrop affordance) and key-handling decisions that change perceived navigation.

## Current Wrapper Behavior (`render_term_prompt`)
- Uses explicit root height `window_resize::layout::MAX_HEIGHT` instead of `h_full` (`src/render_prompts/term.rs:132`, `src/render_prompts/term.rs:289`).
- Uses an edge-to-edge shell with no radius (`src/render_prompts/term.rs:281`) and no shared `prompt_shell_container` helper.
- Uses `capture_key_down` at wrapper level (`src/render_prompts/term.rs:292`) instead of the `on_key_down` pattern used by editor/other wrappers.
- Footer is always present and wired with primary `Close` (`⌘W`) plus secondary actions button (`src/render_prompts/term.rs:269`, `src/render_prompts/term.rs:272`, `src/render_prompts/term.rs:297`).
- Actions dialog overlay uses a full-frame click catcher (`#term-actions-backdrop`) but does not expose pointer affordance (`src/render_prompts/term.rs:340`, `src/render_prompts/term.rs:343`).

## Navigation and Key-Routing Behavior
- SDK term prompt is non-dismissable with `Escape` unless the actions popup is open (`src/render_prompts/term.rs:154`, `src/render_prompts/term.rs:185`).
- Quick terminal keeps utility behavior: `Escape` goes back/close and `⌘W` closes (`src/render_prompts/term.rs:171`, `src/render_prompts/term.rs:178`).
- `⌘K` always toggles an actions surface, with mode selected by SDK-actions availability (`SdkActions` vs `TerminalCommands`) (`src/render_prompts/term.rs:108`, `src/render_prompts/term.rs:196`, `src/render_prompts/term.rs:204`).
- While actions popup is open, terminal input is suppressed (`term.suppress_keys = show_actions`) and keys are routed through shared actions-dialog handling (`src/render_prompts/term.rs:117`, `src/render_prompts/term.rs:210`).
- SDK action shortcuts are still matched in term wrapper when popup is closed (`src/render_prompts/term.rs:246`), unlike editor which explicitly preserves editor-reserved shortcuts (`src/render_prompts/editor.rs:33`, `src/render_prompts/editor.rs:173`).

## Inconsistencies vs Editor/Other Prompt Wrappers
1. Backdrop affordance parity
- Editor and Div wrappers mark actions backdrops as clickable with `.cursor_pointer()` (`src/render_prompts/editor.rs:342`, `src/render_prompts/div.rs:229`).
- Term wrapper backdrop lacks this cue (`src/render_prompts/term.rs:343`).
- Perceived effect: actions popup feels less dismissable/more opaque to mouse users.

2. Event interception model
- Term wrapper uses `capture_key_down` (`src/render_prompts/term.rs:292`), while editor/other wrappers use `on_key_down` (`src/render_prompts/editor.rs:255`, `src/render_prompts/other.rs:120`).
- Perceived effect: terminal wrapper is structurally biased toward parent-first interception; this is intentional for terminal navigation, but it is a divergence that should be explicit in contract docs.

3. Footer secondary-button semantics
- Term footer comment says secondary is shown "when actions are available," but code hardcodes `.show_secondary(true)` (`src/render_prompts/term.rs:268`, `src/render_prompts/term.rs:272`).
- Editor/footer helpers tie secondary visibility to feature availability (`src/render_prompts/editor.rs:22`, `src/render_prompts/editor.rs:277`).
- Perceived effect: contract ambiguity for designers and implementers, even if runtime behavior is currently acceptable due to terminal-command fallback.

4. Shape and shell strategy
- Other wrappers consistently go through `prompt_shell_container(...).on_key_down(...)` and rounded frame defaults (`src/components/prompt_layout_shell.rs:79`, `src/render_prompts/other.rs:119`).
- Term intentionally bypasses shared shell and remains edge-to-edge (`src/render_prompts/term.rs:281`).
- Perceived effect: terminal feels like a full utility surface rather than a modal card; this should be preserved intentionally, not treated as accidental drift.

## Canonical Terminal Layout Contract (Proposed)
1. Root frame
- Terminal wrapper is a `relative + flex_col + w_full + explicit_height + overflow_hidden` frame.
- Keep it edge-to-edge (no rounded corners) to preserve terminal-native continuity.
- Height contract: use explicit terminal height token (`MAX_HEIGHT` for terminal/scratch-like flows), not `h_full`.

2. Content and footer
- Terminal viewport is always `flex_1 + min_h(0) + overflow_hidden`.
- Footer is always visible as the bottom child.
- Primary action is `Close (⌘W)`.
- Secondary action is always present, but semantic source is explicit:
  - `SdkActions` when SDK actions exist.
  - `TerminalCommands` when SDK actions are absent.

3. Actions overlay and affordance
- Overlay must include full-frame backdrop click target with stable ID.
- Backdrop must advertise clickability (`cursor_pointer`) for parity with editor/div wrappers.
- Dialog anchor uses shared tokenized offsets (`prompt_actions_dialog_offsets`).

4. Key-routing and perceived navigation
- Wrapper-level interception remains parent-first for terminal (`capture_key_down`) so close/actions/navigation shortcuts are predictable.
- Non-dismissable SDK terminal behavior: `Escape` is swallowed when popup is closed.
- Quick terminal utility behavior: `Escape` returns/back-closes.
- When actions popup is open, terminal input is suppressed and actions dialog receives navigation keys.
- Shortcut routing should document reserved terminal combos to avoid accidental action-shortcut hijacking of expected terminal input.

## Risk / Gap Notes
- No explicit source test currently guards term backdrop click-affordance parity (editor/div already have this kind of test).
- Footer visibility comment and implementation are currently mismatched and may cause future regressions if someone "fixes" the wrong side.
- Reserved-shortcut policy for terminal is implicit; if action maps grow, perceived terminal navigation could regress.
