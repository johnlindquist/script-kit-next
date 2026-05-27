use std::cmp::Ordering;
use std::sync::Arc;

use super::super::types::{
    MatchEvidence, MatchEvidenceField, MatchIndices, Scriptlet, ScriptletMatch,
};
use super::{
    better_match_evidence, extract_scriptlet_display_path, low_tier_substring_match,
    match_evidence, primary_text_match, NucleoCtx, TIER_ALIAS, TIER_DESCRIPTION, TIER_FILENAME,
    TIER_KEYWORD,
};

/// Fuzzy search scriptlets by query string
/// Searches visible names with primary matching and secondary metadata fields
/// with exact contiguous substring tiers. Scriptlet source/code is not searched.
/// Returns results sorted by relevance score (highest first)
/// Match indices are provided to enable UI highlighting of matched characters
///
/// H1 Optimization: Accepts Arc<Scriptlet> to avoid expensive clones during filter operations.
/// Each ScriptletMatch contains an Arc::clone which is just a refcount bump.
pub fn fuzzy_search_scriptlets(scriptlets: &[Arc<Scriptlet>], query: &str) -> Vec<ScriptletMatch> {
    if query.is_empty() {
        // If no query, return all scriptlets with equal score, sorted by name
        return scriptlets
            .iter()
            .map(|s| {
                let display_file_path = extract_scriptlet_display_path(&s.file_path);
                ScriptletMatch {
                    scriptlet: Arc::clone(s),
                    score: 0,
                    display_file_path,
                    match_indices: MatchIndices::default(),
                    match_evidence: None,
                }
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all scriptlets - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    for scriptlet in scriptlets {
        let mut best = None::<MatchEvidence>;
        // Lazy match indices - don't compute during scoring
        let match_indices = MatchIndices::default();

        let display_file_path = extract_scriptlet_display_path(&scriptlet.file_path);

        let display_name = crate::frontmost_app_tracker::substitute_context_vars(&scriptlet.name);
        better_match_evidence(
            &mut best,
            match_evidence(
                MatchEvidenceField::Name,
                &display_name,
                primary_text_match(&display_name, &query_lower, &mut nucleo),
            ),
        );

        if let Some(ref fp) = display_file_path {
            better_match_evidence(
                &mut best,
                match_evidence(
                    MatchEvidenceField::Filename,
                    fp,
                    low_tier_substring_match(fp, &query_lower, TIER_FILENAME),
                ),
            );
        }

        if let Some(ref desc) = scriptlet.description {
            better_match_evidence(
                &mut best,
                match_evidence(
                    MatchEvidenceField::Description,
                    desc,
                    low_tier_substring_match(desc, &query_lower, TIER_DESCRIPTION),
                ),
            );
        }

        if let Some(ref keyword) = scriptlet.keyword {
            better_match_evidence(
                &mut best,
                match_evidence(
                    MatchEvidenceField::Keyword,
                    keyword,
                    low_tier_substring_match(keyword, &query_lower, TIER_ALIAS),
                ),
            );
        }

        if let Some(ref alias) = scriptlet.alias {
            better_match_evidence(
                &mut best,
                match_evidence(
                    MatchEvidenceField::Alias,
                    alias,
                    low_tier_substring_match(alias, &query_lower, TIER_ALIAS),
                ),
            );
        }

        if let Some(ref shortcut) = scriptlet.shortcut {
            better_match_evidence(
                &mut best,
                match_evidence(
                    MatchEvidenceField::Shortcut,
                    shortcut,
                    low_tier_substring_match(shortcut, &query_lower, TIER_ALIAS),
                ),
            );
        }

        if let Some(ref group) = scriptlet.group {
            if group != "main" {
                better_match_evidence(
                    &mut best,
                    match_evidence(
                        MatchEvidenceField::Source,
                        group,
                        low_tier_substring_match(group, &query_lower, TIER_KEYWORD),
                    ),
                );
            }
        }

        better_match_evidence(
            &mut best,
            match_evidence(
                MatchEvidenceField::Tool,
                &scriptlet.tool,
                low_tier_substring_match(&scriptlet.tool, &query_lower, TIER_KEYWORD),
            ),
        );

        if let Some(best) = best {
            matches.push(ScriptletMatch {
                scriptlet: Arc::clone(scriptlet),
                score: best.score,
                display_file_path,
                match_indices,
                match_evidence: Some(best),
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.scriptlet.name.cmp(&b.scriptlet.name),
        other => other,
    });

    matches
}
