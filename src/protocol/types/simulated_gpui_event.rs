use serde::{Deserialize, Serialize};

/// A high-fidelity input event intended for dispatch through GPUI's real
/// event pipeline, as opposed to the legacy `simulateKey` surface which
/// bypasses GPUI intercepts.
///
/// Used by the `simulateGpuiEvent` protocol command.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SimulatedGpuiEvent {
    /// Simulate a key-down event.
    KeyDown {
        key: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<crate::stdin_commands::KeyModifier>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    /// Simulate a mouse-move to window-relative coordinates.
    MouseMove { x: f64, y: f64 },
    /// Simulate a mouse button press.
    MouseDown {
        x: f64,
        y: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },
    /// Simulate a mouse button release.
    MouseUp {
        x: f64,
        y: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },
}
