use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::dictation::DictationTarget;
use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_section, StoryVariant};
use crate::theme::get_cached_theme;
use crate::theme::opacity::{OPACITY_ACTIVE, OPACITY_SELECTED};
use crate::ui_foundation::HexColorExt;

const GLASS_BG_OPACITY: f32 = 0.24;
const GLASS_BORDER_OPACITY: f32 = 0.18;
const OVERLAY_WIDTH_PX: f32 = 392.0;
const OVERLAY_HEIGHT_PX: f32 = 40.0;
const OVERLAY_RADIUS_PX: f32 = 20.0;
const OVERLAY_HORIZONTAL_PADDING_PX: f32 = 16.0;
const OVERLAY_CONTENT_GAP_PX: f32 = 12.0;
const STATUS_TEXT_SIZE_PX: f32 = 11.5;
const TIMER_SPACER_WIDTH_PX: f32 = 32.0;
const TARGET_BADGE_SLOT_WIDTH_PX: f32 = 72.0;
const WAVEFORM_BAR_COUNT: usize = 9;
const WAVEFORM_BAR_WIDTH_PX: f32 = 4.0;
const WAVEFORM_BAR_GAP_PX: f32 = 4.0;
const WAVEFORM_BAR_MIN_HEIGHT_PX: f32 = 4.0;
const WAVEFORM_BAR_MAX_HEIGHT_PX: f32 = 20.0;
const TRANSCRIBING_DOT_COUNT: usize = 3;
const TRANSCRIBING_DOT_SIZE_PX: f32 = 4.0;
const TRANSCRIBING_DOT_GAP_PX: f32 = 4.0;
const SOUND_THRESHOLD: f32 = 0.10;
const SILENT_BARS: [f32; WAVEFORM_BAR_COUNT] = [0.08; WAVEFORM_BAR_COUNT];
const TRANSCRIBING_PULSE_PERIOD_SECS: f64 = 1.4;
const TRANSCRIBING_PULSE_STAGGER_SECS: f64 = 0.2;
const PULSE_OPACITY_MIN: f32 = 0.3;
const PULSE_OPACITY_MAX: f32 = 1.0;

fn format_elapsed(elapsed: std::time::Duration) -> SharedString {
    let elapsed_secs = elapsed.as_secs();
    format!("{}:{:02}", elapsed_secs / 60, elapsed_secs % 60).into()
}

fn waveform_bar_opacity(level: f32) -> f32 {
    (level.clamp(0.0, 1.0) * 1.5).clamp(0.3, 1.0)
}

fn waveform_bar_height(level: f32) -> f32 {
    (WAVEFORM_BAR_MIN_HEIGHT_PX
        + level.clamp(0.0, 1.0).powf(0.7)
            * (WAVEFORM_BAR_MAX_HEIGHT_PX - WAVEFORM_BAR_MIN_HEIGHT_PX))
        .min(WAVEFORM_BAR_MAX_HEIGHT_PX)
}

fn has_sound(bars: &[f32; WAVEFORM_BAR_COUNT]) -> bool {
    bars.iter().any(|&bar| bar > SOUND_THRESHOLD)
}

fn transcribing_dot_opacities_at(elapsed_secs: f64) -> [f32; TRANSCRIBING_DOT_COUNT] {
    let mut opacities = [0.0_f32; TRANSCRIBING_DOT_COUNT];
    for (i, opacity) in opacities.iter_mut().enumerate() {
        let phase = elapsed_secs - (i as f64 * TRANSCRIBING_PULSE_STAGGER_SECS);
        let t = std::f64::consts::TAU * phase / TRANSCRIBING_PULSE_PERIOD_SECS;
        let wave = 0.5 + 0.5 * t.sin();
        *opacity = PULSE_OPACITY_MIN + (PULSE_OPACITY_MAX - PULSE_OPACITY_MIN) * wave as f32;
    }
    opacities
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreviewPhase {
    Recording,
    Transcribing,
    Confirming,
    Finished,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreviewLayout {
    Standard,
    DualRail,
    Centered,
    DockedBadge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WaveformKind {
    Bars,
    ThinBars,
    Ribbon,
    Thread,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BadgeTone {
    Whisper,
    Present,
    Minimal,
    Docked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SilenceTone {
    Error,
    Neutral,
    Dim,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AccentMode {
    None,
    GoldNeedle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DictationUiVariationSpec {
    id: &'static str,
    name: &'static str,
    family: &'static str,
    description: &'static str,
    phase: PreviewPhase,
    target: DictationTarget,
    elapsed_secs: u64,
    bars: [f32; WAVEFORM_BAR_COUNT],
    layout: PreviewLayout,
    waveform: WaveformKind,
    badge_tone: BadgeTone,
    silence_tone: SilenceTone,
    accent_mode: AccentMode,
    width: f32,
    height: f32,
    horizontal_padding: f32,
    content_gap: f32,
    surface_opacity: f32,
    border_opacity: f32,
    timer_opacity: f32,
    content_opacity: f32,
    status_text: &'static str,
    show_badge: bool,
}

const ACTIVE_BARS: [f32; WAVEFORM_BAR_COUNT] =
    [0.10, 0.22, 0.42, 0.76, 1.0, 0.78, 0.44, 0.24, 0.12];
const QUIET_BARS: [f32; WAVEFORM_BAR_COUNT] =
    [0.06, 0.10, 0.16, 0.24, 0.32, 0.24, 0.16, 0.10, 0.06];
const WIDE_BARS: [f32; WAVEFORM_BAR_COUNT] = [0.20, 0.34, 0.48, 0.72, 0.94, 0.72, 0.48, 0.34, 0.20];
const NEEDLE_BARS: [f32; WAVEFORM_BAR_COUNT] =
    [0.08, 0.14, 0.24, 0.40, 0.92, 0.40, 0.24, 0.14, 0.08];
const RIBBON_BARS: [f32; WAVEFORM_BAR_COUNT] =
    [0.18, 0.28, 0.38, 0.52, 0.60, 0.52, 0.38, 0.28, 0.18];

const SPECS: [DictationUiVariationSpec; 21] = [
    DictationUiVariationSpec {
        id: "stock-tightened",
        name: "Stock Tightened",
        family: "Quiet",
        description: "Current dictation pill with tighter spacing and quieter chrome.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 44,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Error,
        accent_mode: AccentMode::None,
        width: 384.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 14.0,
        content_gap: 10.0,
        surface_opacity: 0.20,
        border_opacity: 0.14,
        timer_opacity: OPACITY_SELECTED,
        content_opacity: OPACITY_ACTIVE,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "timer-forward",
        name: "Timer Forward",
        family: "Quiet",
        description: "Promotes the timer to primary weight while keeping the waveform secondary.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::NotesEditor,
        elapsed_secs: 68,
        bars: QUIET_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::ThinBars,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: 1.0,
        content_opacity: 0.72,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "badge-whisper",
        name: "Badge Whisper",
        family: "Quiet",
        description: "Leaves the destination badge nearly text-like until the user cycles targets.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 31,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Error,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: 0.22,
        border_opacity: 0.12,
        timer_opacity: OPACITY_SELECTED,
        content_opacity: OPACITY_ACTIVE,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "ghost-glass",
        name: "Ghost Glass",
        family: "Quiet",
        description: "Pushes the surface closer to vibrancy with barely-there fill and edge.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 52,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Error,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: 0.16,
        border_opacity: 0.10,
        timer_opacity: 0.66,
        content_opacity: 0.94,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "mono-everywhere",
        name: "Mono Everywhere",
        family: "Quiet",
        description: "Unifies timer, target, and status text under one terminal-like voice.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::MainWindowPrompt,
        elapsed_secs: 96,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::ThinBars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: 0.88,
        content_opacity: 0.88,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "flat-accent",
        name: "Flat Accent",
        family: "Quiet",
        description: "Relies on content and state accents more than on border definition.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::MainWindowFilter,
        elapsed_secs: 23,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::GoldNeedle,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 15.0,
        content_gap: 11.0,
        surface_opacity: 0.20,
        border_opacity: 0.06,
        timer_opacity: 0.72,
        content_opacity: 0.96,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "center-quiet",
        name: "Center Quiet",
        family: "Quiet",
        description: "Lets the center signal relax during pauses so the pill feels calmer.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::NotesEditor,
        elapsed_secs: 18,
        bars: QUIET_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::ThinBars,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Dim,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: 0.21,
        border_opacity: 0.14,
        timer_opacity: 0.62,
        content_opacity: 0.74,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "split-pill",
        name: "Split Pill",
        family: "Structural",
        description: "Uses spacing rhythm to make the timer, signal, and destination read faster.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 83,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Error,
        accent_mode: AccentMode::None,
        width: 408.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 18.0,
        content_gap: 16.0,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: OPACITY_SELECTED,
        content_opacity: OPACITY_ACTIVE,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "dual-rail",
        name: "Dual Rail",
        family: "Structural",
        description:
            "Stacks the target under the timer so the waveform gets more uninterrupted room.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 58,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::DualRail,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 404.0,
        height: 46.0,
        horizontal_padding: 16.0,
        content_gap: 12.0,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: 0.96,
        content_opacity: OPACITY_ACTIVE,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "symmetric-core",
        name: "Symmetric Core",
        family: "Structural",
        description: "Centers the timer above the signal for a more instrument-like reading mode.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 105,
        bars: QUIET_BARS,
        layout: PreviewLayout::Centered,
        waveform: WaveformKind::ThinBars,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 312.0,
        height: 64.0,
        horizontal_padding: 18.0,
        content_gap: 10.0,
        surface_opacity: 0.22,
        border_opacity: 0.14,
        timer_opacity: 0.96,
        content_opacity: 0.84,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "wide-meter",
        name: "Wide Meter",
        family: "Structural",
        description: "Expands the waveform so dictation feels more live than badge-oriented.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::MainWindowPrompt,
        elapsed_secs: 72,
        bars: WIDE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Error,
        accent_mode: AccentMode::None,
        width: 432.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 14.0,
        content_gap: 10.0,
        surface_opacity: 0.22,
        border_opacity: 0.16,
        timer_opacity: 0.66,
        content_opacity: 0.98,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "compact-capsule",
        name: "Compact Capsule",
        family: "Structural",
        description: "Shrinks toward a system HUD while preserving the same information order.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::NotesEditor,
        elapsed_secs: 39,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::ThinBars,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 348.0,
        height: 34.0,
        horizontal_padding: 12.0,
        content_gap: 10.0,
        surface_opacity: 0.22,
        border_opacity: 0.12,
        timer_opacity: 0.88,
        content_opacity: 0.90,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "tall-signal",
        name: "Tall Signal",
        family: "Structural",
        description: "Adds a bit more height so active dictation reads from peripheral vision.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 61,
        bars: WIDE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Error,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: 52.0,
        horizontal_padding: 16.0,
        content_gap: 12.0,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: 0.72,
        content_opacity: 1.0,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "badge-dock",
        name: "Badge Dock",
        family: "Structural",
        description: "Turns the destination into a docked end-cap instead of an internal chip.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 47,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::DockedBadge,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Docked,
        silence_tone: SilenceTone::Error,
        accent_mode: AccentMode::None,
        width: 414.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 14.0,
        content_gap: 8.0,
        surface_opacity: 0.22,
        border_opacity: 0.16,
        timer_opacity: 0.72,
        content_opacity: 0.96,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "green-ready",
        name: "Green Ready",
        family: "State and Signal",
        description: "Uses richer success emphasis while the session is clearly hearing speech.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 28,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: 0.72,
        content_opacity: 1.0,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "red-means-problem",
        name: "Red Means Problem",
        family: "State and Signal",
        description: "Keeps natural pauses neutral and saves red strictly for real errors.",
        phase: PreviewPhase::Failed,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 17,
        bars: SILENT_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: 0.22,
        border_opacity: 0.16,
        timer_opacity: 0.56,
        content_opacity: 0.92,
        status_text: "Mic access denied",
        show_badge: false,
    },
    DictationUiVariationSpec {
        id: "transcribe-thread",
        name: "Transcribe Thread",
        family: "State and Signal",
        description: "Uses a stitched pulse instead of three dots during transcription.",
        phase: PreviewPhase::Transcribing,
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 54,
        bars: RIBBON_BARS,
        layout: PreviewLayout::Centered,
        waveform: WaveformKind::Thread,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 304.0,
        height: 54.0,
        horizontal_padding: 16.0,
        content_gap: 8.0,
        surface_opacity: 0.22,
        border_opacity: 0.14,
        timer_opacity: 0.64,
        content_opacity: 0.90,
        status_text: "Transcribing",
        show_badge: false,
    },
    DictationUiVariationSpec {
        id: "transcript-flash",
        name: "Transcript Flash",
        family: "State and Signal",
        description: "Shows the first dictated words for a beat instead of a generic done state.",
        phase: PreviewPhase::Finished,
        target: DictationTarget::NotesEditor,
        elapsed_secs: 80,
        bars: SILENT_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 418.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 18.0,
        content_gap: 10.0,
        surface_opacity: 0.22,
        border_opacity: 0.16,
        timer_opacity: 0.52,
        content_opacity: 1.0,
        status_text: "drafted release notes and tagged risks...",
        show_badge: false,
    },
    DictationUiVariationSpec {
        id: "confirm-lock",
        name: "Confirm Lock",
        family: "State and Signal",
        description: "Makes stop or continue feel like a stronger temporary mode lock.",
        phase: PreviewPhase::Confirming,
        target: DictationTarget::ExternalApp,
        elapsed_secs: 87,
        bars: SILENT_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: 0.56,
        content_opacity: 1.0,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "gold-needle",
        name: "Gold Needle",
        family: "State and Signal",
        description: "Adds a restrained Script Kit gold tick inside the live signal cluster.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 41,
        bars: NEEDLE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Error,
        accent_mode: AccentMode::GoldNeedle,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: 0.22,
        border_opacity: 0.16,
        timer_opacity: 0.72,
        content_opacity: 1.0,
        status_text: "Done",
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "signal-ribbon",
        name: "Signal Ribbon",
        family: "State and Signal",
        description: "Collapses the live meter into a denser ribbon that feels more continuous.",
        phase: PreviewPhase::Recording,
        target: DictationTarget::MainWindowPrompt,
        elapsed_secs: 63,
        bars: RIBBON_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Ribbon,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 408.0,
        height: 36.0,
        horizontal_padding: 14.0,
        content_gap: 10.0,
        surface_opacity: 0.22,
        border_opacity: 0.14,
        timer_opacity: 0.72,
        content_opacity: 0.92,
        status_text: "Done",
        show_badge: true,
    },
];

pub fn dictation_ui_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id, spec.name)
                .description(spec.description)
                .with_prop("surface", "dictationOverlay")
                .with_prop("family", spec.family)
                .with_prop("variantId", spec.id)
        })
        .collect()
}

pub fn render_dictation_ui_story_preview(stable_id: &str) -> AnyElement {
    render_spec_stage(resolve_spec(stable_id).unwrap_or(SPECS[0]), false)
}

pub fn render_dictation_ui_compare_thumbnail(stable_id: &str) -> AnyElement {
    render_spec_stage(resolve_spec(stable_id).unwrap_or(SPECS[0]), true)
}

pub fn render_dictation_ui_gallery() -> AnyElement {
    let mut root = story_container()
        .gap_6()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(get_cached_theme().colors.text.tertiary))
                        .child("Dictation Overlay"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(get_cached_theme().colors.text.muted))
                        .child(
                            "Twenty-one storybook-only variations built from the current dictation pill geometry, waveform sizing, and phase states.",
                        ),
                ),
        );

    for family in ["Quiet", "Structural", "State and Signal"] {
        let mut section = story_section(family).gap(px(12.0));
        for spec in SPECS.iter().copied().filter(|spec| spec.family == family) {
            section = section.child(render_gallery_item(spec));
        }
        root = root.child(section);
    }

    root.into_any_element()
}

fn resolve_spec(stable_id: &str) -> Option<DictationUiVariationSpec> {
    SPECS.iter().copied().find(|spec| spec.id == stable_id)
}

fn render_gallery_item(spec: DictationUiVariationSpec) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .bg(theme.colors.background.title_bar.with_opacity(0.22))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.18))
        .rounded(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme.colors.text.primary))
                        .child(spec.name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.colors.text.muted))
                        .child(spec.description),
                ),
        )
        .child(render_spec_stage(spec, false))
        .into_any_element()
}

fn render_spec_stage(spec: DictationUiVariationSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let stage_height = if compact { 108.0 } else { 132.0 };
    let scale = if compact { 0.82 } else { 1.0 };

    div()
        .w_full()
        .h(px(stage_height))
        .flex()
        .justify_center()
        .items_end()
        .pb(px(if compact { 12.0 } else { 14.0 }))
        .bg(theme.colors.background.title_bar.with_opacity(0.45))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.12))
        .rounded(px(12.0))
        .child(render_overlay(spec, scale))
        .into_any_element()
}

fn render_overlay(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    let mut surface_bg = rgb(theme.colors.background.main);
    surface_bg.a = spec.surface_opacity;
    let mut border_color = rgb(theme.colors.ui.border);
    border_color.a = spec.border_opacity;

    let width = spec.width * scale;
    let height = spec.height * scale;
    let radius = (spec.height / OVERLAY_HEIGHT_PX) * OVERLAY_RADIUS_PX * scale;
    let padding_x = spec.horizontal_padding * scale;
    let gap = spec.content_gap * scale;

    div()
        .w(px(width))
        .h(px(height))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .px(px(padding_x))
        .gap(px(gap))
        .bg(surface_bg)
        .rounded(px(radius))
        .border_1()
        .border_color(border_color)
        .child(render_overlay_inner(spec, scale))
        .into_any_element()
}

fn render_overlay_inner(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let timer = render_timer(spec, scale);
    let content = render_phase_content(spec, scale);
    let badge = render_target_badge(spec, scale);

    match spec.layout {
        PreviewLayout::Standard => div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .child(timer)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(content),
            )
            .child(badge)
            .into_any_element(),
        PreviewLayout::DualRail => div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.0 * scale))
            .child(
                div()
                    .w(px((TIMER_SPACER_WIDTH_PX + 18.0) * scale))
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(2.0 * scale))
                    .child(timer)
                    .when(spec.show_badge, |d| {
                        d.child(render_dual_rail_badge(spec, scale))
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(content),
            )
            .into_any_element(),
        PreviewLayout::Centered => div()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(4.0 * scale))
            .child(timer)
            .child(content)
            .when(spec.show_badge, |d| {
                d.child(render_dual_rail_badge(spec, scale))
            })
            .into_any_element(),
        PreviewLayout::DockedBadge => div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0 * scale))
            .child(timer)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(content),
            )
            .when(spec.show_badge, |d| {
                d.child(
                    div()
                        .h_full()
                        .px(px(10.0 * scale))
                        .ml(px(2.0 * scale))
                        .bg(theme_badge_dock_bg())
                        .border_l_1()
                        .border_color(get_cached_theme().colors.ui.border.with_opacity(0.24))
                        .child(render_target_badge(spec, scale)),
                )
            })
            .into_any_element(),
    }
}

fn render_timer(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w(px(TIMER_SPACER_WIDTH_PX * scale))
        .text_size(px(STATUS_TEXT_SIZE_PX * scale))
        .font_family(FONT_MONO)
        .text_color(theme.colors.text.muted.with_opacity(spec.timer_opacity))
        .child(format_elapsed(std::time::Duration::from_secs(
            spec.elapsed_secs,
        )))
        .into_any_element()
}

fn render_phase_content(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    match spec.phase {
        PreviewPhase::Recording => render_waveform(spec, scale),
        PreviewPhase::Transcribing => render_transcribing(spec, scale),
        PreviewPhase::Confirming => render_confirming(spec, scale),
        PreviewPhase::Finished => render_status(spec.status_text, true, spec, scale),
        PreviewPhase::Failed => render_status(spec.status_text, false, spec, scale),
    }
}

fn render_waveform(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    let active = has_sound(&spec.bars);
    let base_hex = match spec.silence_tone {
        SilenceTone::Error => {
            if active {
                theme.colors.ui.success
            } else {
                theme.colors.ui.error
            }
        }
        SilenceTone::Neutral => {
            if active {
                theme.colors.ui.success
            } else {
                theme.colors.text.muted
            }
        }
        SilenceTone::Dim => {
            if active {
                theme.colors.ui.success
            } else {
                theme.colors.text.tertiary
            }
        }
    };

    let (bar_width, bar_gap) = match spec.waveform {
        WaveformKind::Bars => (WAVEFORM_BAR_WIDTH_PX * scale, WAVEFORM_BAR_GAP_PX * scale),
        WaveformKind::ThinBars => (3.0 * scale, 3.0 * scale),
        WaveformKind::Ribbon => (6.0 * scale, 1.5 * scale),
        WaveformKind::Thread => (2.5 * scale, 3.0 * scale),
    };

    let mut waveform = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(bar_gap))
        .h(px(WAVEFORM_BAR_MAX_HEIGHT_PX * scale));

    for (index, level) in spec.bars.iter().copied().enumerate() {
        let mut color = base_hex.with_opacity(waveform_bar_opacity(level) * spec.content_opacity);
        if matches!(spec.waveform, WaveformKind::Thread) {
            color = base_hex.with_opacity((0.45 + level * 0.35) * spec.content_opacity);
        }
        let height = match spec.waveform {
            WaveformKind::Thread => (WAVEFORM_BAR_MIN_HEIGHT_PX + level * 6.0) * scale,
            WaveformKind::Ribbon => (6.0 + level * 8.0) * scale,
            _ => waveform_bar_height(level) * scale,
        };

        waveform = waveform.child(
            div()
                .w(px(bar_width))
                .h(px(height))
                .min_h(px(WAVEFORM_BAR_MIN_HEIGHT_PX * scale))
                .bg(color)
                .rounded(px(bar_width.max(2.0))),
        );

        if matches!(spec.accent_mode, AccentMode::GoldNeedle) && index == (WAVEFORM_BAR_COUNT / 2) {
            waveform = waveform.child(
                div()
                    .w(px(1.5 * scale))
                    .h(px((WAVEFORM_BAR_MAX_HEIGHT_PX + 2.0) * scale))
                    .bg(rgb(theme.colors.accent.selected))
                    .rounded(px(999.0)),
            );
        }
    }

    waveform.into_any_element()
}

fn render_transcribing(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    if matches!(spec.waveform, WaveformKind::Thread) {
        return render_waveform(spec, scale);
    }

    let theme = get_cached_theme();
    let dot_opacities = transcribing_dot_opacities_at(0.55);
    let mut dots = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(TRANSCRIBING_DOT_GAP_PX * scale));

    for opacity in dot_opacities.iter().take(TRANSCRIBING_DOT_COUNT) {
        dots = dots.child(
            div()
                .w(px(TRANSCRIBING_DOT_SIZE_PX * scale))
                .h(px(TRANSCRIBING_DOT_SIZE_PX * scale))
                .bg(theme
                    .colors
                    .ui
                    .success
                    .with_opacity(*opacity * spec.content_opacity))
                .rounded(px(999.0)),
        );
    }

    dots.into_any_element()
}

fn render_confirming(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    let text_size = STATUS_TEXT_SIZE_PX * scale;

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0 * scale))
        .child(
            div()
                .px(px(8.0 * scale))
                .py(px(2.0 * scale))
                .bg(theme.colors.ui.error.with_opacity(0.14))
                .border_1()
                .border_color(theme.colors.ui.error.with_opacity(0.45))
                .rounded(px(999.0))
                .text_size(px(text_size))
                .font_family(FONT_MONO)
                .text_color(theme.colors.ui.error.with_opacity(OPACITY_ACTIVE))
                .child("Stop"),
        )
        .child(
            div()
                .px(px(8.0 * scale))
                .py(px(2.0 * scale))
                .bg(theme.colors.ui.success.with_opacity(0.08))
                .border_1()
                .border_color(theme.colors.ui.success.with_opacity(0.35))
                .rounded(px(999.0))
                .text_size(px(text_size))
                .font_family(FONT_MONO)
                .text_color(theme.colors.ui.success.with_opacity(spec.content_opacity))
                .child("Continue"),
        )
        .into_any_element()
}

fn render_status(
    text: &'static str,
    success: bool,
    spec: DictationUiVariationSpec,
    scale: f32,
) -> AnyElement {
    let theme = get_cached_theme();
    let color = if success {
        theme.colors.text.primary.with_opacity(spec.content_opacity)
    } else {
        theme.colors.ui.error.with_opacity(OPACITY_ACTIVE)
    };

    div()
        .max_w(px((spec.width - 48.0) * scale))
        .text_size(px(STATUS_TEXT_SIZE_PX * scale))
        .font_family(FONT_MONO)
        .text_color(color)
        .overflow_hidden()
        .child(text)
        .into_any_element()
}

fn render_target_badge(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    if !spec.show_badge {
        return div()
            .w(px(TARGET_BADGE_SLOT_WIDTH_PX * scale))
            .into_any_element();
    }

    let theme = get_cached_theme();
    let label = spec.target.overlay_label();
    let text_size = STATUS_TEXT_SIZE_PX * scale;
    let slot_width = TARGET_BADGE_SLOT_WIDTH_PX * scale;

    match spec.badge_tone {
        BadgeTone::Whisper => div()
            .w(px(slot_width))
            .flex()
            .justify_end()
            .text_size(px(text_size))
            .font_family(FONT_MONO)
            .text_color(theme.colors.text.muted.with_opacity(0.56))
            .child(label)
            .into_any_element(),
        BadgeTone::Minimal => div()
            .w(px(slot_width))
            .flex()
            .justify_end()
            .text_size(px(text_size))
            .font_family(FONT_MONO)
            .text_color(theme.colors.text.tertiary.with_opacity(0.74))
            .child(label.to_uppercase())
            .into_any_element(),
        BadgeTone::Present => div()
            .w(px(slot_width))
            .flex()
            .justify_end()
            .child(
                div()
                    .px(px(7.0 * scale))
                    .py(px(2.0 * scale))
                    .bg(theme.colors.background.title_bar.with_opacity(0.28))
                    .border_1()
                    .border_color(theme.colors.ui.border.with_opacity(0.22))
                    .rounded(px(999.0))
                    .text_size(px(text_size))
                    .font_family(FONT_MONO)
                    .text_color(theme.colors.text.muted.with_opacity(0.80))
                    .child(label),
            )
            .into_any_element(),
        BadgeTone::Docked => div()
            .flex()
            .items_center()
            .h_full()
            .text_size(px(text_size))
            .font_family(FONT_MONO)
            .text_color(theme.colors.text.primary.with_opacity(0.90))
            .child(label)
            .into_any_element(),
    }
}

fn render_dual_rail_badge(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .text_size(px((STATUS_TEXT_SIZE_PX - 1.0) * scale))
        .font_family(FONT_MONO)
        .text_color(theme.colors.text.tertiary.with_opacity(0.82))
        .child(spec.target.overlay_label())
        .into_any_element()
}

fn theme_badge_dock_bg() -> Hsla {
    get_cached_theme()
        .colors
        .background
        .title_bar
        .with_opacity(0.26)
}

#[cfg(test)]
mod tests {
    use super::{dictation_ui_story_variants, render_dictation_ui_story_preview, SPECS};

    #[test]
    fn dictation_ui_story_exposes_twenty_one_variants() {
        assert_eq!(dictation_ui_story_variants().len(), 21);
        assert_eq!(SPECS.len(), 21);
    }

    #[test]
    fn dictation_ui_variant_ids_are_unique() {
        let mut ids: Vec<_> = dictation_ui_story_variants()
            .into_iter()
            .map(|variant| variant.stable_id())
            .collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 21);
    }

    #[test]
    fn dictation_story_preview_falls_back_to_first_variant() {
        let _ = render_dictation_ui_story_preview("does-not-exist");
    }
}
