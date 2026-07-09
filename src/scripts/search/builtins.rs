use std::cmp::Ordering;

use crate::builtins::{BuiltInEntry, BuiltInFeature, BuiltInGroup};

use super::super::types::{BuiltInMatch, MatchEvidence, MatchEvidenceField};
use super::{
    better_match_evidence, low_tier_substring_match, match_evidence, primary_text_match,
    query_is_ascii_punctuation_only, NucleoCtx, TIER_KEYWORD,
};

const TIER_BUILTIN_DESCRIPTION: i32 = 450;

fn builtin_secondary_fields_are_searchable(query: &str) -> bool {
    query.chars().any(char::is_alphanumeric)
}

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
        // If no query, return all entries with equal score, sorted by name.
        // Query-only entries (experimental Flow UX surfaces) are excluded
        // here and ONLY here — any real typed query searches them normally.
        return entries
            .iter()
            .filter(|e| !crate::builtins::is_query_only_builtin(&e.id))
            .map(|e| BuiltInMatch {
                entry: e.clone(),
                score: 0,
                match_evidence: None,
            })
            .collect();
    }

    if query_is_ascii_punctuation_only(query) {
        return Vec::new();
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
            if !builtin_secondary_fields_are_searchable(&query_lower) {
                continue;
            }
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

        if builtin_secondary_fields_are_searchable(&query_lower) {
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

#[cfg(test)]
mod tests {
    use crate::builtins::{BuiltInEntry, BuiltInFeature, BuiltInGroup};

    use super::{fuzzy_search_builtins, MatchEvidenceField};

    fn builtin(name: &str, description: &str, keywords: &[&str]) -> BuiltInEntry {
        builtin_with_group(name, description, keywords, BuiltInGroup::Core)
    }

    fn builtin_with_group(
        name: &str,
        description: &str,
        keywords: &[&str],
        group: BuiltInGroup,
    ) -> BuiltInEntry {
        BuiltInEntry {
            id: name.to_lowercase().replace(' ', "-"),
            name: name.to_string(),
            description: description.to_string(),
            keywords: keywords.iter().map(|keyword| keyword.to_string()).collect(),
            feature: BuiltInFeature::Settings,
            icon: None,
            group,
        }
    }

    #[test]
    fn punctuation_only_query_does_not_match_builtin_description() {
        let entries = vec![builtin("Settings Hub", "Open app settings.", &[])];

        let matches = fuzzy_search_builtins(&entries, ".");

        assert!(matches.is_empty());
    }

    #[test]
    fn punctuation_only_query_does_not_match_builtin_keywords() {
        let entries = vec![builtin("Settings Hub", "Open app settings", &["config."])];

        let matches = fuzzy_search_builtins(&entries, ".");

        assert!(matches.is_empty());
    }

    #[test]
    fn punctuation_only_query_returns_no_builtin_even_when_non_menu_bar_name_matches() {
        let entries = vec![builtin("Open .env", "Open environment file", &[])];

        let matches = fuzzy_search_builtins(&entries, ".");

        assert!(
            matches.is_empty(),
            "punctuation-only launcher prefixes must not scan or return built-ins"
        );
    }

    #[test]
    fn menu_bar_punctuation_only_query_does_not_match_punctuated_name() {
        let entries = vec![builtin_with_group(
            "Chrome > File > Save As...",
            "",
            &[],
            BuiltInGroup::MenuBar,
        )];

        assert!(fuzzy_search_builtins(&entries, ".").is_empty());
        assert!(fuzzy_search_builtins(&entries, "...").is_empty());
    }

    #[test]
    fn menu_bar_punctuation_only_query_does_not_match_description_or_keywords() {
        let entries = vec![builtin_with_group(
            "Chrome > File > Save As",
            "Save the document as...",
            &["chrome.file.save.as", "..."],
            BuiltInGroup::MenuBar,
        )];

        assert!(fuzzy_search_builtins(&entries, ".").is_empty());
    }

    #[test]
    fn menu_bar_alphanumeric_query_still_matches_name() {
        let entries = vec![builtin_with_group(
            "Chrome > File > Save As...",
            "",
            &[],
            BuiltInGroup::MenuBar,
        )];

        let matches = fuzzy_search_builtins(&entries, "save");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].entry.name, "Chrome > File > Save As...");
        assert_eq!(
            matches[0]
                .match_evidence
                .as_ref()
                .map(|evidence| evidence.field),
            Some(MatchEvidenceField::Name)
        );
    }

    #[test]
    fn punctuation_only_query_skips_all_builtin_groups_and_fields() {
        let entries = vec![
            builtin("Open .env", "Open environment file", &[]),
            builtin("Settings Hub", "Open app settings.", &[]),
            builtin("Command Center", "Open command tools", &["launcher."]),
            builtin_with_group("Chrome > File > Save As...", "", &[], BuiltInGroup::MenuBar),
            builtin_with_group(
                "Chrome > File > Save As",
                "Save the document as...",
                &["chrome.file.save.as", "..."],
                BuiltInGroup::MenuBar,
            ),
        ];

        for query in [".", "...", ":", ";", "!"] {
            assert!(
                fuzzy_search_builtins(&entries, query).is_empty(),
                "query {query:?} should not produce built-in matches"
            );
        }
    }

    #[test]
    fn text_query_still_matches_builtin_description_and_keywords() {
        let entries = vec![
            builtin("Control Panel", "Open app settings", &[]),
            builtin("Command Center", "Open command tools", &["launcher"]),
        ];

        let description_matches = fuzzy_search_builtins(&entries, "settings");
        assert_eq!(description_matches.len(), 1);
        assert_eq!(
            description_matches[0]
                .match_evidence
                .as_ref()
                .map(|evidence| evidence.field),
            Some(MatchEvidenceField::Description)
        );

        let keyword_matches = fuzzy_search_builtins(&entries, "launcher");
        assert_eq!(keyword_matches.len(), 1);
        assert_eq!(
            keyword_matches[0]
                .match_evidence
                .as_ref()
                .map(|evidence| evidence.field),
            Some(MatchEvidenceField::Keyword)
        );
    }

    #[test]
    fn empty_query_still_returns_all_builtins() {
        let entries = vec![
            builtin("Settings Hub", "Open app settings", &[]),
            builtin("Command Center", "Open command tools", &["launcher"]),
        ];

        let matches = fuzzy_search_builtins(&entries, "");

        assert_eq!(matches.len(), 2);
        assert!(matches.iter().all(|result| result.score == 0));
        assert!(matches.iter().all(|result| result.match_evidence.is_none()));
    }
}
