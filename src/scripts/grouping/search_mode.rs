use std::sync::Arc;
use tracing::debug;

use crate::builtins::BuiltInGroup;
use crate::fallbacks::collector::collect_fallbacks;
use crate::frecency::FrecencyStore;
use crate::list_item::GroupedListItem;

use super::super::types::{FallbackMatch, Script, SearchResult};
use super::{MAX_MENU_BAR_ITEMS, MIN_MENU_BAR_SCORE};

pub(super) fn build_search_mode_results(
    mut results: Vec<SearchResult>,
    scripts: &[Arc<Script>],
    frecency_store: &FrecencyStore,
    filter_text: &str,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    // Apply frecency boost: recently/frequently used items get a score bonus.
    // This is how modern launchers (Raycast, Alfred, Spotlight) work.
    // The bonus is capped so a good fuzzy match still beats a poor match with high frecency.
    {
        let max_frecency_bonus = 50i32;

        // Helper to get the frecency path for a result (mirrors grouped-view logic)
        let get_path = |result: &SearchResult| -> Option<String> {
            match result {
                SearchResult::Script(sm) => Some(sm.script.path.to_string_lossy().to_string()),
                SearchResult::App(am) => Some(am.app.path.to_string_lossy().to_string()),
                SearchResult::BuiltIn(bm) => Some(format!("builtin:{}", bm.entry.name)),
                SearchResult::Scriptlet(sm) => Some(format!("scriptlet:{}", sm.scriptlet.name)),
                SearchResult::Window(wm) => {
                    Some(format!("window:{}:{}", wm.window.app, wm.window.title))
                }
                SearchResult::Agent(am) => {
                    Some(format!("agent:{}", am.agent.path.to_string_lossy()))
                }
                SearchResult::Fallback(_) => None,
            }
        };

        // Pre-compute boosted score for every result
        let boosted: Vec<i32> = results
            .iter()
            .map(|result| {
                let frecency_bonus = if let Some(path) = get_path(result) {
                    let score = frecency_store.get_score(&path);
                    if score > 0.0 {
                        // Scale frecency (typically 0-100+) via log so very high values
                        // don't dominate. At least 1 point bonus for any frecency > 0.
                        let scaled =
                            (score.ln().max(0.0) * 10.0).min(max_frecency_bonus as f64) as i32;
                        scaled.max(1)
                    } else {
                        0
                    }
                } else {
                    0
                };
                result.score() + frecency_bonus
            })
            .collect();

        // Build an index array sorted by boosted score descending, then name ascending
        let mut sort_indices: Vec<usize> = (0..results.len()).collect();
        sort_indices.sort_by(|&a, &b| {
            boosted[b]
                .cmp(&boosted[a])
                .then_with(|| results[a].name().cmp(results[b].name()))
        });

        // Re-order results according to boosted sort
        let reordered: Vec<SearchResult> = sort_indices
            .into_iter()
            .map(|i| results[i].clone())
            .collect();
        results = reordered;
    }

    // Partition results into non-menu-bar and menu-bar items
    let mut non_menu_bar_indices: Vec<usize> = Vec::new();
    let mut menu_bar_indices: Vec<usize> = Vec::new();

    for (idx, result) in results.iter().enumerate() {
        if let SearchResult::BuiltIn(bm) = result {
            if bm.entry.group == BuiltInGroup::MenuBar {
                // Only include menu bar items that meet minimum score threshold
                if bm.score >= MIN_MENU_BAR_SCORE {
                    menu_bar_indices.push(idx);
                }
                continue;
            }
        }
        non_menu_bar_indices.push(idx);
    }

    // Limit menu bar items to prevent overwhelming results
    menu_bar_indices.truncate(MAX_MENU_BAR_ITEMS);

    let mut grouped: Vec<GroupedListItem> = Vec::new();

    // Track counts before consuming the vectors
    let non_menu_bar_count = non_menu_bar_indices.len();
    let menu_bar_count = menu_bar_indices.len();
    let has_other_results = non_menu_bar_count > 0 || menu_bar_count > 0;

    // Add non-menu-bar items first
    for idx in non_menu_bar_indices {
        grouped.push(GroupedListItem::Item(idx));
    }

    // Add menu bar section with header if there are menu bar items
    if menu_bar_count > 0 {
        grouped.push(GroupedListItem::SectionHeader(
            "MENU BAR ACTIONS".to_string(),
            None,
        ));
        for idx in menu_bar_indices {
            grouped.push(GroupedListItem::Item(idx));
        }
    }

    // Collect fallback commands and append as "Use {query} with..." section
    let fallbacks = collect_fallbacks(filter_text, scripts);
    let fallback_count = fallbacks.len();

    if !fallbacks.is_empty() {
        // Always show "Use X with..." header (no icon)
        grouped.push(GroupedListItem::SectionHeader(
            format!("Use \"{}\" with...", filter_text),
            None,
        ));

        // Append fallback items to the results vec and add their indices to grouped
        for fallback in fallbacks {
            let idx = results.len();
            results.push(SearchResult::Fallback(FallbackMatch { fallback, score: 0 }));
            grouped.push(GroupedListItem::Item(idx));
        }
    }

    let fallbacks_elevated = fallback_count > 0 && !has_other_results;
    debug!(
        result_count = results.len(),
        menu_bar_count,
        fallback_count,
        fallbacks_elevated,
        "Search mode: returning list with menu bar and fallback sections"
    );

    (grouped, results)
}
