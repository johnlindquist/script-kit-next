Implemented the data-structure improvements from `plans/data-structure-improvements.md` across the hot paths, with tests added for new behavior.

**Changed files**
- `src/keyword_matcher.rs`
  - Replaced string buffer front-trimming with `VecDeque<char>`.
  - Added last-character trigger index: `HashMap<char, Vec<TriggerPattern>>`.
  - Added indexed suffix matching and trigger-index rebuild on register/unregister.
  - Added test: `test_buffer_trims_multibyte_chars_without_dropping_extra`.
- `src/scheduler.rs`
  - Replaced `Vec<ScheduledScript>` with `HashMap<PathBuf, ScheduledScript>` for O(1) add/remove/update by path.
  - `list_scripts()` now returns deterministic sorted output.
  - Added test: `test_scheduler_list_scripts_returns_paths_in_sorted_order`.
- `src/notification/service.rs`
  - Added side indexes:
    - `active_id_index: HashMap<NotificationId, usize>`
    - `active_dedupe_index: HashMap<String, NotificationId>`
  - Updated dedupe/progress/get/dismiss paths to use indexes.
  - Added index rebuild/maintenance helpers.
- `src/shortcuts/registry.rs`
  - Added precomputed effective shortcut index:
    - `HashMap<ShortcutContext, HashMap<String, usize>>`
  - `find_match()` now uses canonical keystroke lookup instead of scanning all bindings.
  - Rebuilds index on register/unregister/override changes.
- `src/keystroke_logger.rs`
  - Replaced `String` front-removal with `VecDeque<char>` ring-style behavior.
  - Added Unicode-safe recent-char handling for multibyte input.
  - Added test: `test_recent_chars_limit_counts_characters_for_multibyte_input`.
- `src/scripts/grouping.rs`
  - Replaced repeated `position()` scans with precomputed name→index map for default suggestions.
- `src/template_variables.rs`
  - Reworked extraction to streaming `chars().peekable()` parsing (no `Vec<char>` allocation).

**Verification run**
- `cargo check` passed earlier during this task.
- Targeted tests that passed:
  - `cargo test test_buffer_trims_multibyte_chars_without_dropping_extra`
  - `cargo test test_scheduler_list_scripts_returns_paths_in_sorted_order`
- Current tree is being modified by other active agents; latest global builds are failing in unrelated areas (`src/hotkeys.rs`, `src/watcher.rs`, config test updates), which blocked a clean final full test sweep.

**How to test**
1. `cargo check`
2. `cargo test test_buffer_trims_multibyte_chars_without_dropping_extra`
3. `cargo test test_scheduler_list_scripts_returns_paths_in_sorted_order`
4. `cargo test test_recent_chars_limit_counts_characters_for_multibyte_input`
5. Optional broader check once parallel agent churn settles: `cargo test`

**Risks / known gaps**
- Full `cargo test` is currently unstable due unrelated concurrent changes outside this task’s scope.
- Shortcut matching now depends on index rebuild correctness; I wired rebuilds into all mutation paths, but future mutation entry points must also call rebuild.

**Commits**
- No commits were created in this run.