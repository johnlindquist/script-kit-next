//! Frontmost-menu snapshot contract for the "Current App Commands" built-in.
//!
//! Captures the menu bar of the frontmost (non–Script Kit) application as a
//! self-contained snapshot that can be converted into searchable `BuiltInEntry`
//! items without holding any platform handles.

use crate::builtins::BuiltInEntry;
use crate::menu_bar::MenuBarItem;

/// A point-in-time capture of the frontmost application's menu bar.
#[derive(Debug, Clone)]
pub struct FrontmostMenuSnapshot {
    /// Localised display name (e.g. "Safari").
    pub app_name: String,
    /// Bundle identifier (e.g. "com.apple.Safari").
    pub bundle_id: String,
    /// Top-level menu bar items with full hierarchy.
    pub items: Vec<MenuBarItem>,
}

/// A machine-readable receipt for a frontmost-menu snapshot capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmostMenuSnapshotReceipt {
    pub app_name: String,
    pub bundle_id: String,
    pub top_level_menu_count: usize,
    pub leaf_entry_count: usize,
    pub placeholder: String,
    pub source: &'static str,
}

impl FrontmostMenuSnapshot {
    /// Convert the snapshot into flat, searchable built-in entries.
    ///
    /// Delegates to [`crate::builtins::menu_bar_items_to_entries`] which skips
    /// the Apple menu, separators, and disabled items.
    pub fn into_entries(self) -> Vec<BuiltInEntry> {
        self.into_entries_with_receipt().0
    }

    /// Convert the snapshot into entries and an audit-friendly receipt.
    pub fn into_entries_with_receipt(self) -> (Vec<BuiltInEntry>, FrontmostMenuSnapshotReceipt) {
        let entries = crate::builtins::menu_bar_items_to_entries(
            &self.items,
            &self.bundle_id,
            &self.app_name,
        );

        let receipt = FrontmostMenuSnapshotReceipt {
            app_name: self.app_name.clone(),
            bundle_id: self.bundle_id.clone(),
            top_level_menu_count: self.items.len(),
            leaf_entry_count: entries.len(),
            placeholder: self.placeholder(),
            source: "frontmost_app_tracker",
        };

        (entries, receipt)
    }

    /// Placeholder text for the filter input (e.g. "Search Safari commands…").
    pub fn placeholder(&self) -> String {
        format!("Search {} commands\u{2026}", self.app_name)
    }
}

// ---------------------------------------------------------------------------
// Platform loader
// ---------------------------------------------------------------------------

/// Load a [`FrontmostMenuSnapshot`] from the current frontmost application.
///
/// On macOS this reads the pre-cached menu items from the frontmost-app tracker.
/// On other platforms it returns a deterministic "unsupported" error.
#[cfg(target_os = "macos")]
pub fn load_frontmost_menu_snapshot() -> anyhow::Result<FrontmostMenuSnapshot> {
    use anyhow::Context;

    let tracked_app = crate::frontmost_app_tracker::get_last_real_app()
        .context("No frontmost application tracked — is the app tracker running?")?;

    let items = crate::frontmost_app_tracker::get_cached_menu_items();

    tracing::info!(
        app_name = %tracked_app.name,
        bundle_id = %tracked_app.bundle_id,
        item_count = items.len(),
        "frontmost_menu_snapshot.loaded"
    );

    Ok(FrontmostMenuSnapshot {
        app_name: tracked_app.name,
        bundle_id: tracked_app.bundle_id,
        items,
    })
}

/// Stub for non-macOS platforms — always returns an error.
#[cfg(not(target_os = "macos"))]
pub fn load_frontmost_menu_snapshot() -> anyhow::Result<FrontmostMenuSnapshot> {
    anyhow::bail!("Current App Commands requires macOS (Accessibility APIs unavailable)")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_bar::MenuBarItem;

    fn apple_menu() -> MenuBarItem {
        MenuBarItem {
            title: "Apple".into(),
            enabled: true,
            shortcut: None,
            children: vec![],
            ax_element_path: vec![0],
        }
    }

    fn leaf(title: &str, path: Vec<usize>) -> MenuBarItem {
        MenuBarItem {
            title: title.into(),
            enabled: true,
            shortcut: None,
            children: vec![],
            ax_element_path: path,
        }
    }

    fn menu(title: &str, children: Vec<MenuBarItem>, path: Vec<usize>) -> MenuBarItem {
        MenuBarItem {
            title: title.into(),
            enabled: true,
            shortcut: None,
            children,
            ax_element_path: path,
        }
    }

    #[test]
    fn into_entries_skips_apple_menu() {
        let snap = FrontmostMenuSnapshot {
            app_name: "TestApp".into(),
            bundle_id: "com.test.app".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf("New Tab", vec![1, 0])],
                    vec![1],
                ),
            ],
        };

        let entries = snap.into_entries();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].name.contains("New Tab"));
    }

    #[test]
    fn into_entries_empty_menu_returns_empty() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Empty".into(),
            bundle_id: "com.test.empty".into(),
            items: vec![],
        };
        assert!(snap.into_entries().is_empty());
    }

    #[test]
    fn placeholder_includes_app_name() {
        let snap = FrontmostMenuSnapshot {
            app_name: "Safari".into(),
            bundle_id: "com.apple.Safari".into(),
            items: vec![],
        };
        assert_eq!(snap.placeholder(), "Search Safari commands\u{2026}");
    }

    #[test]
    fn into_entries_with_receipt_reports_counts() {
        let snap = FrontmostMenuSnapshot {
            app_name: "TestApp".into(),
            bundle_id: "com.test.app".into(),
            items: vec![
                apple_menu(),
                menu(
                    "File",
                    vec![leaf("New Tab", vec![1, 0])],
                    vec![1],
                ),
            ],
        };

        let (entries, receipt) = snap.into_entries_with_receipt();

        assert_eq!(entries.len(), 1);
        assert_eq!(receipt.app_name, "TestApp");
        assert_eq!(receipt.bundle_id, "com.test.app");
        assert_eq!(receipt.top_level_menu_count, 2);
        assert_eq!(receipt.leaf_entry_count, 1);
        assert_eq!(receipt.placeholder, "Search TestApp commands\u{2026}");
        assert_eq!(receipt.source, "frontmost_app_tracker");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn loader_returns_snapshot_or_error() {
        // This test just ensures the loader doesn't panic.
        // It may return Ok or Err depending on whether the tracker is running.
        let _result = load_frontmost_menu_snapshot();
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn loader_returns_unsupported_error() {
        let err = load_frontmost_menu_snapshot().unwrap_err();
        assert!(
            err.to_string().contains("macOS"),
            "Expected macOS-specific error, got: {}",
            err
        );
    }
}
