use crate::protocol::TargetWindowBounds;

pub const COMPUTER_USE_WINDOW_OBSERVATION_SCHEMA_VERSION: u32 = 1;
pub const WINDOW_CAPTURE_REQUIRED_LAYER: i64 = 0;
pub const WINDOW_CAPTURE_MIN_ALPHA: f64 = 0.01;
pub const WINDOW_CAPTURE_MIN_WIDTH: u32 = 120;
pub const WINDOW_CAPTURE_MIN_HEIGHT: u32 = 90;
pub const WINDOW_LIST_REQUIRED_LAYER: i64 = 0;
pub const WINDOW_LIST_MIN_ALPHA: f64 = 0.0;
pub const WINDOW_LIST_MIN_WIDTH: u32 = 60;
pub const WINDOW_LIST_MIN_HEIGHT: u32 = 60;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_candidate: Option<WindowListCandidateV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_selection_candidate: Option<WindowCaptureSelectionCandidateV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enumeration_context: Option<WindowEnumerationContextV1>,
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
pub struct WindowListCandidateV1 {
    pub status: WindowListCandidateStatus,
    pub reason: Option<WindowListDisqualificationReason>,
    pub thresholds: WindowListThresholdsV1,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowListCandidateStatus {
    Candidate,
    Disqualified,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowListDisqualificationReason {
    LayerNonZero,
    AlphaTooLow,
    TooSmall,
    OwnProcessExcludedFromWindowsMenu,
    MetadataIncomplete,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowListThresholdsV1 {
    pub required_layer: i64,
    pub min_alpha: f64,
    pub min_width: u32,
    pub min_height: u32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowEnumerationContextV1 {
    pub source: &'static str,
    pub mode: WindowEnumerationMode,
    pub relative_to_window: u32,
    pub raw_options: WindowEnumerationRawOptionsV1,
    pub offscreen_coverage: WindowEnumerationCoverageStatus,
    pub desktop_element_policy: WindowEnumerationDesktopElementPolicy,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowEnumerationMode {
    OnScreenOnly,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowEnumerationCoverageStatus {
    NotEnumerated,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowEnumerationDesktopElementPolicy {
    NotExcludedByOption,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowEnumerationRawOptionsV1 {
    pub option_on_screen_only: bool,
    pub option_all: bool,
    pub option_exclude_desktop_elements: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowEnumerationObservationInputV1 {
    pub option_on_screen_only: bool,
    pub option_all: bool,
    pub option_exclude_desktop_elements: bool,
    pub relative_to_window: u32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowCaptureSelectionCandidateV1 {
    pub status: WindowCaptureSelectionCandidateStatus,
    pub reason: Option<WindowCaptureSelectionDisqualificationReason>,
    pub selection_basis: WindowCaptureSelectionBasis,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowCaptureSelectionCandidateStatus {
    Candidate,
    Disqualified,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowCaptureSelectionDisqualificationReason {
    LayerNonZero,
    AlphaTooLow,
    SharingStateNone,
    NotOnScreen,
    TooSmall,
    MetadataIncomplete,
    DuplicateWindow,
    EmptyTitleAmongMultipleCandidates,
    OwnProcessExcludedFromWindowsMenu,
    OwnProcessPolicyUnknown,
    SelectionMetadataIncomplete,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowCaptureSelectionBasis {
    CaptureCandidateThenPreferredDuplicateThenTitleFallbackThenOwnProcessPolicy,
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

#[derive(Clone, Debug, PartialEq)]
pub struct WindowCaptureSelectionObservationInputV1 {
    pub capture_candidate_status: WindowCaptureCandidateStatus,
    pub capture_candidate_reason: Option<WindowDisqualificationReason>,
    pub duplicate_group_status: Option<WindowDuplicateGroupStatus>,
    pub title_fallback_status: Option<WindowTitleFallbackStatus>,
    pub own_process_window_policy_status: Option<WindowOwnProcessPolicyStatus>,
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
        list_candidate: None,
        capture_selection_candidate: None,
        enumeration_context: None,
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

pub fn window_list_candidate_v1(
    bounds: &TargetWindowBounds,
    layer: i64,
    alpha: Option<f64>,
    own_process_window_policy_status: Option<WindowOwnProcessPolicyStatus>,
) -> WindowListCandidateV1 {
    let reason = if layer != WINDOW_LIST_REQUIRED_LAYER {
        Some(WindowListDisqualificationReason::LayerNonZero)
    } else if alpha.is_some_and(|value| value <= WINDOW_LIST_MIN_ALPHA) {
        Some(WindowListDisqualificationReason::AlphaTooLow)
    } else if bounds.width < WINDOW_LIST_MIN_WIDTH || bounds.height < WINDOW_LIST_MIN_HEIGHT {
        Some(WindowListDisqualificationReason::TooSmall)
    } else if own_process_window_policy_status
        == Some(WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu)
    {
        Some(WindowListDisqualificationReason::OwnProcessExcludedFromWindowsMenu)
    } else if alpha.is_none() {
        Some(WindowListDisqualificationReason::MetadataIncomplete)
    } else {
        None
    };

    let status = match reason {
        None => WindowListCandidateStatus::Candidate,
        Some(WindowListDisqualificationReason::MetadataIncomplete) => {
            WindowListCandidateStatus::Unknown
        }
        Some(_) => WindowListCandidateStatus::Disqualified,
    };

    WindowListCandidateV1 {
        status,
        reason,
        thresholds: WindowListThresholdsV1 {
            required_layer: WINDOW_LIST_REQUIRED_LAYER,
            min_alpha: WINDOW_LIST_MIN_ALPHA,
            min_width: WINDOW_LIST_MIN_WIDTH,
            min_height: WINDOW_LIST_MIN_HEIGHT,
        },
    }
}

pub fn window_enumeration_context_v1(
    input: WindowEnumerationObservationInputV1,
) -> WindowEnumerationContextV1 {
    let offscreen_coverage = if input.option_on_screen_only {
        WindowEnumerationCoverageStatus::NotEnumerated
    } else {
        WindowEnumerationCoverageStatus::Unknown
    };

    let desktop_element_policy = if input.option_exclude_desktop_elements {
        WindowEnumerationDesktopElementPolicy::Unknown
    } else {
        WindowEnumerationDesktopElementPolicy::NotExcludedByOption
    };

    WindowEnumerationContextV1 {
        source: "coreGraphicsWindowList",
        mode: WindowEnumerationMode::OnScreenOnly,
        relative_to_window: input.relative_to_window,
        raw_options: WindowEnumerationRawOptionsV1 {
            option_on_screen_only: input.option_on_screen_only,
            option_all: input.option_all,
            option_exclude_desktop_elements: input.option_exclude_desktop_elements,
        },
        offscreen_coverage,
        desktop_element_policy,
    }
}

pub fn window_capture_selection_candidates_v1(
    windows: &[WindowCaptureSelectionObservationInputV1],
) -> Vec<WindowCaptureSelectionCandidateV1> {
    windows
        .iter()
        .map(window_capture_selection_candidate_v1)
        .collect()
}

fn window_capture_selection_candidate_v1(
    window: &WindowCaptureSelectionObservationInputV1,
) -> WindowCaptureSelectionCandidateV1 {
    let reason = match window.capture_candidate_status {
        WindowCaptureCandidateStatus::Disqualified
            if window.capture_candidate_reason
                == Some(WindowDisqualificationReason::LayerNonZero)
                && window.own_process_window_policy_status
                    == Some(WindowOwnProcessPolicyStatus::IncludedInWindowsMenu) =>
        {
            window_capture_selection_reason_after_base_candidate(window)
        }
        WindowCaptureCandidateStatus::Disqualified => window
            .capture_candidate_reason
            .clone()
            .map(window_capture_selection_reason_from_capture_reason)
            .or(Some(
                WindowCaptureSelectionDisqualificationReason::SelectionMetadataIncomplete,
            )),
        WindowCaptureCandidateStatus::Unknown => {
            Some(WindowCaptureSelectionDisqualificationReason::MetadataIncomplete)
        }
        WindowCaptureCandidateStatus::Candidate => {
            window_capture_selection_reason_after_base_candidate(window)
        }
    };

    let status = match &reason {
        None => WindowCaptureSelectionCandidateStatus::Candidate,
        Some(WindowCaptureSelectionDisqualificationReason::MetadataIncomplete)
        | Some(WindowCaptureSelectionDisqualificationReason::OwnProcessPolicyUnknown)
        | Some(WindowCaptureSelectionDisqualificationReason::SelectionMetadataIncomplete) => {
            WindowCaptureSelectionCandidateStatus::Unknown
        }
        Some(_) => WindowCaptureSelectionCandidateStatus::Disqualified,
    };

    WindowCaptureSelectionCandidateV1 {
        status,
        reason,
        selection_basis:
            WindowCaptureSelectionBasis::CaptureCandidateThenPreferredDuplicateThenTitleFallbackThenOwnProcessPolicy,
    }
}

fn window_capture_selection_reason_after_base_candidate(
    window: &WindowCaptureSelectionObservationInputV1,
) -> Option<WindowCaptureSelectionDisqualificationReason> {
    if window.duplicate_group_status == Some(WindowDuplicateGroupStatus::Duplicate) {
        Some(WindowCaptureSelectionDisqualificationReason::DuplicateWindow)
    } else if window.title_fallback_status
        == Some(WindowTitleFallbackStatus::EmptyTitleAmongMultipleCandidates)
    {
        Some(WindowCaptureSelectionDisqualificationReason::EmptyTitleAmongMultipleCandidates)
    } else if window.own_process_window_policy_status
        == Some(WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu)
    {
        Some(WindowCaptureSelectionDisqualificationReason::OwnProcessExcludedFromWindowsMenu)
    } else if window.own_process_window_policy_status == Some(WindowOwnProcessPolicyStatus::Unknown)
    {
        Some(WindowCaptureSelectionDisqualificationReason::OwnProcessPolicyUnknown)
    } else if window.own_process_window_policy_status
        == Some(WindowOwnProcessPolicyStatus::IncludedInWindowsMenu)
    {
        None
    } else if window.title_fallback_status.is_none() {
        Some(WindowCaptureSelectionDisqualificationReason::SelectionMetadataIncomplete)
    } else {
        None
    }
}

fn window_capture_selection_reason_from_capture_reason(
    reason: WindowDisqualificationReason,
) -> WindowCaptureSelectionDisqualificationReason {
    match reason {
        WindowDisqualificationReason::LayerNonZero => {
            WindowCaptureSelectionDisqualificationReason::LayerNonZero
        }
        WindowDisqualificationReason::AlphaTooLow => {
            WindowCaptureSelectionDisqualificationReason::AlphaTooLow
        }
        WindowDisqualificationReason::SharingStateNone => {
            WindowCaptureSelectionDisqualificationReason::SharingStateNone
        }
        WindowDisqualificationReason::NotOnScreen => {
            WindowCaptureSelectionDisqualificationReason::NotOnScreen
        }
        WindowDisqualificationReason::TooSmall => {
            WindowCaptureSelectionDisqualificationReason::TooSmall
        }
        WindowDisqualificationReason::MetadataIncomplete => {
            WindowCaptureSelectionDisqualificationReason::MetadataIncomplete
        }
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
                });

            preferred.map(|preferred| WindowDuplicateGroupV1 {
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
    fn window_list_candidate_allows_looser_list_thresholds() {
        let candidate = window_list_candidate_v1(&bounds(60, 60), 0, Some(0.01), None);

        assert_eq!(candidate.status, WindowListCandidateStatus::Candidate);
        assert_eq!(candidate.reason, None);
        assert_eq!(candidate.thresholds.required_layer, 0);
        assert_eq!(candidate.thresholds.min_alpha, 0.0);
        assert_eq!(candidate.thresholds.min_width, 60);
        assert_eq!(candidate.thresholds.min_height, 60);
    }

    #[test]
    fn window_list_candidate_rejects_zero_alpha() {
        let candidate = window_list_candidate_v1(&bounds(60, 60), 0, Some(0.0), None);

        assert_eq!(candidate.status, WindowListCandidateStatus::Disqualified);
        assert_eq!(
            candidate.reason,
            Some(WindowListDisqualificationReason::AlphaTooLow)
        );
    }

    #[test]
    fn window_list_candidate_rejects_too_small_rows() {
        let candidate = window_list_candidate_v1(&bounds(59, 60), 0, Some(0.01), None);

        assert_eq!(candidate.status, WindowListCandidateStatus::Disqualified);
        assert_eq!(
            candidate.reason,
            Some(WindowListDisqualificationReason::TooSmall)
        );
    }

    #[test]
    fn window_list_candidate_rejects_own_process_windows_excluded_from_windows_menu() {
        let candidate = window_list_candidate_v1(
            &bounds(60, 60),
            0,
            Some(0.01),
            Some(WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu),
        );

        assert_eq!(candidate.status, WindowListCandidateStatus::Disqualified);
        assert_eq!(
            candidate.reason,
            Some(WindowListDisqualificationReason::OwnProcessExcludedFromWindowsMenu)
        );
    }

    #[test]
    fn window_list_candidate_marks_missing_alpha_unknown() {
        let candidate = window_list_candidate_v1(&bounds(60, 60), 0, None, None);

        assert_eq!(candidate.status, WindowListCandidateStatus::Unknown);
        assert_eq!(
            candidate.reason,
            Some(WindowListDisqualificationReason::MetadataIncomplete)
        );
    }

    #[test]
    fn window_list_candidate_serializes_camel_case_contract() {
        let candidate = window_list_candidate_v1(
            &bounds(60, 60),
            0,
            Some(0.01),
            Some(WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu),
        );
        let serialized = serde_json::to_value(candidate).expect("serialize candidate");

        assert_eq!(serialized["status"], "disqualified");
        assert_eq!(serialized["reason"], "ownProcessExcludedFromWindowsMenu");
        assert_eq!(serialized["thresholds"]["requiredLayer"], 0);
        assert_eq!(serialized["thresholds"]["minAlpha"], 0.0);
        assert_eq!(serialized["thresholds"]["minWidth"], 60);
        assert_eq!(serialized["thresholds"]["minHeight"], 60);
    }

    #[test]
    fn window_enumeration_context_marks_on_screen_only_inventory() {
        let context = window_enumeration_context_v1(enumeration_input(true, false, false, 0));

        assert_eq!(context.source, "coreGraphicsWindowList");
        assert_eq!(context.mode, WindowEnumerationMode::OnScreenOnly);
        assert_eq!(context.relative_to_window, 0);
        assert_eq!(
            context.offscreen_coverage,
            WindowEnumerationCoverageStatus::NotEnumerated
        );
        assert_eq!(
            context.desktop_element_policy,
            WindowEnumerationDesktopElementPolicy::NotExcludedByOption
        );
    }

    #[test]
    fn window_enumeration_context_preserves_raw_options() {
        let context = window_enumeration_context_v1(enumeration_input(true, false, false, 0));

        assert_eq!(context.raw_options.option_on_screen_only, true);
        assert_eq!(context.raw_options.option_all, false);
        assert_eq!(context.raw_options.option_exclude_desktop_elements, false);
    }

    #[test]
    fn window_enumeration_context_marks_offscreen_not_enumerated_for_on_screen_only() {
        let context = window_enumeration_context_v1(enumeration_input(true, false, false, 0));

        assert_eq!(
            context.offscreen_coverage,
            WindowEnumerationCoverageStatus::NotEnumerated
        );
    }

    #[test]
    fn window_enumeration_context_serializes_camel_case_contract() {
        let context = window_enumeration_context_v1(enumeration_input(true, false, false, 0));
        let serialized = serde_json::to_value(context).expect("serialize context");

        assert_eq!(serialized["source"], "coreGraphicsWindowList");
        assert_eq!(serialized["mode"], "onScreenOnly");
        assert_eq!(serialized["relativeToWindow"], 0);
        assert_eq!(serialized["rawOptions"]["optionOnScreenOnly"], true);
        assert_eq!(serialized["rawOptions"]["optionAll"], false);
        assert_eq!(
            serialized["rawOptions"]["optionExcludeDesktopElements"],
            false
        );
        assert_eq!(serialized["offscreenCoverage"], "notEnumerated");
        assert_eq!(serialized["desktopElementPolicy"], "notExcludedByOption");
    }

    #[test]
    fn window_capture_selection_candidate_passes_clean_candidate() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Candidate,
            None,
            None,
            Some(WindowTitleFallbackStatus::NonEmptyTitle),
            None,
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Candidate
        );
        assert_eq!(candidates[0].reason, None);
    }

    #[test]
    fn window_capture_selection_candidate_maps_base_disqualification_reasons() {
        for (capture_reason, selection_reason) in [
            (
                WindowDisqualificationReason::LayerNonZero,
                WindowCaptureSelectionDisqualificationReason::LayerNonZero,
            ),
            (
                WindowDisqualificationReason::AlphaTooLow,
                WindowCaptureSelectionDisqualificationReason::AlphaTooLow,
            ),
            (
                WindowDisqualificationReason::SharingStateNone,
                WindowCaptureSelectionDisqualificationReason::SharingStateNone,
            ),
            (
                WindowDisqualificationReason::NotOnScreen,
                WindowCaptureSelectionDisqualificationReason::NotOnScreen,
            ),
            (
                WindowDisqualificationReason::TooSmall,
                WindowCaptureSelectionDisqualificationReason::TooSmall,
            ),
        ] {
            let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
                WindowCaptureCandidateStatus::Disqualified,
                Some(capture_reason),
                None,
                None,
                None,
            )]);

            assert_eq!(
                candidates[0].status,
                WindowCaptureSelectionCandidateStatus::Disqualified
            );
            assert_eq!(candidates[0].reason, Some(selection_reason));
        }
    }

    #[test]
    fn window_capture_selection_candidate_allows_own_process_panel_layer() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Disqualified,
            Some(WindowDisqualificationReason::LayerNonZero),
            None,
            Some(WindowTitleFallbackStatus::EmptyTitleSoleCandidate),
            Some(WindowOwnProcessPolicyStatus::IncludedInWindowsMenu),
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Candidate
        );
        assert_eq!(candidates[0].reason, None);
    }

    #[test]
    fn window_capture_selection_candidate_allows_own_process_panel_without_title_metadata() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Disqualified,
            Some(WindowDisqualificationReason::LayerNonZero),
            None,
            None,
            Some(WindowOwnProcessPolicyStatus::IncludedInWindowsMenu),
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Candidate
        );
        assert_eq!(candidates[0].reason, None);
    }

    #[test]
    fn window_capture_selection_candidate_keeps_third_party_panel_layer_rejected() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Disqualified,
            Some(WindowDisqualificationReason::LayerNonZero),
            None,
            Some(WindowTitleFallbackStatus::EmptyTitleSoleCandidate),
            None,
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Disqualified
        );
        assert_eq!(
            candidates[0].reason,
            Some(WindowCaptureSelectionDisqualificationReason::LayerNonZero)
        );
    }

    #[test]
    fn window_capture_selection_candidate_keeps_metadata_incomplete_unknown() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Unknown,
            Some(WindowDisqualificationReason::MetadataIncomplete),
            None,
            None,
            None,
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Unknown
        );
        assert_eq!(
            candidates[0].reason,
            Some(WindowCaptureSelectionDisqualificationReason::MetadataIncomplete)
        );
    }

    #[test]
    fn window_capture_selection_candidate_rejects_duplicate_rows() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Candidate,
            None,
            Some(WindowDuplicateGroupStatus::Duplicate),
            Some(WindowTitleFallbackStatus::NonEmptyTitle),
            None,
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Disqualified
        );
        assert_eq!(
            candidates[0].reason,
            Some(WindowCaptureSelectionDisqualificationReason::DuplicateWindow)
        );
    }

    #[test]
    fn window_capture_selection_candidate_rejects_empty_title_among_multiple_candidates() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Candidate,
            None,
            Some(WindowDuplicateGroupStatus::Preferred),
            Some(WindowTitleFallbackStatus::EmptyTitleAmongMultipleCandidates),
            None,
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Disqualified
        );
        assert_eq!(
            candidates[0].reason,
            Some(WindowCaptureSelectionDisqualificationReason::EmptyTitleAmongMultipleCandidates)
        );
    }

    #[test]
    fn window_capture_selection_candidate_rejects_own_process_excluded_window() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Candidate,
            None,
            None,
            Some(WindowTitleFallbackStatus::NonEmptyTitle),
            Some(WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu),
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Disqualified
        );
        assert_eq!(
            candidates[0].reason,
            Some(WindowCaptureSelectionDisqualificationReason::OwnProcessExcludedFromWindowsMenu)
        );
    }

    #[test]
    fn window_capture_selection_candidate_marks_own_process_policy_unknown() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Candidate,
            None,
            None,
            Some(WindowTitleFallbackStatus::NonEmptyTitle),
            Some(WindowOwnProcessPolicyStatus::Unknown),
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Unknown
        );
        assert_eq!(
            candidates[0].reason,
            Some(WindowCaptureSelectionDisqualificationReason::OwnProcessPolicyUnknown)
        );
    }

    #[test]
    fn window_capture_selection_candidate_marks_missing_title_fallback_unknown_for_base_candidate()
    {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Candidate,
            None,
            None,
            None,
            None,
        )]);

        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Unknown
        );
        assert_eq!(
            candidates[0].reason,
            Some(WindowCaptureSelectionDisqualificationReason::SelectionMetadataIncomplete)
        );
    }

    #[test]
    fn window_capture_selection_candidate_preserves_input_length_and_order() {
        let candidates = window_capture_selection_candidates_v1(&[
            capture_selection_input(
                WindowCaptureCandidateStatus::Candidate,
                None,
                None,
                Some(WindowTitleFallbackStatus::NonEmptyTitle),
                None,
            ),
            capture_selection_input(
                WindowCaptureCandidateStatus::Candidate,
                None,
                Some(WindowDuplicateGroupStatus::Duplicate),
                Some(WindowTitleFallbackStatus::NonEmptyTitle),
                None,
            ),
        ]);

        assert_eq!(candidates.len(), 2);
        assert_eq!(
            candidates[0].status,
            WindowCaptureSelectionCandidateStatus::Candidate
        );
        assert_eq!(
            candidates[1].reason,
            Some(WindowCaptureSelectionDisqualificationReason::DuplicateWindow)
        );
    }

    #[test]
    fn window_capture_selection_candidate_serializes_camel_case_contract() {
        let candidates = window_capture_selection_candidates_v1(&[capture_selection_input(
            WindowCaptureCandidateStatus::Candidate,
            None,
            None,
            Some(WindowTitleFallbackStatus::NonEmptyTitle),
            Some(WindowOwnProcessPolicyStatus::ExcludedFromWindowsMenu),
        )]);
        let serialized = serde_json::to_value(&candidates[0]).expect("serialize candidate");

        assert_eq!(serialized["status"], "disqualified");
        assert_eq!(serialized["reason"], "ownProcessExcludedFromWindowsMenu");
        assert_eq!(
            serialized["selectionBasis"],
            "captureCandidateThenPreferredDuplicateThenTitleFallbackThenOwnProcessPolicy"
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
    fn window_observation_initializes_list_candidate_as_none() {
        assert_eq!(
            computer_use_window_observation_v1(&bounds(120, 90), true, 0, Some(1.0), Some(1))
                .list_candidate,
            None
        );
    }

    #[test]
    fn window_observation_initializes_capture_selection_candidate_as_none() {
        assert_eq!(
            computer_use_window_observation_v1(&bounds(120, 90), true, 0, Some(1.0), Some(1))
                .capture_selection_candidate,
            None
        );
    }

    #[test]
    fn window_observation_initializes_enumeration_context_as_none() {
        assert_eq!(
            computer_use_window_observation_v1(&bounds(120, 90), true, 0, Some(1.0), Some(1))
                .enumeration_context,
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

    fn capture_selection_input(
        capture_candidate_status: WindowCaptureCandidateStatus,
        capture_candidate_reason: Option<WindowDisqualificationReason>,
        duplicate_group_status: Option<WindowDuplicateGroupStatus>,
        title_fallback_status: Option<WindowTitleFallbackStatus>,
        own_process_window_policy_status: Option<WindowOwnProcessPolicyStatus>,
    ) -> WindowCaptureSelectionObservationInputV1 {
        WindowCaptureSelectionObservationInputV1 {
            capture_candidate_status,
            capture_candidate_reason,
            duplicate_group_status,
            title_fallback_status,
            own_process_window_policy_status,
        }
    }

    fn enumeration_input(
        option_on_screen_only: bool,
        option_all: bool,
        option_exclude_desktop_elements: bool,
        relative_to_window: u32,
    ) -> WindowEnumerationObservationInputV1 {
        WindowEnumerationObservationInputV1 {
            option_on_screen_only,
            option_all,
            option_exclude_desktop_elements,
            relative_to_window,
        }
    }
}
