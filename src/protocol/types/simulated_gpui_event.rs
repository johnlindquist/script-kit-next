use serde::{Deserialize, Serialize};

/// The lifecycle phase for a simulated touch-driven input event.
///
/// This mirrors GPUI's `TouchPhase` without exposing GPUI types in the wire
/// protocol.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SimulatedTouchPhase {
    Started,
    Moved,
    Ended,
}

/// Lossless lifecycle phase for direct and momentum scroll streams.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SimulatedScrollPhase {
    #[default]
    None,
    MayBegin,
    Began,
    Changed,
    Stationary,
    Ended,
    Cancelled,
}

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
    /// Simulate a complete mouse click in a single GPUI update.
    ///
    /// GPUI synthesizes `ClickEvent` from state shared between mouse down
    /// and mouse up, so this is the proof-grade primitive for `.on_click`
    /// handlers. Separate `mouseDown`/`mouseUp` RPCs can re-render between
    /// events and lose that pending state.
    MouseClick {
        x: f64,
        y: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        button: Option<String>,
    },
    /// Simulate a pixel-precise scroll-wheel event at window-relative
    /// coordinates.
    ScrollWheel {
        x: f64,
        y: f64,
        #[serde(rename = "deltaX")]
        delta_x: f64,
        #[serde(rename = "deltaY")]
        delta_y: f64,
        phase: SimulatedTouchPhase,
        #[serde(
            rename = "directPhase",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        direct_phase: Option<SimulatedScrollPhase>,
        #[serde(
            rename = "momentumPhase",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        momentum_phase: Option<SimulatedScrollPhase>,
        #[serde(
            rename = "timestampSeconds",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        timestamp_seconds: Option<f64>,
    },
}
