use std::cmp::Ordering;

use crate::app_launcher::AppInfo;

use super::super::types::{AppMatch, MatchEvidence, MatchEvidenceField};
use super::{match_evidence, primary_text_match, NucleoCtx};

/// Fuzzy search applications by query string
/// Searches across the visible app name only. Bundle identifiers and paths are
/// intentionally not admission fields for normal launcher search.
/// Returns results sorted by relevance score (highest first)
pub fn fuzzy_search_apps(apps: &[AppInfo], query: &str) -> Vec<AppMatch> {
    if query.is_empty() {
        // If no query, return all apps with equal score
        return apps
            .iter()
            .map(|a| AppMatch {
                app: a.clone(),
                score: 0,
                match_evidence: None,
            })
            .collect();
    }

    rank_apps(apps, query)
        .into_iter()
        .map(|(index, evidence)| AppMatch {
            app: apps[index].clone(),
            score: evidence.score,
            match_evidence: Some(evidence),
        })
        .collect()
}

/// Two deliberate word prefixes may each omit one interior vowel: FxEd → Fixture Editor.
/// Don't admit arbitrary scattered letters, nonadjacent words, paths or bundle identifiers.
fn shortened_app_words(name: &str, query: &str) -> Option<MatchEvidence> {
    let q = query.as_bytes();
    if !(4..=16).contains(&q.len()) || !q.iter().all(u8::is_ascii_alphabetic) {
        return None;
    }
    let mut words: Vec<Vec<(u8, usize)>> = Vec::new();
    let mut previous = None::<char>;
    for (i, ch) in name.chars().enumerate() {
        if i >= 128 {
            return None;
        }
        if ch.is_ascii_alphabetic() {
            if previous.is_none_or(|p| {
                !p.is_ascii_alphabetic() || p.is_ascii_lowercase() && ch.is_ascii_uppercase()
            }) {
                words.push(Vec::new());
            }
            words.last_mut()?.push((ch.to_ascii_lowercase() as u8, i));
        }
        previous = Some(ch);
    }
    fn prefix(word: &[(u8, usize)], part: &[u8]) -> Option<(Vec<usize>, bool)> {
        let mut out = Vec::with_capacity(part.len());
        let mut skipped = false;
        let mut at = 0;
        for (i, &letter) in part.iter().enumerate() {
            if word.get(at)?.0 != letter && i > 0 && !skipped && b"aeiou".contains(&word[at].0) {
                at += 1;
                skipped = true;
            }
            if word.get(at)?.0 != letter {
                return None;
            }
            out.push(word[at].1);
            at += 1;
        }
        Some((out, skipped))
    }
    for pair in words.windows(2) {
        for cut in 2..=q.len() - 2 {
            if let (Some((mut left, a)), Some((right, b))) =
                (prefix(&pair[0], &q[..cut]), prefix(&pair[1], &q[cut..]))
            {
                if a || b {
                    left.extend(right);
                    let tier = super::match_contract::TIER_COMPACT_FUZZY_PRIMARY;
                    return Some(MatchEvidence {
                        field: MatchEvidenceField::Name,
                        text: name.into(),
                        indices: left,
                        tier,
                        score: super::match_contract::score_from_tier(tier, 100),
                    });
                }
            }
        }
    }
    None
}

/// The apps whose names match `query`, best first, as positions in `apps` with the
/// evidence that admitted them. The launcher and the Apps list share this one rule, so
/// an abbreviation such as `gcal` finds Google Calendar in both.
pub fn rank_apps(apps: &[AppInfo], query: &str) -> Vec<(usize, MatchEvidence)> {
    let query_lower = query.to_lowercase();
    let mut matches: Vec<(usize, MatchEvidence)> = Vec::new();

    // Create nucleo context once for all apps - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    for (index, app) in apps.iter().enumerate() {
        if let Some(evidence) = match_evidence(
            MatchEvidenceField::Name,
            &app.name,
            primary_text_match(&app.name, &query_lower, &mut nucleo),
        )
        .or_else(|| shortened_app_words(&app.name, &query_lower))
        {
            matches.push((index, evidence));
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|(a_idx, a_evidence), (b_idx, b_evidence)| {
        match b_evidence.score.cmp(&a_evidence.score) {
            Ordering::Equal => apps[*a_idx].name.cmp(&apps[*b_idx].name),
            other => other,
        }
    });
    matches
}

#[cfg(test)]
mod shortened_apps_tests {
    use super::*;
    #[test]
    fn abbreviated_word_pairs_still_require_two_real_word_prefixes() {
        assert_eq!(
            shortened_app_words("Fixture Editor", "fxed")
                .unwrap()
                .indices,
            [0, 2, 8, 9]
        );
        assert!(shortened_app_words("Fixture Text Editor", "fxed").is_none());
        assert!(shortened_app_words("Fixture Editor", "fted").is_none());
        assert!(shortened_app_words("Fixture Editor", "xed").is_none());
    }
}
