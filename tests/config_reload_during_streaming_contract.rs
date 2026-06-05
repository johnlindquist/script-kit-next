//! Source-level contract test for the `config-reload-during-streaming` user
//! story.
//!
//! The story wants two guarantees at once:
//!
//! 1. Config change is picked up — when the user edits `config.ts` (the
//!    Script Kit runtime config; the story's "`.kit/config.json` or
//!    equivalent"), subsequent `load_config()` callers see the updated
//!    value without restarting the app.
//!
//! 2. In-flight ACP stream continues to completion without interruption —
//!    a config edit while an ACP turn is streaming MUST NOT tear down the
//!    running agent subprocess, re-spawn bun, or cancel the stream.
//!
//! These two requirements sound contradictory, but the codebase splits them
//! cleanly: the **agent-side** `AcpAgentConfig` is frozen per-process (so a
//! mid-stream edit cannot disturb the running subprocess) while the
//! **Script Kit-side** `load_config()` re-fingerprints `config.ts` on every
//! call (so the next ACP open / next config read picks up the edit).
//!
//! Specifically:
//!
//! - `CACHED_AGENT_CONFIG: OnceLock<AcpAgentConfig>` in
//!   `src/ai/acp/config.rs` is a one-shot cell — primed by
//!   `prewarm_agent_config()` at startup or by the first ACP open, and
//!   never invalidated afterwards. That is what keeps an in-flight stream
//!   stable: the subprocess was spawned with args derived from the cached
//!   `AcpAgentConfig`, and the hot path re-entering
//!   `claude_code_agent_config_cached()` short-circuits on the cached
//!   value — no new bun subprocess, no filesystem re-read.
//!
//! - `load_config()` in `src/config/loader.rs` holds no process-global
//!   `OnceLock<Config>`; instead it computes a file fingerprint (len +
//!   mtime_ms) on every call and looks up a disk-keyed cache
//!   (`try_load_cached_config`). A fingerprint change invalidates the
//!   cache and re-runs bun; same fingerprint serves instantly from disk.
//!   This is the "picked up on next call" half of the story.
//!
//! The two halves compose: the one-shot `OnceLock<AcpAgentConfig>`
//! protects in-flight streams from churn, and the fingerprint-keyed disk
//! cache in `load_config()` ensures the next read sees the edit. If either
//! primitive regresses — the agent cache growing an invalidation path, or
//! `load_config()` sprouting a process-global OnceLock — the story's
//! contract fractures.
//!
//! This test pins both primitives so a future refactor that "helps" by
//! adding reload semantics in the wrong place fails loudly in CI.

const ACP_CONFIG_SOURCE: &str = include_str!("../src/ai/acp/config.rs");
const LOADER_SOURCE: &str = include_str!("../src/config/loader.rs");

#[test]
fn acp_agent_config_is_cached_per_process_via_oncelock() {
    assert!(
        ACP_CONFIG_SOURCE
            .contains("static CACHED_AGENT_CONFIG: OnceLock<AcpAgentConfig> = OnceLock::new();"),
        "CACHED_AGENT_CONFIG must remain a one-shot OnceLock<AcpAgentConfig> \
         at module scope — this is the primitive that keeps an in-flight \
         ACP stream insulated from mid-stream config edits. If this \
         regresses into a Mutex<Option<...>> or gains an invalidation \
         path (e.g., `CACHED_AGENT_CONFIG.take()` or a clear fn), a config \
         edit during streaming could race with a re-read and yield \
         inconsistent agent args across the session."
    );
}

#[test]
fn cached_agent_config_hot_path_short_circuits_on_cached_value() {
    assert!(
        ACP_CONFIG_SOURCE.contains(
            "pub(crate) fn claude_code_agent_config_cached() -> anyhow::Result<AcpAgentConfig> {"
        ),
        "claude_code_agent_config_cached must remain the single hot-path \
         accessor for ACP agent config — callers in the streaming \
         pipeline rely on its short-circuit semantics to avoid spawning \
         bun mid-turn"
    );
    assert!(
        ACP_CONFIG_SOURCE.contains("if let Some(cached) = CACHED_AGENT_CONFIG.get() {")
            && ACP_CONFIG_SOURCE.contains("return Ok(cached.clone());"),
        "claude_code_agent_config_cached must early-return with a clone of \
         the cached value when the OnceLock is populated — without this \
         short-circuit, a mid-stream re-read would spawn bun and \
         potentially block the streaming thread"
    );
}

#[test]
fn agent_config_cache_has_no_invalidation_path() {
    for forbidden in [
        "CACHED_AGENT_CONFIG.take()",
        "CACHED_AGENT_CONFIG.replace(",
        "fn clear_cached_agent_config",
        "fn invalidate_agent_config",
        "fn reload_agent_config",
    ] {
        assert!(
            !ACP_CONFIG_SOURCE.contains(forbidden),
            "`{forbidden}` must not appear in src/ai/acp/config.rs — the \
             cached agent config is deliberately one-shot so in-flight \
             streams are shielded from mid-stream config churn. Adding \
             an invalidation path undoes that contract and can cause a \
             live stream to observe args that drift away from what its \
             subprocess was spawned with."
        );
    }
}

#[test]
fn prewarm_primes_the_agent_config_cache_on_startup() {
    assert!(
        ACP_CONFIG_SOURCE.contains("pub(crate) fn prewarm_agent_config() {"),
        "prewarm_agent_config must remain the public startup prewarmer — \
         it pays the ~100-500ms bun transpile cost off the main thread \
         so the first ACP open does not block, which matters here \
         because the prewarmed value is what the in-flight stream will \
         be insulated by"
    );
    assert!(
        ACP_CONFIG_SOURCE.contains("let _ = CACHED_AGENT_CONFIG.set(config);"),
        "prewarm_agent_config must populate CACHED_AGENT_CONFIG via `.set` \
         — if it stops populating the OnceLock, the first streaming \
         request pays the bun cost synchronously AND the in-flight \
         stream protection hinges on whatever path primes the cache \
         instead (racy)"
    );
}

#[test]
fn load_config_holds_no_process_global_oncelock_of_config() {
    for forbidden in [
        "static CACHED_CONFIG: OnceLock<Config>",
        "static CONFIG_CACHE: OnceLock<Config>",
        "static LOADED_CONFIG: OnceLock<Config>",
        "static CONFIG: OnceLock<Config>",
        "Lazy::new(|| Mutex::new(Config",
        "LazyLock::new(|| Config",
    ] {
        assert!(
            !LOADER_SOURCE.contains(forbidden),
            "`{forbidden}` must not appear in src/config/loader.rs — \
             load_config() is required to re-read the config source on \
             every call (via fingerprint + disk cache), so a post-edit \
             caller picks up the new value. A process-global OnceLock of \
             Config would freeze the Script Kit-side config for the \
             process lifetime, breaking the story's \"picked up\" \
             requirement."
        );
    }
}

#[test]
fn load_config_uses_file_fingerprint_for_disk_cache_lookup() {
    for needed in [
        "fn fingerprint_config_file(path: &Path) -> Option<ConfigSourceFingerprint> {",
        "fn try_load_cached_config(",
        "let fingerprint = fingerprint_config_file(&config_path);",
        "if let Some(config) = try_load_cached_config(&config_path, fp, &correlation_id) {",
    ] {
        assert!(
            LOADER_SOURCE.contains(needed),
            "src/config/loader.rs must retain `{needed}` — the \
             fingerprint-keyed disk cache is the mechanism that lets \
             load_config() detect a config.ts edit and re-run bun, while \
             still serving unchanged config instantly. If the fingerprint \
             machinery is removed, either every call spawns bun (slow) \
             or every call returns stale data (broken story)."
        );
    }
    assert!(
        LOADER_SOURCE.contains("len: metadata.len(),") && LOADER_SOURCE.contains("modified_ms,"),
        "ConfigSourceFingerprint must encode both `len` and `modified_ms` \
         — a single-axis fingerprint (e.g., mtime only) misses the \
         narrow but real edge where an edit preserves mtime (some editors \
         restore timestamps); two axes widen the change-detection window"
    );
}
