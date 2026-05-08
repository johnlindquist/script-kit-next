# Logging

Dual-output structured logging (JSONL + pretty stderr) lives under `src/logging/`. Every log site that interpolates untrusted user input must route through the safe-value + rate-limit contract so log volume stays bounded.

## Safe user-value contract

Oracle-Session `logging-observability-next-pass` PR1 introduced a two-helper safety contract for any log site that interpolates untrusted input (stdin text, chat titles, dictation queries, triggerBuiltin names, ACP command displays, …).

- **Byte-capped preview** via [[src/logging/safe_user_value.rs#log_user_value]] returning [[src/logging/safe_user_value.rs#LogSafe]]. The cap is **bytes**, not chars — log budget is disk + JSONL bytes, and a 120-char emoji/combining-mark string can exceed 480 bytes. The helper walks back to a UTF-8 char boundary, trims trailing whitespace, and appends `…` inside the byte budget. Default cap is [[src/logging/safe_user_value.rs#SAFE_USER_VALUE_MAX_BYTES]] (200 bytes); override via [[src/logging/safe_user_value.rs#log_user_value_with_limit]].
- **Time-window rate limit** via [[src/logging/rate_limit.rs#log_rate_limit]] returning [[src/logging/rate_limit.rs#LogRateDecision]]. The limiter buckets `(category, key.len(), hash(key))` so the raw untrusted string is never retained. Default window is 30s, stale buckets are GC'd after 120s, and the map is capped at 2048 keys with automatic pruning — a hostile client randomizing keys cannot grow the map unbounded.

Every untrusted-value log must emit these structured fields alongside the preview: `*_preview = %safe`, `*_bytes = safe.raw_bytes`, `*_safe_bytes = safe.safe_bytes`, `*_truncated = safe.truncated`, and `suppressed = rate.suppressed`. Byte-level metadata lets downstream budget accounting key off the values without re-measuring the string.

The reference migration is [[src/app_impl/trigger_builtin_dispatch.rs#ScriptListApp#log_unknown_trigger_builtin]], which replaced an ad-hoc `chars().take(120)` preview plus occurrence-count gate with the shared helpers. [[src/app_impl/trigger_builtin_dispatch.rs#ScriptListApp#log_deprecated_trigger_builtin_name]] and [[src/app_impl/trigger_builtin_dispatch.rs#ScriptListApp#log_invalid_trigger_builtin]] use the same pattern.

The two gates are complementary. `logging::log_rate_limit` bounds **same-key bursts inside a 30s window** (so a fuzzer looping the same unknown name cannot produce same-second spam). `protocol_stats::should_log_occurrence(total)` still bumps counters and fires the 1st / 100th occurrence — but is no longer the only defense.

## Live debug trace markers

Verbose reproduction traces use stable marker strings so `./dev.sh` sessions can be filtered while a user exercises a broken UI flow.

The `DO_IN_TRACE` marker follows shared filter-input changes, current-app command normalization, intent resolution, and built-in execution routing for the "Do in Current App" flow. The `SCROLL_TRACE` marker follows current-app command list rendering, wheel-step accumulation, scroll metrics, wheel-owned selection notes, and reanchor decisions. Trace fields that include user-entered text use [[src/logging/safe_user_value.rs#log_user_value]] previews with byte metadata.

The `script_kit::input_history` target follows main-menu Up/Down routing when arrow keys may switch between list navigation and saved input history. It records route decisions, render-paced key-repeat coalescing, render acknowledgments, obsolete-pending cancellations, programmatic filter echo suppression, history indices, and ScriptList key-up receipts so a dev-loop trace can distinguish held/repeat keydowns from post-release navigation. History text uses safe previews rather than raw values.

The ignored `main_menu_history_render_prep_benchmark` test measures the render-prep side of that same path. It asserts that history recall list-state replacement stays below frame budget and that the hot path does not reintroduce full-list `measure_all` work. The `scripts/bench-main-menu-history-render.mjs` gate mirrors those source invariants without depending on proc-macro test compilation.

## Source files

Current code references for this page.

- [src/logging/mod.rs](../src/logging/mod.rs)
- [src/logging/safe_user_value.rs](../src/logging/safe_user_value.rs)
- [src/logging/rate_limit.rs](../src/logging/rate_limit.rs)
- [src/app_impl/trigger_builtin_dispatch.rs](../src/app_impl/trigger_builtin_dispatch.rs)
