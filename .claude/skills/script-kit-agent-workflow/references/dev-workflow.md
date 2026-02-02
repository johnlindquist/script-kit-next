# Development Workflow

## Hot Reload

Run `./dev.sh` (cargo-watch) for automatic rebuilds.

Triggers:
- `.rs` → rebuild
- `~/.scriptkit/theme.json` → live reload
- `~/.scriptkit/scripts/` → live reload
- `~/.scriptkit/config.ts` → restart

## Debug Tools

- Logs panel: `Cmd+L`
- Tags: `[UI] [EXEC] [KEY] [THEME] [FOCUS] [HOTKEY] [PANEL]`
- Perf tags: `[KEY_PERF] [SCROLL_TIMING] [PERF_SLOW]`

## Performance Tests

```bash
bun run tests/sdk/test-scroll-perf.ts
npx tsx scripts/scroll-bench.ts
```

Thresholds:
- P95 key latency <50ms
- Single key <16.67ms
- Scroll op <8ms

## Lessons Learned

### Mass Test Failure Triage

When a refactor causes 100+ test failures:
1. **Stop** - don't fix tests one by one
2. **Count error types:**
   ```bash
   cargo test 2>&1 | grep "error\[E" | sort | uniq -c
   ```
3. Find the single root cause (usually one type dominates)
4. Create systematic helpers for bulk fixes
5. Verify incrementally: `cargo check` before `cargo test`

### Type Migration Checklist (T → Arc<T>)

1. structs
2. signatures
3. return types
4. test helpers
5. transform test data
6. fix field assigns
7. `cargo check`
8. fix stragglers
9. full `cargo test`

Example helper:
```rust
fn wrap_scripts(scripts: Vec<Script>) -> Vec<Arc<Script>> {
    scripts.into_iter().map(Arc::new).collect()
}
```
