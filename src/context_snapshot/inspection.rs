use super::AiContextSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextSnapshotInspectionReceipt {
    pub schema_version: u32,
    pub warning_count: usize,
    pub has_selected_text: bool,
    pub has_frontmost_app: bool,
    pub top_level_menu_count: usize,
    pub has_browser: bool,
    pub has_focused_window: bool,
    pub json_bytes: usize,
    pub status: String,
}

pub fn build_inspection_receipt(
    snapshot: &AiContextSnapshot,
    json_bytes: usize,
) -> ContextSnapshotInspectionReceipt {
    let warning_count = snapshot.warnings.len();

    ContextSnapshotInspectionReceipt {
        schema_version: snapshot.schema_version,
        warning_count,
        has_selected_text: snapshot
            .selected_text
            .as_ref()
            .is_some_and(|text| !text.trim().is_empty()),
        has_frontmost_app: snapshot.frontmost_app.is_some(),
        top_level_menu_count: snapshot.menu_bar_items.len(),
        has_browser: snapshot.browser.is_some(),
        has_focused_window: snapshot.focused_window.is_some(),
        json_bytes,
        status: if warning_count == 0 {
            "ok".to_string()
        } else {
            "partial".to_string()
        },
    }
}

pub fn build_inspection_hud_message(receipt: &ContextSnapshotInspectionReceipt) -> String {
    if receipt.warning_count == 0 {
        "Copied current context snapshot".to_string()
    } else {
        format!(
            "Copied current context snapshot ({} warning{})",
            receipt.warning_count,
            if receipt.warning_count == 1 { "" } else { "s" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_snapshot::{
        AiContextSnapshot, BrowserContext, FocusedWindowContext, FrontmostAppContext,
        MenuBarItemSummary, AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
    };

    #[test]
    fn build_inspection_receipt_reports_context_shape() {
        let snapshot = AiContextSnapshot {
            schema_version: AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
            selected_text: Some("selected".to_string()),
            frontmost_app: Some(FrontmostAppContext {
                pid: 42,
                bundle_id: "com.apple.Safari".to_string(),
                name: "Safari".to_string(),
            }),
            menu_bar_items: vec![MenuBarItemSummary {
                title: "File".to_string(),
                enabled: true,
                shortcut: None,
                children: Vec::new(),
            }],
            browser: Some(BrowserContext {
                url: "https://example.com".to_string(),
            }),
            focused_window: Some(FocusedWindowContext {
                title: "Example".to_string(),
                width: 1440,
                height: 900,
                used_fallback: false,
            }),
            warnings: Vec::new(),
        };

        let receipt = build_inspection_receipt(&snapshot, 512);

        assert_eq!(receipt.schema_version, AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(receipt.warning_count, 0);
        assert!(receipt.has_selected_text);
        assert!(receipt.has_frontmost_app);
        assert_eq!(receipt.top_level_menu_count, 1);
        assert!(receipt.has_browser);
        assert!(receipt.has_focused_window);
        assert_eq!(receipt.json_bytes, 512);
        assert_eq!(receipt.status, "ok");
    }

    #[test]
    fn build_inspection_receipt_partial_when_warnings_present() {
        let snapshot = AiContextSnapshot {
            warnings: vec!["selected_text: no accessibility permission".to_string()],
            ..AiContextSnapshot::default()
        };

        let receipt = build_inspection_receipt(&snapshot, 64);
        assert_eq!(receipt.warning_count, 1);
        assert_eq!(receipt.status, "partial");
    }

    #[test]
    fn build_inspection_hud_message_clean() {
        let receipt = ContextSnapshotInspectionReceipt {
            schema_version: AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
            warning_count: 0,
            has_selected_text: false,
            has_frontmost_app: true,
            top_level_menu_count: 0,
            has_browser: false,
            has_focused_window: true,
            json_bytes: 128,
            status: "ok".to_string(),
        };

        assert_eq!(
            build_inspection_hud_message(&receipt),
            "Copied current context snapshot"
        );
    }

    #[test]
    fn build_inspection_hud_message_mentions_warning_count() {
        let receipt = ContextSnapshotInspectionReceipt {
            schema_version: AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
            warning_count: 2,
            has_selected_text: false,
            has_frontmost_app: true,
            top_level_menu_count: 0,
            has_browser: false,
            has_focused_window: true,
            json_bytes: 128,
            status: "partial".to_string(),
        };

        assert_eq!(
            build_inspection_hud_message(&receipt),
            "Copied current context snapshot (2 warnings)"
        );
    }

    #[test]
    fn build_inspection_hud_message_singular_warning() {
        let receipt = ContextSnapshotInspectionReceipt {
            schema_version: AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
            warning_count: 1,
            has_selected_text: false,
            has_frontmost_app: false,
            top_level_menu_count: 0,
            has_browser: false,
            has_focused_window: false,
            json_bytes: 32,
            status: "partial".to_string(),
        };

        assert_eq!(
            build_inspection_hud_message(&receipt),
            "Copied current context snapshot (1 warning)"
        );
    }
}
