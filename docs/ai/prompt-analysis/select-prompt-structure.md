# SelectPrompt Structure Analysis

## Scope

- `src/prompts/select/render.rs`
- `src/render_prompts/other.rs`
- `src/components/prompt_container.rs`
- `src/components/prompt_layout_shell.rs`
- `src/components/focusable_prompt_wrapper.rs`

## Current Structure (SelectPrompt)

1. Wrapper shell is created in `render_select_prompt` using:
   - `prompt_shell_container(...)`
   - `prompt_shell_content(entity)`
   - parent `.on_key_down(other_prompt_shell_handle_key_default)`
   - Reference: `src/render_prompts/other.rs:107`
2. Inside the entity, `SelectPrompt::render` builds a second full frame manually:
   - custom search/header row (`input_container`)
   - custom list body (`choices_container`)
   - no explicit footer slot
   - Reference: `src/prompts/select/render.rs:149`
3. Focus and key routing are attached by `FocusablePrompt::build(...)` on the inner SelectPrompt root:
   - Escape intercepted and consumed in app-level handler (calls `submit_cancel`)
   - Cmd+W / Cmd+K are not consumed at this level
   - navigation + filtering keys handled in entity-level handler
   - Reference: `src/prompts/select/render.rs:351`
4. `FocusablePrompt` interception model is two-level and expects explicit ownership decisions:
   - intercepted: `Escape`, `CmdW`, `CmdK`
   - consume = stop propagation, otherwise bubble/fall through
   - Reference: `src/components/focusable_prompt_wrapper.rs:23`

## Shared Building Blocks Available

1. `PromptContainer` already models explicit slots and ordering:
   - `header` -> optional divider -> `content` (fill or intrinsic) -> `footer`/`hint`
   - Reference: `src/components/prompt_container.rs:157`
2. `prompt_layout_shell` already standardizes the outer frame contract:
   - relative shell root, full size, overflow clipping, rounded corners
   - Reference: `src/components/prompt_layout_shell.rs:72`
3. Other prompt implementations (notably PathPrompt) already use a slot model:
   - `PromptHeader` for top input/actions
   - `PromptContainer` with hint/footer area
   - `FocusablePrompt` for local key ownership
   - Reference: `src/prompts/path/render.rs:90`

## Inconsistencies Identified

1. Header ownership is inconsistent.
   - Select defines its own header row inline in `select/render.rs`.
   - Path uses shared `PromptHeader` inside `PromptContainer`.
   - Result: duplicate semantics for search input, placeholder, action affordances, and spacing.

2. Footer/hint placement is inconsistent.
   - Select has no footer slot and no helper/info channel.
   - Multi-select status (`"N selected"`) is embedded in header.
   - Prompts like Path/Env surface helper text and actions in a dedicated footer/hint area.

3. Key ownership is split across two layers in a way that is hard to reason about.
   - Outer wrapper (`other.rs`) handles global shortcuts and Cmd+K action toggle.
   - Inner SelectPrompt consumes Escape but defers Cmd+K/Cmd+W.
   - Effective behavior depends on bubbling, not a single clear owner.

4. Focus indication is ambiguous.
   - Row background is shared for both focused and selected states (`resolve_row_bg_hex`), so keyboard focus is visually conflated with selection.
   - Search/header focus state is not surfaced with a shared focused treatment (compared to `PromptHeader` focused cursor and input styling).

5. Shell and container responsibilities are duplicated.
   - Outer shell gives rounded/overflow framing.
   - Inner SelectPrompt creates another rounded bordered container and custom chrome.
   - This duplicates frame/chrome concerns that `PromptContainer` is intended to centralize.

## Refactoring Recommendations

### Target Contract

Adopt a predictable prompt schema for SelectPrompt:

1. `Outer shell` (wrapper level):
   - Keep `prompt_shell_container + prompt_shell_content`.
   - Keep wrapper-level global shortcut interception only.
2. `Header` (entity level, slot-based):
   - Render via `PromptHeader` (or a thin shared search-header variant if Select needs reduced controls).
3. `Content` (entity level):
   - Render choice list into `PromptContainer::content(...)`.
4. `Footer` (entity level):
   - Render helper/info/actions via `PromptFooter` (or at minimum `PromptContainer::hint(...)`).

### Concrete SelectPrompt Migration

1. Replace custom `input_container` in `src/prompts/select/render.rs` with a `PromptHeader` config:
   - `filter_text` <- `self.filter_text`
   - `placeholder` <- existing placeholder logic
   - `primary_button_label` <- submit action label (`Select` / `Continue`)
   - `show_actions_button` <- true when SDK actions exist (if wired at entity level)
2. Wrap list body in `PromptContainer`:
   - `.header(prompt_header)`
   - `.content(choices_content)`
   - `.footer(prompt_footer)` or `.hint(...)`
3. Move `"N selected"` and filter/result summary out of header into footer `info_label` or helper text.
4. Separate focused vs selected row treatment:
   - keep selected fill state
   - add explicit focus signal (accent bar, ring, or distinct alpha) so keyboard focus remains visible when selection differs.
5. Normalize key ownership:
   - Prefer handling prompt-specific intercepted keys (`Escape`, `CmdK`) inside SelectPrompt `FocusablePrompt` app handler.
   - Keep wrapper handler restricted to truly global behavior (e.g. Cmd+W).
   - This matches the two-level contract in `FocusablePrompt` and reduces hidden bubbling dependencies.

## Suggested Incremental Plan

1. Phase 1: Structural alignment (no behavior change)
   - Introduce `PromptContainer` usage in SelectPrompt with existing custom header/list content.
   - Keep current key routing.
2. Phase 2: Header/footer normalization
   - Switch header to `PromptHeader`.
   - Add footer/hint via `PromptFooter`/`PromptContainer::hint`.
3. Phase 3: Key-routing cleanup
   - Move Cmd+K ownership to SelectPrompt app-level handler in `FocusablePrompt::build`.
   - Simplify wrapper handler for select path.
4. Phase 4: Focus affordance polish
   - Distinguish keyboard focus from selected state in row visuals.

## Risks / Watchouts

1. Actions popup ownership can regress if Cmd+K handling moves without updating host routing expectations in `ScriptListApp`.
2. Footer introduction changes vertical space allocation; list viewport height and scroll behavior should be verified.
3. Switching to shared header may alter placeholder/cursor rendering details users currently rely on.
4. Focus visuals must preserve accessibility contrast in both light and dark themes.

## Validation Suggestions for Follow-up PRs

1. Add source-level structure tests (like `prompt_layout_shell_tests`) asserting SelectPrompt uses `PromptContainer` slots.
2. Add targeted key-routing tests for:
   - Escape cancels prompt
   - Cmd+K toggles actions when available
   - Cmd+W remains global
3. Add focused-vs-selected row rendering tests for `resolve_row_bg_hex` (or successor resolver) to prevent regressions.
