# Rust Crates Research: Best Practices & Gotchas

Research document for crates used in Script Kit GPUI, compiled from official documentation, GitHub repos, and project code patterns.

---

## 1. rusqlite 0.38 (with bundled feature)

### Connection Management Patterns

**Global Connection Pattern (Used in this project):**
```rust
// clipboard_history/database.rs, notes/storage.rs, ai/storage.rs
static DB_CONNECTION: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

pub fn get_connection() -> Result<Arc<Mutex<Connection>>> {
    if let Some(conn) = DB_CONNECTION.get() {
        return Ok(conn.clone());
    }
    let conn = Connection::open(&db_path)?;
    // ... setup ...
    Arc::new(Mutex::new(conn))
}
```

**Gotchas:**
- **Thread Safety**: SQLite itself is thread-safe (with PRAGMA busy_timeout set), but `rusqlite::Connection` is NOT `Send + Sync`. You MUST wrap in `Arc<Mutex<_>>` for multi-threaded access.
- **Mutex Poisoning**: Use `parking_lot::Mutex` instead of `std::sync::Mutex` to avoid poisoning on panics (this project doesn't, but worth noting).
- **OnceLock Pattern**: Safe for one-time initialization, but requires fallible initialization inside. The pattern in this project handles race conditions correctly via `set()` + `get()`.

**Best Practice in this Project:**
```rust
// Explicit lock release before cross-module updates
let conn = conn.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
// ... query ...
drop(conn);  // Release lock explicitly
refresh_entry_cache();  // Don't hold lock during cache update
```

### Parameter Binding

**Positional Binding (Primary method):**
```rust
conn.execute(
    "INSERT INTO history (id, content) VALUES (?1, ?2)",
    params![&id, &content],  // Macro expands to (&id, &content)
)?;
```

**Named Binding (For complex queries):**
```rust
conn.execute(
    "UPDATE history SET timestamp = :ts WHERE id = :id",
    params_from_iter(vec![
        (":ts", timestamp.to_string()),
        (":id", id.to_string()),
    ])?
)?;
```

**Gotchas:**
- **Parameter Order Matters**: `?1` refers to first param, `?2` to second. Mis-indexing silently reads NULL.
- **String Lifetimes**: Use `&str` or `String` directly—they implement `rusqlite::ToSql`. Avoid temporary string builders.
- **Named Parameters**: Requires colon prefix in SQL (`:name`) and in the params map.
- **NULL Handling**: `Option<T>` maps to SQL NULL automatically; `None` → NULL, `Some(v)` → v.

**This Project's Pattern:**
Uses `params![]` macro consistently with positional binding `?1, ?2, ...` for clarity.

### Error Handling Patterns

**Not `unwrap()` / `expect()`:**
```rust
// ✗ Bad - violates CLAUDE.md rule #3
conn.execute("...", []).unwrap();

// ✓ Good - from this project
conn.execute("...", []).context("Failed to execute query")?;
```

**Query Result Extraction:**
```rust
// Optional result (query_row returns Err if no rows)
let value: Option<String> = conn
    .query_row("SELECT content FROM history WHERE id = ?", params![id], |row| row.get(0))
    .ok();  // Convert Err → None

// Mandatory result with context
let value: String = conn
    .query_row("SELECT content FROM history WHERE id = ?", params![id], |row| row.get(0))
    .context("Failed to fetch content for ID")?;
```

**Stream Results (query_map):**
```rust
let mut stmt = conn.prepare("SELECT id, content FROM history")?;
let rows = stmt.query_map([], |row| {
    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
})?;

// Collect with error handling
let entries: Vec<_> = rows
    .filter_map(|r| r.ok())  // Skip rows with read errors
    .collect();
```

**This Project's Pattern:** Always uses `.context()` with `?`, never `unwrap()` in production code.

### Transaction Usage

**Explicit Transaction (Not used in this project yet):**
```rust
let tx = conn.transaction()?;
tx.execute("INSERT ...", [])?;
tx.execute("UPDATE ...", [])?;
tx.commit()?;
```

**Savepoints:**
```rust
let sp = conn.savepoint()?;
sp.execute("...", [])?;
sp.commit()?;  // or implicit rollback on drop
```

**Gotchas:**
- **Drop Behavior**: By default, transactions ROLLBACK on drop if not explicitly committed.
- **Savepoints Auto-Release**: Savepoints release on drop (no rollback).
- **Nested Transactions**: SQLite 3.6.8+ supports savepoints natively; older versions fail.
- **Lock Duration**: Transactions hold locks. Release ASAP by explicit `.drop(tx)` or `drop(tx)`.

**This Project's Pattern:** No explicit transactions; relies on individual `execute()` calls with busy_timeout for concurrency.

### Common Mistakes with Lifetimes

**Gotcha #1: Borrowed Data in Closures**
```rust
// ✗ Borrowed reference lives only within query_map iteration
let values: Vec<String> = vec!["a", "b", "c"];
let results = stmt.query_map([], |row| {
    // `values` borrow is OK here but...
    row.get::<_, String>(0)
})?;
// results contain references to values—DROP values, results become invalid

// ✓ Collect before dropping references
let collected: Vec<_> = results.filter_map(|r| r.ok()).collect();
drop(values);
// OK, collected is owned
```

**Gotcha #2: Statement Lifetime**
```rust
let mut stmt = conn.prepare("SELECT ...")?;
let rows = stmt.query_map([], |row| row.get(0))?;
// Don't drop stmt yet; rows may still reference it
// rows own the statement implicitly via the handle
```

**This Project's Pattern:** Converts `query_map` results immediately to `Vec<T>` to avoid lifetime issues.

### Thread Safety (Send + Sync)

**CRITICAL:** `rusqlite::Connection` is **NOT** `Send + Sync`.
- Cannot be passed between threads directly.
- Must wrap in `Arc<Mutex<Connection>>`.
- Each thread can have its own `Connection` (SQLite supports multi-connection per file via locks).

**This Project:** Uses single global `Arc<Mutex<Connection>>` with busy_timeout, avoiding per-thread connections.

### WAL Mode & Concurrency Pragmas

```rust
conn.execute_batch("PRAGMA journal_mode=WAL;")?;      // Write-Ahead Logging
conn.execute_batch("PRAGMA synchronous=NORMAL;")?;    // Balanced safety/speed
conn.execute_batch("PRAGMA auto_vacuum=INCREMENTAL;")?;  // Reclaim disk space
conn.busy_timeout(Duration::from_secs(5))?;           // Retry for 5 seconds if locked
```

**Gotchas:**
- **WAL Requires Coordination**: Multiple writers need locks; busy_timeout is essential.
- **PRAGMA busy_timeout Must Be Set Per-Connection**: The setting in this project (5000ms = 5s) prevents "database is locked" errors from brief contention.
- **auto_vacuum = INCREMENTAL**: Requires manual `PRAGMA incremental_vacuum()` or continuous background work.
- **synchronous=NORMAL + WAL**: Safe even on power failure (WAL handles crash recovery), faster than FULL.

**This Project's Configuration:**
- `journal_mode=WAL` + `synchronous=NORMAL` → good concurrency
- `busy_timeout=5000ms` → prevents "database is locked" errors
- `auto_vacuum=INCREMENTAL` → manual vacuum calls in cleanup tasks

---

## 2. serde + serde_json for JSONL Protocol

### Deserialization with `#[serde(default)]`

**Pattern (Used in this project):**
```rust
#[derive(serde::Deserialize)]
pub struct ExternalCommand {
    #[serde(default, rename = "requestId")]
    request_id: Option<ExternalCommandRequestId>,
}
```

**Behavior:**
- `#[serde(default)]` on `Option<T>` field: missing JSON key → `None`, not an error.
- Without `default`: missing key causes deserialization error.
- `#[serde(default = "func")]` for non-Option defaults (e.g., `#[serde(default = "default_value")]`).

**Gotchas:**
- **Explicit None vs Missing**: `{"requestId": null}` and missing key both deserialize to `None`; you can't distinguish them.
- **Override Defaults**: `#[serde(default)]` doesn't apply to fields without `Option<T>`; use `#[serde(default = "literal")]`.

### Handling Unknown Fields

**Strict Mode (Used in this project):**
```rust
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]  // Fail if JSON has extra keys
pub enum ExternalCommand {
    Run { path: String },
}

// {"type": "run", "path": "/x", "extra": "field"} → Error!
```

**Permissive Mode:**
```rust
// No deny_unknown_fields → extra fields silently ignored
```

**Gotchas:**
- **Tag + Content**: With `#[serde(tag = "type", ...)]`, the tag field name is implicit; unknown fields inside variants are still rejected if `deny_unknown_fields` is set.
- **Flattening**: `#[serde(flatten)]` merges fields into outer struct; unknown field detection gets tricky.
- **Version Compatibility**: `deny_unknown_fields` makes forward-incompatible APIs (breaking for old clients seeing new fields). Use permissive mode for APIs that evolve.

**This Project's Pattern:** `deny_unknown_fields` on `ExternalCommand` is strict, good for test protocols where extra keys indicate bugs.

### Enum Serialization (`#[serde(tag)]`)

**Tagged Enum (Used in this project):**
```rust
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ExternalCommand {
    Run { path: String },
    Show {},
}

// Serializes to: {"type": "run", "path": "..."} or {"type": "show"}
```

**Gotchas:**
- **Tag Field Name**: The `tag` is a separate field in JSON; can't have a variant field also named `type`.
- **Content Tag**: `#[serde(tag = "...", content = "...")]` creates `{"type": "variant", "content": {...}}` for more complex cases.
- **Untagged Enum**: `#[serde(untagged)]` tries to disambiguate by trying each variant in order—slower, error messages less helpful.

**This Project's Pattern:**
```rust
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum ExternalCommand {
    Run { path: String },
    Show { request_id: Option<...> },
    Hide { request_id: Option<...> },
    ...
}
// JSON: {"type": "run", "path": "...", ...} ← type is the tag
```

### Common Mistakes with Option<T> Deserialization

**Mistake #1: Confused Semantics**
```rust
// ✗ Ambiguous intent
struct Config {
    name: Option<String>,  // Is this optional in JSON or always present?
}
// Without serde annotation, missing JSON key errors. With #[serde(default)], None.

// ✓ Clear intent
struct Config {
    #[serde(default)]
    name: Option<String>,  // OK to be missing from JSON
}
```

**Mistake #2: Nested Options**
```rust
// ✗ Confusing—what does double-Some/None mean?
struct Metadata {
    optional_optional: Option<Option<String>>,
}

// ✓ Use single Option, or separate boolean
struct Metadata {
    exists: bool,
    value: Option<String>,
}
```

**Gotchas:**
- **Option Without Default**: Field required in JSON; can only be `Some(v)` or explicitly `null`.
- **Option With Default**: Field optional; missing → `None`, `null` → `None`, present → `Some(v)`.

**This Project's Pattern:** Consistent use of `#[serde(default)]` on all optional `request_id` fields.

### String vs &str in Deserialized Structs

**Pattern (This project uses `String`):**
```rust
#[serde(transparent)]
pub struct ExternalCommandRequestId(String);  // Owned, not &'a str

#[serde(deserialize_with = "...")]
pub struct User<'a> {
    name: &'a str,  // Borrowed from input
}
```

**Gotchas:**
- **Borrowed Lifetimes**: `&str` fields require `Deserializer<'de>` where `'de` is the input lifetime. Works only with borrowed deserialization (e.g., `from_slice`, not `from_str` on owned String).
- **Owned Strings**: `String` fields are always allowed; easiest path.
- **Performance**: `&str` avoids heap allocation but requires input persistence.

**This Project's Pattern:** All command types use `String` (owned), making deserialization simpler and safer.

### Flattening Nested Structures

**Pattern:**
```rust
#[serde(flatten)]
struct Inner {
    a: String,
    b: i32,
}

struct Outer {
    name: String,
    #[serde(flatten)]
    inner: Inner,
}

// JSON: {"name": "x", "a": "y", "b": 42}  ← fields from Inner are at top level
```

**Gotchas:**
- **Field Name Conflicts**: If Inner.a and Outer have same field name, deserialization errors.
- **Serialization Asymmetry**: Flattening during ser/deser must match perfectly.
- **Unknown Fields**: With `deny_unknown_fields`, flattened fields are checked in the flattened struct, not outer.

**This Project:** No flattening used; enums with tagged variants are more explicit.

### Error Messages & Debugging

**Serde Error Types:**
```rust
match serde_json::from_str::<ExternalCommand>(json) {
    Ok(cmd) => { /* ... */ }
    Err(e) => {
        eprintln!("JSON error: {}", e);  // "missing field `path` at line 1 column 20"
        eprintln!("Classify: {:?}", e.classify());  // Syntax, EOF, Io, Eof
    }
}
```

**This Project's Pattern:**
```rust
match serde_json::from_str::<ExternalCommand>(trimmed) {
    Ok(cmd) => { /* handle */ }
    Err(e) => {
        tracing::warn!(error = %e, "Failed to parse external command");
    }
}
```

---

## 3. async_channel 2.3 (Bounded)

### send_blocking vs send().await

**send_blocking() - Use from Sync Code:**
```rust
// From a sync thread (not async)
let (tx, rx) = async_channel::bounded(100);
std::thread::spawn(move || {
    tx.send_blocking(msg).ok();  // Blocks until space available
});
```

**send().await - Use from Async Code:**
```rust
async {
    let (tx, rx) = async_channel::bounded(100);
    tx.send(msg).await.ok();  // Async wait for space
}
```

**Gotchas:**
- **send_blocking in Async**: Will panic/deadlock if called inside async runtime! Use only in sync threads.
- **send().await in Sync**: Requires async runtime; won't work in sync code.
- **Both Return Result**: Err means receiver dropped; message not sent.

**This Project's Pattern:**
```rust
// src/stdin_commands/mod.rs
std::thread::spawn(move || {
    let (tx, rx) = async_channel::bounded(100);
    // In sync thread, use send_blocking
    tx.send_blocking(envelope).ok();  // OK if receiver exists
});

// In async code consuming the channel
while let Ok(envelope) = rx.recv().await {
    // Handle message
}
```

### Bounded vs Unbounded Channels

| Aspect | Bounded | Unbounded |
|--------|---------|-----------|
| **Creation** | `async_channel::bounded(cap)` | `async_channel::unbounded()` |
| **Memory** | Fixed capacity; blocks when full | Unbounded; can OOM if sender faster |
| **Latency** | Potential blocking (backpressure) | Always non-blocking |
| **Use Case** | Producer-consumer rate control | Fire-and-forget events |

**Gotchas:**
- **Unbounded Grows Infinitely**: If producer >> consumer, memory explodes. This project correctly uses **bounded** for stdin.
- **Bounded Blocks Sender**: `send_blocking()` waits for space; can cause deadlock if receiver is in same thread.
- **Capacity Tuning**: Too small → frequent blocking; too large → wastes memory.

**This Project's Choice:**
```rust
let (tx, rx) = async_channel::bounded(100);  // ← Generous but bounded
// stdin is slow (< 10 commands/sec), 100 is comfortable headroom
```

### Capacity Sizing Guidelines

**Rule of Thumb:**
- **Stdin/Network**: 10-100 (messages are rare, UI can't react faster than ~10Hz).
- **Internal Events**: 100-1000 (app events, multiple sources).
- **High-Speed IPC**: 1000+ or unbounded (e.g., frame rendering).

**This Project:**
- `stdin_listener`: **100** (external commands, slow)
- `app_watcher`: **10** (file watcher events, bursty)
- `deeplink_channel`: **10** (URL scheme events, rare)

**Gotchas:**
- **Capacity Too Small**: Frequent blocking/dropped messages under burst load.
- **Capacity Too Large**: Hides backpressure; receiver might fall behind.

### Common Deadlock Patterns

**Deadlock #1: Both Sides in Same Thread**
```rust
let (tx, rx) = async_channel::bounded(1);
tx.send_blocking(1).await;  // ← Tries to send, channel full
// Waiting for space, but receiver never runs (same thread, blocked)
// Deadlock!
```

**Prevention:** Spawn sender/receiver in different threads.

**Deadlock #2: Nested Channels**
```rust
let (tx1, rx1) = async_channel::bounded(1);
let (tx2, rx2) = async_channel::bounded(1);

thread1: rx1.recv().await → forward to tx2
thread2: rx2.recv().await → forward to tx1
// If channels fill, threads wait for each other
```

**Prevention:** Size channels adequately; don't create circular message flows.

**Deadlock #3: Sync Lock Held During Async Wait**
```rust
let lock = Mutex::new(data);
let guard = lock.lock();
tx.send(data).await;  // ← Waiting but lock held
// Other thread needs lock to process the message
```

**Prevention:** Drop the lock before awaiting.

**This Project's Pattern:** Avoids these by:
1. Separate thread spawning (stdin listener runs in own thread).
2. Simple channel topology (single producer, single consumer per channel).
3. No nested message forwarding.
4. No locks held across channel operations.

### recv() vs recv_blocking()

**recv().await - From Async:**
```rust
while let Ok(cmd) = rx.recv().await {
    handle(cmd);
}
```

**recv_blocking() - From Sync:**
```rust
std::thread::spawn(move || {
    while let Ok(cmd) = rx.recv_blocking() {
        handle(cmd);
    }
});
```

**Gotchas:**
- **recv_blocking in Async**: May panic or deadlock.
- **recv().await in Sync**: Won't compile; requires async runtime.
- **Channel Closed**: Both return `Err(RecvError)` when all senders dropped.

**This Project:** Uses `rx.recv().await` in async context (app main loop), correctly.

### Message Ordering & Delivery Guarantees

**Guaranteed:**
- FIFO ordering within one channel.
- Each message received by exactly one consumer.

**Not Guaranteed:**
- Real-time ordering across multiple channels.
- Fairness if multiple tasks await the same channel.

**This Project:** Single consumer per channel, so no fairness issues.

---

## 4. tracing 0.1

### Structured Logging with Fields

**Correct Pattern (This project):**
```rust
use tracing::{info, debug, warn, error};

// ✓ Structured fields
info!(
    user_id = %user_id,
    action = "login",
    result = "success",
    "User authentication completed"
);

// ✓ Debug formatting with ?
warn!(
    error = ?err,
    "Operation failed"
);

// ✓ Display formatting with %
debug!(
    path = %file_path,
    "File opened"
);
```

**Gotchas:**
- **String Formatting Anti-Pattern:**
```rust
// ✗ Inefficient—formats string even if trace level disabled
debug!("User {} logged in with id {}", user_name, user_id);

// ✓ Efficient—fields formatted only when enabled
debug!(user_name, user_id, "User logged in");
```
- **Format Specifiers Ignored in Spans**: `span!("msg={x}") ignores the {x} format; use fields instead.

### Span Creation & Entering

**Auto-Instrumentation (Recommended):**
```rust
#[tracing::instrument]
fn process_message(id: u64, msg: &str) {
    // Automatically creates span "process_message" with fields id, msg
    // ...
}
```

**Manual Span Creation:**
```rust
use tracing::span;

let my_span = span!(tracing::Level::INFO, "operation", user_id = 42);
let _guard = my_span.enter();
// Code here is inside the span
// Guard is automatically exited when dropped
```

**Gotchas:**
- **Async/Await Across Spans**: Holding `_guard` across `.await` creates incorrect traces.
```rust
// ✗ Wrong—guard holds span across await
async fn process() {
    let span = span!(Level::INFO, "fetch");
    let _guard = span.enter();
    let data = async_fetch().await;  // ← Guard still held!
}

// ✓ Correct—use Instrument trait
use tracing::Instrument;
async fn process() {
    async_fetch()
        .instrument(span!(Level::INFO, "fetch"))
        .await
}
```
- **Span Nesting**: Spans nest automatically; no manual management needed.

### The #[instrument] Attribute

**Behavior:**
```rust
#[tracing::instrument(level = "debug", skip(password))]
fn login(username: &str, password: &str) -> Result<()> {
    // Creates span "login" with field username (password skipped)
    // Automatically enters span for the function duration
    // Works with sync functions only!
}

#[tracing::instrument(level = "debug")]
async fn async_login(username: &str) -> Result<()> {
    // Also works with async! Instrument handles await correctly.
}
```

**Gotchas:**
- **skip() Arguments**: Parameters not in `skip()` are added as fields; large objects should be skipped.
- **Return Values Not Recorded**: Result is not logged automatically; add explicit logging in error paths.
- **Attribute Position**: Must be on function definition, not in impl block (mostly).

**This Project's Usage:**
```rust
#[tracing::instrument(skip_all)]  // Skip all fields (no data to log)
pub fn start_stdin_listener() -> async_channel::Receiver<...> {
    // ...
}
```

### Common Mistakes with Format Strings

**Mistake #1: Format Strings as Fields**
```rust
// ✗ Inefficient and ugly
info!("Processing user {}", user_id);

// ✓ Structured field
info!(user_id, "Processing user");
```

**Mistake #2: Format Specifiers in Event Messages**
```rust
// ✗ Ignored—message is static, fields are dynamic
info!(x = 5, "Value is {x}");  // {x} doesn't substitute

// ✓ Use message template or field only
info!(x = 5, "Value computed");
```

**Mistake #3: Expensive Computations in Fields**
```rust
// ✗ Expensive lookup even if debug disabled
debug!(computed = expensive_fn(), "Event");

// ✓ Gate expensive fields
if tracing::enabled!(tracing::Level::DEBUG) {
    debug!(computed = expensive_fn(), "Event");
}
```

### Emitter Types: info!, debug!, warn!, error!, trace!

| Macro | Level | Use Case |
|-------|-------|----------|
| `error!()` | ERROR | Unrecoverable errors, crashes |
| `warn!()` | WARN | Recoverable issues, degraded behavior |
| `info!()` | INFO | State changes, important events |
| `debug!()` | DEBUG | Internal details, troubleshooting |
| `trace!()` | TRACE | High-volume, fine-grained tracing |

**This Project's Pattern:**
- `info!()` → entry/exit of major operations, state changes
- `debug!()` → query results, lock events, cache updates
- `warn!()` → lock errors, parse failures, retries
- `error!()` → database failures, unhandled errors
- `trace!()` → Not used (high-frequency overhead)

### span! vs trace_span!, debug_span!, info_span!

**Verbose:**
```rust
let span = tracing::span!(tracing::Level::DEBUG, "operation");
```

**Shorthand:**
```rust
let span = tracing::debug_span!("operation");
let span = tracing::info_span!("operation", user_id = 42);
```

**Gotcha:** Shorthand macros are just convenience; functionally identical.

### Subscriber Installation (Tracing Setup)

**Libraries:**
```rust
// Libraries DON'T install subscribers
// They just emit trace data via macros
// Consumers decide where traces go
pub fn my_lib_fn() {
    info!("Event");  // Emitted, but nobody listening unless app installs subscriber
}
```

**Executables:**
```rust
// main.rs installs a subscriber
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    my_lib_fn();  // Now traces appear on stdout
}
```

**Gotchas:**
- **Multiple init() Calls**: Only first succeeds; subsequent calls ignored.
- **No Subscriber = Silent**: Events are emitted but unobserved (no panic, no error).

**This Project:** Initializes subscriber in `main.rs` (via `logging::setup()`), allowing all modules to emit traces.

### Correlation IDs & Context

**This Project's Pattern:**
```rust
// From logging module
pub fn set_correlation_id(id: String) -> impl Drop {
    // Returns RAII guard that sets correlation ID in current scope
}

// Usage in stdin handler
let correlation_id = format!("stdin:req:{}", request_id);
let _guard = logging::set_correlation_id(correlation_id.clone());
info!(correlation_id = %correlation_id, "Processing command");
// All logs in this scope include correlation_id field
```

**Gotcha:** Correlation IDs must be manually threaded via `tracing::Span::record()` or context scopes. There's no global context by default (use `tracing_log` or task-local storage for that).

---

## Summary: Common Pitfalls for AI Agents

### rusqlite
1. **Always lock before querying** on Arc<Mutex<Connection>>.
2. **Set busy_timeout** to avoid "database is locked" errors.
3. **Use params![] macros** for binding; don't build SQL strings.
4. **Release locks explicitly** before cross-module calls (drop the guard).
5. **Use Result<T> + context()** never unwrap().

### serde_json
1. **Use #[serde(default)]** on Option<T> fields to allow missing JSON keys.
2. **Use deny_unknown_fields** on APIs you control (protocols); use permissive for public APIs.
3. **Use #[serde(tag = "type")]** for strongly-typed enums; avoid untagged.
4. **Test deserialization errors** explicitly—serde errors are unhelpful without context.
5. **Use String not &str** in deserialized structs unless you absolutely need borrowed lifetimes.

### async_channel
1. **Use bounded() not unbounded()** to prevent memory growth.
2. **Capacity ~100 for slow sources** (stdin, network), ~1000+ for high-speed.
3. **Use send_blocking() only in sync threads**, recv().await only in async.
4. **Don't hold locks across send/recv** operations.
5. **Watch for FIFO assumptions**—if you need priority, use multiple channels.

### tracing
1. **Use structured fields**, not format strings in message.
2. **Use #[instrument]** for sync fns, Instrument trait for async.
3. **Don't hold span guards across .await** points.
4. **Skip large parameters** in #[instrument] to avoid overhead.
5. **Libraries emit only; executables install subscriber.**
6. **Use correlation IDs** to tie related events together.

---

## Code Example: Correct Pattern Integration

```rust
// Clipboard history database access pattern
#[tracing::instrument(skip(content), fields(content_len = content.len()))]
pub fn add_entry(content: &str, content_type: ContentType) -> Result<String> {
    // 1. Get connection (Arc<Mutex<>>)
    let conn = get_connection()?;
    let conn = conn
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    // 2. Bind parameters safely with params![]
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp_millis();

    conn.execute(
        "INSERT INTO history (id, content, timestamp) VALUES (?1, ?2, ?3)",
        params![&id, content, timestamp],
    )
    .context("Failed to insert clipboard entry")?;

    // 3. Release lock explicitly before cross-module work
    drop(conn);

    // 4. Use structured logging
    debug!(id = %id, content_type = %content_type.as_str(), "Added entry");

    // 5. Update cache outside lock
    upsert_entry_in_cache(ClipboardEntryMeta { id: id.clone(), ... });

    Ok(id)
}

// Stdin listener using async_channel
pub fn start_stdin_listener() -> async_channel::Receiver<ExternalCommandEnvelope> {
    // 1. Bounded channel (prevent unbounded growth)
    let (tx, rx) = async_channel::bounded(100);

    // 2. Spawn sync thread with send_blocking
    std::thread::spawn(move || {
        for line in reader.lines() {
            // 3. Deserialize with strict schema
            match serde_json::from_str::<ExternalCommand>(line) {
                Ok(cmd) => {
                    // 4. Use correlation ID for tracing
                    let correlation_id = cmd.request_id()
                        .map(|id| format!("stdin:req:{}", id))
                        .unwrap_or_else(|| format!("stdin:{}", Uuid::new_v4()));
                    let _guard = logging::set_correlation_id(correlation_id.clone());

                    info!(correlation_id = %correlation_id, "Processing command");

                    // 5. Send via blocking channel (OK in sync thread)
                    let _ = tx.send_blocking(ExternalCommandEnvelope {
                        command: cmd,
                        correlation_id,
                    });
                }
                Err(e) => {
                    warn!(error = %e, "Failed to parse command");
                }
            }
        }
    });

    rx
}
```

This pattern demonstrates best practices from all four crates working together.
