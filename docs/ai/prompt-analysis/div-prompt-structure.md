<!-- markdownlint-disable MD013 MD032 -->

# Div Prompt Structure Analysis

Snapshot date: 2026-02-11.

## Scope

- `src/render_prompts/div.rs`
- `src/prompts/div/render.rs`
- `src/components/prompt_footer.rs`
- `src/components/prompt_layout_shell.rs`
- `src/components/focusable_prompt_wrapper.rs`

No Rust code changes were made. This is documentation-only analysis.

## End-to-End Composition (Current)

```text
ScriptListApp::render_div_prompt (wrapper)
  -> prompt_shell_container(radius, vibrancy)
  -> wrapper key handler (global shortcuts, Cmd+K, actions dialog routing)
  -> wrapper header lane ("Script Output" + "Enter to continue")
  -> wrapper divider lane
  -> prompt_shell_content(entity)
  -> PromptFooter("Continue", "Actions", helper/info text)
  -> actions overlay (absolute backdrop + absolute dialog)

DivPrompt::render (entity)
  -> parse HTML
  -> resolve render colors + background + container padding
  -> build scroll owner (id + overflow_y_scroll + track_scroll)
  -> FocusablePrompt wrapper for entity-level key handling
      - Escape intercepted and submits
      - Enter/Return/Escape submit in entity key handler
  -> link click handling (submit:value, http(s), file)
```

Key references:
- Wrapper shell/chrome: `src/render_prompts/div.rs:134`, `src/render_prompts/div.rs:140`, `src/render_prompts/div.rs:181`, `src/render_prompts/div.rs:200`
- Shared shell slot API: `src/components/prompt_layout_shell.rs:79`, `src/components/prompt_layout_shell.rs:86`
- Entity scroll and key handling: `src/prompts/div/render.rs:113`, `src/prompts/div/render.rs:132`, `src/prompts/div/render.rs:138`, `src/prompts/div/render.rs:157`
- Focus wrapper interception model: `src/components/focusable_prompt_wrapper.rs:31`, `src/components/focusable_prompt_wrapper.rs:92`

## Duplication And Unclear Ownership

### 1) Header/hint guidance is split across header lane and footer lane

Evidence:
- Wrapper header hardcodes title + hint: `"Script Output"` and `"Enter to continue"` in `src/render_prompts/div.rs:161`, `src/render_prompts/div.rs:168`.
- Footer helper repeats guidance with running-status copy: `running_status_text("review output and press Enter")` in `src/render_prompts/div.rs:125`.
- Footer text is rendered from `PromptFooterConfig.helper_text` in `src/components/prompt_footer.rs:531`.

Why this is unclear:
- Two different chrome lanes describe the same action model (continue via Enter).
- There is no single source of truth for prompt-level guidance copy.
- Div wrapper depends on Arg helper functions for status/footer construction (`src/render_prompts/arg/helpers.rs:10`, `src/render_prompts/arg/helpers.rs:23`), which makes ownership implicit.

### 2) Scroll area ownership is mostly correct but still mentally split across layers

Evidence:
- Wrapper provides a fill slot with `flex_1 + min_h(0) + overflow_hidden` via `prompt_shell_content(...)` (`src/components/prompt_layout_shell.rs:63`, `src/components/prompt_layout_shell.rs:86`) and uses it in Div wrapper (`src/render_prompts/div.rs:177`).
- Entity also establishes full-height container + scroll owner (`src/prompts/div/render.rs:120`, `src/prompts/div/render.rs:113`).

Why this is unclear:
- Wrapper owns the body lane frame while entity owns scroll behavior and body padding, so both layers are shaping the same visual region.
- It is not obvious from API names whether spacing and scroll are expected to be wrapper-owned or entity-owned for content prompts.

### 3) Submit behavior has multiple paths with different semantics

Evidence:
- Footer primary click submits directly from app wrapper with `None` payload (`src/render_prompts/div.rs:186`).
- Entity Enter/Escape keyboard submit path uses `this.submit()` -> `on_submit(id, None)` (`src/prompts/div/render.rs:144`, `src/prompts/div/prompt.rs:107`).
- Link-driven submit path can send `Some(value)` via `submit:value` (`src/prompts/div/prompt.rs:121`, `src/prompts/div/prompt.rs:112`).

Why this is unclear:
- Same prompt can submit either `None` or `Some(value)` depending on interaction path.
- Footer primary bypasses entity logic, so submit semantics are split between wrapper and entity.
- The behavior is valid, but the model is implicit and easy to regress.

### 4) Footer/actions overlay composition is wrapper-owned but coupled to shared Arg helpers

Evidence:
- Div wrapper owns footer/action UI: `PromptFooter` and actions overlay in `src/render_prompts/div.rs:181`, `src/render_prompts/div.rs:220`.
- Overlay anchoring uses `prompt_actions_dialog_offsets(...)` (`src/render_prompts/div.rs:26`), defined in Arg helper file (`src/render_prompts/arg/helpers.rs:2`).
- Footer config/status string construction also comes from Arg helper file (`src/render_prompts/arg/helpers.rs:23`).

Why this is unclear:
- Div prompt chrome behavior depends on helper names/concepts that live under Arg prompt implementation.
- The coupling is structural (same include scope), but not semantically discoverable.

### 5) Escape handling is spread across three layers

Evidence:
- Wrapper layer handles global shortcuts when actions popup is closed (`src/render_prompts/div.rs:48`).
- Wrapper routes keys to actions dialog when popup is open (`src/render_prompts/div.rs:70`).
- Entity `FocusablePrompt` app-level handler treats plain Escape as submit (`src/prompts/div/render.rs:139`).
- Entity key handler also treats Escape as submit because `is_div_submit_key` includes escape (`src/prompts/div/types.rs:105`, `src/prompts/div/render.rs:157`).
- Shared focus wrapper intercepts Escape before entity key handler and can pass/fall through by callback result (`src/components/focusable_prompt_wrapper.rs:35`, `src/components/focusable_prompt_wrapper.rs:121`).

Why this is unclear:
- Escape can mean modal close, global dismiss, or submit-continue depending on prompt state and layer.
- The ordering works today, but the intent is not encoded as a single policy.

## Recommended Mental Model For Read-Only / Content Prompts

### 1) Distinguish prompt intent from prompt chrome

Use two explicit content-prompt intents:
- `ContentAckPrompt`: read-only content where primary action is acknowledge/continue.
- `ContentInspectPrompt`: read-only content where Escape means dismiss/cancel and submit is optional.

Div currently behaves like `ContentAckPrompt`.

### 2) Use explicit chrome lanes with single ownership

- `Shell Lane (wrapper)` owns window frame, title/subtitle, footer, and overlay mounting.
- `Body Lane (entity)` owns content rendering, scroll owner, and link interaction.
- `Modal Lane (wrapper)` owns actions popup lifecycle and Escape-to-close while open.

This keeps the entity free of shell chrome decisions.

### 3) Standardize key-routing ownership by layer

- Shell/global layer owns: `Cmd+W`, `Cmd+K`, modal-open key routing.
- Entity layer owns: content-local keys only (`Enter`, optional `Escape` based on prompt intent).
- Modal layer owns: Escape and navigation while overlay is open.

Avoid dual Escape-submit checks in both intercepted and non-intercepted entity paths.

### 4) Standardize submit outcomes for content prompts

Define semantic outcomes (even if mapped to current callback shape):
- `SubmitAck` -> currently `on_submit(id, None)`.
- `SubmitValue(value)` -> currently `on_submit(id, Some(value))` from `submit:value` links.
- `Cancel` -> no submit payload; closes/dismisses prompt per policy.

Footer primary and keyboard Enter should trigger the same semantic action path.

### 5) Compose chrome via one prompt-level spec

For content prompts, wrapper should receive one spec that drives header + footer + overlay together:

```text
ContentPromptChromeSpec {
  title
  subtitle_hint
  helper_text
  info_label
  show_actions
  escape_behavior
  submit_behavior
}
```

This removes duplicate literal strings and avoids per-prompt drift between header hint and footer helper copy.

## Suggested Div-Specific Direction (Documentation, Not Implemented)

- Keep Div entity focused on HTML rendering, scroll, and link behaviors.
- Move all Div chrome copy/config assembly behind a content-prompt chrome contract (not Arg helper names).
- Route footer primary submit through the same semantic submit path used by keyboard Enter.
- Make Escape policy explicit for Div (`EscapeBehavior::SubmitAck` or `EscapeBehavior::Dismiss`) instead of implicit cross-layer behavior.

## Open Questions

- Should plain Escape in Div continue (`SubmitAck`) or dismiss without submit?
- Should `submit:value` links be available in all content prompts, or only in Div-like HTML prompts?
- Should header hint and footer helper both be shown for content prompts, or should one lane own user guidance?
