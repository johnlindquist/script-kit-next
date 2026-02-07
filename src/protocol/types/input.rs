use serde::{Deserialize, Serialize};

/// Mouse data for mouse actions
///
/// Contains coordinates and optional button for click events.
/// The `action` field in the Mouse message determines the semantics
/// (move, click, setPosition), so we use a single flat struct here
/// rather than an untagged enum (which would cause ambiguity since
/// move and setPosition have identical shapes).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MouseData {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Mouse button for click actions (e.g., "left", "right", "middle")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub button: Option<String>,
}

impl MouseData {
    /// Create new mouse data with coordinates
    pub fn new(x: f64, y: f64) -> Self {
        MouseData { x, y, button: None }
    }

    /// Create new mouse data with coordinates and button
    pub fn with_button(x: f64, y: f64, button: String) -> Self {
        MouseData {
            x,
            y,
            button: Some(button),
        }
    }

    /// Get coordinates as (x, y) tuple
    pub fn coordinates(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

/// Deprecated: Use MouseData instead
///
/// This enum had a bug where Move and SetPosition had identical shapes,
/// making SetPosition unreachable due to #[serde(untagged)].
/// Kept for backwards compatibility during transition.
#[deprecated(since = "0.2.0", note = "Use MouseData struct instead")]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MouseEventData {
    /// Move to position
    Move { x: f64, y: f64 },
    /// Click at position with optional button
    Click {
        x: f64,
        y: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },
    /// Set absolute position (unreachable due to untagged - use MouseData instead)
    SetPosition { x: f64, y: f64 },
}

#[allow(deprecated)]
impl MouseEventData {
    /// Get coordinates as (x, y) tuple
    pub fn coordinates(&self) -> (f64, f64) {
        match self {
            MouseEventData::Move { x, y } => (*x, *y),
            MouseEventData::Click { x, y, .. } => (*x, *y),
            MouseEventData::SetPosition { x, y } => (*x, *y),
        }
    }

    /// Convert to the new MouseData struct
    pub fn to_mouse_data(&self) -> MouseData {
        match self {
            MouseEventData::Move { x, y } => MouseData::new(*x, *y),
            MouseEventData::Click { x, y, button } => MouseData {
                x: *x,
                y: *y,
                button: button.clone(),
            },
            MouseEventData::SetPosition { x, y } => MouseData::new(*x, *y),
        }
    }
}

/// Exec command options
///
/// Options for the exec command including working directory, environment, and timeout.
/// Unknown fields are captured in `extra` for forward-compatibility.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExecOptions {
    /// Working directory for the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Environment variables (key-value pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Whether to capture stdout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_stdout: Option<bool>,
    /// Whether to capture stderr
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_stderr: Option<bool>,
    /// Forward-compatibility: captures unknown fields from newer SDK versions.
    /// This allows older app versions to preserve and pass through new options
    /// without losing them.
    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}
