# Clipboard + Database Crate Audit

Date: 2026-02-07  
Repo: `script-kit-gpui`

## Scope

Audited crates and integration points:
- `arboard = "3.6"` (`Cargo.toml:71`)
- `rusqlite = { version = "0.38", default-features = false, features = ["bundled"] }` (`Cargo.toml:72`)
- `lru = "0.12"` (`Cargo.toml:73`)
- `sha2 = "0.10"` (`Cargo.toml:74`)
- `base64 = "0.22"` (`Cargo.toml:67`)
- `uuid = { version = "1.0", features = ["v4"] }` (`Cargo.toml:85`)

Primary code paths reviewed:
- Clipboard history: `src/clipboard_history/*.rs`
- Other SQLite stores: `src/ai/storage.rs`, `src/notes/storage.rs`, `src/menu_cache.rs`, `src/app_launcher.rs`

## Executive Summary

- `arboard` on macOS is **mostly functional but not fully robust under transient clipboard failures**.
- `rusqlite` connection pooling is **not required** for current SQLite usage; a dedicated DB worker per store is a better scaling path than a generic pool.
- Runtime SQL is **mostly parameterized**; no direct user-input SQL interpolation found in query execution paths.
- `lru` image cache limit is **likely oversized** for low-memory devices (100 decoded images can be very large).
- `sha2` usage for dedup is **efficient overall** and correctly leveraged with content-addressed blobs.
- `base64` usage is mostly legacy/fallback and reasonably optimized.
- `uuid v4` usage is good for IDs; correlation ID usage exists but is not consistently applied to all relevant logs.

## Findings

### 1) `arboard` (3.6): macOS robustness

**What works well**
- Monitor uses OS-level `NSPasteboard` change-count detection before payload reads (`src/clipboard_history/change_detection.rs:22`, `src/clipboard_history/monitor.rs:168`).
- Polling interval is reduced when change-count is available (`src/clipboard_history/monitor.rs:142-183`).
- For copy-back of image blobs, code uses native macOS pasteboard API to publish both image + file URL (CleanShot-style), which avoids some common app-compat issues (`src/clipboard_history/macos_paste.rs:37-104`, `src/clipboard_history/clipboard.rs:61-66`).

**Robustness gaps**
- Monitor keeps a single long-lived `Clipboard` instance and does not reinitialize on repeated `get_text`/`get_image` failures (`src/clipboard_history/monitor.rs:132`, `src/clipboard_history/monitor.rs:204`, `src/clipboard_history/monitor.rs:257`).
- `get_text()` and `get_image()` errors are effectively silent in hot path (pattern is `if let Ok(...)` with no error logging), which can hide real clipboard access failures.
- Copy operations (`set_text`, `set_image`) do not currently implement retry/backoff for transient lock/contention failures (`src/clipboard_history/clipboard.rs:55-74`).

**Assessment**
- Robust enough for normal operation.
- Not robust enough for intermittent pasteboard contention or temporary OS clipboard failures.

**Recommendation**
- Add bounded retry + reinit policy for `Clipboard` in monitor and copy paths (e.g., 2-3 retries with short jittered backoff).
- Add structured warn logs for repeated read failures (with `correlation_id`) and a throttled counter metric.

---

### 2) `rusqlite` (0.38 bundled): is connection pooling needed?

**Current architecture**
- Multiple stores use `OnceLock<Arc<Mutex<Connection>>>` singleton connections:
  - Clipboard: `src/clipboard_history/database.rs:23`
  - AI chats: `src/ai/storage.rs:16`
  - Notes: `src/notes/storage.rs:16`
  - Menu cache: `src/menu_cache.rs:26`
  - App launcher cache: `src/app_launcher.rs:102`
- WAL + busy timeout are enabled in key stores (`src/clipboard_history/database.rs:54-63`, `src/ai/storage.rs:49-64`, `src/notes/storage.rs:47-49`).

**Pool necessity**
- For SQLite in-process desktop usage, a generic connection pool is usually low-value and can increase complexity.
- Current bottleneck risk is more about global mutex serialization than connection count.
- A dedicated writer-thread / request-queue model is already scaffolded for clipboard (`src/clipboard_history/db_worker/mod.rs:1-9`) and is a more appropriate scaling strategy than pooling.

**Assessment**
- Connection pooling is **not needed right now**.
- If contention appears, prefer completing DB worker migration (or split read/write connections) over adding a general pool.

---

### 3) SQL parameterization / injection risk

**Positive**
- Query execution paths are consistently parameterized with placeholders + `params![]` across clipboard/ai/notes/menu-cache/app-launcher.
- FTS/LIKE searches use bound params with query sanitization wrappers in AI and Notes stores (`src/ai/storage.rs:443-448`, `src/ai/storage.rs:485-516`, `src/notes/storage.rs:250-253`, `src/notes/storage.rs:286-321`).

**Notable exception**
- Dynamic SQL in migration helper:
  - `src/clipboard_history/db_worker/mod.rs:209-212`
  - `src/clipboard_history/db_worker/mod.rs:221`
- This interpolates column identifiers/types with `format!`. In this code path, inputs are internal constant names from `run_migrations` (`src/clipboard_history/db_worker/mod.rs:181-186`), so practical injection risk is low.

**Assessment**
- No user-input SQL injection issue identified in runtime query paths.
- Minor hardening opportunity: avoid `format!` even in migrations by validating identifiers against a fixed enum/list before interpolation.

---

### 4) `lru` (0.12): cache sizing

**Current settings**
- Decoded image LRU: `MAX_IMAGE_CACHE_ENTRIES = 100` (`src/clipboard_history/cache.rs:16`)
- Commented estimate: 1-4MB per image => ~100-400MB worst-case (`src/clipboard_history/cache.rs:15-24`)
- Metadata cache: 500 entries (`src/clipboard_history/cache.rs:19`, `src/clipboard_history/cache.rs:107`).

**Assessment**
- Metadata cache size looks reasonable.
- Decoded image cache default (100) is likely aggressive for laptops under memory pressure.

**Recommendation**
- Make image-cache capacity configurable (config/env) with safer default (for example 24-48).
- Consider adaptive cap by total RAM or by decoded byte budget, not only entry count.

---

### 5) `sha2` (0.10): dedup efficiency

**Where used**
- Clipboard DB content hash for dedup index lookup (`src/clipboard_history/database.rs:25-30`, `src/clipboard_history/database.rs:282-289`).
- Blob-store content addressing for PNG files (`src/clipboard_history/blob_store.rs:25-31`, `src/clipboard_history/blob_store.rs:42-49`).

**Efficiency analysis**
- Good: content-addressed blob store hashes PNG bytes once and skips duplicate writes if file exists (`src/clipboard_history/blob_store.rs:42-49`).
- Good: DB dedup for images operates on `blob:<hash>` content strings in `add_entry`, so it avoids re-hashing full image bytes at DB insert time.
- Good: monitor uses a fast non-crypto sampler hash for change detection (`src/clipboard_history/image.rs:412-424`) and reserves SHA-256 for actual dedup/persistence.

**Assessment**
- `sha2` is used efficiently for the dedup responsibilities it serves.

---

### 6) `base64` (0.22) and `uuid` (1.0 v4)

**`base64`**
- Primarily used for legacy image formats and compatibility paths (`src/clipboard_history/image.rs:29-41`, `src/clipboard_history/image.rs:65-76`).
- Current primary path uses blobs, which avoids base64 overhead (`src/clipboard_history/image.rs:20-27`).
- Includes fast PNG-header parsing that decodes only 32 base64 chars for dimensions (`src/clipboard_history/image.rs:370-409`).

Assessment: reasonable usage; no major concern.

**`uuid v4`**
- Used for clipboard entry IDs (`src/clipboard_history/database.rs:322`) and some correlation IDs in monitor/maintenance logs (`src/clipboard_history/monitor.rs:75-80`, `src/clipboard_history/monitor.rs:215-220`, `src/clipboard_history/database.rs:404-410`).

Assessment: ID generation is appropriate; correlation-ID discipline is partial rather than comprehensive.

## Prioritized Recommendations

1. `P1` Improve macOS clipboard resilience around `arboard`:
   - Retry + reinitialize clipboard on transient read/write failures.
   - Add throttled structured failure logs with `correlation_id`.
2. `P1` Right-size decoded image LRU:
   - Lower default and make configurable.
   - Prefer byte-budget eviction if feasible.
3. `P2` Harden migration SQL construction:
   - Keep dynamic SQL for identifiers only, but enforce strict identifier whitelist/enum.
4. `P2` (Optional) If DB contention increases, finish clipboard DB worker migration instead of adding connection pooling.

## Bottom Line Answers

- Is `arboard` clipboard access robust on macOS? **Partially. Works, but needs retry/reinit + better error surfacing for true robustness.**
- Is `rusqlite` connection pooling needed? **No, not currently. Prefer worker-thread/read-write split if contention is observed.**
- Are SQL queries parameterized (no injection)? **Yes for runtime user-driven queries; one low-risk dynamic migration helper should be hardened.**
- Is the LRU cache sized appropriately? **Metadata cache yes; decoded image cache is likely too high by default.**
- Are we using `sha2` efficiently for dedup? **Yes. SHA-256 is used in the right places with content-addressed storage and indexed dedup.**
