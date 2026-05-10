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
}
