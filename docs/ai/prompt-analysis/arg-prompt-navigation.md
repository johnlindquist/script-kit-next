# Arg Prompt Navigation & Cognitive Load Analysis

## Scope
- `src/render_prompts/arg/render.rs`
- `src/components/prompt_input.rs`
- `src/components/prompt_footer.rs`
- `src/components/prompt_layout_shell.rs`

## Current Interaction Model

### Visual regions today
- Header: custom text input rendering with manual cursor/selection/placeholder (`src/render_prompts/arg/render.rs:334`).
- Content: optional list region only when `has_choices`; empty-state text appears inside that region (`src/render_prompts/arg/render.rs:393`, `src/render_prompts/arg/render.rs:266`).
- Footer: shared `PromptFooter` with helper text + option count + actions button (`src/render_prompts/arg/render.rs:412`).
- Overlay: absolute Cmd+K actions layer with full-size clickable backdrop (`src/render_prompts/arg/render.rs:454`).

### Keyboard routing order
`render_arg_prompt` runs one monolithic handler with this precedence (`src/render_prompts/arg/render.rs:110`):
1. Global shortcuts (except when actions popup is open).
2. Cmd+K toggle.
3. Actions dialog routing (modal handling).
4. SDK action shortcuts.
5. Up/Down list navigation.
6. Tab completion.
7. Enter submit.
8. All remaining keys to `TextInputState` editing.

## Navigation + Cognitive Load Issues

### 1. Focus intent is implicit, not visible
- Input cursor visibility depends on `focused_input == FocusedInput::ArgPrompt`, while list selection is independent state (`src/render_prompts/arg/render.rs:9`, `src/render_prompts/arg/render.rs:260`).
- Result: users see one surface but two concurrent “targets” (typed text + highlighted list row) without an explicit mode indicator.

### 2. Selection vs typed value is surprising on submit
- Submit prefers selected choice value whenever filtered choices are present (`src/render_prompts/arg/helpers.rs:179`).
- Typed text is only submitted when no choice is selected/matching.
- Result: user intent is ambiguous when the input text is edited but a list item remains selected.

### 3. Feedback is split across three locations
- No-match message in content region (`src/render_prompts/arg/render.rs:273`).
- Status helper in footer (`src/render_prompts/arg/helpers.rs:100`).
- Empty-submit error as transient HUD (`src/render_prompts/arg/helpers.rs:194`).
- Result: users must scan header/content/footer/toast to understand one interaction step.

### 4. Cmd+K overlay is modal in behavior, but weakly modal in presentation
- Keys are routed modally through actions dialog routing (`src/render_prompts/arg/render.rs:145`), but the overlay is only a transparent backdrop + floating panel (`src/render_prompts/arg/render.rs:482`).
- Result: underlying input/list/footer remain visually “active,” increasing mode confusion.

### 5. Hidden affordances increase memory load
- Tab completion exists (`src/render_prompts/arg/render.rs:211`) but helper text only emphasizes arrows/Enter (`src/render_prompts/arg/helpers.rs:102`).
- Result: interaction model depends on undocumented shortcuts.

### 6. Arg prompt diverges from shared shell/input contracts
- Arg uses bespoke frame + header rendering instead of `prompt_shell_container`/`prompt_shell_content` and `PromptInput` (`src/render_prompts/arg/render.rs:319`, `src/components/prompt_layout_shell.rs:79`, `src/components/prompt_input.rs:401`).
- Result: header/content/footer behavior is less predictable across prompt types.

## Refactoring Recommendations

### 1) Enforce an explicit header/content/footer/overlay scaffold for ArgPrompt
Adopt a fixed composition contract:
- Header slot: input + inline state.
- Content slot: list or empty-state panel.
- Footer slot: actions + concise hint.
- Overlay slot: modal layers rendered last.

Implementation direction:
- Use `prompt_shell_container(...)` as root.
- Use `prompt_shell_content(...)` for the content fill area.
- Keep `PromptFooter` in a stable footer slot.
- Keep actions dialog in a dedicated overlay slot.

### 2) Introduce a small ArgPrompt view-model with explicit interaction mode
Define derived state before rendering:
- `ArgInteractionMode`: `Typing`, `ChoiceNavigation`, `ActionsOverlay`.
- `ArgContentState`: `ChoiceList`, `NoMatches`, `FreeTextOnly`.
- `ArgSubmitBehavior`: `SubmitSelectedChoice` vs `SubmitTypedValue`.

This removes implicit coupling between `arg_input`, `arg_selected_index`, `show_actions_popup`, and footer text.

### 3) Split key handling by mode
Replace the single large key router with mode-specific handlers:
- `handle_arg_overlay_keys(...)`
- `handle_arg_choice_nav_keys(...)`
- `handle_arg_text_keys(...)`

Benefits:
- Fewer hidden precedence rules.
- Easier to reason about Enter/Tab behavior.
- Lower risk of regressions when adding shortcuts.

### 4) Consolidate user guidance into one primary status surface
Make one location authoritative for immediate state:
- Use either header inline status or content empty panel for contextual guidance.
- Keep footer hints short and stable (shortcut legend), not state narration.
- Reserve HUD only for cross-cutting notifications.

### 5) Clarify submit semantics
Pick one explicit rule set and communicate it in UI text:
- Option A: `Enter` always submits selected choice, `Cmd+Enter` forces typed value.
- Option B: `Enter` submits typed value when text changed since last explicit selection.

Whichever rule is chosen, encode it in a single helper source to avoid drift.

### 6) Make overlay mode visibly modal
When actions overlay is open:
- Dim or tint backdrop (not just click-capture).
- Temporarily suppress/disable footer actions beneath overlay.
- Show a short “Actions” mode label near header or overlay anchor.

## Suggested rollout slices
1. Extract Arg scaffold into explicit header/content/footer/overlay builders.
2. Introduce derived mode/content enums and migrate footer helper decisions to derived state.
3. Split key routing by mode without changing external behavior.
4. Unify guidance surfaces and submit semantics copy.
5. Add modal presentation tweaks for Cmd+K overlay.
