//! Geometry helpers for `inspectAutomationWindow`.
//!
//! Computes screenshot-relative target bounds, default hit points, and
//! suggested click targets from resolved automation window metadata.

use super::automation_inspect::{InspectBoundsInScreenshot, InspectPoint, SuggestedHitPoint};
use super::automation_window::{
    AutomationWindowInfo, AutomationWindowKind, AutomationWindowTarget,
};

/// Compute the bounding rectangle of the target surface inside the
/// captured screenshot.
///
/// For attached surfaces (ActionsDialog, PromptPopup), this is offset
/// from the parent (main) window's origin. For detached windows, the
/// origin is `(0, 0)`.
pub fn target_bounds_in_screenshot(
    resolved: &AutomationWindowInfo,
) -> Option<InspectBoundsInScreenshot> {
    let bounds = resolved.bounds.as_ref()?;

    match resolved.kind {
        AutomationWindowKind::ActionsDialog | AutomationWindowKind::PromptPopup => {
            let main =
                crate::windows::resolve_automation_window(Some(&AutomationWindowTarget::Main))
                    .ok()?;
            let main_bounds = main.bounds.as_ref()?;
            Some(InspectBoundsInScreenshot {
                x: bounds.x - main_bounds.x,
                y: bounds.y - main_bounds.y,
                width: bounds.width,
                height: bounds.height,
            })
        }
        _ => Some(InspectBoundsInScreenshot {
            x: 0.0,
            y: 0.0,
            width: bounds.width,
            height: bounds.height,
        }),
    }
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
    resolved: &AutomationWindowInfo,
    bounds: Option<&InspectBoundsInScreenshot>,
) -> Vec<SuggestedHitPoint> {
    let Some(bounds) = bounds else {
        return Vec::new();
    };

    let center = default_surface_hit_point(bounds);

    let semantic_id = match resolved.kind {
        AutomationWindowKind::ActionsDialog => "panel:actions-dialog",
        AutomationWindowKind::PromptPopup => "panel:prompt-popup",
        AutomationWindowKind::Notes => "input:notes-editor",
        AutomationWindowKind::AcpDetached => "input:acp-composer",
        _ => "panel:window",
    };

    vec![SuggestedHitPoint {
        semantic_id: semantic_id.to_string(),
        x: center.x,
        y: center.y,
        reason: "surface_center".to_string(),
    }]
}

#[cfg(test)]
mod tests {
    use super::super::automation_window::AutomationWindowBounds;
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
        }
    }

    #[test]
    fn detached_window_bounds_at_origin() {
        let info = make_info(
            AutomationWindowKind::Notes,
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
        let info = make_info(AutomationWindowKind::Notes, None);
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
        let info = make_info(AutomationWindowKind::Main, None);
        let hits = default_suggested_hit_points(&info, None);
        assert!(hits.is_empty());
    }

    #[test]
    fn suggested_hit_uses_correct_semantic_id() {
        let bounds = InspectBoundsInScreenshot {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
        };
        let info = make_info(
            AutomationWindowKind::AcpDetached,
            Some(make_bounds(0.0, 0.0, 400.0, 300.0)),
        );
        let hits = default_suggested_hit_points(&info, Some(&bounds));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].semantic_id, "input:acp-composer");
        assert_eq!(hits[0].reason, "surface_center");
    }
}
