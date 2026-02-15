The simplest fix without new deps: add a test-only static mutex to serialize all cache tests. Each test should acquire this lock before touching the shared global cache.

Now I have all the information. Here are the swarm tasks:

---

### Task 1: fix-cache-test-races
**Scope:** `src/clipboard_history/cache.rs`
**Task:**

The two failing tests race against each other (and against all other cache tests in this module) because they all mutate shared global statics (`ENTRY_CACHE`, `IMAGE_CACHE`) via `OnceLock<Mutex<...>>`. Cargo runs `#[test]` functions in parallel within the same process, so `invalidate_entry_cache()` in one test can fire between an `upsert_entry_in_cache()` and an `assert!` in another.

**Fix:** Add a `static TEST_MUTEX: Mutex<()>` at the top of the `#[cfg(test)] mod tests` block. At the start of every test function in this module, acquire `let _guard = TEST_MUTEX.lock().unwrap();` before doing anything. This serializes all cache tests so global state is never corrupted by concurrent access.

Concretely, in `src/clipboard_history/cache.rs`, inside `mod tests`:

1. Add after `use super::*;`:
   ```rust
   use std::sync::Mutex as StdMutex;
   static TEST_MUTEX: StdMutex<()> = StdMutex::new(());
   ```

2. Add `let _guard = TEST_MUTEX.lock().expect("test mutex");` as the first line of every `#[test]` function (there are 9 tests total in this module):
   - `test_upsert_new_entry`
   - `test_upsert_updates_existing`
   - `test_upsert_maintains_pinned_order`
   - `test_remove_entry_from_cache`
   - `test_update_pin_status`
   - `test_update_ocr_text_in_cache_sets_text_when_entry_exists`
   - `test_upsert_entry_in_cache_truncates_when_exceeding_max_capacity`
   - `test_upsert_entry_in_cache_keeps_pinned_items_first_when_inserting_unpinned`
   - `test_remove_entry_from_cache_does_not_replace_cache_when_id_missing`
   - `test_clear_all_caches_empties_entry_cache_and_remains_empty_on_repeat_call`

This adds zero new dependencies, changes zero production code, and directly fixes both assertion failures by preventing interleaved cache mutations.

---

NEXT_AREA: Audit remaining `unwrap()` / `.expect()` calls in non-test `src/clipboard_history/` production code (70 instances flagged) and convert to `?` or graceful handling.

Shall I go ahead and implement this fix?
