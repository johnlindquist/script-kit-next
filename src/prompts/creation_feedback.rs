//! CreationFeedbackPanel - persistent "Created" feedback UI for newly created paths.
//!
//! This panel is intentionally inline and callback-driven so the app layer can wire
//! platform-specific behavior for each action.

use gpui::{div, prelude::*, px, rgb, rgba, App, FontWeight, RenderOnce, SharedString, Window};
use std::path::PathBuf;
use std::sync::Arc;

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::designs::DesignVariant;
use crate::theme::{self, TypographyResolver};

/// Callback for path-based quick actions from the creation feedback panel.
pub type CreationFeedbackPathAction = Box<dyn Fn(&PathBuf, &mut Window, &mut App) + 'static>;

/// Callback for dismissing the creation feedback panel.
pub type CreationFeedbackDismissAction = Box<dyn Fn(&mut Window, &mut App) + 'static>;

/// Inline panel that renders post-creation feedback and quick path actions.
#[derive(IntoElement)]
pub struct CreationFeedbackPanel {
    path: PathBuf,
    theme: Arc<theme::Theme>,
    on_reveal_in_finder: Option<CreationFeedbackPathAction>,
    on_copy_path: Option<CreationFeedbackPathAction>,
    on_open: Option<CreationFeedbackPathAction>,
    on_dismiss: Option<CreationFeedbackDismissAction>,
}

impl CreationFeedbackPanel {
    pub fn new(path: PathBuf, theme: Arc<theme::Theme>) -> Self {
        tracing::debug!(
            path = %path.display(),
            "creation_feedback_panel_initialized"
        );
        Self {
            path,
            theme,
            on_reveal_in_finder: None,
            on_copy_path: None,
            on_open: None,
            on_dismiss: None,
        }
    }

    pub fn on_reveal_in_finder(mut self, callback: CreationFeedbackPathAction) -> Self {
        self.on_reveal_in_finder = Some(callback);
        self
    }

    pub fn on_copy_path(mut self, callback: CreationFeedbackPathAction) -> Self {
        self.on_copy_path = Some(callback);
        self
    }

    pub fn on_open(mut self, callback: CreationFeedbackPathAction) -> Self {
        self.on_open = Some(callback);
        self
    }

    pub fn on_dismiss(mut self, callback: CreationFeedbackDismissAction) -> Self {
        self.on_dismiss = Some(callback);
        self
    }

    fn path_text(&self) -> SharedString {
        self.path.to_string_lossy().to_string().into()
    }
}

impl RenderOnce for CreationFeedbackPanel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let path_text = self.path_text();
        let CreationFeedbackPanel {
            path,
            theme,
            on_reveal_in_finder,
            on_copy_path,
            on_open,
            on_dismiss,
        } = self;

        let text_primary = rgb(theme.colors.text.primary);
        let text_secondary = rgba((theme.colors.text.secondary << 8) | 0xD0);
        let border_color = rgba((theme.colors.ui.border << 8) | 0x90);
        // Translucent to preserve vibrancy from outer shell.
        let path_surface = rgba((theme.colors.accent.selected_subtle << 8) | 0x30);
        let button_colors = ButtonColors::from_theme(&theme);
        let mono_font = TypographyResolver::new(&theme, DesignVariant::Default).mono_font();

        let reveal_button = match on_reveal_in_finder {
            Some(callback) => {
                let path = path.clone();
                Button::new("Reveal in Finder", button_colors)
                    .variant(ButtonVariant::Ghost)
                    .on_click(Box::new(move |_event, window, cx| {
                        tracing::debug!(
                            path = %path.display(),
                            "creation_feedback_panel_action_reveal_in_finder"
                        );
                        callback(&path, window, cx);
                    }))
            }
            None => Button::new("Reveal in Finder", button_colors)
                .variant(ButtonVariant::Ghost)
                .disabled(true),
        };

        let copy_path_button = match on_copy_path {
            Some(callback) => {
                let path = path.clone();
                Button::new("Copy Path", button_colors)
                    .variant(ButtonVariant::Ghost)
                    .on_click(Box::new(move |_event, window, cx| {
                        tracing::debug!(
                            path = %path.display(),
                            "creation_feedback_panel_action_copy_path"
                        );
                        callback(&path, window, cx);
                    }))
            }
            None => Button::new("Copy Path", button_colors)
                .variant(ButtonVariant::Ghost)
                .disabled(true),
        };

        let open_button = match on_open {
            Some(callback) => {
                let path = path.clone();
                Button::new("Open", button_colors)
                    .variant(ButtonVariant::Ghost)
                    .on_click(Box::new(move |_event, window, cx| {
                        tracing::debug!(
                            path = %path.display(),
                            "creation_feedback_panel_action_open"
                        );
                        callback(&path, window, cx);
                    }))
            }
            None => Button::new("Open", button_colors)
                .variant(ButtonVariant::Ghost)
                .disabled(true),
        };

        let dismiss_button = match on_dismiss {
            Some(callback) => Button::new("Dismiss", button_colors)
                .variant(ButtonVariant::Primary)
                .on_click(Box::new(move |_event, window, cx| {
                    tracing::debug!("creation_feedback_panel_action_dismiss");
                    callback(window, cx);
                })),
            None => Button::new("Dismiss", button_colors)
                .variant(ButtonVariant::Primary)
                .disabled(true),
        };

        div()
            .id("creation-feedback-panel")
            .w_full()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(12.))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(text_primary)
                    .child("Created"),
            )
            .child(div().text_xs().text_color(text_secondary).child("Path"))
            .child(
                div()
                    .id("creation-feedback-path-container")
                    .w_full()
                    .rounded(px(6.))
                    .border_1()
                    .border_color(border_color)
                    .bg(path_surface)
                    .px(px(10.))
                    .py(px(8.))
                    .overflow_x_scroll()
                    .overflow_y_hidden()
                    .child(
                        div()
                            .text_sm()
                            .font_family(mono_font)
                            .text_color(text_primary)
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(path_text),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(reveal_button)
                    .child(copy_path_button)
                    .child(open_button)
                    .child(dismiss_button),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_feedback_panel_defaults_to_no_callbacks() {
        let panel = CreationFeedbackPanel::new(
            PathBuf::from("/tmp/new-script.ts"),
            Arc::new(theme::Theme::default()),
        );

        assert!(panel.on_reveal_in_finder.is_none());
        assert!(panel.on_copy_path.is_none());
        assert!(panel.on_open.is_none());
        assert!(panel.on_dismiss.is_none());
    }

    #[test]
    fn test_creation_feedback_panel_sets_callbacks_when_provided() {
        let panel = CreationFeedbackPanel::new(
            PathBuf::from("/tmp/new-extension"),
            Arc::new(theme::Theme::default()),
        )
        .on_reveal_in_finder(Box::new(|_, _, _| {}))
        .on_copy_path(Box::new(|_, _, _| {}))
        .on_open(Box::new(|_, _, _| {}))
        .on_dismiss(Box::new(|_, _| {}));

        assert!(panel.on_reveal_in_finder.is_some());
        assert!(panel.on_copy_path.is_some());
        assert!(panel.on_open.is_some());
        assert!(panel.on_dismiss.is_some());
    }

    #[test]
    fn test_creation_feedback_panel_path_text_returns_full_path() {
        let panel = CreationFeedbackPanel::new(
            PathBuf::from("/tmp/projects/script-kit/new-script.ts"),
            Arc::new(theme::Theme::default()),
        );

        assert_eq!(
            panel.path_text().to_string(),
            "/tmp/projects/script-kit/new-script.ts"
        );
    }
}
