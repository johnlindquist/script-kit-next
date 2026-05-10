use crate::computer_use::runtime_bridge::ComputerUseAppWindowInfo;
use crate::computer_use::window_observation::{
    WindowCaptureSelectionCandidateStatus, WindowCaptureSelectionDisqualificationReason,
};

#[derive(Clone, Debug, PartialEq)]
pub enum NativeWindowCaptureSelectionError {
    WindowNotFound,
    AmbiguousNativeWindowRows {
        candidate_count: usize,
    },
    NotCaptureCandidate {
        status: String,
        reason: Option<String>,
    },
    MissingObservation,
    MissingCaptureSelectionCandidate,
}

pub fn select_capture_candidate_for_native_window(
    windows: &[ComputerUseAppWindowInfo],
    native_window_id: u32,
) -> Result<ComputerUseAppWindowInfo, NativeWindowCaptureSelectionError> {
    let matches = windows
        .iter()
        .filter(|window| window.native_window_id == native_window_id)
        .collect::<Vec<_>>();

    if matches.is_empty() {
        return Err(NativeWindowCaptureSelectionError::WindowNotFound);
    }

    let candidates = matches
        .iter()
        .copied()
        .filter(|window| is_capture_selection_candidate(window))
        .collect::<Vec<_>>();

    match candidates.len() {
        1 => Ok(candidates[0].clone()),
        count if count > 1 => Err(
            NativeWindowCaptureSelectionError::AmbiguousNativeWindowRows {
                candidate_count: count,
            },
        ),
        _ => Err(first_rejection(matches[0])),
    }
}

fn is_capture_selection_candidate(window: &ComputerUseAppWindowInfo) -> bool {
    window
        .observation
        .as_ref()
        .and_then(|observation| observation.capture_selection_candidate.as_ref())
        .is_some_and(|candidate| {
            candidate.status == WindowCaptureSelectionCandidateStatus::Candidate
        })
}

fn first_rejection(window: &ComputerUseAppWindowInfo) -> NativeWindowCaptureSelectionError {
    let Some(observation) = window.observation.as_ref() else {
        return NativeWindowCaptureSelectionError::MissingObservation;
    };
    let Some(candidate) = observation.capture_selection_candidate.as_ref() else {
        return NativeWindowCaptureSelectionError::MissingCaptureSelectionCandidate;
    };

    NativeWindowCaptureSelectionError::NotCaptureCandidate {
        status: format!("{:?}", candidate.status),
        reason: candidate.reason.as_ref().map(selection_reason_slug),
    }
}

fn selection_reason_slug(reason: &WindowCaptureSelectionDisqualificationReason) -> String {
    format!("{reason:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computer_use::window_observation::{
        computer_use_window_observation_v1, window_capture_selection_candidates_v1,
        WindowCaptureCandidateStatus, WindowCaptureSelectionBasis,
        WindowCaptureSelectionCandidateV1, WindowCaptureSelectionObservationInputV1,
        WindowDisqualificationReason,
    };
    use crate::protocol::TargetWindowBounds;

    fn window(id: u32, status: WindowCaptureSelectionCandidateStatus) -> ComputerUseAppWindowInfo {
        let bounds = TargetWindowBounds {
            x: 0,
            y: 0,
            width: 500,
            height: 400,
        };
        let mut observation =
            computer_use_window_observation_v1(&bounds, true, 0, Some(1.0), Some(1));
        observation.capture_selection_candidate = Some(WindowCaptureSelectionCandidateV1 {
            status,
            reason: None,
            selection_basis:
                WindowCaptureSelectionBasis::CaptureCandidateThenPreferredDuplicateThenTitleFallbackThenOwnProcessPolicy,
        });

        ComputerUseAppWindowInfo {
            native_window_id: id,
            title: Some("Window".to_string()),
            bounds,
            is_on_screen: true,
            layer: 0,
            z_order: 0,
            observation: Some(observation),
        }
    }

    #[test]
    fn selects_exact_candidate_id() {
        let windows = vec![
            window(10, WindowCaptureSelectionCandidateStatus::Candidate),
            window(11, WindowCaptureSelectionCandidateStatus::Candidate),
        ];

        let selected =
            select_capture_candidate_for_native_window(&windows, 11).expect("candidate selected");

        assert_eq!(selected.native_window_id, 11);
    }

    #[test]
    fn rejects_missing_exact_id() {
        let windows = vec![window(10, WindowCaptureSelectionCandidateStatus::Candidate)];

        let error = select_capture_candidate_for_native_window(&windows, 99)
            .expect_err("missing window should fail");

        assert_eq!(error, NativeWindowCaptureSelectionError::WindowNotFound);
    }

    #[test]
    fn rejects_non_candidate_with_reason() {
        let bounds = TargetWindowBounds {
            x: 0,
            y: 0,
            width: 500,
            height: 400,
        };
        let mut observation =
            computer_use_window_observation_v1(&bounds, true, 1, Some(1.0), Some(1));
        observation.capture_selection_candidate = Some(
            window_capture_selection_candidates_v1(&[WindowCaptureSelectionObservationInputV1 {
                capture_candidate_status: WindowCaptureCandidateStatus::Disqualified,
                capture_candidate_reason: Some(WindowDisqualificationReason::LayerNonZero),
                duplicate_group_status: None,
                title_fallback_status: None,
                own_process_window_policy_status: None,
            }])
            .remove(0),
        );
        let windows = vec![ComputerUseAppWindowInfo {
            native_window_id: 10,
            title: Some("Window".to_string()),
            bounds,
            is_on_screen: true,
            layer: 1,
            z_order: 0,
            observation: Some(observation),
        }];

        let error = select_capture_candidate_for_native_window(&windows, 10)
            .expect_err("non-candidate should fail");

        assert_eq!(
            error,
            NativeWindowCaptureSelectionError::NotCaptureCandidate {
                status: "Disqualified".to_string(),
                reason: Some("LayerNonZero".to_string()),
            }
        );
    }

    #[test]
    fn rejects_ambiguous_candidate_rows_for_same_id() {
        let windows = vec![
            window(10, WindowCaptureSelectionCandidateStatus::Candidate),
            window(10, WindowCaptureSelectionCandidateStatus::Candidate),
        ];

        let error = select_capture_candidate_for_native_window(&windows, 10)
            .expect_err("ambiguous candidates should fail");

        assert_eq!(
            error,
            NativeWindowCaptureSelectionError::AmbiguousNativeWindowRows { candidate_count: 2 }
        );
    }
}
