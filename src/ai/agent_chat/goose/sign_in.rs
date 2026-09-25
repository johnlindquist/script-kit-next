//! Browser sign-in for subscription providers (ChatGPT, Gemini, xAI).
//!
//! goose runs these providers' OAuth login itself the first time a request
//! needs a token: it opens the login page in the default browser, waits for
//! the redirect on a fixed localhost port, and stores the result at
//! `<goose root>/config/<provider>/tokens.json`.
//!
//! This module owns what Script Kit knows about that state:
//! - whether a provider is signed in (cached, so render paths never stat),
//! - a single-flight gate, because every goose process would otherwise open
//!   its own browser tab and race for the same callback port,
//! - forgetting a sign-in for Switch Account.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

/// goose providers that sign in through an interactive browser login.
const BROWSER_SIGN_IN_PROVIDERS: [&str; 3] = ["chatgpt_codex", "gemini_oauth", "xai_oauth"];

/// goose waits five minutes for the browser redirect; wait slightly longer so
/// a follower never gives up before the leader's login can finish.
pub(crate) const SIGN_IN_WAIT: Duration = Duration::from_secs(310);

pub(crate) fn is_browser_sign_in_provider(provider: &str) -> bool {
    BROWSER_SIGN_IN_PROVIDERS.contains(&provider)
}

/// The user-facing name of the account a provider signs in to.
pub(crate) fn provider_display_name(provider: &str) -> &'static str {
    match provider {
        "chatgpt_codex" => "ChatGPT",
        "gemini_oauth" => "Google",
        "xai_oauth" => "xAI",
        _ => "your AI provider",
    }
}

pub(crate) fn tokens_path(goose_root: &Path, provider: &str) -> PathBuf {
    goose_root.join("config").join(provider).join("tokens.json")
}

fn default_goose_root() -> &'static Path {
    static ROOT: LazyLock<PathBuf> =
        LazyLock::new(|| super::launch_spec::goose_root(&crate::setup::get_kit_path()));
    ROOT.as_path()
}

/// Cached sign-in state: goose root → provider → signed in. The first query
/// for a provider stats its token file; later queries (render paths) are
/// allocation-free map lookups.
static SIGNED_IN: LazyLock<Mutex<HashMap<PathBuf, HashMap<String, bool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Providers with a browser login in flight, per goose root.
static IN_FLIGHT: LazyLock<Mutex<HashSet<(PathBuf, String)>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn key(goose_root: &Path, provider: &str) -> (PathBuf, String) {
    (goose_root.to_path_buf(), provider.to_string())
}

fn record(goose_root: &Path, provider: &str, signed_in: bool) {
    SIGNED_IN
        .lock()
        .entry(goose_root.to_path_buf())
        .or_default()
        .insert(provider.to_string(), signed_in);
}

/// Whether `provider` still needs a browser sign-in under `goose_root`.
/// Always `false` for providers that do not sign in through a browser.
pub(crate) fn needs_sign_in_at(goose_root: &Path, provider: &str) -> bool {
    if !is_browser_sign_in_provider(provider) {
        return false;
    }
    let cached = SIGNED_IN
        .lock()
        .get(goose_root)
        .and_then(|providers| providers.get(provider).copied());
    let signed_in = match cached {
        Some(signed_in) => signed_in,
        None => refresh_at(goose_root, provider),
    };
    !signed_in
}

/// [`needs_sign_in_at`] for Script Kit's own goose root.
pub(crate) fn needs_sign_in(provider: &str) -> bool {
    needs_sign_in_at(default_goose_root(), provider)
}

/// Re-read the token file, bypassing the cache. Returns whether signed in.
pub(crate) fn refresh_at(goose_root: &Path, provider: &str) -> bool {
    let signed_in = tokens_path(goose_root, provider).is_file();
    record(goose_root, provider, signed_in);
    signed_in
}

/// Forget the stored sign-in so the next request signs in again.
pub(crate) fn forget(provider: &str) -> std::io::Result<()> {
    let root = default_goose_root();
    match std::fs::remove_file(tokens_path(root, provider)) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    record(root, provider, false);
    Ok(())
}

/// Whether a browser sign-in for `provider` is in flight in this process.
pub(crate) fn in_flight(goose_root: &Path, provider: &str) -> bool {
    IN_FLIGHT.lock().contains(&key(goose_root, provider))
}

/// Who performs the login for a turn that needs one.
#[derive(Debug)]
pub(crate) enum SignInRole {
    /// This turn lets goose open the browser. Dropping the lease ends the
    /// in-flight marker whether or not the login succeeded.
    Leader(LeaderLease),
    /// Another turn is already signing in; wait for its result.
    Follower,
}

#[derive(Debug)]
pub(crate) struct LeaderLease {
    key: (PathBuf, String),
}

impl Drop for LeaderLease {
    fn drop(&mut self) {
        IN_FLIGHT.lock().remove(&self.key);
    }
}

pub(crate) fn begin(goose_root: &Path, provider: &str) -> SignInRole {
    let key = key(goose_root, provider);
    let mut in_flight = IN_FLIGHT.lock();
    if in_flight.contains(&key) {
        SignInRole::Follower
    } else {
        in_flight.insert(key.clone());
        SignInRole::Leader(LeaderLease { key })
    }
}

/// Wait until `provider` is signed in, or until no login is in flight and
/// none succeeded, or until `timeout`. Returns whether sign-in completed.
pub(crate) async fn wait_for_sign_in(goose_root: &Path, provider: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let key = key(goose_root, provider);
    loop {
        if refresh_at(goose_root, provider) {
            return true;
        }
        let leader_active = IN_FLIGHT.lock().contains(&key);
        if !leader_active || Instant::now() >= deadline {
            // One last look: the leader may have finished between checks.
            return refresh_at(goose_root, provider);
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_browser_providers_without_tokens_need_sign_in() {
        let temp = tempfile::tempdir().unwrap();
        assert!(needs_sign_in_at(temp.path(), "chatgpt_codex"));
        assert!(!needs_sign_in_at(temp.path(), "ollama"));
        assert!(!needs_sign_in_at(temp.path(), "anthropic"));

        let path = tokens_path(temp.path(), "chatgpt_codex");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "{}").unwrap();
        // The cache holds until the token file is re-read.
        assert!(needs_sign_in_at(temp.path(), "chatgpt_codex"));
        assert!(refresh_at(temp.path(), "chatgpt_codex"));
        assert!(!needs_sign_in_at(temp.path(), "chatgpt_codex"));
    }

    #[test]
    fn only_one_turn_leads_a_sign_in_until_its_lease_drops() {
        let temp = tempfile::tempdir().unwrap();
        let lease = begin(temp.path(), "chatgpt_codex");
        assert!(matches!(lease, SignInRole::Leader(_)));
        assert!(matches!(
            begin(temp.path(), "chatgpt_codex"),
            SignInRole::Follower
        ));
        assert!(matches!(
            begin(temp.path(), "xai_oauth"),
            SignInRole::Leader(_)
        ));
        drop(lease);
        assert!(matches!(
            begin(temp.path(), "chatgpt_codex"),
            SignInRole::Leader(_)
        ));
    }

    #[test]
    fn followers_finish_when_the_leader_signs_in_or_gives_up() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(followers_scenario());
    }

    async fn followers_scenario() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        // Leader gives up without tokens: the follower stops waiting.
        let lease = begin(&root, "chatgpt_codex");
        drop(lease);
        assert!(!wait_for_sign_in(&root, "chatgpt_codex", Duration::from_secs(5)).await);

        // Leader succeeds: the follower sees the stored sign-in.
        let lease = begin(&root, "chatgpt_codex");
        let writer_root = root.clone();
        let writer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let path = tokens_path(&writer_root, "chatgpt_codex");
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, "{}").unwrap();
            drop(lease);
        });
        assert!(wait_for_sign_in(&root, "chatgpt_codex", Duration::from_secs(5)).await);
        writer.await.unwrap();
    }
}
