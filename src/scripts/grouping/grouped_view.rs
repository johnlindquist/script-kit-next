use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use tracing::debug;

use crate::config::SuggestedConfig;
use crate::frecency::FrecencyStore;
use crate::list_item::GroupedListItem;

use super::super::types::SearchResult;
use super::DEFAULT_SUGGESTED_ITEMS;

pub(super) fn build_grouped_view_results(
    results: Vec<SearchResult>,
    frecency_store: &FrecencyStore,
    suggested_config: &SuggestedConfig,
) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
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

    // Helper to get kit name from a result (scripts, scriptlets, and agents)
    let get_kit_name = |result: &SearchResult| -> Option<String> {
        match result {
            SearchResult::Script(sm) => sm.script.kit_name.clone(),
            SearchResult::Scriptlet(sm) => sm.scriptlet.group.clone(),
            SearchResult::Agent(am) => am.agent.kit.clone(),
            _ => None,
        }
    };

    // Find indices of results that are "suggested" and categorize non-suggested by kit or type
    let mut suggested_indices: Vec<(usize, f64)> = Vec::new();
    // Kit-based grouping: HashMap<kit_name, Vec<index>> (includes scripts, scriptlets, and agents)
    let mut kit_indices: HashMap<String, Vec<usize>> = HashMap::new();
    let mut commands_indices: Vec<usize> = Vec::new();
    let mut apps_indices: Vec<usize> = Vec::new();

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
                // Categorize by kit (for scripts/scriptlets/agents) or by type (for others)
                match result {
                    SearchResult::Script(_)
                    | SearchResult::Scriptlet(_)
                    | SearchResult::Agent(_) => {
                        // Group by kit name (default to "main" if no kit specified)
                        let kit = get_kit_name(result).unwrap_or_else(|| "main".to_string());
                        kit_indices.entry(kit).or_default().push(idx);
                    }
                    SearchResult::BuiltIn(_) | SearchResult::Window(_) => {
                        commands_indices.push(idx)
                    }
                    SearchResult::App(_) => apps_indices.push(idx),
                    // Fallbacks should never appear in grouped view - they're search-mode only
                    SearchResult::Fallback(_) => {}
                }
            }
        } else {
            // If no path, categorize by type (shouldn't happen, but handle gracefully)
            match result {
                SearchResult::Script(_) | SearchResult::Scriptlet(_) | SearchResult::Agent(_) => {
                    let kit = get_kit_name(result).unwrap_or_else(|| "main".to_string());
                    kit_indices.entry(kit).or_default().push(idx);
                }
                SearchResult::BuiltIn(_) | SearchResult::Window(_) => commands_indices.push(idx),
                SearchResult::App(_) => apps_indices.push(idx),
                // Fallbacks should never appear in grouped view - they're search-mode only
                SearchResult::Fallback(_) => {}
            }
        }
    }

    // Sort suggested items by frecency score (highest first)
    suggested_indices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    // Limit suggested items to max_items from config
    suggested_indices.truncate(suggested_config.max_items);

    // If no frecency-based suggestions and frecency store is empty (new user),
    // populate with default suggested items to help users discover features
    let default_suggested_indices: Vec<usize> =
        if suggested_indices.is_empty() && frecency_store.is_empty() && suggested_config.enabled {
            let mut first_index_by_name: HashMap<&str, usize> =
                HashMap::with_capacity(results.len());
            for (idx, result) in results.iter().enumerate() {
                first_index_by_name.entry(result.name()).or_insert(idx);
            }

            // Find indices of default suggested items by name (preserve order from DEFAULT_SUGGESTED_ITEMS)
            DEFAULT_SUGGESTED_ITEMS
                .iter()
                .filter_map(|&default_name| first_index_by_name.get(default_name).copied())
                .collect()
        } else {
            Vec::new()
        };

    // Remove default suggested items from other sections to avoid duplication
    if !default_suggested_indices.is_empty() {
        let default_set: HashSet<usize> = default_suggested_indices.iter().copied().collect();
        commands_indices.retain(|idx| !default_set.contains(idx));
        apps_indices.retain(|idx| !default_set.contains(idx));
        for indices in kit_indices.values_mut() {
            indices.retain(|idx| !default_set.contains(idx));
        }
    }

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

    // Get non-main kit names sorted alphabetically
    let mut other_kit_names: Vec<&String> = kit_indices
        .keys()
        .filter(|k| k.as_str() != "main")
        .collect();
    other_kit_names.sort_by_key(|a| a.to_lowercase());

    // Build grouped list in order: SUGGESTED, MAIN, COMMANDS, other kits, APPS
    // Each section header includes an item count suffix (e.g., "SUGGESTED · 5")
    // 1. SUGGESTED (frecency-based, or default items for new users)
    if suggested_config.enabled {
        if !suggested_indices.is_empty() {
            // User has frecency data - show their frequently used items
            grouped.push(GroupedListItem::SectionHeader(
                format!("SUGGESTED · {}", suggested_indices.len()),
                Some("StarFilled".to_string()),
            ));
            for (idx, _score) in &suggested_indices {
                grouped.push(GroupedListItem::Item(*idx));
            }
        } else if !default_suggested_indices.is_empty() {
            // New user with no frecency - show default suggestions
            grouped.push(GroupedListItem::SectionHeader(
                format!("SUGGESTED · {}", default_suggested_indices.len()),
                Some("StarFilled".to_string()),
            ));
            for idx in &default_suggested_indices {
                grouped.push(GroupedListItem::Item(*idx));
            }
        }
    }

    // 2. MAIN kit (if it has items)
    if let Some(main_indices) = kit_indices.get("main") {
        if !main_indices.is_empty() {
            grouped.push(GroupedListItem::SectionHeader(
                format!("MAIN · {}", main_indices.len()),
                Some("Code".to_string()),
            ));
            for idx in main_indices {
                grouped.push(GroupedListItem::Item(*idx));
            }
        }
    }

    // 3. COMMANDS (built-ins and window controls)
    if !commands_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader(
            format!("COMMANDS · {}", commands_indices.len()),
            Some("Terminal".to_string()),
        ));
        for idx in &commands_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    // 4. Other kit sections (CLEANSHOT, etc.) - alphabetically sorted
    for kit_name in &other_kit_names {
        if let Some(indices) = kit_indices.get(*kit_name) {
            if !indices.is_empty() {
                // Use uppercase kit name as section header with count and bolt icon
                grouped.push(GroupedListItem::SectionHeader(
                    format!("{} · {}", kit_name.to_uppercase(), indices.len()),
                    Some("BoltFilled".to_string()),
                ));
                for idx in indices {
                    grouped.push(GroupedListItem::Item(*idx));
                }
            }
        }
    }

    // 5. APPS (installed applications)
    if !apps_indices.is_empty() {
        grouped.push(GroupedListItem::SectionHeader(
            format!("APPS · {}", apps_indices.len()),
            Some("Folder".to_string()),
        ));
        for idx in &apps_indices {
            grouped.push(GroupedListItem::Item(*idx));
        }
    }

    // Note: Agents are now grouped by kit, no separate AGENTS section

    // Calculate kit counts for logging
    let kit_count: usize = kit_indices.values().map(|v| v.len()).sum();

    debug!(
        suggested_count = suggested_indices.len(),
        kit_sections = kit_indices.len(),
        kit_items_count = kit_count,
        commands_count = commands_indices.len(),
        apps_count = apps_indices.len(),
        total_grouped = grouped.len(),
        "Grouped view: created kit-based sections (scripts, scriptlets, agents grouped by kit)"
    );

    (grouped, results)
}
