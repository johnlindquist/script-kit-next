//! `kit://brain` MCP resources: the brain's transparency and agent-access
//! surface.
//!
//! - `kit://brain` — status: doc/embedding/signal counts, model presence.
//! - `kit://brain/recall?q=...` — hybrid retrieval rendered as a context
//!   block (what Agent Chat stages per turn, available to any MCP consumer).
//! - `kit://brain/signals?limit=...` — recent attention signals ("why does
//!   the brain think I care about X").

use super::store;

pub const BRAIN_RESOURCE_URI: &str = "kit://brain";

pub fn is_brain_resource_uri(uri: &str) -> bool {
    uri == BRAIN_RESOURCE_URI || uri.starts_with("kit://brain?") || uri.starts_with("kit://brain/")
}

fn query_param(uri: &str, key: &str) -> Option<String> {
    let query = uri.split_once('?')?.1;
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if k == key {
            let decoded = v.replace('+', " ");
            let decoded = percent_decode(&decoded);
            return Some(decoded);
        }
    }
    None
}

/// Minimal percent-decoding (enough for q= text); invalid sequences pass through.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&value[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Resolve a brain resource URI to (mime_type, body).
pub fn read_brain_resource(uri: &str) -> Result<(String, String), String> {
    store::init_brain_db().map_err(|error| format!("brain init failed: {error}"))?;
    if uri == BRAIN_RESOURCE_URI || uri.starts_with("kit://brain?") {
        let (docs, embedded, signals) =
            store::doc_stats().map_err(|error| format!("brain stats failed: {error}"))?;
        let model = super::embedder::resolve_embed_model();
        let body = serde_json::json!({
            "docs": docs,
            "embedded": embedded,
            "signals": signals,
            "semanticSearch": model.is_some(),
            "embedModel": model.map(|m| m.model_id),
            "store": "~/.scriptkit/db/brain.sqlite",
        });
        return Ok(("application/json".to_string(), body.to_string()));
    }
    if uri.starts_with("kit://brain/recall") {
        let query = query_param(uri, "q").unwrap_or_default();
        if query.trim().is_empty() {
            return Err("kit://brain/recall requires ?q=<query>".to_string());
        }
        super::record_ask_signals(&query);
        let block = super::recall_context_block(&query)
            .map_err(|error| format!("brain recall failed: {error}"))?
            .unwrap_or_else(|| "(no relevant memories)".to_string());
        return Ok(("text/markdown".to_string(), block));
    }
    if uri.starts_with("kit://brain/focus") {
        if query_param(uri, "refresh").as_deref() == Some("1") {
            super::curator::run_focus_review()
                .map_err(|error| format!("focus review failed: {error}"))?;
        }
        let review = store::get_doc(super::store::DocSource::Activity, "focus-review")
            .map_err(|error| format!("focus read failed: {error}"))?;
        let body = review.map(|doc| doc.content).unwrap_or_else(|| {
            "(no focus review yet — add ?refresh=1 to generate one)".to_string()
        });
        return Ok(("text/markdown".to_string(), body));
    }
    if uri.starts_with("kit://brain/signals") {
        let limit = query_param(uri, "limit")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(50)
            .min(500);
        let signals = store::recent_signals(limit)
            .map_err(|error| format!("brain signals failed: {error}"))?;
        let body = serde_json::json!(signals
            .iter()
            .map(|s| {
                serde_json::json!({
                    "topic": s.topic,
                    "weight": s.weight,
                    "source": s.source,
                    "createdAt": s.created_at,
                })
            })
            .collect::<Vec<_>>());
        return Ok(("application/json".to_string(), body.to_string()));
    }
    Err(format!("Brain resource not found: {uri}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_matching() {
        assert!(is_brain_resource_uri("kit://brain"));
        assert!(is_brain_resource_uri("kit://brain/recall?q=hello"));
        assert!(is_brain_resource_uri("kit://brain/signals?limit=10"));
        assert!(!is_brain_resource_uri("kit://notes"));
    }

    #[test]
    fn query_param_decoding() {
        assert_eq!(
            query_param("kit://brain/recall?q=hello+world", "q").as_deref(),
            Some("hello world")
        );
        assert_eq!(
            query_param("kit://brain/recall?q=a%20b%26c", "q").as_deref(),
            Some("a b&c")
        );
        assert_eq!(query_param("kit://brain/recall", "q"), None);
    }
}
