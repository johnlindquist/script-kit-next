//! Result grouping for the main menu
//!
//! This module provides functions for grouping search results into
//! sections like RECENT, SCRIPTS, APPS, etc.
//!
//! When the filter is empty (grouped view), items are organized by their source kit:
//! - SUGGESTED (frecency-based recent items)
//! - {KIT_NAME} (e.g., CLEANSHOT, MAIN - containing scripts and scriptlets from that kit)
//! - COMMANDS (built-ins and window controls)
//! - APPS (installed applications)
//! - AGENTS (AI agents)

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, instrument};

use crate::app_launcher::AppInfo;
use crate::builtins::{menu_bar_items_to_entries, BuiltInEntry, BuiltInGroup};
use crate::config::SuggestedConfig;
use crate::fallbacks::collector::collect_fallbacks;
use crate::frecency::FrecencyStore;
use crate::list_item::GroupedListItem;
use crate::menu_bar::MenuBarItem;

use super::search::fuzzy_search_unified_all;
use super::types::{FallbackMatch, Script, Scriptlet, SearchResult};

/// Default maximum number of items to show in the RECENT section
pub const DEFAULT_MAX_RECENT_ITEMS: usize = 10;

/// Maximum number of menu bar items to show in search results
/// This prevents menu bar actions from overwhelming the results
pub const MAX_MENU_BAR_ITEMS: usize = 5;

/// Minimum score required for a menu bar item to appear in results
/// This filters out weak matches that would clutter the list
pub const MIN_MENU_BAR_SCORE: i32 = 25;

/// Get grouped results with SUGGESTED/MAIN sections based on frecency
///
/// This function creates a grouped view of search results:
///
/// **When filter_text is empty (grouped view):**
/// 1. Returns `SectionHeader("SUGGESTED")` if any items have frecency score > 0
/// 2. Suggested items sorted by frecency score (top 5-10 with score > 0)
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
/// * `suggested_config` - Configuration for the SUGGESTED section
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
    suggested_config: &SuggestedConfig,
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
        let mut results = results; // Make results mutable so we can append fallbacks

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
            ));
            for idx in menu_bar_indices {
                grouped.push(GroupedListItem::Item(idx));
            }
        }

        // Collect fallback commands and append as "Use {query} with..." section OR as primary results
        let fallbacks = collect_fallbacks(filter_text, scripts);
        let fallback_count = fallbacks.len();

        if !fallbacks.is_empty() {
            // When there are no other results, fallbacks are elevated to primary (no header)
            // This gives Raycast-style behavior where fallbacks become the main focus
            if has_other_results {
                // Add section header with the actual query text (fallbacks are secondary)
                grouped.push(GroupedListItem::SectionHeader(format!(
                    "Use \"{}\" with...",
                    filter_text
                )));
            }
            // When has_other_results is false, no header is added - fallbacks become primary

            // Append fallback items to the results vec and add their indices to grouped
            for fallback in fallbacks {
                let idx = results.len();
                results.push(SearchResult::Fallback(FallbackMatch {
                    fallback,
                    score: 0, // Fallbacks don't use score, they use priority
                }));
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
        return (grouped, results);
    }

    // Grouped view mode: create SUGGESTED and kit-based sections
    let mut grouped = Vec::new();

    // Get suggested items from frecency store (respecting config)
    let suggested_items = if suggested_config.enabled {
        frecency_store.get_recent_items(suggested_config.max_items)
    } else {
        Vec::new()
    };

    // Build a set of paths that are "suggested" (have frecency score above min_score)
    let min_score = suggested_config.min_score;
    let suggested_paths: HashSet<String> = suggested_items
        .iter()
        .filter(|(_, score): &&(String, f64)| *score >= min_score)
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
            SearchResult::Agent(am) => Some(format!("agent:{}", am.agent.path.to_string_lossy())),
            // Fallbacks don't have paths - they're only shown in search mode, not grouped view
            SearchResult::Fallback(_) => None,
        }
    };

    // Helper to get kit name from a result
    let get_kit_name = |result: &SearchResult| -> Option<String> {
        match result {
            SearchResult::Script(sm) => sm.script.kit_name.clone(),
            SearchResult::Scriptlet(sm) => sm.scriptlet.group.clone(),
            _ => None,
        }
    };

    // Find indices of results that are "suggested" and categorize non-suggested by kit or type
    let mut suggested_indices: Vec<(usize, f64)> = Vec::new();
    // Kit-based grouping: HashMap<kit_name, Vec<index>>
    let mut kit_indices: HashMap<String, Vec<usize>> = HashMap::new();
    let mut commands_indices: Vec<usize> = Vec::new();
    let mut apps_indices: Vec<usize> = Vec::new();
    let mut agents_indices: Vec<usize> = Vec::new();

    // Get excluded commands for filtering builtins from SUGGESTED section
    let excluded_commands = &suggested_config.excluded_commands;

    for (idx, result) in results.iter().enumerate() {
        if let Some(path) = get_result_path(result) {
            let score = frecency_store.get_score(&path);

            // Check if this builtin should be excluded from SUGGESTED
            // (e.g., "Quit Script Kit" shouldn't appear in suggested even if it has frecency)
            let is_excluded_builtin = match result {
                SearchResult::BuiltIn(bm) => {
                    bm.entry.should_exclude_from_frecency(excluded_commands)
                }
                _ => false,
            };

            if score >= min_score && suggested_paths.contains(&path) && !is_excluded_builtin {
                suggested_indices.push((idx, score));
            } else {
                // Categorize by kit (for scripts/scriptlets) or by type (for others)
                match result {
                    SearchResult::Script(_) | SearchResult::Scriptlet(_) => {
                        // Group by kit name (default to "main" if no kit specified)
                        let kit = get_kit_name(result).unwrap_or_else(|| "main".to_string());
                        kit_indices.entry(kit).or_default().push(idx);
                    }
                    SearchResult::BuiltIn(_) | SearchResult::Window(_) => {
                        commands_indices.push(idx)
                    }
                    SearchResult::App(_) => apps_indices.push(idx),
                    SearchResult::Agent(_) => agents_indices.push(idx),
                    // Fallbacks should never appear in grouped view - they're search-mode only
                    SearchResult::Fallback(_) => {}
                }
            }
        } else {
            // If no path, categorize by type (shouldn't happen, but handle gracefully)
            match result {
                SearchResult::Script(_) | SearchResult::Scriptlet(_) => {
                    let kit = get_kit_name(result).unwrap_or_else(|| "main".to_string());
                    kit_indices.entry(kit).or_default().push(idx);
                }
                SearchResult::BuiltIn(_) | SearchResult::Window(_) => commands_indices.push(idx),
                SearchResult::App(_) => apps_indices.push(idx),
                SearchResult::Agent(_) => agents_indices.push(idx),
                // Fallbacks should never appear in grouped view - they're search-mode only
                SearchResult::Fallback(_) => {}
            }
        }
    }

    // Sort suggested items by frecency score (highest first)
    suggested_indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    // Limit suggested items to max_items from config
    suggested_indices.truncate(suggested_config.max_items);

    // Sort each section alphabetically by name (case-insensitive)
    let sort_alphabetically = |indices: &mut Vec<usize>| {
        indices.sort_by(|&a, &b| {
            results[a]
                .name()
                .to_lowercase()
                .cmp(&results[b].name().to_lowercase())
        });
    };

    // Sort items within each kit section
    for indices in kit_indices.values_mut() {
        sort_alphabetically(indices);
    }
    sort_alphabetically(&mut commands_indices);
    sort_alphabetically(&mut apps_indices);
    sort_alphabetically(&mut agents_indices);

    // Get kit names sorted alphabetically (but "main" comes last for better UX)
    let mut kit_names: Vec<&String> = kit_indices.keys().collect();
    kit_names.sort_by(|a, b| {
        // "main" kit goes last, others sorted alphabetically
        match (a.as_str(), b.as_str()) {
            ("main", "main") => Ordering::Equal,
            ("main", _) => Ordering::Greater,
            (_, "main") => Ordering::Less,
            _ => a.to_lowercase().cmp(&b.to_lowercase()),
        }
    });

    // Build grouped list: SUGGESTED first (if enabled), then kit sections, COMMANDS, APPS, AGENTS
    if suggested_config.enabled && !suggested_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader("SUGGESTED".to_string()));
        for (idx, _score) in &suggested_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    // Add kit-based sections (e.g., "CLEANSHOT", "MAIN")
    for kit_name in &kit_names {
        if let Some(indices) = kit_indices.get(*kit_name) {
            if !indices.is_empty() {
                // Use uppercase kit name as section header
                grouped.push(GroupedListItem::SectionHeader(kit_name.to_uppercase()));
                for idx in indices {
                    grouped.push(GroupedListItem::Item(*idx));
                }
            }
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

    if !agents_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader("AGENTS".to_string()));
        for idx in &agents_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    // Calculate kit counts for logging
    let kit_count: usize = kit_indices.values().map(|v| v.len()).sum();

    debug!(
        suggested_count = suggested_indices.len(),
        kit_sections = kit_names.len(),
        kit_items_count = kit_count,
        commands_count = commands_indices.len(),
        apps_count = apps_indices.len(),
        agents_count = agents_indices.len(),
        total_grouped = grouped.len(),
        "Grouped view: created kit-based sections"
    );

    (grouped, results)
}
