//! Geometry helpers for `inspectAutomationWindow`.
//!
//! Computes screenshot-relative target bounds, default hit points, and
//! suggested click targets from resolved automation window metadata.

use super::automation_inspect::{InspectBoundsInScreenshot, InspectPoint, SuggestedHitPoint};
use super::automation_window::AutomationWindowInfo;

/// Compute the bounding rectangle of the target surface inside the
/// captured screenshot. Every automation target owns its own window, so the
/// origin is `(0, 0)`.
pub fn target_bounds_in_screenshot(
    resolved: &AutomationWindowInfo,
) -> Option<InspectBoundsInScreenshot> {
    let bounds = resolved.bounds.as_ref()?;
    Some(InspectBoundsInScreenshot {
        x: 0.0,
        y: 0.0,
        width: bounds.width,
        height: bounds.height,
    })
}

/// Return the center of the given screenshot-relative bounds.
pub fn default_surface_hit_point(bounds: &InspectBoundsInScreenshot) -> InspectPoint {
    InspectPoint {
        x: bounds.x + (bounds.width / 2.0),
        y: bounds.y + (bounds.height / 2.0),
    }
}

/// Build a list of suggested named click targets for the surface.
pub fn default_suggested_hit_points(
    bounds: Option<&InspectBoundsInScreenshot>,
) -> Vec<SuggestedHitPoint> {
    let Some(bounds) = bounds else {
        return Vec::new();
    };

    let center = default_surface_hit_point(bounds);

    vec![SuggestedHitPoint {
        semantic_id: "panel:window".to_string(),
        x: center.x,
        y: center.y,
        reason: "surface_center".to_string(),
    }]
}

#[cfg(test)]
mod tests {
    use super::super::automation_window::{AutomationWindowBounds, AutomationWindowKind};
    use super::*;

    fn make_bounds(x: f64, y: f64, w: f64, h: f64) -> AutomationWindowBounds {
        AutomationWindowBounds {
            x,
            y,
            width: w,
            height: h,
        }
    }

    fn make_info(
        kind: AutomationWindowKind,
        bounds: Option<AutomationWindowBounds>,
    ) -> AutomationWindowInfo {
        AutomationWindowInfo {
            id: format!("{kind:?}:test"),
            kind,
            title: None,
            focused: false,
            visible: true,
            semantic_surface: None,
            bounds,
            parent_window_id: None,
            parent_window_generation: None,
            parent_kind: None,
            pid: None,
            generation: None,
        }
    }

    #[test]
    fn registered_window_bounds_at_origin() {
        let info = make_info(
            AutomationWindowKind::SnapOverlay,
            Some(make_bounds(500.0, 300.0, 800.0, 600.0)),
        );
        let result = target_bounds_in_screenshot(&info).expect("should compute");
        assert!((result.x - 0.0).abs() < f64::EPSILON);
        assert!((result.y - 0.0).abs() < f64::EPSILON);
        assert!((result.width - 800.0).abs() < f64::EPSILON);
        assert!((result.height - 600.0).abs() < f64::EPSILON);
    }

    #[test]
    fn no_bounds_returns_none() {
        let info = make_info(AutomationWindowKind::SnapOverlay, None);
        assert!(target_bounds_in_screenshot(&info).is_none());
    }

    #[test]
    fn default_hit_point_is_center() {
        let bounds = InspectBoundsInScreenshot {
            x: 100.0,
            y: 50.0,
            width: 200.0,
            height: 100.0,
        };
        let point = default_surface_hit_point(&bounds);
        assert!((point.x - 200.0).abs() < f64::EPSILON);
        assert!((point.y - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn no_bounds_no_suggested_hits() {
        let hits = default_suggested_hit_points(None);
        assert!(hits.is_empty());
    }

    #[test]
    fn suggested_hit_targets_the_window_panel() {
        let bounds = InspectBoundsInScreenshot {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
        };
        let hits = default_suggested_hit_points(Some(&bounds));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].semantic_id, "panel:window");
        assert_eq!(hits[0].reason, "surface_center");
    }
}
