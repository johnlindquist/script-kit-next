//! Result grouping for the main menu
//!
//! This module provides functions for grouping search results into
//! sections based on their source kit.
//!
//! When the filter is empty (grouped view), items are organized by their source kit:
//! - SUGGESTED (frecency-based recent items)
//! - {KIT_NAME} (e.g., CLEANSHOT, MAIN - containing scripts, scriptlets, AND agents from that kit)
//! - COMMANDS (built-ins and window controls)
//! - APPS (installed applications)
//!
//! Note: Scripts, scriptlets, and agents are all grouped under their source kit section.
//! The "main" kit appears last in the kit-based sections.

use std::sync::Arc;
use tracing::instrument;

use crate::app_launcher::AppInfo;
use crate::builtins::{menu_bar_items_to_entries, BuiltInEntry};
use crate::config::SuggestedConfig;
use crate::frecency::FrecencyStore;
use crate::list_item::GroupedListItem;
use crate::menu_bar::MenuBarItem;

use super::search::fuzzy_search_unified_all;
use super::types::{Script, Scriptlet, SearchResult};

mod grouped_view;
mod search_mode;

/// Default maximum number of items to show in the RECENT section
pub const DEFAULT_MAX_RECENT_ITEMS: usize = 10;

/// Default suggested item names for new users without frecency data.
/// These appear in the SUGGESTED section when the user has no usage history.
/// Order matters - items will appear in this order.
pub const DEFAULT_SUGGESTED_ITEMS: &[&str] = &[
    "AI Chat",
    "Notes",
    "Clipboard History",
    "Quick Terminal",
    "Search Files",
    "Configure Vercel AI Gateway",
];

/// Maximum number of menu bar items to show in search results
/// This prevents menu bar actions from overwhelming the results
pub const MAX_MENU_BAR_ITEMS: usize = 5;

/// Minimum score required for a menu bar item to appear in results
/// This filters out weak matches that would clutter the list
pub const MIN_MENU_BAR_SCORE: i32 = 25;

/// Get grouped results with SUGGESTED/MAIN sections based on frecency.
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
    // When filter is non-empty and we have menu bar items, include them in search.
    let all_builtins: Vec<BuiltInEntry>;
    let builtins_to_use: &[BuiltInEntry] = if let Some(bundle_id) =
        menu_bar_bundle_id.filter(|_| !filter_text.is_empty() && !menu_bar_items.is_empty())
    {
        let app_name = bundle_id.rsplit('.').next().unwrap_or(bundle_id);
        let menu_entries = menu_bar_items_to_entries(menu_bar_items, bundle_id, app_name);
        all_builtins = builtins.iter().cloned().chain(menu_entries).collect();
        &all_builtins
    } else {
        builtins
    };

    let results = fuzzy_search_unified_all(scripts, scriptlets, builtins_to_use, apps, filter_text);

    if !filter_text.is_empty() {
        return search_mode::build_search_mode_results(
            results,
            scripts,
            frecency_store,
            filter_text,
        );
    }

    grouped_view::build_grouped_view_results(results, frecency_store, suggested_config)
}
