Implemented the search/filter audit report at `plans/search-filter-improvements.md`.

**Changed files**
- `plans/search-filter-improvements.md`

**Verification run**
1. `cargo check`  
   - Passed.
2. `cargo test --lib scripts::search::tests::test_exact_name_match_ranks_first`  
   - Failed due pre-existing unrelated compile error: missing `AiApp::message_body_content` in `src/ai/window.rs:8549`.

**How to test**
1. Open `plans/search-filter-improvements.md` and review findings/roadmap.
2. Re-run:
   - `cargo check`
   - `cargo test --lib scripts::search::tests::test_exact_name_match_ranks_first` (currently blocked by the existing `src/ai/window.rs` error).

**Risks / known gaps**
- This task produced an audit/report only; no search/filter code was implemented yet.
- Findings are code-audit based; no new runtime profiling traces were added in this pass.
- Test execution beyond `cargo check` is currently blocked by the unrelated lib-test compile issue above.

**Commits**
- No commits were made.