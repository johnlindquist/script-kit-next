// ============================================================================
// Menu Bar Item Conversion
// ============================================================================

/// Convert menu bar items to built-in entries for search
///
/// This flattens the menu hierarchy into searchable entries, skipping the
/// Apple menu (first item) and only including leaf items (no submenus).
///
/// # Arguments
/// * `items` - The menu bar items from the frontmost application
/// * `bundle_id` - The bundle identifier of the application (e.g., "com.apple.Safari")
/// * `app_name` - The display name of the application (e.g., "Safari")
///
/// # Returns
/// A vector of `BuiltInEntry` items that can be added to search results
#[allow(dead_code)] // Will be used when menu bar integration is complete
pub fn menu_bar_items_to_entries(
    items: &[MenuBarItem],
    bundle_id: &str,
    app_name: &str,
) -> Vec<BuiltInEntry> {
    let mut entries = Vec::new();

    // Skip first item (Apple menu)
    for item in items.iter().skip(1) {
        flatten_menu_item(item, bundle_id, app_name, &[], &mut entries);
    }

    debug!(
        count = entries.len(),
        bundle_id = bundle_id,
        app_name = app_name,
        "Menu bar items converted to entries"
    );
    entries
}
/// Recursively flatten a menu item and its children into entries
#[allow(dead_code)] // Will be used when menu bar integration is complete
fn flatten_menu_item(
    item: &MenuBarItem,
    bundle_id: &str,
    app_name: &str,
    parent_path: &[String],
    entries: &mut Vec<BuiltInEntry>,
) {
    // Skip separators and disabled items
    if item.title.is_empty() || item.title == "-" || item.is_separator() || !item.enabled {
        return;
    }

    let mut current_path = parent_path.to_vec();
    current_path.push(item.title.clone());

    // Only add leaf items (items without children) as entries
    if item.children.is_empty() {
        let id = format!(
            "menubar-{}-{}",
            bundle_id,
            current_path.join("-").to_lowercase().replace(' ', "-")
        );
        let name = current_path.join(" → ");
        let description = if let Some(ref shortcut) = item.shortcut {
            format!("{}  {}", app_name, shortcut.to_display_string())
        } else {
            app_name.to_string()
        };
        let keywords: Vec<String> = current_path.iter().map(|s| s.to_lowercase()).collect();
        let icon = get_menu_icon(&current_path[0]);

        entries.push(BuiltInEntry {
            id,
            name,
            description,
            keywords,
            feature: BuiltInFeature::MenuBarAction(MenuBarActionInfo {
                bundle_id: bundle_id.to_string(),
                menu_path: current_path,
                enabled: item.enabled,
                shortcut: item.shortcut.as_ref().map(|s| s.to_display_string()),
            }),
            icon: Some(icon.to_string()),
            group: BuiltInGroup::MenuBar,
        });
    } else {
        // Recurse into children
        for child in &item.children {
            flatten_menu_item(child, bundle_id, app_name, &current_path, entries);
        }
    }
}
/// Get an appropriate icon for a top-level menu
#[allow(dead_code)] // Will be used when menu bar integration is complete
fn get_menu_icon(top_menu: &str) -> &'static str {
    match top_menu.to_lowercase().as_str() {
        "file" => "📁",
        "edit" => "📋",
        "view" => "👁",
        "window" => "🪟",
        "help" => "❓",
        "format" => "🎨",
        "tools" => "🔧",
        "go" => "➡️",
        "bookmarks" | "favorites" => "⭐",
        "history" => "🕐",
        "develop" | "developer" => "🛠",
        _ => "📌",
    }
}
