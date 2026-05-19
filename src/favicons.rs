use gpui::RenderImage;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

/// In-memory cache of decoded favicon images keyed by domain.
/// `None` means a fetch was attempted but failed.
static FAVICON_CACHE: LazyLock<Mutex<HashMap<String, Option<Arc<RenderImage>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Look up a cached favicon for the given URL's domain.
/// Returns `None` if not yet fetched or fetch failed.
pub fn cached_favicon(url: &str) -> Option<Arc<RenderImage>> {
    let domain = domain_from_url(url)?;
    let cache = FAVICON_CACHE.lock().ok()?;
    cache.get(domain)?.clone()
}

/// Extract the host from a URL (e.g. "https://docs.google.com/foo" → "docs.google.com").
pub fn domain_from_url(url: &str) -> Option<&str> {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host = after_scheme.split('/').next().unwrap_or(after_scheme);
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

/// Return the list of unique domains from `urls` that are not yet in the cache.
pub fn domains_needing_favicons<S: AsRef<str>>(urls: &[S]) -> Vec<String> {
    let cache = FAVICON_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let mut seen = std::collections::HashSet::new();
    let mut domains = Vec::new();

    for url in urls {
        if let Some(domain) = domain_from_url(url.as_ref()) {
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
    if domains.is_empty() {
        return;
    }

    let agent = ureq::Agent::new_with_defaults();

    for domain in domains {
        let url = format!("https://www.google.com/s2/favicons?domain={}&sz=64", domain);

        let result = (|| -> Option<Arc<RenderImage>> {
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
