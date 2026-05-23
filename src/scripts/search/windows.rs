use std::cmp::Ordering;

use crate::scripts::RootWindowEntry;
use crate::window_control::WindowInfo;

use super::super::types::WindowMatch;
use super::{
    better_match, low_tier_substring_match, primary_text_match, NucleoCtx, TIER_DESCRIPTION,
};

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
    let entries = windows
        .iter()
        .cloned()
        .map(|window| RootWindowEntry {
            subtitle: window.descriptor.clone(),
            window,
            app_icon: None,
            duplicate_rank: None,
            duplicate_count: 1,
            local_recency_seq: None,
        })
        .collect::<Vec<_>>();
    fuzzy_search_root_windows(&entries, query)
}

/// Fuzzy search root/unified window entries with app-layer icon/subtitle data.
pub fn fuzzy_search_root_windows(windows: &[RootWindowEntry], query: &str) -> Vec<WindowMatch> {
    if query.is_empty() {
        // If no query, browse by practical focus/recency signals with stable fallbacks.
        let mut matches: Vec<usize> = (0..windows.len()).collect();
        matches.sort_by(|a_idx, b_idx| compare_window_entries(&windows[*a_idx], &windows[*b_idx]));
        return matches
            .into_iter()
            .map(|index| WindowMatch {
                window: windows[index].window.clone(),
                app_icon: windows[index].app_icon.clone(),
                subtitle: windows[index].subtitle.clone(),
                score: 0,
            })
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches: Vec<(usize, i32)> = Vec::with_capacity(windows.len());

    // Create nucleo context once for all windows - reuses buffer across calls
    let mut nucleo = NucleoCtx::new(&query_lower);
    for (index, entry) in windows.iter().enumerate() {
        let window = &entry.window;
        let mut best = None;
        better_match(
            &mut best,
            primary_text_match(&window.app, &query_lower, &mut nucleo),
        );
        better_match(
            &mut best,
            primary_text_match(&window.title, &query_lower, &mut nucleo),
        );
        better_match(
            &mut best,
            low_tier_substring_match(&entry.subtitle, &query_lower, TIER_DESCRIPTION),
        );

        if let Some(best) = best {
            matches.push((index, best.score));
        }
    }

    // Sort by score (highest first), then focus/recency signals for ties.
    matches.sort_by(
        |(a_idx, a_score), (b_idx, b_score)| match b_score.cmp(a_score) {
            Ordering::Equal => compare_window_entries(&windows[*a_idx], &windows[*b_idx]),
            other => other,
        },
    );

    matches
        .into_iter()
        .map(|(index, score)| WindowMatch {
            window: windows[index].window.clone(),
            app_icon: windows[index].app_icon.clone(),
            subtitle: windows[index].subtitle.clone(),
            score,
        })
        .collect()
}

fn compare_window_entries(a: &RootWindowEntry, b: &RootWindowEntry) -> Ordering {
    b.window
        .is_frontmost_app
        .cmp(&a.window.is_frontmost_app)
        .then_with(|| b.window.is_focused.cmp(&a.window.is_focused))
        .then_with(|| b.window.is_main.cmp(&a.window.is_main))
        .then_with(|| b.local_recency_seq.cmp(&a.local_recency_seq))
        .then_with(|| a.window.is_minimized.cmp(&b.window.is_minimized))
        .then_with(|| a.window.app_order.cmp(&b.window.app_order))
        .then_with(|| a.window.window_index.cmp(&b.window.window_index))
        .then_with(|| a.window.title.cmp(&b.window.title))
        .then_with(|| a.window.id.cmp(&b.window.id))
}
