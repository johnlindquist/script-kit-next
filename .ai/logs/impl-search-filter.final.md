Implemented search/filter improvements in two unclaimed files:

- `src/scripts/search.rs:266`
- `src/file_search.rs:1381`

**What changed**
- `src/scripts/search.rs:266`
  - Added a Unicode-safe highlight path (`SearchHighlightMatchCtx` + `UnicodeHighlightCtx`) using `nucleo` `Pattern::indices(...)`.
  - Kept ASCII fast-path for ASCII pairs.
  - Switched `compute_match_indices_for_result(...)` to use the shared context, so highlight indices now align better with Unicode fuzzy ranking.
- `src/scripts/search.rs:1876`
  - Added regression tests for Unicode normalization highlight behavior in name/description.
- `src/file_search.rs:1391`
  - Refactored file-search ranking to a shared `(index, score)` core helper (`rank_file_results_nucleo`), reducing intermediate allocation/cloning paths.
  - Added deterministic tie-break ordering (score desc, then name, then index) for stable results.
- `src/file_search.rs:1963`
  - Added regression test for deterministic tie-break behavior on equal-score matches.

**Verification run**
- `cargo check` ✅ passed.

**Test status**
- Targeted `cargo test` is currently blocked by unrelated existing test-compilation issues in the working tree:
  - `src/ai/window.rs:8729` (`AiApp::message_body_content` missing)
  - duplicate `tests` module errors when building bin tests (example: `src/render_prompts/arg.rs:740` and `src/main.rs:3859`)

**How to test**
1. `cargo check`
2. After unrelated test compile issues are fixed:
   - `cargo test unicode_normalization -- --nocapture`
   - `cargo test test_filter_results_nucleo_empty_pattern_uses_name_tiebreaker -- --nocapture`

**Risks / known gaps**
- Plan items touching `src/prompts/select.rs` and `src/app_impl.rs` were not included in this patch (those files were owned/claimed by other active agents during this work).
- Full `cargo test`/`cargo clippy --all-targets -- -D warnings` verification is pending once the unrelated compile failures are resolved.

Commits made: none.