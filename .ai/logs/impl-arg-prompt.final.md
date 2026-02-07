**Changed Files**
- `src/render_prompts/arg.rs`

**Implemented**
1. Added a shared arg submit resolver and unified submission path for both Enter and footer primary click (`src/render_prompts/arg.rs:57`, `src/render_prompts/arg.rs:197`, `src/render_prompts/arg.rs:452`, `src/render_prompts/arg.rs:674`).
2. Added Tab autocomplete for arg prompt using current filtered selection (single-match and selected-item fallback) (`src/render_prompts/arg.rs:125`, `src/render_prompts/arg.rs:209`, `src/render_prompts/arg.rs:445`).
3. Added contextual helper state/text so footer guidance reflects matching state, plus clearer no-match list message (`src/render_prompts/arg.rs:88`, `src/render_prompts/arg.rs:109`, `src/render_prompts/arg.rs:507`, `src/render_prompts/arg.rs:651`).
4. Added unit tests for submit outcome, helper status, and tab completion behavior (`src/render_prompts/arg.rs:782`).

**How To Test**
1. `cargo check`  
2. `cargo test --lib test_resolve_arg_submit_outcome_returns_invalid_when_input_is_empty`  
3. `cargo test test_resolve_arg_submit_outcome_returns_invalid_when_input_is_empty`

**Verification Run**
- `cargo check`: passed.
- `cargo test --lib ...`: fails due unrelated existing compile error in `src/ai/window.rs:8549` (`AiApp::message_body_content` missing).
- `cargo test ...` (non-lib targeted): also blocked by unrelated existing test compile issues in `tests/prompt_footer.rs` and the same `src/ai/window.rs:8549` error.

**Risks / Known Gaps**
1. Raw typed value still submits on first Enter when no matches; no two-step confirmation flow was added.
2. Tab completion currently applies `choice.name` (not `choice.value`) to input text.
3. Full test suite is currently blocked by pre-existing unrelated failures.

**Commits**
- No commits were created.