//! StoryBrowser - Main UI for browsing and previewing stories
//!
//! Features:
//! - Left sidebar with searchable story list grouped by category
//! - Right panel showing selected story preview
//! - Theme and design variant controls in toolbar
//! - Keyboard navigation support
//! - Screenshot capture (Cmd+Shift+S)

use gpui::*;
use std::fs;
use std::path::PathBuf;

use crate::designs::DesignVariant;
use crate::storybook::{all_categories, all_stories, StoryEntry};

/// Main browser view for the storybook
pub struct StoryBrowser {
    stories: Vec<&'static StoryEntry>,
    selected_index: usize,
    filter: String,
    theme_name: String,
    design_variant: DesignVariant,
    focus_handle: FocusHandle,
    screenshot_dir: PathBuf,
}

impl StoryBrowser {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let stories: Vec<_> = all_stories().collect();

        // Set up screenshot directory
        let screenshot_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("test-screenshots");
        fs::create_dir_all(&screenshot_dir).ok();

        Self {
            stories,
            selected_index: 0,
            filter: String::new(),
            theme_name: "Default".to_string(),
            design_variant: DesignVariant::Default,
            focus_handle: cx.focus_handle(),
            screenshot_dir,
        }
    }

    pub fn load_theme(&mut self, theme_name: &str) {
        // For now, just update the name
        self.theme_name = theme_name.to_string();
    }

    pub fn select_story(&mut self, story_id: &str) {
        if let Some(pos) = self.stories.iter().position(|s| s.story.id() == story_id) {
            self.selected_index = pos;
        }
    }

    pub fn set_design_variant(&mut self, variant: DesignVariant) {
        self.design_variant = variant;
    }

    fn filtered_stories(&self) -> Vec<&'static StoryEntry> {
        if self.filter.is_empty() {
            self.stories.clone()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.stories
                .iter()
                .filter(|s| {
                    s.story.name().to_lowercase().contains(&filter_lower)
                        || s.story.category().to_lowercase().contains(&filter_lower)
                })
                .copied()
                .collect()
        }
    }

    fn render_search_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let filter = self.filter.clone();
        div()
            .p_2()
            .border_b_1()
            .border_color(rgb(theme.colors.ui.border))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .bg(rgb(theme.colors.background.title_bar))
                    .rounded_md()
                    .child(
                        // Search icon
                        div().text_color(rgb(theme.colors.text.dimmed)).child("🔍"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(if filter.is_empty() {
                                rgb(theme.colors.text.dimmed)
                            } else {
                                rgb(theme.colors.text.secondary)
                            })
                            .child(if filter.is_empty() {
                                "Search stories...".to_string()
                            } else {
                                filter
                            }),
                    ),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_this, _event, _window, _cx| {
                    // TODO: Focus search input and enable text input
                }),
            )
    }

    fn render_story_list(
        &self,
        filtered: &[&'static StoryEntry],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let categories = all_categories();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .children(categories.into_iter().map(|category| {
                let category_stories: Vec<_> = filtered
                    .iter()
                    .filter(|s| s.story.category() == category)
                    .copied()
                    .collect();

                if category_stories.is_empty() {
                    return div().into_any_element();
                }

                div()
                    .flex()
                    .flex_col()
                    .child(
                        // Category header
                        div()
                            .px_3()
                            .py_2()
                            .text_xs()
                            .text_color(rgb(theme.colors.text.tertiary))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(category.to_uppercase()),
                    )
                    .children(category_stories.into_iter().map(|story| {
                        let is_selected = self
                            .stories
                            .iter()
                            .position(|s| s.story.id() == story.story.id())
                            == Some(self.selected_index);

                        let story_id = story.story.id();

                        let base = div()
                            .id(ElementId::Name(story_id.into()))
                            .px_3()
                            .py_1()
                            .cursor_pointer()
                            .text_sm()
                            .rounded_sm()
                            .child(story.story.name())
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                if let Some(pos) =
                                    this.stories.iter().position(|s| s.story.id() == story_id)
                                {
                                    this.selected_index = pos;
                                    cx.notify();
                                }
                            }));

                        if is_selected {
                            base.bg(rgb(theme.colors.ui.info))
                                .text_color(rgb(theme.colors.text.primary))
                        } else {
                            base.text_color(rgb(theme.colors.text.secondary))
                                .hover(|s| s.bg(rgb(theme.colors.ui.border)))
                        }
                    }))
                    .into_any_element()
            }))
    }

    fn render_toolbar(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .border_b_1()
            .border_color(rgb(theme.colors.ui.border))
            .bg(rgb(theme.colors.background.title_bar))
            .child(
                // Left: Story info
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(theme.colors.text.primary))
                            .child(
                                self.stories
                                    .get(self.selected_index)
                                    .map(|s| s.story.name())
                                    .unwrap_or("No story selected"),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme.colors.text.dimmed))
                            .child(
                                self.stories
                                    .get(self.selected_index)
                                    .map(|s| format!("({})", s.story.category()))
                                    .unwrap_or_default(),
                            ),
                    ),
            )
            .child(
                // Right: Theme & Design controls
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.colors.text.tertiary))
                                    .child("Theme:"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .text_color(rgb(theme.colors.text.secondary))
                                    .bg(rgb(theme.colors.background.title_bar))
                                    .rounded_sm()
                                    .cursor_pointer()
                                    .child(self.theme_name.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.colors.text.tertiary))
                                    .child("Design:"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .text_xs()
                                    .text_color(rgb(theme.colors.text.secondary))
                                    .bg(rgb(theme.colors.background.title_bar))
                                    .rounded_sm()
                                    .cursor_pointer()
                                    .child(format!("{:?}", self.design_variant)),
                            ),
                    ),
            )
    }

    fn render_preview(&self) -> AnyElement {
        let theme = crate::theme::get_cached_theme();
        if let Some(story) = self.stories.get(self.selected_index) {
            story.story.render()
        } else {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .text_color(rgb(theme.colors.text.dimmed))
                .child("No story selected")
                .into_any_element()
        }
    }

    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_stories();
        if filtered.is_empty() {
            return;
        }

        // Find current story in filtered list
        if let Some(current) = self.stories.get(self.selected_index) {
            if let Some(pos) = filtered
                .iter()
                .position(|s| s.story.id() == current.story.id())
            {
                if pos > 0 {
                    // Move to previous in filtered list
                    let prev_story = filtered[pos - 1];
                    if let Some(main_pos) = self
                        .stories
                        .iter()
                        .position(|s| s.story.id() == prev_story.story.id())
                    {
                        self.selected_index = main_pos;
                        cx.notify();
                    }
                }
            }
        }
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_stories();
        if filtered.is_empty() {
            return;
        }

        // Find current story in filtered list
        if let Some(current) = self.stories.get(self.selected_index) {
            if let Some(pos) = filtered
                .iter()
                .position(|s| s.story.id() == current.story.id())
            {
                if pos < filtered.len() - 1 {
                    // Move to next in filtered list
                    let next_story = filtered[pos + 1];
                    if let Some(main_pos) = self
                        .stories
                        .iter()
                        .position(|s| s.story.id() == next_story.story.id())
                    {
                        self.selected_index = main_pos;
                        cx.notify();
                    }
                }
            }
        }
    }

    /// Capture a screenshot of the current storybook window
    fn capture_screenshot(&self, _window: &mut Window, _cx: &mut Context<Self>) {
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;
        use xcap::Window as XCapWindow;

        let story_id = self
            .stories
            .get(self.selected_index)
            .map(|s| s.story.id())
            .unwrap_or("unknown");

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let filename = format!("storybook-{}-{}.png", story_id, timestamp);
        let filepath = self.screenshot_dir.join(&filename);

        // Find storybook window using xcap
        match XCapWindow::all() {
            Ok(windows) => {
                for win in windows {
                    let title = win.title().unwrap_or_default();
                    let app_name = win.app_name().unwrap_or_default();

                    // Match storybook window
                    if title.contains("Storybook") || app_name.contains("storybook") {
                        match win.capture_image() {
                            Ok(img) => {
                                let width = img.width();
                                let height = img.height();
                                let rgba_data = img.into_raw();

                                // Encode as PNG
                                let mut png_data = Vec::new();
                                let encoder = PngEncoder::new(&mut png_data);
                                if let Err(e) = encoder.write_image(
                                    &rgba_data,
                                    width,
                                    height,
                                    image::ExtendedColorType::Rgba8,
                                ) {
                                    eprintln!("[SCREENSHOT ERROR] PNG encode failed: {}", e);
                                    return;
                                }

                                // Save to file
                                if let Err(e) = fs::write(&filepath, &png_data) {
                                    eprintln!("[SCREENSHOT ERROR] Save failed: {}", e);
                                } else {
                                    eprintln!("[SCREENSHOT] Saved: {}", filepath.display());
                                }
                                return;
                            }
                            Err(e) => {
                                eprintln!("[SCREENSHOT ERROR] Capture failed: {}", e);
                            }
                        }
                    }
                }
                eprintln!("[SCREENSHOT ERROR] Storybook window not found");
            }
            Err(e) => {
                eprintln!("[SCREENSHOT ERROR] Failed to enumerate windows: {}", e);
            }
        }
    }
}

impl Render for StoryBrowser {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = crate::theme::get_cached_theme();
        let filtered = self.filtered_stories();

        // Render the story preview - stories are stateless so no App context needed
        let preview = self.render_preview();

        div()
            .id("story-browser")
            .key_context("StoryBrowser")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                let modifiers = &event.keystroke.modifiers;

                // Cmd+Shift+S for screenshot
                if key == "s" && modifiers.platform && modifiers.shift {
                    this.capture_screenshot(window, cx);
                    return;
                }

                match key {
                    "up" | "arrowup" => this.move_selection_up(cx),
                    "down" | "arrowdown" => this.move_selection_down(cx),
                    _ => {}
                }
            }))
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(theme.colors.background.main))
            .text_color(rgb(theme.colors.text.secondary))
            // Left sidebar: story list
            .child(
                div()
                    .w(px(280.))
                    .border_r_1()
                    .border_color(rgb(theme.colors.ui.border))
                    .flex()
                    .flex_col()
                    .bg(rgb(theme.colors.background.title_bar))
                    .child(
                        // Header
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(rgb(theme.colors.ui.border))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(theme.colors.text.primary))
                                    .child("Script Kit Storybook"),
                            ),
                    )
                    .child(self.render_search_bar(cx))
                    // Scrollable story list
                    .child(
                        div()
                            .id("story-list-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .child(self.render_story_list(&filtered, cx)),
                    ),
            )
            // Right panel: story preview
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(self.render_toolbar(cx))
                    // Scrollable preview area
                    .child(
                        div()
                            .id("story-preview-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .child(preview),
                    ),
            )
    }
}

impl Focusable for StoryBrowser {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
