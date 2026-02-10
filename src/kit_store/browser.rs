//! GitHub-backed kit discovery helpers.

use std::collections::HashSet;
use std::time::Duration;

use base64::Engine as _;
use serde::Deserialize;

const GITHUB_API_BASE_URL: &str = "https://api.github.com";
const GITHUB_SEARCH_REPOSITORIES_PATH: &str = "/search/repositories";
const GITHUB_API_ACCEPT_HEADER: &str = "application/vnd.github+json";
const GITHUB_API_VERSION_HEADER: &str = "2022-11-28";
const GITHUB_USER_AGENT: &str = "script-kit-gpui-kit-store";
const GITHUB_KIT_TOPICS: [&str; 2] = ["scriptkit-kit", "script-kit"];
const GITHUB_CONNECT_TIMEOUT_SECS: u64 = 5;
const GITHUB_RESPONSE_TIMEOUT_SECS: u64 = 20;
const GITHUB_READ_TIMEOUT_SECS: u64 = 30;
const GITHUB_GLOBAL_TIMEOUT_SECS: u64 = 30;

/// A GitHub repository result surfaced in the kit browser.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct KitSearchResult {
    pub name: String,
    pub full_name: String,
    pub description: String,
    pub stars: u64,
    pub updated_at: String,
    pub html_url: String,
    pub clone_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubSearchRepositoriesResponse {
    #[serde(default)]
    items: Vec<GithubRepositoryItem>,
}

#[derive(Debug, Deserialize)]
struct GithubRepositoryItem {
    #[serde(default)]
    name: String,
    #[serde(default)]
    full_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    stargazers_count: u64,
    #[serde(default)]
    updated_at: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    clone_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubReadmeResponse {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    encoding: Option<String>,
}

/// Search public GitHub repositories tagged as Script Kit kits.
///
/// This function is intentionally best-effort:
/// - returns an empty list on transport/parsing errors;
/// - returns an empty list when unauthenticated rate limits are hit.
pub fn search_github_kits(query: &str) -> Vec<KitSearchResult> {
    let agent = create_github_agent();
    let mut seen_full_names = HashSet::new();
    let mut combined_results = Vec::new();

    for topic in GITHUB_KIT_TOPICS {
        let url = build_search_url(topic, query);
        let Some(parsed) = fetch_github_json::<GithubSearchRepositoriesResponse>(
            &agent,
            &url,
            "search_github_kits",
        ) else {
            continue;
        };

        for item in parsed.items {
            if item.full_name.is_empty() || !seen_full_names.insert(item.full_name.clone()) {
                continue;
            }

            combined_results.push(KitSearchResult {
                name: item.name,
                full_name: item.full_name,
                description: item.description.unwrap_or_default(),
                stars: item.stargazers_count,
                updated_at: item.updated_at,
                html_url: item.html_url,
                clone_url: item.clone_url,
            });
        }
    }

    combined_results.sort_by(|a, b| {
        b.stars
            .cmp(&a.stars)
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    combined_results
}

/// Fetch repository README text for preview.
///
/// Returns `None` when:
/// - repository/readme is missing;
/// - the request fails;
/// - unauthenticated rate limits are hit.
pub fn fetch_kit_readme(full_name: &str) -> Option<String> {
    let full_name = full_name.trim();
    if full_name.is_empty() {
        return None;
    }

    let agent = create_github_agent();
    let url = format!("{GITHUB_API_BASE_URL}/repos/{full_name}/readme");

    let readme = fetch_github_json::<GithubReadmeResponse>(&agent, &url, "fetch_kit_readme")?;
    decode_readme_content(readme.content.as_deref(), readme.encoding.as_deref())
}

fn create_github_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .https_only(true)
        .timeout_connect(Some(Duration::from_secs(GITHUB_CONNECT_TIMEOUT_SECS)))
        .timeout_recv_response(Some(Duration::from_secs(GITHUB_RESPONSE_TIMEOUT_SECS)))
        .timeout_recv_body(Some(Duration::from_secs(GITHUB_READ_TIMEOUT_SECS)))
        .timeout_global(Some(Duration::from_secs(GITHUB_GLOBAL_TIMEOUT_SECS)))
        .build()
        .new_agent()
}

fn build_search_url(topic: &str, query: &str) -> String {
    let trimmed_query = query.trim();
    if trimmed_query.is_empty() {
        return format!(
            "{GITHUB_API_BASE_URL}{GITHUB_SEARCH_REPOSITORIES_PATH}?q=topic:{topic}&sort=stars&order=desc"
        );
    }

    let normalized_query = trimmed_query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("+");
    format!(
        "{GITHUB_API_BASE_URL}{GITHUB_SEARCH_REPOSITORIES_PATH}?q=topic:{topic}+{normalized_query}&sort=stars&order=desc"
    )
}

fn fetch_github_json<T>(agent: &ureq::Agent, url: &str, operation: &str) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let response = match agent
        .get(url)
        .header("Accept", GITHUB_API_ACCEPT_HEADER)
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION_HEADER)
        .header("User-Agent", GITHUB_USER_AGENT)
        .call()
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(
                operation,
                url,
                error = %error,
                "GitHub request failed before receiving a response"
            );
            return None;
        }
    };

    let status = response.status().as_u16();
    if is_github_rate_limited(status, &response) {
        let reset_epoch = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown");
        tracing::warn!(
            operation,
            url,
            status,
            reset_epoch,
            "GitHub unauthenticated rate limit hit"
        );
        return None;
    }

    if !(200..300).contains(&status) {
        tracing::warn!(
            operation,
            url,
            status,
            "GitHub request returned non-success status"
        );
        return None;
    }

    let mut body = response.into_body();
    match body.read_json::<T>() {
        Ok(parsed) => Some(parsed),
        Err(error) => {
            tracing::warn!(
                operation,
                url,
                error = %error,
                "Failed to parse GitHub response body as JSON"
            );
            None
        }
    }
}

fn is_github_rate_limited(status: u16, response: &ureq::http::Response<ureq::Body>) -> bool {
    status == 429
        || (status == 403
            && response
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|value| value.to_str().ok())
                == Some("0"))
}

fn decode_readme_content(content: Option<&str>, encoding: Option<&str>) -> Option<String> {
    let content = content?;
    match encoding.unwrap_or_default() {
        "base64" => {
            let compact_content = content.replace('\n', "");
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(compact_content.as_bytes())
                .ok()?;
            String::from_utf8(bytes).ok()
        }
        "" => Some(content.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_search_url_adds_topic_only_when_query_is_blank() {
        let url = build_search_url("scriptkit-kit", "   ");
        assert_eq!(
            url,
            "https://api.github.com/search/repositories?q=topic:scriptkit-kit&sort=stars&order=desc"
        );
    }

    #[test]
    fn test_build_search_url_normalizes_whitespace_when_query_has_multiple_spaces() {
        let url = build_search_url("script-kit", "  clipboard   manager  ");
        assert_eq!(
            url,
            "https://api.github.com/search/repositories?q=topic:script-kit+clipboard+manager&sort=stars&order=desc"
        );
    }

    #[test]
    fn test_decode_readme_content_decodes_base64_when_payload_is_valid() {
        let text = decode_readme_content(Some("aGVsbG8gd29ybGQ=\n"), Some("base64"));
        assert_eq!(text.as_deref(), Some("hello world"));
    }

    #[test]
    fn test_decode_readme_content_returns_none_when_payload_is_not_utf8() {
        let text = decode_readme_content(Some("//8=\n"), Some("base64"));
        assert!(text.is_none());
    }

    #[test]
    fn test_decode_readme_content_returns_plain_text_when_encoding_is_missing() {
        let text = decode_readme_content(Some("README plain text"), None);
        assert_eq!(text.as_deref(), Some("README plain text"));
    }
}
