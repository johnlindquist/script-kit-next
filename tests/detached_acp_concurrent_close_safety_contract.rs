//! Source-level contract pinning the concurrent-close safety invariant for the
//! detached ACP chat window.
//!
//! The detached popup has THREE cleanup sites that can run concurrently against
//! the same `CHAT_WINDOW: OnceLock<Mutex<Option<ChatWindowState>>>`:
//!   1. `chat_window_options`'s `on_window_should_close` (placeholder open path,
//!      src/ai/acp/chat_window.rs ~line 116)
//!   2. `open_chat_window_with_thread`'s `on_window_should_close` (thread open
//!      path, ~line 184)
//!   3. `close_chat_window` helper (external TriggerAction path, ~line 525)
//!
//! The rust-level safety guarantee is that the slot is a single Mutex; whichever
//! path wins the race locks first. The FUNCTIONAL safety guarantee — that
//! cleanup runs EXACTLY ONCE across both paths — requires every site to use the
//! `slot.lock() + g.take()` pattern so that:
//!   - Winner: receives `Some(state)`, runs full cleanup.
//!   - Loser: receives `None` from `.take()`, becomes a no-op.
//!
//! A future refactor that replaced `.take()` with `.as_ref().cloned()` or
//! equivalent would break the exactly-once guarantee: both paths could observe
//! `Some(state)` and double-run `remove_runtime_window_handle` /
//! `remove_automation_window` on the same id, corrupting the registry.
//!
//! Pass #48 of the AFK audit (Run 2) pins this invariant so the three close
//! sites cannot silently drift apart.

const CHAT_WINDOW_RS: &str = include_str!("../src/ai/acp/chat_window.rs");

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

#[test]
fn every_close_site_takes_state_out_of_the_mutex() {
    // The `g.take()` pattern is what makes the three close sites race-safe:
    // whichever lock-holder runs first observes Some(state) and does the work;
    // any subsequent lock-holder observes None and becomes a no-op.
    //
    // Expected count: 3 (one per close site).
    // - chat_window_options placeholder on_window_should_close
    // - open_chat_window_with_thread on_window_should_close
    // - close_chat_window helper
    let take_calls = count_occurrences(CHAT_WINDOW_RS, "g.take()");
    assert!(
        take_calls >= 3,
        "src/ai/acp/chat_window.rs must call `g.take()` at every CHAT_WINDOW \
         cleanup site to preserve exactly-once cleanup under concurrent close. \
         Found {} occurrence(s); expected at least 3 (placeholder on_close, \
         thread on_close, close_chat_window helper). If you refactored a close \
         site to use `.as_ref()` or `.clone()` instead of `.take()`, both \
         paths could observe the same state and double-run \
         remove_runtime_window_handle / remove_automation_window, corrupting \
         the registry.",
        take_calls
    );
}

#[test]
fn close_sites_use_the_same_chat_window_mutex() {
    // All three sites must lock the SAME static. A refactor that introduced a
    // second OnceLock<Mutex<...>> would appear to work in isolation but would
    // break the race guarantee (different mutexes = no mutual exclusion).
    let static_declarations = count_occurrences(
        CHAT_WINDOW_RS,
        "static CHAT_WINDOW: OnceLock<Mutex<Option<ChatWindowState>>>",
    );
    assert_eq!(
        static_declarations, 1,
        "src/ai/acp/chat_window.rs must declare `CHAT_WINDOW` exactly once. \
         Found {} declaration(s). Adding a second OnceLock<Mutex<...>> for \
         the chat window state would silently break the concurrent-close \
         safety contract — every lock-acquisition site must serialize through \
         the same mutex to guarantee exactly-once cleanup.",
        static_declarations
    );

    // Every `slot.lock()` site must be preceded by a `CHAT_WINDOW.get_or_init`
    // that points at the one-and-only static. We count `CHAT_WINDOW.get_or_init`
    // occurrences as a lower bound on lock sites; a drift where some sites
    // looked up a different static would reduce this count relative to
    // `slot.lock().` occurrences.
    let get_or_init_sites = count_occurrences(
        CHAT_WINDOW_RS,
        "CHAT_WINDOW.get_or_init(|| Mutex::new(None))",
    );
    assert!(
        get_or_init_sites >= 3,
        "src/ai/acp/chat_window.rs must initialize the CHAT_WINDOW Mutex at \
         every close/cleanup site. Found {} occurrence(s); expected at least \
         3 for the three close paths (placeholder on_close, thread on_close, \
         close_chat_window helper).",
        get_or_init_sites
    );
}

#[test]
fn no_close_site_uses_non_take_extraction() {
    // A drive-by refactor might replace `g.take()` with `g.as_ref().cloned()`
    // "because cloning feels safer" — but cloning breaks exactly-once. This
    // assertion catches any non-`take` extraction in a close-adjacent context.
    //
    // Look for the sentinel patterns that would break the invariant. These
    // patterns are rare enough in this file that any occurrence is suspicious
    // and worth manual review.
    // Anchor `g` as a standalone guard binding (preceded by whitespace) so we
    // don't false-positive on `dialog.clone()`, `config.clone()`, etc.
    let forbidden_patterns = [
        " g.as_ref().cloned()",
        " g.clone()",
        " (*g).clone()",
        "\tg.as_ref().cloned()",
        "\tg.clone()",
    ];
    for pattern in forbidden_patterns {
        let count = count_occurrences(CHAT_WINDOW_RS, pattern);
        assert_eq!(
            count, 0,
            "src/ai/acp/chat_window.rs contains `{}` — cloning ChatWindowState \
             out of the mutex breaks exactly-once cleanup because a concurrent \
             close observing the clone would double-run remove_runtime_window_handle \
             / remove_automation_window on the same id. Use `g.take()` instead so \
             the loser of the race observes None and no-ops.",
            pattern
        );
    }
}
