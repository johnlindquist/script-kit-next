//! Source-level contract pinning the concurrent-close safety invariant for the
//! detached Agent Chat chat window.
//!
//! The detached popup has THREE cleanup sites that can run concurrently against
//! the same `CHAT_WINDOW: OnceLock<Mutex<Option<ChatWindowState>>>`:
//!   1. `open_chat_window`'s `on_window_should_close` (placeholder open path,
//!      src/ai/agent_chat/ui/chat_window.rs)
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

const CHAT_WINDOW_RS: &str = include_str!("../src/ai/agent_chat/ui/chat_window.rs");

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// The three close sites that must each take state out of the CHAT_WINDOW mutex.
const CLOSE_SITE_FNS: [&str; 3] = [
    "pub fn open_chat_window(",
    "pub fn open_chat_window_with_thread(",
    "pub fn close_chat_window(",
];

/// Brace-matched body of the function starting at `signature`.
fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function: {signature}"));
    let rest = &source[start..];
    let open = rest.find('{').expect("function body open brace");
    let mut depth = 0usize;
    for (index, ch) in rest[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &rest[open..open + index + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body for {signature}");
}

#[test]
fn every_close_site_takes_state_out_of_the_mutex() {
    // The `g.take()` pattern is what makes the three close sites race-safe:
    // whichever lock-holder runs first observes Some(state) and does the work;
    // any subsequent lock-holder observes None and becomes a no-op.
    //
    // Each named close site must keep its own `g.take()`:
    // - chat_window_options placeholder on_window_should_close
    // - open_chat_window_with_thread on_window_should_close
    // - close_chat_window helper
    for close_fn in CLOSE_SITE_FNS {
        assert!(
            function_body(CHAT_WINDOW_RS, close_fn).contains("g.take()"),
            "{close_fn} in src/ai/agent_chat/ui/chat_window.rs must call `g.take()` \
             at its CHAT_WINDOW cleanup site to preserve exactly-once cleanup under \
             concurrent close. If you refactored this close site to use `.as_ref()` \
             or `.clone()` instead of `.take()`, both paths could observe the same \
             state and double-run remove_runtime_window_handle / \
             remove_automation_window, corrupting the registry."
        );
    }
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
        "src/ai/agent_chat/ui/chat_window.rs must declare `CHAT_WINDOW` exactly once. \
         Found {} declaration(s). Adding a second OnceLock<Mutex<...>> for \
         the chat window state would silently break the concurrent-close \
         safety contract — every lock-acquisition site must serialize through \
         the same mutex to guarantee exactly-once cleanup.",
        static_declarations
    );

    // Every close site must look up the one-and-only static through
    // `CHAT_WINDOW.get_or_init`; a drift where some site looked up a different
    // static would break mutual exclusion between the close paths.
    for close_fn in CLOSE_SITE_FNS {
        assert!(
            function_body(CHAT_WINDOW_RS, close_fn)
                .contains("CHAT_WINDOW.get_or_init(|| Mutex::new(None))"),
            "{close_fn} in src/ai/agent_chat/ui/chat_window.rs must acquire its lock \
             through the single CHAT_WINDOW static (placeholder on_close, thread \
             on_close, and close_chat_window must all serialize through the same \
             mutex)."
        );
    }
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
        assert!(
            !CHAT_WINDOW_RS.contains(pattern),
            "src/ai/agent_chat/ui/chat_window.rs contains `{}` — cloning ChatWindowState \
             out of the mutex breaks exactly-once cleanup because a concurrent \
             close observing the clone would double-run remove_runtime_window_handle \
             / remove_automation_window on the same id. Use `g.take()` instead so \
             the loser of the race observes None and no-ops.",
            pattern
        );
    }
}
