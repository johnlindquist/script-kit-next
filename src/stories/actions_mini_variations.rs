//! Actions Dialog — Mini Redesign Variations
//!
//! 8 distilled, minimalistic compositions for the actions dialog.
//! Each explores a different take on "clean, keyboard-first, no excess chrome."

use gpui::*;

use crate::storybook::{
    actions_dialog_story_variants, resolve_surface_live, ActionsDialogSurface, Story, StorySurface,
    StoryVariant,
};
use crate::ui_foundation::HexColorExt;

pub struct ActionsMiniVariationsStory;

impl Story for ActionsMiniVariationsStory {
    fn id(&self) -> &'static str {
        "actions-mini-variations"
    }

    fn name(&self) -> &'static str {
        "Actions Mini Redesign (8)"
    }

    fn category(&self) -> &'static str {
        "Layouts"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::ActionDialog
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let (_style, resolution) =
            resolve_surface_live::<ActionsDialogSurface>(Some(variant.stable_id().as_str()));
        // TODO: Replace these mock renderers with the extracted shared ActionsDialog renderer
        // once the live surface render path is available as a reusable storybook entry point.
        render_mini_variant(&resolution.resolved_variant_id)
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();
        crate::storybook::story_container()
            .child(
                crate::storybook::story_section("Actions Mini Redesign").children(
                    variants.into_iter().enumerate().map(|(i, v)| {
                        crate::storybook::story_item(
                            &format!("{}. {}", i + 1, v.name),
                            self.render_variant(&v),
                        )
                    }),
                ),
            )
            .into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        actions_dialog_story_variants()
    }
}

// ─── Data ───────────────────────────────────────────────────────────────

struct ActionItem {
    icon: &'static str,
    label: &'static str,
    shortcut: &'static str,
}

const ACTIONS: &[ActionItem] = &[
    ActionItem {
        icon: "↵",
        label: "Open Application",
        shortcut: "↵",
    },
    ActionItem {
        icon: "🔍",
        label: "Show in Finder",
        shortcut: "⌘↵",
    },
    ActionItem {
        icon: "ℹ",
        label: "Show Info",
        shortcut: "⌘I",
    },
    ActionItem {
        icon: "📦",
        label: "Package Contents",
        shortcut: "⌥⌘I",
    },
    ActionItem {
        icon: "⭐",
        label: "Add to Favorites",
        shortcut: "⇧⌘F",
    },
    ActionItem {
        icon: "📋",
        label: "Copy Path",
        shortcut: "⇧⌘C",
    },
];

fn render_mini_variant(stable_id: &str) -> AnyElement {
    match stable_id {
        "current" => render_current(),
        "whisper" => render_whisper(),
        "ghost-pills" => render_ghost_pills(),
        "typewriter" => render_typewriter(),
        "single-column" => render_single_column(),
        "inline-keys" => render_inline_keys(),
        "search-focused" => render_search_focused(),
        "dot-accent" => render_dot_accent(),
        _ => render_current(),
    }
}

// ─── Shell ──────────────────────────────────────────────────────────────

fn mini_shell(theme: &crate::theme::Theme, _width: f32) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .rounded(px(10.))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.3))
        .bg(theme.colors.background.main.to_rgb())
}

fn cursor_bar(theme: &crate::theme::Theme) -> Div {
    div()
        .w(px(1.5))
        .h(px(14.))
        .bg(theme.colors.accent.selected.to_rgb())
        .rounded(px(1.))
}

// ─── 1. Whisper ─────────────────────────────────────────────────────────

fn render_current() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 0;

    mini_shell(&theme, 320.)
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .h(px(36.))
                .px(px(14.))
                .border_b_1()
                .border_color(theme.colors.ui.border.with_opacity(0.2))
                .flex()
                .flex_row()
                .items_center()
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child("Search actions..."),
                ),
        )
        .child(
            div()
                .w_full()
                .h(px(24.))
                .px(px(12.))
                .flex()
                .items_center()
                .text_size(px(11.))
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child("Actions"),
        )
        .child(div().w_full().py(px(4.)).flex().flex_col().children(
            ACTIONS.iter().enumerate().map(|(i, action)| {
                let is_sel = i == selected;
                let mut row = div()
                    .w_full()
                    .h(px(30.))
                    .px(px(12.))
                    .flex()
                    .flex_row()
                    .items_center();

                if is_sel {
                    row = row.bg(theme.colors.accent.selected.with_opacity(0.08));
                }

                row.child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .text_size(px(13.))
                        .text_color(if is_sel {
                            theme.colors.text.primary.to_rgb()
                        } else {
                            theme.colors.text.secondary.to_rgb()
                        })
                        .child(action.label),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(theme.colors.text.dimmed.with_opacity(0.4))
                        .child(action.shortcut),
                )
            }),
        ))
        .into_any_element()
}

fn render_whisper() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 0;

    div()
        .w_full()
        .flex()
        .flex_col()
        .rounded(px(10.))
        .bg(theme.colors.background.main.to_rgb())
        // No border at all
        .py(px(6.))
        .children(ACTIONS.iter().enumerate().map(|(i, action)| {
            let is_sel = i == selected;
            let mut row = div()
                .w_full()
                .h(px(30.))
                .px(px(14.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.));

            if is_sel {
                row = row.bg(theme.colors.accent.selected.with_opacity(0.08));
            }

            row.child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .text_size(px(13.))
                    .text_color(if is_sel {
                        theme.colors.text.primary.to_rgb()
                    } else {
                        theme.colors.text.muted.to_rgb()
                    })
                    .child(action.label),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(theme.colors.text.dimmed.with_opacity(0.5))
                    .child(action.shortcut),
            )
        }))
        .into_any_element()
}

// ─── 2. Ghost Pills ────────────────────────────────────────────────────

fn render_ghost_pills() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 1;

    mini_shell(&theme, 320.)
        .py(px(6.))
        .px(px(6.))
        .gap(px(2.))
        .children(ACTIONS.iter().enumerate().map(|(i, action)| {
            let is_sel = i == selected;
            let mut row = div()
                .w_full()
                .h(px(32.))
                .px(px(10.))
                .rounded(px(16.)) // Full pill
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.));

            if is_sel {
                row = row
                    .bg(theme.colors.accent.selected.with_opacity(0.12))
                    .border_1()
                    .border_color(theme.colors.accent.selected.with_opacity(0.25));
            }

            row.child(
                div()
                    .w(px(16.))
                    .text_size(px(12.))
                    .text_color(if is_sel {
                        theme.colors.accent.selected.to_rgb()
                    } else {
                        theme.colors.text.dimmed.to_rgb()
                    })
                    .child(action.icon),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .text_size(px(13.))
                    .text_color(if is_sel {
                        theme.colors.text.primary.to_rgb()
                    } else {
                        theme.colors.text.secondary.to_rgb()
                    })
                    .child(action.label),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(theme.colors.text.dimmed.with_opacity(0.4))
                    .child(action.shortcut),
            )
        }))
        .into_any_element()
}

// ─── 3. Typewriter ──────────────────────────────────────────────────────

fn render_typewriter() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 2;
    let mono = crate::list_item::FONT_MONO;

    div()
        .w_full()
        .flex()
        .flex_col()
        .rounded(px(0.)) // Sharp corners
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.2))
        .bg(theme.colors.background.main.to_rgb())
        // Input line
        .child(
            div()
                .w_full()
                .h(px(32.))
                .px(px(12.))
                .border_b_1()
                .border_color(theme.colors.ui.border.with_opacity(0.15))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .font_family(mono)
                        .text_size(px(13.))
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child(">"),
                )
                .child(cursor_bar(&theme)),
        )
        // Items
        .children(ACTIONS.iter().enumerate().map(|(i, action)| {
            let is_sel = i == selected;
            let mut row = div()
                .w_full()
                .h(px(28.))
                .px(px(12.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.));

            if is_sel {
                row = row.bg(theme.colors.text.primary.with_opacity(0.06));
            }

            row.child(
                div()
                    .font_family(mono)
                    .text_size(px(12.))
                    .text_color(if is_sel {
                        theme.colors.text.primary.to_rgb()
                    } else {
                        theme.colors.text.dimmed.to_rgb()
                    })
                    .child(if is_sel { "▸" } else { " " }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .font_family(mono)
                    .text_size(px(12.))
                    .text_color(if is_sel {
                        theme.colors.text.primary.to_rgb()
                    } else {
                        theme.colors.text.muted.to_rgb()
                    })
                    .child(action.label),
            )
            .child(
                div()
                    .font_family(mono)
                    .text_size(px(10.))
                    .text_color(theme.colors.text.dimmed.with_opacity(0.4))
                    .child(action.shortcut),
            )
        }))
        .into_any_element()
}

// ─── 4. Single Column ───────────────────────────────────────────────────

fn render_single_column() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 0;

    div()
        .w(px(240.))
        .flex()
        .flex_col()
        .rounded(px(8.))
        .border_1()
        .border_color(theme.colors.ui.border.with_opacity(0.2))
        .bg(theme.colors.background.main.to_rgb())
        .py(px(4.))
        .children(ACTIONS.iter().enumerate().map(|(i, action)| {
            let is_sel = i == selected;
            let mut row = div()
                .w_full()
                .h(px(30.))
                .px(px(12.))
                .flex()
                .flex_row()
                .items_center();

            if is_sel {
                row = row.bg(theme.colors.accent.selected.with_opacity(0.1));
            }

            row.child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .text_size(px(13.))
                    .font_weight(if is_sel {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(if is_sel {
                        theme.colors.text.primary.to_rgb()
                    } else {
                        theme.colors.text.muted.to_rgb()
                    })
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(action.label),
            )
        }))
        .into_any_element()
}

// ─── 5. Inline Keys ────────────────────────────────────────────────────

fn render_inline_keys() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 1;

    mini_shell(&theme, 320.)
        .py(px(4.))
        .children(ACTIONS.iter().enumerate().map(|(i, action)| {
            let is_sel = i == selected;
            let mut row = div()
                .w_full()
                .h(px(30.))
                .px(px(12.))
                .flex()
                .flex_row()
                .items_center();

            if is_sel {
                row = row.bg(theme.colors.accent.selected.with_opacity(0.08));
            }

            // Label with shortcut inline as dimmed suffix
            row.child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(if is_sel {
                                theme.colors.text.primary.to_rgb()
                            } else {
                                theme.colors.text.secondary.to_rgb()
                            })
                            .child(action.label),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(theme.colors.text.dimmed.with_opacity(0.35))
                            .child(action.shortcut),
                    ),
            )
        }))
        .into_any_element()
}

// ─── 7. Search Focused ──────────────────────────────────────────────────

fn render_search_focused() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 0;

    mini_shell(&theme, 340.)
        .overflow_hidden()
        // Giant search
        .child(
            div()
                .w_full()
                .px(px(16.))
                .py(px(14.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .text_size(px(18.))
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child("Search actions..."),
                )
                .child(cursor_bar(&theme)),
        )
        .child(
            div()
                .w_full()
                .h(px(1.))
                .bg(theme.colors.ui.border.with_opacity(0.2)),
        )
        // Compact results
        .child(div().w_full().py(px(4.)).flex().flex_col().children(
            ACTIONS.iter().enumerate().map(|(i, action)| {
                let is_sel = i == selected;
                let mut row = div()
                    .w_full()
                    .h(px(28.))
                    .px(px(16.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.));

                if is_sel {
                    row = row.bg(theme.colors.accent.selected.with_opacity(0.08));
                }

                row.child(
                    div()
                        .w(px(14.))
                        .text_size(px(11.))
                        .text_color(theme.colors.text.dimmed.to_rgb())
                        .child(action.icon),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .text_size(px(13.))
                        .text_color(if is_sel {
                            theme.colors.text.primary.to_rgb()
                        } else {
                            theme.colors.text.muted.to_rgb()
                        })
                        .child(action.label),
                )
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(theme.colors.text.dimmed.with_opacity(0.35))
                        .child(action.shortcut),
                )
            }),
        ))
        .into_any_element()
}

// ─── 8. Dot Accent ──────────────────────────────────────────────────────

fn render_dot_accent() -> AnyElement {
    let theme = crate::theme::get_cached_theme();
    let selected = 2;

    mini_shell(&theme, 300.)
        .py(px(6.))
        .children(ACTIONS.iter().enumerate().map(|(i, action)| {
            let is_sel = i == selected;
            div()
                .w_full()
                .h(px(30.))
                .px(px(12.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                // Dot indicator
                .child(div().w(px(5.)).h(px(5.)).rounded(px(3.)).bg(if is_sel {
                    theme.colors.accent.selected.to_rgb()
                } else {
                    gpui::transparent_black()
                }))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .text_size(px(13.))
                        .text_color(if is_sel {
                            theme.colors.text.primary.to_rgb()
                        } else {
                            theme.colors.text.muted.to_rgb()
                        })
                        .child(action.label),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(theme.colors.text.dimmed.with_opacity(0.4))
                        .child(action.shortcut),
                )
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::ActionsMiniVariationsStory;
    use crate::storybook::{Story, StorySurface};

    #[test]
    fn actions_mini_story_is_compare_ready() {
        let story = ActionsMiniVariationsStory;
        assert_eq!(story.surface(), StorySurface::ActionDialog);
        assert_eq!(story.variants().len(), 8);
    }
}
