//! Root launcher passive "From Your Brain" section plumbing.
//!
//! Mirrors the Notes passive-source pattern (`src/notes/storage.rs`): a small
//! options struct sourced from config, an eligibility gate, and a direct
//! search entry point that maps brain documents into lightweight UI hits.
//!
//! IMPORTANT: this path must never record attention signals — passive search
//! feeding the attention log would self-amplify whatever the user happens to
//! be typing.

use super::store::DocSource;

/// Maximum characters carried into a row excerpt.
const ROOT_BRAIN_EXCERPT_MAX_CHARS: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootBrainSectionOptions {
    pub enabled: bool,
    pub max_results: usize,
    pub min_query_chars: usize,
}

impl Default for RootBrainSectionOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            max_results: 4,
            min_query_chars: 3,
        }
    }
}

/// A brain document projected down to exactly what the launcher row needs.
/// Full document content intentionally stays behind in the store.
#[derive(Debug, Clone)]
pub struct RootBrainSearchHit {
    pub title: String,
    pub excerpt: String,
    pub source_label: &'static str,
    pub source: DocSource,
    pub source_id: String,
}

pub fn root_brain_query_is_eligible(query: &str, options: RootBrainSectionOptions) -> bool {
    let query = query.trim();
    options.enabled && !query.contains('\n') && query.chars().count() >= options.min_query_chars
}

/// Search the brain for passive root-launcher rows. Returns an empty list when
/// the section is disabled, the query is too short, or the store errors —
/// passive sources must never surface failures into the launcher.
pub fn search_root_brain_direct(
    query: &str,
    options: &RootBrainSectionOptions,
) -> Vec<RootBrainSearchHit> {
    if !root_brain_query_is_eligible(query, *options) || options.max_results == 0 {
        return Vec::new();
    }

    // v1 is lexical-only (query_vec: None); a later commit adds async semantic.
    super::brain_search(query.trim(), None, None, options.max_results)
        .unwrap_or_default()
        .into_iter()
        .map(|hit| {
            let excerpt = excerpt_for_content(&hit.doc.content);
            let title = if hit.doc.title.trim().is_empty() {
                hit.doc.source.label().to_string()
            } else {
                hit.doc.title.clone()
            };
            RootBrainSearchHit {
                title,
                excerpt,
                source_label: hit.doc.source.label(),
                source: hit.doc.source,
                source_id: hit.doc.source_id.clone(),
            }
        })
        .collect()
}

/// First non-empty content line, truncated to a row-friendly excerpt.
fn excerpt_for_content(content: &str) -> String {
    let line = content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");
    if line.chars().count() <= ROOT_BRAIN_EXCERPT_MAX_CHARS {
        return line.to_string();
    }
    let mut excerpt: String = line.chars().take(ROOT_BRAIN_EXCERPT_MAX_CHARS).collect();
    excerpt.push('…');
    excerpt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eligibility_mirrors_notes_gating() {
        let options = RootBrainSectionOptions {
            enabled: true,
            max_results: 4,
            min_query_chars: 3,
        };
        assert!(root_brain_query_is_eligible("fix", options));
        assert!(!root_brain_query_is_eligible("fi", options));
        assert!(!root_brain_query_is_eligible("fix\nbug", options));
        assert!(!root_brain_query_is_eligible(
            "fix",
            RootBrainSectionOptions {
                enabled: false,
                ..options
            }
        ));
    }

    #[test]
    fn excerpt_uses_first_non_empty_line_and_truncates() {
        assert_eq!(
            excerpt_for_content("\n\n  hello world  \nsecond"),
            "hello world"
        );
        let long = "x".repeat(300);
        let excerpt = excerpt_for_content(&long);
        assert_eq!(excerpt.chars().count(), ROOT_BRAIN_EXCERPT_MAX_CHARS + 1);
        assert!(excerpt.ends_with('…'));
    }

    #[test]
    fn disabled_or_short_queries_return_empty() {
        let options = RootBrainSectionOptions {
            enabled: false,
            ..Default::default()
        };
        assert!(search_root_brain_direct("anything", &options).is_empty());
        let options = RootBrainSectionOptions::default();
        assert!(search_root_brain_direct("ab", &options).is_empty());
    }
}
