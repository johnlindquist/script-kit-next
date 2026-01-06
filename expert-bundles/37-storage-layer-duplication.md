# Expert Bundle 37: Storage Layer Code Duplication

## Goal
Identify opportunities to consolidate duplicated SQLite database patterns across 6 storage modules into a unified abstraction layer.

## Current State

The codebase has **6 separate database storage modules** with nearly identical patterns:
- `src/notes/storage.rs` (386 lines) - Notes database
- `src/ai/storage.rs` (1315 lines) - AI chats database  
- `src/clipboard_history/database.rs` (914 lines) - Clipboard history
- `src/menu_cache.rs` (583 lines) - Menu cache
- `src/app_launcher.rs` (1277 lines) - App launcher cache
- `src/clipboard_history/db_worker/mod.rs` (349 lines) - DB worker

Each module independently implements:
- Global `OnceLock<Arc<Mutex<Connection>>>` singleton pattern
- `get_*_db_path()` returning `~/.scriptkit/db/*.sqlite`
- `init_*_db()` with idempotent check and schema creation
- Lock acquisition with identical error mapping
- CRUD operations with similar patterns

## Specific Concerns

1. **Singleton Boilerplate (5 copies)**: Each module has identical `static DB: OnceLock<Arc<Mutex<Connection>>>` declarations and initialization logic.

2. **Lock Error Handling (30+ copies)**: The exact pattern `.map_err(|e| anyhow::anyhow!("DB lock error: {}", e))?` appears 30+ times across files.

3. **DateTime Parsing (6 copies)**: Identical RFC3339 parsing with fallback to `Utc::now()` is duplicated in row-to-struct conversions.

4. **Inconsistent PRAGMA Configuration**: Notes/Clipboard have WAL mode; AI storage is **missing WAL** - potential bug and performance issue.

5. **FTS5 Setup Duplication**: Both notes and AI storage have ~50 lines of nearly identical FTS5 virtual table and trigger setup.

## Key Questions

1. Should we create a generic `DatabaseManager` that handles connection pooling, initialization, and lock acquisition for all modules?

2. What's the best trait-based abstraction for CRUD operations that share soft-delete, timestamp, and search patterns?

3. Is there value in a single unified database vs. separate SQLite files for isolation?

4. How should we handle the FTS5 trigger generation - macro, helper function, or templating?

5. What's the safest migration path that doesn't break existing user databases?

## Implementation Checklist

- [ ] Create `src/db/mod.rs` with `DatabaseManager` singleton
- [ ] Add `get_db_path(name: &str) -> PathBuf` helper
- [ ] Create `with_db<F, T>()` lock helper to eliminate error mapping duplication
- [ ] Add `parse_datetime()` and `parse_datetime_opt()` helpers
- [ ] Standardize PRAGMA configuration (fix missing WAL in AI storage)
- [ ] Create `fts5_schema(table, columns)` helper function
- [ ] Consider `SoftDeletable` trait for shared CRUD patterns
- [ ] Migrate one module at a time to new abstractions
