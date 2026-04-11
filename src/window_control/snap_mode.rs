use anyhow::Context as _;
use std::sync::LazyLock;

use parking_lot::Mutex;

/// Snap mode controls how many tile positions are available during drag snapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SnapMode {
    /// Snap is disabled; runtime returns early.
    Off,
    /// Halves, quadrants, center, almost-maximize.
    Simple,
    /// Simple + horizontal/vertical thirds + two-thirds.
    Expanded,
    /// Expanded + sixths.
    Precision,
}

static SNAP_MODE: LazyLock<Mutex<SnapMode>> = LazyLock::new(|| Mutex::new(SnapMode::Expanded));

/// Read the current snap mode.
pub fn current_snap_mode() -> SnapMode {
    *SNAP_MODE.lock()
}

/// Change the snap mode. Returns the newly set mode.
pub fn set_snap_mode(mode: SnapMode) -> SnapMode {
    *SNAP_MODE.lock() = mode;
    tracing::info!(
        target: "script_kit::snap_mode",
        event = "snap_mode_changed",
        ?mode,
        "snap mode changed"
    );
    mode
}

/// Load snap mode from config-backed preferences and update the in-memory state.
pub fn load_snap_mode_from_preferences() -> anyhow::Result<SnapMode> {
    let prefs = crate::config::load_user_preferences();
    let mode = prefs
        .window_management
        .snap_mode
        .unwrap_or(SnapMode::Expanded);

    *SNAP_MODE.lock() = mode;

    tracing::info!(
        target: "script_kit::snap_mode",
        event = "snap_mode_loaded_from_preferences",
        ?mode,
        "loaded snap mode from config-backed preferences"
    );

    Ok(mode)
}

/// Persist snap mode to config-backed preferences, then update the in-memory state.
pub fn persist_snap_mode(mode: SnapMode) -> anyhow::Result<SnapMode> {
    let mut prefs = crate::config::load_user_preferences();
    prefs.window_management.snap_mode = Some(mode);

    crate::config::save_user_preferences(&prefs)
        .context("failed to save snap mode to config.ts")?;

    let mode = set_snap_mode(mode);

    tracing::info!(
        target: "script_kit::snap_mode",
        event = "snap_mode_persisted",
        ?mode,
        "persisted snap mode to config-backed preferences"
    );

    Ok(mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_snap_mode_is_expanded() {
        let mode = *SNAP_MODE.lock();
        assert_eq!(mode, SnapMode::Expanded);
    }

    #[test]
    fn set_and_read_snap_mode() {
        let mut guard = SNAP_MODE.lock();
        let previous = *guard;

        *guard = SnapMode::Precision;
        assert_eq!(*guard, SnapMode::Precision);

        *guard = SnapMode::Off;
        assert_eq!(*guard, SnapMode::Off);

        *guard = previous;
    }

    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&SnapMode::Expanded).expect("serialize");
        assert_eq!(json, "\"expanded\"");
        let mode: SnapMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(mode, SnapMode::Expanded);
    }
}
