//! Types for the `inspectAutomationWindow` protocol command.
//!
//! Returns a compact, machine-readable snapshot of one exact automation
//! window: resolved target identity, screenshot dimensions, optional
//! pixel probe results, and semantic elements when available.

use serde::{Deserialize, Serialize};

/// Current schema version for automation inspect snapshots.
pub const AUTOMATION_INSPECT_SCHEMA_VERSION: u32 = 1;

/// A pixel coordinate to sample from the captured screenshot.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelProbe {
    pub x: u32,
    pub y: u32,
}

/// RGBA color at a sampled pixel coordinate.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelProbeResult {
    pub x: u32,
    pub y: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Compact proof-oriented snapshot of one exact automation window.
///
/// Bundles resolved target identity, screenshot dimensions, optional
/// pixel probes, and semantic elements into a single response so agents
/// do not need separate `getElements` + `captureScreenshot` round-trips.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationInspectSnapshot {
    /// Schema version for forward compatibility.
    pub schema_version: u32,

    /// Resolved automation window ID (e.g. `"main:0"`, `"acpDetached:thread-2"`).
    pub window_id: String,

    /// Automation window kind as a string (e.g. `"Main"`, `"Notes"`, `"AcpDetached"`).
    pub window_kind: String,

    /// Window title if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Semantic UI elements (empty when collection is unavailable for this window kind).
    #[serde(default)]
    pub elements: Vec<super::ElementInfo>,

    /// Total element count before any limit was applied.
    #[serde(default)]
    pub total_count: usize,

    /// Semantic ID of the currently focused element, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_semantic_id: Option<String>,

    /// Semantic ID of the currently selected element, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_semantic_id: Option<String>,

    /// Width of the captured screenshot in pixels (None if capture failed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_width: Option<u32>,

    /// Height of the captured screenshot in pixels (None if capture failed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_height: Option<u32>,

    /// RGBA values at requested pixel coordinates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pixel_probes: Vec<PixelProbeResult>,

    /// Native OS window ID (CGWindowID on macOS) for strict screenshot capture.
    ///
    /// Present when the inspect handler successfully matched an OS window.
    /// Agents use this for `--capture-window-id` in verify-shot flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_window_id: Option<u32>,

    /// Machine-readable warnings (e.g. `"semantic_elements_non_main_pending"`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::elements_actions_scriptlets::{ElementInfo, ElementType};

    #[test]
    fn snapshot_serde_roundtrip_minimal() {
        let snapshot = AutomationInspectSnapshot {
            schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
            window_id: "main:0".to_string(),
            window_kind: "Main".to_string(),
            title: None,
            elements: Vec::new(),
            total_count: 0,
            focused_semantic_id: None,
            selected_semantic_id: None,
            screenshot_width: None,
            screenshot_height: None,
            pixel_probes: Vec::new(),
            os_window_id: None,
            warnings: Vec::new(),
        };

        let json = serde_json::to_string(&snapshot).expect("serialize");
        let parsed: AutomationInspectSnapshot =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, snapshot);
        // Verify camelCase
        assert!(json.contains("schemaVersion"));
        assert!(json.contains("windowId"));
        assert!(json.contains("windowKind"));
    }

    #[test]
    fn snapshot_serde_roundtrip_full() {
        let snapshot = AutomationInspectSnapshot {
            schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
            window_id: "notes:0".to_string(),
            window_kind: "Notes".to_string(),
            title: Some("Script Kit Notes".to_string()),
            elements: vec![ElementInfo {
                semantic_id: "panel:notes".to_string(),
                element_type: ElementType::Panel,
                text: None,
                value: None,
                selected: None,
                focused: None,
                index: None,
            }],
            total_count: 1,
            focused_semantic_id: Some("panel:notes".to_string()),
            selected_semantic_id: None,
            screenshot_width: Some(1440),
            screenshot_height: Some(900),
            pixel_probes: vec![
                PixelProbeResult {
                    x: 24,
                    y: 24,
                    r: 28,
                    g: 28,
                    b: 30,
                    a: 255,
                },
            ],
            os_window_id: Some(12345),
            warnings: vec!["semantic_elements_non_main_pending".to_string()],
        };

        let json = serde_json::to_string(&snapshot).expect("serialize");
        let parsed: AutomationInspectSnapshot =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, snapshot);
        assert!(json.contains("\"osWindowId\":12345"));
    }

    #[test]
    fn pixel_probe_serde() {
        let probe = PixelProbe { x: 100, y: 200 };
        let json = serde_json::to_string(&probe).expect("serialize");
        assert!(json.contains("\"x\":100"));
        let parsed: PixelProbe = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, probe);
    }

    #[test]
    fn pixel_probe_result_serde() {
        let result = PixelProbeResult {
            x: 10,
            y: 20,
            r: 255,
            g: 128,
            b: 0,
            a: 255,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let parsed: PixelProbeResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, result);
    }

    #[test]
    fn empty_collections_skipped_in_json() {
        let snapshot = AutomationInspectSnapshot {
            schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
            window_id: "main:0".to_string(),
            window_kind: "Main".to_string(),
            title: None,
            elements: Vec::new(),
            total_count: 0,
            focused_semantic_id: None,
            selected_semantic_id: None,
            screenshot_width: None,
            screenshot_height: None,
            pixel_probes: Vec::new(),
            os_window_id: None,
            warnings: Vec::new(),
        };
        let json = serde_json::to_string(&snapshot).expect("serialize");
        // Empty vecs with skip_serializing_if should not appear
        assert!(!json.contains("pixelProbes"));
        assert!(!json.contains("warnings"));
        // None options should not appear
        assert!(!json.contains("title"));
        assert!(!json.contains("screenshotWidth"));
        assert!(!json.contains("osWindowId"));
    }
}
