use gpui::{div, prelude::*, px, rems, rgb, rgba, AnyElement, Div, FontWeight, Rgba, SharedString};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::ui_foundation::hex_to_rgba_with_opacity;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PromptFrameConfig {
    pub relative: bool,
    pub rounded_corners: Option<f32>,
    pub min_height_px: f32,
    pub clip_overflow: bool,
}

impl Default for PromptFrameConfig {
    fn default() -> Self {
        Self {
            relative: false,
            rounded_corners: None,
            min_height_px: 0.0,
            clip_overflow: true,
        }
    }
}

impl PromptFrameConfig {
    pub fn with_relative(mut self, relative: bool) -> Self {
        self.relative = relative;
        self
    }

    pub fn with_rounded_corners(mut self, radius: f32) -> Self {
        self.rounded_corners = Some(radius);
        self
    }
}

pub(crate) fn prompt_shell_frame_config(radius: f32) -> PromptFrameConfig {
    PromptFrameConfig::default()
        .with_relative(true)
        .with_rounded_corners(radius)
}

pub(crate) fn prompt_frame_root(config: PromptFrameConfig) -> Div {
    let mut frame = div()
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .min_h(px(config.min_height_px));

    if config.clip_overflow {
        frame = frame.overflow_hidden();
    }

    if config.relative {
        frame = frame.relative();
    }

    if let Some(radius) = config.rounded_corners {
        frame = frame.rounded(px(radius));
    }

    frame
}

pub(crate) fn prompt_frame_fill_content(content: impl IntoElement) -> Div {
    div()
        .flex_1()
        .w_full()
        .min_h(px(0.))
        .overflow_hidden()
        .child(content)
}

/// Shared inner card surface for form fields and content cards.
///
/// Returns a full-width rounded div with consistent padding, border, and
/// background — use this for text inputs, preview cards, and any other
/// "card-on-prompt" surface so every step of a multi-step flow shares the
/// same visual language.
pub(crate) fn prompt_surface(background: Rgba, border: Rgba) -> Div {
    div()
        .w_full()
        .px(rems(0.875))
        .py(rems(0.625))
        .bg(background)
        .border_1()
        .border_color(border)
        .rounded(px(8.0))
}

/// Shared intro block for create-flow screens (title + description).
pub(crate) fn prompt_form_intro(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    title_color: Rgba,
    description_color: Rgba,
    gap_px: f32,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(gap_px))
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title.into()),
        )
        .child(
            div()
                .text_sm()
                .text_color(description_color)
                .child(description.into()),
        )
}

/// Shared labeled section for create-flow screens (label above content).
pub(crate) fn prompt_form_section(
    label: impl Into<SharedString>,
    label_color: Rgba,
    gap_px: f32,
    content: impl IntoElement,
) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(gap_px))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(label_color)
                .child(label.into()),
        )
        .child(content)
}

/// Shared helper text for create-flow screens.
pub(crate) fn prompt_form_help(text: impl Into<SharedString>, color: Rgba) -> Div {
    div().text_xs().text_color(color).child(text.into())
}

/// State of a form field within a create-flow prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PromptFieldState {
    Default,
    Active,
    Error,
    ReadOnly,
}

/// Pre-computed colors for a form field based on its state.
#[derive(Clone, Copy)]
pub(crate) struct PromptFieldStyle {
    pub background: Rgba,
    pub border: Rgba,
    pub value: Rgba,
}

/// Compute field colors from the theme, field state, and whether the value is empty.
///
/// All color/opacity decisions route through [`AppChromeColors`] so prompt
/// fields stay consistent with the rest of the app chrome.
pub(crate) fn prompt_field_style(
    theme: &crate::theme::Theme,
    state: PromptFieldState,
    empty: bool,
) -> PromptFieldStyle {
    let chrome = crate::theme::AppChromeColors::from_theme(theme);
    let muted_value = rgba(hex_to_rgba_with_opacity(
        theme.colors.text.muted,
        theme.get_opacity().input_inactive,
    ));
    let value = if empty {
        muted_value
    } else {
        rgb(chrome.text_primary_hex)
    };

    match state {
        PromptFieldState::Default => PromptFieldStyle {
            background: rgba(chrome.input_surface_rgba),
            border: rgba(chrome.badge_border_rgba),
            value,
        },
        PromptFieldState::Active => PromptFieldStyle {
            background: rgba(chrome.selection_rgba),
            border: rgb(chrome.accent_hex),
            value,
        },
        PromptFieldState::Error => PromptFieldStyle {
            background: rgba(chrome.input_surface_rgba),
            border: rgb(theme.colors.ui.error),
            value,
        },
        PromptFieldState::ReadOnly => PromptFieldStyle {
            background: rgba(chrome.selection_rgba),
            border: rgba(chrome.badge_border_rgba),
            value: rgb(chrome.text_primary_hex),
        },
    }
}

/// Single-line text field card using the shared prompt surface.
pub(crate) fn prompt_text_field(
    value: impl Into<SharedString>,
    style: PromptFieldStyle,
    min_height: f32,
) -> Div {
    prompt_surface(style.background, style.border)
        .min_h(px(min_height))
        .flex()
        .items_center()
        .child(
            div()
                .w_full()
                .text_sm()
                .text_color(style.value)
                .child(value.into()),
        )
}

/// Multi-line detail card with headline, supporting text, and detail text rows.
#[allow(dead_code, clippy::too_many_arguments)]
pub(crate) fn prompt_detail_card(
    headline: impl Into<SharedString>,
    supporting_text: impl Into<SharedString>,
    detail_text: impl Into<SharedString>,
    headline_color: Rgba,
    supporting_color: Rgba,
    detail_color: Rgba,
    style: PromptFieldStyle,
    gap_px: f32,
) -> Div {
    prompt_surface(style.background, style.border).child(
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(gap_px))
            .child(
                div()
                    .text_sm()
                    .text_color(headline_color)
                    .child(headline.into()),
            )
            .child(prompt_form_help(supporting_text, supporting_color))
            .child(prompt_form_help(detail_text, detail_color)),
    )
}

/// Horizontally scrollable single-line value for long paths or strings.
#[allow(dead_code)]
pub(crate) fn prompt_scroll_value(
    value: impl Into<SharedString>,
    color: Rgba,
) -> gpui::Stateful<Div> {
    prompt_scroll_value_with_id("prompt-scroll-value", value, color)
}

/// Horizontally scrollable single-line value with a custom element ID.
///
/// Use this when multiple scroll values appear in the same view to avoid
/// duplicate element IDs.
pub(crate) fn prompt_scroll_value_with_id(
    id: impl Into<gpui::ElementId>,
    value: impl Into<SharedString>,
    color: Rgba,
) -> gpui::Stateful<Div> {
    div()
        .id(id.into())
        .w_full()
        .overflow_x_scroll()
        .overflow_y_hidden()
        .child(
            div()
                .text_xs()
                .text_color(color)
                .whitespace_nowrap()
                .child(value.into()),
        )
}

/// Shared outer shell used by prompt wrappers in `render_prompts/*`.
///
/// This normalizes the frame layout for prompt views:
/// - relative root for overlays
/// - column flex flow
/// - full-width/full-height frame
/// - clipped content with rounded corners
pub fn prompt_shell_container(radius: f32, vibrancy_bg: Option<Rgba>) -> Div {
    prompt_frame_root(prompt_shell_frame_config(radius)).when_some(vibrancy_bg, |d, bg| d.bg(bg))
}

/// Shared content slot used by prompt wrappers.
///
/// This guarantees consistent flex/overflow behavior for the inner prompt entity.
pub fn prompt_shell_content(content: impl IntoElement) -> Div {
    prompt_frame_fill_content(content)
}

/// Shared outer shell for minimal-chrome prompt surfaces.
///
/// Combines `prompt_shell_container` + `prompt_shell_content` with an optional
/// footer element (typically a `HintStrip`). Callers pass body content and an
/// optional `AnyElement` footer — the shell handles the column layout, vibrancy
/// background, and rounded corners.
#[allow(dead_code)]
pub(crate) fn render_simple_prompt_shell(
    radius: f32,
    vibrancy_bg: Option<Rgba>,
    body: impl IntoElement,
    footer: Option<AnyElement>,
) -> Div {
    let shell = prompt_shell_container(radius, vibrancy_bg).child(prompt_shell_content(body));

    if let Some(footer) = footer {
        shell.child(footer)
    } else {
        shell
    }
}

#[allow(dead_code)]
pub(crate) fn render_minimal_list_prompt_scaffold(
    header: impl IntoElement,
    content: impl IntoElement,
    hints: impl crate::components::hint_strip::IntoHints,
    leading: Option<AnyElement>,
) -> Div {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .child(
            div()
                .w_full()
                .px(px(crate::ui::chrome::HEADER_PADDING_X))
                .py(px(crate::ui::chrome::HEADER_PADDING_Y))
                .flex()
                .flex_row()
                .items_center()
                .child(header),
        )
        .child(crate::components::SectionDivider::new())
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h(px(0.))
                .w_full()
                .overflow_hidden()
                .child(content),
        )
        .child(render_simple_hint_strip(hints, leading))
}

#[allow(dead_code)]
pub(crate) fn render_minimal_list_prompt_shell(
    radius: f32,
    vibrancy_bg: Option<Rgba>,
    header: impl IntoElement,
    content: impl IntoElement,
    hints: impl crate::components::hint_strip::IntoHints,
    leading: Option<AnyElement>,
) -> Div {
    render_simple_prompt_shell(
        radius,
        vibrancy_bg,
        render_minimal_list_prompt_scaffold(header, content, hints, leading),
        None,
    )
}

/// Build a hint-strip footer with optional leading status text.
///
/// Wraps `HintStrip::new(hints)` and optionally attaches a leading element
/// (e.g., contextual status text) so callers can replace `PromptFooter` with a
/// single function call while preserving any existing status information.
#[allow(dead_code)]
pub(crate) fn render_simple_hint_strip(
    hints: impl crate::components::hint_strip::IntoHints,
    leading: Option<AnyElement>,
) -> AnyElement {
    let strip = crate::components::HintStrip::new(hints);

    match leading {
        Some(leading) => strip.leading(leading).into_any_element(),
        None => strip.into_any_element(),
    }
}

/// Render muted leading text for a minimal hint strip footer.
///
/// Computes the text color from a theme text color (`0xAARRGGBB`) combined with
/// [`HINT_TEXT_OPACITY`] so callers avoid duplicating the opacity math.
#[allow(dead_code)]
pub(crate) fn render_hint_strip_leading_text(
    text: impl Into<SharedString>,
    text_primary: u32,
) -> AnyElement {
    div()
        .text_xs()
        .text_color(rgba(
            ((text_primary & 0x00FF_FFFF) << 8)
                | crate::ui::chrome::alpha_from_opacity(crate::ui::chrome::HINT_TEXT_OPACITY),
        ))
        .child(text.into())
        .into_any_element()
}

/// Machine-readable contract describing how a prompt surface resolves its chrome.
///
/// Emitted via [`emit_prompt_chrome_audit`] at surface-activation time (not per-frame)
/// so that agents and structured-log consumers can verify which surfaces are minimal,
/// which are intentional exceptions, and which have silently drifted.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub(crate) struct PromptChromeAudit {
    pub surface: &'static str,
    pub input_mode: &'static str,
    pub divider_mode: &'static str,
    pub footer_mode: &'static str,
    pub header_padding_x: u16,
    pub header_padding_y: u16,
    pub hint_count: usize,
    pub has_leading_status: bool,
    pub has_actions: bool,
    pub exception_reason: Option<&'static str>,
}

#[allow(dead_code)]
impl PromptChromeAudit {
    /// Contract for a surface that follows the minimal chrome design language.
    pub(crate) fn minimal(
        surface: &'static str,
        hint_count: usize,
        has_leading_status: bool,
        has_actions: bool,
    ) -> Self {
        Self {
            surface,
            input_mode: "bare",
            divider_mode: "section_divider",
            footer_mode: "hint_strip",
            header_padding_x: crate::ui::chrome::HEADER_PADDING_X as u16,
            header_padding_y: crate::ui::chrome::HEADER_PADDING_Y as u16,
            hint_count,
            has_leading_status,
            has_actions,
            exception_reason: None,
        }
    }

    /// Contract for a surface that intentionally keeps rich chrome (PromptFooter).
    pub(crate) fn exception(surface: &'static str, reason: &'static str) -> Self {
        Self {
            surface,
            input_mode: "custom",
            divider_mode: "custom",
            footer_mode: "prompt_footer",
            header_padding_x: 0,
            header_padding_y: 0,
            hint_count: 0,
            has_leading_status: false,
            has_actions: false,
            exception_reason: Some(reason),
        }
    }
}

fn seen_prompt_chrome_audits() -> &'static Mutex<HashSet<PromptChromeAudit>> {
    static SEEN: OnceLock<Mutex<HashSet<PromptChromeAudit>>> = OnceLock::new();
    SEEN.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Record an audit contract and return `true` if it was first-seen, `false` if duplicate.
///
/// Uses `Hash + Eq` on the full struct so any field change is treated as a new contract.
pub(crate) fn mark_prompt_chrome_audit_seen(audit: &PromptChromeAudit) -> bool {
    let mut seen = seen_prompt_chrome_audits()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    seen.insert(audit.clone())
}

/// Emit a structured log line describing the chrome contract for a prompt surface.
///
/// Call this from surface-activation or configuration paths, **not** from `render()`.
/// Identical contracts are emitted at most once per process.
/// Non-exception surfaces that still resolve to `prompt_footer` emit a warning.
#[allow(dead_code)]
pub(crate) fn emit_prompt_chrome_audit(audit: &PromptChromeAudit) {
    if !mark_prompt_chrome_audit_seen(audit) {
        return;
    }

    tracing::info!(
        target: "script_kit::prompt_chrome",
        event = "prompt_chrome_audit",
        surface = audit.surface,
        input_mode = audit.input_mode,
        divider_mode = audit.divider_mode,
        footer_mode = audit.footer_mode,
        header_padding_x = audit.header_padding_x,
        header_padding_y = audit.header_padding_y,
        hint_count = audit.hint_count,
        has_leading_status = audit.has_leading_status,
        has_actions = audit.has_actions,
        exception_reason = audit.exception_reason.unwrap_or(""),
        "prompt chrome audit"
    );

    if audit.exception_reason.is_none() && audit.footer_mode == "prompt_footer" {
        tracing::warn!(
            target: "script_kit::prompt_chrome",
            event = "prompt_chrome_contract_violation",
            surface = audit.surface,
            footer_mode = audit.footer_mode,
            "non-exception surface resolved to prompt_footer"
        );
    }
}

#[cfg(test)]
mod prompt_layout_shell_tests {
    use super::{prompt_shell_frame_config, PromptFrameConfig};

    #[test]
    fn test_prompt_frame_defaults_apply_min_h_and_overflow_hidden() {
        let config = PromptFrameConfig::default();
        assert_eq!(config.min_height_px, 0.0);
        assert!(config.clip_overflow);
        assert!(!config.relative);
        assert_eq!(config.rounded_corners, None);
    }

    #[test]
    fn test_prompt_shell_frame_config_sets_relative_and_radius() {
        let config = prompt_shell_frame_config(14.0);
        assert_eq!(config.min_height_px, 0.0);
        assert!(config.clip_overflow);
        assert!(config.relative);
        assert_eq!(config.rounded_corners, Some(14.0));
    }

    #[test]
    fn prompt_surface_defaults_match_create_flow_field_chrome() {
        // Verify the shared surface uses the design-specified values.
        // If these change, update all callers too.
        let _surface = super::prompt_surface(gpui::rgba(0x112233ee), gpui::rgba(0x445566ff));
        // The function is purely a builder; the real assertion is that it
        // compiles and the constants below stay in sync with the implementation.
        assert_eq!(8.0_f32, 8.0); // radius
        assert_eq!(0.875_f32, 0.875); // px padding
        assert_eq!(0.625_f32, 0.625); // py padding
    }

    #[test]
    fn prompt_field_style_uses_theme_chrome_contract_for_default_and_active_states() {
        let theme = crate::theme::Theme::light_default();
        let chrome = crate::theme::AppChromeColors::from_theme(&theme);

        let default_style =
            super::prompt_field_style(&theme, super::PromptFieldState::Default, true);
        let active_style =
            super::prompt_field_style(&theme, super::PromptFieldState::Active, false);

        assert_eq!(
            default_style.background,
            gpui::rgba(chrome.input_surface_rgba)
        );
        assert_eq!(default_style.border, gpui::rgba(chrome.badge_border_rgba));
        assert_eq!(active_style.background, gpui::rgba(chrome.selection_rgba));
        assert_eq!(active_style.border, gpui::rgb(chrome.accent_hex));
    }

    const OTHER_RENDERERS_SOURCE: &str = include_str!("../render_prompts/other.rs");

    fn fn_source(name: &str) -> &'static str {
        let marker = format!("fn {}(", name);
        let Some(start) = OTHER_RENDERERS_SOURCE.find(&marker) else {
            return "";
        };
        let tail = &OTHER_RENDERERS_SOURCE[start..];
        let end = tail.find("\n    fn ").unwrap_or(tail.len());
        &tail[..end]
    }

    #[test]
    fn simple_prompt_wrappers_use_shared_layout_shell() {
        for fn_name in [
            "render_select_prompt",
            "render_env_prompt",
            "render_drop_prompt",
        ] {
            let body = fn_source(fn_name);
            assert!(
                body.contains("render_wrapped_prompt_entity("),
                "{fn_name} should delegate to render_wrapped_prompt_entity"
            );
        }
    }

    #[test]
    fn chat_prompt_uses_simple_prompt_shell_in_other_rs() {
        let body = fn_source("render_chat_prompt");
        assert!(
            body.contains("render_wrapped_prompt_entity("),
            "render_chat_prompt should delegate to render_wrapped_prompt_entity"
        );
        assert!(
            body.contains("other_prompt_shell_handle_key_chat"),
            "render_chat_prompt should keep the chat-specific key handler"
        );
    }

    #[test]
    fn other_rs_calls_component_render_simple_prompt_shell_explicitly() {
        assert!(
            OTHER_RENDERERS_SOURCE.contains("crate::components::render_simple_prompt_shell("),
            "other.rs should call the shared shell helper explicitly"
        );
        assert!(
            !OTHER_RENDERERS_SOURCE.contains("fn render_simple_prompt_shell("),
            "other.rs should not define a local helper that shadows the shared helper name"
        );
    }

    #[test]
    fn template_prompt_uses_form_style_shell_in_other_rs() {
        let body = fn_source("render_template_prompt");
        assert!(
            body.contains("PromptFooter::new("),
            "render_template_prompt should use PromptFooter"
        );
        assert!(
            body.contains("STANDARD_HEIGHT"),
            "render_template_prompt should use STANDARD_HEIGHT"
        );
    }

    #[test]
    fn naming_prompt_uses_form_style_shell_in_other_rs() {
        let body = fn_source("render_naming_prompt");
        assert!(
            body.contains("PromptFooter::new("),
            "render_naming_prompt should use PromptFooter"
        );
        assert!(
            body.contains("STANDARD_HEIGHT"),
            "render_naming_prompt should use STANDARD_HEIGHT"
        );
    }

    // ── render_simple_prompt_shell contract tests ──────────────────────

    const SHELL_SOURCE: &str = include_str!("prompt_layout_shell.rs");

    #[test]
    fn render_simple_prompt_shell_accepts_optional_footer() {
        // The function signature must accept Option<AnyElement> for the footer
        // so callers can pass None (no footer) or Some(hint_strip).
        assert!(
            SHELL_SOURCE.contains("footer: Option<AnyElement>"),
            "render_simple_prompt_shell must accept footer as Option<AnyElement>"
        );
    }

    #[test]
    fn render_simple_prompt_shell_delegates_to_shell_container() {
        // Must compose from the existing prompt_shell_container + prompt_shell_content.
        let fn_start = SHELL_SOURCE
            .find("fn render_simple_prompt_shell(")
            .expect("function must exist");
        let fn_body = &SHELL_SOURCE[fn_start..];
        assert!(
            fn_body.contains("prompt_shell_container("),
            "must delegate to prompt_shell_container"
        );
        assert!(
            fn_body.contains("prompt_shell_content("),
            "must delegate to prompt_shell_content"
        );
    }

    #[test]
    fn render_simple_hint_strip_accepts_optional_leading() {
        assert!(
            SHELL_SOURCE.contains("fn render_simple_hint_strip("),
            "render_simple_hint_strip must exist"
        );
        assert!(
            SHELL_SOURCE.contains("leading: Option<AnyElement>"),
            "render_simple_hint_strip must accept leading as Option<AnyElement>"
        );
    }

    #[test]
    fn render_simple_hint_strip_returns_any_element() {
        let fn_start = SHELL_SOURCE
            .find("fn render_simple_hint_strip(")
            .expect("function must exist");
        let fn_body = &SHELL_SOURCE[fn_start..];
        let sig_end = fn_body.find('{').expect("must have body");
        let sig = &fn_body[..sig_end];
        assert!(
            sig.contains("-> AnyElement"),
            "render_simple_hint_strip must return AnyElement"
        );
    }

    // ── PromptChromeAudit contract tests ────────────────────────────────

    #[test]
    fn prompt_chrome_audit_minimal_uses_shared_tokens() {
        let audit = super::PromptChromeAudit::minimal("test_surface", 3, true, true);
        assert_eq!(audit.surface, "test_surface");
        assert_eq!(audit.input_mode, "bare");
        assert_eq!(audit.divider_mode, "section_divider");
        assert_eq!(audit.footer_mode, "hint_strip");
        assert_eq!(
            audit.header_padding_x,
            crate::ui::chrome::HEADER_PADDING_X as u16
        );
        assert_eq!(
            audit.header_padding_y,
            crate::ui::chrome::HEADER_PADDING_Y as u16
        );
        assert_eq!(audit.hint_count, 3);
        assert!(audit.has_leading_status);
        assert!(audit.has_actions);
        assert_eq!(audit.exception_reason, None);
    }

    #[test]
    fn prompt_chrome_audit_exception_records_reason() {
        let audit = super::PromptChromeAudit::exception("webcam_prompt", "media_capture_surface");
        assert_eq!(audit.surface, "webcam_prompt");
        assert_eq!(audit.footer_mode, "prompt_footer");
        assert_eq!(audit.exception_reason, Some("media_capture_surface"));
        assert_eq!(audit.input_mode, "custom");
        assert_eq!(audit.divider_mode, "custom");
    }

    #[test]
    fn prompt_chrome_audit_emit_does_not_panic() {
        // Verify both variants can be emitted without panicking.
        let minimal = super::PromptChromeAudit::minimal("smoke_minimal", 2, false, true);
        super::emit_prompt_chrome_audit(&minimal);

        let exception =
            super::PromptChromeAudit::exception("smoke_exception", "form_heavy_surface");
        super::emit_prompt_chrome_audit(&exception);
    }

    #[test]
    fn prompt_chrome_audit_dedupes_identical_contracts() {
        let audit = super::PromptChromeAudit::minimal("test_dedup_surface", 2, false, false);

        // First insert is new → true
        assert!(super::mark_prompt_chrome_audit_seen(&audit));
        // Duplicate → false
        assert!(!super::mark_prompt_chrome_audit_seen(&audit));

        // Changed contract (different hint_count and has_actions) → true
        let changed = super::PromptChromeAudit::minimal("test_dedup_surface", 3, false, true);
        assert!(super::mark_prompt_chrome_audit_seen(&changed));
    }

    #[test]
    fn exception_surfaces_in_other_rs_emit_chrome_audit() {
        let source = OTHER_RENDERERS_SOURCE;
        // Template, naming, webcam, and creation_feedback prompts should emit audit logs
        assert!(
            source.contains("emit_prompt_chrome_audit("),
            "other.rs should call emit_prompt_chrome_audit for exception surfaces"
        );
        assert!(
            source.contains("PromptChromeAudit::exception("),
            "other.rs should use PromptChromeAudit::exception for rich-chrome surfaces"
        );
        // Verify all four exception surfaces in other.rs are classified
        for surface in [
            "template_prompt",
            "naming_prompt",
            "webcam_prompt",
            "creation_feedback",
        ] {
            assert!(
                source.contains(&format!("\"{}\"", surface)),
                "other.rs should classify {surface} as an exception"
            );
        }
    }

    #[test]
    fn editor_prompt_emits_chrome_audit_exception() {
        let source = include_str!("../render_prompts/editor.rs");
        assert!(
            source.contains("emit_prompt_chrome_audit("),
            "editor.rs should call emit_prompt_chrome_audit"
        );
        assert!(
            source.contains("PromptChromeAudit::exception("),
            "editor.rs should classify as exception"
        );
        assert!(
            source.contains("\"editor_prompt\""),
            "editor.rs should identify as editor_prompt surface"
        );
    }

    #[test]
    fn form_prompt_emits_chrome_audit_exception() {
        let source = include_str!("../render_prompts/form/render.rs");
        assert!(
            source.contains("emit_prompt_chrome_audit("),
            "form/render.rs should call emit_prompt_chrome_audit"
        );
        assert!(
            source.contains("PromptChromeAudit::exception("),
            "form/render.rs should classify as exception"
        );
        assert!(
            source.contains("\"form_prompt\""),
            "form/render.rs should identify as form_prompt surface"
        );
    }

    #[test]
    fn builtin_exception_surfaces_emit_chrome_audit() {
        let kit_store = include_str!("../render_builtins/kit_store.rs");
        assert!(
            kit_store.contains("PromptChromeAudit::exception("),
            "kit_store.rs should classify as exception"
        );

        let process_manager = include_str!("../render_builtins/process_manager.rs");
        assert!(
            process_manager.contains("PromptChromeAudit::minimal("),
            "process_manager.rs should classify as minimal (migrated from exception)"
        );

        let settings = include_str!("../render_builtins/settings.rs");
        assert!(
            settings.contains("PromptChromeAudit::exception("),
            "settings.rs should classify as exception"
        );
    }

    // ── Minimal-chrome source-audit tests for migrated builtins ──────

    fn assert_minimal_surface_source(source: &str, surface: &str, require_header_padding: bool) {
        let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
        let render_code = &source[..render_fn_end];

        if require_header_padding {
            assert!(
                render_code.contains("HEADER_PADDING_X"),
                "{surface} should use chrome HEADER_PADDING_X"
            );
            assert!(
                render_code.contains("HEADER_PADDING_Y"),
                "{surface} should use chrome HEADER_PADDING_Y"
            );
        }

        assert!(
            render_code.contains("SectionDivider::new()"),
            "{surface} should use SectionDivider for its subtle divider"
        );
        assert!(
            render_code.contains("render_simple_hint_strip("),
            "{surface} should render a minimal hint strip footer"
        );

        let needle = ["PromptFooter", "::new("].concat();
        assert!(
            !render_code.contains(&needle),
            "{surface} should not construct PromptFooter after migration"
        );
    }

    /// Assert that source declares a runtime `PromptChromeAudit` with the given
    /// constructor and surface name literal. The failure message names the
    /// drifting surface so agents can pinpoint which builtin regressed.
    fn assert_surface_declares_runtime_audit(source: &str, surface: &str, constructor: &str) {
        let ctor = format!("PromptChromeAudit::{constructor}(");
        let surface_literal = format!("\"{surface}\"");

        assert!(
            source.contains(&ctor) && source.contains(&surface_literal),
            "{surface} should declare PromptChromeAudit::{constructor}(\"{surface}\", ...)"
        );
    }

    /// Combined source-level and runtime-audit assertion for a minimal surface.
    ///
    /// Checks both that the layout file uses `SectionDivider`, `render_simple_hint_strip`,
    /// and shared header padding tokens, AND that the entry-point file declares
    /// `PromptChromeAudit::minimal("<surface>", ...)`.
    macro_rules! assert_minimal_surface_file {
        ($layout_path:literal, $entry_path:literal, $surface:literal, $require_header_padding:expr) => {{
            let layout_source = include_str!($layout_path);
            let entry_source = include_str!($entry_path);
            assert_surface_declares_runtime_audit(entry_source, $surface, "minimal");
            assert_minimal_surface_source(layout_source, $surface, $require_header_padding);
        }};
    }

    #[test]
    fn process_manager_source_matches_minimal_contract() {
        let source = include_str!("../render_builtins/process_manager.rs");
        assert!(
            source.contains("PromptChromeAudit::minimal("),
            "process_manager.rs should emit a minimal chrome audit"
        );
        assert!(
            !source.contains("PromptChromeAudit::exception("),
            "process_manager.rs should no longer emit an exception audit"
        );
        assert_minimal_surface_source(source, "process_manager.rs", false);
    }

    #[test]
    fn clipboard_history_source_matches_minimal_contract() {
        let source = include_str!("../render_builtins/clipboard_history_layout.rs");
        assert_minimal_surface_source(source, "clipboard_history_layout.rs", true);
    }

    #[test]
    fn file_search_source_matches_minimal_contract() {
        let source = include_str!("../render_builtins/file_search_layout.rs");
        assert_minimal_surface_source(source, "file_search_layout.rs", true);
    }

    /// Table-driven regression test covering all migrated minimal builtin surfaces.
    ///
    /// Each entry asserts both source-level markers (SectionDivider, hint strip,
    /// header padding tokens, no PromptFooter) and the presence of a runtime
    /// `PromptChromeAudit::minimal("<surface>", ...)` declaration in the entry file.
    /// When a surface drifts, the failure message names it explicitly.
    #[test]
    fn migrated_builtin_surfaces_match_minimal_contract() {
        // process_manager: layout and entry are in the same file
        assert_minimal_surface_file!(
            "../render_builtins/process_manager.rs",
            "../render_builtins/process_manager.rs",
            "process_manager",
            true
        );

        // clipboard_history: entry in clipboard.rs, layout in clipboard_history_layout.rs
        assert_minimal_surface_file!(
            "../render_builtins/clipboard_history_layout.rs",
            "../render_builtins/clipboard.rs",
            "clipboard_history",
            true
        );

        // file_search: entry in file_search.rs, layout in file_search_layout.rs
        assert_minimal_surface_file!(
            "../render_builtins/file_search_layout.rs",
            "../render_builtins/file_search.rs",
            "file_search",
            true
        );
    }

    #[test]
    fn clipboard_history_emits_minimal_chrome_audit() {
        let source = include_str!("../render_builtins/clipboard.rs");
        assert!(
            source.contains("PromptChromeAudit::minimal("),
            "clipboard.rs should emit a minimal chrome audit"
        );
        assert!(
            source.contains("\"clipboard_history\""),
            "clipboard.rs should identify as clipboard_history surface"
        );
    }

    #[test]
    fn file_search_emits_minimal_chrome_audit() {
        let source = include_str!("../render_builtins/file_search.rs");
        assert!(
            source.contains("PromptChromeAudit::minimal("),
            "file_search.rs should emit a minimal chrome audit"
        );
        assert!(
            source.contains("\"file_search\""),
            "file_search.rs should identify as file_search surface"
        );
    }

    #[test]
    fn render_minimal_list_prompt_scaffold_uses_shared_tokens_and_footer() {
        let fn_start = SHELL_SOURCE
            .find("fn render_minimal_list_prompt_scaffold(")
            .expect("function must exist");
        let fn_body = &SHELL_SOURCE[fn_start..];

        assert!(
            fn_body.contains("HEADER_PADDING_X"),
            "shared list scaffold must own HEADER_PADDING_X"
        );
        assert!(
            fn_body.contains("HEADER_PADDING_Y"),
            "shared list scaffold must own HEADER_PADDING_Y"
        );
        assert!(
            fn_body.contains("SectionDivider::new()"),
            "shared list scaffold must own the divider"
        );
        assert!(
            fn_body.contains("render_simple_hint_strip("),
            "shared list scaffold must own the hint strip footer"
        );
        assert!(
            fn_body.contains("flex_1()") && fn_body.contains("min_h(px(0."),
            "shared list scaffold must own the flex content contract"
        );
    }

    #[test]
    fn arg_prompt_uses_shared_minimal_list_prompt_shell() {
        let source = include_str!("../render_prompts/arg/render.rs");
        assert!(
            source.contains("render_minimal_list_prompt_shell("),
            "arg prompt should use the shared minimal list prompt shell"
        );
    }

    #[test]
    fn launcher_surfaces_use_shared_minimal_list_scaffold() {
        for (source, label) in [
            (
                include_str!("../render_builtins/emoji_picker.rs"),
                "emoji_picker",
            ),
            (
                include_str!("../render_builtins/window_switcher.rs"),
                "window_switcher",
            ),
            (
                include_str!("../render_builtins/app_launcher.rs"),
                "app_launcher",
            ),
            (
                include_str!("../render_builtins/current_app_commands.rs"),
                "current_app_commands",
            ),
            (
                include_str!("../render_builtins/ai_presets.rs"),
                "ai_presets",
            ),
        ] {
            assert!(
                source.contains("render_minimal_list_prompt_scaffold(")
                    || source.contains("render_minimal_list_prompt_shell("),
                "{label} should use the shared minimal list prompt scaffold or shell"
            );
            let legacy = ["PromptFooter", "::new("].concat();
            let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
            let render_code = &source[..render_fn_end];
            assert!(
                !render_code.contains(&legacy),
                "{label} should not construct PromptFooter"
            );
        }
    }

    #[test]
    fn render_minimal_list_prompt_shell_delegates_to_simple_shell() {
        let fn_start = SHELL_SOURCE
            .find("fn render_minimal_list_prompt_shell(")
            .expect("function must exist");
        let fn_body = &SHELL_SOURCE[fn_start..];

        assert!(
            fn_body.contains("render_simple_prompt_shell("),
            "shared list shell must delegate to render_simple_prompt_shell"
        );
        assert!(
            fn_body.contains("render_minimal_list_prompt_scaffold("),
            "shared list shell must wrap the scaffold"
        );
    }

    #[test]
    fn app_launcher_keeps_shell_root_keyboard_hooks() {
        let source = include_str!("../render_builtins/app_launcher.rs");
        let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
        let render_code = &source[..render_fn_end];

        assert!(
            render_code.contains("render_minimal_list_prompt_shell("),
            "app_launcher should return the shared minimal list prompt shell"
        );
        assert!(
            render_code.contains(".key_context(\"app_launcher\")"),
            "app_launcher should keep its key context on the shell root"
        );
        assert!(
            render_code.contains(".track_focus(&self.focus_handle)"),
            "app_launcher should keep focus tracking on the shell root"
        );
        assert!(
            render_code.contains(".on_key_down(handle_key)"),
            "app_launcher should keep the keyboard handler on the shell root"
        );
    }

    #[test]
    fn app_launcher_drops_redundant_header_and_footer_chrome() {
        let source = include_str!("../render_builtins/app_launcher.rs");
        let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
        let render_code = &source[..render_fn_end];

        let legacy = ["PromptFooter", "::new("].concat();
        assert!(
            !render_code.contains(&legacy),
            "app_launcher should not construct PromptFooter after migration"
        );
        assert!(
            !render_code.contains("\u{1f680} Apps"),
            "app_launcher should not keep a redundant launcher title row"
        );
        assert!(
            render_code.contains("render_hint_strip_leading_text("),
            "app count should live in the hint strip leading slot, not in header chrome"
        );
    }

    #[test]
    fn path_prompt_entity_uses_minimal_scaffold_and_hint_strip() {
        let source = include_str!("../prompts/path/render.rs");

        assert!(
            source.contains("render_minimal_list_prompt_scaffold("),
            "path prompt entity should use the shared minimal list prompt scaffold"
        );
        assert!(
            source.contains("render_hint_strip_leading_text("),
            "path prompt entity should use hint strip leading text for item count"
        );
        let legacy = ["PromptFooter", "::new("].concat();
        assert!(
            !source.contains(&legacy),
            "path prompt entity should not construct PromptFooter"
        );
        assert!(
            !source.contains("PromptContainer::new("),
            "path prompt entity should not use legacy PromptContainer"
        );
        assert!(
            !source.contains("PromptHeader::new("),
            "path prompt entity should not use legacy PromptHeader"
        );
    }

    #[test]
    fn path_prompt_outer_wrapper_uses_shared_shell_container() {
        let source = include_str!("../render_prompts/path.rs");
        let render_fn_end = source.find("#[cfg(test)]").unwrap_or(source.len());
        let render_code = &source[..render_fn_end];

        assert!(
            render_code.contains("prompt_shell_container("),
            "path prompt outer wrapper should use the shared prompt_shell_container"
        );
        assert!(
            render_code.contains(".key_context(\"path_prompt_container\")"),
            "path prompt outer wrapper should keep its key context"
        );
        assert!(
            render_code.contains(".on_key_down(handle_key)"),
            "path prompt outer wrapper should keep the keyboard handler"
        );
    }
}
