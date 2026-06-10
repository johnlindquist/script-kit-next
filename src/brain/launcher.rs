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

    // The sync per-keystroke pass stays lexical-only (query_vec: None); the
    // async pass ([`search_root_brain_semantic`]) upgrades it to hybrid.
    map_root_brain_hits(
        super::brain_search(query.trim(), None, None, options.max_results).unwrap_or_default(),
    )
}

/// Async hybrid pass for the root launcher: embed the query on the warm
/// indexer thread (hard ~200ms budget), then run hybrid FTS+cosine search.
/// Returns `None` when no embedding model is warm — callers keep the lexical
/// hits. Blocking; call from a background thread only, never the UI thread.
pub fn search_root_brain_semantic(
    query: &str,
    options: &RootBrainSectionOptions,
) -> Option<Vec<RootBrainSearchHit>> {
    if !root_brain_query_is_eligible(query, *options) || options.max_results == 0 {
        return None;
    }
    let query = query.trim();
    let (model_id, query_vec) = super::indexer::embed_query_within_budget(query)?;
    Some(map_root_brain_hits(
        super::brain_search(
            query,
            Some(&query_vec),
            Some(&model_id),
            options.max_results,
        )
        .unwrap_or_default(),
    ))
}

/// Prefer async semantic hits over the sync lexical pass when they were
/// computed for exactly the query the launcher is currently showing.
/// `semantic` is `(stored_query, hits)` from app state; `None` (or a stale
/// stored query) means "use lexical". Pure so it's testable without GPUI.
pub fn semantic_root_brain_hits_for_query(
    current_query: &str,
    semantic: Option<&(String, Vec<RootBrainSearchHit>)>,
    options: &RootBrainSectionOptions,
) -> Option<Vec<RootBrainSearchHit>> {
    if !root_brain_query_is_eligible(current_query, *options) || options.max_results == 0 {
        return None;
    }
    let (stored_query, hits) = semantic?;
    if stored_query != current_query.trim() {
        return None;
    }
    let mut hits = hits.clone();
    hits.truncate(options.max_results);
    Some(hits)
}

/// Map full brain hits down to launcher rows. Shared by the sync lexical and
/// async semantic passes so both produce identical row shapes.
pub(crate) fn map_root_brain_hits(hits: Vec<super::search::BrainHit>) -> Vec<RootBrainSearchHit> {
    hits.into_iter()
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

    fn semantic_hit(title: &str) -> RootBrainSearchHit {
        RootBrainSearchHit {
            title: title.to_string(),
            excerpt: String::new(),
            source_label: "Note",
            source: DocSource::Note,
            source_id: title.to_string(),
        }
    }

    #[test]
    fn root_brain_prefers_semantic_hits_when_stored_query_matches() {
        let options = RootBrainSectionOptions::default();
        let stored = (
            "fix bug".to_string(),
            vec![semantic_hit("a"), semantic_hit("b")],
        );
        let hits = semantic_root_brain_hits_for_query("fix bug", Some(&stored), &options)
            .expect("matching query should prefer semantic hits");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].title, "a");
        // Whitespace around the live query must not break the match — the
        // stored query is the trimmed search text.
        assert!(
            semantic_root_brain_hits_for_query("  fix bug  ", Some(&stored), &options).is_some()
        );
    }

    #[test]
    fn root_brain_falls_back_to_lexical_when_semantic_is_stale_or_missing() {
        let options = RootBrainSectionOptions::default();
        let stored = ("fix bug".to_string(), vec![semantic_hit("a")]);
        // Stale stored query → lexical.
        assert!(semantic_root_brain_hits_for_query("fix bugs", Some(&stored), &options).is_none());
        // No semantic results yet → lexical.
        assert!(semantic_root_brain_hits_for_query("fix bug", None, &options).is_none());
    }

    #[test]
    fn root_brain_semantic_merge_respects_eligibility_and_caps() {
        let stored = (
            "fix".to_string(),
            vec![semantic_hit("a"), semantic_hit("b"), semantic_hit("c")],
        );
        // Disabled section → never serve semantic hits.
        let disabled = RootBrainSectionOptions {
            enabled: false,
            ..Default::default()
        };
        assert!(semantic_root_brain_hits_for_query("fix", Some(&stored), &disabled).is_none());
        // Query below min chars → ineligible.
        let options = RootBrainSectionOptions::default();
        let short_stored = ("fi".to_string(), vec![semantic_hit("a")]);
        assert!(semantic_root_brain_hits_for_query("fi", Some(&short_stored), &options).is_none());
        // max_results caps stored hits (e.g. options shrank since the spawn).
        let capped = RootBrainSectionOptions {
            max_results: 2,
            ..Default::default()
        };
        let hits = semantic_root_brain_hits_for_query("fix", Some(&stored), &capped)
            .expect("eligible matching query");
        assert_eq!(hits.len(), 2);
        // max_results == 0 → section renders nothing; keep lexical path.
        let zero = RootBrainSectionOptions {
            max_results: 0,
            ..Default::default()
        };
        assert!(semantic_root_brain_hits_for_query("fix", Some(&stored), &zero).is_none());
    }

    #[test]
    fn root_brain_semantic_empty_hits_still_count_as_results() {
        // Hybrid search subsumes lexical: an empty semantic batch for the
        // current query means "the brain has nothing", not "fall back".
        let options = RootBrainSectionOptions::default();
        let stored = ("fix bug".to_string(), Vec::new());
        let hits = semantic_root_brain_hits_for_query("fix bug", Some(&stored), &options)
            .expect("empty semantic batch is authoritative");
        assert!(hits.is_empty());
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
