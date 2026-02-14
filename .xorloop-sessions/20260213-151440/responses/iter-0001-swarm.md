## Swarm Summary

Both tasks completed successfully and are committed:

**Task 1: `rm-dead-clone-fn`** — Deleted 2 dead-code functions (`load_apps_from_db_with_icons` and `app_needs_update`) that had zero call sites and were marked `#[allow(dead_code)]`.

**Task 2: `extract-db-conn-helper`** — Created `with_apps_db<T>(default, closure)` helper to extract the repeated `get_apps_db()` + `db.lock()` boilerplate. Refactored all 3 remaining DB functions (`load_apps_from_db`, `save_app_to_db`, `get_apps_db_stats`) to use it.

**Result:** `src/app_launcher/db_cache.rs` went from 378 lines down to **263 lines** (-130 lines net, -232 deleted / +102 added). `cargo check` passes clean. The commit is `234631f3 refactor(app_launcher): extract apps DB access helper`.
