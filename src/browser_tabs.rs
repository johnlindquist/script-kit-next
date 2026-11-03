use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const BROWSER_TIMEOUT: Duration = Duration::from_secs(3);
const ROOT_BROWSER_TABS_MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const ROOT_BROWSER_TABS_FAILURE_BACKOFF_BASE: Duration = Duration::from_secs(15);
const ROOT_BROWSER_TABS_FAILURE_BACKOFF_MAX: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BrowserQueryDiagnostic {
    bytes: usize,
    chars: usize,
}

fn browser_query_diagnostic(query: &str) -> BrowserQueryDiagnostic {
    BrowserQueryDiagnostic {
        bytes: query.len(),
        chars: query.chars().count(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserFamily {
    Safari,
    Chromium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SupportedBrowser {
    app_name: &'static str,
    bundle_id: &'static str,
    family: BrowserFamily,
}

const SUPPORTED_BROWSERS: &[SupportedBrowser] = &[
    SupportedBrowser {
        app_name: "Safari",
        bundle_id: "com.apple.Safari",
        family: BrowserFamily::Safari,
    },
    SupportedBrowser {
        app_name: "Google Chrome",
        bundle_id: "com.google.Chrome",
        family: BrowserFamily::Chromium,
    },
    SupportedBrowser {
        app_name: "Arc",
        bundle_id: "company.thebrowser.Browser",
        family: BrowserFamily::Chromium,
    },
    SupportedBrowser {
        app_name: "Brave Browser",
        bundle_id: "com.brave.Browser",
        family: BrowserFamily::Chromium,
    },
    SupportedBrowser {
        app_name: "Microsoft Edge",
        bundle_id: "com.microsoft.edgemac",
        family: BrowserFamily::Chromium,
    },
    SupportedBrowser {
        app_name: "Chromium",
        bundle_id: "org.chromium.Chromium",
        family: BrowserFamily::Chromium,
    },
    SupportedBrowser {
        app_name: "Vivaldi",
        bundle_id: "com.vivaldi.Vivaldi",
        family: BrowserFamily::Chromium,
    },
    SupportedBrowser {
        app_name: "Opera",
        bundle_id: "com.operasoftware.Opera",
        family: BrowserFamily::Chromium,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BrowserTabIdentity {
    pub pid: i32,
    pub window: Arc<str>,
    pub tab: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BrowserTabInfo {
    pub browser_name: Arc<str>,
    pub browser_bundle_id: Arc<str>,
    pub window_index: usize,
    pub tab_index: usize,
    #[serde(default)]
    pub identity: Option<BrowserTabIdentity>,
    pub title: Arc<str>,
    pub url: Arc<str>,
}

impl BrowserTabInfo {
    pub fn display_title(&self) -> &str {
        if !self.title.trim().is_empty() {
            self.title.trim()
        } else if !self.url.trim().is_empty() {
            self.url.trim()
        } else {
            "Untitled Tab"
        }
    }
}

pub(crate) fn browser_tab_stable_key(tab: &BrowserTabInfo) -> String {
    if let Some(id) = &tab.identity {
        // Reordering is not a new target. Navigation must not inherit the previous selection.
        let mut hash = Sha256::new();
        for part in [
            id.window.as_bytes(),
            id.tab.as_deref().unwrap_or("").as_bytes(),
            tab.title.as_bytes(),
            tab.url.as_bytes(),
        ] {
            hash.update((part.len() as u64).to_be_bytes());
            hash.update(part);
        }
        return format!(
            "{}{}/{}/{:x}",
            sk_one_window::contract::OPEN_TAB_KEY_PREFIX,
            tab.browser_bundle_id,
            id.pid,
            hash.finalize()
        );
    }
    // Legacy snapshots remain searchable but carry no native focus authority.
    format!(
        "{}{}/{}/{}/{}",
        sk_one_window::contract::OPEN_TAB_KEY_PREFIX,
        tab.browser_bundle_id,
        tab.window_index,
        tab.tab_index,
        tab.url
    )
}

pub(crate) fn browser_tab_host(tab: &BrowserTabInfo) -> String {
    host_from_url(&tab.url).to_string()
}

/// Returns whether the query is a single hostname-like token, optionally
/// wrapped in an HTTP(S) scheme and/or one trailing slash.
pub(crate) fn query_is_bare_domain(query: &str) -> bool {
    let mut candidate = query.trim();
    if candidate.is_empty() || candidate.chars().any(char::is_whitespace) {
        return false;
    }

    if candidate
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
    {
        candidate = &candidate[8..];
    } else if candidate
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
    {
        candidate = &candidate[7..];
    }
    candidate = candidate.strip_suffix('/').unwrap_or(candidate);

    let labels = candidate.split('.').collect::<Vec<_>>();
    if labels.len() < 2 {
        return false;
    }
    if !labels.iter().all(|label| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    }) {
        return false;
    }

    labels
        .last()
        .is_some_and(|tld| tld.len() >= 2 && tld.bytes().all(|byte| byte.is_ascii_alphabetic()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTabMatch {
    pub tab: BrowserTabInfo,
    pub score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct RootBrowserTabsSectionOptions {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
    pub scan_limit: usize,
    pub search_urls: bool,
    pub providers: Vec<crate::config::BrowserTabProvider>,
    pub cache_ttl_ms: u64,
}

impl Default for RootBrowserTabsSectionOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            max_results: crate::config::defaults::DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_MAX_RESULTS,
            min_query_chars:
                crate::config::defaults::DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_MIN_QUERY_CHARS,
            scan_limit: usize::MAX,
            search_urls: crate::config::defaults::DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_SEARCH_URLS,
            providers: crate::config::BrowserTabProvider::default_root_providers(),
            cache_ttl_ms: crate::config::defaults::DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_CACHE_TTL_MS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct RootPassiveSnapshotStatus {
    pub generation: u64,
    pub refreshing: bool,
    pub cached_count: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RootBrowserTabSearchHit {
    pub stable_key: String,
    pub url: String,
    pub provider_label: String,
    pub tab: BrowserTabInfo,
    pub title: String,
    pub domain: String,
    pub score: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RootBrowserTabSnapshot {
    captured_at: Instant,
    tabs: Arc<Vec<BrowserTabInfo>>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct RootBrowserTabSnapshotState {
    snapshot: Option<RootBrowserTabSnapshot>,
    refresh_in_flight: bool,
    refresh_needed: bool,
    generation: u64,
    last_refresh_error: Option<String>,
    last_attempt_at: Option<Instant>,
    last_success_at: Option<Instant>,
    next_refresh_after: Option<Instant>,
    failure_count: u32,
}

#[allow(dead_code)]
static ROOT_BROWSER_TAB_SNAPSHOT: LazyLock<Mutex<RootBrowserTabSnapshotState>> =
    LazyLock::new(|| Mutex::new(RootBrowserTabSnapshotState::default()));

#[derive(Debug, Clone, Copy)]
pub(crate) struct RootBrowserTabsRefresh {
    generation: u64,
    started_at: Instant,
}

pub fn list_open_tabs() -> Result<Vec<BrowserTabInfo>> {
    list_open_tabs_for_root_providers(&[])
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BrowserTabsTestProvider {
    Rows(Vec<BrowserTabInfo>),
    Config {
        #[serde(default, rename = "delayMs")]
        delay_ms: u64,
        #[serde(default)]
        fail: bool,
        #[serde(default)]
        error: Option<String>,
        #[serde(default)]
        tabs: Vec<BrowserTabInfo>,
    },
}

fn list_open_tabs_for_root_providers(
    providers: &[crate::config::BrowserTabProvider],
) -> Result<Vec<BrowserTabInfo>> {
    let _span = tracing::info_span!("list_open_tabs").entered();

    if let Ok(raw) = std::env::var("SCRIPT_KIT_BROWSER_TABS_TEST_PROVIDER") {
        let provider: BrowserTabsTestProvider =
            serde_json::from_str(&raw).context("parse SCRIPT_KIT_BROWSER_TABS_TEST_PROVIDER")?;
        return match provider {
            BrowserTabsTestProvider::Rows(rows) => Ok(rows),
            BrowserTabsTestProvider::Config {
                delay_ms,
                fail,
                error,
                tabs,
            } => {
                if delay_ms > 0 {
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
                if fail {
                    bail!(
                        "{}",
                        error.unwrap_or_else(|| "browser tabs test provider failure".to_string())
                    );
                }
                Ok(tabs)
            }
        };
    }

    let running = running_bundle_ids()?;
    let results = running_tab_browsers(&running, providers)
        .into_par_iter()
        .map(|browser| {
            let tabs = list_tabs_for_browser(browser);
            match &tabs {
                Ok(rows) => tracing::info!(
                    browser = browser.app_name,
                    tab_count = rows.len(),
                    "list_tabs_for_browser_completed"
                ),
                Err(error) => tracing::warn!(
                    browser = browser.app_name,
                    // The outer context identifies the failed operation; the inner
                    // transport error can include private browser output.
                    operation = %error,
                    "list_tabs_for_browser_failed"
                ),
            }
            tabs
        })
        .collect();

    combine_browser_tab_results(results)
}

fn combine_browser_tab_results(
    results: Vec<Result<Vec<BrowserTabInfo>>>,
) -> Result<Vec<BrowserTabInfo>> {
    let mut tabs = Vec::new();
    for result in results {
        tabs.append(&mut result?);
    }
    Ok(tabs)
}

pub(crate) fn named_tab_browser(name: &str) -> Option<&'static str> {
    SUPPORTED_BROWSERS
        .iter()
        .find(|b| b.app_name.eq_ignore_ascii_case(name) || b.bundle_id.eq_ignore_ascii_case(name))
        .map(|b| b.app_name)
}

pub(crate) fn named_tab_program(app: &str, query: &str) -> Option<String> {
    let browser = SUPPORTED_BROWSERS.iter().find(|b| b.app_name == app)?;
    snapshot_program(browser, TabRequest::Named(query)).ok()
}

#[allow(
    dead_code,
    reason = "the binary intent launcher dispatches named tab focus"
)]
pub(crate) fn focus_named_tab(app: &str, query: &str) -> Result<()> {
    anyhow::ensure!(
        !crate::runtime_policy::is_owned_evaluation(),
        "native tab activation unavailable in owned evaluation"
    );
    let script = named_tab_program(app, query).context("Unsupported browser")?;
    let result =
        crate::platform::run_jxa_with_timeout(&script, "focus_named_browser_tab", BROWSER_TIMEOUT)
            .with_context(|| {
                format!("Couldn't verify the tab in {app} — check Automation access")
            })?;
    match result.trim() {
        "focused" => Ok(()),
        "not-running" => bail!("{app} isn't running"),
        "not-found" => bail!("No {app} tab matches “{query}”"),
        "ambiguous" => bail!("Several {app} tabs match “{query}” — use a more specific title"),
        "changed" => bail!("The browser tabs changed; couldn't confirm the requested tab"),
        "unconfirmed" => bail!("Couldn't confirm that {app} came to the front"),
        _ => bail!("Couldn't verify the selected tab in {app}"),
    }
}

pub fn activate_tab(tab: &BrowserTabInfo) -> Result<()> {
    let browser = SUPPORTED_BROWSERS
        .iter()
        .find(|b| b.bundle_id == tab.browser_bundle_id.as_ref())
        .context("Unsupported browser")?;
    anyhow::ensure!(
        tab.identity.is_some(),
        "Tab identity is unavailable; refresh the list"
    );
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::NativeInput)?;
    let script = snapshot_program(browser, TabRequest::Selected(tab))?;
    let result =
        crate::platform::run_jxa_with_timeout(&script, "browser_tabs_focus_exact", BROWSER_TIMEOUT)
            .map_err(|_| anyhow!("Could not verify the browser tab; check Automation access"))?;
    match result.trim() {
        "focused" => Ok(()),
        "ambiguous" => bail!("Several identical tabs are open in the selected window"),
        "not-running" => bail!("The selected browser is no longer running"),
        "unconfirmed" => bail!("Couldn't confirm that the selected browser came to the front"),
        _ => bail!("The selected tab changed; refresh the list"),
    }
}

pub fn fuzzy_search_browser_tabs(tabs: &[BrowserTabInfo], query: &str) -> Vec<BrowserTabMatch> {
    let query_diagnostic = browser_query_diagnostic(query);
    let _span = tracing::info_span!(
        "fuzzy_search_browser_tabs",
        tab_count = tabs.len(),
        query_bytes = query_diagnostic.bytes,
        query_chars = query_diagnostic.chars,
    )
    .entered();

    if query.trim().is_empty() {
        return tabs
            .iter()
            .cloned()
            .map(|tab| BrowserTabMatch { tab, score: 0 })
            .collect();
    }

    let query_lower = query.trim().to_lowercase();
    let query_is_ascii = query_lower.is_ascii();
    let use_nucleo = query_lower.len() >= crate::scripts::search::MIN_FUZZY_QUERY_LEN;

    let mut matches: Vec<BrowserTabMatch> = tabs
        .par_chunks(128)
        .flat_map(|chunk| {
            let mut nucleo = if use_nucleo {
                Some(crate::scripts::NucleoCtx::new(&query_lower))
            } else {
                None
            };

            chunk
                .iter()
                .filter_map(|tab| {
                    let mut score = 0i32;
                    let host = host_from_url(&tab.url);

                    if query_is_ascii && tab.title.is_ascii() {
                        if let Some(pos) =
                            crate::scripts::search::find_ignore_ascii_case(&tab.title, &query_lower)
                        {
                            score += if pos == 0 { 240 } else { 190 };
                        }
                    }

                    if query_is_ascii && host.is_ascii() {
                        if let Some(pos) =
                            crate::scripts::search::find_ignore_ascii_case(host, &query_lower)
                        {
                            score += if pos == 0 { 180 } else { 135 };
                        }
                    }

                    if query_is_ascii && tab.url.is_ascii() {
                        if let Some(pos) =
                            crate::scripts::search::find_ignore_ascii_case(&tab.url, &query_lower)
                        {
                            score += if pos == 0 { 120 } else { 90 };
                        }
                    }

                    if query_is_ascii && tab.browser_name.is_ascii() {
                        if let Some(pos) = crate::scripts::search::find_ignore_ascii_case(
                            &tab.browser_name,
                            &query_lower,
                        ) {
                            score += if pos == 0 { 80 } else { 55 };
                        }
                    }

                    if let Some(ref mut n) = nucleo {
                        if let Some(nucleo_score) = n.score(&tab.title) {
                            score += 110 + (nucleo_score / 20) as i32;
                        }
                        if !host.is_empty() {
                            if let Some(nucleo_score) = n.score(host) {
                                score += 70 + (nucleo_score / 28) as i32;
                            }
                        }
                        if let Some(nucleo_score) = n.score(&tab.url) {
                            score += 55 + (nucleo_score / 35) as i32;
                        }
                        if let Some(nucleo_score) = n.score(&tab.browser_name) {
                            score += 35 + (nucleo_score / 40) as i32;
                        }
                    }

                    if score > 0 {
                        Some(BrowserTabMatch {
                            tab: tab.clone(),
                            score,
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => match a.tab.browser_name.cmp(&b.tab.browser_name) {
            Ordering::Equal => match a.tab.display_title().cmp(b.tab.display_title()) {
                Ordering::Equal => a.tab.url.cmp(&b.tab.url),
                other => other,
            },
            other => other,
        },
        other => other,
    });

    matches
}

pub(crate) fn root_browser_tabs_query_is_eligible(
    query: &str,
    options: RootBrowserTabsSectionOptions,
) -> bool {
    let query = query.trim();
    options.enabled
        && crate::scripts::search::query_meets_min_query_chars(query, options.min_query_chars)
        && !query.contains('\n')
        && !query.contains('\r')
}

#[allow(dead_code)]
pub(crate) fn search_root_browser_tabs_meta(
    query: &str,
    options: RootBrowserTabsSectionOptions,
) -> Vec<RootBrowserTabSearchHit> {
    search_root_browser_tabs_meta_cached(query, options)
}

#[allow(dead_code)]
pub(crate) fn search_root_browser_tabs_meta_cached(
    query: &str,
    options: RootBrowserTabsSectionOptions,
) -> Vec<RootBrowserTabSearchHit> {
    search_root_browser_tabs_internal(query, options, RootBrowserTabsLookupMode::CachedOnly)
}

#[allow(dead_code)]
pub(crate) fn search_root_browser_tabs_meta_direct(
    query: &str,
    options: RootBrowserTabsSectionOptions,
) -> Vec<RootBrowserTabSearchHit> {
    // Preserve explicit-mode permissive matching, but never begin a detached
    // refresh here: only the app owner can bind completion to its live query.
    search_root_browser_tabs_internal(query, options, RootBrowserTabsLookupMode::RefreshThenCached)
}

#[allow(dead_code)]
pub(crate) fn focus_root_browser_tab(hit: &RootBrowserTabSearchHit) -> Result<()> {
    activate_tab(&hit.tab)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RootBrowserTabsLookupMode {
    CachedOnly,
    RefreshThenCached,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RootBrowserTabsFuzzyMode {
    Permissive,
    Compact,
}

fn search_root_browser_tabs_internal(
    query: &str,
    options: RootBrowserTabsSectionOptions,
    mode: RootBrowserTabsLookupMode,
) -> Vec<RootBrowserTabSearchHit> {
    let query_diagnostic = browser_query_diagnostic(query);
    let _span = tracing::info_span!(
        "search_root_browser_tabs_internal",
        query_bytes = query_diagnostic.bytes,
        query_chars = query_diagnostic.chars,
        mode = ?mode,
        cache_ttl_ms = options.cache_ttl_ms
    )
    .entered();

    if !root_browser_tabs_query_is_eligible(query, options.clone()) {
        return Vec::new();
    }

    let tabs = cached_root_browser_tabs_snapshot(options.cache_ttl_ms);
    let candidates = tabs
        .iter()
        .filter(|tab| root_tab_provider_is_enabled(tab, &options.providers))
        .take(options.scan_limit);

    let fuzzy_mode = match mode {
        RootBrowserTabsLookupMode::CachedOnly => RootBrowserTabsFuzzyMode::Compact,
        RootBrowserTabsLookupMode::RefreshThenCached => RootBrowserTabsFuzzyMode::Permissive,
    };

    root_fuzzy_search_browser_tabs(candidates, query, options.search_urls, fuzzy_mode)
        .into_iter()
        .take(options.max_results)
        .map(|tab_match| {
            let domain = host_from_url(&tab_match.tab.url).to_string();
            RootBrowserTabSearchHit {
                stable_key: browser_tab_stable_key(&tab_match.tab),
                url: tab_match.tab.url.to_string(),
                provider_label: tab_match.tab.browser_name.to_string(),
                tab: tab_match.tab.clone(),
                title: tab_match.tab.title.to_string(),
                domain,
                score: tab_match.score as f32,
            }
        })
        .collect()
}

#[allow(dead_code)]
fn cached_root_browser_tabs_snapshot(cache_ttl_ms: u64) -> Arc<Vec<BrowserTabInfo>> {
    let ttl = Duration::from_millis(cache_ttl_ms);
    if let Ok(cache) = ROOT_BROWSER_TAB_SNAPSHOT.try_lock() {
        if let Some(snapshot) = cache.snapshot.as_ref() {
            let _expired = crate::runtime_policy::root_search_now()
                .saturating_duration_since(snapshot.captured_at)
                > ttl;
            return snapshot.tabs.clone();
        }
    }

    Arc::new(Vec::new())
}

fn root_browser_tabs_cache_is_fresh(
    cache: &RootBrowserTabSnapshotState,
    now: Instant,
    ttl: Duration,
) -> bool {
    !cache.refresh_needed
        && cache
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| now.saturating_duration_since(snapshot.captured_at) <= ttl)
}

#[allow(dead_code)] // Root search cache receipts use this through the binary app layer.
fn fresh_root_browser_tabs_cache_status(
    cache: &RootBrowserTabSnapshotState,
    now: Instant,
    ttl: Duration,
) -> Option<RootPassiveSnapshotStatus> {
    if cache.refresh_in_flight
        || cache.last_refresh_error.is_some()
        || !root_browser_tabs_cache_is_fresh(cache, now, ttl)
    {
        return None;
    }
    Some(RootPassiveSnapshotStatus {
        generation: cache.generation,
        refreshing: false,
        cached_count: cache.snapshot.as_ref()?.tabs.len(),
    })
}

#[allow(dead_code)] // Root search cache receipts use this through the binary app layer.
/// Positive current snapshot evidence; missing, contested, stale, or failed
/// caches are unknown rather than an invented empty successful snapshot.
pub(crate) fn root_browser_tabs_fresh_cache_status(
    cache_ttl_ms: u64,
) -> Option<RootPassiveSnapshotStatus> {
    let cache = ROOT_BROWSER_TAB_SNAPSHOT.try_lock().ok()?;
    fresh_root_browser_tabs_cache_status(
        &cache,
        crate::runtime_policy::root_search_now(),
        Duration::from_millis(cache_ttl_ms),
    )
}

#[allow(dead_code)]
pub(crate) fn root_browser_tabs_snapshot_status() -> RootPassiveSnapshotStatus {
    ROOT_BROWSER_TAB_SNAPSHOT
        .try_lock()
        .map(|cache| RootPassiveSnapshotStatus {
            generation: cache.generation,
            refreshing: cache.refresh_in_flight,
            cached_count: cache
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.tabs.len())
                .unwrap_or(0),
        })
        .unwrap_or(RootPassiveSnapshotStatus {
            generation: 0,
            refreshing: false,
            cached_count: 0,
        })
}

#[allow(dead_code)]
fn ensure_root_browser_tabs_refresh(
    cache_ttl_ms: u64,
    reason: &'static str,
    providers: Vec<crate::config::BrowserTabProvider>,
) {
    let Some(refresh) = try_begin_root_browser_tabs_refresh(cache_ttl_ms, providers.len(), reason)
    else {
        return;
    };

    std::thread::spawn(move || {
        let result = refresh_root_browser_tabs_snapshot(providers);
        let _ = finish_root_browser_tabs_refresh(refresh, result);
    });
}

#[allow(dead_code)]
pub(crate) fn try_begin_root_browser_tabs_refresh(
    cache_ttl_ms: u64,
    provider_count: usize,
    reason: &'static str,
) -> Option<RootBrowserTabsRefresh> {
    let ttl = Duration::from_millis(cache_ttl_ms);
    let Ok(mut cache) = ROOT_BROWSER_TAB_SNAPSHOT.try_lock() else {
        return None;
    };
    let now = crate::runtime_policy::root_search_now();
    let is_fresh = root_browser_tabs_cache_is_fresh(&cache, now, ttl);
    let too_soon = cache
        .last_attempt_at
        .is_some_and(|last| now.duration_since(last) < ROOT_BROWSER_TABS_MIN_REFRESH_INTERVAL);
    let in_backoff = cache.next_refresh_after.is_some_and(|next| now < next);
    if is_fresh || cache.refresh_in_flight || too_soon || in_backoff {
        return None;
    }
    let generation = cache.generation.checked_add(1)?;
    cache.refresh_in_flight = true;
    cache.last_attempt_at = Some(now);
    cache.generation = generation;
    let cache_age_ms = cache
        .snapshot
        .as_ref()
        .map(|snapshot| {
            now.saturating_duration_since(snapshot.captured_at)
                .as_millis() as u64
        })
        .unwrap_or(0);
    let row_count = cache
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.tabs.len())
        .unwrap_or(0);
    tracing::info!(
        source = "browser_tabs",
        generation,
        cache_age_ms,
        ttl_ms = cache_ttl_ms,
        row_count,
        reason,
        provider_count,
        "root_passive_snapshot_refresh_started"
    );

    Some(RootBrowserTabsRefresh {
        generation,
        started_at: Instant::now(),
    })
}

#[allow(dead_code)]
pub(crate) fn refresh_root_browser_tabs_snapshot(
    providers: Vec<crate::config::BrowserTabProvider>,
) -> Result<Vec<BrowserTabInfo>> {
    anyhow::ensure!(
        !crate::runtime_policy::is_owned_evaluation(),
        "owned_source_snapshot_required"
    );
    list_open_tabs_for_root_providers(&providers)
}

#[allow(dead_code)] // Owned search fixtures call this through the binary app layer.
pub(crate) fn owned_root_browser_tabs_snapshot(
    result: Result<Vec<BrowserTabInfo>>,
) -> Result<Vec<BrowserTabInfo>> {
    anyhow::ensure!(
        crate::runtime_policy::is_owned_evaluation(),
        "owned_runtime_required"
    );
    result
}

#[allow(dead_code)] // Owned search fixtures call this through the binary app layer.
pub(crate) fn reset_owned_root_browser_tabs() -> Result<()> {
    anyhow::ensure!(
        crate::runtime_policy::is_owned_evaluation(),
        "owned_runtime_required"
    );
    let mut cache = ROOT_BROWSER_TAB_SNAPSHOT
        .lock()
        .map_err(|_| anyhow::anyhow!("browser_tabs_cache_poisoned"))?;
    let generation = cache
        .generation
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("browser_tabs_generation_exhausted"))?;
    *cache = RootBrowserTabSnapshotState {
        generation,
        ..Default::default()
    };
    Ok(())
}

#[allow(dead_code)] // Owned search source changes call this through the binary app layer.
pub(crate) fn invalidate_owned_root_browser_tabs_freshness() -> Result<()> {
    anyhow::ensure!(
        crate::runtime_policy::is_owned_evaluation(),
        "owned_runtime_required"
    );
    let mut cache = ROOT_BROWSER_TAB_SNAPSHOT
        .lock()
        .map_err(|_| anyhow::anyhow!("browser_tabs_cache_poisoned"))?;
    cache.generation = cache
        .generation
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("browser_tabs_generation_exhausted"))?;
    cache.refresh_in_flight = false;
    cache.refresh_needed = true;
    cache.last_attempt_at = None;
    cache.next_refresh_after = None;
    Ok(())
}

fn discard_root_browser_tabs_refresh_from_state(
    cache: &mut RootBrowserTabSnapshotState,
    generation: u64,
) -> bool {
    if cache.generation != generation || !cache.refresh_in_flight {
        return false;
    }

    cache.refresh_in_flight = false;
    cache.generation = cache.generation.saturating_add(1);
    // This attempt belonged to the canceled query, not the next one. Keep a
    // genuine failure backoff, but do not impose its five-second start throttle.
    cache.last_attempt_at = None;
    true
}

#[allow(dead_code)]
pub(crate) fn discard_root_browser_tabs_refresh(refresh: RootBrowserTabsRefresh) -> bool {
    let Ok(mut cache) = ROOT_BROWSER_TAB_SNAPSHOT.lock() else {
        return false;
    };
    discard_root_browser_tabs_refresh_from_state(&mut cache, refresh.generation)
}

#[allow(dead_code)]
pub(crate) fn finish_root_browser_tabs_refresh(
    refresh: RootBrowserTabsRefresh,
    result: Result<Vec<BrowserTabInfo>>,
) -> bool {
    let elapsed_ms = refresh.started_at.elapsed().as_millis() as u64;

    let Ok(mut cache) = ROOT_BROWSER_TAB_SNAPSHOT.lock() else {
        return false;
    };
    if cache.generation != refresh.generation || !cache.refresh_in_flight {
        return false;
    }

    match result {
        Ok(tabs) => {
            let row_count = tabs.len();
            cache.snapshot = Some(RootBrowserTabSnapshot {
                captured_at: crate::runtime_policy::root_search_now(),
                tabs: Arc::new(tabs.clone()),
            });
            cache.refresh_needed = false;
            cache.last_refresh_error = None;
            cache.last_success_at = Some(crate::runtime_policy::root_search_now());
            cache.next_refresh_after = None;
            cache.failure_count = 0;

            if !crate::runtime_policy::is_owned_evaluation() {
                let urls: Vec<String> = tabs.iter().map(|t| t.url.to_string()).collect();
                std::thread::spawn(move || {
                    let needing = crate::favicons::domains_needing_favicons(&urls);
                    crate::favicons::fetch_favicons_blocking(&needing);
                });
            }

            tracing::info!(
                source = "browser_tabs",
                generation = refresh.generation,
                elapsed_ms,
                row_count,
                "root_passive_snapshot_refresh_completed"
            );
        }
        Err(error) => {
            cache.failure_count = cache.failure_count.saturating_add(1);
            let backoff = root_browser_tabs_failure_backoff(cache.failure_count);
            cache.next_refresh_after = Some(crate::runtime_policy::root_search_now() + backoff);
            cache.last_refresh_error = Some(error.to_string());
            tracing::warn!(
                source = "browser_tabs",
                generation = refresh.generation,
                elapsed_ms,
                failure_count = cache.failure_count,
                backoff_ms = backoff.as_millis() as u64,
                error_bytes = error.to_string().len(),
                "root_passive_snapshot_refresh_failed"
            );
        }
    }
    cache.refresh_in_flight = false;
    cache.generation = cache.generation.saturating_add(1);
    true
}

fn root_browser_tabs_failure_backoff(failure_count: u32) -> Duration {
    let exponent = failure_count.saturating_sub(1).min(8);
    let multiplier = 1u32.checked_shl(exponent).unwrap_or(u32::MAX);
    ROOT_BROWSER_TABS_FAILURE_BACKOFF_BASE
        .saturating_mul(multiplier)
        .min(ROOT_BROWSER_TABS_FAILURE_BACKOFF_MAX)
}

#[cfg(test)]
fn reset_root_browser_tabs_snapshot_for_test() {
    if let Ok(mut cache) = ROOT_BROWSER_TAB_SNAPSHOT.lock() {
        *cache = RootBrowserTabSnapshotState::default();
    }
}

#[cfg(test)]
fn store_root_browser_tabs_snapshot_for_test(captured_at: Instant, tabs: Vec<BrowserTabInfo>) {
    if let Ok(mut cache) = ROOT_BROWSER_TAB_SNAPSHOT.lock() {
        cache.snapshot = Some(RootBrowserTabSnapshot {
            captured_at,
            tabs: Arc::new(tabs),
        });
        cache.refresh_in_flight = false;
    }
}

#[allow(dead_code)]
fn root_fuzzy_search_browser_tabs<'a>(
    tabs: impl IntoIterator<Item = &'a BrowserTabInfo>,
    query: &str,
    search_urls: bool,
    mode: RootBrowserTabsFuzzyMode,
) -> Vec<BrowserTabMatch> {
    let query_diagnostic = browser_query_diagnostic(query);
    let tabs = tabs.into_iter();
    let _span = tracing::info_span!(
        "root_fuzzy_search_browser_tabs",
        query_bytes = query_diagnostic.bytes,
        query_chars = query_diagnostic.chars,
        search_urls,
        mode = ?mode
    )
    .entered();

    if query.trim().is_empty() {
        return tabs
            .cloned()
            .map(|tab| BrowserTabMatch { tab, score: 0 })
            .collect();
    }

    let query_lower = query.trim().to_lowercase();
    let query_is_ascii = query_lower.is_ascii();
    let use_nucleo = query_lower.len() >= crate::scripts::search::MIN_FUZZY_QUERY_LEN;

    let mut nucleo = if use_nucleo {
        Some(crate::scripts::NucleoCtx::new(&query_lower))
    } else {
        None
    };
    let mut matches: Vec<BrowserTabMatch> = tabs
        .filter_map(|tab| {
            let mut score = 0i32;
            let host = host_from_url(&tab.url);

            let position = |field: &str| {
                if query_is_ascii {
                    crate::scripts::search::find_ignore_ascii_case(field, &query_lower)
                } else {
                    field.to_lowercase().find(&query_lower)
                }
            };
            if let Some(pos) = position(&tab.title) {
                score += if pos == 0 { 240 } else { 190 };
            }
            if let Some(pos) = position(host) {
                score += if pos == 0 { 180 } else { 135 };
            }
            if search_urls {
                if let Some(pos) = position(&tab.url) {
                    score += if pos == 0 { 120 } else { 90 };
                }
            }
            if let Some(pos) = position(&tab.browser_name) {
                score += if pos == 0 { 80 } else { 55 };
            }

            if let Some(ref mut n) = nucleo {
                if let Some(nucleo_score) = match mode {
                    RootBrowserTabsFuzzyMode::Permissive => n.score(&tab.title),
                    RootBrowserTabsFuzzyMode::Compact => n.compact_score(&tab.title, &query_lower),
                } {
                    score += 110 + (nucleo_score / 20) as i32;
                }
                if !host.is_empty() {
                    if let Some(nucleo_score) = match mode {
                        RootBrowserTabsFuzzyMode::Permissive => n.score(host),
                        RootBrowserTabsFuzzyMode::Compact => n.compact_score(host, &query_lower),
                    } {
                        score += 70 + (nucleo_score / 28) as i32;
                    }
                }
                if search_urls {
                    if let Some(nucleo_score) = match mode {
                        RootBrowserTabsFuzzyMode::Permissive => n.score(&tab.url),
                        RootBrowserTabsFuzzyMode::Compact => {
                            n.compact_score(&tab.url, &query_lower)
                        }
                    } {
                        score += 55 + (nucleo_score / 35) as i32;
                    }
                }
                if let Some(nucleo_score) = match mode {
                    RootBrowserTabsFuzzyMode::Permissive => n.score(&tab.browser_name),
                    RootBrowserTabsFuzzyMode::Compact => {
                        n.compact_score(&tab.browser_name, &query_lower)
                    }
                } {
                    score += 35 + (nucleo_score / 40) as i32;
                }
            }

            if score > 0 {
                Some(BrowserTabMatch {
                    tab: tab.clone(),
                    score,
                })
            } else {
                None
            }
        })
        .collect();

    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => match a.tab.browser_name.cmp(&b.tab.browser_name) {
            Ordering::Equal => match a.tab.display_title().cmp(b.tab.display_title()) {
                Ordering::Equal => a.tab.url.cmp(&b.tab.url),
                other => other,
            },
            other => other,
        },
        other => other,
    });

    matches
}

#[allow(dead_code)]
fn root_tab_provider_is_enabled(
    tab: &BrowserTabInfo,
    providers: &[crate::config::BrowserTabProvider],
) -> bool {
    let Some(provider) = browser_tab_provider_for_bundle_id(&tab.browser_bundle_id) else {
        return false;
    };
    providers.is_empty() || providers.contains(&provider)
}

fn root_supported_browser_provider_is_enabled(
    browser: &SupportedBrowser,
    providers: &[crate::config::BrowserTabProvider],
) -> bool {
    if providers.is_empty() {
        return true;
    }
    let Some(provider) = browser_tab_provider_for_bundle_id(browser.bundle_id) else {
        return false;
    };
    providers.contains(&provider)
}

fn browser_tab_provider_for_bundle_id(
    bundle_id: &str,
) -> Option<crate::config::BrowserTabProvider> {
    match bundle_id {
        "company.thebrowser.Browser" => Some(crate::config::BrowserTabProvider::Arc),
        "com.google.Chrome" => Some(crate::config::BrowserTabProvider::Chrome),
        "com.brave.Browser" => Some(crate::config::BrowserTabProvider::Brave),
        "com.microsoft.edgemac" => Some(crate::config::BrowserTabProvider::Edge),
        "com.apple.Safari" => Some(crate::config::BrowserTabProvider::Safari),
        "org.chromium.Chromium" => Some(crate::config::BrowserTabProvider::Chromium),
        "com.vivaldi.Vivaldi" => Some(crate::config::BrowserTabProvider::Vivaldi),
        "com.operasoftware.Opera" => Some(crate::config::BrowserTabProvider::Opera),
        _ => None,
    }
}

fn list_tabs_for_browser(browser: &SupportedBrowser) -> Result<Vec<BrowserTabInfo>> {
    let script = snapshot_program(browser, TabRequest::List)?;
    let output =
        crate::platform::run_jxa_with_timeout(&script, "browser_tabs_list", BROWSER_TIMEOUT)
            .with_context(|| format!("list_browser_tabs_failed: browser={}", browser.app_name))?;
    serde_json::from_str(&output).context("Invalid browser tab snapshot")
}

/// One read-only AppKit snapshot per scan. A slow System Events process (or an
/// absent browser) must not be able to veto another browser's tab list.
#[cfg(target_os = "macos")]
fn running_bundle_ids() -> Result<HashSet<String>> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)?;
    Ok(objc::rc::autoreleasepool(|| {
        let apps = objc2_app_kit::NSWorkspace::sharedWorkspace().runningApplications();
        (0..apps.len())
            .filter_map(|i| apps.objectAtIndex(i).bundleIdentifier())
            .map(|id| id.to_string())
            .collect()
    }))
}

#[cfg(not(target_os = "macos"))]
fn running_bundle_ids() -> Result<HashSet<String>> {
    bail!("Browser tab discovery requires macOS")
}

fn running_tab_browsers(
    running: &HashSet<String>,
    providers: &[crate::config::BrowserTabProvider],
) -> Vec<&'static SupportedBrowser> {
    SUPPORTED_BROWSERS
        .iter()
        .filter(|b| {
            running.contains(b.bundle_id)
                && root_supported_browser_provider_is_enabled(b, providers)
        })
        .collect()
}

fn host_from_url(url: &str) -> &str {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    after_scheme.split('/').next().unwrap_or(after_scheme)
}

// The adapter returns rows only for List. Neither focus route returns browser contents.
#[derive(Serialize)]
#[serde(untagged)]
enum TabRequest<'a> {
    List,
    Named(&'a str),
    Selected(&'a BrowserTabInfo),
}

/// Bind a running process and its window before resolving the page. Neither cached positions
/// nor an identical URL in another window are substitutes for the selected target.
fn snapshot_program(browser: &SupportedBrowser, target: TabRequest<'_>) -> Result<String> {
    let bundle = serde_json::to_string(browser.bundle_id)?;
    let name = serde_json::to_string(browser.app_name)?;
    let target = serde_json::to_string(&target)?;
    let safari = browser.family == BrowserFamily::Safari;
    Ok(format!(
        r#"ObjC.import('AppKit');
const bundle = {bundle};
const pid = () => {{ const apps = $.NSRunningApplication.runningApplicationsWithBundleIdentifier(bundle); return apps.count === 1 ? Number(apps.objectAtIndex(0).processIdentifier) : 0; }};
const frontPid = () => {{ const front = $.NSWorkspace.sharedWorkspace.frontmostApplication; return front ? Number(front.processIdentifier) : 0; }};
{}(Application(bundle), bundle, {name}, {safari}, pid, {target}, frontPid)"#,
        include_str!("browser_tabs/snapshot.js").trim_end()
    ))
}

// Favicon logic moved to src/favicons.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_tab_snapshots_and_resets_require_runtime_authority() {
        assert!(owned_root_browser_tabs_snapshot(Ok(Vec::new())).is_err());
        assert!(reset_owned_root_browser_tabs().is_err());
        assert!(invalidate_owned_root_browser_tabs_freshness().is_err());
    }

    #[test]
    fn browser_tab_read_failure_is_not_partial_success_or_empty_success() {
        let tab = BrowserTabInfo {
            identity: None,
            browser_name: "Safari".into(),
            browser_bundle_id: "com.apple.Safari".into(),
            window_index: 1,
            tab_index: 1,
            title: "last good".into(),
            url: "https://example.com".into(),
        };
        let result = combine_browser_tab_results(vec![
            Ok(vec![tab]),
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into()),
        ]);
        assert_eq!(
            result
                .unwrap_err()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert!(combine_browser_tab_results(vec![Ok(Vec::new())])
            .unwrap()
            .is_empty());
    }

    #[test]
    fn browser_query_diagnostics_retain_only_sizes_not_private_query_text() {
        let private_query = "private-browser-query-canary-🔒";
        let diagnostic = browser_query_diagnostic(private_query);

        assert_eq!(diagnostic.bytes, private_query.len());
        assert_eq!(diagnostic.chars, private_query.chars().count());
        assert_ne!(diagnostic.bytes, diagnostic.chars);
        assert!(!format!("{diagnostic:?}").contains("private-browser-query-canary"));
    }

    #[test]
    fn bare_domain_query_recognizes_hostname_intent() {
        for query in [
            "x.com",
            "news.ycombinator.com",
            "www.example.org",
            "HTTPS://WWW.Example.COM/",
            "http://x.com",
        ] {
            assert!(query_is_bare_domain(query), "expected domain: {query:?}");
        }

        for query in [
            "deploy",
            "x.com launcher",
            "~/src",
            "/tmp/x",
            "1.2.3",
            ":tabs x.com",
            "tabs:x.com",
            "@x.com",
            "https://x.com/path",
        ] {
            assert!(
                !query_is_bare_domain(query),
                "expected rejection: {query:?}"
            );
        }
    }

    #[test]
    fn fuzzy_search_browser_tabs_prefers_title_match() {
        let tabs = vec![
            BrowserTabInfo {
                identity: None,
                browser_name: "Safari".into(),
                browser_bundle_id: "com.apple.Safari".into(),
                window_index: 1,
                tab_index: 1,
                title: "Build a Claude Managed Agent".into(),
                url: "https://vercel.com/kb/guide".into(),
            },
            BrowserTabInfo {
                identity: None,
                browser_name: "Google Chrome".into(),
                browser_bundle_id: "com.google.Chrome".into(),
                window_index: 1,
                tab_index: 2,
                title: "Home".into(),
                url: "https://claude-managed-agent.example.com".into(),
            },
        ];

        let matches = fuzzy_search_browser_tabs(&tabs, "managed agent");
        assert_eq!(matches.len(), 2);
        assert_eq!(
            matches[0].tab.title.as_ref(),
            "Build a Claude Managed Agent",
            "title hit should outrank URL-only hit"
        );
    }

    #[test]
    fn root_browser_tabs_compact_mode_rejects_sparse_natural_language_url_match() {
        let tabs = vec![BrowserTabInfo {
            identity: None,
            browser_name: "Google Chrome".into(),
            browser_bundle_id: "com.google.Chrome".into(),
            window_index: 1,
            tab_index: 1,
            title: "Delta".into(),
            url: "https://delta.example.com/wxxxxhxxxxaxxxxtxxxxixxxsxxxxtxxxhxxxixxxsxxxxaxxxxnxxxxyxxxxwxxxxaxxxxy".into(),
        }];

        let permissive = root_fuzzy_search_browser_tabs(
            &tabs,
            "What is this anyway",
            true,
            RootBrowserTabsFuzzyMode::Permissive,
        );
        assert_eq!(
            permissive.len(),
            1,
            "fixture must demonstrate the previous sparse fuzzy match"
        );

        let compact = root_fuzzy_search_browser_tabs(
            &tabs,
            "What is this anyway",
            true,
            RootBrowserTabsFuzzyMode::Compact,
        );
        assert!(
            compact.is_empty(),
            "implicit root tab search should reject sparse long-URL fuzzy matches"
        );

        let direct_title =
            root_fuzzy_search_browser_tabs(&tabs, "delta", true, RootBrowserTabsFuzzyMode::Compact);
        assert_eq!(direct_title.len(), 1);
    }

    #[test]
    fn universal_search_finds_late_open_tabs_from_the_first_character() {
        let base = BrowserTabInfo {
            identity: None,
            browser_name: "Google Chrome".into(),
            browser_bundle_id: "com.google.Chrome".into(),
            title: "Unrelated".into(),
            url: "https://example.com".into(),
            window_index: 1,
            tab_index: 1,
        };
        let mut tabs = vec![base.clone(); 300];
        let target = BrowserTabInfo {
            browser_name: "Safari".into(),
            browser_bundle_id: "com.apple.Safari".into(),
            title: "📖 Zettelkasten".into(),
            url: "https://notes.example.org".into(),
            ..base
        };
        tabs.push(target.clone());
        store_root_browser_tabs_snapshot_for_test(Instant::now(), tabs);
        let mut config = crate::config::UnifiedSearchConfig::default();
        let search = |q: &str, config: &crate::config::UnifiedSearchConfig| {
            search_root_browser_tabs_meta_cached(q, config.browser_tabs_section_options())
        };
        for q in ["z", "notes.example.org"] {
            let hits = search(q, &config);
            assert_eq!(hits.len(), 1, "{q}");
            assert_eq!(hits[0].tab, target);
        }
        assert!(
            search("", &config).is_empty(),
            "the empty launcher stays unchanged"
        );
        config.browser_tabs.scan_limit = Some(80);
        assert!(search("z", &config).is_empty());
        config.browser_tabs.scan_limit = None;
        config.browser_tabs.providers = vec![crate::config::BrowserTabProvider::Chrome];
        assert!(search("z", &config).is_empty());
        reset_root_browser_tabs_snapshot_for_test();
    }

    #[test]
    fn expired_root_browser_tab_snapshot_returns_stale_rows() {
        reset_root_browser_tabs_snapshot_for_test();
        let tab = BrowserTabInfo {
            identity: None,
            browser_name: "Google Chrome".into(),
            browser_bundle_id: "com.google.Chrome".into(),
            window_index: 1,
            tab_index: 1,
            title: "Root Passive Snapshot".into(),
            url: "https://example.com/root-passive-snapshot".into(),
        };
        store_root_browser_tabs_snapshot_for_test(
            Instant::now() - Duration::from_millis(500),
            vec![tab.clone()],
        );

        let rows = cached_root_browser_tabs_snapshot(1);
        assert_eq!(rows.as_ref(), &vec![tab]);
        reset_root_browser_tabs_snapshot_for_test();
    }

    #[test]
    fn missing_root_browser_tab_snapshot_returns_empty_rows() {
        reset_root_browser_tabs_snapshot_for_test();
        assert!(cached_root_browser_tabs_snapshot(1).is_empty());
    }

    #[test]
    fn direct_browser_tabs_lookup_is_snapshot_only_and_keeps_permissive_matching() {
        reset_root_browser_tabs_snapshot_for_test();
        let previous_rows = Arc::new(vec![BrowserTabInfo {
            identity: None,
            browser_name: "Google Chrome".into(),
            browser_bundle_id: "com.google.Chrome".into(),
            window_index: 1,
            tab_index: 1,
            title: "Delta".into(),
            url: "https://delta.example.com/wxxxxhxxxxaxxxxtxxxxixxxsxxxxtxxxhxxxixxxsxxxxaxxxxnxxxxyxxxxwxxxxaxxxxy".into(),
        }]);
        {
            let mut cache = ROOT_BROWSER_TAB_SNAPSHOT
                .lock()
                .expect("browser tabs cache");
            *cache = RootBrowserTabSnapshotState {
                snapshot: Some(RootBrowserTabSnapshot {
                    captured_at: Instant::now() - Duration::from_secs(7),
                    tabs: Arc::clone(&previous_rows),
                }),
                generation: 41,
                ..RootBrowserTabSnapshotState::default()
            };
        }

        let before = root_browser_tabs_snapshot_status();
        let options = RootBrowserTabsSectionOptions {
            enabled: true,
            min_query_chars: 0,
            cache_ttl_ms: 10_000,
            ..RootBrowserTabsSectionOptions::default()
        };
        let hits = search_root_browser_tabs_meta_direct("What is this anyway", options);
        let after = root_browser_tabs_snapshot_status();

        assert_eq!(
            hits.len(),
            1,
            "explicit lookup must retain permissive matching"
        );
        assert_eq!(
            before, after,
            "direct lookup cannot own a refresh generation"
        );
        assert!(!after.refreshing);
        {
            let cache = ROOT_BROWSER_TAB_SNAPSHOT
                .lock()
                .expect("browser tabs cache");
            assert!(Arc::ptr_eq(
                &cache.snapshot.as_ref().expect("snapshot preserved").tabs,
                &previous_rows
            ));
        }
        reset_root_browser_tabs_snapshot_for_test();
    }

    #[test]
    fn cold_direct_browser_tabs_lookup_cannot_start_an_unowned_refresh() {
        reset_root_browser_tabs_snapshot_for_test();
        let before = root_browser_tabs_snapshot_status();
        let options = RootBrowserTabsSectionOptions {
            enabled: true,
            min_query_chars: 0,
            ..RootBrowserTabsSectionOptions::default()
        };

        assert!(search_root_browser_tabs_meta_direct("private query", options).is_empty());
        assert_eq!(root_browser_tabs_snapshot_status(), before);
        assert!(!root_browser_tabs_snapshot_status().refreshing);
    }

    #[test]
    fn fresh_browser_tabs_cache_evidence_requires_present_idle_successful_snapshot() {
        let now = Instant::now();
        let ttl = Duration::from_secs(1);
        let mut cache = RootBrowserTabSnapshotState::default();
        assert!(fresh_root_browser_tabs_cache_status(&cache, now, ttl).is_none());
        cache.snapshot = Some(RootBrowserTabSnapshot {
            captured_at: now,
            tabs: Arc::new(Vec::new()),
        });
        cache.generation = 7;
        let status = fresh_root_browser_tabs_cache_status(&cache, now + ttl, ttl)
            .expect("a present empty snapshot is valid through its TTL boundary");
        assert_eq!(status.generation, 7);
        assert_eq!(status.cached_count, 0);
        assert!(fresh_root_browser_tabs_cache_status(
            &cache,
            now + ttl + Duration::from_nanos(1),
            ttl
        )
        .is_none());
        cache.refresh_in_flight = true;
        assert!(fresh_root_browser_tabs_cache_status(&cache, now, ttl).is_none());
        cache.refresh_in_flight = false;
        cache.refresh_needed = true;
        assert!(fresh_root_browser_tabs_cache_status(&cache, now, ttl).is_none());
        cache.refresh_needed = false;
        cache.last_refresh_error = Some("read failed".into());
        assert!(fresh_root_browser_tabs_cache_status(&cache, now, ttl).is_none());
    }

    #[test]
    fn canceled_browser_tabs_generation_preserves_snapshot_and_rearms_current_query() {
        let previous_rows = Arc::new(Vec::new());
        let backoff_until = Instant::now() + Duration::from_secs(10);
        let mut cache = RootBrowserTabSnapshotState {
            snapshot: Some(RootBrowserTabSnapshot {
                captured_at: Instant::now(),
                tabs: Arc::clone(&previous_rows),
            }),
            refresh_in_flight: true,
            generation: 7,
            last_refresh_error: Some("previous failure".to_owned()),
            last_attempt_at: Some(Instant::now()),
            next_refresh_after: Some(backoff_until),
            failure_count: 2,
            ..RootBrowserTabSnapshotState::default()
        };

        assert!(discard_root_browser_tabs_refresh_from_state(&mut cache, 7));
        assert!(!cache.refresh_in_flight);
        assert_eq!(cache.generation, 8);
        assert!(cache.last_attempt_at.is_none());
        assert_eq!(cache.next_refresh_after, Some(backoff_until));
        assert_eq!(
            cache.last_refresh_error.as_deref(),
            Some("previous failure")
        );
        assert_eq!(cache.failure_count, 2);
        assert!(Arc::ptr_eq(
            &cache.snapshot.as_ref().expect("preserved snapshot").tabs,
            &previous_rows
        ));
    }

    #[test]
    fn older_browser_tabs_completion_cannot_cancel_newer_in_flight_generation() {
        let attempted_at = Instant::now();
        let mut cache = RootBrowserTabSnapshotState {
            refresh_in_flight: true,
            generation: 9,
            last_attempt_at: Some(attempted_at),
            ..RootBrowserTabSnapshotState::default()
        };

        assert!(!discard_root_browser_tabs_refresh_from_state(&mut cache, 7));
        assert!(cache.refresh_in_flight);
        assert_eq!(cache.generation, 9);
        assert_eq!(cache.last_attempt_at, Some(attempted_at));
    }

    #[test]
    fn root_browser_tabs_refresh_completion_advances_generation_and_stores_rows() {
        reset_root_browser_tabs_snapshot_for_test();
        let before = root_browser_tabs_snapshot_status();
        let refresh =
            try_begin_root_browser_tabs_refresh(30_000, 1, "test").expect("refresh should start");
        assert!(root_browser_tabs_snapshot_status().refreshing);

        let tab = BrowserTabInfo {
            identity: None,
            browser_name: "Google Chrome".into(),
            browser_bundle_id: "com.google.Chrome".into(),
            window_index: 1,
            tab_index: 1,
            title: "Root Completion Tab".into(),
            url: "https://example.com/root-completion-tab".into(),
        };
        assert!(finish_root_browser_tabs_refresh(
            refresh,
            Ok(vec![tab.clone()])
        ));

        let after = root_browser_tabs_snapshot_status();
        assert!(after.generation > before.generation);
        assert!(!after.refreshing);
        assert_eq!(after.cached_count, 1);
        assert_eq!(
            cached_root_browser_tabs_snapshot(30_000).as_ref(),
            &vec![tab]
        );
        reset_root_browser_tabs_snapshot_for_test();
    }

    #[test]
    fn root_browser_tabs_failure_backoff_is_exponential_and_capped() {
        assert_eq!(
            root_browser_tabs_failure_backoff(1),
            ROOT_BROWSER_TABS_FAILURE_BACKOFF_BASE
        );
        assert_eq!(
            root_browser_tabs_failure_backoff(2),
            ROOT_BROWSER_TABS_FAILURE_BACKOFF_BASE * 2
        );
        assert_eq!(
            root_browser_tabs_failure_backoff(99),
            ROOT_BROWSER_TABS_FAILURE_BACKOFF_MAX
        );
    }

    #[test]
    fn cached_root_browser_tabs_snapshot_does_not_block_on_refresh_lock() {
        reset_root_browser_tabs_snapshot_for_test();
        let _guard = ROOT_BROWSER_TAB_SNAPSHOT.lock().expect("snapshot lock");
        let started = Instant::now();
        let rows = cached_root_browser_tabs_snapshot(1);
        assert!(rows.is_empty());
        assert!(
            started.elapsed() < Duration::from_millis(25),
            "foreground snapshot read should not wait on refresh lock"
        );
    }

    #[test]
    fn root_browser_tabs_refresh_uses_configured_providers_only() {
        let providers = vec![crate::config::BrowserTabProvider::Chrome];
        assert!(root_supported_browser_provider_is_enabled(
            &SUPPORTED_BROWSERS[1],
            &providers
        ));
        assert!(!root_supported_browser_provider_is_enabled(
            &SUPPORTED_BROWSERS[0],
            &providers
        ));
    }

    #[test]
    fn tab_selection_follows_identity_across_reorder_but_not_navigation() {
        let mut tab: BrowserTabInfo = serde_json::from_value(serde_json::json!({
            "browser_name":"Safari", "browser_bundle_id":"com.apple.Safari", "window_index":1, "tab_index":1,
            "title":"Docs", "url":"https://example.invalid/one", "identity":{"pid":42,"window":"9","tab":null}
        })).unwrap();
        let original = browser_tab_stable_key(&tab);
        tab.window_index = 3;
        tab.tab_index = 4;
        assert_eq!(original, browser_tab_stable_key(&tab));
        tab.url = "https://example.invalid/two".into();
        assert_ne!(original, browser_tab_stable_key(&tab));
        tab.identity = None;
        assert!(activate_tab(&tab)
            .unwrap_err()
            .to_string()
            .contains("identity"));
    }

    #[test]
    fn tab_scan_queries_only_running_enabled_browsers() {
        let running = HashSet::from([
            "com.google.Chrome".to_owned(),
            "com.apple.Safari.SafariNetworkingService".to_owned(),
            "com.brave.Browser".to_owned(),
        ]);
        let selected = |providers: &[crate::config::BrowserTabProvider]| {
            running_tab_browsers(&running, providers)
                .into_iter()
                .map(|b| b.app_name)
                .collect::<Vec<_>>()
        };
        assert_eq!(selected(&[]), ["Google Chrome", "Brave Browser"]);
        assert_eq!(
            selected(&[crate::config::BrowserTabProvider::Chrome]),
            ["Google Chrome"]
        );
        assert!(selected(&[crate::config::BrowserTabProvider::Arc]).is_empty());
    }
}
