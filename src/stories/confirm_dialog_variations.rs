//! Confirm Dialog — 21 Minimalist Design Variations
//!
//! Explorations ranging from ultra-minimal to structured-but-clean.
//! Each variant is a self-contained render of a confirm dialog surface
//! using the project's semantic opacity tokens and theme system.

use gpui::*;

use crate::list_item::FONT_MONO;
use crate::storybook::{story_container, story_section, Story, StorySurface, StoryVariant};
use crate::theme::get_cached_theme;
use crate::theme::opacity::*;
use crate::ui_foundation::HexColorExt;

pub struct ConfirmDialogVariationsStory;

impl Story for ConfirmDialogVariationsStory {
    fn id(&self) -> &'static str {
        "confirm-dialog-variations"
    }

    fn name(&self) -> &'static str {
        "Confirm Dialog Redesign (21)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Component
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        render_confirm_variant(&variant.stable_id())
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();
        story_container()
            .child(story_section("Confirm Dialog Redesign").children(
                variants.into_iter().enumerate().map(|(i, v)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .mb_4()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xfbbf24))
                                .child(format!("{}. {}", i + 1, v.name)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgba(0xffffff99))
                                .child(v.description.clone().unwrap_or_default()),
                        )
                        .child(
                            div()
                                .w(px(420.))
                                .child(render_confirm_variant(&v.stable_id())),
                        )
                }),
            ))
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            // ── Ultra-minimal ──────────────────────────────────
            StoryVariant::default_named("ghost-whisper", "Ghost Whisper").description(
                "Near-invisible surface. Title + two text buttons. No borders, no dividers.",
            ),
            StoryVariant::default_named("text-only", "Text Only")
                .description("Pure text — no backgrounds on buttons, just colored text links."),
            StoryVariant::default_named("single-line", "Single Line")
                .description("Title and buttons on one horizontal line. Maximum density."),
            StoryVariant::default_named("bottom-bar", "Bottom Bar")
                .description("Title/body top, thin bottom bar with right-aligned buttons."),
            StoryVariant::default_named("fade-stack", "Fade Stack")
                .description("Vertically stacked full-width buttons with ghost backgrounds."),
            // ── Whisper chrome ──────────────────────────────────
            StoryVariant::default_named("hairline-split", "Hairline Split")
                .description("Hairline divider between content and buttons. Ghost surface."),
            StoryVariant::default_named("accent-confirm", "Accent Confirm")
                .description("Ghost cancel, gold-tinted confirm. Subtle hierarchy."),
            StoryVariant::default_named("pill-buttons", "Pill Buttons")
                .description("Fully rounded pill-shaped buttons. Playful but clean."),
            StoryVariant::default_named("keycap-hints", "Keycap Hints")
                .description("Buttons with separated keycap-style shortcut glyphs."),
            StoryVariant::default_named("centered-stack", "Centered Stack")
                .description("Center-aligned title, body, and side-by-side buttons."),
            // ── Structured minimal ──────────────────────────────
            StoryVariant::default_named("raycast-clean", "Raycast Clean")
                .description("Raycast-inspired: tight padding, sharp edges, muted chrome."),
            StoryVariant::default_named("floating-card", "Floating Card")
                .description("Rounded card with subtle shadow and generous whitespace."),
            StoryVariant::default_named("inline-actions", "Inline Actions")
                .description("Body text flows into inline action links — no button row."),
            StoryVariant::default_named("right-aligned", "Right-Aligned Actions")
                .description("Content left, compact buttons right-aligned at bottom."),
            StoryVariant::default_named("icon-title", "Icon + Title")
                .description("Warning icon left of title. Danger variant showcase."),
            // ── Danger variants ─────────────────────────────────
            StoryVariant::default_named("danger-ghost", "Danger Ghost")
                .description("Red-tinted ghost surface. Destructive action emphasis."),
            StoryVariant::default_named("danger-solid", "Danger Solid")
                .description("Solid red confirm button, ghost cancel. Clear danger signal."),
            StoryVariant::default_named("danger-outline", "Danger Outline")
                .description("Red outline confirm, no fill. Restrained danger."),
            // ── Compact ─────────────────────────────────────────
            StoryVariant::default_named("micro", "Micro")
                .description("Smallest possible: tiny text, tight spacing, inline buttons."),
            StoryVariant::default_named("tooltip-style", "Tooltip Style")
                .description("Tooltip-like compact surface with arrow-key hint."),
            StoryVariant::default_named("command-bar", "Command Bar").description(
                "Horizontal bar: message left, buttons right. Like a notification bar.",
            ),
        ]
    }
}

// ─── Shared demo content ──────────────────────────────────────────────

struct ConfirmDemo {
    title: &'static str,
    body: &'static str,
    confirm: &'static str,
    cancel: &'static str,
    is_danger: bool,
}

fn demo_normal() -> ConfirmDemo {
    ConfirmDemo {
        title: "Clear Conversation",
        body: "This will remove all messages. You can't undo this.",
        confirm: "Clear",
        cancel: "Cancel",
        is_danger: false,
    }
}

fn demo_danger() -> ConfirmDemo {
    ConfirmDemo {
        title: "Delete Script",
        body: "This will permanently delete the script and its data.",
        confirm: "Delete",
        cancel: "Cancel",
        is_danger: true,
    }
}

// ─── Variant renderer ──────────────────────────────────────────────────

fn render_confirm_variant(variant_id: &str) -> AnyElement {
    let theme = get_cached_theme();
    let text_primary = theme.colors.text.primary.to_rgb();
    let text_secondary = theme.colors.text.secondary.to_rgb();
    let text_dimmed = theme.colors.text.dimmed.to_rgb();
    let border = theme.colors.ui.border;
    let bg_main = theme.colors.background.main;
    let accent = theme.colors.accent.selected;
    let error = theme.colors.ui.error;

    match variant_id {
        // ── 1. Ghost Whisper ───────────────────────────────────
        "ghost-whisper" => {
            let d = demo_normal();
            div()
                .p(px(16.))
                .bg(bg_main.with_opacity(OPACITY_GHOST_SOFT))
                .rounded(px(12.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_dimmed).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(16.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 2. Text Only ───────────────────────────────────────
        "text-only" => {
            let d = demo_normal();
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(20.))
                        .pt(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(accent.to_rgb())
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 3. Single Line ─────────────────────────────────────
        "single-line" => {
            let d = demo_normal();
            div()
                .px(px(14.))
                .py(px(10.))
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(10.))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .child(
                            div()
                                .px(px(10.))
                                .py(px(4.))
                                .rounded(px(6.))
                                .bg(border.with_opacity(OPACITY_GHOST))
                                .text_xs()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .px(px(10.))
                                .py(px(4.))
                                .rounded(px(6.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 4. Bottom Bar ──────────────────────────────────────
        "bottom-bar" => {
            let d = demo_normal();
            div()
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(12.))
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(
                    div()
                        .p(px(16.))
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_primary)
                                .child(d.title),
                        )
                        .child(div().text_xs().text_color(text_secondary).child(d.body)),
                )
                .child(
                    div()
                        .w_full()
                        .px(px(16.))
                        .py(px(8.))
                        .bg(border.with_opacity(OPACITY_GHOST_SOFT))
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(format!("{} Esc", d.cancel)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(format!("{} ↵", d.confirm)),
                        ),
                )
                .into_any_element()
        }

        // ── 5. Fade Stack ──────────────────────────────────────
        "fade-stack" => {
            let d = demo_normal();
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .pt(px(4.))
                        .child(
                            div()
                                .w_full()
                                .py(px(8.))
                                .rounded(px(8.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        )
                        .child(
                            div()
                                .w_full()
                                .py(px(8.))
                                .rounded(px(8.))
                                .bg(border.with_opacity(OPACITY_GHOST))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        ),
                )
                .into_any_element()
        }

        // ── 6. Hairline Split ──────────────────────────────────
        "hairline-split" => {
            let d = demo_normal();
            div()
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(12.))
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(
                    div()
                        .px(px(16.))
                        .pt(px(14.))
                        .pb(px(12.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_primary)
                                .child(d.title),
                        )
                        .child(div().text_xs().text_color(text_secondary).child(d.body)),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(1.))
                        .bg(border.with_opacity(OPACITY_GHOST)),
                )
                .child(
                    div()
                        .px(px(16.))
                        .py(px(10.))
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .child(
                            div()
                                .flex_1()
                                .py(px(6.))
                                .rounded(px(8.))
                                .bg(border.with_opacity(OPACITY_GHOST_SOFT))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .flex_1()
                                .py(px(6.))
                                .rounded(px(8.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 7. Accent Confirm ──────────────────────────────────
        "accent-confirm" => {
            let d = demo_normal();
            div()
                .p(px(16.))
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(12.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .child(
                            div()
                                .flex_1()
                                .py(px(7.))
                                .rounded(px(8.))
                                .border_1()
                                .border_color(border.with_opacity(OPACITY_GHOST))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .flex_1()
                                .py(px(7.))
                                .rounded(px(8.))
                                .bg(accent.with_opacity(0.20))
                                .border_1()
                                .border_color(accent.with_opacity(OPACITY_SELECTED))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(accent.to_rgb())
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 8. Pill Buttons ────────────────────────────────────
        "pill-buttons" => {
            let d = demo_normal();
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            div()
                                .px(px(14.))
                                .py(px(5.))
                                .rounded(px(99.))
                                .bg(border.with_opacity(OPACITY_GHOST))
                                .text_xs()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .px(px(14.))
                                .py(px(5.))
                                .rounded(px(99.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 9. Keycap Hints ────────────────────────────────────
        "keycap-hints" => {
            let d = demo_normal();
            div()
                .p(px(16.))
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(12.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .child(
                            // Cancel button with Esc keycap
                            div()
                                .flex_1()
                                .h(px(32.))
                                .px(px(10.))
                                .rounded(px(8.))
                                .bg(border.with_opacity(OPACITY_GHOST_SOFT))
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .cursor_pointer()
                                .child(div().text_sm().text_color(text_dimmed).child(d.cancel))
                                .child(
                                    div()
                                        .px(px(5.))
                                        .py(px(1.))
                                        .rounded(px(3.))
                                        .bg(border.with_opacity(OPACITY_GHOST))
                                        .text_xs()
                                        .font_family(FONT_MONO)
                                        .text_color(text_dimmed)
                                        .child("Esc"),
                                ),
                        )
                        .child(
                            // Confirm button with Enter keycap
                            div()
                                .flex_1()
                                .h(px(32.))
                                .px(px(10.))
                                .rounded(px(8.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .cursor_pointer()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_primary)
                                        .child(d.confirm),
                                )
                                .child(
                                    div()
                                        .px(px(5.))
                                        .py(px(1.))
                                        .rounded(px(3.))
                                        .bg(accent.with_opacity(OPACITY_GHOST))
                                        .text_xs()
                                        .font_family(FONT_MONO)
                                        .text_color(accent.to_rgb())
                                        .child("↵"),
                                ),
                        ),
                )
                .into_any_element()
        }

        // ── 10. Centered Stack ─────────────────────────────────
        "centered-stack" => {
            let d = demo_normal();
            div()
                .p(px(20.))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(10.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .text_center()
                        .child(d.title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(text_secondary)
                        .text_center()
                        .max_w(px(280.))
                        .child(d.body),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .pt(px(4.))
                        .child(
                            div()
                                .px(px(16.))
                                .py(px(6.))
                                .rounded(px(8.))
                                .bg(border.with_opacity(OPACITY_GHOST))
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .px(px(16.))
                                .py(px(6.))
                                .rounded(px(8.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 11. Raycast Clean ──────────────────────────────────
        "raycast-clean" => {
            let d = demo_normal();
            div()
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(10.))
                .border_1()
                .border_color(border.with_opacity(OPACITY_GHOST))
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(
                    div()
                        .px(px(14.))
                        .pt(px(12.))
                        .pb(px(10.))
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_primary)
                                .child(d.title),
                        )
                        .child(div().text_xs().text_color(text_secondary).child(d.body)),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(1.))
                        .bg(border.with_opacity(OPACITY_GHOST)),
                )
                .child(
                    div()
                        .px(px(14.))
                        .py(px(8.))
                        .flex()
                        .flex_row()
                        .gap(px(6.))
                        .child(
                            div()
                                .flex_1()
                                .h(px(28.))
                                .rounded(px(6.))
                                .bg(border.with_opacity(OPACITY_GHOST_SOFT))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_xs()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .flex_1()
                                .h(px(28.))
                                .rounded(px(6.))
                                .bg(accent.with_opacity(0.18))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 12. Floating Card ──────────────────────────────────
        "floating-card" => {
            let d = demo_normal();
            div()
                .p(px(20.))
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(14.))
                .border_1()
                .border_color(border.with_opacity(OPACITY_GHOST_SOFT))
                .flex()
                .flex_col()
                .gap(px(14.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_primary)
                                .child(d.title),
                        )
                        .child(div().text_xs().text_color(text_secondary).child(d.body)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            div()
                                .px(px(12.))
                                .py(px(5.))
                                .rounded(px(7.))
                                .bg(border.with_opacity(OPACITY_GHOST_SOFT))
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .px(px(12.))
                                .py(px(5.))
                                .rounded(px(7.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 13. Inline Actions ─────────────────────────────────
        "inline-actions" => {
            let d = demo_normal();
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .text_xs()
                        .child(
                            div()
                                .text_color(text_secondary)
                                .child(format!("{} ", d.body)),
                        )
                        .child(
                            div()
                                .text_color(accent.to_rgb())
                                .font_weight(FontWeight::MEDIUM)
                                .cursor_pointer()
                                .child(d.confirm),
                        )
                        .child(div().text_color(text_dimmed).child(" · "))
                        .child(
                            div()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        ),
                )
                .into_any_element()
        }

        // ── 14. Right-Aligned Actions ──────────────────────────
        "right-aligned" => {
            let d = demo_normal();
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_primary)
                                .child(d.title),
                        )
                        .child(div().text_xs().text_color(text_secondary).child(d.body)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .px(px(10.))
                                .py(px(4.))
                                .rounded(px(6.))
                                .text_xs()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .px(px(10.))
                                .py(px(4.))
                                .rounded(px(6.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 15. Icon + Title ───────────────────────────────────
        "icon-title" => {
            let d = demo_danger();
            div()
                .p(px(16.))
                .bg(error.with_opacity(OPACITY_GHOST_SOFT))
                .rounded(px(12.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.))
                        .child(div().text_sm().text_color(error.to_rgb()).child("⚠"))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_primary)
                                .child(d.title),
                        ),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .child(
                            div()
                                .flex_1()
                                .py(px(6.))
                                .rounded(px(8.))
                                .bg(border.with_opacity(OPACITY_GHOST_SOFT))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .flex_1()
                                .py(px(6.))
                                .rounded(px(8.))
                                .bg(error.with_opacity(OPACITY_DANGER_BG))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(error.to_rgb())
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 16. Danger Ghost ───────────────────────────────────
        "danger-ghost" => {
            let d = demo_danger();
            div()
                .p(px(16.))
                .bg(error.with_opacity(OPACITY_GHOST_SOFT))
                .rounded(px(12.))
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(12.))
                        .pt(px(2.))
                        .child(
                            div()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(error.to_rgb())
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 17. Danger Solid ───────────────────────────────────
        "danger-solid" => {
            let d = demo_danger();
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .child(
                            div()
                                .flex_1()
                                .py(px(7.))
                                .rounded(px(8.))
                                .bg(border.with_opacity(OPACITY_GHOST))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .flex_1()
                                .py(px(7.))
                                .rounded(px(8.))
                                .bg(error.with_opacity(OPACITY_DANGER_BG))
                                .border_1()
                                .border_color(error.with_opacity(OPACITY_SELECTED))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(error.to_rgb())
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 18. Danger Outline ─────────────────────────────────
        "danger-outline" => {
            let d = demo_danger();
            div()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_secondary).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .child(
                            div()
                                .flex_1()
                                .py(px(6.))
                                .rounded(px(8.))
                                .border_1()
                                .border_color(border.with_opacity(OPACITY_GHOST))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .flex_1()
                                .py(px(6.))
                                .rounded(px(8.))
                                .border_1()
                                .border_color(error.with_opacity(OPACITY_SELECTED))
                                .flex()
                                .justify_center()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(error.to_rgb())
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 19. Micro ──────────────────────────────────────────
        "micro" => {
            let d = demo_normal();
            div()
                .px(px(10.))
                .py(px(8.))
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(8.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_primary)
                        .child(d.title),
                )
                .child(div().text_xs().text_color(text_dimmed).child(d.body))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(6.))
                        .child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .rounded(px(4.))
                                .text_xs()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .px(px(6.))
                                .py(px(2.))
                                .rounded(px(4.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // ── 20. Tooltip Style ──────────────────────────────────
        "tooltip-style" => {
            let d = demo_normal();
            div()
                .px(px(12.))
                .py(px(10.))
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(8.))
                .border_1()
                .border_color(border.with_opacity(OPACITY_GHOST))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_primary)
                        .child(format!("{} — {}", d.title, d.body)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(12.))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(1.))
                                        .rounded(px(3.))
                                        .bg(border.with_opacity(OPACITY_GHOST))
                                        .text_xs()
                                        .font_family(FONT_MONO)
                                        .text_color(text_dimmed)
                                        .child("Esc"),
                                )
                                .child(div().text_xs().text_color(text_dimmed).child(d.cancel)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .px(px(4.))
                                        .py(px(1.))
                                        .rounded(px(3.))
                                        .bg(accent.with_opacity(OPACITY_GHOST))
                                        .text_xs()
                                        .font_family(FONT_MONO)
                                        .text_color(accent.to_rgb())
                                        .child("↵"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(text_primary)
                                        .child(d.confirm),
                                ),
                        ),
                )
                .into_any_element()
        }

        // ── 21. Command Bar ────────────────────────────────────
        "command-bar" => {
            let d = demo_normal();
            div()
                .w_full()
                .px(px(14.))
                .py(px(10.))
                .bg(bg_main.with_opacity(OPACITY_GHOST))
                .rounded(px(10.))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_primary)
                                .child(d.title),
                        )
                        .child(div().text_xs().text_color(text_dimmed).child(d.body)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(6.))
                        .child(
                            div()
                                .px(px(8.))
                                .py(px(4.))
                                .rounded(px(6.))
                                .bg(border.with_opacity(OPACITY_GHOST_SOFT))
                                .text_xs()
                                .text_color(text_dimmed)
                                .cursor_pointer()
                                .child(d.cancel),
                        )
                        .child(
                            div()
                                .px(px(8.))
                                .py(px(4.))
                                .rounded(px(6.))
                                .bg(accent.with_opacity(OPACITY_SUBTLE))
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_primary)
                                .cursor_pointer()
                                .child(d.confirm),
                        ),
                )
                .into_any_element()
        }

        // Fallback
        _ => div()
            .p(px(16.))
            .child(
                div()
                    .text_sm()
                    .text_color(text_secondary)
                    .child(format!("Unknown variant: {}", variant_id)),
            )
            .into_any_element(),
    }
}
