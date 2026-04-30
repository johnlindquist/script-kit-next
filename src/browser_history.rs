use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;

const CHROMIUM_EPOCH_OFFSET_SECS: i64 = 11_644_473_600;
const SAFARI_EPOCH_OFFSET_SECS: i64 = 978_307_200;
const PER_BROWSER_LIMIT: usize = 250;
const HISTORY_CACHE_TTL: Duration = Duration::from_secs(30);

static HISTORY_LOAD_CACHE: LazyLock<Mutex<Option<BrowserHistoryLoadCache>>> =
    LazyLock::new(|| Mutex::new(None));
static HISTORY_SEARCH_CACHE: LazyLock<Mutex<Option<BrowserHistorySearchCache>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Clone)]
struct BrowserHistoryLoadCache {
    loaded_at: Instant,
    requested_limit: usize,
    entries: Vec<BrowserHistoryEntry>,
}

#[derive(Debug, Clone)]
struct BrowserHistorySearchCache {
    entries_ptr: usize,
    entries_len: usize,
    query: String,
    matches: Vec<BrowserHistoryMatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrowserHistoryFamily {
    Safari,
    Chromium,
    Firefox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SupportedBrowserHistory {
    app_name: &'static str,
    bundle_id: &'static str,
    family: BrowserHistoryFamily,
    profile_root: &'static str,
}

const SUPPORTED_BROWSERS: &[SupportedBrowserHistory] = &[
    SupportedBrowserHistory {
        app_name: "Safari",
        bundle_id: "com.apple.Safari",
        family: BrowserHistoryFamily::Safari,
        profile_root: "Library/Safari",
    },
    SupportedBrowserHistory {
        app_name: "Google Chrome",
        bundle_id: "com.google.Chrome",
        family: BrowserHistoryFamily::Chromium,
        profile_root: "Library/Application Support/Google/Chrome",
    },
    SupportedBrowserHistory {
        app_name: "Arc",
        bundle_id: "company.thebrowser.Browser",
        family: BrowserHistoryFamily::Chromium,
        profile_root: "Library/Application Support/Arc/User Data",
    },
    SupportedBrowserHistory {
        app_name: "Brave Browser",
        bundle_id: "com.brave.Browser",
        family: BrowserHistoryFamily::Chromium,
        profile_root: "Library/Application Support/BraveSoftware/Brave-Browser",
    },
    SupportedBrowserHistory {
        app_name: "Microsoft Edge",
        bundle_id: "com.microsoft.edgemac",
        family: BrowserHistoryFamily::Chromium,
        profile_root: "Library/Application Support/Microsoft Edge",
    },
    SupportedBrowserHistory {
        app_name: "Chromium",
        bundle_id: "org.chromium.Chromium",
        family: BrowserHistoryFamily::Chromium,
        profile_root: "Library/Application Support/Chromium",
    },
    SupportedBrowserHistory {
        app_name: "Vivaldi",
        bundle_id: "com.vivaldi.Vivaldi",
        family: BrowserHistoryFamily::Chromium,
        profile_root: "Library/Application Support/Vivaldi",
    },
    SupportedBrowserHistory {
        app_name: "Opera",
        bundle_id: "com.operasoftware.Opera",
        family: BrowserHistoryFamily::Chromium,
        profile_root: "Library/Application Support/com.operasoftware.Opera",
    },
    SupportedBrowserHistory {
        app_name: "Firefox",
        bundle_id: "org.mozilla.firefox",
        family: BrowserHistoryFamily::Firefox,
        profile_root: "Library/Application Support/Firefox/Profiles",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserHistoryEntry {
    pub browser_name: String,
    pub browser_bundle_id: String,
    pub title: String,
    pub url: String,
    pub host: String,
    pub last_visited_at_ms: i64,
    pub visit_count: i64,
    pub profile: String,
}

impl BrowserHistoryEntry {
    pub fn display_title(&self) -> &str {
        let trimmed = self.title.trim();
        if !trimmed.is_empty() {
            trimmed
        } else if !self.host.trim().is_empty() {
            self.host.trim()
        } else if !self.url.trim().is_empty() {
            self.url.trim()
        } else {
            "Untitled History Entry"
        }
    }

    pub fn history_key(&self) -> String {
        format!("{}:{}:{}", self.browser_bundle_id, self.profile, self.url)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserHistoryMatch {
    pub entry: BrowserHistoryEntry,
    pub score: i32,
}

pub fn list_recent_history(limit: usize) -> Result<Vec<BrowserHistoryEntry>> {
    let final_limit = limit.max(1);

    if let Some(cached) = load_history_from_cache(final_limit) {
        return Ok(cached);
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("HOME is not set"))?;
    let entries = list_recent_history_from_home(&home, final_limit)?;
    store_history_load_cache(final_limit, &entries);
    Ok(entries)
}

fn list_recent_history_from_home(home: &Path, limit: usize) -> Result<Vec<BrowserHistoryEntry>> {
    let final_limit = limit.max(1);
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    for browser in SUPPORTED_BROWSERS {
        let db_paths = history_db_paths_for_browser(browser, home);
        if db_paths.is_empty() {
            continue;
        }

        match load_history_for_browser(browser, &db_paths, PER_BROWSER_LIMIT) {
            Ok(mut browser_entries) => entries.append(&mut browser_entries),
            Err(error) => errors.push(format!("{}: {error}", browser.app_name)),
        }
    }

    if entries.is_empty() && !errors.is_empty() {
        bail!("Failed to read browser history: {}", errors.join(" | "));
    }

    entries.sort_by(|a, b| {
        b.last_visited_at_ms
            .cmp(&a.last_visited_at_ms)
            .then_with(|| a.browser_name.cmp(&b.browser_name))
            .then_with(|| a.url.cmp(&b.url))
    });

    Ok(dedupe_history_entries(entries, final_limit))
}

pub fn fuzzy_search_browser_history(
    entries: &[BrowserHistoryEntry],
    query: &str,
) -> Vec<BrowserHistoryMatch> {
    let query_trimmed = query.trim();
    if let Some(cached) = load_history_search_cache(entries, query_trimmed) {
        return cached;
    }

    let matches = fuzzy_search_browser_history_uncached(entries, query_trimmed);
    store_history_search_cache(entries, query_trimmed, &matches);
    matches
}

fn fuzzy_search_browser_history_uncached(
    entries: &[BrowserHistoryEntry],
    query: &str,
) -> Vec<BrowserHistoryMatch> {
    if query.trim().is_empty() {
        return entries
            .iter()
            .cloned()
            .map(|entry| BrowserHistoryMatch { entry, score: 0 })
            .collect();
    }

    let query_lower = query.trim().to_lowercase();
    let query_is_ascii = query_lower.is_ascii();
    let use_nucleo = query_lower.len() >= crate::scripts::search::MIN_FUZZY_QUERY_LEN;
    let mut nucleo = crate::scripts::NucleoCtx::new(&query_lower);
    let mut matches = Vec::with_capacity(entries.len());

    for entry in entries {
        let mut score = 0i32;

        if query_is_ascii && entry.title.is_ascii() {
            if let Some(pos) =
                crate::scripts::search::find_ignore_ascii_case(&entry.title, &query_lower)
            {
                score += if pos == 0 { 260 } else { 210 };
            }
        }

        if query_is_ascii && entry.host.is_ascii() {
            if let Some(pos) =
                crate::scripts::search::find_ignore_ascii_case(&entry.host, &query_lower)
            {
                score += if pos == 0 { 220 } else { 170 };
            }
        }

        if query_is_ascii && entry.url.is_ascii() {
            if let Some(pos) =
                crate::scripts::search::find_ignore_ascii_case(&entry.url, &query_lower)
            {
                score += if pos == 0 { 140 } else { 110 };
            }
        }

        if query_is_ascii && entry.browser_name.is_ascii() {
            if let Some(pos) =
                crate::scripts::search::find_ignore_ascii_case(&entry.browser_name, &query_lower)
            {
                score += if pos == 0 { 90 } else { 60 };
            }
        }

        if use_nucleo {
            if let Some(nucleo_score) = nucleo.score(&entry.title) {
                score += 120 + (nucleo_score / 18) as i32;
            }
            if !entry.host.is_empty() {
                if let Some(nucleo_score) = nucleo.score(&entry.host) {
                    score += 85 + (nucleo_score / 24) as i32;
                }
            }
            if let Some(nucleo_score) = nucleo.score(&entry.url) {
                score += 60 + (nucleo_score / 32) as i32;
            }
        }

        if score > 0 {
            score += recency_bonus(entry.last_visited_at_ms);
            matches.push(BrowserHistoryMatch {
                entry: entry.clone(),
                score,
            });
        }
    }

    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => match b.entry.last_visited_at_ms.cmp(&a.entry.last_visited_at_ms) {
            Ordering::Equal => match a.entry.browser_name.cmp(&b.entry.browser_name) {
                Ordering::Equal => a.entry.url.cmp(&b.entry.url),
                other => other,
            },
            other => other,
        },
        other => other,
    });

    matches
}

fn load_history_from_cache(limit: usize) -> Option<Vec<BrowserHistoryEntry>> {
    let cache = HISTORY_LOAD_CACHE.lock().ok()?;
    let cached = cache.as_ref()?;
    if cached.loaded_at.elapsed() > HISTORY_CACHE_TTL {
        return None;
    }

    let cache_is_complete = cached.entries.len() < cached.requested_limit;
    if cached.requested_limit >= limit || cache_is_complete {
        return Some(cached.entries.iter().take(limit).cloned().collect());
    }

    None
}

fn store_history_load_cache(limit: usize, entries: &[BrowserHistoryEntry]) {
    if let Ok(mut cache) = HISTORY_LOAD_CACHE.lock() {
        *cache = Some(BrowserHistoryLoadCache {
            loaded_at: Instant::now(),
            requested_limit: limit,
            entries: entries.to_vec(),
        });
    }
}

fn load_history_search_cache(
    entries: &[BrowserHistoryEntry],
    query: &str,
) -> Option<Vec<BrowserHistoryMatch>> {
    let cache = HISTORY_SEARCH_CACHE.lock().ok()?;
    let cached = cache.as_ref()?;
    let entries_ptr = entries.as_ptr() as usize;
    if cached.entries_ptr == entries_ptr
        && cached.entries_len == entries.len()
        && cached.query == query
    {
        return Some(cached.matches.clone());
    }

    None
}

fn store_history_search_cache(
    entries: &[BrowserHistoryEntry],
    query: &str,
    matches: &[BrowserHistoryMatch],
) {
    if let Ok(mut cache) = HISTORY_SEARCH_CACHE.lock() {
        *cache = Some(BrowserHistorySearchCache {
            entries_ptr: entries.as_ptr() as usize,
            entries_len: entries.len(),
            query: query.to_string(),
            matches: matches.to_vec(),
        });
    }
}

fn dedupe_history_entries(
    entries: Vec<BrowserHistoryEntry>,
    limit: usize,
) -> Vec<BrowserHistoryEntry> {
    let mut deduped = Vec::with_capacity(entries.len());
    let mut seen_exact = HashSet::new();
    let mut seen_urls = HashSet::new();
    let mut seen_title_host = HashSet::new();

    for entry in entries {
        let exact_key = entry.history_key();
        if !seen_exact.insert(exact_key) {
            continue;
        }

        let normalized_url = normalized_url_key(&entry.url);
        if normalized_url
            .as_ref()
            .is_some_and(|key| seen_urls.contains(key))
        {
            continue;
        }

        let title_host = normalized_title_host_key(&entry);
        if title_host
            .as_ref()
            .is_some_and(|key| seen_title_host.contains(key))
        {
            continue;
        }

        if let Some(key) = normalized_url {
            seen_urls.insert(key);
        }
        if let Some(key) = title_host {
            seen_title_host.insert(key);
        }

        deduped.push(entry);
        if deduped.len() >= limit {
            break;
        }
    }

    deduped
}

fn normalized_url_key(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_fragment = trimmed.split('#').next().unwrap_or(trimmed);
    let normalized = without_fragment.trim_end_matches('/').to_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalized_title_host_key(entry: &BrowserHistoryEntry) -> Option<String> {
    let title = entry.display_title().trim().to_lowercase();
    let host = entry.host.trim().to_lowercase();
    if title.is_empty() || host.is_empty() {
        return None;
    }

    Some(format!("{host}::{title}"))
}

pub fn format_history_timestamp(last_visited_at_ms: i64) -> String {
    if last_visited_at_ms <= 0 {
        return "Unknown time".to_string();
    }

    let Some(utc) = DateTime::<Utc>::from_timestamp_millis(last_visited_at_ms) else {
        return "Unknown time".to_string();
    };
    let local = utc.with_timezone(&Local);
    local.format("%Y-%m-%d %H:%M").to_string()
}

fn recency_bonus(last_visited_at_ms: i64) -> i32 {
    let Some(utc) = DateTime::<Utc>::from_timestamp_millis(last_visited_at_ms) else {
        return 0;
    };
    let age_hours = (Utc::now() - utc).num_hours();
    if age_hours <= 24 {
        40
    } else if age_hours <= 24 * 7 {
        25
    } else if age_hours <= 24 * 30 {
        10
    } else {
        0
    }
}

fn history_db_paths_for_browser(browser: &SupportedBrowserHistory, home: &Path) -> Vec<PathBuf> {
    let root = home.join(browser.profile_root);
    match browser.family {
        BrowserHistoryFamily::Safari => {
            let path = root.join("History.db");
            if path.exists() {
                vec![path]
            } else {
                Vec::new()
            }
        }
        BrowserHistoryFamily::Firefox => collect_profile_db_paths(&root, "places.sqlite"),
        BrowserHistoryFamily::Chromium => {
            let mut paths = collect_profile_db_paths(&root, "History");
            let root_history = root.join("History");
            if root_history.exists() && !paths.iter().any(|path| path == &root_history) {
                paths.push(root_history);
            }
            paths
        }
    }
}

fn collect_profile_db_paths(root: &Path, db_name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }

    let direct = root.join(db_name);
    if direct.exists() {
        out.push(direct);
    }

    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return out,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let db_path = path.join(db_name);
        if db_path.exists() {
            out.push(db_path);
        }
    }

    out.sort();
    out
}

fn load_history_for_browser(
    browser: &SupportedBrowserHistory,
    db_paths: &[PathBuf],
    per_browser_limit: usize,
) -> Result<Vec<BrowserHistoryEntry>> {
    let mut entries = Vec::new();
    for db_path in db_paths {
        let profile = db_path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Default")
            .to_string();

        let mut profile_entries =
            load_history_from_db(browser, db_path, &profile, per_browser_limit).with_context(
                || format!("load_browser_history_failed: path={}", db_path.display()),
            )?;
        entries.append(&mut profile_entries);
    }
    Ok(entries)
}

fn load_history_from_db(
    browser: &SupportedBrowserHistory,
    db_path: &Path,
    profile: &str,
    limit: usize,
) -> Result<Vec<BrowserHistoryEntry>> {
    let temp_dir = tempfile::tempdir().context("create temp dir for browser history db copy")?;
    let copied_db = copy_sqlite_db_snapshot(db_path, temp_dir.path())?;
    let conn = Connection::open_with_flags(
        &copied_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .with_context(|| {
        format!(
            "open_browser_history_db_failed: path={}",
            copied_db.display()
        )
    })?;

    match browser.family {
        BrowserHistoryFamily::Chromium => query_chromium_history(&conn, browser, profile, limit),
        BrowserHistoryFamily::Safari => query_safari_history(&conn, browser, profile, limit),
        BrowserHistoryFamily::Firefox => query_firefox_history(&conn, browser, profile, limit),
    }
}

fn copy_sqlite_db_snapshot(db_path: &Path, dest_root: &Path) -> Result<PathBuf> {
    let db_name = db_path
        .file_name()
        .ok_or_else(|| anyhow!("missing database filename"))?;
    let dest_db = dest_root.join(db_name);
    std::fs::copy(db_path, &dest_db)
        .with_context(|| format!("copy_browser_history_db_failed: {}", db_path.display()))?;

    for suffix in ["-wal", "-shm"] {
        let candidate = PathBuf::from(format!("{}{}", db_path.display(), suffix));
        if candidate.exists() {
            let candidate_name = candidate
                .file_name()
                .ok_or_else(|| anyhow!("missing sqlite sidecar filename"))?;
            let dest = dest_root.join(candidate_name);
            let _ = std::fs::copy(&candidate, dest);
        }
    }

    Ok(dest_db)
}

fn query_chromium_history(
    conn: &Connection,
    browser: &SupportedBrowserHistory,
    profile: &str,
    limit: usize,
) -> Result<Vec<BrowserHistoryEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            urls.url,
            COALESCE(urls.title, ''),
            COALESCE(urls.visit_count, 0),
            MAX(visits.visit_time)
        FROM urls
        JOIN visits ON visits.url = urls.id
        GROUP BY urls.id
        ORDER BY MAX(visits.visit_time) DESC
        LIMIT ?1
        "#,
    )?;

    let rows = stmt.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get(1)?;
        let visit_count: i64 = row.get(2)?;
        let visit_time: i64 = row.get(3)?;
        Ok(browser_history_entry(
            browser,
            profile,
            title,
            url,
            visit_count,
            chromium_visit_time_to_unix_ms(visit_time),
        ))
    })?;

    collect_rows(rows)
}

fn query_safari_history(
    conn: &Connection,
    browser: &SupportedBrowserHistory,
    profile: &str,
    limit: usize,
) -> Result<Vec<BrowserHistoryEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            history_items.url,
            COALESCE(MAX(NULLIF(history_visits.title, '')), ''),
            COUNT(history_visits.id),
            MAX(history_visits.visit_time)
        FROM history_items
        JOIN history_visits ON history_visits.history_item = history_items.id
        GROUP BY history_items.id
        ORDER BY MAX(history_visits.visit_time) DESC
        LIMIT ?1
        "#,
    )?;

    let rows = stmt.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get(1)?;
        let visit_count: i64 = row.get(2)?;
        let visit_time: f64 = row.get(3)?;
        Ok(browser_history_entry(
            browser,
            profile,
            title,
            url,
            visit_count,
            safari_visit_time_to_unix_ms(visit_time),
        ))
    })?;

    collect_rows(rows)
}

fn query_firefox_history(
    conn: &Connection,
    browser: &SupportedBrowserHistory,
    profile: &str,
    limit: usize,
) -> Result<Vec<BrowserHistoryEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            moz_places.url,
            COALESCE(moz_places.title, ''),
            COALESCE(moz_places.visit_count, 0),
            MAX(moz_historyvisits.visit_date)
        FROM moz_places
        JOIN moz_historyvisits ON moz_historyvisits.place_id = moz_places.id
        WHERE moz_places.hidden = 0
        GROUP BY moz_places.id
        ORDER BY MAX(moz_historyvisits.visit_date) DESC
        LIMIT ?1
        "#,
    )?;

    let rows = stmt.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
        let url: String = row.get(0)?;
        let title: String = row.get(1)?;
        let visit_count: i64 = row.get(2)?;
        let visit_time: i64 = row.get(3)?;
        Ok(browser_history_entry(
            browser,
            profile,
            title,
            url,
            visit_count,
            firefox_visit_time_to_unix_ms(visit_time),
        ))
    })?;

    collect_rows(rows)
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn browser_history_entry(
    browser: &SupportedBrowserHistory,
    profile: &str,
    title: String,
    url: String,
    visit_count: i64,
    last_visited_at_ms: i64,
) -> BrowserHistoryEntry {
    BrowserHistoryEntry {
        browser_name: browser.app_name.to_string(),
        browser_bundle_id: browser.bundle_id.to_string(),
        host: host_from_url(&url).to_string(),
        title,
        url,
        last_visited_at_ms,
        visit_count,
        profile: profile.to_string(),
    }
}

fn host_from_url(url: &str) -> &str {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    after_scheme.split('/').next().unwrap_or(after_scheme)
}

fn chromium_visit_time_to_unix_ms(visit_time: i64) -> i64 {
    ((visit_time / 1_000_000) - CHROMIUM_EPOCH_OFFSET_SECS) * 1000
}

fn safari_visit_time_to_unix_ms(visit_time: f64) -> i64 {
    ((visit_time as i64) + SAFARI_EPOCH_OFFSET_SECS) * 1000
}

fn firefox_visit_time_to_unix_ms(visit_time: i64) -> i64 {
    visit_time / 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fuzzy_search_prefers_title_match_over_browser_name_only() {
        let entries = vec![
            BrowserHistoryEntry {
                browser_name: "Google Chrome".to_string(),
                browser_bundle_id: "com.google.Chrome".to_string(),
                title: "Script Kit browser history portal".to_string(),
                url: "https://example.com/script-kit".to_string(),
                host: "example.com".to_string(),
                last_visited_at_ms: Utc::now().timestamp_millis(),
                visit_count: 3,
                profile: "Default".to_string(),
            },
            BrowserHistoryEntry {
                browser_name: "Chrome".to_string(),
                browser_bundle_id: "com.google.Chrome".to_string(),
                title: "Home".to_string(),
                url: "https://example.com/browser-portal".to_string(),
                host: "example.com".to_string(),
                last_visited_at_ms: Utc::now().timestamp_millis(),
                visit_count: 2,
                profile: "Default".to_string(),
            },
        ];

        let matches = fuzzy_search_browser_history(&entries, "portal");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].entry.title, "Script Kit browser history portal");
    }

    #[test]
    fn chromium_timestamp_converts_to_unix_ms() {
        let utc = Utc
            .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
            .single()
            .unwrap_or_else(|| panic!("expected stable timestamp"));
        let visit_time = (utc.timestamp() + CHROMIUM_EPOCH_OFFSET_SECS) * 1_000_000;
        assert_eq!(
            chromium_visit_time_to_unix_ms(visit_time),
            utc.timestamp_millis()
        );
    }

    #[test]
    fn history_db_paths_finds_chromium_profiles() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let root = temp
            .path()
            .join("Library/Application Support/Google/Chrome");
        std::fs::create_dir_all(root.join("Default"))
            .unwrap_or_else(|error| panic!("create default profile failed: {error}"));
        std::fs::create_dir_all(root.join("Profile 1"))
            .unwrap_or_else(|error| panic!("create second profile failed: {error}"));
        std::fs::write(root.join("Default/History"), "")
            .unwrap_or_else(|error| panic!("write default history failed: {error}"));
        std::fs::write(root.join("Profile 1/History"), "")
            .unwrap_or_else(|error| panic!("write profile history failed: {error}"));

        let paths = history_db_paths_for_browser(&SUPPORTED_BROWSERS[1], temp.path());
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn dedupe_history_entries_collapses_normalized_url_duplicates() {
        let newer = BrowserHistoryEntry {
            browser_name: "Google Chrome".to_string(),
            browser_bundle_id: "com.google.Chrome".to_string(),
            title: "Portal".to_string(),
            url: "https://example.com/docs#intro".to_string(),
            host: "example.com".to_string(),
            last_visited_at_ms: Utc::now().timestamp_millis(),
            visit_count: 5,
            profile: "Default".to_string(),
        };
        let older = BrowserHistoryEntry {
            last_visited_at_ms: newer.last_visited_at_ms - 10_000,
            url: "https://example.com/docs/".to_string(),
            visit_count: 2,
            ..newer.clone()
        };

        let deduped = dedupe_history_entries(vec![newer.clone(), older], 10);
        assert_eq!(deduped, vec![newer]);
    }

    #[test]
    fn dedupe_history_entries_collapses_same_title_and_host_across_browsers() {
        let newer = BrowserHistoryEntry {
            browser_name: "Google Chrome".to_string(),
            browser_bundle_id: "com.google.Chrome".to_string(),
            title: "Inbox (1,626) - johnlindquist@gmail.com - Gmail".to_string(),
            url: "https://mail.google.com/mail/u/0/#inbox".to_string(),
            host: "mail.google.com".to_string(),
            last_visited_at_ms: Utc::now().timestamp_millis(),
            visit_count: 7,
            profile: "Default".to_string(),
        };
        let older = BrowserHistoryEntry {
            browser_name: "Arc".to_string(),
            browser_bundle_id: "company.thebrowser.Browser".to_string(),
            url: "https://mail.google.com/mail/u/1/#inbox".to_string(),
            profile: "Profile 1".to_string(),
            last_visited_at_ms: newer.last_visited_at_ms - 5_000,
            ..newer.clone()
        };

        let deduped = dedupe_history_entries(vec![newer.clone(), older], 10);
        assert_eq!(deduped, vec![newer]);
    }
}
