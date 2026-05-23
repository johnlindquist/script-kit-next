use std::cmp::Ordering;

use crate::builtins::{BuiltInEntry, BuiltInFeature, BuiltInGroup};

use super::super::types::BuiltInMatch;
use super::{
    better_match, low_tier_substring_match, primary_text_match, score_from_tier, NucleoCtx,
    TIER_KEYWORD,
};

const TIER_BUILTIN_DESCRIPTION: i32 = 450;

fn restricted_builtin_alias_score(entry: &BuiltInEntry, query_lower: &str) -> Option<i32> {
    if !matches!(entry.feature, BuiltInFeature::AiVault) {
        return None;
    }

    let mut best = None::<i32>;
    let mut score_candidate = |candidate: &str| {
        if !candidate.is_ascii() {
            return;
        }

        let candidate_lower = candidate.to_lowercase();
        let score = if candidate_lower == query_lower {
            Some(score_from_tier(1000, 900))
        } else if query_lower.len() >= 3 && candidate_lower.starts_with(query_lower) {
            Some(score_from_tier(950, 900))
        } else if query_lower.len() >= 3 && candidate_lower.contains(query_lower) {
            Some(score_from_tier(850, 900))
        } else {
            None
        };

        if let Some(score) = score {
            best = Some(best.map_or(score, |current| current.max(score)));
        }
    };

    score_candidate(&entry.name);
    for keyword in &entry.keywords {
        score_candidate(keyword);
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
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    let mut nucleo = NucleoCtx::new(&query_lower);

    for entry in entries {
        if matches!(entry.feature, BuiltInFeature::AiVault) {
            if let Some(score) = restricted_builtin_alias_score(entry, &query_lower) {
                matches.push(BuiltInMatch {
                    entry: entry.clone(),
                    score,
                });
            }
            continue;
        }

        let mut best = None;

        if entry.group == BuiltInGroup::MenuBar {
            let leaf_name = entry.leaf_name();
            better_match(
                &mut best,
                low_tier_substring_match(leaf_name, &query_lower, 900),
            );
        } else {
            better_match(
                &mut best,
                primary_text_match(&entry.name, &query_lower, &mut nucleo),
            );
        }

        better_match(
            &mut best,
            low_tier_substring_match(&entry.description, &query_lower, TIER_BUILTIN_DESCRIPTION),
        );

        for keyword in &entry.keywords {
            better_match(
                &mut best,
                low_tier_substring_match(keyword, &query_lower, TIER_KEYWORD),
            );
        }

        if let Some(best) = best {
            matches.push(BuiltInMatch {
                entry: entry.clone(),
                score: best.score,
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
