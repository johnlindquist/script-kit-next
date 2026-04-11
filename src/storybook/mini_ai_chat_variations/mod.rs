//! Adoptable design variations for the mini AI chat window.
//!
//! Each variation captures a distinct aesthetic for the mini chat chrome:
//! titlebar, composer, messages, welcome screen, and hint strip.
//! The `Current` spec mirrors the production `MINI_*` constants from
//! `src/ai/window/types.rs` so the baseline is mechanically verifiable.

use super::adoption::{
    adopted_surface_live, resolve_surface_live, AdoptableSurface, SurfaceSelectionResolution,
    VariationId,
};
use super::runtime_fixture;
use super::StoryVariant;

// ─── Production constants (synced from src/ai/window/types.rs) ──────────

const PROD_TITLEBAR_HEIGHT: f32 = 44.0;
const PROD_TITLEBAR_BORDER_OPACITY: f32 = 0.06;
const PROD_TITLEBAR_TITLE_OPACITY: f32 = 0.55;
const PROD_TITLEBAR_ACTION_OPACITY: f32 = 0.45;
const PROD_COMPOSER_BG_OPACITY: f32 = 0.03;
const PROD_COMPOSER_HAIRLINE_OPACITY: f32 = 0.03;
const PROD_COMPOSER_HINT_OPACITY: f32 = 0.38;
const PROD_COMPOSER_ACTIVE_ICON_OPACITY: f32 = 0.50;
const PROD_MESSAGE_USER_BG_OPACITY: f32 = 0.06;
const PROD_MESSAGE_ASSISTANT_BG_OPACITY: f32 = 0.03;
const PROD_MESSAGE_PX: f32 = 12.0;
const PROD_MESSAGE_PY: f32 = 2.0;
const PROD_MESSAGE_GAP: f32 = 8.0;
const PROD_WELCOME_ICON_OPACITY: f32 = 0.35;
const PROD_WELCOME_HEADING_OPACITY: f32 = 0.40;
const PROD_WELCOME_TITLE_OPACITY: f32 = 0.72;
const PROD_WELCOME_BADGE_BG_OPACITY: f32 = 0.04;
const PROD_ACTION_HINT_REVEAL_OPACITY: f32 = 0.65;
const PROD_SUGGESTION_COUNT: usize = 2;

// ─── Variation IDs ──────────────────────────────────────────────────────

/// Stable IDs for adoptable mini AI chat visual styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MiniAiChatVariationId {
    Current,
    Minilastic,
    Bubbles,
    Terminal,
    Flush,
}

impl MiniAiChatVariationId {
    pub const ALL: [Self; 5] = [
        Self::Current,
        Self::Minilastic,
        Self::Bubbles,
        Self::Terminal,
        Self::Flush,
    ];
}

impl VariationId for MiniAiChatVariationId {
    fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Minilastic => "minilastic",
            Self::Bubbles => "bubbles",
            Self::Terminal => "terminal",
            Self::Flush => "flush",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Current => "Current",
            Self::Minilastic => "Minilastic",
            Self::Bubbles => "Bubbles",
            Self::Terminal => "Terminal",
            Self::Flush => "Flush",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Current => "Matches the current live mini AI chat whisper treatment",
            Self::Minilastic => {
                "Matches the mini main window: hint strip footer, no titlebar border, tighter spacing"
            }
            Self::Bubbles => {
                "Rounded message containers with role labels and visible separation"
            }
            Self::Terminal => "Monospace font with prefix markers and zero decoration",
            Self::Flush => {
                "Extreme minimalism: no borders, no backgrounds, pure text on vibrancy"
            }
        }
    }

    fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "current" => Some(Self::Current),
            "minilastic" => Some(Self::Minilastic),
            "bubbles" => Some(Self::Bubbles),
            "terminal" => Some(Self::Terminal),
            "flush" => Some(Self::Flush),
            _ => None,
        }
    }
}

// ─── Style struct ───────────────────────────────────────────────────────

/// Typed live style consumed by both storybook previews and the real mini AI chat.
/// Each field maps to a `MINI_*` constant in `src/ai/window/types.rs`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiniAiChatStyle {
    // Titlebar
    pub titlebar_height: f32,
    pub titlebar_border_opacity: f32,
    pub titlebar_title_opacity: f32,
    pub titlebar_action_opacity: f32,
    pub show_titlebar_border: bool,

    // Composer
    pub composer_bg_opacity: f32,
    pub composer_hairline_opacity: f32,
    pub composer_hint_opacity: f32,
    pub composer_active_icon_opacity: f32,
    pub show_hint_strip: bool,

    // Messages
    pub message_user_bg_opacity: f32,
    pub message_assistant_bg_opacity: f32,
    pub message_padding_x: f32,
    pub message_padding_y: f32,
    pub message_gap: f32,
    pub message_border_radius: f32,
    pub show_role_labels: bool,
    pub mono_font: bool,
    pub user_prefix: Option<&'static str>,
    pub assistant_prefix: Option<&'static str>,

    // Welcome
    pub suggestion_count: usize,
    pub welcome_icon_opacity: f32,
    pub welcome_heading_opacity: f32,
    pub welcome_title_opacity: f32,
    pub welcome_badge_bg_opacity: f32,

    // Action hints
    pub action_hint_reveal_opacity: f32,
    pub show_action_hints: bool,

    // Footer hint strip
    pub footer_hint_text: &'static str,
}

// ─── Specs ──────────────────────────────────────────────────────────────

/// Declarative registry entry for a mini AI chat style variation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiniAiChatVariationSpec {
    pub id: MiniAiChatVariationId,
    pub style: MiniAiChatStyle,
}

pub const SPECS: [MiniAiChatVariationSpec; 5] = [
    // 0: Current — exact production whisper treatment
    MiniAiChatVariationSpec {
        id: MiniAiChatVariationId::Current,
        style: MiniAiChatStyle {
            titlebar_height: PROD_TITLEBAR_HEIGHT,
            titlebar_border_opacity: PROD_TITLEBAR_BORDER_OPACITY,
            titlebar_title_opacity: PROD_TITLEBAR_TITLE_OPACITY,
            titlebar_action_opacity: PROD_TITLEBAR_ACTION_OPACITY,
            show_titlebar_border: true,
            composer_bg_opacity: PROD_COMPOSER_BG_OPACITY,
            composer_hairline_opacity: PROD_COMPOSER_HAIRLINE_OPACITY,
            composer_hint_opacity: PROD_COMPOSER_HINT_OPACITY,
            composer_active_icon_opacity: PROD_COMPOSER_ACTIVE_ICON_OPACITY,
            show_hint_strip: true,
            message_user_bg_opacity: PROD_MESSAGE_USER_BG_OPACITY,
            message_assistant_bg_opacity: PROD_MESSAGE_ASSISTANT_BG_OPACITY,
            message_padding_x: PROD_MESSAGE_PX,
            message_padding_y: PROD_MESSAGE_PY,
            message_gap: PROD_MESSAGE_GAP,
            message_border_radius: 0.0,
            show_role_labels: false,
            mono_font: false,
            user_prefix: None,
            assistant_prefix: None,
            suggestion_count: PROD_SUGGESTION_COUNT,
            welcome_icon_opacity: PROD_WELCOME_ICON_OPACITY,
            welcome_heading_opacity: PROD_WELCOME_HEADING_OPACITY,
            welcome_title_opacity: PROD_WELCOME_TITLE_OPACITY,
            welcome_badge_bg_opacity: PROD_WELCOME_BADGE_BG_OPACITY,
            action_hint_reveal_opacity: PROD_ACTION_HINT_REVEAL_OPACITY,
            show_action_hints: true,
            footer_hint_text: "\u{23ce} Send \u{00b7} \u{2318}K Actions \u{00b7} Esc Dismiss",
        },
    },
    // 1: Minilastic — matches mini main window chrome language
    MiniAiChatVariationSpec {
        id: MiniAiChatVariationId::Minilastic,
        style: MiniAiChatStyle {
            titlebar_height: 40.0,
            titlebar_border_opacity: 0.0,
            titlebar_title_opacity: 0.50,
            titlebar_action_opacity: 0.40,
            show_titlebar_border: false,
            composer_bg_opacity: 0.0,
            composer_hairline_opacity: 0.0,
            composer_hint_opacity: 0.45,
            composer_active_icon_opacity: 0.45,
            show_hint_strip: true,
            message_user_bg_opacity: 0.04,
            message_assistant_bg_opacity: 0.0,
            message_padding_x: 14.0,
            message_padding_y: 2.0,
            message_gap: 6.0,
            message_border_radius: 0.0,
            show_role_labels: false,
            mono_font: false,
            user_prefix: None,
            assistant_prefix: None,
            suggestion_count: 2,
            welcome_icon_opacity: 0.30,
            welcome_heading_opacity: 0.35,
            welcome_title_opacity: 0.65,
            welcome_badge_bg_opacity: 0.03,
            action_hint_reveal_opacity: 0.55,
            show_action_hints: false,
            footer_hint_text: "\u{23ce} Send \u{00b7} \u{2318}K Actions \u{00b7} Esc Dismiss",
        },
    },
    // 2: Bubbles — rounded message containers with role labels
    MiniAiChatVariationSpec {
        id: MiniAiChatVariationId::Bubbles,
        style: MiniAiChatStyle {
            titlebar_height: PROD_TITLEBAR_HEIGHT,
            titlebar_border_opacity: 0.06,
            titlebar_title_opacity: 0.60,
            titlebar_action_opacity: 0.45,
            show_titlebar_border: true,
            composer_bg_opacity: 0.04,
            composer_hairline_opacity: 0.04,
            composer_hint_opacity: 0.38,
            composer_active_icon_opacity: 0.55,
            show_hint_strip: true,
            message_user_bg_opacity: 0.08,
            message_assistant_bg_opacity: 0.04,
            message_padding_x: 14.0,
            message_padding_y: 8.0,
            message_gap: 12.0,
            message_border_radius: 12.0,
            show_role_labels: true,
            mono_font: false,
            user_prefix: None,
            assistant_prefix: None,
            suggestion_count: 2,
            welcome_icon_opacity: 0.40,
            welcome_heading_opacity: 0.45,
            welcome_title_opacity: 0.75,
            welcome_badge_bg_opacity: 0.05,
            action_hint_reveal_opacity: 0.65,
            show_action_hints: true,
            footer_hint_text: "\u{23ce} Send \u{00b7} \u{2318}K Actions \u{00b7} Esc Dismiss",
        },
    },
    // 3: Terminal — monospace with prefix markers, zero decoration
    MiniAiChatVariationSpec {
        id: MiniAiChatVariationId::Terminal,
        style: MiniAiChatStyle {
            titlebar_height: 36.0,
            titlebar_border_opacity: 0.0,
            titlebar_title_opacity: 0.50,
            titlebar_action_opacity: 0.40,
            show_titlebar_border: false,
            composer_bg_opacity: 0.0,
            composer_hairline_opacity: 0.0,
            composer_hint_opacity: 0.40,
            composer_active_icon_opacity: 0.45,
            show_hint_strip: true,
            message_user_bg_opacity: 0.0,
            message_assistant_bg_opacity: 0.0,
            message_padding_x: 10.0,
            message_padding_y: 1.0,
            message_gap: 4.0,
            message_border_radius: 0.0,
            show_role_labels: false,
            mono_font: true,
            user_prefix: Some(">"),
            assistant_prefix: Some("<"),
            suggestion_count: 3,
            welcome_icon_opacity: 0.30,
            welcome_heading_opacity: 0.35,
            welcome_title_opacity: 0.65,
            welcome_badge_bg_opacity: 0.03,
            action_hint_reveal_opacity: 0.55,
            show_action_hints: false,
            footer_hint_text: "\u{23ce} Send \u{00b7} \u{2318}K Actions \u{00b7} Esc Dismiss",
        },
    },
    // 4: Flush — extreme minimalism, pure text on vibrancy
    MiniAiChatVariationSpec {
        id: MiniAiChatVariationId::Flush,
        style: MiniAiChatStyle {
            titlebar_height: 36.0,
            titlebar_border_opacity: 0.0,
            titlebar_title_opacity: 0.45,
            titlebar_action_opacity: 0.35,
            show_titlebar_border: false,
            composer_bg_opacity: 0.0,
            composer_hairline_opacity: 0.0,
            composer_hint_opacity: 0.35,
            composer_active_icon_opacity: 0.40,
            show_hint_strip: true,
            message_user_bg_opacity: 0.0,
            message_assistant_bg_opacity: 0.0,
            message_padding_x: 12.0,
            message_padding_y: 1.0,
            message_gap: 6.0,
            message_border_radius: 0.0,
            show_role_labels: false,
            mono_font: false,
            user_prefix: None,
            assistant_prefix: None,
            suggestion_count: 2,
            welcome_icon_opacity: 0.25,
            welcome_heading_opacity: 0.30,
            welcome_title_opacity: 0.60,
            welcome_badge_bg_opacity: 0.0,
            action_hint_reveal_opacity: 0.50,
            show_action_hints: false,
            footer_hint_text: "\u{23ce} Send \u{00b7} \u{2318}K Actions \u{00b7} Esc",
        },
    },
];

// ─── AdoptableSurface ───────────────────────────────────────────────────

pub struct MiniAiChatSurface;

impl AdoptableSurface for MiniAiChatSurface {
    type Id = MiniAiChatVariationId;
    type Spec = MiniAiChatVariationSpec;
    type Live = MiniAiChatStyle;

    const STORY_ID: &'static str = "mini-ai-chat-variations";
    const DEFAULT_ID: Self::Id = MiniAiChatVariationId::Current;

    fn specs() -> &'static [Self::Spec] {
        &SPECS
    }

    fn spec_id(spec: &Self::Spec) -> Self::Id {
        spec.id
    }

    fn live_from_spec(spec: &Self::Spec) -> Self::Live {
        spec.style
    }
}

// ─── Public helpers ─────────────────────────────────────────────────────

pub fn mini_ai_chat_story_variants() -> Vec<StoryVariant> {
    SPECS
        .iter()
        .map(|spec| {
            StoryVariant::default_named(spec.id.as_str(), spec.id.name())
                .description(spec.id.description())
                .with_prop("surface", "miniAiChat")
                .with_prop("representation", "runtimeFixture")
                .with_prop("fixtureSurface", "mini-ai-chat")
                .with_prop("variantId", spec.id.as_str())
                .with_prop(
                    "showTitlebarBorder",
                    if spec.style.show_titlebar_border {
                        "true"
                    } else {
                        "false"
                    },
                )
                .with_prop(
                    "messageBorderRadius",
                    format!("{:.0}", spec.style.message_border_radius),
                )
                .with_prop(
                    "monoFont",
                    if spec.style.mono_font {
                        "true"
                    } else {
                        "false"
                    },
                )
        })
        .collect()
}

pub fn resolve_mini_ai_chat_style(
    selected: Option<&str>,
) -> (MiniAiChatStyle, SurfaceSelectionResolution) {
    resolve_surface_live::<MiniAiChatSurface>(selected)
}

pub fn adopted_mini_ai_chat_style() -> MiniAiChatStyle {
    adopted_surface_live::<MiniAiChatSurface>()
}

/// Render a Mini ACP Chat storybook preview using the runtime-fixture host.
pub fn render_mini_ai_chat_story_preview(stable_id: &str) -> gpui::AnyElement {
    runtime_fixture::render_runtime_fixture("mini-ai-chat", stable_id, false)
}

/// Render a Mini ACP Chat compare-mode thumbnail via the runtime-fixture host.
pub fn render_mini_ai_chat_compare_thumbnail(stable_id: &str) -> gpui::AnyElement {
    runtime_fixture::render_runtime_fixture("mini-ai-chat", stable_id, true)
}

#[cfg(test)]
mod tests;
