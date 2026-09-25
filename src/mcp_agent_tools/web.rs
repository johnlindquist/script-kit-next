//! `web_search` and `web_fetch` for agent sessions.
//!
//! Search reads DuckDuckGo's HTML results (no API key needed) and falls back
//! to Bing's RSS results when DuckDuckGo returns nothing. Fetch returns a
//! page's readable text.

use std::io::Read as _;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use serde_json::Value;

use super::arg_str;

const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) ScriptKit";
const MAX_SEARCH_BODY: u64 = 2 * 1024 * 1024;
const MAX_FETCH_BODY: u64 = 5 * 1024 * 1024;
const DEFAULT_RESULTS: usize = 6;
const DEFAULT_FETCH_CHARS: usize = 8_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(20)))
        .timeout_connect(Some(Duration::from_secs(8)))
        .build()
        .new_agent()
}

fn get_text(url: &str, limit: u64) -> Result<(u16, String, String), String> {
    // Web tools reach the public internet on the model's behalf, the same
    // effect class as a provider call.
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Provider)
        .map_err(|refusal| refusal.to_string())?;
    let response = agent()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept-Language", "en-US,en;q=0.9")
        .call()
        .map_err(|error| format!("Request failed: {error}"))?;
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut bytes = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Reading the response failed: {error}"))?;
    Ok((
        status,
        content_type,
        String::from_utf8_lossy(&bytes).into_owned(),
    ))
}

pub(super) fn web_search(args: &Value) -> Result<String, String> {
    let query = arg_str(args, "query").ok_or("web_search needs a query")?;
    let max_results = args
        .get("max_results")
        .and_then(Value::as_u64)
        .map(|value| value.clamp(1, 10) as usize)
        .unwrap_or(DEFAULT_RESULTS);

    let mut results = duckduckgo_html(query).unwrap_or_default();
    if results.is_empty() {
        results = bing_rss(query).unwrap_or_default();
    }
    if results.is_empty() {
        return Err(format!("No web results found for \"{query}\"."));
    }
    results.truncate(max_results);
    Ok(format_results(query, &results))
}

fn format_results(query: &str, results: &[SearchResult]) -> String {
    let mut out = format!("Web results for \"{query}\":\n");
    for (index, result) in results.iter().enumerate() {
        out.push_str(&format!(
            "\n{}. {}\n   {}\n",
            index + 1,
            result.title,
            result.url
        ));
        if !result.snippet.is_empty() {
            out.push_str(&format!("   {}\n", result.snippet));
        }
    }
    out
}

fn bing_rss(query: &str) -> Result<Vec<SearchResult>, String> {
    let url = format!(
        "https://www.bing.com/search?format=rss&setlang=en-US&q={}",
        percent_encode(query)
    );
    let (status, _, body) = get_text(&url, MAX_SEARCH_BODY)?;
    if status != 200 {
        return Err(format!("search returned HTTP {status}"));
    }
    Ok(parse_bing_rss(&body))
}

pub(super) fn parse_bing_rss(body: &str) -> Vec<SearchResult> {
    static ITEM: LazyLock<Option<Regex>> =
        LazyLock::new(|| Regex::new(r"(?s)<item>(.*?)</item>").ok());
    let Some(item_pattern) = ITEM.as_ref() else {
        return Vec::new();
    };
    item_pattern
        .captures_iter(body)
        .filter_map(|item| {
            let item = item.get(1)?.as_str();
            let title = xml_field(item, "title")?;
            let url = xml_field(item, "link")?;
            let snippet = xml_field(item, "description").unwrap_or_default();
            (!title.is_empty() && url.starts_with("http")).then_some(SearchResult {
                title,
                url,
                snippet,
            })
        })
        .collect()
}

fn xml_field(item: &str, tag: &str) -> Option<String> {
    let start = item.find(&format!("<{tag}>"))? + tag.len() + 2;
    let end = start + item[start..].find(&format!("</{tag}>"))?;
    let raw = item[start..end].trim();
    let raw = raw
        .strip_prefix("<![CDATA[")
        .and_then(|value| value.strip_suffix("]]>"))
        .unwrap_or(raw);
    Some(collapse_whitespace(&decode_entities(&strip_tags(raw))))
}

fn duckduckgo_html(query: &str) -> Result<Vec<SearchResult>, String> {
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        percent_encode(query)
    );
    let (status, _, body) = get_text(&url, MAX_SEARCH_BODY)?;
    if status != 200 {
        return Err(format!("search returned HTTP {status}"));
    }
    Ok(parse_duckduckgo_html(&body))
}

pub(super) fn parse_duckduckgo_html(body: &str) -> Vec<SearchResult> {
    static LINK: LazyLock<Option<Regex>> = LazyLock::new(|| {
        Regex::new(r#"(?s)<a[^>]*class="result__a"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#).ok()
    });
    static SNIPPET: LazyLock<Option<Regex>> =
        LazyLock::new(|| Regex::new(r#"(?s)<a[^>]*class="result__snippet"[^>]*>(.*?)</a>"#).ok());
    let (Some(link_pattern), Some(snippet_pattern)) = (LINK.as_ref(), SNIPPET.as_ref()) else {
        return Vec::new();
    };
    let snippets: Vec<String> = snippet_pattern
        .captures_iter(body)
        .map(|capture| collapse_whitespace(&decode_entities(&strip_tags(&capture[1]))))
        .collect();
    link_pattern
        .captures_iter(body)
        .enumerate()
        .filter_map(|(index, capture)| {
            let url = duckduckgo_target(&decode_entities(&capture[1]))?;
            let title = collapse_whitespace(&decode_entities(&strip_tags(&capture[2])));
            (!title.is_empty()).then(|| SearchResult {
                title,
                url,
                snippet: snippets.get(index).cloned().unwrap_or_default(),
            })
        })
        .collect()
}

/// DuckDuckGo wraps result links in a redirect carrying the target in `uddg`.
fn duckduckgo_target(href: &str) -> Option<String> {
    let absolute = if href.starts_with("//") {
        format!("https:{href}")
    } else {
        href.to_string()
    };
    let parsed = url::Url::parse(&absolute).ok()?;
    if parsed
        .host_str()
        .is_some_and(|host| host.ends_with("duckduckgo.com"))
    {
        parsed
            .query_pairs()
            .find(|(key, _)| key == "uddg")
            .map(|(_, value)| value.into_owned())
    } else {
        Some(absolute)
    }
}

pub(super) fn web_fetch(args: &Value) -> Result<String, String> {
    let raw_url = arg_str(args, "url").ok_or("web_fetch needs a url")?;
    let url = url::Url::parse(raw_url).map_err(|error| format!("Invalid URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("web_fetch only fetches http and https URLs".to_string());
    }
    if is_local_host(url.host_str().unwrap_or_default()) {
        return Err("web_fetch does not fetch local or private addresses".to_string());
    }
    let max_chars = args
        .get("max_chars")
        .and_then(Value::as_u64)
        .map(|value| value.clamp(500, 50_000) as usize)
        .unwrap_or(DEFAULT_FETCH_CHARS);
    let (status, content_type, body) = get_text(url.as_str(), MAX_FETCH_BODY)?;
    if !(200..300).contains(&status) {
        return Err(format!("{url} returned HTTP {status}"));
    }
    let text = if content_type.contains("html") || body.trim_start().starts_with('<') {
        html_to_text(&body)
    } else {
        body
    };
    let mut text: String = text.chars().take(max_chars).collect();
    if text.chars().count() == max_chars {
        text.push_str("\n…[truncated]");
    }
    Ok(format!("Content of {url}:\n\n{text}"))
}

fn is_local_host(host: &str) -> bool {
    let host = host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".local") || host.ends_with(".localhost") {
        return true;
    }
    match host.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(ip)) => {
            ip.is_loopback() || ip.is_private() || ip.is_link_local() || ip.is_unspecified()
        }
        Ok(std::net::IpAddr::V6(ip)) => ip.is_loopback() || ip.is_unspecified(),
        Err(_) => false,
    }
}

pub(super) fn html_to_text(html: &str) -> String {
    static DROP: LazyLock<Option<Regex>> = LazyLock::new(|| {
        Regex::new(
            r"(?is)<(script|style|noscript|svg|head)\b.*?</(script|style|noscript|svg|head)>",
        )
        .ok()
    });
    static BLOCK: LazyLock<Option<Regex>> = LazyLock::new(|| {
        Regex::new(r"(?i)<(br|/p|/div|/li|/h[1-6]|/tr|/section|/article)\b[^>]*>").ok()
    });
    let without = replace_all(&DROP, html, " ");
    let with_breaks = replace_all(&BLOCK, &without, "\n");
    let text = decode_entities(&strip_tags(&with_breaks));
    text.lines()
        .map(collapse_whitespace)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_tags(text: &str) -> String {
    static TAG: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"(?s)<[^>]*>").ok());
    replace_all(&TAG, text, "")
}

/// Replace every match of a fixed pattern. The patterns are literals, so a
/// missing regex only happens if the engine rejects one; the text then passes
/// through unchanged rather than failing the tool.
fn replace_all(pattern: &Option<Regex>, text: &str, replacement: &str) -> String {
    match pattern {
        Some(pattern) => pattern.replace_all(text, replacement).into_owned(),
        None => text.to_string(),
    }
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_entities(text: &str) -> String {
    static ENTITY: LazyLock<Option<Regex>> =
        LazyLock::new(|| Regex::new(r"&(#[0-9]+|#x[0-9a-fA-F]+|[a-zA-Z]+);").ok());
    let Some(entity_pattern) = ENTITY.as_ref() else {
        return text.to_string();
    };
    entity_pattern
        .replace_all(text, |capture: &regex::Captures| {
            let entity = &capture[1];
            let decoded = if let Some(hex) = entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
            {
                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
            } else if let Some(decimal) = entity.strip_prefix('#') {
                decimal.parse::<u32>().ok().and_then(char::from_u32)
            } else {
                match entity {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    "nbsp" => Some(' '),
                    _ => None,
                }
            };
            decoded
                .map(String::from)
                .unwrap_or_else(|| capture[0].to_string())
        })
        .into_owned()
}

fn percent_encode(text: &str) -> String {
    percent_encoding::utf8_percent_encode(text, percent_encoding::NON_ALPHANUMERIC).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bing_rss_items_become_results_with_decoded_text() {
        let body = r#"<rss><channel><title>Bing</title>
            <item><title>Rust &amp; goose</title><link>https://example.com/a</link><description>First &lt;b&gt;result&lt;/b&gt;</description></item>
            <item><title>Second</title><link>https://example.com/b</link><description><![CDATA[Two]]></description></item>
        </channel></rss>"#;
        let results = parse_bing_rss(body);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Rust & goose");
        assert_eq!(results[0].url, "https://example.com/a");
        assert_eq!(results[0].snippet, "First <b>result</b>");
        assert_eq!(results[1].snippet, "Two");
    }

    #[test]
    fn duckduckgo_redirect_links_resolve_to_their_targets() {
        let body = r#"<a rel="nofollow" class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fpage&amp;rut=x">Example <b>Page</b></a>
            <a class="result__snippet" href="x">A <b>snippet</b> here</a>"#;
        let results = parse_duckduckgo_html(body);
        assert_eq!(
            results,
            vec![SearchResult {
                title: "Example Page".into(),
                url: "https://example.com/page".into(),
                snippet: "A snippet here".into(),
            }]
        );
    }

    #[test]
    fn html_text_drops_scripts_and_keeps_block_breaks() {
        let text = html_to_text(
            "<html><head><title>x</title></head><body><script>var a=1;</script><p>Hello&nbsp;there</p><div>Next</div></body></html>",
        );
        assert_eq!(text, "Hello there\nNext");
    }

    #[test]
    fn fetch_refuses_local_addresses() {
        for host in [
            "localhost",
            "127.0.0.1",
            "10.0.0.5",
            "192.168.1.2",
            "[::1]",
            "printer.local",
        ] {
            assert!(is_local_host(host), "{host}");
        }
        assert!(!is_local_host("example.com"));
    }
}
