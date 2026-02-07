use std::cmp::Ordering;

use crate::window_control::WindowInfo;

use super::super::types::WindowMatch;
use super::{find_ignore_ascii_case, NucleoCtx, MIN_FUZZY_QUERY_LEN};

/// Fuzzy search windows by query string
/// Searches across app name and window title
/// Returns results sorted by relevance score (highest first)
///
/// Scoring priorities:
/// - App name match at start: 100 points
/// - App name match elsewhere: 75 points
/// - Window title match at start: 90 points  
/// - Window title match elsewhere: 65 points
/// - Fuzzy match on app name: 50 points
/// - Fuzzy match on window title: 40 points
pub fn fuzzy_search_windows(windows: &[WindowInfo], query: &str) -> Vec<WindowMatch> {
    if query.is_empty() {
        // If no query, return all windows with equal score, sorted by app name then title
        let mut matches: Vec<WindowMatch> = windows
            .iter()
            .map(|w| WindowMatch {
                window: w.clone(),
                score: 0,
            })
            .collect();
        matches.sort_by(|a, b| match a.window.app.cmp(&b.window.app) {
            Ordering::Equal => a.window.title.cmp(&b.window.title),
            other => other,
        });
        return matches;
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    // Create nucleo context once for all windows - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    // Check if query is ASCII once for all items
    let query_is_ascii = query_lower.is_ascii();

    // Gate nucleo fuzzy matching on minimum query length to reduce noise
    let use_nucleo = query_lower.len() >= MIN_FUZZY_QUERY_LEN;

    for window in windows {
        let mut score = 0i32;

        // Score by app name match - highest priority
        // App names can have Unicode
        if query_is_ascii && window.app.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&window.app, &query_lower) {
                // Bonus for exact substring match at start of app name
                score += if pos == 0 { 100 } else { 75 };
            }
        }

        // Score by window title match - high priority
        // Window titles can have Unicode content
        if query_is_ascii && window.title.is_ascii() {
            if let Some(pos) = find_ignore_ascii_case(&window.title, &query_lower) {
                // Bonus for exact substring match at start of title
                score += if pos == 0 { 90 } else { 65 };
            }
        }

        // Fuzzy character matching in app name using nucleo (handles Unicode)
        if use_nucleo {
            if let Some(nucleo_s) = nucleo.score(&window.app) {
                // Scale nucleo score to match existing weights (~50 for app name fuzzy match)
                score += 50 + (nucleo_s / 20) as i32;
            }
        }

        // Fuzzy character matching in window title using nucleo (handles Unicode)
        if use_nucleo {
            if let Some(nucleo_s) = nucleo.score(&window.title) {
                // Scale nucleo score to match existing weights (~40 for title fuzzy match)
                score += 40 + (nucleo_s / 25) as i32;
            }
        }

        if score > 0 {
            matches.push(WindowMatch {
                window: window.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by app name, then by title for ties
    matches.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => match a.window.app.cmp(&b.window.app) {
            Ordering::Equal => a.window.title.cmp(&b.window.title),
            other => other,
        },
        other => other,
    });

    matches
}
