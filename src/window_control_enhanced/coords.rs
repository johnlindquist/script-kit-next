//! Coordinate conversion utilities between AX (top-left) and AppKit (bottom-left)

use super::bounds::WindowBounds;

/// Convert AppKit coordinates (origin at bottom-left of main screen) to AX coordinates
/// (origin at top-left of main screen).
///
/// # Arguments
/// * `x` - X coordinate in AppKit space
/// * `y` - Y coordinate in AppKit space (distance from bottom)
/// * `height` - Height of the rect being converted
/// * `main_screen_height` - Total height of the main screen
pub fn appkit_to_ax(x: f64, y: f64, height: f64, main_screen_height: f64) -> (f64, f64) {
    // In AppKit, y is distance from bottom of main screen
    // In AX, y is distance from top of main screen
    // AX_y = main_height - (appkit_y + rect_height)
    let ax_y = main_screen_height - (y + height);
    (x, ax_y)
}

/// Convert AX coordinates to AppKit coordinates
///
/// # Arguments
/// * `x` - X coordinate in AX space
/// * `y` - Y coordinate in AX space (distance from top)
/// * `height` - Height of the rect being converted
/// * `main_screen_height` - Total height of the main screen
pub fn ax_to_appkit(x: f64, y: f64, height: f64, main_screen_height: f64) -> (f64, f64) {
    // Inverse of appkit_to_ax
    // appkit_y = main_height - (ax_y + rect_height)
    let appkit_y = main_screen_height - (y + height);
    (x, appkit_y)
}

/// Convert an AppKit NSRect (origin at bottom-left) to WindowBounds (AX coords)
pub fn nsrect_to_bounds(
    origin_x: f64,
    origin_y: f64,
    width: f64,
    height: f64,
    main_screen_height: f64,
) -> WindowBounds {
    let (ax_x, ax_y) = appkit_to_ax(origin_x, origin_y, height, main_screen_height);
    WindowBounds::new(ax_x, ax_y, width, height)
}

/// Convert WindowBounds (AX coords) to AppKit coordinates
/// Returns (origin_x, origin_y, width, height) in AppKit space
pub fn bounds_to_nsrect(bounds: &WindowBounds, main_screen_height: f64) -> (f64, f64, f64, f64) {
    let (appkit_x, appkit_y) = ax_to_appkit(bounds.x, bounds.y, bounds.height, main_screen_height);
    (appkit_x, appkit_y, bounds.width, bounds.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAIN_HEIGHT: f64 = 1080.0;

    #[test]
    fn test_appkit_to_ax_top_of_screen() {
        // A window at AppKit coords (0, 555) with height 500 should be near top
        // AX_y = 1080 - (555 + 500) = 25
        let (ax_x, ax_y) = appkit_to_ax(0.0, 555.0, 500.0, MAIN_HEIGHT);
        assert_eq!(ax_x, 0.0);
        assert_eq!(ax_y, 25.0);
    }

    #[test]
    fn test_appkit_to_ax_bottom_of_screen() {
        // A window at AppKit coords (0, 0) with height 500
        // AX_y = 1080 - (0 + 500) = 580
        let (ax_x, ax_y) = appkit_to_ax(0.0, 0.0, 500.0, MAIN_HEIGHT);
        assert_eq!(ax_x, 0.0);
        assert_eq!(ax_y, 580.0);
    }

    #[test]
    fn test_ax_to_appkit_roundtrip() {
        let original_x = 100.0;
        let original_y = 200.0;
        let height = 400.0;

        let (ax_x, ax_y) = appkit_to_ax(original_x, original_y, height, MAIN_HEIGHT);
        let (back_x, back_y) = ax_to_appkit(ax_x, ax_y, height, MAIN_HEIGHT);

        assert!((back_x - original_x).abs() < 0.001);
        assert!((back_y - original_y).abs() < 0.001);
    }

    #[test]
    fn test_nsrect_to_bounds() {
        let bounds = nsrect_to_bounds(100.0, 50.0, 800.0, 600.0, MAIN_HEIGHT);
        assert_eq!(bounds.x, 100.0);
        assert_eq!(bounds.width, 800.0);
        assert_eq!(bounds.height, 600.0);
        // AX_y = 1080 - (50 + 600) = 430
        assert_eq!(bounds.y, 430.0);
    }

    #[test]
    fn test_bounds_to_nsrect() {
        let bounds = WindowBounds::new(100.0, 430.0, 800.0, 600.0);
        let (x, y, w, h) = bounds_to_nsrect(&bounds, MAIN_HEIGHT);
        assert_eq!(x, 100.0);
        assert_eq!(w, 800.0);
        assert_eq!(h, 600.0);
        // AppKit_y = 1080 - (430 + 600) = 50
        assert_eq!(y, 50.0);
    }
}
