use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use anyhow::{anyhow, bail, Context, Result};

const FIELD_SEPARATOR: char = '\u{1e}';
const RECORD_SEPARATOR: char = '\u{1f}';

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTabInfo {
    pub browser_name: String,
    pub browser_bundle_id: String,
    pub window_index: usize,
    pub tab_index: usize,
    pub title: String,
    pub url: String,
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

pub fn list_open_tabs() -> Result<Vec<BrowserTabInfo>> {
    let mut tabs = Vec::new();
    let mut errors = Vec::new();

    for browser in SUPPORTED_BROWSERS {
        match is_browser_running(browser.bundle_id) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(error) => {
                errors.push(format!("{}: {error}", browser.app_name));
                continue;
            }
        }

        match list_tabs_for_browser(browser) {
            Ok(mut browser_tabs) => tabs.append(&mut browser_tabs),
            Err(error) => errors.push(format!("{}: {error}", browser.app_name)),
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
                tab.browser_bundle_id.as_str()
            } else {
                tab.browser_name.as_str()
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
    let mut nucleo = crate::scripts::NucleoCtx::new(&query_lower);
    let mut matches = Vec::with_capacity(tabs.len());

    for tab in tabs {
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
            if let Some(pos) = crate::scripts::search::find_ignore_ascii_case(host, &query_lower) {
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
            if let Some(pos) =
                crate::scripts::search::find_ignore_ascii_case(&tab.browser_name, &query_lower)
            {
                score += if pos == 0 { 80 } else { 55 };
            }
        }

        if use_nucleo {
            if let Some(nucleo_score) = nucleo.score(&tab.title) {
                score += 110 + (nucleo_score / 20) as i32;
            }
            if !host.is_empty() {
                if let Some(nucleo_score) = nucleo.score(host) {
                    score += 70 + (nucleo_score / 28) as i32;
                }
            }
            if let Some(nucleo_score) = nucleo.score(&tab.url) {
                score += 55 + (nucleo_score / 35) as i32;
            }
            if let Some(nucleo_score) = nucleo.score(&tab.browser_name) {
                score += 35 + (nucleo_score / 40) as i32;
            }
        }

        if score > 0 {
            matches.push(BrowserTabMatch {
                tab: tab.clone(),
                score,
            });
        }
    }

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

fn list_tabs_for_browser(browser: &SupportedBrowser) -> Result<Vec<BrowserTabInfo>> {
    let script = build_list_tabs_jxa(browser);
    let output = crate::platform::run_jxa(&script, "browser_tabs_list")
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

    let output = crate::platform::run_osascript(&script, "browser_tabs_is_running")
        .with_context(|| format!("browser_tabs_is_running_failed: bundle_id={bundle_id}"))?;
    Ok(output.trim().eq_ignore_ascii_case("true"))
}

fn supported_browser_for(tab: &BrowserTabInfo) -> Option<&'static SupportedBrowser> {
    SUPPORTED_BROWSERS.iter().find(|browser| {
        browser.bundle_id == tab.browser_bundle_id || browser.app_name == tab.browser_name
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
                browser_name: browser.app_name.to_string(),
                browser_bundle_id: browser.bundle_id.to_string(),
                window_index,
                tab_index,
                title,
                url,
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
var SE  = Application("System Events");
var procs = SE.applicationProcesses.whose({{bundleIdentifier: "{bundle_id}"}});
if (procs.length === 0) {{ ""; }}
else {{
    var FS = "\u001e";
    var RS = "\u001f";
    var rows = [];
    var wins = app.windows();
    for (var w = 0; w < wins.length; w++) {{
        var tabs = wins[w].tabs();
        for (var t = 0; t < tabs.length; t++) {{
            var title = "";
            var url   = "";
            try {{ title = tabs[t].{title_property}() || ""; }} catch(e) {{}}
            try {{ url   = tabs[t].url()   || ""; }} catch(e) {{}}
            title = title.replace(/[\r\n\u001e\u001f]/g, " ");
            url   = url.replace(/[\r\n\u001e\u001f]/g, " ");
            if (title || url) {{
                rows.push((w + 1) + FS + (t + 1) + FS + title + FS + url);
            }}
        }}
    }}
    rows.join(RS);
}}
"#,
        app_name = browser.app_name,
        bundle_id = browser.bundle_id,
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

// ── Favicon cache ──────────────────────────────────────────────────────

/// In-memory cache of decoded favicon images keyed by domain.
/// `None` means a fetch was attempted but failed.
static FAVICON_CACHE: LazyLock<Mutex<HashMap<String, Option<Arc<gpui::RenderImage>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Look up a cached favicon for the given URL's domain.
/// Returns `None` if not yet fetched or fetch failed.
pub fn cached_favicon(url: &str) -> Option<Arc<gpui::RenderImage>> {
    let domain = domain_from_url(url)?;
    let cache = FAVICON_CACHE.lock().ok()?;
    cache.get(domain)?.clone()
}

/// Extract the host from a URL (e.g. "https://docs.google.com/foo" → "docs.google.com").
fn domain_from_url(url: &str) -> Option<&str> {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host = after_scheme.split('/').next().unwrap_or(after_scheme);
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

/// Return the list of unique domains from `tabs` that are not yet in the cache.
pub fn domains_needing_favicons(tabs: &[BrowserTabInfo]) -> Vec<String> {
    let cache = FAVICON_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let mut seen = std::collections::HashSet::new();
    let mut domains = Vec::new();

    for tab in tabs {
        if let Some(domain) = domain_from_url(&tab.url) {
            let d = domain.to_string();
            if !cache.contains_key(&d) && seen.insert(d.clone()) {
                domains.push(d);
            }
        }
    }
    domains
}

/// Fetch favicons for the given domains (blocking). Call from a background thread.
/// Results are stored in the static `FAVICON_CACHE`.
pub fn fetch_favicons_blocking(domains: &[String]) {
    let agent = ureq::Agent::new_with_defaults();

    for domain in domains {
        let url = format!("https://www.google.com/s2/favicons?domain={}&sz=64", domain);

        let result = (|| -> Option<Arc<gpui::RenderImage>> {
            let response = agent.get(&url).call().ok()?;
            let body = response.into_body().read_to_vec().ok()?;
            if body.len() < 100 {
                // Too small — likely a placeholder/error
                return None;
            }
            crate::list_item::decode_png_to_render_image(&body).ok()
        })();

        let mut cache = FAVICON_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        cache.insert(domain.clone(), result);
    }
}

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
        assert_eq!(rows[0].browser_name, "Google Chrome");
        assert_eq!(rows[0].window_index, 1);
        assert_eq!(rows[0].tab_index, 1);
        assert_eq!(rows[0].title, "Docs");
        assert_eq!(rows[1].url, "https://chat.openai.com");
    }

    #[test]
    fn fuzzy_search_browser_tabs_prefers_title_match() {
        let tabs = vec![
            BrowserTabInfo {
                browser_name: "Safari".to_string(),
                browser_bundle_id: "com.apple.Safari".to_string(),
                window_index: 1,
                tab_index: 1,
                title: "Build a Claude Managed Agent".to_string(),
                url: "https://vercel.com/kb/guide".to_string(),
            },
            BrowserTabInfo {
                browser_name: "Google Chrome".to_string(),
                browser_bundle_id: "com.google.Chrome".to_string(),
                window_index: 1,
                tab_index: 2,
                title: "Home".to_string(),
                url: "https://claude-managed-agent.example.com".to_string(),
            },
        ];

        let matches = fuzzy_search_browser_tabs(&tabs, "managed agent");
        assert_eq!(matches.len(), 2);
        assert_eq!(
            matches[0].tab.title, "Build a Claude Managed Agent",
            "title hit should outrank URL-only hit"
        );
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

        assert!(safari_script.contains(r#"tabs[t].name()"#));
        assert!(chrome_script.contains(r#"tabs[t].title()"#));
        assert!(chrome_script.contains(r#"tabs[t].url()"#));
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
