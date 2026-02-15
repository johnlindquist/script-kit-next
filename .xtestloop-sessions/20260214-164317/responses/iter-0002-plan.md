The swarm task list is above. Four tasks, each scoped to <=10 files:

1. **error-handling-macos-paste** — Replace 4 `expect()` calls with `?` + `Context` in `macos_paste.rs`, add `error_class` structured tracing
2. **db-worker-error-tracing** — Add `error_class` tracing to silent `unwrap_or()` fallback paths in `db_worker/`
3. **db-impl-crud-integration-test** — Full CRUD lifecycle characterization tests against in-memory SQLite in `db_impl.rs`
4. **cache-boundary-characterization-tests** — Boundary tests for cache truncation, pinned ordering, noop remove, and clear-all in `cache.rs`
