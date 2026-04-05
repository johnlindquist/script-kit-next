//! Machine-readable ACP state types for agentic testing.
//!
//! Provides structured snapshots of ACP chat view state — input, cursor,
//! picker, accepted items, thread status, and layout stability metrics —
//! so that autonomous agents can verify ACP interactions without screenshots.

use serde::{Deserialize, Serialize};

/// Schema version for the ACP state response envelope.
pub const ACP_STATE_SCHEMA_VERSION: u32 = 1;

/// Top-level ACP state snapshot returned by `getAcpState`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AcpStateSnapshot {
    /// Schema version for forward compatibility.
    pub schema_version: u32,

    /// Current thread status: "idle", "streaming", "waitingForPermission", "error", "setup".
    pub status: String,

    /// Composer input text (redacted to length only when content logging is off).
    pub input_text: String,

    /// Cursor position as a character index (0-based).
    pub cursor_index: usize,

    /// Whether the composer has a text selection (not just a caret).
    pub has_selection: bool,

    /// Selection range as character indices `[start, end)`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_range: Option<[usize; 2]>,

    /// Number of messages in the thread history.
    pub message_count: usize,

    /// Picker state (None when picker is closed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picker: Option<AcpPickerState>,

    /// The most recently accepted picker item, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_accepted_item: Option<AcpAcceptedItem>,

    /// Context chip count (staged context parts in the composer).
    pub context_chip_count: usize,

    /// Whether the context bootstrap is still in progress.
    pub context_ready: bool,

    /// Pending permission request, if any.
    pub has_pending_permission: bool,

    /// Layout stability metrics for the single-line input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_layout: Option<AcpInputLayoutMetrics>,
}

impl Default for AcpStateSnapshot {
    fn default() -> Self {
        Self {
            schema_version: ACP_STATE_SCHEMA_VERSION,
            status: "idle".to_string(),
            input_text: String::new(),
            cursor_index: 0,
            has_selection: false,
            selection_range: None,
            message_count: 0,
            picker: None,
            last_accepted_item: None,
            context_chip_count: 0,
            context_ready: true,
            has_pending_permission: false,
            input_layout: None,
        }
    }
}

/// State of the inline mention/slash picker overlay.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AcpPickerState {
    /// Whether the picker is currently visible.
    pub open: bool,

    /// Trigger character that opened the picker: "@" or "/".
    pub trigger: String,

    /// Number of items in the picker.
    pub item_count: usize,

    /// Currently highlighted row index.
    pub selected_index: usize,

    /// Label of the currently highlighted item (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_label: Option<String>,
}

/// Metadata about the most recently accepted picker item.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AcpAcceptedItem {
    /// The item label that was accepted.
    pub label: String,

    /// The item ID from the picker.
    pub id: String,

    /// Trigger that was used: "@" or "/".
    pub trigger: String,

    /// Cursor index after the accepted text was inserted.
    pub cursor_after: usize,
}

/// Single-line input layout metrics for visual stability verification.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AcpInputLayoutMetrics {
    /// Total character count in the input.
    pub char_count: usize,

    /// Visible window start (character index).
    pub visible_start: usize,

    /// Visible window end (character index, exclusive).
    pub visible_end: usize,

    /// Cursor position within the visible window (0-based from visible_start).
    pub cursor_in_window: usize,
}

/// ACP-specific wait condition variants.
///
/// These extend the existing `WaitDetailedCondition` enum for ACP views.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum AcpWaitCondition {
    /// Wait until the ACP view is ready (context bootstrapped, status idle).
    AcpReady,

    /// Wait until the mention/slash picker is open.
    AcpPickerOpen,

    /// Wait until the mention/slash picker is closed.
    AcpPickerClosed,

    /// Wait until a picker item has been accepted.
    AcpItemAccepted,

    /// Wait until the cursor reaches a specific character index.
    AcpCursorAt {
        /// Target character index.
        index: usize,
    },

    /// Wait until the ACP thread reaches a specific status.
    AcpStatus {
        /// Target status: "idle", "streaming", "waitingForPermission", "error".
        status: String,
    },

    /// Wait until the ACP input text matches an exact value.
    AcpInputMatch {
        /// Expected input text.
        text: String,
    },

    /// Wait until the ACP input text contains a substring.
    AcpInputContains {
        /// Substring that must appear in the input.
        substring: String,
    },
}

/// Structured telemetry event for ACP key routing decisions.
///
/// Emitted via `tracing::info!` on the `script_kit::acp_telemetry` target.
/// Contains no user content — only indices, booleans, and route decisions.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AcpKeyRouteTelemetry {
    /// The key that was pressed (e.g., "enter", "tab", "up", "@").
    pub key: String,

    /// Where the key was routed to.
    pub route: AcpKeyRoute,

    /// Whether the picker was open at the time of the key press.
    pub picker_open: bool,

    /// Whether an ACP permission approval surface was active.
    pub permission_active: bool,

    /// Cursor index before the key was processed.
    pub cursor_before: usize,

    /// Cursor index after the key was processed.
    pub cursor_after: usize,

    /// Whether the key caused a submission.
    pub caused_submit: bool,

    /// Whether the key was consumed (stop_propagation called).
    pub consumed: bool,
}

/// Routing destination for an ACP key event.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcpKeyRoute {
    /// Key was handled by the picker (navigation or accept).
    Picker,
    /// Key was handled by the permission approval surface.
    Permission,
    /// Key was handled by the search overlay.
    Search,
    /// Key was handled by the composer input.
    Composer,
    /// Key was handled by a command shortcut (Cmd+K, Cmd+F, etc.).
    Command,
    /// Key was propagated to the parent (not consumed).
    Propagated,
    /// Key was handled by the setup mode handler.
    Setup,
}

/// Structured telemetry event emitted when a picker item is accepted.
///
/// Emitted on `script_kit::acp_telemetry` target alongside the key-route event.
/// Preserves the trigger, item identity, and cursor position so agents can
/// verify acceptance without fuzzy log matching.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AcpPickerItemAcceptedTelemetry {
    /// Trigger character that opened the picker: `@` or `/`.
    pub trigger: String,

    /// Human-readable label of the accepted item.
    pub item_label: String,

    /// Machine ID of the accepted item (e.g. `built_in:context`).
    pub item_id: String,

    /// The key that caused the accept: `"enter"` or `"tab"`.
    pub accepted_via_key: String,

    /// Cursor character index after the accepted text was inserted.
    pub cursor_after: usize,

    /// Whether the accept also caused a message submission.
    pub caused_submit: bool,
}

/// Schema version for the ACP test probe response envelope.
pub const ACP_TEST_PROBE_SCHEMA_VERSION: u32 = 1;

/// Maximum number of events stored per category in the test probe ring buffer.
pub const ACP_TEST_PROBE_MAX_EVENTS: usize = 32;

/// Top-level ACP test probe snapshot returned by `getAcpTestProbe`.
///
/// Contains a bounded tail of recent key-route, picker-acceptance, and
/// input-layout telemetry events so agents can verify native ACP
/// interactions without grepping logs or inferring from screenshots.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AcpTestProbeSnapshot {
    /// Schema version for forward compatibility.
    pub schema_version: u32,

    /// Monotonically increasing event sequence counter.
    pub event_seq: u64,

    /// Recent key-route telemetry events (bounded ring buffer tail).
    pub key_routes: Vec<AcpKeyRouteTelemetry>,

    /// Recent picker-acceptance telemetry events (bounded ring buffer tail).
    pub accepted_items: Vec<AcpPickerItemAcceptedTelemetry>,

    /// Most recent input-layout telemetry snapshot, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_layout: Option<AcpInputLayoutTelemetry>,

    /// Current ACP state snapshot at the time the probe was queried.
    pub state: AcpStateSnapshot,
}

impl Default for AcpTestProbeSnapshot {
    fn default() -> Self {
        Self {
            schema_version: ACP_TEST_PROBE_SCHEMA_VERSION,
            event_seq: 0,
            key_routes: Vec::new(),
            accepted_items: Vec::new(),
            input_layout: None,
            state: AcpStateSnapshot::default(),
        }
    }
}

/// Structured telemetry event for single-line input layout after mutations.
///
/// Emitted on `script_kit::acp_telemetry` target after acceptance or cursor
/// moves that may shift the visible window.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AcpInputLayoutTelemetry {
    /// Total character count in the input.
    pub char_count: usize,

    /// Visible window start (character index).
    pub visible_start: usize,

    /// Visible window end (character index, exclusive).
    pub visible_end: usize,

    /// Cursor position within the visible window (0-based from visible_start).
    pub cursor_in_window: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── AcpStateSnapshot serde ──────────────────────────────────

    #[test]
    fn acp_state_snapshot_default_round_trips() {
        let snap = AcpStateSnapshot::default();
        let json = serde_json::to_value(&snap).expect("serialize default snapshot");
        assert_eq!(json["schemaVersion"], ACP_STATE_SCHEMA_VERSION);
        assert_eq!(json["status"], "idle");
        assert_eq!(json["cursorIndex"], 0);
        assert!(!json["hasSelection"].as_bool().unwrap_or(true));
        assert!(json["picker"].is_null());
        assert!(json["lastAcceptedItem"].is_null());

        let back: AcpStateSnapshot =
            serde_json::from_value(json).expect("deserialize default snapshot");
        assert_eq!(back, snap);
    }

    #[test]
    fn acp_state_snapshot_with_picker_round_trips() {
        let snap = AcpStateSnapshot {
            picker: Some(AcpPickerState {
                open: true,
                trigger: "@".to_string(),
                item_count: 5,
                selected_index: 2,
                selected_label: Some("file.rs".to_string()),
            }),
            ..Default::default()
        };
        let json = serde_json::to_value(&snap).expect("serialize with picker");
        assert_eq!(json["picker"]["open"], true);
        assert_eq!(json["picker"]["trigger"], "@");
        assert_eq!(json["picker"]["itemCount"], 5);
        assert_eq!(json["picker"]["selectedIndex"], 2);
        assert_eq!(json["picker"]["selectedLabel"], "file.rs");

        let back: AcpStateSnapshot = serde_json::from_value(json).expect("deserialize with picker");
        assert_eq!(back, snap);
    }

    #[test]
    fn acp_state_snapshot_with_accepted_item_round_trips() {
        let snap = AcpStateSnapshot {
            last_accepted_item: Some(AcpAcceptedItem {
                label: "context".to_string(),
                id: "built_in:context".to_string(),
                trigger: "@".to_string(),
                cursor_after: 9,
            }),
            ..Default::default()
        };
        let json = serde_json::to_value(&snap).expect("serialize with accepted item");
        assert_eq!(json["lastAcceptedItem"]["label"], "context");
        assert_eq!(json["lastAcceptedItem"]["cursorAfter"], 9);

        let back: AcpStateSnapshot =
            serde_json::from_value(json).expect("deserialize with accepted item");
        assert_eq!(back, snap);
    }

    #[test]
    fn acp_state_snapshot_with_layout_metrics_round_trips() {
        let snap = AcpStateSnapshot {
            input_layout: Some(AcpInputLayoutMetrics {
                char_count: 42,
                visible_start: 10,
                visible_end: 35,
                cursor_in_window: 5,
            }),
            ..Default::default()
        };
        let json = serde_json::to_value(&snap).expect("serialize with layout");
        assert_eq!(json["inputLayout"]["charCount"], 42);
        assert_eq!(json["inputLayout"]["visibleStart"], 10);

        let back: AcpStateSnapshot = serde_json::from_value(json).expect("deserialize with layout");
        assert_eq!(back, snap);
    }

    // ── AcpWaitCondition serde ──────────────────────────────────

    #[test]
    fn acp_wait_condition_ready_round_trips() {
        let cond = AcpWaitCondition::AcpReady;
        let json = serde_json::to_value(&cond).expect("serialize");
        assert_eq!(json["type"], "acpReady");

        let back: AcpWaitCondition = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cond);
    }

    #[test]
    fn acp_wait_condition_picker_open_round_trips() {
        let cond = AcpWaitCondition::AcpPickerOpen;
        let json = serde_json::to_value(&cond).expect("serialize");
        assert_eq!(json["type"], "acpPickerOpen");

        let back: AcpWaitCondition = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cond);
    }

    #[test]
    fn acp_wait_condition_picker_closed_round_trips() {
        let cond = AcpWaitCondition::AcpPickerClosed;
        let json = serde_json::to_value(&cond).expect("serialize");
        assert_eq!(json["type"], "acpPickerClosed");

        let back: AcpWaitCondition = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cond);
    }

    #[test]
    fn acp_wait_condition_item_accepted_round_trips() {
        let cond = AcpWaitCondition::AcpItemAccepted;
        let json = serde_json::to_value(&cond).expect("serialize");
        assert_eq!(json["type"], "acpItemAccepted");

        let back: AcpWaitCondition = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cond);
    }

    #[test]
    fn acp_wait_condition_cursor_at_round_trips() {
        let cond = AcpWaitCondition::AcpCursorAt { index: 15 };
        let json = serde_json::to_value(&cond).expect("serialize");
        assert_eq!(json["type"], "acpCursorAt");
        assert_eq!(json["index"], 15);

        let back: AcpWaitCondition = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cond);
    }

    #[test]
    fn acp_wait_condition_status_round_trips() {
        let cond = AcpWaitCondition::AcpStatus {
            status: "streaming".to_string(),
        };
        let json = serde_json::to_value(&cond).expect("serialize");
        assert_eq!(json["type"], "acpStatus");
        assert_eq!(json["status"], "streaming");

        let back: AcpWaitCondition = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cond);
    }

    #[test]
    fn acp_wait_condition_input_match_round_trips() {
        let cond = AcpWaitCondition::AcpInputMatch {
            text: "@context ".to_string(),
        };
        let json = serde_json::to_value(&cond).expect("serialize");
        assert_eq!(json["type"], "acpInputMatch");
        assert_eq!(json["text"], "@context ");

        let back: AcpWaitCondition = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cond);
    }

    #[test]
    fn acp_wait_condition_input_contains_round_trips() {
        let cond = AcpWaitCondition::AcpInputContains {
            substring: "hello".to_string(),
        };
        let json = serde_json::to_value(&cond).expect("serialize");
        assert_eq!(json["type"], "acpInputContains");
        assert_eq!(json["substring"], "hello");

        let back: AcpWaitCondition = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, cond);
    }

    // ─�� AcpKeyRouteTelemetry serde ��─────────────────────────────

    #[test]
    fn acp_key_route_telemetry_round_trips() {
        let telemetry = AcpKeyRouteTelemetry {
            key: "enter".to_string(),
            route: AcpKeyRoute::Picker,
            picker_open: true,
            permission_active: false,
            cursor_before: 5,
            cursor_after: 9,
            caused_submit: false,
            consumed: true,
        };
        let json = serde_json::to_value(&telemetry).expect("serialize telemetry");
        assert_eq!(json["key"], "enter");
        assert_eq!(json["route"], "picker");
        assert!(json["pickerOpen"].as_bool().unwrap_or(false));
        assert_eq!(json["cursorBefore"], 5);
        assert_eq!(json["cursorAfter"], 9);

        let back: AcpKeyRouteTelemetry =
            serde_json::from_value(json).expect("deserialize telemetry");
        assert_eq!(back, telemetry);
    }

    #[test]
    fn acp_key_route_variants_serialize_correctly() {
        let routes = vec![
            (AcpKeyRoute::Picker, "picker"),
            (AcpKeyRoute::Permission, "permission"),
            (AcpKeyRoute::Search, "search"),
            (AcpKeyRoute::Composer, "composer"),
            (AcpKeyRoute::Command, "command"),
            (AcpKeyRoute::Propagated, "propagated"),
            (AcpKeyRoute::Setup, "setup"),
        ];
        for (route, expected) in routes {
            let json = serde_json::to_value(&route).expect("serialize route");
            assert_eq!(json.as_str().unwrap_or(""), expected);
        }
    }

    // ── AcpPickerState serde ────────────────────────────────────

    #[test]
    fn acp_picker_state_round_trips() {
        let state = AcpPickerState {
            open: true,
            trigger: "/".to_string(),
            item_count: 3,
            selected_index: 1,
            selected_label: Some("context-full".to_string()),
        };
        let json = serde_json::to_value(&state).expect("serialize picker state");
        assert_eq!(json["trigger"], "/");
        assert_eq!(json["itemCount"], 3);

        let back: AcpPickerState = serde_json::from_value(json).expect("deserialize picker state");
        assert_eq!(back, state);
    }

    // ── Full snapshot JSON shape ────────────────────────────────

    #[test]
    fn acp_state_snapshot_full_json_shape() {
        let snap = AcpStateSnapshot {
            schema_version: ACP_STATE_SCHEMA_VERSION,
            status: "streaming".to_string(),
            input_text: "hello @context".to_string(),
            cursor_index: 14,
            has_selection: false,
            selection_range: None,
            message_count: 3,
            picker: None,
            last_accepted_item: Some(AcpAcceptedItem {
                label: "context".to_string(),
                id: "built_in:context".to_string(),
                trigger: "@".to_string(),
                cursor_after: 14,
            }),
            context_chip_count: 1,
            context_ready: true,
            has_pending_permission: false,
            input_layout: Some(AcpInputLayoutMetrics {
                char_count: 14,
                visible_start: 0,
                visible_end: 14,
                cursor_in_window: 14,
            }),
        };
        let json = serde_json::to_string_pretty(&snap).expect("serialize full snapshot");
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("parse full snapshot JSON");

        // Verify top-level keys are present
        assert!(parsed["schemaVersion"].is_number());
        assert!(parsed["status"].is_string());
        assert!(parsed["inputText"].is_string());
        assert!(parsed["cursorIndex"].is_number());
        assert!(parsed["hasSelection"].is_boolean());
        assert!(parsed["messageCount"].is_number());
        assert!(parsed["lastAcceptedItem"].is_object());
        assert!(parsed["contextChipCount"].is_number());
        assert!(parsed["contextReady"].is_boolean());
        assert!(parsed["hasPendingPermission"].is_boolean());
        assert!(parsed["inputLayout"].is_object());
    }

    // ── Deserialization from external JSON ──────────────────────

    #[test]
    fn acp_state_snapshot_deserializes_from_minimal_json() {
        let json = serde_json::json!({
            "schemaVersion": 1,
            "status": "idle",
            "inputText": "",
            "cursorIndex": 0,
            "hasSelection": false,
            "messageCount": 0,
            "contextChipCount": 0,
            "contextReady": true,
            "hasPendingPermission": false,
        });
        let snap: AcpStateSnapshot =
            serde_json::from_value(json).expect("deserialize minimal JSON");
        assert_eq!(snap.status, "idle");
        assert!(snap.picker.is_none());
        assert!(snap.last_accepted_item.is_none());
        assert!(snap.input_layout.is_none());
    }

    #[test]
    fn acp_wait_condition_deserializes_from_external_json() {
        let json = serde_json::json!({
            "type": "acpCursorAt",
            "index": 42,
        });
        let cond: AcpWaitCondition =
            serde_json::from_value(json).expect("deserialize external JSON");
        assert_eq!(cond, AcpWaitCondition::AcpCursorAt { index: 42 });
    }

    // ── AcpPickerItemAcceptedTelemetry serde ───────────────────

    #[test]
    fn acp_picker_item_accepted_telemetry_round_trips() {
        let telemetry = AcpPickerItemAcceptedTelemetry {
            trigger: "@".to_string(),
            item_label: "context".to_string(),
            item_id: "built_in:context".to_string(),
            accepted_via_key: "enter".to_string(),
            cursor_after: 9,
            caused_submit: false,
        };
        let json = serde_json::to_value(&telemetry).expect("serialize");
        assert_eq!(json["trigger"], "@");
        assert_eq!(json["itemLabel"], "context");
        assert_eq!(json["itemId"], "built_in:context");
        assert_eq!(json["acceptedViaKey"], "enter");
        assert_eq!(json["cursorAfter"], 9);
        assert_eq!(json["causedSubmit"], false);

        let back: AcpPickerItemAcceptedTelemetry =
            serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, telemetry);
    }

    #[test]
    fn acp_picker_item_accepted_telemetry_tab_vs_enter_distinct() {
        let enter = AcpPickerItemAcceptedTelemetry {
            trigger: "@".to_string(),
            item_label: "context".to_string(),
            item_id: "built_in:context".to_string(),
            accepted_via_key: "enter".to_string(),
            cursor_after: 9,
            caused_submit: false,
        };
        let tab = AcpPickerItemAcceptedTelemetry {
            accepted_via_key: "tab".to_string(),
            ..enter.clone()
        };
        let enter_json = serde_json::to_value(&enter).expect("serialize enter");
        let tab_json = serde_json::to_value(&tab).expect("serialize tab");

        assert_eq!(enter_json["acceptedViaKey"], "enter");
        assert_eq!(tab_json["acceptedViaKey"], "tab");
        assert_ne!(
            enter_json["acceptedViaKey"], tab_json["acceptedViaKey"],
            "enter and tab must produce distinct acceptedViaKey values"
        );
    }

    #[test]
    fn acp_picker_item_accepted_telemetry_slash_trigger() {
        let telemetry = AcpPickerItemAcceptedTelemetry {
            trigger: "/".to_string(),
            item_label: "compact".to_string(),
            item_id: "slash:compact".to_string(),
            accepted_via_key: "enter".to_string(),
            cursor_after: 9,
            caused_submit: true,
        };
        let json = serde_json::to_value(&telemetry).expect("serialize");
        assert_eq!(json["trigger"], "/");
        assert_eq!(json["causedSubmit"], true);

        let back: AcpPickerItemAcceptedTelemetry =
            serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, telemetry);
    }

    // ── AcpInputLayoutTelemetry serde ──────────────────────────

    #[test]
    fn acp_input_layout_telemetry_round_trips() {
        let telemetry = AcpInputLayoutTelemetry {
            char_count: 42,
            visible_start: 10,
            visible_end: 35,
            cursor_in_window: 5,
        };
        let json = serde_json::to_value(&telemetry).expect("serialize");
        assert_eq!(json["charCount"], 42);
        assert_eq!(json["visibleStart"], 10);
        assert_eq!(json["visibleEnd"], 35);
        assert_eq!(json["cursorInWindow"], 5);

        let back: AcpInputLayoutTelemetry = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, telemetry);
    }

    #[test]
    fn acp_input_layout_telemetry_matches_layout_metrics_fields() {
        // Verify the telemetry mirrors AcpInputLayoutMetrics field names exactly.
        let metrics = AcpInputLayoutMetrics {
            char_count: 24,
            visible_start: 0,
            visible_end: 24,
            cursor_in_window: 11,
        };
        let telemetry = AcpInputLayoutTelemetry {
            char_count: metrics.char_count,
            visible_start: metrics.visible_start,
            visible_end: metrics.visible_end,
            cursor_in_window: metrics.cursor_in_window,
        };
        let m_json = serde_json::to_value(&metrics).expect("serialize metrics");
        let t_json = serde_json::to_value(&telemetry).expect("serialize telemetry");
        assert_eq!(
            m_json, t_json,
            "telemetry and metrics must serialize to identical JSON"
        );
    }

    // ── Key route telemetry with picker-accepted context ───────

    #[test]
    fn acp_key_route_telemetry_enter_picker_accept() {
        let route_event = AcpKeyRouteTelemetry {
            key: "enter".to_string(),
            route: AcpKeyRoute::Picker,
            picker_open: true,
            permission_active: false,
            cursor_before: 1,
            cursor_after: 9,
            caused_submit: false,
            consumed: true,
        };
        let json = serde_json::to_value(&route_event).expect("serialize");
        assert_eq!(json["key"], "enter");
        assert_eq!(json["route"], "picker");
        assert!(json["pickerOpen"].as_bool().expect("bool"));
    }

    #[test]
    fn acp_key_route_telemetry_tab_picker_accept() {
        let route_event = AcpKeyRouteTelemetry {
            key: "tab".to_string(),
            route: AcpKeyRoute::Picker,
            picker_open: true,
            permission_active: false,
            cursor_before: 1,
            cursor_after: 9,
            caused_submit: false,
            consumed: true,
        };
        let json = serde_json::to_value(&route_event).expect("serialize");
        assert_eq!(json["key"], "tab");
        assert_eq!(json["route"], "picker");
    }

    #[test]
    fn acp_key_route_enter_vs_tab_distinct_key_field() {
        let enter = AcpKeyRouteTelemetry {
            key: "enter".to_string(),
            route: AcpKeyRoute::Picker,
            picker_open: true,
            permission_active: false,
            cursor_before: 1,
            cursor_after: 9,
            caused_submit: false,
            consumed: true,
        };
        let tab = AcpKeyRouteTelemetry {
            key: "tab".to_string(),
            ..enter.clone()
        };
        let e_json = serde_json::to_value(&enter).expect("serialize enter");
        let t_json = serde_json::to_value(&tab).expect("serialize tab");
        assert_ne!(
            e_json["key"], t_json["key"],
            "enter and tab key-route events must have distinct key fields"
        );
    }

    // ── AcpTestProbeSnapshot serde ─────────────────────────────

    #[test]
    fn acp_test_probe_snapshot_default_round_trips() {
        let snap = AcpTestProbeSnapshot::default();
        let json = serde_json::to_value(&snap).expect("serialize default probe");
        assert_eq!(json["schemaVersion"], ACP_TEST_PROBE_SCHEMA_VERSION);
        assert_eq!(json["eventSeq"], 0);
        assert!(json["keyRoutes"].as_array().expect("array").is_empty());
        assert!(json["acceptedItems"].as_array().expect("array").is_empty());
        assert!(json.get("inputLayout").is_none());

        let back: AcpTestProbeSnapshot =
            serde_json::from_value(json).expect("deserialize default probe");
        assert_eq!(back, snap);
    }

    #[test]
    fn acp_test_probe_snapshot_with_events_round_trips() {
        let snap = AcpTestProbeSnapshot {
            schema_version: ACP_TEST_PROBE_SCHEMA_VERSION,
            event_seq: 14,
            key_routes: vec![AcpKeyRouteTelemetry {
                key: "tab".to_string(),
                route: AcpKeyRoute::Picker,
                picker_open: true,
                permission_active: false,
                cursor_before: 1,
                cursor_after: 17,
                caused_submit: false,
                consumed: true,
            }],
            accepted_items: vec![AcpPickerItemAcceptedTelemetry {
                trigger: "@".to_string(),
                item_label: "Current Context".to_string(),
                item_id: "built_in:context".to_string(),
                accepted_via_key: "tab".to_string(),
                cursor_after: 17,
                caused_submit: false,
            }],
            input_layout: Some(AcpInputLayoutTelemetry {
                char_count: 27,
                visible_start: 0,
                visible_end: 27,
                cursor_in_window: 17,
            }),
            state: AcpStateSnapshot {
                status: "idle".to_string(),
                cursor_index: 17,
                ..Default::default()
            },
        };
        let json = serde_json::to_value(&snap).expect("serialize probe with events");
        assert_eq!(json["eventSeq"], 14);
        assert_eq!(json["keyRoutes"][0]["key"], "tab");
        assert_eq!(json["acceptedItems"][0]["acceptedViaKey"], "tab");
        assert_eq!(json["acceptedItems"][0]["cursorAfter"], 17);
        assert_eq!(json["inputLayout"]["cursorInWindow"], 17);
        assert_eq!(json["state"]["cursorIndex"], 17);

        let back: AcpTestProbeSnapshot =
            serde_json::from_value(json).expect("deserialize probe with events");
        assert_eq!(back, snap);
    }

    #[test]
    fn acp_test_probe_snapshot_schema_version_constant() {
        assert_eq!(ACP_TEST_PROBE_SCHEMA_VERSION, 1);
    }

    #[test]
    fn acp_test_probe_max_events_constant() {
        assert_eq!(ACP_TEST_PROBE_MAX_EVENTS, 32);
    }
}
