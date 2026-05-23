use std::cmp::Ordering;

use crate::app_launcher::AppInfo;

use super::super::types::AppMatch;
use super::{primary_text_match, NucleoCtx};

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
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches: Vec<(usize, i32)> = Vec::with_capacity(apps.len());

    // Create nucleo context once for all apps - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    for (index, app) in apps.iter().enumerate() {
        if let Some(name_match) = primary_text_match(&app.name, &query_lower, &mut nucleo) {
            matches.push((index, name_match.score));
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(
        |(a_idx, a_score), (b_idx, b_score)| match b_score.cmp(a_score) {
            Ordering::Equal => apps[*a_idx].name.cmp(&apps[*b_idx].name),
            other => other,
        },
    );

    matches
        .into_iter()
        .map(|(index, score)| AppMatch {
            app: apps[index].clone(),
            score,
        })
        .collect()
}
