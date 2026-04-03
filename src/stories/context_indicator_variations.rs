//! Context Chip Variations — Single Chip Below Input (9 Styles)
//!
//! The context chip represents the focused target item from a list.
//! Always 0 or 1 chip: no chip = general question, one chip = about this item.
//! The × dismiss returns to freeform mode.

use gpui::*;

use crate::storybook::{story_container, story_section, Story, StorySurface, StoryVariant};
use crate::theme;

pub struct ContextIndicatorVariationsStory;

impl Story for ContextIndicatorVariationsStory {
    fn id(&self) -> &'static str {
        "context-indicator-variations"
    }

    fn name(&self) -> &'static str {
        "Single Context Chip (9)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::MiniAiChat
    }

    fn render(&self) -> AnyElement {
        let mut container = story_container();

        for variant in self.variants() {
            container = container.child(
                story_section(&format!("{}. {}", variant.id, variant.name)).child(
                    div().child(self.render_variant(&variant)).child(
                        if let Some(desc) = &variant.description {
                            div()
                                .text_xs()
                                .text_color(rgb(theme::get_cached_theme().colors.text.dimmed))
                                .mt(px(4.0))
                                .child(desc.clone())
                        } else {
                            div()
                        },
                    ),
                ),
            );
        }

        container.into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let id = variant.stable_id();
        match id.as_str() {
            "1" => render_ghost_hairline(),
            "2" => render_pill_soft(),
            "3" => render_accent_bar(),
            "4" => render_inline_tag(),
            "5" => render_flush_label(),
            "6" => render_capsule_mono(),
            "7" => render_floating_pill(),
            "8" => render_outlined_badge(),
            "9" => render_icon_chip(),
            _ => render_ghost_hairline(),
        }
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant::default_named("1", "Ghost Hairline").description(
                "Ghost bg, hairline border, dimmed text. × in muted. Maximum whisper.",
            ),
            StoryVariant::default_named("2", "Pill Soft")
                .description("Full rounded pill, no border, soft fill. × blends into the shape."),
            StoryVariant::default_named("3", "Accent Left Bar").description(
                "Gold 2px left bar echoing the list item focus. Ties chip to its origin.",
            ),
            StoryVariant::default_named("4", "Inline Tag")
                .description("Tight square-cornered tag. Dense, compact, terminal-native."),
            StoryVariant::default_named("5", "Flush Label")
                .description("No container — just text + × flush left. Absolute minimum chrome."),
            StoryVariant::default_named("6", "Capsule Mono")
                .description("Monospace in a bordered capsule. Code-native feel."),
            StoryVariant::default_named("7", "Floating Pill").description(
                "Centered pill with more breathing room. Feels like a search scope badge.",
            ),
            StoryVariant::default_named("8", "Outlined Badge").description(
                "Border-only chip with no fill. Just an outline containing the label.",
            ),
            StoryVariant::default_named("9", "Icon + Label").description(
                "Leading icon glyph (⌘) before the label. Visual type hint for the context source.",
            ),
        ]
    }
}

// ─── Shared ─────────────────────────────────────────────────────────

const LABEL: &str = "Command: Dictate to AI";

/// Composer shell: input on top, single chip below, footer at bottom.
fn composer_shell(chip_row: impl IntoElement) -> AnyElement {
    let t = theme::get_cached_theme();
    let bg = t.colors.background.main;
    let border = t.colors.ui.border;
    let muted = t.colors.text.muted;
    let dimmed = t.colors.text.dimmed;

    div()
        .id("mock-composer")
        .w(px(520.0))
        .flex()
        .flex_col()
        .bg(rgb(bg))
        .rounded(px(8.0))
        .border_1()
        .border_color(rgba((border << 8) | 0x30))
        .overflow_hidden()
        // Input row
        .child(
            div().px(px(12.0)).py(px(10.0)).child(
                div()
                    .text_sm()
                    .text_color(rgb(dimmed))
                    .child("Ask Claude Code..."),
            ),
        )
        // Chip row (below input)
        .child(chip_row)
        // Footer
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(12.0))
                .py(px(6.0))
                .border_t_1()
                .border_color(rgba((border << 8) | 0x10))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(dimmed))
                        .child("Sonnet 4.6 \u{25be}"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(muted))
                        .child("\u{21b5} Send \u{00b7} \u{2318}K Actions \u{00b7} \u{2318}W Close"),
                ),
        )
        .into_any_element()
}

/// Shared × dismiss button.
fn dismiss_button(id: &str, text_color: u32, hover_bg: u32) -> gpui::Stateful<Div> {
    div()
        .id(SharedString::from(id.to_string()))
        .cursor_pointer()
        .text_xs()
        .text_color(rgba((text_color << 8) | 0x60))
        .px(px(4.0))
        .py(px(1.0))
        .rounded(px(999.0))
        .hover(|el| {
            el.text_color(rgb(text_color))
                .bg(rgba((hover_bg << 8) | 0x18))
                .rounded(px(999.0))
        })
        .child("\u{00d7}")
}

// ─── 1. Ghost Hairline ──────────────────────────────────────────────

fn render_ghost_hairline() -> AnyElement {
    let t = theme::get_cached_theme();
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    let chip = div()
        .id("v1-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(rgba((border << 8) | 0x08))
        .border_1()
        .border_color(rgba((border << 8) | 0x10))
        .child(div().text_xs().text_color(rgb(dimmed)).child(LABEL))
        .child(dismiss_button("v1-x", muted, border));

    let row = div().id("v1-row").px(px(12.0)).pb(px(8.0)).child(chip);

    composer_shell(row)
}

// ─── 2. Pill Soft ───────────────────────────────────────────────────

fn render_pill_soft() -> AnyElement {
    let t = theme::get_cached_theme();
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    let chip = div()
        .id("v2-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .px(px(10.0))
        .py(px(3.0))
        .rounded(px(999.0))
        .bg(rgba((border << 8) | 0x14))
        .child(div().text_xs().text_color(rgb(dimmed)).child(LABEL))
        .child(dismiss_button("v2-x", muted, border));

    let row = div().id("v2-row").px(px(12.0)).pb(px(8.0)).child(chip);

    composer_shell(row)
}

// ─── 3. Accent Left Bar ─────────────────────────────────────────────

fn render_accent_bar() -> AnyElement {
    let t = theme::get_cached_theme();
    let accent = t.colors.accent.selected;
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    let chip = div()
        .id("v3-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.0))
        .child(
            div()
                .w(px(2.0))
                .h(px(14.0))
                .rounded(px(1.0))
                .bg(rgb(accent)),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .px(px(4.0))
                .py(px(2.0))
                .rounded(px(3.0))
                .bg(rgba((border << 8) | 0x0A))
                .child(div().text_xs().text_color(rgb(dimmed)).child(LABEL))
                .child(dismiss_button("v3-x", muted, border)),
        );

    let row = div().id("v3-row").px(px(12.0)).pb(px(8.0)).child(chip);

    composer_shell(row)
}

// ─── 4. Inline Tag ──────────────────────────────────────────────────

fn render_inline_tag() -> AnyElement {
    let t = theme::get_cached_theme();
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    let chip = div()
        .id("v4-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(2.0))
        .bg(rgba((border << 8) | 0x18))
        .child(div().text_xs().text_color(rgb(dimmed)).child(LABEL))
        .child(dismiss_button("v4-x", muted, border));

    let row = div().id("v4-row").px(px(12.0)).pb(px(8.0)).child(chip);

    composer_shell(row)
}

// ─── 5. Flush Label ─────────────────────────────────────────────────

fn render_flush_label() -> AnyElement {
    let t = theme::get_cached_theme();
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;
    let border = t.colors.ui.border;

    let chip = div()
        .id("v5-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(div().text_xs().text_color(rgb(dimmed)).child(LABEL))
        .child(dismiss_button("v5-x", muted, border));

    let row = div().id("v5-row").px(px(12.0)).pb(px(8.0)).child(chip);

    composer_shell(row)
}

// ─── 6. Capsule Mono ────────────────────────────────────────────────

fn render_capsule_mono() -> AnyElement {
    let t = theme::get_cached_theme();
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    let chip = div()
        .id("v6-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(rgba((border << 8) | 0x10))
        .border_1()
        .border_color(rgba((border << 8) | 0x18))
        .child(
            div()
                .font_family(crate::list_item::FONT_MONO)
                .text_xs()
                .text_color(rgb(dimmed))
                .child(LABEL),
        )
        .child(dismiss_button("v6-x", muted, border));

    let row = div().id("v6-row").px(px(12.0)).pb(px(8.0)).child(chip);

    composer_shell(row)
}

// ─── 7. Floating Pill ───────────────────────────────────────────────

fn render_floating_pill() -> AnyElement {
    let t = theme::get_cached_theme();
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    let chip = div()
        .id("v7-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .px(px(12.0))
        .py(px(4.0))
        .rounded(px(999.0))
        .bg(rgba((border << 8) | 0x0C))
        .border_1()
        .border_color(rgba((border << 8) | 0x14))
        .child(div().text_xs().text_color(rgb(dimmed)).child(LABEL))
        .child(dismiss_button("v7-x", muted, border));

    let row = div()
        .id("v7-row")
        .flex()
        .justify_center()
        .px(px(12.0))
        .pb(px(8.0))
        .child(chip);

    composer_shell(row)
}

// ─── 8. Outlined Badge ──────────────────────────────────────────────

fn render_outlined_badge() -> AnyElement {
    let t = theme::get_cached_theme();
    let border = t.colors.ui.border;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    let chip = div()
        .id("v8-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.0))
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .border_1()
        .border_color(rgba((border << 8) | 0x24))
        .child(div().text_xs().text_color(rgb(dimmed)).child(LABEL))
        .child(dismiss_button("v8-x", muted, border));

    let row = div().id("v8-row").px(px(12.0)).pb(px(8.0)).child(chip);

    composer_shell(row)
}

// ─── 9. Icon + Label ────────────────────────────────────────────────

fn render_icon_chip() -> AnyElement {
    let t = theme::get_cached_theme();
    let border = t.colors.ui.border;
    let accent = t.colors.accent.selected;
    let dimmed = t.colors.text.dimmed;
    let muted = t.colors.text.muted;

    let chip = div()
        .id("v9-chip")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.0))
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(rgba((border << 8) | 0x08))
        .border_1()
        .border_color(rgba((border << 8) | 0x10))
        .child(div().text_xs().text_color(rgb(accent)).child("\u{2318}"))
        .child(div().text_xs().text_color(rgb(dimmed)).child(LABEL))
        .child(dismiss_button("v9-x", muted, border));

    let row = div().id("v9-row").px(px(12.0)).pb(px(8.0)).child(chip);

    composer_shell(row)
}

#[cfg(test)]
mod tests {
    use super::ContextIndicatorVariationsStory;
    use crate::storybook::Story;

    #[test]
    fn context_indicator_story_has_9_variants() {
        let story = ContextIndicatorVariationsStory;
        assert_eq!(story.variants().len(), 9);
    }

    #[test]
    fn all_variant_ids_are_unique() {
        let story = ContextIndicatorVariationsStory;
        let ids: Vec<_> = story.variants().iter().map(|v| v.stable_id()).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len());
    }
}
