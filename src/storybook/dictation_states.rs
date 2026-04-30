use std::time::Duration;

use gpui::*;

use crate::dictation::{
    render_dictation_overlay_state_preview, DictationOverlayState, DictationSessionPhase,
    DictationTarget,
};
use crate::storybook::{story_container, story_section, StoryVariant};
use crate::theme::{get_cached_theme, AppChromeColors};
use crate::ui_foundation::HexColorExt;

const ACTIVE_BARS: [f32; 9] = [0.10, 0.22, 0.42, 0.76, 1.0, 0.78, 0.44, 0.24, 0.12];
const QUIET_BARS: [f32; 9] = [0.06, 0.08, 0.10, 0.08, 0.06, 0.08, 0.10, 0.08, 0.06];
const SILENT_BARS: [f32; 9] = [0.08; 9];

#[derive(Clone, Copy, Debug, PartialEq)]
struct DictationCanonicalStateSpec {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    phase: DictationCanonicalPhase,
    target: DictationTarget,
    elapsed_secs: u64,
    bars: [f32; 9],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DictationCanonicalPhase {
    Idle,
    Recording,
    Confirming,
    Transcribing,
    Finished,
    Failed,
}

impl DictationCanonicalPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Recording => "recording",
            Self::Confirming => "confirming",
            Self::Transcribing => "transcribing",
            Self::Finished => "finished",
            Self::Failed => "failed",
        }
    }

    fn session_phase(self) -> DictationSessionPhase {
        match self {
            Self::Idle => DictationSessionPhase::Idle,
            Self::Recording => DictationSessionPhase::Recording,
            Self::Confirming => DictationSessionPhase::Confirming,
            Self::Transcribing => DictationSessionPhase::Transcribing,
            Self::Finished => DictationSessionPhase::Finished,
            Self::Failed => DictationSessionPhase::Failed("No speech detected".into()),
        }
    }
}

impl DictationCanonicalStateSpec {
    fn state(self) -> DictationOverlayState {
        DictationOverlayState {
            phase: self.phase.session_phase(),
            elapsed: Duration::from_secs(self.elapsed_secs),
            bars: self.bars,
            transcript: SharedString::default(),
            target: self.target,
        }
    }
}

const SPECS: [DictationCanonicalStateSpec; 10] = [
    DictationCanonicalStateSpec {
        id: "idle-hidden",
        name: "Idle / Hidden",
        description: "No overlay window is visible before dictation starts or after it closes.",
        phase: DictationCanonicalPhase::Idle,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 0,
        bars: [0.0; 9],
    },
    DictationCanonicalStateSpec {
        id: "listening-quiet",
        name: "Listening Quiet",
        description: "Recording with low input; bars stay neutral instead of implying failure.",
        phase: DictationCanonicalPhase::Recording,
        target: DictationTarget::NotesEditor,
        elapsed_secs: 12,
        bars: QUIET_BARS,
    },
    DictationCanonicalStateSpec {
        id: "active-speech",
        name: "Active Speech",
        description: "Recording with speech present; waveform uses the live success color.",
        phase: DictationCanonicalPhase::Recording,
        target: DictationTarget::NotesEditor,
        elapsed_secs: 39,
        bars: ACTIVE_BARS,
    },
    DictationCanonicalStateSpec {
        id: "target-script-kit",
        name: "Script Kit Target",
        description: "Recording targeted at the launcher filter with the Script Kit badge.",
        phase: DictationCanonicalPhase::Recording,
        target: DictationTarget::MainWindowFilter,
        elapsed_secs: 28,
        bars: ACTIVE_BARS,
    },
    DictationCanonicalStateSpec {
        id: "target-acp",
        name: "ACP Target",
        description: "Recording targeted at ACP handoff with the ACP badge.",
        phase: DictationCanonicalPhase::Recording,
        target: DictationTarget::TabAiHarness,
        elapsed_secs: 31,
        bars: ACTIVE_BARS,
    },
    DictationCanonicalStateSpec {
        id: "target-external-app",
        name: "External App Target",
        description: "Recording targeted at the frontmost external app fallback.",
        phase: DictationCanonicalPhase::Recording,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 35,
        bars: ACTIVE_BARS,
    },
    DictationCanonicalStateSpec {
        id: "confirming-stop",
        name: "Stop Confirmation",
        description: "Long-running recording after Escape, with Stop and Continue controls.",
        phase: DictationCanonicalPhase::Confirming,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 68,
        bars: ACTIVE_BARS,
    },
    DictationCanonicalStateSpec {
        id: "transcribing",
        name: "Transcribing",
        description: "Post-recording transcription state using the live three-dot pulse geometry.",
        phase: DictationCanonicalPhase::Transcribing,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 74,
        bars: SILENT_BARS,
    },
    DictationCanonicalStateSpec {
        id: "finished",
        name: "Finished",
        description: "Completed overlay state before automatic close.",
        phase: DictationCanonicalPhase::Finished,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 76,
        bars: SILENT_BARS,
    },
    DictationCanonicalStateSpec {
        id: "error",
        name: "Error",
        description: "Failure state for no speech or transcription errors.",
        phase: DictationCanonicalPhase::Failed,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 18,
        bars: SILENT_BARS,
    },
];

pub fn dictation_state_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id, spec.name)
                .description(spec.description)
                .with_prop("surface", "dictationOverlay")
                .with_prop("representation", "liveSurface")
                .with_prop("phase", spec.phase.label())
                .with_prop("target", spec.target.overlay_label())
                .with_prop("variantId", spec.id)
        })
        .collect()
}

pub fn render_dictation_state_story_preview(stable_id: &str) -> AnyElement {
    render_state_stage(resolve_spec(stable_id).unwrap_or(SPECS[0]), false)
}

pub fn render_dictation_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_state_stage(resolve_spec(stable_id).unwrap_or(SPECS[0]), true)
}

pub fn render_dictation_state_gallery() -> AnyElement {
    let theme = get_cached_theme();
    let opacity = theme.get_opacity();
    let chrome = AppChromeColors::from_theme(&theme);

    let mut root = story_container().gap(px(18.0)).child(
        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.colors.text.primary.with_opacity(opacity.text_strong))
                    .child("Dictation States"),
            )
            .child(
                div()
                    .text_xs()
                    .max_w(px(720.0))
                    .text_color(
                        theme
                            .colors
                            .text
                            .primary
                            .with_opacity(opacity.text_muted_alpha),
                    )
                    .child("Canonical live-capsule snapshots for the dictation overlay lifecycle."),
            ),
    );

    let mut section = story_section("Overlay Lifecycle").gap(px(10.0));
    for spec in SPECS {
        section = section.child(render_gallery_item(spec));
    }
    root = root.child(
        section
            .border_t_1()
            .border_color(rgba(chrome.divider_rgba))
            .pt(px(12.0)),
    );

    root.into_any_element()
}

fn resolve_spec(stable_id: &str) -> Option<DictationCanonicalStateSpec> {
    SPECS.iter().copied().find(|spec| spec.id == stable_id)
}

fn render_gallery_item(spec: DictationCanonicalStateSpec) -> AnyElement {
    let theme = get_cached_theme();
    let opacity = theme.get_opacity();
    let chrome = AppChromeColors::from_theme(&theme);

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .bg(rgba(chrome.surface_rgba))
        .border_1()
        .border_color(rgba(chrome.divider_rgba))
        .rounded(px(8.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.colors.text.primary.with_opacity(opacity.text_strong))
                        .child(spec.name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.colors.text.primary.with_opacity(opacity.text_hint))
                        .child(spec.description),
                ),
        )
        .child(render_state_stage(spec, false))
        .into_any_element()
}

fn render_state_stage(spec: DictationCanonicalStateSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let stage_height = if compact { 92.0 } else { 118.0 };

    div()
        .w_full()
        .h(px(stage_height))
        .flex()
        .justify_center()
        .items_center()
        .p(px(if compact { 6.0 } else { 8.0 }))
        .bg(rgba(chrome.preview_surface_rgba))
        .border_1()
        .border_color(rgba(chrome.divider_rgba))
        .rounded(px(8.0))
        .child(render_dictation_overlay_state_preview(&spec.state()))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{dictation_state_story_variants, render_dictation_state_story_preview, SPECS};

    #[test]
    fn dictation_state_story_exposes_canonical_lifecycle_states() {
        assert_eq!(dictation_state_story_variants().len(), 10);
        assert_eq!(SPECS.len(), 10);
    }

    #[test]
    fn dictation_state_variants_use_live_surface_representation() {
        for variant in dictation_state_story_variants() {
            assert_eq!(
                variant.props.get("representation").map(String::as_str),
                Some("liveSurface")
            );
        }
    }

    #[test]
    fn dictation_state_preview_falls_back_to_idle() {
        let _ = render_dictation_state_story_preview("does-not-exist");
    }
}
