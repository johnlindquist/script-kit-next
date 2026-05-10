use crate::protocol::TargetWindowBounds;

pub const COMPUTER_USE_WINDOW_OBSERVATION_SCHEMA_VERSION: u32 = 1;
pub const WINDOW_CAPTURE_REQUIRED_LAYER: i64 = 0;
pub const WINDOW_CAPTURE_MIN_ALPHA: f64 = 0.01;
pub const WINDOW_CAPTURE_MIN_WIDTH: u32 = 120;
pub const WINDOW_CAPTURE_MIN_HEIGHT: u32 = 90;
pub const CG_WINDOW_SHARING_NONE: i64 = 0;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputerUseWindowObservationV1 {
    pub schema_version: u32,
    pub source: &'static str,
    pub metadata_quality: WindowObservationMetadataQuality,
    pub alpha: Option<f64>,
    pub sharing_state: Option<i64>,
    pub capture_candidate: WindowCaptureCandidateV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplicate_group: Option<WindowDuplicateGroupV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_fallback: Option<WindowTitleFallbackV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub own_process_window_policy: Option<WindowOwnProcessPolicyV1>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowObservationMetadataQuality {
    Full,
    Partial,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowCaptureCandidateV1 {
    pub status: WindowCaptureCandidateStatus,
    pub reason: Option<WindowDisqualificationReason>,
    pub thresholds: WindowCaptureThresholdsV1,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowCaptureCandidateStatus {
    Candidate,
    Disqualified,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowDisqualificationReason {
    LayerNonZero,
    AlphaTooLow,
    SharingStateNone,
    NotOnScreen,
    TooSmall,
    MetadataIncomplete,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowCaptureThresholdsV1 {
    pub required_layer: i64,
    pub min_alpha: f64,
    pub min_width: u32,
    pub min_height: u32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowDuplicateGroupV1 {
    pub status: WindowDuplicateGroupStatus,
    pub group_count: usize,
    pub preferred_z_order: u32,
    pub selection_basis: WindowDuplicateSelectionBasis,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowDuplicateGroupStatus {
    Preferred,
    Duplicate,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowDuplicateSelectionBasis {
    OnScreenThenLargestAreaThenLowestZOrder,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowDuplicateObservationInputV1 {
    pub native_window_id: u32,
    pub bounds: TargetWindowBounds,
    pub is_on_screen: bool,
    pub z_order: u32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowTitleFallbackV1 {
    pub status: WindowTitleFallbackStatus,
    pub eligible_candidate_count: usize,
    pub selection_basis: WindowTitleFallbackSelectionBasis,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowTitleFallbackStatus {
    NonEmptyTitle,
    EmptyTitleSoleCandidate,
    EmptyTitleAmongMultipleCandidates,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowTitleFallbackSelectionBasis {
    PreferNonEmptyTitleThenAllowEmptyOnlyIfSoleCandidate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowTitleFallbackObservationInputV1 {
    pub title: Option<String>,
    pub capture_candidate_status: WindowCaptureCandidateStatus,
    pub duplicate_group_status: Option<WindowDuplicateGroupStatus>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowOwnProcessPolicyV1 {
    pub source: &'static str,
    pub status: WindowOwnProcessPolicyStatus,
    pub is_excluded_from_windows_menu: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowOwnProcessPolicyStatus {
    IncludedInWindowsMenu,
    ExcludedFromWindowsMenu,
    Unknown,
}

pub fn computer_use_window_observation_v1(
    bounds: &TargetWindowBounds,
    is_on_screen: bool,
    layer: i64,
    alpha: Option<f64>,
    sharing_state: Option<i64>,
) -> ComputerUseWindowObservationV1 {
    let metadata_quality = if alpha.is_some() && sharing_state.is_some() {
        WindowObservationMetadataQuality::Full
    } else {
        WindowObservationMetadataQuality::Partial
    };

    ComputerUseWindowObservationV1 {
        schema_version: COMPUTER_USE_WINDOW_OBSERVATION_SCHEMA_VERSION,
        source: "coreGraphicsWindowList",
        metadata_quality,
        alpha,
        sharing_state,
        capture_candidate: window_capture_candidate_v1(
            bounds,
            is_on_screen,
            layer,
            alpha,
            sharing_state,
        ),
        duplicate_group: None,
        title_fallback: None,
        own_process_window_policy: None,
    }
}

pub fn window_capture_candidate_v1(
    bounds: &TargetWindowBounds,
    is_on_screen: bool,
    layer: i64,
    alpha: Option<f64>,
    sharing_state: Option<i64>,
) -> WindowCaptureCandidateV1 {
    let reason = if layer != WINDOW_CAPTURE_REQUIRED_LAYER {
        Some(WindowDisqualificationReason::LayerNonZero)
    } else if alpha.is_some_and(|value| value <= WINDOW_CAPTURE_MIN_ALPHA) {
        Some(WindowDisqualificationReason::AlphaTooLow)
    } else if sharing_state == Some(CG_WINDOW_SHARING_NONE) {
        Some(WindowDisqualificationReason::SharingStateNone)
    } else if !is_on_screen {
        Some(WindowDisqualificationReason::NotOnScreen)
    } else if bounds.width < WINDOW_CAPTURE_MIN_WIDTH || bounds.height < WINDOW_CAPTURE_MIN_HEIGHT {
        Some(WindowDisqualificationReason::TooSmall)
    } else if alpha.is_none() || sharing_state.is_none() {
        Some(WindowDisqualificationReason::MetadataIncomplete)
    } else {
        None
    };

    let status = match reason {
        None => WindowCaptureCandidateStatus::Candidate,
        Some(WindowDisqualificationReason::MetadataIncomplete) => {
            WindowCaptureCandidateStatus::Unknown
        }
        Some(_) => WindowCaptureCandidateStatus::Disqualified,
    };

    WindowCaptureCandidateV1 {
        status,
        reason,
        thresholds: WindowCaptureThresholdsV1 {
            required_layer: WINDOW_CAPTURE_REQUIRED_LAYER,
            min_alpha: WINDOW_CAPTURE_MIN_ALPHA,
            min_width: WINDOW_CAPTURE_MIN_WIDTH,
            min_height: WINDOW_CAPTURE_MIN_HEIGHT,
        },
    }
}

pub fn window_duplicate_groups_v1(
    windows: &[WindowDuplicateObservationInputV1],
) -> Vec<Option<WindowDuplicateGroupV1>> {
    windows
        .iter()
        .map(|window| {
            let group_count = windows
                .iter()
                .filter(|candidate| candidate.native_window_id == window.native_window_id)
                .count();

            if group_count < 2 {
                return None;
            }

            let preferred = windows
                .iter()
                .filter(|candidate| candidate.native_window_id == window.native_window_id)
                .max_by_key(|candidate| {
                    (
                        candidate.is_on_screen,
                        window_area(&candidate.bounds),
                        std::cmp::Reverse(candidate.z_order),
                    )
                })
                .expect("duplicate group has at least one window");

            Some(WindowDuplicateGroupV1 {
                status: if std::ptr::eq(preferred, window) {
                    WindowDuplicateGroupStatus::Preferred
                } else {
                    WindowDuplicateGroupStatus::Duplicate
                },
                group_count,
                preferred_z_order: preferred.z_order,
                selection_basis:
                    WindowDuplicateSelectionBasis::OnScreenThenLargestAreaThenLowestZOrder,
            })
        })
        .collect()
}

fn window_area(bounds: &TargetWindowBounds) -> u64 {
    bounds.width as u64 * bounds.height as u64
}

pub fn window_title_fallbacks_v1(
    windows: &[WindowTitleFallbackObservationInputV1],
) -> Vec<Option<WindowTitleFallbackV1>> {
    let eligible_candidate_count = windows.iter().filter(|window| window.is_eligible()).count();

    windows
        .iter()
        .map(|window| {
            if !window.is_eligible() {
                return None;
            }

            let status = if window
                .title
                .as_deref()
                .is_some_and(|title| !title.trim().is_empty())
            {
                WindowTitleFallbackStatus::NonEmptyTitle
            } else if eligible_candidate_count == 1 {
                WindowTitleFallbackStatus::EmptyTitleSoleCandidate
            } else {
                WindowTitleFallbackStatus::EmptyTitleAmongMultipleCandidates
            };

            Some(WindowTitleFallbackV1 {
                status,
                eligible_candidate_count,
                selection_basis:
                    WindowTitleFallbackSelectionBasis::PreferNonEmptyTitleThenAllowEmptyOnlyIfSoleCandidate,
            })
        })
        .collect()
}

impl WindowTitleFallbackObservationInputV1 {
    fn is_eligible(&self) -> bool {
        self.capture_candidate_status == WindowCaptureCandidateStatus::Candidate
            && self.duplicate_group_status != Some(WindowDuplicateGroupStatus::Duplicate)
    }
}

pub fn window_own_process_policy_v1(
    is_current_process_window: bool,
    is_excluded_from_windows_menu: Option<bool>,
) -> Option<WindowOwnProcessPolicyV1> {
    if !is_current_process_window {
        return None;
    }

    let status = match is_excluded_from_windows_menu {
        Some(true) => WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu,
        Some(false) => WindowOwnProcessPolicyStatus::IncludedInWindowsMenu,
        None => WindowOwnProcessPolicyStatus::Unknown,
    };

    Some(WindowOwnProcessPolicyV1 {
        source: "nsWindow",
        status,
        is_excluded_from_windows_menu,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(width: u32, height: u32) -> TargetWindowBounds {
        TargetWindowBounds {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    #[test]
    fn window_capture_candidate_pins_disqualification_order() {
        for (candidate, status, reason) in [
            (
                window_capture_candidate_v1(&bounds(120, 90), true, 1, Some(1.0), Some(1)),
                WindowCaptureCandidateStatus::Disqualified,
                Some(WindowDisqualificationReason::LayerNonZero),
            ),
            (
                window_capture_candidate_v1(&bounds(120, 90), true, 0, Some(0.0), Some(1)),
                WindowCaptureCandidateStatus::Disqualified,
                Some(WindowDisqualificationReason::AlphaTooLow),
            ),
            (
                window_capture_candidate_v1(&bounds(120, 90), true, 0, Some(1.0), Some(0)),
                WindowCaptureCandidateStatus::Disqualified,
                Some(WindowDisqualificationReason::SharingStateNone),
            ),
            (
                window_capture_candidate_v1(&bounds(120, 90), false, 0, Some(1.0), Some(1)),
                WindowCaptureCandidateStatus::Disqualified,
                Some(WindowDisqualificationReason::NotOnScreen),
            ),
            (
                window_capture_candidate_v1(&bounds(119, 90), true, 0, Some(1.0), Some(1)),
                WindowCaptureCandidateStatus::Disqualified,
                Some(WindowDisqualificationReason::TooSmall),
            ),
            (
                window_capture_candidate_v1(&bounds(120, 90), true, 0, None, Some(1)),
                WindowCaptureCandidateStatus::Unknown,
                Some(WindowDisqualificationReason::MetadataIncomplete),
            ),
            (
                window_capture_candidate_v1(&bounds(120, 90), true, 0, Some(1.0), None),
                WindowCaptureCandidateStatus::Unknown,
                Some(WindowDisqualificationReason::MetadataIncomplete),
            ),
            (
                window_capture_candidate_v1(&bounds(120, 90), true, 0, Some(1.0), Some(1)),
                WindowCaptureCandidateStatus::Candidate,
                None,
            ),
        ] {
            assert_eq!(candidate.status, status);
            assert_eq!(candidate.reason, reason);
            assert_eq!(candidate.thresholds.required_layer, 0);
            assert_eq!(candidate.thresholds.min_alpha, 0.01);
            assert_eq!(candidate.thresholds.min_width, 120);
            assert_eq!(candidate.thresholds.min_height, 90);
        }
    }

    #[test]
    fn window_observation_marks_metadata_quality() {
        assert_eq!(
            computer_use_window_observation_v1(&bounds(120, 90), true, 0, Some(1.0), Some(1))
                .metadata_quality,
            WindowObservationMetadataQuality::Full
        );
        assert_eq!(
            computer_use_window_observation_v1(&bounds(120, 90), true, 0, None, Some(1))
                .metadata_quality,
            WindowObservationMetadataQuality::Partial
        );
    }

    #[test]
    fn window_duplicate_groups_omits_unique_windows() {
        let groups = window_duplicate_groups_v1(&[
            duplicate_input(1, 120, 90, true, 0),
            duplicate_input(2, 120, 90, true, 1),
        ]);

        assert_eq!(groups, vec![None, None]);
    }

    #[test]
    fn window_duplicate_groups_marks_largest_area_as_preferred() {
        let groups = window_duplicate_groups_v1(&[
            duplicate_input(7, 120, 90, true, 0),
            duplicate_input(7, 300, 200, true, 1),
        ]);

        assert_eq!(
            groups[0].as_ref().map(|group| &group.status),
            Some(&WindowDuplicateGroupStatus::Duplicate)
        );
        assert_eq!(
            groups[1].as_ref().map(|group| &group.status),
            Some(&WindowDuplicateGroupStatus::Preferred)
        );
        assert_eq!(groups[0].as_ref().map(|group| group.group_count), Some(2));
        assert_eq!(
            groups[0].as_ref().map(|group| group.preferred_z_order),
            Some(1)
        );
    }

    #[test]
    fn window_duplicate_groups_ties_on_on_screen_then_lowest_z_order() {
        let offscreen_larger = duplicate_input(9, 500, 500, false, 0);
        let onscreen_smaller = duplicate_input(9, 120, 90, true, 1);
        let onscreen_same_area_later = duplicate_input(9, 120, 90, true, 2);

        let groups = window_duplicate_groups_v1(&[
            offscreen_larger,
            onscreen_smaller,
            onscreen_same_area_later,
        ]);

        assert_eq!(
            groups[0].as_ref().map(|group| &group.status),
            Some(&WindowDuplicateGroupStatus::Duplicate)
        );
        assert_eq!(
            groups[1].as_ref().map(|group| &group.status),
            Some(&WindowDuplicateGroupStatus::Preferred)
        );
        assert_eq!(
            groups[2].as_ref().map(|group| &group.status),
            Some(&WindowDuplicateGroupStatus::Duplicate)
        );
        assert_eq!(
            groups[2].as_ref().map(|group| group.preferred_z_order),
            Some(1)
        );
    }

    #[test]
    fn window_duplicate_groups_preserves_input_length_and_order() {
        let groups = window_duplicate_groups_v1(&[
            duplicate_input(1, 120, 90, true, 0),
            duplicate_input(2, 400, 300, true, 1),
            duplicate_input(2, 120, 90, true, 2),
            duplicate_input(3, 120, 90, true, 3),
        ]);

        assert_eq!(groups.len(), 4);
        assert!(groups[0].is_none());
        assert_eq!(
            groups[1].as_ref().map(|group| &group.status),
            Some(&WindowDuplicateGroupStatus::Preferred)
        );
        assert_eq!(
            groups[2].as_ref().map(|group| &group.status),
            Some(&WindowDuplicateGroupStatus::Duplicate)
        );
        assert!(groups[3].is_none());
    }

    #[test]
    fn window_duplicate_groups_marks_only_one_preferred_when_z_order_repeats() {
        let groups = window_duplicate_groups_v1(&[
            duplicate_input(4, 200, 100, true, 1),
            duplicate_input(4, 200, 100, true, 1),
        ]);

        let preferred_count = groups
            .iter()
            .filter(|group| {
                group
                    .as_ref()
                    .is_some_and(|group| group.status == WindowDuplicateGroupStatus::Preferred)
            })
            .count();

        assert_eq!(preferred_count, 1);
    }

    #[test]
    fn window_title_fallback_marks_non_empty_title() {
        let fallbacks = window_title_fallbacks_v1(&[title_input(
            Some("Console".to_string()),
            WindowCaptureCandidateStatus::Candidate,
            None,
        )]);

        assert_eq!(
            fallbacks[0].as_ref().map(|fallback| &fallback.status),
            Some(&WindowTitleFallbackStatus::NonEmptyTitle)
        );
    }

    #[test]
    fn window_title_fallback_allows_empty_title_when_sole_candidate() {
        let fallbacks = window_title_fallbacks_v1(&[title_input(
            None,
            WindowCaptureCandidateStatus::Candidate,
            None,
        )]);

        assert_eq!(
            fallbacks[0].as_ref().map(|fallback| &fallback.status),
            Some(&WindowTitleFallbackStatus::EmptyTitleSoleCandidate)
        );
        assert_eq!(
            fallbacks[0]
                .as_ref()
                .map(|fallback| fallback.eligible_candidate_count),
            Some(1)
        );
    }

    #[test]
    fn window_title_fallback_discourages_empty_title_among_multiple_candidates() {
        let fallbacks = window_title_fallbacks_v1(&[
            title_input(None, WindowCaptureCandidateStatus::Candidate, None),
            title_input(
                Some("Editor".to_string()),
                WindowCaptureCandidateStatus::Candidate,
                None,
            ),
        ]);

        assert_eq!(
            fallbacks[0].as_ref().map(|fallback| &fallback.status),
            Some(&WindowTitleFallbackStatus::EmptyTitleAmongMultipleCandidates)
        );
        assert_eq!(
            fallbacks[0]
                .as_ref()
                .map(|fallback| fallback.eligible_candidate_count),
            Some(2)
        );
    }

    #[test]
    fn window_title_fallback_treats_whitespace_title_as_empty() {
        let fallbacks = window_title_fallbacks_v1(&[
            title_input(
                Some("   ".to_string()),
                WindowCaptureCandidateStatus::Candidate,
                None,
            ),
            title_input(
                Some("Terminal".to_string()),
                WindowCaptureCandidateStatus::Candidate,
                None,
            ),
        ]);

        assert_eq!(
            fallbacks[0].as_ref().map(|fallback| &fallback.status),
            Some(&WindowTitleFallbackStatus::EmptyTitleAmongMultipleCandidates)
        );
    }

    #[test]
    fn window_title_fallback_ignores_capture_disqualified_and_unknown_rows() {
        let fallbacks = window_title_fallbacks_v1(&[
            title_input(None, WindowCaptureCandidateStatus::Disqualified, None),
            title_input(None, WindowCaptureCandidateStatus::Unknown, None),
            title_input(None, WindowCaptureCandidateStatus::Candidate, None),
        ]);

        assert!(fallbacks[0].is_none());
        assert!(fallbacks[1].is_none());
        assert_eq!(
            fallbacks[2].as_ref().map(|fallback| &fallback.status),
            Some(&WindowTitleFallbackStatus::EmptyTitleSoleCandidate)
        );
    }

    #[test]
    fn window_title_fallback_ignores_duplicate_rows() {
        let fallbacks = window_title_fallbacks_v1(&[
            title_input(
                None,
                WindowCaptureCandidateStatus::Candidate,
                Some(WindowDuplicateGroupStatus::Duplicate),
            ),
            title_input(
                None,
                WindowCaptureCandidateStatus::Candidate,
                Some(WindowDuplicateGroupStatus::Preferred),
            ),
        ]);

        assert!(fallbacks[0].is_none());
        assert_eq!(
            fallbacks[1].as_ref().map(|fallback| &fallback.status),
            Some(&WindowTitleFallbackStatus::EmptyTitleSoleCandidate)
        );
    }

    #[test]
    fn window_title_fallback_preserves_input_length_and_order() {
        let fallbacks = window_title_fallbacks_v1(&[
            title_input(
                Some("A".to_string()),
                WindowCaptureCandidateStatus::Candidate,
                None,
            ),
            title_input(None, WindowCaptureCandidateStatus::Disqualified, None),
            title_input(None, WindowCaptureCandidateStatus::Candidate, None),
        ]);

        assert_eq!(fallbacks.len(), 3);
        assert_eq!(
            fallbacks[0].as_ref().map(|fallback| &fallback.status),
            Some(&WindowTitleFallbackStatus::NonEmptyTitle)
        );
        assert!(fallbacks[1].is_none());
        assert_eq!(
            fallbacks[2].as_ref().map(|fallback| &fallback.status),
            Some(&WindowTitleFallbackStatus::EmptyTitleAmongMultipleCandidates)
        );
    }

    #[test]
    fn own_process_window_policy_omits_non_current_process_windows() {
        assert_eq!(window_own_process_policy_v1(false, Some(true)), None);
    }

    #[test]
    fn own_process_window_policy_marks_excluded_windows() {
        let policy = window_own_process_policy_v1(true, Some(true)).expect("policy");

        assert_eq!(
            policy.status,
            WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu
        );
        assert_eq!(policy.is_excluded_from_windows_menu, Some(true));
    }

    #[test]
    fn own_process_window_policy_marks_included_windows() {
        let policy = window_own_process_policy_v1(true, Some(false)).expect("policy");

        assert_eq!(
            policy.status,
            WindowOwnProcessPolicyStatus::IncludedInWindowsMenu
        );
        assert_eq!(policy.is_excluded_from_windows_menu, Some(false));
    }

    #[test]
    fn own_process_window_policy_marks_unknown_when_nswindow_lookup_fails() {
        let policy = window_own_process_policy_v1(true, None).expect("policy");

        assert_eq!(policy.status, WindowOwnProcessPolicyStatus::Unknown);
        assert_eq!(policy.is_excluded_from_windows_menu, None);
    }

    #[test]
    fn window_observation_initializes_own_process_policy_as_none() {
        assert_eq!(
            computer_use_window_observation_v1(&bounds(120, 90), true, 0, Some(1.0), Some(1))
                .own_process_window_policy,
            None
        );
    }

    #[test]
    fn own_process_window_policy_serializes_camel_case_contract() {
        let policy = window_own_process_policy_v1(true, Some(true)).expect("policy");
        let serialized = serde_json::to_value(policy).expect("serialize policy");

        assert_eq!(serialized["source"], "nsWindow");
        assert_eq!(serialized["status"], "excludedFromWindowsMenu");
        assert_eq!(serialized["isExcludedFromWindowsMenu"], true);
    }

    fn duplicate_input(
        native_window_id: u32,
        width: u32,
        height: u32,
        is_on_screen: bool,
        z_order: u32,
    ) -> WindowDuplicateObservationInputV1 {
        WindowDuplicateObservationInputV1 {
            native_window_id,
            bounds: bounds(width, height),
            is_on_screen,
            z_order,
        }
    }

    fn title_input(
        title: Option<String>,
        capture_candidate_status: WindowCaptureCandidateStatus,
        duplicate_group_status: Option<WindowDuplicateGroupStatus>,
    ) -> WindowTitleFallbackObservationInputV1 {
        WindowTitleFallbackObservationInputV1 {
            title,
            capture_candidate_status,
            duplicate_group_status,
        }
    }
}
