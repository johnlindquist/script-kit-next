use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::dictation::DictationTarget;
use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_section, StoryVariant};
use crate::theme::opacity::{OPACITY_ACTIVE, OPACITY_SELECTED};
use crate::theme::{get_cached_theme, AppChromeColors, Theme};
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
const SOUND_THRESHOLD: f32 = 0.10;
const SILENT_BARS: [f32; WAVEFORM_BAR_COUNT] = [0.08; WAVEFORM_BAR_COUNT];

fn semantic_text(theme: &Theme, opacity: f32) -> Hsla {
    theme
        .colors
        .text
        .primary
        .with_opacity(opacity.clamp(0.0, 1.0))
}

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

fn uses_native_rim(spec: DictationUiVariationSpec) -> bool {
    spec.id == "compact-capsule"
}

fn overlay_radius_for_spec(spec: DictationUiVariationSpec, height: f32, scale: f32) -> f32 {
    let radius = match spec.id {
        "compact-capsule" | "timer-forward" | "micro-dots" => height / 2.0,
        "badge-dock" | "timer-sidecar" | "launcher-command-strip" => 7.0,
        "notes-shelf" | "symmetric-core" | "transcribe-thread" => 14.0,
        "signal-ribbon" | "red-means-problem" | "center-quiet" => 5.0,
        "tall-signal" | "dual-rail" => 11.0,
        "flat-accent" | "gold-needle" => 3.0,
        _ => (spec.height / OVERLAY_HEIGHT_PX) * OVERLAY_RADIUS_PX,
    };
    radius * scale
}

fn min_overlay_height_for_spec(spec: DictationUiVariationSpec) -> f32 {
    match spec.layout {
        PreviewLayout::Standard | PreviewLayout::DockedBadge => spec.height.max(72.0),
        PreviewLayout::DualRail | PreviewLayout::Sidecar => spec.height.max(76.0),
        PreviewLayout::Centered | PreviewLayout::TopRail | PreviewLayout::BottomRail => {
            spec.height.max(88.0)
        }
    }
}

fn overlay_vertical_padding_for_spec(spec: DictationUiVariationSpec) -> f32 {
    match spec.layout {
        PreviewLayout::Standard | PreviewLayout::DockedBadge => 8.0,
        PreviewLayout::DualRail | PreviewLayout::Sidecar => 9.0,
        PreviewLayout::Centered | PreviewLayout::TopRail | PreviewLayout::BottomRail => 10.0,
    }
}

fn storybook_dictation_stop_keycap() -> String {
    crate::config::load_config()
        .get_dictation_hotkey()
        .map(|hotkey| hotkey.to_display_string().replace("Semicolon", ";"))
        .filter(|key| !key.trim().is_empty())
        .unwrap_or_else(|| "⇧⌘;".to_string())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreviewLayout {
    Standard,
    DualRail,
    Centered,
    DockedBadge,
    TopRail,
    BottomRail,
    Sidecar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WaveformKind {
    Bars,
    ThinBars,
    Ribbon,
    Thread,
    Dots,
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

const SPECS: [DictationUiVariationSpec; 25] = [
    DictationUiVariationSpec {
        id: "compact-capsule",
        name: "Compact Capsule",
        family: "Shape Rhythm",
        description:
            "Compact standalone capsule with a neutral native rim like the main menu edge.",
        target: DictationTarget::NotesEditor,
        elapsed_secs: 39,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::ThinBars,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 312.0,
        height: 32.0,
        horizontal_padding: 11.0,
        content_gap: 8.0,
        surface_opacity: 0.20,
        border_opacity: 0.36,
        timer_opacity: 0.78,
        content_opacity: 0.94,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "timer-forward",
        name: "Timer Lead",
        family: "Default Surface",
        description: "Keeps elapsed time readable with the same restrained weight as metadata.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "badge-whisper",
        name: "Destination Whisper",
        family: "Default Surface",
        description: "Keeps the destination as quiet text instead of a button-like chip.",
        target: DictationTarget::ExternalApp,
        elapsed_secs: 31,
        bars: ACTIVE_BARS,
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
        border_opacity: 0.12,
        timer_opacity: OPACITY_SELECTED,
        content_opacity: OPACITY_ACTIVE,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "ghost-glass",
        name: "Glass Bar",
        family: "Default Surface",
        description: "Uses a lighter vibrancy read so the standalone surface separates cleanly.",
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 52,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: 0.16,
        border_opacity: 0.10,
        timer_opacity: 0.66,
        content_opacity: 0.94,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "mono-everywhere",
        name: "Utility Readout",
        family: "Default Surface",
        description:
            "Keeps timer and target utility text monospaced on the launcher opacity ladder.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "flat-accent",
        name: "Accent Needle",
        family: "Default Surface",
        description: "Adds one restrained accent tick without changing the overlay into a control.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "center-quiet",
        name: "Pause Quiet",
        family: "Default Surface",
        description: "Treats idle input as muted signal instead of an error-like state.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "split-pill",
        name: "Balanced Columns",
        family: "Shape Rhythm",
        description: "Balances timer, waveform, and destination inside one standalone bar.",
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 83,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 408.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 18.0,
        content_gap: 16.0,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: OPACITY_SELECTED,
        content_opacity: OPACITY_ACTIVE,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "dual-rail",
        name: "Left Rail",
        family: "Shape Rhythm",
        description: "Stacks timer and target at left so the meter remains visually centered.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "symmetric-core",
        name: "Center Stack",
        family: "Shape Rhythm",
        description: "Tests a compact centered overlay with vertical information hierarchy.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "wide-meter",
        name: "Wide Signal",
        family: "Shape Rhythm",
        description: "Gives speech signal more width while destination remains secondary.",
        target: DictationTarget::MainWindowPrompt,
        elapsed_secs: 72,
        bars: WIDE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 432.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 14.0,
        content_gap: 10.0,
        surface_opacity: 0.22,
        border_opacity: 0.16,
        timer_opacity: 0.66,
        content_opacity: 0.98,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "stock-tightened",
        name: "Default Quiet",
        family: "Default Surface",
        description: "Refines the standalone pill with launcher text tiers and quieter chrome.",
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 44,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 384.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 14.0,
        content_gap: 10.0,
        surface_opacity: 0.20,
        border_opacity: 0.14,
        timer_opacity: OPACITY_SELECTED,
        content_opacity: OPACITY_ACTIVE,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "tall-signal",
        name: "Tall Signal",
        family: "Shape Rhythm",
        description: "Adds height for peripheral readability without adding persistent controls.",
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 61,
        bars: WIDE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: OVERLAY_WIDTH_PX,
        height: 52.0,
        horizontal_padding: 16.0,
        content_gap: 12.0,
        surface_opacity: GLASS_BG_OPACITY,
        border_opacity: GLASS_BORDER_OPACITY,
        timer_opacity: 0.72,
        content_opacity: 1.0,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "badge-dock",
        name: "Docked Target",
        family: "Shape Rhythm",
        description: "Pins destination metadata to the edge with a divider rather than a chip.",
        target: DictationTarget::ExternalApp,
        elapsed_secs: 47,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::DockedBadge,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Docked,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 414.0,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: 14.0,
        content_gap: 8.0,
        surface_opacity: 0.22,
        border_opacity: 0.16,
        timer_opacity: 0.72,
        content_opacity: 0.96,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "green-ready",
        name: "Active Speech",
        family: "Signal Language",
        description: "Shows a confident success signal only when speech is actually present.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "red-means-problem",
        name: "Quiet Listening",
        family: "Signal Language",
        description: "Keeps low input neutral and legible instead of implying failure.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "transcribe-thread",
        name: "Thread Meter",
        family: "Signal Language",
        description: "Uses a stitched low-height signal for a calmer always-on meter.",
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
        show_badge: false,
    },
    DictationUiVariationSpec {
        id: "transcript-flash",
        name: "Soft Glow",
        family: "Signal Language",
        description: "Raises the glass density slightly for contrast without making it heavy.",
        target: DictationTarget::NotesEditor,
        elapsed_secs: 80,
        bars: QUIET_BARS,
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
        show_badge: false,
    },
    DictationUiVariationSpec {
        id: "confirm-lock",
        name: "Action Ready",
        family: "Signal Language",
        description: "Keeps the default completion affordance quiet and inline with the signal.",
        target: DictationTarget::ExternalApp,
        elapsed_secs: 87,
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
        timer_opacity: 0.56,
        content_opacity: 1.0,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "gold-needle",
        name: "Accent Peak",
        family: "Signal Language",
        description:
            "Marks the live peak with a single theme accent instead of a new color family.",
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 41,
        bars: NEEDLE_BARS,
        layout: PreviewLayout::Standard,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Present,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::GoldNeedle,
        width: OVERLAY_WIDTH_PX,
        height: OVERLAY_HEIGHT_PX,
        horizontal_padding: OVERLAY_HORIZONTAL_PADDING_PX,
        content_gap: OVERLAY_CONTENT_GAP_PX,
        surface_opacity: 0.22,
        border_opacity: 0.16,
        timer_opacity: 0.72,
        content_opacity: 1.0,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "signal-ribbon",
        name: "Signal Ribbon",
        family: "Signal Language",
        description: "Compresses the meter into a continuous low-height readout.",
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
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "notes-shelf",
        name: "Notes Shelf",
        family: "Window Alignment",
        description: "Borrows the Notes window rhythm: metadata rides above a calm meter shelf.",
        target: DictationTarget::NotesEditor,
        elapsed_secs: 73,
        bars: ACTIVE_BARS,
        layout: PreviewLayout::TopRail,
        waveform: WaveformKind::Ribbon,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::GoldNeedle,
        width: 386.0,
        height: 62.0,
        horizontal_padding: 16.0,
        content_gap: 7.0,
        surface_opacity: 0.20,
        border_opacity: 0.12,
        timer_opacity: 0.78,
        content_opacity: 0.96,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "launcher-command-strip",
        name: "Command Strip",
        family: "Window Alignment",
        description: "Feels closer to the main launcher footer: signal first, destination below.",
        target: DictationTarget::MainWindowFilter,
        elapsed_secs: 35,
        bars: WIDE_BARS,
        layout: PreviewLayout::BottomRail,
        waveform: WaveformKind::ThinBars,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::None,
        width: 420.0,
        height: 58.0,
        horizontal_padding: 18.0,
        content_gap: 8.0,
        surface_opacity: 0.18,
        border_opacity: 0.10,
        timer_opacity: 0.70,
        content_opacity: 0.98,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "timer-sidecar",
        name: "Timer Sidecar",
        family: "Window Alignment",
        description: "Makes the elapsed time a fixed utility rail while the signal owns the span.",
        target: DictationTarget::AiChatComposer,
        elapsed_secs: 112,
        bars: NEEDLE_BARS,
        layout: PreviewLayout::Sidecar,
        waveform: WaveformKind::Bars,
        badge_tone: BadgeTone::Minimal,
        silence_tone: SilenceTone::Neutral,
        accent_mode: AccentMode::GoldNeedle,
        width: 398.0,
        height: 50.0,
        horizontal_padding: 12.0,
        content_gap: 12.0,
        surface_opacity: 0.22,
        border_opacity: 0.14,
        timer_opacity: 0.92,
        content_opacity: 1.0,
        show_badge: true,
    },
    DictationUiVariationSpec {
        id: "micro-dots",
        name: "Micro Dots",
        family: "Window Alignment",
        description: "Tests the quietest possible status object: a compact timer plus live dots.",
        target: DictationTarget::ExternalApp,
        elapsed_secs: 12,
        bars: RIBBON_BARS,
        layout: PreviewLayout::Centered,
        waveform: WaveformKind::Dots,
        badge_tone: BadgeTone::Whisper,
        silence_tone: SilenceTone::Dim,
        accent_mode: AccentMode::None,
        width: 242.0,
        height: 58.0,
        horizontal_padding: 18.0,
        content_gap: 6.0,
        surface_opacity: 0.18,
        border_opacity: 0.08,
        timer_opacity: 0.74,
        content_opacity: 0.90,
        show_badge: false,
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
    let theme = get_cached_theme();
    let opacity = theme.get_opacity();
    let chrome = AppChromeColors::from_theme(&theme);

    let mut root = story_container()
        .gap(px(18.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(semantic_text(&theme, opacity.text_strong))
                        .child("Dictation Overlay"),
                )
                .child(
                    div()
                        .text_xs()
                        .max_w(px(720.0))
                        .text_color(semantic_text(&theme, opacity.text_muted_alpha))
                        .child(
                            "Twenty-five storybook-only standalone dictation overlay concepts using launcher-aligned density, contrast, text hierarchy, quiet chrome, and the current Stop/Cancel hotkey rail.",
                        ),
                ),
        );

    for family in [
        "Default Surface",
        "Shape Rhythm",
        "Signal Language",
        "Window Alignment",
    ] {
        let mut section = story_section(family).gap(px(10.0));
        for spec in SPECS.iter().copied().filter(|spec| spec.family == family) {
            section = section.child(render_gallery_item(spec));
        }
        root = root.child(
            section
                .border_t_1()
                .border_color(rgba(chrome.divider_rgba))
                .pt(px(12.0)),
        );
    }

    root.into_any_element()
}

fn resolve_spec(stable_id: &str) -> Option<DictationUiVariationSpec> {
    SPECS.iter().copied().find(|spec| spec.id == stable_id)
}

fn render_gallery_item(spec: DictationUiVariationSpec) -> AnyElement {
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
                        .text_color(semantic_text(&theme, opacity.text_strong))
                        .child(spec.name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(semantic_text(&theme, opacity.text_hint))
                        .child(spec.description),
                ),
        )
        .child(render_spec_stage(spec, false))
        .into_any_element()
}

fn render_spec_stage(spec: DictationUiVariationSpec, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let stage_height = if compact { 138.0 } else { 178.0 };
    let scale = if compact { 0.78 } else { 1.0 };

    div()
        .w_full()
        .h(px(stage_height))
        .flex()
        .justify_center()
        .items_center()
        .p(px(if compact { 6.0 } else { 8.0 }))
        .bg(rgb(theme.colors.background.main))
        .border_1()
        .border_color(rgba(chrome.divider_rgba))
        .rounded(px(8.0))
        .child(
            div()
                .w_full()
                .h_full()
                .flex()
                .justify_center()
                .items_center()
                .rounded(px(7.0))
                .border_1()
                .border_color(rgba(chrome.divider_rgba))
                .bg(rgb(theme.colors.background.main))
                .child(
                    div()
                        .shadow(vec![BoxShadow {
                            color: theme.colors.ui.border.with_opacity(0.18),
                            offset: point(px(0.0), px(8.0)),
                            blur_radius: px(18.0),
                            spread_radius: px(0.0),
                        }])
                        .child(render_overlay(spec, scale)),
                ),
        )
        .into_any_element()
}

fn render_overlay(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let native_rim = uses_native_rim(spec);
    let surface_bg = rgba(chrome.window_surface_rgba);
    let border_color = if native_rim {
        semantic_text(&theme, if theme.is_dark_mode() { 0.34 } else { 0.28 })
    } else {
        theme
            .colors
            .ui
            .border
            .with_opacity(spec.border_opacity.max(0.16))
    };

    let width = spec.width * scale;
    let raw_height = min_overlay_height_for_spec(spec);
    let height = raw_height * scale;
    let radius = overlay_radius_for_spec(spec, raw_height, scale);
    let padding_x = spec.horizontal_padding * scale;
    let padding_y = overlay_vertical_padding_for_spec(spec) * scale;
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
        .py(px(padding_y))
        .gap(px(gap))
        .bg(surface_bg)
        .rounded(px(radius))
        .border_1()
        .border_color(border_color)
        .when(native_rim, |d| {
            d.shadow(vec![BoxShadow {
                color: theme.colors.ui.border.with_opacity(0.22),
                offset: point(px(0.0), px(8.0 * scale)),
                blur_radius: px(20.0 * scale),
                spread_radius: px(0.0),
            }])
        })
        .child(render_overlay_inner(spec, scale))
        .into_any_element()
}

fn render_overlay_inner(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let timer = render_timer(spec, scale);
    let content = render_waveform(spec, scale);
    let badge = render_target_badge(spec, scale);

    let body = match spec.layout {
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
        PreviewLayout::TopRail => div()
            .w_full()
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(6.0 * scale))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(timer)
                    .when(spec.show_badge, |d| {
                        d.child(render_target_badge(spec, scale))
                    }),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(content),
            )
            .into_any_element(),
        PreviewLayout::BottomRail => div()
            .w_full()
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(5.0 * scale))
            .child(
                div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(content),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(timer)
                    .when(spec.show_badge, |d| {
                        d.child(render_dual_rail_badge(spec, scale))
                    }),
            )
            .into_any_element(),
        PreviewLayout::Sidecar => div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.0 * scale))
            .child(
                div()
                    .h_full()
                    .min_w(px(58.0 * scale))
                    .px(px(8.0 * scale))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(theme_badge_dock_bg())
                    .border_r_1()
                    .border_color(get_cached_theme().colors.ui.border.with_opacity(0.18))
                    .child(timer),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(4.0 * scale))
                    .child(content)
                    .when(spec.show_badge, |d| {
                        d.child(render_dual_rail_badge(spec, scale))
                    }),
            )
            .into_any_element(),
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .justify_center()
        .items_center()
        .gap(px(5.0 * scale))
        .child(render_signal_band(body, spec, scale))
        .child(render_action_rail(scale))
        .into_any_element()
}

fn render_signal_band(body: AnyElement, spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);
    let radius = match spec.id {
        "flat-accent" | "gold-needle" | "signal-ribbon" => 3.0,
        "badge-dock" | "timer-sidecar" | "launcher-command-strip" => 6.0,
        "compact-capsule" | "timer-forward" | "split-pill" | "wide-meter" | "tall-signal" => 999.0,
        _ => 9.0,
    } * scale;

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .px(px(10.0 * scale))
        .py(px(6.0 * scale))
        .bg(rgba(chrome.selection_rgba))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.10))
        .rounded(px(radius))
        .child(body)
        .into_any_element()
}

fn render_action_rail(scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    let chrome = AppChromeColors::from_theme(&theme);

    div()
        .id("dictation-action-rail")
        .w_full()
        .min_h(px(24.0))
        .border_t_1()
        .border_color(rgba(chrome.divider_rgba))
        .pt(px((4.0 * scale).max(3.0)))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(render_action_chip(
            "Stop",
            storybook_dictation_stop_keycap(),
            96.0,
        ))
        .child(render_action_chip("Cancel", "esc".to_string(), 96.0))
        .into_any_element()
}

fn render_action_chip(label: &'static str, key: String, width: f32) -> AnyElement {
    let theme = get_cached_theme();
    let footer_text = theme
        .colors
        .text
        .primary
        .with_opacity(crate::window_resize::mini_layout::HINT_TEXT_OPACITY)
        .to_rgb();
    let shortcut_colors = crate::components::hint_strip::whisper_inline_shortcut_colors(
        footer_text.into(),
        theme.colors.text.primary.to_rgb(),
        false,
    );
    let shortcut_tokens = crate::components::hint_strip::shortcut_tokens_from_hint(&key);

    div()
        .w(px(width))
        .h_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .child(
            div()
                .px(px(4.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(3.0))
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(footer_text)
                        .child(label),
                )
                .child(crate::components::hint_strip::render_inline_shortcut_keys(
                    shortcut_tokens.iter().map(|token| token.as_str()),
                    shortcut_colors,
                )),
        )
        .into_any_element()
}

fn render_timer(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w(px(TIMER_SPACER_WIDTH_PX * scale))
        .text_size(px(STATUS_TEXT_SIZE_PX * scale))
        .font_family(FONT_MONO)
        .text_color(semantic_text(&theme, spec.timer_opacity))
        .child(format_elapsed(std::time::Duration::from_secs(
            spec.elapsed_secs,
        )))
        .into_any_element()
}

fn render_waveform(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    let active = has_sound(&spec.bars);
    let base_hex = if active {
        theme.colors.ui.success
    } else {
        theme.colors.text.primary
    };
    let inactive_scale = match spec.silence_tone {
        SilenceTone::Neutral if !active => theme.get_opacity().text_hint,
        SilenceTone::Dim if !active => theme.get_opacity().text_placeholder,
        _ => 1.0,
    };

    let (bar_width, bar_gap) = match spec.waveform {
        WaveformKind::Bars => (WAVEFORM_BAR_WIDTH_PX * scale, WAVEFORM_BAR_GAP_PX * scale),
        WaveformKind::ThinBars => (3.0 * scale, 3.0 * scale),
        WaveformKind::Ribbon => (6.0 * scale, 1.5 * scale),
        WaveformKind::Thread => (2.5 * scale, 3.0 * scale),
        WaveformKind::Dots => (7.0 * scale, 4.0 * scale),
    };

    let mut waveform = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(bar_gap))
        .h(px(WAVEFORM_BAR_MAX_HEIGHT_PX * scale));

    for (index, level) in spec.bars.iter().copied().enumerate() {
        let mut color = base_hex
            .with_opacity(waveform_bar_opacity(level) * spec.content_opacity * inactive_scale);
        if matches!(spec.waveform, WaveformKind::Thread) {
            color = base_hex
                .with_opacity((0.45 + level * 0.35) * spec.content_opacity * inactive_scale);
        }
        let height = match spec.waveform {
            WaveformKind::Thread => (WAVEFORM_BAR_MIN_HEIGHT_PX + level * 6.0) * scale,
            WaveformKind::Ribbon => (6.0 + level * 8.0) * scale,
            WaveformKind::Dots => (5.0 + level * 5.0) * scale,
            _ => waveform_bar_height(level) * scale,
        };

        waveform = waveform.child(
            div()
                .w(px(bar_width))
                .h(px(height))
                .min_h(px(WAVEFORM_BAR_MIN_HEIGHT_PX * scale))
                .bg(color)
                .rounded(px(if matches!(spec.waveform, WaveformKind::Dots) {
                    999.0
                } else {
                    bar_width.max(2.0)
                })),
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

fn render_target_badge(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    if !spec.show_badge {
        return div()
            .w(px(TARGET_BADGE_SLOT_WIDTH_PX * scale))
            .into_any_element();
    }

    let theme = get_cached_theme();
    let opacity = theme.get_opacity();
    let chrome = AppChromeColors::from_theme(&theme);
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
            .text_color(semantic_text(&theme, opacity.text_placeholder))
            .child(label)
            .into_any_element(),
        BadgeTone::Minimal => div()
            .w(px(slot_width))
            .flex()
            .justify_end()
            .text_size(px(text_size))
            .font_family(FONT_MONO)
            .text_color(semantic_text(&theme, opacity.text_hint))
            .child(label)
            .into_any_element(),
        BadgeTone::Present => div()
            .w(px(slot_width))
            .flex()
            .justify_end()
            .child(
                div()
                    .px(px(7.0 * scale))
                    .py(px(2.0 * scale))
                    .bg(rgba(chrome.badge_bg_rgba))
                    .border_1()
                    .border_color(rgba(chrome.badge_border_rgba))
                    .rounded(px(999.0))
                    .text_size(px(text_size))
                    .font_family(FONT_MONO)
                    .text_color(semantic_text(&theme, opacity.text_strong))
                    .child(label),
            )
            .into_any_element(),
        BadgeTone::Docked => div()
            .flex()
            .items_center()
            .h_full()
            .text_size(px(text_size))
            .font_family(FONT_MONO)
            .text_color(semantic_text(&theme, opacity.text_strong))
            .child(label)
            .into_any_element(),
    }
}

fn render_dual_rail_badge(spec: DictationUiVariationSpec, scale: f32) -> AnyElement {
    let theme = get_cached_theme();
    let opacity = theme.get_opacity();
    div()
        .text_size(px((STATUS_TEXT_SIZE_PX - 1.0) * scale))
        .font_family(FONT_MONO)
        .text_color(semantic_text(&theme, opacity.text_hint))
        .child(spec.target.overlay_label())
        .into_any_element()
}

fn theme_badge_dock_bg() -> Hsla {
    let theme = get_cached_theme();
    theme
        .colors
        .text
        .primary
        .with_opacity(theme.get_opacity().hover)
}

#[cfg(test)]
mod tests {
    use super::{dictation_ui_story_variants, render_dictation_ui_story_preview, SPECS};

    #[test]
    fn dictation_ui_story_exposes_twenty_five_variants() {
        assert_eq!(dictation_ui_story_variants().len(), 25);
        assert_eq!(SPECS.len(), 25);
    }

    #[test]
    fn dictation_ui_variant_ids_are_unique() {
        let mut ids: Vec<_> = dictation_ui_story_variants()
            .into_iter()
            .map(|variant| variant.stable_id())
            .collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 25);
    }

    #[test]
    fn dictation_story_preview_falls_back_to_first_variant() {
        let _ = render_dictation_ui_story_preview("does-not-exist");
    }
}
