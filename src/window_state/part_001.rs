// ============================================================================
// Per-Display Position Storage (Notes Window)
// ============================================================================

#[allow(dead_code)]
pub fn save_notes_position_for_display(display: &DisplayBounds, bounds: PersistedWindowBounds) {
    let key = display_key(display);
    let mut state = load_state_file().unwrap_or_default();
    state.version = 3;
    state.notes_per_display.insert(key.clone(), bounds);
    state.notes = Some(bounds);
    save_state_file(&state);
    logging::log(
        "WINDOW_STATE",
        &format!("Saved Notes position for display {}", key),
    );
}
#[allow(dead_code)]
pub fn get_notes_position_for_mouse_display(
    mouse_x: f64,
    mouse_y: f64,
    displays: &[DisplayBounds],
) -> Option<(PersistedWindowBounds, DisplayBounds)> {
    let display = find_display_containing_point(mouse_x, mouse_y, displays)?;
    let key = display_key(display);
    let state = load_state_file()?;

    if let Some(saved) = state.notes_per_display.get(&key) {
        logging::log(
            "WINDOW_STATE",
            &format!("Restoring Notes per-display position for {}", key),
        );
        return Some((*saved, display.clone()));
    }

    if let Some(legacy) = state.notes {
        if let Some(legacy_display) = find_best_display_for_bounds(&legacy, displays) {
            if display_key(legacy_display) == key {
                return Some((legacy, display.clone()));
            }
        }
    }

    logging::log(
        "WINDOW_STATE",
        &format!("No saved Notes position for display {}", key),
    );
    None
}
// ============================================================================
// Visibility Validation
// ============================================================================

const MIN_VISIBLE_AREA: f64 = 64.0 * 64.0;
const MIN_EDGE_MARGIN: f64 = 50.0;
/// Check if saved bounds are still visible on current displays.
pub fn is_bounds_visible(bounds: &PersistedWindowBounds, displays: &[DisplayBounds]) -> bool {
    if displays.is_empty() {
        return false;
    }
    for display in displays {
        if let Some((_, _, w, h)) = rect_intersection(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            display.origin_x,
            display.origin_y,
            display.width,
            display.height,
        ) {
            if w * h >= MIN_VISIBLE_AREA {
                return true;
            }
        }
    }
    false
}
#[allow(clippy::too_many_arguments)]
fn rect_intersection(
    x1: f64,
    y1: f64,
    w1: f64,
    h1: f64,
    x2: f64,
    y2: f64,
    w2: f64,
    h2: f64,
) -> Option<(f64, f64, f64, f64)> {
    let left = x1.max(x2);
    let top = y1.max(y2);
    let right = (x1 + w1).min(x2 + w2);
    let bottom = (y1 + h1).min(y2 + h2);
    if left < right && top < bottom {
        Some((left, top, right - left, bottom - top))
    } else {
        None
    }
}
pub fn clamp_bounds_to_displays(
    bounds: &PersistedWindowBounds,
    displays: &[DisplayBounds],
) -> Option<PersistedWindowBounds> {
    if displays.is_empty() {
        return None;
    }
    let target = find_best_display_for_bounds(bounds, displays)?;
    let mut clamped = *bounds;
    clamped.width = clamped.width.min(target.width - MIN_EDGE_MARGIN * 2.0);
    clamped.height = clamped.height.min(target.height - MIN_EDGE_MARGIN * 2.0);
    let min_x = target.origin_x + MIN_EDGE_MARGIN;
    let max_x = target.origin_x + target.width - clamped.width - MIN_EDGE_MARGIN;
    clamped.x = clamped.x.max(min_x).min(max_x);
    let min_y = target.origin_y + MIN_EDGE_MARGIN;
    let max_y = target.origin_y + target.height - clamped.height - MIN_EDGE_MARGIN;
    clamped.y = clamped.y.max(min_y).min(max_y);
    Some(clamped)
}
fn find_best_display_for_bounds<'a>(
    bounds: &PersistedWindowBounds,
    displays: &'a [DisplayBounds],
) -> Option<&'a DisplayBounds> {
    let cx = bounds.x + bounds.width / 2.0;
    let cy = bounds.y + bounds.height / 2.0;
    for d in displays {
        if cx >= d.origin_x
            && cx < d.origin_x + d.width
            && cy >= d.origin_y
            && cy < d.origin_y + d.height
        {
            return Some(d);
        }
    }
    let mut best: Option<&DisplayBounds> = None;
    let mut best_area = 0.0;
    for d in displays {
        if let Some((_, _, w, h)) = rect_intersection(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            d.origin_x,
            d.origin_y,
            d.width,
            d.height,
        ) {
            if w * h > best_area {
                best_area = w * h;
                best = Some(d);
            }
        }
    }
    best.or_else(|| displays.first())
}
// ============================================================================
// High-Level API
// ============================================================================

pub fn get_initial_bounds(
    role: WindowRole,
    default_bounds: Bounds<Pixels>,
    displays: &[DisplayBounds],
) -> Bounds<Pixels> {
    if let Some(saved) = load_window_bounds(role) {
        if is_bounds_visible(&saved, displays) {
            logging::log(
                "WINDOW_STATE",
                &format!(
                    "Restoring {} to ({:.0}, {:.0})",
                    role.as_str(),
                    saved.x,
                    saved.y
                ),
            );
            return saved.to_gpui().get_bounds();
        }
        if let Some(clamped) = clamp_bounds_to_displays(&saved, displays) {
            logging::log(
                "WINDOW_STATE",
                &format!(
                    "Clamped {} to ({:.0}, {:.0})",
                    role.as_str(),
                    clamped.x,
                    clamped.y
                ),
            );
            return clamped.to_gpui().get_bounds();
        }
        logging::log(
            "WINDOW_STATE",
            &format!("{} saved position no longer visible", role.as_str()),
        );
    }
    default_bounds
}
pub fn save_window_from_gpui(role: WindowRole, window_bounds: WindowBounds) {
    save_window_bounds(role, PersistedWindowBounds::from_gpui(window_bounds));
}
// Tests are in src/window_state_persistence_tests.rs
