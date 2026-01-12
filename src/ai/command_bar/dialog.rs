//! AI Command Bar Dialog
//!
//! A searchable, keyboard-driven dropdown following patterns from DESIGNING_POPUP_WINDOWS.md.
//! Uses uniform_list for virtualized scrolling with fixed-height items.

use gpui::{
    div, prelude::FluentBuilder, px, rgba, uniform_list, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, UniformListScrollHandle, Window,
};
use std::ops::Range;

use super::constants::*;
use crate::designs::icon_variations::IconName as LocalIconName;

/// Action available in the command bar
#[derive(Clone)]
pub struct CommandBarAction {
    /// Unique identifier
    pub id: &'static str,
    /// Display name
    pub name: &'static str,
    /// Icon (SVG)
    pub icon: LocalIconName,
    /// Keyboard shortcut display (e.g., "cmd+shift+c")
    pub shortcut: Option<&'static str>,
    /// Section group
    pub section: &'static str,
}

impl CommandBarAction {
    /// Create all available AI actions
    pub fn all_actions() -> Vec<CommandBarAction> {
        vec![
            // Response actions
            CommandBarAction {
                id: "copy_response",
                name: "Copy Response",
                icon: LocalIconName::Copy,
                shortcut: Some("cmd+shift+c"),
                section: "Response",
            },
            CommandBarAction {
                id: "copy_chat",
                name: "Copy Chat",
                icon: LocalIconName::Copy,
                shortcut: Some("opt+shift+cmd+c"),
                section: "Response",
            },
            CommandBarAction {
                id: "copy_last_code",
                name: "Copy Last Code Block",
                icon: LocalIconName::Code,
                shortcut: Some("opt+cmd+c"),
                section: "Response",
            },
            // Submit actions
            CommandBarAction {
                id: "submit",
                name: "Submit",
                icon: LocalIconName::ArrowUp,
                shortcut: Some("enter"),
                section: "Actions",
            },
            CommandBarAction {
                id: "new_chat",
                name: "New Chat",
                icon: LocalIconName::Plus,
                shortcut: Some("cmd+n"),
                section: "Actions",
            },
            CommandBarAction {
                id: "delete_chat",
                name: "Delete Chat",
                icon: LocalIconName::Trash,
                shortcut: Some("cmd+backspace"),
                section: "Actions",
            },
            // Attachments
            CommandBarAction {
                id: "add_attachment",
                name: "Add Attachments...",
                icon: LocalIconName::Plus,
                shortcut: Some("cmd+shift+a"),
                section: "Attachments",
            },
            CommandBarAction {
                id: "paste_image",
                name: "Paste Image from Clipboard",
                icon: LocalIconName::File,
                shortcut: Some("cmd+v"),
                section: "Attachments",
            },
            // Settings
            CommandBarAction {
                id: "change_model",
                name: "Change Model",
                icon: LocalIconName::Settings,
                shortcut: None,
                section: "Settings",
            },
        ]
    }

    /// Score an action against a search query (higher = better match)
    fn score(&self, query: &str) -> i32 {
        let name_lower = self.name.to_lowercase();
        let query_lower = query.to_lowercase();

        if name_lower.starts_with(&query_lower) {
            100 // Prefix match
        } else if name_lower.contains(&query_lower) {
            50 // Contains match
        } else if Self::fuzzy_match(&name_lower, &query_lower) {
            25 // Subsequence match
        } else {
            0 // No match
        }
    }

    /// Check if query is a subsequence of text
    fn fuzzy_match(text: &str, query: &str) -> bool {
        let mut query_chars = query.chars().peekable();
        for c in text.chars() {
            if query_chars.peek() == Some(&c) {
                query_chars.next();
            }
        }
        query_chars.peek().is_none()
    }
}

/// Colors extracted for closure use (must be Copy)
#[derive(Clone, Copy)]
pub struct CommandBarColors {
    pub background: u32,
    pub border: u32,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_dimmed: u32,
    pub accent: u32,
}

impl Default for CommandBarColors {
    fn default() -> Self {
        Self {
            background: 0x1e1e1e,
            border: 0x464647,
            text_primary: 0xffffff,
            text_secondary: 0xcccccc,
            text_dimmed: 0x666666,
            accent: 0xfbbf24,
        }
    }
}

/// The command bar dialog component
pub struct CommandBarDialog {
    /// All actions
    actions: Vec<CommandBarAction>,
    /// Filtered action indices
    filtered_indices: Vec<usize>,
    /// Current search text
    search_text: String,
    /// Selected index in filtered list
    selected_index: usize,
    /// Scroll handle for uniform_list
    scroll_handle: UniformListScrollHandle,
    /// Focus handle
    focus_handle: FocusHandle,
    /// Callback when action is selected
    on_select: Box<dyn Fn(String) + 'static>,
    /// Whether cursor is visible (for blinking)
    cursor_visible: bool,
    /// Colors (cached for closures)
    colors: CommandBarColors,
}

impl CommandBarDialog {
    /// Create a new command bar dialog
    pub fn new<F>(cx: &mut Context<Self>, on_select: F) -> Self
    where
        F: Fn(String) + 'static,
    {
        let actions = CommandBarAction::all_actions();
        let filtered_indices: Vec<usize> = (0..actions.len()).collect();

        Self {
            actions,
            filtered_indices,
            search_text: String::new(),
            selected_index: 0,
            scroll_handle: UniformListScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            on_select: Box::new(on_select),
            cursor_visible: true,
            colors: CommandBarColors::default(),
        }
    }

    /// Update colors from theme
    pub fn set_colors(&mut self, colors: CommandBarColors, cx: &mut Context<Self>) {
        self.colors = colors;
        cx.notify();
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        self.search_text.push(ch);
        self.refilter();
        cx.notify();
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.search_text.is_empty() {
            self.search_text.pop();
            self.refilter();
            cx.notify();
        }
    }

    /// Clear search text
    pub fn clear_search(&mut self, cx: &mut Context<Self>) {
        self.search_text.clear();
        self.refilter();
        cx.notify();
    }

    /// Move selection up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.scroll_handle
                .scroll_to_item(self.selected_index, gpui::ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    /// Move selection down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        let max_idx = self.filtered_indices.len().saturating_sub(1);
        if self.selected_index < max_idx {
            self.selected_index += 1;
            self.scroll_handle
                .scroll_to_item(self.selected_index, gpui::ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    /// Submit the selected action
    pub fn submit_selected(&self) {
        if let Some(&action_idx) = self.filtered_indices.get(self.selected_index) {
            if let Some(action) = self.actions.get(action_idx) {
                (self.on_select)(action.id.to_string());
            }
        }
    }

    /// Cancel (close without selection)
    pub fn submit_cancel(&self) {
        (self.on_select)("__cancel__".to_string());
    }

    /// Get the currently selected action ID (for external handling)
    pub fn get_selected_action_id(&self) -> Option<String> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.actions.get(idx))
            .map(|action| action.id.to_string())
    }

    /// Toggle cursor visibility (for blinking)
    pub fn toggle_cursor(&mut self, cx: &mut Context<Self>) {
        self.cursor_visible = !self.cursor_visible;
        cx.notify();
    }

    /// Refilter actions based on search text
    fn refilter(&mut self) {
        if self.search_text.is_empty() {
            self.filtered_indices = (0..self.actions.len()).collect();
        } else {
            let mut scored: Vec<(usize, i32)> = self
                .actions
                .iter()
                .enumerate()
                .filter_map(|(idx, action)| {
                    let score = action.score(&self.search_text);
                    if score > 0 {
                        Some((idx, score))
                    } else {
                        None
                    }
                })
                .collect();

            // Sort by score descending
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_indices = scored.into_iter().map(|(idx, _)| idx).collect();
        }

        // Reset selection
        self.selected_index = 0;
        if !self.filtered_indices.is_empty() {
            self.scroll_handle
                .scroll_to_item(0, gpui::ScrollStrategy::Nearest);
        }
    }

    /// Format shortcut for display with keycap symbols
    fn format_shortcut(shortcut: &str) -> String {
        shortcut
            .split('+')
            .map(|part| match part.trim().to_lowercase().as_str() {
                "cmd" | "command" | "meta" => "\u{2318}".to_string(),
                "ctrl" | "control" => "\u{2303}".to_string(),
                "alt" | "opt" | "option" => "\u{2325}".to_string(),
                "shift" => "\u{21E7}".to_string(),
                "enter" | "return" => "\u{21B5}".to_string(),
                "escape" | "esc" => "\u{238B}".to_string(),
                "tab" => "\u{21E5}".to_string(),
                "backspace" | "delete" => "\u{232B}".to_string(),
                "space" => "\u{2423}".to_string(),
                "up" | "arrowup" => "\u{2191}".to_string(),
                "down" | "arrowdown" => "\u{2193}".to_string(),
                "left" | "arrowleft" => "\u{2190}".to_string(),
                "right" | "arrowright" => "\u{2192}".to_string(),
                other => other.to_uppercase(),
            })
            .collect()
    }

    /// Parse shortcut into individual keycaps
    fn parse_shortcut_keycaps(shortcut: &str) -> Vec<String> {
        let formatted = Self::format_shortcut(shortcut);
        let mut keycaps = Vec::new();

        for ch in formatted.chars() {
            match ch {
                '\u{2318}' | '\u{2303}' | '\u{2325}' | '\u{21E7}' | '\u{21B5}' | '\u{238B}'
                | '\u{21E5}' | '\u{232B}' | '\u{2423}' | '\u{2191}' | '\u{2193}' | '\u{2190}'
                | '\u{2192}' => {
                    keycaps.push(ch.to_string());
                }
                _ => {
                    keycaps.push(ch.to_uppercase().to_string());
                }
            }
        }
        keycaps
    }

    /// Render the search input area
    fn render_search_input(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let search_text = self.search_text.clone();
        let cursor_visible = self.cursor_visible;

        let input_bg_alpha = if search_text.is_empty() {
            ALPHA_INPUT_EMPTY
        } else {
            ALPHA_INPUT_ACTIVE
        };

        let border_alpha = if !search_text.is_empty() {
            0x60u8 // Accent when has text
        } else {
            ALPHA_KEYCAP_BG // Subtle when empty
        };

        div()
            .w(px(POPUP_WIDTH))
            .h(px(SEARCH_INPUT_HEIGHT))
            .px(px(CONTENT_PADDING_X))
            .py(px(10.0))
            .border_b_1()
            .border_color(rgba(hex_with_alpha(colors.border, ALPHA_KEYCAP_BG)))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.0))
            // Keyboard hint
            .child(
                div()
                    .w(px(24.0))
                    .text_xs()
                    .text_color(rgba(hex_with_alpha(colors.text_dimmed, 0xFF)))
                    .child("\u{2318}K"),
            )
            // Inner input field
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(SEARCH_INPUT_INNER_WIDTH))
                    .h(px(SEARCH_INPUT_INNER_HEIGHT))
                    .px(px(8.0))
                    .py(px(4.0))
                    .bg(rgba(hex_with_alpha(colors.background, input_bg_alpha)))
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(rgba(hex_with_alpha(colors.accent, border_alpha)))
                    .flex()
                    .flex_row()
                    .items_center()
                    .text_sm()
                    // Text content
                    .child(
                        div()
                            .text_color(rgba(hex_with_alpha(colors.text_primary, 0xFF)))
                            .child(search_text.clone()),
                    )
                    // Blinking cursor
                    .child(
                        div()
                            .w(px(2.0))
                            .h(px(16.0))
                            .ml(px(2.0))
                            .rounded(px(1.0))
                            .when(cursor_visible, |d| {
                                d.bg(rgba(hex_with_alpha(colors.accent, 0xFF)))
                            }),
                    )
                    // Placeholder when empty
                    .when(search_text.is_empty(), |d| {
                        d.child(
                            div()
                                .text_color(rgba(hex_with_alpha(colors.text_dimmed, 0xFF)))
                                .child("Search actions..."),
                        )
                    }),
            )
    }

    /// Render a single action item
    fn render_action_item(
        &self,
        idx: usize,
        action: &CommandBarAction,
        is_selected: bool,
        prev_section: Option<&str>,
    ) -> impl IntoElement {
        let colors = self.colors;
        let action_id = action.id;
        let icon_path = action.icon.external_path();
        let name = action.name.to_string();
        let shortcut = action.shortcut;
        let is_section_start = prev_section
            .map(|s| s != action.section)
            .unwrap_or(idx == 0);

        // Selection/hover colors (white base + alpha for vibrancy)
        let selected_bg = rgba(hex_with_alpha(0xFFFFFF, ALPHA_SELECTED));
        let hover_bg = rgba(hex_with_alpha(0xFFFFFF, ALPHA_HOVER));

        // Build the item
        let mut item = div()
            .id(SharedString::from(format!("action-{}", action_id)))
            .w_full()
            .h(px(ACTION_ITEM_HEIGHT))
            .px(px(ACTION_ROW_INSET))
            .py(px(2.0))
            .flex()
            .flex_col()
            .justify_center();

        // Add section separator if this is a new section
        if is_section_start && idx > 0 {
            item = item
                .border_t_1()
                .border_color(rgba(hex_with_alpha(colors.border, ALPHA_SEPARATOR)));
        }

        // Inner row (pill-style selection)
        let mut inner_row = div()
            .w_full()
            .flex_1()
            .flex()
            .flex_row()
            .items_center()
            .px(px(CONTENT_PADDING_X))
            .rounded(px(SELECTION_RADIUS))
            .cursor_pointer();

        if is_selected {
            inner_row = inner_row.bg(selected_bg);
        } else {
            inner_row = inner_row.hover(|s| s.bg(hover_bg));
        }

        // Icon
        inner_row = inner_row.child(gpui::svg().path(icon_path).size(px(16.0)).text_color(rgba(
            hex_with_alpha(
                if is_selected {
                    colors.text_primary
                } else {
                    colors.text_secondary
                },
                0xFF,
            ),
        )));

        // Title
        inner_row = inner_row.child(
            div()
                .flex_1()
                .ml(px(12.0))
                .text_sm()
                .text_color(rgba(hex_with_alpha(colors.text_primary, 0xFF)))
                .when(is_selected, |d| d.font_weight(gpui::FontWeight::MEDIUM))
                .child(name),
        );

        // Keycap badges for shortcut
        if let Some(shortcut) = shortcut {
            let keycaps = Self::parse_shortcut_keycaps(shortcut);
            let mut keycap_row = div().flex().flex_row().items_center().gap(px(KEYCAP_GAP));

            for keycap in keycaps {
                keycap_row = keycap_row.child(
                    div()
                        .min_w(px(KEYCAP_MIN_WIDTH))
                        .h(px(KEYCAP_HEIGHT))
                        .px(px(KEYCAP_PADDING_X))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba(hex_with_alpha(colors.border, ALPHA_KEYCAP_BG)))
                        .border_1()
                        .border_color(rgba(hex_with_alpha(colors.border, ALPHA_KEYCAP_BORDER)))
                        .rounded(px(KEYCAP_RADIUS))
                        .text_xs()
                        .text_color(rgba(hex_with_alpha(colors.text_dimmed, 0xFF)))
                        .child(keycap),
                );
            }

            inner_row = inner_row.child(keycap_row);
        }

        item.child(inner_row)
    }

    /// Render the footer with keyboard hints
    fn render_footer(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;

        div()
            .w_full()
            .h(px(FOOTER_HEIGHT))
            .px(px(CONTENT_PADDING_X))
            .border_t_1()
            .border_color(rgba(hex_with_alpha(colors.border, ALPHA_KEYCAP_BG)))
            .flex()
            .items_center()
            .gap(px(16.0))
            .text_xs()
            .text_color(rgba(hex_with_alpha(colors.text_dimmed, 0xFF)))
            // Navigate hint
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child("\u{2191}\u{2193}")
                    .child("Navigate"),
            )
            // Select hint
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child("\u{21B5}")
                    .child("Select"),
            )
            // Close hint
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child("esc")
                    .child("Close"),
            )
    }

    /// Calculate the dynamic height for the popup
    fn calculate_height(&self) -> f32 {
        let num_items = self.filtered_indices.len();
        let items_height =
            (num_items as f32 * ACTION_ITEM_HEIGHT).min(POPUP_MAX_HEIGHT - SEARCH_INPUT_HEIGHT);
        items_height + SEARCH_INPUT_HEIGHT + FOOTER_HEIGHT + 2.0 // +2 for borders
    }
}

impl Focusable for CommandBarDialog {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CommandBarDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let filtered_len = self.filtered_indices.len();
        let total_height = self.calculate_height();
        let selected_index = self.selected_index;

        // Clone data for the closure
        let actions = self.actions.clone();
        let filtered_indices = self.filtered_indices.clone();

        // Build the list using uniform_list for virtualization
        let list = uniform_list(
            "command-bar-list",
            filtered_len,
            cx.processor(move |this: &mut Self, range: Range<usize>, _window, _cx| {
                range
                    .map(|visible_idx| {
                        let action_idx = filtered_indices[visible_idx];
                        let action = &actions[action_idx];
                        let is_selected = visible_idx == selected_index;

                        // Get previous section for separator logic
                        let prev_section = if visible_idx > 0 {
                            let prev_action_idx = filtered_indices[visible_idx - 1];
                            Some(actions[prev_action_idx].section)
                        } else {
                            None
                        };

                        this.render_action_item(visible_idx, action, is_selected, prev_section)
                            .into_any_element()
                    })
                    .collect()
            }),
        )
        .flex_1()
        .w_full()
        .track_scroll(&self.scroll_handle);

        // Main container
        div()
            .w(px(POPUP_WIDTH))
            .h(px(total_height))
            .bg(rgba(hex_with_alpha(colors.background, 0xE6))) // 90% opacity for vibrancy
            .rounded(px(POPUP_RADIUS))
            .border_1()
            .border_color(rgba(hex_with_alpha(colors.border, ALPHA_KEYCAP_BG)))
            .overflow_hidden()
            .flex()
            .flex_col()
            .track_focus(&self.focus_handle)
            // Search input
            .child(self.render_search_input(cx))
            // Action list
            .child(list)
            // Footer
            .child(self.render_footer(cx))
    }
}

/// Helper to create hex color with alpha
fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
    (hex << 8) | (alpha as u32)
}
