use std::cmp::Ordering;

use crate::app_launcher::AppInfo;

use super::super::types::AppMatch;
use super::{
    contains_ignore_ascii_case, find_ignore_ascii_case, is_exact_name_match,
    is_word_boundary_match, NucleoCtx, MIN_FUZZY_QUERY_LEN,
};

/// Fuzzy search applications by query string
/// Searches across name and bundle_id
/// Returns results sorted by relevance score (highest first)
pub fn fuzzy_search_apps(apps: &[AppInfo], query: &str) -> Vec<AppMatch> {
    if query.is_empty() {
        // If no query, return all apps with equal score, sorted by name
        return apps
            .iter()
            .map(|a| AppMatch {
                app: a.clone(),
                score: 0,
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all apps - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    let query_is_ascii = query_lower.is_ascii();

    // Gate nucleo fuzzy matching on minimum query length to reduce noise
    let use_nucleo = query_lower.len() >= MIN_FUZZY_QUERY_LEN;

    for app in apps {
        let mut score = 0i32;

        // Exact name match boost
        if query_is_ascii && app.name.is_ascii() && is_exact_name_match(&app.name, &query_lower) {
            score += 500;
        }

        // Score by name match - highest priority
        // App names can have Unicode (e.g., "日本語アプリ")
        if query_is_ascii && app.name.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&app.name, &query_lower) {
                // Bonus for exact substring match at start of name
                score += if pos == 0 { 100 } else { 75 };
                // Extra bonus for word-boundary matches
                if pos > 0 && is_word_boundary_match(&app.name, pos) {
                    score += 20;
                }
            }
        }

        // Fuzzy character matching in name using nucleo (handles Unicode)
        if use_nucleo {
            if let Some(nucleo_s) = nucleo.score(&app.name) {
                // Scale nucleo score to match existing weights (~50 for fuzzy match)
                score += 50 + (nucleo_s / 20) as i32;
            }
        }

        // Score by bundle_id match - lower priority
        // Bundle IDs are always ASCII (e.g., "com.apple.Safari")
        if let Some(ref bundle_id) = app.bundle_id {
            if query_is_ascii
                && bundle_id.is_ascii()
                && contains_ignore_ascii_case(bundle_id, &query_lower)
            {
                score += 15;
            }
        }

        // Score by path match - lowest priority
        // Paths are typically ASCII
        let path_str = app.path.to_string_lossy();
        if query_is_ascii
            && path_str.is_ascii()
            && contains_ignore_ascii_case(&path_str, &query_lower)
        {
            score += 5;
        }

        if score > 0 {
            matches.push(AppMatch {
                app: app.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.app.name.cmp(&b.app.name),
        other => other,
    });

    matches
}
