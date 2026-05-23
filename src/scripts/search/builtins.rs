use std::cmp::Ordering;

use crate::builtins::{BuiltInEntry, BuiltInFeature, BuiltInGroup};

use super::super::types::{BuiltInMatch, MatchEvidence, MatchEvidenceField};
use super::{
    better_match_evidence, low_tier_substring_match, match_evidence, primary_text_match, NucleoCtx,
    TIER_KEYWORD,
};

const TIER_BUILTIN_DESCRIPTION: i32 = 450;

fn restricted_builtin_alias_match(
    entry: &BuiltInEntry,
    query_lower: &str,
) -> Option<MatchEvidence> {
    if !matches!(entry.feature, BuiltInFeature::AiVault) {
        return None;
    }

    let mut best = None::<MatchEvidence>;
    better_match_evidence(
        &mut best,
        match_evidence(
            MatchEvidenceField::Name,
            &entry.name,
            low_tier_substring_match(&entry.name, query_lower, 1000),
        ),
    );
    for keyword in &entry.keywords {
        better_match_evidence(
            &mut best,
            match_evidence(
                MatchEvidenceField::Keyword,
                keyword,
                low_tier_substring_match(keyword, query_lower, TIER_KEYWORD),
            ),
        );
    }

    best
}

/// Fuzzy search built-in entries by query string
/// Searches across name, description, and keywords
/// Returns results sorted by relevance score (highest first)
///
/// Scoring priorities (name matches are weighted MUCH higher than keywords):
/// - Name prefix match (starts with query): 200 points
/// - Name substring match (contains query): 150 points
/// - Name fuzzy match (nucleo): 100 + scaled nucleo score
/// - Description match: 25 points
/// - Keyword exact match: 40 points (much lower than name)
/// - Keyword fuzzy match: 20 + scaled nucleo score
pub fn fuzzy_search_builtins(entries: &[BuiltInEntry], query: &str) -> Vec<BuiltInMatch> {
    if query.is_empty() {
        // If no query, return all entries with equal score, sorted by name
        return entries
            .iter()
            .map(|e| BuiltInMatch {
                entry: e.clone(),
                score: 0,
                match_evidence: None,
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    let mut nucleo = NucleoCtx::new(&query_lower);

    for entry in entries {
        if matches!(entry.feature, BuiltInFeature::AiVault) {
            if let Some(evidence) = restricted_builtin_alias_match(entry, &query_lower) {
                matches.push(BuiltInMatch {
                    entry: entry.clone(),
                    score: evidence.score,
                    match_evidence: Some(evidence),
                });
            }
            continue;
        }

        let mut best = None;

        if entry.group == BuiltInGroup::MenuBar {
            better_match_evidence(
                &mut best,
                match_evidence(
                    MatchEvidenceField::Name,
                    &entry.name,
                    low_tier_substring_match(&entry.name, &query_lower, 900),
                ),
            );
        } else {
            better_match_evidence(
                &mut best,
                match_evidence(
                    MatchEvidenceField::Name,
                    &entry.name,
                    primary_text_match(&entry.name, &query_lower, &mut nucleo),
                ),
            );
        }

        better_match_evidence(
            &mut best,
            match_evidence(
                MatchEvidenceField::Description,
                &entry.description,
                low_tier_substring_match(
                    &entry.description,
                    &query_lower,
                    TIER_BUILTIN_DESCRIPTION,
                ),
            ),
        );

        for keyword in &entry.keywords {
            better_match_evidence(
                &mut best,
                match_evidence(
                    MatchEvidenceField::Keyword,
                    keyword,
                    low_tier_substring_match(keyword, &query_lower, TIER_KEYWORD),
                ),
            );
        }

        if let Some(best) = best {
            matches.push(BuiltInMatch {
                entry: entry.clone(),
                score: best.score,
                match_evidence: Some(best),
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.entry.name.cmp(&b.entry.name),
        other => other,
    });

    matches
}
