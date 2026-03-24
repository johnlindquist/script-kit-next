use super::types::*;
use super::*;
use crate::theme::opacity::{OPACITY_DISABLED, OPACITY_HOVER, OPACITY_SELECTED};

/// Explicit category labels for grouping keyboard shortcuts in the Cmd+/ overlay.
/// Each variant corresponds 1:1 with an entry in [`AI_SHORTCUT_SECTIONS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShortcutCategory {
    Navigation,
    Chat,
    Input,
    Actions,
}

impl ShortcutCategory {
    pub(super) const ALL: [Self; 4] = [Self::Navigation, Self::Chat, Self::Input, Self::Actions];

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Navigation => "Navigation",
            Self::Chat => "Chat",
            Self::Input => "Input",
            Self::Actions => "Actions",
        }
    }
}

impl AiApp {
    pub(super) fn render_command_bar_overlay(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        // Command bar now renders in a separate vibrancy window (not inline)
        // See CommandBar component for window management
        div().id("command-bar-overlay-deprecated")
    }

    /// Render the keyboard shortcuts overlay (Cmd+/).
    /// Reads from AI_SHORTCUT_SECTIONS so the overlay stays in sync with actual keybindings.
    pub(super) fn render_shortcuts_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let overlay_bg = Self::get_modal_overlay_background();
        let panel_bg = cx.theme().background;
        let border = cx.theme().border;
        let fg = cx.theme().foreground;
        let muted = cx.theme().muted_foreground;
        let accent = cx.theme().accent;
        let current_mode = self.window_mode;
        let is_mini = current_mode.is_mini();

        div()
            .id("shortcuts-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(overlay_bg)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.showing_shortcuts_overlay = false;
                    cx.notify();
                }),
            )
            .child(
                div()
                    .id("shortcuts-panel")
                    // Compact sizing in mini mode to fit the 720×440 window
                    .w(if is_mini { px(380.) } else { px(420.) })
                    .max_h(if is_mini { px(360.) } else { px(520.) })
                    .rounded(R_LG)
                    .bg(panel_bg)
                    .border_1()
                    .border_color(border)
                    .p(S4)
                    .flex()
                    .flex_col()
                    .gap(S3)
                    .overflow_y_scrollbar()
                    // Prevent clicks inside the panel from closing the overlay
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(S2)
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(fg)
                                            .child("Keyboard Shortcuts"),
                                    )
                                    .child(
                                        div()
                                            .px(S1)
                                            .py(px(1.))
                                            .rounded(R_SM)
                                            .bg(accent.opacity(OPACITY_DISABLED))
                                            .text_xs()
                                            .text_color(accent)
                                            .child(if is_mini { "Mini" } else { "Full" }),
                                    ),
                            )
                            .child(
                                div()
                                    .px(S2)
                                    .py(S1)
                                    .rounded(R_SM)
                                    .bg(cx.theme().muted.opacity(OPACITY_HOVER))
                                    .text_xs()
                                    .text_color(muted)
                                    .child("\u{2318}/"),
                            ),
                    )
                    // Divider
                    .child(div().w_full().h(px(1.)).bg(border))
                    // Grouped shortcut sections
                    .children(
                        ShortcutCategory::ALL
                            .into_iter()
                            .zip(AI_SHORTCUT_SECTIONS.iter())
                            .filter_map(|(category, section)| {
                                // Filter items to those relevant for the current mode
                                let visible_items: Vec<_> = section
                                    .items
                                    .iter()
                                    .filter(|item| match item.mode {
                                        None => true, // shown in both modes
                                        Some(m) => m == current_mode,
                                    })
                                    .collect();
                                // Skip empty sections entirely
                                if visible_items.is_empty() {
                                    return None;
                                }
                                Some(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(S1)
                                        .child(
                                            div()
                                                .pt(S1)
                                                .pb(S1)
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(accent.opacity(OPACITY_SELECTED))
                                                .child(category.label()),
                                        )
                                        .children(visible_items.into_iter().map(|item| {
                                            let key_s: SharedString = item.keys.into();
                                            let desc_s: SharedString = item.description.into();
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .py(S1)
                                                .gap(S3)
                                                .child(div().text_sm().text_color(fg).child(desc_s))
                                                .child(
                                                    div()
                                                        .px(S2)
                                                        .py(S1)
                                                        .rounded(R_SM)
                                                        .border_1()
                                                        .border_color(
                                                            border.opacity(OPACITY_DISABLED),
                                                        )
                                                        .bg(cx.theme().muted.opacity(OPACITY_HOVER))
                                                        .text_xs()
                                                        .text_color(muted)
                                                        .child(key_s),
                                                )
                                        })),
                                )
                            }),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_category_labels_match_ai_shortcut_sections() {
        let category_labels: Vec<&str> = ShortcutCategory::ALL
            .into_iter()
            .map(ShortcutCategory::label)
            .collect();
        let section_titles: Vec<&str> = AI_SHORTCUT_SECTIONS
            .iter()
            .map(|section| section.title)
            .collect();

        assert_eq!(category_labels, section_titles);
    }
}
