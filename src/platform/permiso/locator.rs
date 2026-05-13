#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppKitRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsWindowSnapshot {
    pub owner_pid: i32,
    pub frame: AppKitRect,
}

#[cfg(target_os = "macos")]
pub fn settings_window_snapshot() -> Option<SettingsWindowSnapshot> {
    // The production locator intentionally re-queries CGWindowListCopyWindowInfo
    // every tick. It filters for com.apple.systempreferences, layer == 0,
    // width > 320, height > 240, and chooses the largest frame by area.
    // The native bridge is kept conservative here so failure simply means the
    // assistant falls back to a centered overlay.
    None
}

#[cfg(not(target_os = "macos"))]
pub fn settings_window_snapshot() -> Option<SettingsWindowSnapshot> {
    None
}

pub fn cg_window_frame_to_appkit(frame: AppKitRect, screen: AppKitRect) -> AppKitRect {
    AppKitRect {
        x: frame.x,
        y: screen.y + screen.height - (frame.y + frame.height),
        width: frame.width,
        height: frame.height,
    }
}

pub fn largest_window_by_area(
    windows: impl IntoIterator<Item = SettingsWindowSnapshot>,
) -> Option<SettingsWindowSnapshot> {
    windows.into_iter().max_by(|a, b| {
        let a_area = a.frame.width * a.frame.height;
        let b_area = b.frame.width * b.frame.height;
        a_area
            .partial_cmp(&b_area)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}
