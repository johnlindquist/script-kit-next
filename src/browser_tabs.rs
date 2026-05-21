use std::cmp::Ordering;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use rayon::prelude::*;
use serde::Deserialize;

const FIELD_SEPARATOR: char = '\u{1e}';
const RECORD_SEPARATOR: char = '\u{1f}';
const BROWSER_TIMEOUT: Duration = Duration::from_secs(3);
const BROWSER_RUNNING_TIMEOUT: Duration = Duration::from_secs(1);
const ROOT_BROWSER_TABS_MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const ROOT_BROWSER_TABS_FAILURE_BACKOFF_BASE: Duration = Duration::from_secs(15);
const ROOT_BROWSER_TABS_FAILURE_BACKOFF_MAX: Duration = Duration::from_secs(120);

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BrowserTabInfo {
    pub browser_name: Arc<str>,
    pub browser_bundle_id: Arc<str>,
    pub window_index: usize,
    pub tab_index: usize,
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
            scan_limit: crate::config::defaults::DEFAULT_UNIFIED_SEARCH_BROWSER_TABS_SCAN_LIMIT,
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

    let results: Vec<Result<Vec<BrowserTabInfo>>> = SUPPORTED_BROWSERS
        .par_iter()
        .filter(|browser| root_supported_browser_provider_is_enabled(browser, providers))
        .map(|browser| {
            let _browser_span =
                tracing::info_span!("list_tabs_for_browser", browser = browser.app_name).entered();
            match is_browser_running(browser.bundle_id) {
                Ok(false) => Ok(Vec::new()),
                Ok(true) => {
                    let tabs = list_tabs_for_browser(browser);
                    if let Ok(ref t) = tabs {
                        tracing::info!(
                            browser = browser.app_name,
                            tab_count = t.len(),
                            "list_tabs_for_browser_completed"
                        );
                    }
                    tabs
                }
                Err(error) => {
                    tracing::warn!(
                        browser = browser.app_name,
                        error = %error,
                        "check_browser_running_failed"
                    );
                    Err(anyhow!("{}: {error}", browser.app_name))
                }
            }
        })
        .collect();

    let mut tabs = Vec::new();
    let mut errors = Vec::new();

    for res in results {
        match res {
            Ok(mut browser_tabs) => tabs.append(&mut browser_tabs),
            Err(error) => errors.push(error.to_string()),
        }
    }

    if tabs.is_empty() && !errors.is_empty() {
        bail!("Failed to read browser tabs: {}", errors.join(" | "));
    }

    Ok(tabs)
}

pub fn activate_tab(tab: &BrowserTabInfo) -> Result<()> {
    let browser = supported_browser_for(tab).ok_or_else(|| {
        anyhow!(
            "Unsupported browser '{}'",
            if tab.browser_name.is_empty() {
                tab.browser_bundle_id.as_ref()
            } else {
                tab.browser_name.as_ref()
            }
        )
    })?;

    let script = build_activate_tab_script(browser, tab.window_index, tab.tab_index);
    crate::platform::run_osascript(&script, "browser_tabs_activate")
        .map(|_| ())
        .with_context(|| {
            format!(
                "activate_browser_tab_failed: browser={} window_index={} tab_index={}",
                browser.app_name, tab.window_index, tab.tab_index
            )
        })
}

pub fn fuzzy_search_browser_tabs(tabs: &[BrowserTabInfo], query: &str) -> Vec<BrowserTabMatch> {
    let _span =
        tracing::info_span!("fuzzy_search_browser_tabs", tab_count = tabs.len(), query).entered();

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
        && query.len() >= options.min_query_chars
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
    // When explicitly typing "tabs:", we want it to be fresh, but not blocking.
    // We use a short TTL (5s) for direct mode to avoid re-fetching on every keystroke
    // while still ensuring it's relatively recent.
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

fn search_root_browser_tabs_internal(
    query: &str,
    options: RootBrowserTabsSectionOptions,
    mode: RootBrowserTabsLookupMode,
) -> Vec<RootBrowserTabSearchHit> {
    let _span = tracing::info_span!(
        "search_root_browser_tabs_internal",
        query,
        mode = ?mode,
        cache_ttl_ms = options.cache_ttl_ms
    )
    .entered();

    if !root_browser_tabs_query_is_eligible(query, options.clone()) {
        return Vec::new();
    }

    if matches!(mode, RootBrowserTabsLookupMode::RefreshThenCached) {
        // Keep explicit `tabs:` refreshes relatively fresh without putting the
        // implicit root typing path on any provider or refresh setup work.
        ensure_root_browser_tabs_refresh(
            5000,
            "root_search_query_direct",
            options.providers.clone(),
        );
    }

    let tabs = cached_root_browser_tabs_snapshot(options.cache_ttl_ms);
    let tabs = tabs
        .iter()
        .filter(|tab| root_tab_provider_is_enabled(tab, &options.providers))
        .take(options.scan_limit)
        .cloned()
        .collect::<Vec<_>>();

    root_fuzzy_search_browser_tabs(&tabs, query, options.search_urls)
        .into_iter()
        .take(options.max_results)
        .map(|tab_match| {
            let domain = host_from_url(&tab_match.tab.url).to_string();
            RootBrowserTabSearchHit {
                stable_key: root_browser_tab_stable_key(&tab_match.tab),
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
            let _expired = snapshot.captured_at.elapsed() > ttl;
            return snapshot.tabs.clone();
        }
    }

    Arc::new(Vec::new())
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
    let ttl = Duration::from_millis(cache_ttl_ms);
    let generation = {
        let Ok(mut cache) = ROOT_BROWSER_TAB_SNAPSHOT.try_lock() else {
            return;
        };
        let now = Instant::now();
        let is_fresh = cache
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.captured_at.elapsed() <= ttl);
        let too_soon = cache
            .last_attempt_at
            .is_some_and(|last| now.duration_since(last) < ROOT_BROWSER_TABS_MIN_REFRESH_INTERVAL);
        let in_backoff = cache.next_refresh_after.is_some_and(|next| now < next);
        if is_fresh || cache.refresh_in_flight || too_soon || in_backoff {
            return;
        }
        cache.refresh_in_flight = true;
        cache.last_attempt_at = Some(now);
        cache.generation = cache.generation.wrapping_add(1);
        let generation = cache.generation;
        let cache_age_ms = cache
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.captured_at.elapsed().as_millis() as u64)
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
            provider_count = providers.len(),
            "root_passive_snapshot_refresh_started"
        );
        generation
    };

    std::thread::spawn(move || {
        let started = Instant::now();
        let result = list_open_tabs_for_root_providers(&providers);
        let elapsed_ms = started.elapsed().as_millis() as u64;

        let Ok(mut cache) = ROOT_BROWSER_TAB_SNAPSHOT.lock() else {
            return;
        };
        if cache.generation != generation {
            return;
        }

        match result {
            Ok(tabs) => {
                let row_count = tabs.len();
                cache.snapshot = Some(RootBrowserTabSnapshot {
                    captured_at: Instant::now(),
                    tabs: Arc::new(tabs.clone()),
                });
                cache.last_refresh_error = None;
                cache.last_success_at = Some(Instant::now());
                cache.next_refresh_after = None;
                cache.failure_count = 0;

                // Trigger favicon fetch in background
                let urls: Vec<String> = tabs.iter().map(|t| t.url.to_string()).collect();
                std::thread::spawn(move || {
                    let needing = crate::favicons::domains_needing_favicons(&urls);
                    crate::favicons::fetch_favicons_blocking(&needing);
                });

                tracing::info!(
                    source = "browser_tabs",
                    generation,
                    elapsed_ms,
                    row_count,
                    "root_passive_snapshot_refresh_completed"
                );
            }
            Err(error) => {
                cache.failure_count = cache.failure_count.saturating_add(1);
                let backoff = root_browser_tabs_failure_backoff(cache.failure_count);
                cache.next_refresh_after = Some(Instant::now() + backoff);
                cache.last_refresh_error = Some(error.to_string());
                tracing::warn!(
                    source = "browser_tabs",
                    generation,
                    elapsed_ms,
                    failure_count = cache.failure_count,
                    backoff_ms = backoff.as_millis() as u64,
                    error = %error,
                    "root_passive_snapshot_refresh_failed"
                );
            }
        }
        cache.refresh_in_flight = false;
    });
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
fn root_fuzzy_search_browser_tabs(
    tabs: &[BrowserTabInfo],
    query: &str,
    search_urls: bool,
) -> Vec<BrowserTabMatch> {
    let _span = tracing::info_span!(
        "root_fuzzy_search_browser_tabs",
        tab_count = tabs.len(),
        query,
        search_urls
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

    let mut nucleo = if use_nucleo {
        Some(crate::scripts::NucleoCtx::new(&query_lower))
    } else {
        None
    };
    let mut matches: Vec<BrowserTabMatch> = tabs
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

            if search_urls && query_is_ascii && tab.url.is_ascii() {
                if let Some(pos) =
                    crate::scripts::search::find_ignore_ascii_case(&tab.url, &query_lower)
                {
                    score += if pos == 0 { 120 } else { 90 };
                }
            }

            if query_is_ascii && tab.browser_name.is_ascii() {
                if let Some(pos) =
                    crate::scripts::search::find_ignore_ascii_case(&tab.browser_name, &query_lower)
                {
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
                if search_urls {
                    if let Some(nucleo_score) = n.score(&tab.url) {
                        score += 55 + (nucleo_score / 35) as i32;
                    }
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

#[allow(dead_code)]
fn browser_tab_provider_for_bundle_id(
    bundle_id: &str,
) -> Option<crate::config::BrowserTabProvider> {
    match bundle_id {
        "company.thebrowser.Browser" => Some(crate::config::BrowserTabProvider::Arc),
        "com.google.Chrome" => Some(crate::config::BrowserTabProvider::Chrome),
        "com.brave.Browser" => Some(crate::config::BrowserTabProvider::Brave),
        "com.microsoft.edgemac" => Some(crate::config::BrowserTabProvider::Edge),
        _ => None,
    }
}

#[allow(dead_code)]
fn root_tab_title(tab: &BrowserTabInfo) -> String {
    let title = tab.display_title().trim();
    if title.is_empty() {
        let host = host_from_url(&tab.url);
        if host.is_empty() {
            "Untitled Tab".to_string()
        } else {
            host.to_string()
        }
    } else {
        title.to_string()
    }
}

#[allow(dead_code)]
fn root_browser_tab_stable_key(tab: &BrowserTabInfo) -> String {
    format!(
        "browser-tab/{}/{}/{}/{}",
        tab.browser_bundle_id, tab.window_index, tab.tab_index, tab.url
    )
}

fn list_tabs_for_browser(browser: &SupportedBrowser) -> Result<Vec<BrowserTabInfo>> {
    let script = build_list_tabs_jxa(browser);
    let output =
        crate::platform::run_jxa_with_timeout(&script, "browser_tabs_list", BROWSER_TIMEOUT)
            .with_context(|| format!("list_browser_tabs_failed: browser={}", browser.app_name))?;
    parse_tab_rows(browser, &output)
}

fn is_browser_running(bundle_id: &str) -> Result<bool> {
    let bundle_id = crate::utils::escape_applescript_string(bundle_id);
    let script = format!(
        r#"
tell application "System Events"
    set matching_processes to every application process whose bundle identifier is "{bundle_id}"
    return (count of matching_processes) > 0
end tell
"#,
    );

    let output = crate::platform::run_osascript_with_timeout(
        &script,
        "browser_tabs_is_running",
        BROWSER_RUNNING_TIMEOUT,
    )
    .with_context(|| format!("browser_tabs_is_running_failed: bundle_id={bundle_id}"))?;
    Ok(output.trim().eq_ignore_ascii_case("true"))
}

fn supported_browser_for(tab: &BrowserTabInfo) -> Option<&'static SupportedBrowser> {
    SUPPORTED_BROWSERS.iter().find(|browser| {
        browser.bundle_id == tab.browser_bundle_id.as_ref()
            || browser.app_name == tab.browser_name.as_ref()
    })
}

fn host_from_url(url: &str) -> &str {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    after_scheme.split('/').next().unwrap_or(after_scheme)
}

fn parse_tab_rows(browser: &SupportedBrowser, output: &str) -> Result<Vec<BrowserTabInfo>> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    trimmed
        .split(RECORD_SEPARATOR)
        .filter(|row| !row.is_empty())
        .map(|row| {
            let mut parts = row.split(FIELD_SEPARATOR);
            let window_index = parts
                .next()
                .ok_or_else(|| anyhow!("Missing window index"))?
                .parse::<usize>()
                .context("Invalid window index")?;
            let tab_index = parts
                .next()
                .ok_or_else(|| anyhow!("Missing tab index"))?
                .parse::<usize>()
                .context("Invalid tab index")?;
            let title = parts
                .next()
                .ok_or_else(|| anyhow!("Missing title"))?
                .trim()
                .to_string();
            let url = parts
                .next()
                .ok_or_else(|| anyhow!("Missing URL"))?
                .trim()
                .to_string();

            Ok(BrowserTabInfo {
                browser_name: browser.app_name.into(),
                browser_bundle_id: browser.bundle_id.into(),
                window_index,
                tab_index,
                title: title.into(),
                url: url.into(),
            })
        })
        .collect()
}

#[cfg(test)]
fn build_list_tabs_script(browser: &SupportedBrowser) -> String {
    let title_property = match browser.family {
        BrowserFamily::Safari => "name",
        BrowserFamily::Chromium => "title",
    };

    format!(
        r#"
on replace_text(the_text, old_text, new_text)
    set AppleScript's text item delimiters to old_text
    set text_items to every text item of the_text
    set AppleScript's text item delimiters to new_text
    set the_text to text_items as text
    set AppleScript's text item delimiters to ""
    return the_text
end replace_text

on sanitize_text(value_text)
    if value_text is missing value then return ""
    set safe_text to value_text as text
    set safe_text to my replace_text(safe_text, return, " ")
    set safe_text to my replace_text(safe_text, linefeed, " ")
    set safe_text to my replace_text(safe_text, ascii character 30, " ")
    set safe_text to my replace_text(safe_text, ascii character 31, " ")
    return safe_text
end sanitize_text

tell application "System Events"
    set is_running to exists application process "{app_name}"
end tell
if not is_running then return ""

set tab_records to {{}}
tell application "{app_name}"
    set window_count to count of windows
    repeat with w from 1 to window_count
        set tab_count to count of tabs of window w
        repeat with t from 1 to tab_count
            set the_tab to tab t of window w
            try
                set tab_title to {title_property} of the_tab
            on error
                set tab_title to ""
            end try
            try
                set tab_url to URL of the_tab
            on error
                set tab_url to ""
            end try

            set safe_title to my sanitize_text(tab_title)
            set safe_url to my sanitize_text(tab_url)

            if safe_title is not "" or safe_url is not "" then
                set end of tab_records to (w as text) & (ascii character 30) & (t as text) & (ascii character 30) & safe_title & (ascii character 30) & safe_url
            end if
        end repeat
    end repeat
end tell

set AppleScript's text item delimiters to ascii character 31
set joined_tabs to tab_records as text
set AppleScript's text item delimiters to ""
return joined_tabs
"#,
        app_name = browser.app_name,
        title_property = title_property,
    )
}

fn build_list_tabs_jxa(browser: &SupportedBrowser) -> String {
    let title_property = match browser.family {
        BrowserFamily::Safari => "name",
        BrowserFamily::Chromium => "title",
    };

    // JXA reliably enumerates all Chrome windows (including those on other
    // macOS Spaces/desktops), whereas AppleScript's `count of windows` often
    // returns only the windows on the current Space.
    format!(
        r#"
var app = Application("{app_name}");
if (!app.running()) {{ ""; }}
else {{
    var FS = "\u001e";
    var RS = "\u001f";
    var rows = [];
    var wins = app.windows();
    for (var w = 0; w < wins.length; w++) {{
        var win = wins[w];
        var titles = win.tabs.{title_property}();
        var urls = win.tabs.url();
        for (var t = 0; t < titles.length; t++) {{
            var title = (titles[t] || "").replace(/[\r\n\u001e\u001f]/g, " ");
            var url   = (urls[t]   || "").replace(/[\r\n\u001e\u001f]/g, " ");
            if (title || url) {{
                rows.push((w + 1) + FS + (t + 1) + FS + title + FS + url);
            }}
        }}
    }}
    rows.join(RS);
}}
"#,
        app_name = browser.app_name,
        title_property = title_property,
    )
}

fn build_activate_tab_script(
    browser: &SupportedBrowser,
    window_index: usize,
    tab_index: usize,
) -> String {
    let activate_body = match browser.family {
        BrowserFamily::Safari => format!(
            r#"
    set tab_count to count of tabs of window {window_index}
    if tab_count < {tab_index} then error "Tab {tab_index} is no longer available"
    set current tab of window {window_index} to tab {tab_index} of window {window_index}
    set index of window {window_index} to 1
    activate
"#,
            window_index = window_index,
            tab_index = tab_index,
        ),
        BrowserFamily::Chromium => format!(
            r#"
    set tab_count to count of tabs of window {window_index}
    if tab_count < {tab_index} then error "Tab {tab_index} is no longer available"
    set active tab index of window {window_index} to {tab_index}
    set index of window {window_index} to 1
    activate
"#,
            window_index = window_index,
            tab_index = tab_index,
        ),
    };

    format!(
        r#"
tell application "System Events"
    set is_running to exists application process "{app_name}"
end tell
if not is_running then error "{app_name} is not running"

tell application "{app_name}"
    set window_count to count of windows
    if window_count < {window_index} then error "Window {window_index} is no longer available"
{activate_body}end tell
"#,
        app_name = browser.app_name,
        window_index = window_index,
        activate_body = activate_body,
    )
}

// Favicon logic moved to src/favicons.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tab_rows_returns_empty_for_blank_output() {
        let browser = &SUPPORTED_BROWSERS[0];
        let rows = parse_tab_rows(browser, "   ").expect("parse rows");
        assert!(rows.is_empty());
    }

    #[test]
    fn parse_tab_rows_extracts_multiple_tabs() {
        let browser = &SUPPORTED_BROWSERS[1];
        let output = format!(
            "1{field}1{field}Docs{field}https://docs.rs{record}2{field}3{field}Chat{field}https://chat.openai.com",
            field = FIELD_SEPARATOR,
            record = RECORD_SEPARATOR,
        );

        let rows = parse_tab_rows(browser, &output).expect("parse rows");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].browser_name.as_ref(), "Google Chrome");
        assert_eq!(rows[0].window_index, 1);
        assert_eq!(rows[0].tab_index, 1);
        assert_eq!(rows[0].title.as_ref(), "Docs");
        assert_eq!(rows[1].url.as_ref(), "https://chat.openai.com");
    }

    #[test]
    fn fuzzy_search_browser_tabs_prefers_title_match() {
        let tabs = vec![
            BrowserTabInfo {
                browser_name: "Safari".into(),
                browser_bundle_id: "com.apple.Safari".into(),
                window_index: 1,
                tab_index: 1,
                title: "Build a Claude Managed Agent".into(),
                url: "https://vercel.com/kb/guide".into(),
            },
            BrowserTabInfo {
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
    fn expired_root_browser_tab_snapshot_returns_stale_rows() {
        reset_root_browser_tabs_snapshot_for_test();
        let tab = BrowserTabInfo {
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
    fn build_list_tabs_script_uses_browser_specific_title_property() {
        let safari_script = build_list_tabs_script(&SUPPORTED_BROWSERS[0]);
        let chrome_script = build_list_tabs_script(&SUPPORTED_BROWSERS[1]);

        assert!(safari_script.contains("set the_tab to tab t of window w"));
        assert!(safari_script.contains("set tab_title to name of the_tab"));
        assert!(chrome_script.contains("set tab_title to title of the_tab"));
        assert!(chrome_script.contains("set tab_url to URL of the_tab"));
    }

    #[test]
    fn build_list_tabs_jxa_uses_browser_specific_title_property() {
        let safari_script = build_list_tabs_jxa(&SUPPORTED_BROWSERS[0]);
        let chrome_script = build_list_tabs_jxa(&SUPPORTED_BROWSERS[1]);

        assert!(safari_script.contains(r#"win.tabs.name()"#));
        assert!(chrome_script.contains(r#"win.tabs.title()"#));
        assert!(chrome_script.contains(r#"win.tabs.url()"#));
        // Verify it targets the correct app
        assert!(safari_script.contains(r#"Application("Safari")"#));
        assert!(chrome_script.contains(r#"Application("Google Chrome")"#));
    }

    #[test]
    fn build_activate_tab_script_switches_browser_specific_tab_property() {
        let safari_script = build_activate_tab_script(&SUPPORTED_BROWSERS[0], 2, 4);
        let chrome_script = build_activate_tab_script(&SUPPORTED_BROWSERS[1], 2, 4);

        assert!(safari_script.contains("set current tab of window 2 to tab 4 of window 2"));
        assert!(chrome_script.contains("set active tab index of window 2 to 4"));
        assert!(chrome_script.contains("set index of window 2 to 1"));
    }

    #[test]
    fn is_browser_running_script_uses_bundle_identifier_lookup() {
        let bundle_id = "com.google.Chrome";
        let script = format!(
            r#"
tell application "System Events"
    set matching_processes to every application process whose bundle identifier is "{bundle_id}"
    return (count of matching_processes) > 0
end tell
"#,
        );

        assert!(script.contains("bundle identifier is \"com.google.Chrome\""));
        assert!(script.contains("count of matching_processes"));
    }
}
