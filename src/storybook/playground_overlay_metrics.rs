//! Shared overlay-metrics builders for confirm and context-picker playgrounds.
//!
//! Both playground renderers delegate placement geometry to these helpers
//! instead of embedding one-off `IntegratedOverlayPlacement::new(...)` literals.

use crate::ai::acp::popup_window::{dense_picker_width_for_labels, DENSE_PICKER_LEFT_MARGIN};

use super::{
    context_picker_popup_playground::{ContextPickerPopupSceneState, ContextPickerPopupTrigger},
    IntegratedOverlayAnchor, IntegratedOverlayPlacement, IntegratedSurfaceShellConfig,
};

// ---------------------------------------------------------------------------
// Confirm playground
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfirmPlaygroundOverlayMetrics {
    pub placement: IntegratedOverlayPlacement,
}

pub fn confirm_playground_overlay_metrics(
    shell: IntegratedSurfaceShellConfig,
) -> ConfirmPlaygroundOverlayMetrics {
    let width = (shell.width - 240.0).clamp(300.0, 360.0);
    let left = ((shell.width - width) / 2.0).round();
    let top = (shell.height - shell.footer_height - 134.0).max(112.0);

    tracing::info!(
        event = "storybook_confirm_overlay_metrics_built",
        shell_width = shell.width,
        shell_height = shell.height,
        overlay_left = left,
        overlay_top = top,
        overlay_width = width,
        "Built storybook confirm overlay metrics"
    );

    ConfirmPlaygroundOverlayMetrics {
        placement: IntegratedOverlayPlacement::new(
            IntegratedOverlayAnchor::Footer,
            left,
            top,
            width,
        ),
    }
}

// ---------------------------------------------------------------------------
// Context-picker playground
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextPickerPlaygroundOverlayMetrics {
    pub placement: IntegratedOverlayPlacement,
}

pub fn context_picker_playground_overlay_metrics<'a, I>(
    shell: IntegratedSurfaceShellConfig,
    trigger: ContextPickerPopupTrigger,
    state: ContextPickerPopupSceneState,
    show_synopsis: bool,
    labels: I,
) -> ContextPickerPlaygroundOverlayMetrics
where
    I: IntoIterator<Item = &'a str>,
{
    let measured_width = dense_picker_width_for_labels(shell.width, labels, show_synopsis);
    let width = (measured_width + if show_synopsis { 32.0 } else { 20.0 }).clamp(280.0, 340.0);

    let left = match trigger {
        ContextPickerPopupTrigger::Mention => DENSE_PICKER_LEFT_MARGIN + 84.0,
        ContextPickerPopupTrigger::Slash => DENSE_PICKER_LEFT_MARGIN + 68.0,
    };

    let top = match state {
        ContextPickerPopupSceneState::Results | ContextPickerPopupSceneState::Loading => 118.0,
        ContextPickerPopupSceneState::Empty | ContextPickerPopupSceneState::Error => 122.0,
    };

    tracing::info!(
        event = "storybook_context_picker_overlay_metrics_built",
        shell_width = shell.width,
        shell_height = shell.height,
        overlay_left = left,
        overlay_top = top,
        overlay_width = width,
        trigger = match trigger {
            ContextPickerPopupTrigger::Mention => "mention",
            ContextPickerPopupTrigger::Slash => "slash",
        },
        state = match state {
            ContextPickerPopupSceneState::Results => "results",
            ContextPickerPopupSceneState::Loading => "loading",
            ContextPickerPopupSceneState::Empty => "empty",
            ContextPickerPopupSceneState::Error => "error",
        },
        show_synopsis,
        "Built storybook context-picker overlay metrics"
    );

    ContextPickerPlaygroundOverlayMetrics {
        placement: IntegratedOverlayPlacement::new(
            IntegratedOverlayAnchor::Composer,
            left,
            top,
            width,
        ),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_metrics_match_expected_values() {
        let shell = IntegratedSurfaceShellConfig {
            width: 560.0,
            height: 320.0,
            ..Default::default()
        };
        let metrics = confirm_playground_overlay_metrics(shell);
        assert_eq!(metrics.placement.left, 120.0);
        assert_eq!(metrics.placement.top, 150.0);
        assert_eq!(metrics.placement.width, 320.0);
    }

    #[test]
    fn context_picker_metrics_left_matches_trigger() {
        let shell = IntegratedSurfaceShellConfig {
            width: 560.0,
            height: 300.0,
            ..Default::default()
        };
        let mention = context_picker_playground_overlay_metrics(
            shell,
            ContextPickerPopupTrigger::Mention,
            ContextPickerPopupSceneState::Results,
            true,
            ["Screenshot", "Selection", "Browser URL", "Git Diff"],
        );
        assert_eq!(mention.placement.left, 92.0);
        assert_eq!(mention.placement.top, 118.0);
        assert!((280.0..=340.0).contains(&mention.placement.width));

        let slash = context_picker_playground_overlay_metrics(
            shell,
            ContextPickerPopupTrigger::Slash,
            ContextPickerPopupSceneState::Results,
            true,
            [
                "Current Context",
                "Full Context",
                "Browser URL",
                "Focused Window",
            ],
        );
        assert_eq!(slash.placement.left, 76.0);
        assert_eq!(slash.placement.top, 118.0);
    }

    #[test]
    fn context_picker_empty_state_shifts_top() {
        let shell = IntegratedSurfaceShellConfig::default();
        let empty = context_picker_playground_overlay_metrics(
            shell,
            ContextPickerPopupTrigger::Mention,
            ContextPickerPopupSceneState::Empty,
            false,
            ["a"],
        );
        assert_eq!(empty.placement.top, 122.0);
    }
}
