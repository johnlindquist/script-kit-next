<!-- markdownlint-disable MD013 -->

# Prompt Input Patterns Analysis

## Scope

- `src/components/prompt_input.rs`
- `src/render_prompts/arg/render.rs`
- `src/prompts/select/render.rs`
- `src/prompts/env/render.rs`

Goal: explain why users cannot reliably predict input behavior across prompt types, then define one mental model for cursor/focus, placeholder behavior, selection/clipboard support, and hint/error placement.

## Current Behavior Map

### 1) Shared `PromptInput` contract exists but is not the active renderer for these prompts

- `PromptInputConfig` declares a unified surface (`placeholder`, `show_path_prefix`, `enable_selection`, `enable_clipboard`, `cursor_visible`, `is_focused`) in `src/components/prompt_input.rs:94`.
- `PromptInput` renders a cursor/placeholder/text model in `src/components/prompt_input.rs:401`.
- `PromptInput` is used in shared header UI (`PromptHeader`) at `src/components/prompt_header/component.rs:56` and `src/components/prompt_header/component.rs:92`.
- Arg/Select/Env prompt renderers in scope each hand-roll their own input UI instead of using `PromptInput`.

### 2) Arg prompt (`render_prompts/arg/render.rs`) uses full text editing state, but custom visuals and list-first key routing

- Cursor/selection drawing is custom in `render_arg_input_text` (`src/render_prompts/arg/render.rs:2`).
- Empty placeholder uses manual negative-margin alignment (`src/render_prompts/arg/render.rs:381`).
- Cursor visibility is tied to `focused_input == FocusedInput::ArgPrompt` plus blink state (`src/render_prompts/arg/render.rs:361`).
- Up/Down are consumed for list navigation before input editing (`src/render_prompts/arg/render.rs:179`).
- Most remaining keys delegate to `TextInputState::handle_key` for selection/clipboard/navigation (`src/render_prompts/arg/render.rs:232`).
- Help text lives in footer status (`src/render_prompts/arg/render.rs:415`).

### 3) Select prompt (`prompts/select/render.rs`) uses a display-only filter row (no cursor/selection model)

- Filter row shows either placeholder or raw `filter_text` (`src/prompts/select/render.rs:143`) with no caret rendering.
- Keyboard handling is custom and minimal: Up/Down, Enter, Backspace, printable char append (`src/prompts/select/render.rs:364`).
- No delegation to `TextInputState`, so no cursor movement, range selection, word-jump, copy/cut/paste.
- `Cmd/Ctrl+A` means “select all filtered choices” (`src/prompts/select/render.rs:376`), not “select filter text”.
- Secondary hint appears as “N selected” inline in input row (`src/prompts/select/render.rs:174`).

### 4) Env prompt (`prompts/env/render.rs`) uses `TextInputState` behavior but custom, always-accent visual treatment

- Input shell is custom, icon-led, with accent border always visible (`src/prompts/env/render.rs:123`).
- Empty state always paints an accent cursor block (`src/prompts/env/render.rs:151`), not tied to focus/cursor blink.
- Non-empty text/cursor/selection drawing is delegated to local helper (`src/prompts/env/render.rs:173`) implemented in `src/prompts/env/prompt.rs:197`.
- Editing delegates to `TextInputState::handle_key` (`src/prompts/env/render.rs:338`) so clipboard/selection/navigation are available.
- Hints/errors are stacked in-body (storage hint, validation error, running status), while footer also says “Script running” (`src/prompts/env/render.rs:187`, `src/prompts/env/render.rs:190`, `src/prompts/env/render.rs:200`, `src/prompts/env/render.rs:288`).

## Why Users Cannot Predict Behavior

- Same concept, different primitives: users see “an input field” in Arg/Select/Env, but each field is powered by a different rendering and key model.
- Cursor policy changes by prompt type: Arg is focus + blink gated, Env empty state is always accent-caret, and Select has no visible caret.
- Placeholder alignment/composition drift: Arg uses negative-margin overlay, Env uses fixed left margin after icon, Select uses plain text substitution with search emoji.
- Text editing semantics are inconsistent: Arg/Env support `TextInputState`, while Select only supports append/backspace and uses `Cmd/Ctrl+A` for list selection.
- Focus target is ambiguous in list prompts: Arg and Select both have “input + list”, but the user has no single rule for arrow keys (text edit vs list movement).
- Hint/error placement is non-uniform: Arg uses footer helper text, Select uses inline row hint, and Env duplicates status across body and footer.

## Recommended Single Input Mental Model

### A. One input primitive for visuals

- `PromptInput` is the only component that draws cursor + placeholder + text baseline for prompt inputs.
- Prompt-specific UIs can wrap it (icons, chips, counters), but do not re-implement caret/placeholder geometry.

### B. One focus model

- Exactly one active text input per prompt view.
- Caret is visible only when that input is focused and blink-visible.
- Unfocused input keeps layout slot but hides caret.
- Prompts with list + input still allow list focus state, but keyboard text-edit commands always target the active text input first.

### C. One cursor and editing rule set

- Use `TextInputState` anywhere the user can type filter/input text.
- Baseline guarantees across Arg/Select/Env: Left/Right/Home/End/Option+Arrow behave as text navigation.
- Baseline guarantees across Arg/Select/Env: Shift+navigation creates selection.
- Baseline guarantees across Arg/Select/Env: Cmd/Ctrl+C, X, V, A behave as text clipboard/select-all.
- List actions should use explicit bindings that do not shadow baseline text editing (for example, reserve row-selection toggles for dedicated shortcuts, not `Cmd/Ctrl+A`).

### D. One placeholder rule

- Placeholder is purely an empty-input state and never implies a different keyboard mode.
- Placeholder, typed text, and caret share a stable text origin (no layout jump).

### E. One hints/errors placement rule

- Input-local validation errors live directly below the input field.
- Non-error guidance (shortcuts/status/help) lives in footer helper text.
- Avoid duplicate status in body and footer at the same time.

## Prompt-Specific Adaptation Under This Model

- Arg prompt: keep list navigation and tab-completion semantics.
- Arg prompt: move visual input rendering to `PromptInput`.
- Arg prompt: keep helper status in footer.
- Select prompt: replace string-only filter editing with `TextInputState`.
- Select prompt: keep multi-select list logic, but stop overloading text-edit shortcuts.
- Select prompt: render filter input through `PromptInput` with optional leading search icon.
- Env prompt: keep secret masking and storage metadata.
- Env prompt: gate caret visibility by focus/blink (not always-on accent block).
- Env prompt: consolidate status messaging to one location and keep validation directly under input.

## Acceptance Criteria For “Predictable Input”

1. A user can switch between Arg, Select, and Env and keep the same expectations for caret visibility, text selection, and clipboard shortcuts.
2. Placeholder behavior does not change input mode or key map.
3. Error placement is always input-local.
4. Global or list-specific shortcuts do not steal baseline text-edit shortcuts without explicit visual affordance.
5. Input visuals in all three prompts are traceable to `PromptInput` rather than prompt-local caret drawing.
