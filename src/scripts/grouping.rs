//! Result grouping for the main menu
//!
//! This module provides functions for grouping search results into
//! sections like RECENT, SCRIPTS, APPS, etc.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::app_launcher::AppInfo;
use crate::builtins::{menu_bar_items_to_entries, BuiltInEntry, BuiltInGroup};
use crate::frecency::FrecencyStore;
use crate::list_item::GroupedListItem;
use crate::menu_bar::MenuBarItem;

use super::search::fuzzy_search_unified_all;
use super::types::{Script, Scriptlet, SearchResult};

/// Default maximum number of items to show in the RECENT section
pub const DEFAULT_MAX_RECENT_ITEMS: usize = 10;

/// Get grouped results with RECENT/MAIN sections based on frecency
///
/// This function creates a grouped view of search results:
///
/// **When filter_text is empty (grouped view):**
/// 1. Returns `SectionHeader("RECENT")` if any items have frecency score > 0
/// 2. Recent items sorted by frecency score (top 5-10 with score > 0)
/// 3. Returns `SectionHeader("MAIN")`
/// 4. Remaining items sorted alphabetically by name
///
/// **When filter_text has content (search mode):**
/// - Returns flat list of `Item(index)` - no headers
/// - Uses existing fuzzy_search_unified logic for filtering
/// - Also includes menu bar items from the frontmost application (if provided)
///
/// # Arguments
/// * `scripts` - Scripts to include in results
/// * `scriptlets` - Scriptlets to include in results
/// * `builtins` - Built-in entries to include in results
/// * `apps` - Application entries to include in results
/// * `frecency_store` - Store containing frecency data for ranking
/// * `filter_text` - Search filter text (empty = grouped view, non-empty = search mode)
/// * `max_recent_items` - Maximum items to show in RECENT section (from config)
/// * `menu_bar_items` - Optional menu bar items from the frontmost application
/// * `menu_bar_bundle_id` - Optional bundle ID of the frontmost application
///
/// # Returns
/// `(Vec<GroupedListItem>, Vec<SearchResult>)` - Grouped items and the flat results array.
/// The `usize` in `Item(usize)` is the index into the flat results array.
///
/// H1 Optimization: Accepts Arc<Script> and Arc<Scriptlet> to avoid expensive clones.
#[instrument(level = "debug", skip_all, fields(filter_len = filter_text.len()))]
#[allow(clippy::too_many_arguments)]
pub fn get_grouped_results(
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    builtins: &[BuiltInEntry],
    apps: &[AppInfo],
    frecency_store: &FrecencyStore,
    filter_text: &str,
    max_recent_items: usize,
    menu_bar_items: &[MenuBarItem],
    menu_bar_bundle_id: Option<&str>,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
    // When filter is non-empty and we have menu bar items, include them in search
    let all_builtins: Vec<BuiltInEntry>;
    let builtins_to_use: &[BuiltInEntry] = if let Some(bundle_id) =
        menu_bar_bundle_id.filter(|_| !filter_text.is_empty() && !menu_bar_items.is_empty())
    {
        // Extract app name from bundle_id (e.g., "com.apple.Safari" -> "Safari")
        let app_name = bundle_id.rsplit('.').next().unwrap_or(bundle_id);
        let menu_entries = menu_bar_items_to_entries(menu_bar_items, bundle_id, app_name);
        // Combine builtins with menu bar entries
        all_builtins = builtins.iter().cloned().chain(menu_entries).collect();
        &all_builtins
    } else {
        builtins
    };

    // Get all unified search results
    let results = fuzzy_search_unified_all(scripts, scriptlets, builtins_to_use, apps, filter_text);

    // Search mode: return flat list with section header for menu bar items
    if !filter_text.is_empty() {
        // Partition results into non-menu-bar and menu-bar items
        let mut non_menu_bar_indices: Vec<usize> = Vec::new();
        let mut menu_bar_indices: Vec<usize> = Vec::new();

        for (idx, result) in results.iter().enumerate() {
            if matches!(result, SearchResult::BuiltIn(bm) if bm.entry.group == BuiltInGroup::MenuBar)
            {
                menu_bar_indices.push(idx);
            } else {
                non_menu_bar_indices.push(idx);
            }
        }

        let mut grouped: Vec<GroupedListItem> = Vec::new();

        // Add non-menu-bar items first
        for idx in non_menu_bar_indices {
            grouped.push(GroupedListItem::Item(idx));
        }

        // Add menu bar section with header if there are menu bar items
        let menu_bar_count = menu_bar_indices.len();
        if !menu_bar_indices.is_empty() {
            grouped.push(GroupedListItem::SectionHeader(
                "MENU BAR ACTIONS".to_string(),
            ));
            for idx in menu_bar_indices {
                grouped.push(GroupedListItem::Item(idx));
            }
        }

        debug!(
            result_count = results.len(),
            menu_bar_count, "Search mode: returning list with menu bar section"
        );
        return (grouped, results);
    }

    // Grouped view mode: create RECENT and type-based sections
    let mut grouped = Vec::new();

    // Get recent items from frecency store
    let recent_items = frecency_store.get_recent_items(max_recent_items);

    // Build a set of paths that are "recent" (have frecency score > 0)
    let recent_paths: HashSet<String> = recent_items
        .iter()
        .filter(|(_, score): &&(String, f64)| *score > 0.0)
        .map(|(path, _): &(String, f64)| path.clone())
        .collect();

    // Map each result to its frecency score (if any)
    // We need to get the path for each result type
    let get_result_path = |result: &SearchResult| -> Option<String> {
        match result {
            SearchResult::Script(sm) => Some(sm.script.path.to_string_lossy().to_string()),
            SearchResult::App(am) => Some(am.app.path.to_string_lossy().to_string()),
            SearchResult::BuiltIn(bm) => Some(format!("builtin:{}", bm.entry.name)),
            SearchResult::Scriptlet(sm) => Some(format!("scriptlet:{}", sm.scriptlet.name)),
            SearchResult::Window(wm) => {
                Some(format!("window:{}:{}", wm.window.app, wm.window.title))
            }
        }
    };

    // Find indices of results that are "recent" and categorize non-recent by type
    let mut recent_indices: Vec<(usize, f64)> = Vec::new();
    let mut scripts_indices: Vec<usize> = Vec::new();
    let mut scriptlets_indices: Vec<usize> = Vec::new();
    let mut commands_indices: Vec<usize> = Vec::new();
    let mut apps_indices: Vec<usize> = Vec::new();

    for (idx, result) in results.iter().enumerate() {
        if let Some(path) = get_result_path(result) {
            let score = frecency_store.get_score(&path);
            if score > 0.0 && recent_paths.contains(&path) {
                recent_indices.push((idx, score));
            } else {
                // Categorize by SearchResult variant
                match result {
                    SearchResult::Script(_) => scripts_indices.push(idx),
                    SearchResult::Scriptlet(_) => scriptlets_indices.push(idx),
                    SearchResult::BuiltIn(_) | SearchResult::Window(_) => {
                        commands_indices.push(idx)
                    }
                    SearchResult::App(_) => apps_indices.push(idx),
                }
            }
        } else {
            // If no path, categorize by type (shouldn't happen, but handle gracefully)
            match result {
                SearchResult::Script(_) => scripts_indices.push(idx),
                SearchResult::Scriptlet(_) => scriptlets_indices.push(idx),
                SearchResult::BuiltIn(_) | SearchResult::Window(_) => commands_indices.push(idx),
                SearchResult::App(_) => apps_indices.push(idx),
            }
        }
    }

    // Sort recent items by frecency score (highest first)
    recent_indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    // Limit recent items to max_recent_items
    recent_indices.truncate(max_recent_items);

    // Sort each type section alphabetically by name (case-insensitive)
    let sort_alphabetically = |indices: &mut Vec<usize>| {
        indices.sort_by(|&a, &b| {
            results[a]
                .name()
                .to_lowercase()
                .cmp(&results[b].name().to_lowercase())
        });
    };

    sort_alphabetically(&mut scripts_indices);
    sort_alphabetically(&mut scriptlets_indices);
    sort_alphabetically(&mut commands_indices);
    sort_alphabetically(&mut apps_indices);

    // Build grouped list: RECENT first, then SCRIPTS, SCRIPTLETS, COMMANDS, APPS
    if !recent_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader("RECENT".to_string()));
        for (idx, _score) in &recent_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    if !scripts_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader("SCRIPTS".to_string()));
        for idx in &scripts_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    if !scriptlets_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader("SCRIPTLETS".to_string()));
        for idx in &scriptlets_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    if !commands_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader("COMMANDS".to_string()));
        for idx in &commands_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    if !apps_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader("APPS".to_string()));
        for idx in &apps_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    debug!(
        recent_count = recent_indices.len(),
        scripts_count = scripts_indices.len(),
        scriptlets_count = scriptlets_indices.len(),
        commands_count = commands_indices.len(),
        apps_count = apps_indices.len(),
        total_grouped = grouped.len(),
        "Grouped view: created type-based sections"
    );

    (grouped, results)
}
