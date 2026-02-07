//! SelectPrompt - Multi-select from choices
//!
//! Features:
//! - Select multiple items from a list
//! - Toggle selection with Cmd/Ctrl+Space
//! - Filter choices by typing
//! - Submit selected items

use gpui::{
    div, prelude::*, px, rgb, uniform_list, AnyElement, Context, FocusHandle, Focusable, Render,
    ScrollStrategy, SharedString, UniformListScrollHandle, Window,
};
use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;

use crate::components::{
    Density, ItemState, LeadingContent, TextContent, TrailingContent, UnifiedListItem,
    UnifiedListItemColors,
};
use crate::designs::{get_tokens, DesignColors, DesignVariant};
use crate::list_item::LIST_ITEM_HEIGHT;
use crate::logging;
use crate::panel::PROMPT_INPUT_FIELD_HEIGHT;
use crate::protocol::{generate_semantic_id, Choice};
use crate::scripts;
use crate::theme;
use crate::ui_foundation::get_vibrancy_background;

use super::SubmitCallback;

/// SelectPrompt - Multi-select from choices
///
/// Allows selecting multiple items from a list of choices.
/// Use Cmd/Ctrl+Space to toggle selection, Enter to submit selected items.
pub struct SelectPrompt {
    /// Unique ID for this prompt instance
    pub id: String,
    /// Placeholder text for the search input
    pub placeholder: Option<String>,
    /// Available choices
    pub choices: Vec<Choice>,
    /// Cached searchable/indexed choice data to reduce refilter work
    choice_index: Vec<SelectChoiceIndex>,
    /// Indices of selected choices
    pub selected: HashSet<usize>,
    /// Filtered choice indices (for display)
    pub filtered_choices: Vec<usize>,
    /// Currently focused index in filtered list
    pub focused_index: usize,
    /// Filter text
    pub filter_text: String,
    /// Whether multiple selection is allowed
    pub multiple: bool,
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Callback when user submits
    pub on_submit: SubmitCallback,
    /// Theme for styling
    pub theme: Arc<theme::Theme>,
    /// Design variant for styling
    pub design_variant: DesignVariant,
    /// Scroll handle for virtualized choices list
    pub list_scroll_handle: UniformListScrollHandle,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ChoiceDisplayMetadata {
    description: Option<String>,
    item_type: Option<String>,
    shortcut: Option<String>,
    last_run: Option<String>,
}

#[derive(Debug, Clone)]
struct SelectChoiceIndex {
    metadata: ChoiceDisplayMetadata,
    name_lower: String,
    description_lower: String,
    value_lower: String,
    item_type_lower: String,
    last_run_lower: String,
    shortcut_lower: String,
    stable_semantic_id: String,
}

impl SelectChoiceIndex {
    fn from_choice(choice: &Choice, source_index: usize) -> Self {
        let metadata = ChoiceDisplayMetadata::from_choice(choice);

        SelectChoiceIndex {
            name_lower: choice.name.to_lowercase(),
            description_lower: choice
                .description
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
            value_lower: choice.value.to_lowercase(),
            item_type_lower: metadata
                .item_type
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
            last_run_lower: metadata
                .last_run
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
            shortcut_lower: metadata
                .shortcut
                .as_deref()
                .unwrap_or_default()
                .to_lowercase(),
            stable_semantic_id: fallback_select_semantic_id(source_index, &choice.value),
            metadata,
        }
    }
}

impl ChoiceDisplayMetadata {
    fn from_choice(choice: &Choice) -> Self {
        let mut metadata = Self::default();
        let mut description_parts = Vec::new();

        if let Some(description) = choice.description.as_deref() {
            for token in description
                .split(['•', '|', '\n'])
                .map(str::trim)
                .filter(|token| !token.is_empty())
            {
                if metadata.shortcut.is_none() {
                    if let Some(shortcut) = extract_shortcut_token(token) {
                        metadata.shortcut = Some(shortcut);
                        continue;
                    }
                }

                if metadata.item_type.is_none() {
                    if let Some(item_type) = extract_script_type_token(token) {
                        metadata.item_type = Some(item_type);
                        continue;
                    }
                }

                if metadata.last_run.is_none() {
                    if let Some(last_run) = extract_last_run_token(token) {
                        metadata.last_run = Some(last_run);
                        continue;
                    }
                }

                description_parts.push(token.to_string());
            }
        }

        if !description_parts.is_empty() {
            metadata.description = Some(description_parts.join(" • "));
        }

        if metadata.item_type.is_none() {
            metadata.item_type = infer_script_type(choice);
        }

        metadata
    }

    fn subtitle_text(&self) -> Option<String> {
        let mut parts = Vec::new();

        if let Some(description) = self.description.as_deref() {
            if !description.is_empty() {
                parts.push(description.to_string());
            }
        }

        let mut metadata_parts = Vec::new();
        if let Some(item_type) = self.item_type.as_deref() {
            metadata_parts.push(item_type.to_string());
        }
        if let Some(last_run) = self.last_run.as_deref() {
            metadata_parts.push(last_run.to_string());
        }

        if !metadata_parts.is_empty() {
            parts.push(metadata_parts.join(" · "));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" • "))
        }
    }
}

fn infer_script_type(choice: &Choice) -> Option<String> {
    let name_lower = choice.name.to_lowercase();
    let value_lower = choice.value.to_lowercase();
    let description_lower = choice
        .description
        .as_deref()
        .unwrap_or_default()
        .to_lowercase();
    let combined = format!("{} {} {}", name_lower, description_lower, value_lower);

    if combined.contains("scriptlet")
        || value_lower.contains(".md#")
        || value_lower.contains("/snippets/")
    {
        return Some("Scriptlet".to_string());
    }

    if combined.contains("extension")
        || value_lower.contains("/extensions/")
        || value_lower.contains("/extension/")
    {
        return Some("Extension".to_string());
    }

    if combined.contains("agent") {
        return Some("Agent".to_string());
    }

    let script_extensions = [
        ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".sh", ".py", ".rb", ".ps1", ".zsh", ".bash",
    ];
    if combined.contains("script")
        || script_extensions
            .iter()
            .any(|ext| value_lower.ends_with(ext) || value_lower.contains(&format!("{ext}#")))
    {
        return Some("Script".to_string());
    }

    None
}

fn looks_like_shortcut(token: &str) -> bool {
    let lower = token.to_lowercase();
    if token.len() > 28 || token.is_empty() {
        return false;
    }

    let has_modifier = [
        "cmd", "command", "ctrl", "control", "alt", "option", "shift", "meta", "⌘", "⌃", "⌥", "⇧",
    ]
    .iter()
    .any(|needle| lower.contains(needle));

    let has_key_like = token.chars().any(|ch| ch.is_ascii_alphanumeric())
        || token.contains('↵')
        || token.contains('⌫')
        || token.contains('↑')
        || token.contains('↓')
        || token.contains('←')
        || token.contains('→');

    has_modifier && has_key_like
}

fn normalize_shortcut_label(raw: &str) -> String {
    if raw.chars().any(|ch| "⌘⌥⌃⇧↵⌫↑↓←→".contains(ch)) {
        return raw.trim().replace(' ', "");
    }

    let mut normalized = raw.to_lowercase();
    normalized = normalized
        .replace("command", "cmd")
        .replace("control", "ctrl")
        .replace("option", "alt");

    normalized
        .split(['+', '-', ' '])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| match part {
            "cmd" | "meta" => "⌘".to_string(),
            "ctrl" => "⌃".to_string(),
            "alt" | "opt" => "⌥".to_string(),
            "shift" => "⇧".to_string(),
            "enter" | "return" => "↵".to_string(),
            "delete" | "backspace" => "⌫".to_string(),
            "up" | "arrowup" => "↑".to_string(),
            "down" | "arrowdown" => "↓".to_string(),
            "left" | "arrowleft" => "←".to_string(),
            "right" | "arrowright" => "→".to_string(),
            _ => part.to_ascii_uppercase(),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn extract_shortcut_token(token: &str) -> Option<String> {
    let lower = token.to_lowercase();

    if lower.starts_with("shortcut")
        || lower.starts_with("key")
        || lower.starts_with("hotkey")
        || lower.starts_with("shortcut ")
    {
        let shortcut_value = token
            .split_once(':')
            .or_else(|| token.split_once('='))
            .map(|(_, value)| value.trim())
            .unwrap_or_default();
        if !shortcut_value.is_empty() {
            return Some(normalize_shortcut_label(shortcut_value));
        }
    }

    if looks_like_shortcut(token) {
        return Some(normalize_shortcut_label(token));
    }

    None
}

fn extract_script_type_token(token: &str) -> Option<String> {
    let lower = token.trim().to_lowercase();
    if lower == "script" || lower.starts_with("type: script") {
        return Some("Script".to_string());
    }
    if lower == "scriptlet" || lower.starts_with("type: scriptlet") {
        return Some("Scriptlet".to_string());
    }
    if lower == "extension" || lower.starts_with("type: extension") {
        return Some("Extension".to_string());
    }
    if lower == "agent" || lower.starts_with("type: agent") {
        return Some("Agent".to_string());
    }
    None
}

fn extract_last_run_token(token: &str) -> Option<String> {
    let trimmed = token.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("last run") || lower.starts_with("last ran") {
        return Some(trimmed.to_string());
    }
    if (lower.starts_with("ran ") || lower.contains(" ago"))
        && (lower.contains("run") || lower.contains("ran"))
    {
        return Some(trimmed.to_string());
    }
    None
}

fn score_field(
    nucleo: &mut scripts::NucleoCtx,
    haystack: &str,
    haystack_lower: &str,
    query_lower: &str,
    field_boost: u32,
) -> Option<u32> {
    if haystack.is_empty() {
        return None;
    }

    let mut score = nucleo.score(haystack)?;

    if haystack_lower == query_lower {
        score += 600;
    } else if haystack_lower.starts_with(query_lower) {
        score += 320;
    } else if haystack_lower.contains(query_lower) {
        score += 140;
    }

    Some(score + field_boost)
}

fn score_choice_for_filter(
    choice: &Choice,
    indexed_choice: &SelectChoiceIndex,
    query_lower: &str,
    nucleo: &mut scripts::NucleoCtx,
) -> Option<u32> {
    let mut best_score: Option<u32> = None;

    for score in [
        score_field(
            nucleo,
            &choice.name,
            &indexed_choice.name_lower,
            query_lower,
            900,
        ),
        score_field(
            nucleo,
            choice.description.as_deref().unwrap_or_default(),
            &indexed_choice.description_lower,
            query_lower,
            450,
        ),
        score_field(
            nucleo,
            &choice.value,
            &indexed_choice.value_lower,
            query_lower,
            260,
        ),
        score_field(
            nucleo,
            indexed_choice
                .metadata
                .item_type
                .as_deref()
                .unwrap_or_default(),
            &indexed_choice.item_type_lower,
            query_lower,
            240,
        ),
        score_field(
            nucleo,
            indexed_choice
                .metadata
                .last_run
                .as_deref()
                .unwrap_or_default(),
            &indexed_choice.last_run_lower,
            query_lower,
            180,
        ),
        score_field(
            nucleo,
            indexed_choice
                .metadata
                .shortcut
                .as_deref()
                .unwrap_or_default(),
            &indexed_choice.shortcut_lower,
            query_lower,
            120,
        ),
    ]
    .into_iter()
    .flatten()
    {
        best_score = Some(best_score.map_or(score, |current| current.max(score)));
    }

    best_score
}

fn char_indices_to_byte_ranges(text: &str, indices: &[usize]) -> Vec<Range<usize>> {
    if indices.is_empty() {
        return Vec::new();
    }

    let mut offsets: Vec<usize> = text.char_indices().map(|(byte_idx, _)| byte_idx).collect();
    offsets.push(text.len());

    let mut ranges: Vec<Range<usize>> = Vec::new();
    for &char_index in indices {
        if char_index + 1 >= offsets.len() {
            continue;
        }
        let start = offsets[char_index];
        let end = offsets[char_index + 1];
        if start >= end {
            continue;
        }

        if let Some(last) = ranges.last_mut() {
            if last.end == start {
                last.end = end;
                continue;
            }
        }
        ranges.push(start..end);
    }

    ranges
}

fn highlighted_choice_title(choice_name: &str, query: &str) -> TextContent {
    let trimmed_query = query.trim();
    if trimmed_query.is_empty() {
        return TextContent::plain(choice_name.to_string());
    }

    let query_lower = trimmed_query.to_lowercase();
    let (matched, indices) =
        crate::scripts::search::fuzzy_match_with_indices_ascii(choice_name, &query_lower);
    if !matched || indices.is_empty() {
        return TextContent::plain(choice_name.to_string());
    }

    let ranges = char_indices_to_byte_ranges(choice_name, &indices);
    if ranges.is_empty() {
        TextContent::plain(choice_name.to_string())
    } else {
        TextContent::highlighted(choice_name.to_string(), ranges)
    }
}

fn choice_selection_indicator(is_multiple: bool, is_selected: bool) -> &'static str {
    if is_multiple {
        if is_selected {
            "☑"
        } else {
            "☐"
        }
    } else if is_selected {
        "●"
    } else {
        "○"
    }
}

fn fallback_select_semantic_id(source_index: usize, value: &str) -> String {
    generate_semantic_id("select", source_index, value)
}

fn should_append_to_filter(ch: char) -> bool {
    !ch.is_control()
}

fn are_all_filtered_selected(
    selected_indices: &HashSet<usize>,
    filtered_indices: &[usize],
) -> bool {
    !filtered_indices.is_empty()
        && filtered_indices
            .iter()
            .all(|idx| selected_indices.contains(idx))
}

fn toggle_filtered_selection(selected_indices: &mut HashSet<usize>, filtered_indices: &[usize]) {
    if are_all_filtered_selected(selected_indices, filtered_indices) {
        for idx in filtered_indices {
            selected_indices.remove(idx);
        }
    } else {
        selected_indices.extend(filtered_indices.iter().copied());
    }
}

fn resolve_submission_indices(
    is_multiple: bool,
    selected_indices: &[usize],
    focused_choice_index: Option<usize>,
) -> Vec<usize> {
    if !selected_indices.is_empty() {
        return selected_indices.to_vec();
    }

    if !is_multiple {
        return focused_choice_index.into_iter().collect();
    }

    Vec::new()
}

fn resolve_search_box_bg_hex(
    theme: &theme::Theme,
    design_variant: DesignVariant,
    design_colors: &DesignColors,
) -> u32 {
    if design_variant == DesignVariant::Default {
        theme.colors.background.search_box
    } else {
        design_colors.background_secondary
    }
}

impl SelectPrompt {
    pub fn new(
        id: String,
        placeholder: Option<String>,
        choices: Vec<Choice>,
        multiple: bool,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<theme::Theme>,
    ) -> Self {
        logging::log(
            "PROMPTS",
            &format!(
                "SelectPrompt::new with {} choices (multiple: {})",
                choices.len(),
                multiple
            ),
        );

        let filtered_choices: Vec<usize> = (0..choices.len()).collect();
        let choice_index: Vec<SelectChoiceIndex> = choices
            .iter()
            .enumerate()
            .map(|(source_index, choice)| SelectChoiceIndex::from_choice(choice, source_index))
            .collect();

        SelectPrompt {
            id,
            placeholder,
            choices,
            choice_index,
            selected: HashSet::new(),
            filtered_choices,
            focused_index: 0,
            filter_text: String::new(),
            multiple,
            focus_handle,
            on_submit,
            theme,
            design_variant: DesignVariant::Default,
            list_scroll_handle: UniformListScrollHandle::new(),
        }
    }

    /// Refilter choices based on current filter_text
    fn refilter(&mut self) {
        let trimmed_filter = self.filter_text.trim();
        if trimmed_filter.is_empty() {
            self.filtered_choices = (0..self.choices.len()).collect();
            self.focused_index = 0;
            return;
        }

        let query_lower = trimmed_filter.to_lowercase();
        let mut nucleo = scripts::NucleoCtx::new(trimmed_filter);
        let mut scored_matches: Vec<(usize, u32)> = self
            .choices
            .iter()
            .enumerate()
            .filter_map(|(idx, choice)| {
                score_choice_for_filter(choice, &self.choice_index[idx], &query_lower, &mut nucleo)
                    .map(|score| (idx, score))
            })
            .collect();

        scored_matches.sort_by(|(a_idx, a_score), (b_idx, b_score)| {
            b_score.cmp(a_score).then_with(|| {
                self.choice_index[*a_idx]
                    .name_lower
                    .cmp(&self.choice_index[*b_idx].name_lower)
            })
        });

        self.filtered_choices = scored_matches.into_iter().map(|(idx, _)| idx).collect();
        self.focused_index = 0;
    }

    /// Set the filter text programmatically
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if self.filter_text == text {
            return;
        }

        self.filter_text = text;
        self.refilter();
        self.list_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    /// Toggle selection of currently focused item
    fn toggle_selection(&mut self, cx: &mut Context<Self>) {
        if let Some(&choice_idx) = self.filtered_choices.get(self.focused_index) {
            if self.multiple {
                if self.selected.contains(&choice_idx) {
                    self.selected.remove(&choice_idx);
                } else {
                    self.selected.insert(choice_idx);
                }
            } else {
                // Single select mode - replace selection
                self.selected.clear();
                self.selected.insert(choice_idx);
            }
            cx.notify();
        }
    }

    /// Submit selected items as JSON array
    fn submit(&mut self) {
        let mut selected_indices: Vec<usize> = self.selected.iter().copied().collect();
        selected_indices.sort_unstable();
        let focused_choice_index = self.filtered_choices.get(self.focused_index).copied();
        let resolved_indices =
            resolve_submission_indices(self.multiple, &selected_indices, focused_choice_index);

        let selected_values: Vec<String> = resolved_indices
            .iter()
            .filter_map(|&idx| self.choices.get(idx).map(|choice| choice.value.clone()))
            .collect();

        let json_str = serde_json::to_string(&selected_values).unwrap_or_else(|_| "[]".to_string());
        (self.on_submit)(self.id.clone(), Some(json_str));
    }

    /// Cancel - submit None
    fn submit_cancel(&mut self) {
        (self.on_submit)(self.id.clone(), None);
    }

    /// Move focus up
    fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.focused_index > 0 {
            self.focused_index -= 1;
            self.list_scroll_handle
                .scroll_to_item(self.focused_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    /// Move focus down
    fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.focused_index < self.filtered_choices.len().saturating_sub(1) {
            self.focused_index += 1;
            self.list_scroll_handle
                .scroll_to_item(self.focused_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    /// Handle character input
    fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
        if !should_append_to_filter(ch) {
            return;
        }
        self.filter_text.push(ch);
        self.refilter();
        self.list_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    /// Handle backspace
    fn handle_backspace(&mut self, cx: &mut Context<Self>) {
        if !self.filter_text.is_empty() {
            self.filter_text.pop();
            self.refilter();
            self.list_scroll_handle
                .scroll_to_item(0, ScrollStrategy::Top);
            cx.notify();
        }
    }

    /// Select all choices (Ctrl+A)
    fn toggle_select_all_filtered(&mut self, cx: &mut Context<Self>) {
        if !self.multiple {
            return;
        }

        toggle_filtered_selection(&mut self.selected, &self.filtered_choices);
        cx.notify();
    }
}

impl Focusable for SelectPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SelectPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tokens = get_tokens(self.design_variant);
        let colors = tokens.colors();
        let spacing = tokens.spacing();

        let handle_key = cx.listener(
            |this: &mut Self,
             event: &gpui::KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                let key_str = event.keystroke.key.to_lowercase();
                let has_ctrl = event.keystroke.modifiers.platform; // Cmd on macOS, Ctrl on others

                // Handle Ctrl/Cmd+A for select all
                if has_ctrl && key_str == "a" {
                    this.toggle_select_all_filtered(cx);
                    return;
                }

                match key_str.as_str() {
                    "up" | "arrowup" => this.move_up(cx),
                    "down" | "arrowdown" => this.move_down(cx),
                    "space" | " " => {
                        if has_ctrl {
                            this.toggle_selection(cx);
                        } else {
                            this.handle_char(' ', cx);
                        }
                    }
                    "enter" | "return" => this.submit(),
                    "escape" | "esc" => this.submit_cancel(),
                    "backspace" => this.handle_backspace(cx),
                    _ => {
                        if let Some(ref key_char) = event.keystroke.key_char {
                            if let Some(ch) = key_char.chars().next() {
                                if should_append_to_filter(ch) {
                                    this.handle_char(ch, cx);
                                }
                            }
                        }
                    }
                }
            },
        );

        // VIBRANCY: Use foundation helper - returns None when vibrancy enabled (let Root handle bg)
        let vibrancy_bg = get_vibrancy_background(&self.theme);

        let (_main_bg, text_color, muted_color, border_color) =
            if self.design_variant == DesignVariant::Default {
                (
                    rgb(self.theme.colors.background.main),
                    rgb(self.theme.colors.text.secondary),
                    rgb(self.theme.colors.text.muted),
                    rgb(self.theme.colors.ui.border),
                )
            } else {
                (
                    rgb(colors.background),
                    rgb(colors.text_secondary),
                    rgb(colors.text_muted),
                    rgb(colors.border),
                )
            };
        let search_box_bg = rgb(resolve_search_box_bg_hex(
            &self.theme,
            self.design_variant,
            &colors,
        ));

        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| "Search...".to_string());

        let input_display = if self.filter_text.is_empty() {
            SharedString::from(placeholder)
        } else {
            SharedString::from(self.filter_text.clone())
        };

        // Search input
        let input_container = div()
            .id(gpui::ElementId::Name("input:select-filter".into()))
            .w_full()
            .min_h(px(PROMPT_INPUT_FIELD_HEIGHT))
            .px(px(spacing.item_padding_x))
            .py(px(spacing.padding_md))
            .bg(search_box_bg)
            .border_b_1()
            .border_color(border_color)
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            .child(div().text_color(muted_color).child("🔍"))
            .child(
                div()
                    .flex_1()
                    .text_color(if self.filter_text.is_empty() {
                        muted_color
                    } else {
                        text_color
                    })
                    .child(input_display),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(muted_color)
                    .child(format!("{} selected", self.selected.len())),
            );

        // Choices list
        let filtered_len = self.filtered_choices.len();
        let choices_content: AnyElement = if filtered_len == 0 {
            div()
                .w_full()
                .py(px(spacing.padding_xl))
                .px(px(spacing.item_padding_x))
                .text_color(muted_color)
                .child("No choices match your filter")
                .into_any_element()
        } else {
            uniform_list(
                "select-choices",
                filtered_len,
                cx.processor(
                    move |this: &mut SelectPrompt,
                          visible_range: std::ops::Range<usize>,
                          _window,
                          _cx| {
                        let row_colors = UnifiedListItemColors::from_theme(&this.theme);
                        let mut rows = Vec::with_capacity(visible_range.len());

                        for display_idx in visible_range {
                            if let Some(&choice_idx) = this.filtered_choices.get(display_idx) {
                                if let Some(choice) = this.choices.get(choice_idx) {
                                    if let Some(indexed_choice) = this.choice_index.get(choice_idx)
                                    {
                                        let is_focused = display_idx == this.focused_index;
                                        let is_selected = this.selected.contains(&choice_idx);
                                        let semantic_id =
                                            choice.semantic_id.clone().unwrap_or_else(|| {
                                                indexed_choice.stable_semantic_id.clone()
                                            });
                                        let indicator =
                                            choice_selection_indicator(this.multiple, is_selected);
                                        let subtitle = indexed_choice
                                            .metadata
                                            .subtitle_text()
                                            .map(TextContent::plain);
                                        let title = highlighted_choice_title(
                                            &choice.name,
                                            &this.filter_text,
                                        );
                                        let trailing =
                                            indexed_choice.metadata.shortcut.clone().map(
                                                |shortcut| {
                                                    TrailingContent::Shortcut(shortcut.into())
                                                },
                                            );

                                        rows.push(
                                            div()
                                                .id(display_idx)
                                                .w_full()
                                                .h(px(LIST_ITEM_HEIGHT))
                                                .border_b_1()
                                                .border_color(border_color)
                                                .child(
                                                    UnifiedListItem::new(
                                                        gpui::ElementId::Name(semantic_id.into()),
                                                        title,
                                                    )
                                                    .subtitle_opt(subtitle)
                                                    .leading(LeadingContent::Emoji(
                                                        indicator.into(),
                                                    ))
                                                    .trailing_opt(trailing)
                                                    .state(ItemState {
                                                        is_selected: is_focused,
                                                        is_hovered: false,
                                                        is_disabled: false,
                                                    })
                                                    .density(Density::Comfortable)
                                                    .colors(row_colors)
                                                    .with_accent_bar(is_selected),
                                                ),
                                        );
                                    }
                                }
                            }
                        }

                        rows
                    },
                ),
            )
            .h_full()
            .w_full()
            .track_scroll(&self.list_scroll_handle)
            .into_any_element()
        };

        let choices_container = div()
            .id(gpui::ElementId::Name("list:select-choices".into()))
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .child(choices_content);

        div()
            .id(gpui::ElementId::Name("window:select".into()))
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .when_some(vibrancy_bg, |d, bg| d.bg(bg)) // Only apply bg when vibrancy disabled
            .text_color(text_color)
            .key_context("select_prompt")
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .child(input_container)
            .child(choices_container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(name: &str, value: &str, description: Option<&str>) -> Choice {
        let mut choice = Choice::new(name.to_string(), value.to_string());
        choice.description = description.map(str::to_string);
        choice
    }

    #[test]
    fn metadata_parses_shortcut_type_and_last_run() {
        let choice = choice(
            "Deploy API",
            "/Users/me/.scriptkit/scripts/deploy.ts",
            Some("Shortcut: cmd+shift+d • script • Last run 2h ago"),
        );

        let metadata = ChoiceDisplayMetadata::from_choice(&choice);

        assert_eq!(metadata.shortcut.as_deref(), Some("⌘⇧D"));
        assert_eq!(metadata.item_type.as_deref(), Some("Script"));
        assert_eq!(metadata.last_run.as_deref(), Some("Last run 2h ago"));
        assert!(metadata.description.is_none());
    }

    #[test]
    fn score_choice_matches_description_and_value() {
        let choice = choice(
            "Deploy",
            "/Users/me/.scriptkit/scripts/deploy-api.ts",
            Some("Publish service to production"),
        );

        let mut description_ctx = scripts::NucleoCtx::new("production");
        let mut value_ctx = scripts::NucleoCtx::new("deploy-api.ts");
        let indexed_choice = SelectChoiceIndex::from_choice(&choice, 0);

        assert!(score_choice_for_filter(
            &choice,
            &indexed_choice,
            "production",
            &mut description_ctx
        )
        .is_some());
        assert!(
            score_choice_for_filter(&choice, &indexed_choice, "deploy-api.ts", &mut value_ctx)
                .is_some()
        );
    }

    #[test]
    fn score_choice_prefers_name_over_description_only_matches() {
        let name_match = choice(
            "Open Logs",
            "/tmp/open-logs.ts",
            Some("Tail runtime output"),
        );
        let description_match = choice("Tail Runtime", "/tmp/tail-runtime.ts", Some("Open logs"));
        let query = "open logs";

        let mut name_ctx = scripts::NucleoCtx::new(query);
        let mut description_ctx = scripts::NucleoCtx::new(query);
        let indexed_name_match = SelectChoiceIndex::from_choice(&name_match, 0);
        let indexed_description_match = SelectChoiceIndex::from_choice(&description_match, 1);

        let name_score =
            score_choice_for_filter(&name_match, &indexed_name_match, query, &mut name_ctx)
                .unwrap();
        let description_score = score_choice_for_filter(
            &description_match,
            &indexed_description_match,
            query,
            &mut description_ctx,
        )
        .unwrap();

        assert!(
            name_score > description_score,
            "expected name match score ({name_score}) to beat description-only score ({description_score})"
        );
    }

    #[test]
    fn char_indices_to_byte_ranges_handles_utf8_boundaries() {
        let text = "a😀b";
        // Indices for 😀 and b
        let ranges = char_indices_to_byte_ranges(text, &[1, 2]);
        assert_eq!(ranges, vec![1..6]);
    }

    #[test]
    fn test_select_prompt_accepts_space_in_filter_query() {
        assert!(should_append_to_filter(' '));
    }

    #[test]
    fn test_select_prompt_submit_uses_focused_item_in_single_mode_when_none_toggled() {
        let selected_indices = Vec::new();
        let resolved = resolve_submission_indices(false, &selected_indices, Some(4));
        assert_eq!(resolved, vec![4]);
    }

    #[test]
    fn test_select_prompt_cmd_a_toggles_only_when_all_filtered_items_are_selected() {
        let mut selected_indices = std::collections::HashSet::from([1, 7]);
        let filtered_indices = vec![1, 2, 3];

        assert!(!are_all_filtered_selected(
            &selected_indices,
            &filtered_indices
        ));
        toggle_filtered_selection(&mut selected_indices, &filtered_indices);
        assert_eq!(
            selected_indices,
            std::collections::HashSet::from([1, 2, 3, 7])
        );

        assert!(are_all_filtered_selected(
            &selected_indices,
            &filtered_indices
        ));
        toggle_filtered_selection(&mut selected_indices, &filtered_indices);
        assert_eq!(selected_indices, std::collections::HashSet::from([7]));
    }

    #[test]
    fn test_select_prompt_select_all_preserves_existing_off_filter_selection() {
        let mut selected_indices = std::collections::HashSet::from([9]);
        let filtered_indices = vec![1, 2, 3];

        toggle_filtered_selection(&mut selected_indices, &filtered_indices);

        assert_eq!(
            selected_indices,
            std::collections::HashSet::from([1, 2, 3, 9])
        );
    }

    #[test]
    fn test_select_prompt_generates_stable_semantic_id_when_filter_order_changes() {
        let stable_id = fallback_select_semantic_id(17, "scripts/demo.ts");

        assert_eq!(
            stable_id,
            fallback_select_semantic_id(17, "scripts/demo.ts")
        );
        assert_ne!(stable_id, fallback_select_semantic_id(3, "scripts/demo.ts"));
    }

    #[test]
    fn test_select_prompt_resolves_search_box_bg_by_design_variant() {
        let mut theme = theme::Theme::default();
        theme.colors.background.search_box = 0x112233;

        let mut design_colors = DesignColors::default();
        design_colors.background_secondary = 0x445566;

        assert_eq!(
            resolve_search_box_bg_hex(&theme, DesignVariant::Default, &design_colors),
            0x112233
        );
        assert_eq!(
            resolve_search_box_bg_hex(&theme, DesignVariant::Minimal, &design_colors),
            0x445566
        );
    }
}
