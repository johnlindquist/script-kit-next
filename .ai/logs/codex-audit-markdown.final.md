**Findings (Markdown Audit)**
1. High: Nested list parent text could be dropped when a child list started, because pending spans were cleared before being attached to the parent item. Fixed in `src/prompts/markdown/parse.rs:3` and mirrored test parser in `src/prompts/markdown/test_support.rs:4`.
2. Medium: Linked images used the image URL instead of the outer link URL for click target (`[![alt](img)](link)`). Fixed in `src/prompts/markdown/parse.rs:159`.
3. Medium (performance): Parser was cloning span vectors in several hot paths (paragraph/heading/table). Replaced with move-based `std::mem::take` transfers to reduce allocation churn on large markdown documents in `src/prompts/markdown/parse.rs:108` and `src/prompts/markdown/test_support.rs:118`.

**Changed Files**
- `src/prompts/markdown/parse.rs`
- `src/prompts/markdown/test_support.rs`
- `src/prompts/markdown/tests.rs`

**How To Test**
1. `cargo test --lib prompts::markdown::tests`
2. `cargo check --lib`
3. `cargo clippy --lib -- -D warnings -A clippy::field-reassign-with-default`

Additional note: strict `cargo clippy --lib -- -D warnings` was blocked by unrelated in-progress changes in `src/prompts/div/render_html.rs`.

**Risks / Known Gaps**
- I did not run full-workspace gates (`cargo test`/full clippy) due parallel-agent constraints and unrelated active changes.
- Markdown images are still rendered as text-link placeholders (`[Image: ...]`) by design; no inline image widget behavior was changed.

**Commits**
- `40bb1bd` `fix(markdown): preserve nested list text and link-wrapped image URLs`