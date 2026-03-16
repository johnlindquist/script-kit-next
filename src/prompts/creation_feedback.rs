//! CreationFeedbackPanel - persistent "Created" feedback UI for newly created paths.
//!
//! This panel is intentionally inline and callback-driven so the app layer can wire
//! platform-specific behavior for each action.

use gpui::{div, prelude::*, px, rgb, rgba, App, RenderOnce, SharedString, Window};
use std::path::PathBuf;
use std::sync::Arc;

use crate::components::button::{Button, ButtonColors, ButtonVariant};
use crate::designs::DesignVariant;
use crate::theme::opacity::{OPACITY_BORDER, OPACITY_CARD_BG, OPACITY_PROMINENT};
use crate::theme::{self, TypographyResolver};
use crate::ui_foundation::hex_to_rgba_with_opacity;

/// Callback for path-based quick actions from the creation feedback panel.
pub type CreationFeedbackPathAction = Box<dyn Fn(&PathBuf, &mut Window, &mut App) + 'static>;

/// Inline panel that renders post-creation feedback and quick path actions.
#[derive(IntoElement)]
pub struct CreationFeedbackPanel {
    path: PathBuf,
    theme: Arc<theme::Theme>,
    design_variant: DesignVariant,
    on_reveal_in_finder: Option<CreationFeedbackPathAction>,
    on_copy_path: Option<CreationFeedbackPathAction>,
    on_open: Option<CreationFeedbackPathAction>,
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
            design_variant: DesignVariant::Default,
            on_reveal_in_finder: None,
            on_copy_path: None,
            on_open: None,
        }
    }

    pub fn design_variant(mut self, design_variant: DesignVariant) -> Self {
        self.design_variant = design_variant;
        self
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
            design_variant,
            on_reveal_in_finder,
            on_copy_path,
            on_open,
        } = self;

        let text_primary = rgb(theme.colors.text.primary);
        let text_secondary =
            rgba(hex_to_rgba_with_opacity(theme.colors.text.secondary, OPACITY_PROMINENT));
        let border_color =
            rgba(hex_to_rgba_with_opacity(theme.colors.ui.border, OPACITY_BORDER));
        // Translucent to preserve vibrancy from outer shell.
        let path_surface = rgba(hex_to_rgba_with_opacity(
            theme.colors.accent.selected_subtle,
            OPACITY_CARD_BG,
        ));
        let button_colors = ButtonColors::from_theme(&theme);
        let mono_font = TypographyResolver::new(&theme, design_variant)
            .mono_font()
            .to_string();

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

        let tokens = crate::designs::get_tokens(design_variant);
        let spacing = tokens.spacing();

        div()
            .id("creation-feedback-panel")
            .w_full()
            .flex()
            .flex_col()
            .gap(px(spacing.gap_lg))
            .child(crate::components::prompt_form_intro(
                "Created",
                "Your new file is ready. Use the actions below to jump to it.",
                text_primary,
                text_secondary,
                spacing.gap_sm,
            ))
            .child(crate::components::prompt_form_section(
                "Path",
                text_secondary,
                spacing.gap_sm,
                crate::components::prompt_surface(path_surface, border_color)
                    .id("creation-feedback-path-container")
                    .overflow_x_scroll()
                    .overflow_y_hidden()
                    .child(
                        div()
                            .text_sm()
                            .font_family(mono_font)
                            .text_color(text_primary)
                            .whitespace_nowrap()
                            .child(path_text),
                    ),
            ))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(spacing.gap_md))
                    .child(reveal_button)
                    .child(copy_path_button)
                    .child(open_button),
            )
    }
}

#[cfg(test)]
mod create_flow_layout_tests {
    const SOURCE: &str = include_str!("creation_feedback.rs");

    #[test]
    fn creation_feedback_uses_shared_create_flow_helpers() {
        assert!(
            SOURCE.contains("prompt_form_intro("),
            "creation_feedback.rs should use prompt_form_intro"
        );
        assert!(
            SOURCE.contains("prompt_form_section("),
            "creation_feedback.rs should use prompt_form_section"
        );
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
    }

    #[test]
    fn test_creation_feedback_panel_sets_callbacks_when_provided() {
        let panel = CreationFeedbackPanel::new(
            PathBuf::from("/tmp/new-extension"),
            Arc::new(theme::Theme::default()),
        )
        .on_reveal_in_finder(Box::new(|_, _, _| {}))
        .on_copy_path(Box::new(|_, _, _| {}))
        .on_open(Box::new(|_, _, _| {}));

        assert!(panel.on_reveal_in_finder.is_some());
        assert!(panel.on_copy_path.is_some());
        assert!(panel.on_open.is_some());
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
