use gpui::{
    div, prelude::*, px, App, ClickEvent, Context, ElementId, Entity, FocusHandle, Focusable,
    KeyDownEvent, Render, Window,
};

use crate::components::{FormCheckbox, FormFieldColors, FormTextArea, FormTextField};
use crate::{form_parser, logging, protocol};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormPromptOutputMode {
    ObjectByName,
    ArrayByOrder,
    Preset,
}

/// Enum to hold different types of form field entities.
#[derive(Clone)]
pub enum FormFieldEntity {
    TextField(Entity<FormTextField>),
    TextArea(Entity<FormTextArea>),
    Checkbox(Entity<FormCheckbox>),
}

/// Form prompt state - holds the parsed form fields and their entities.
pub struct FormPromptState {
    /// Prompt ID for response.
    pub id: String,
    /// Original HTML for reference.
    #[allow(dead_code)]
    pub html: String,
    /// Parsed field definitions and their corresponding entities.
    pub fields: Vec<(protocol::Field, FormFieldEntity)>,
    /// Colors for form fields.
    pub colors: FormFieldColors,
    /// Currently focused field index (for Tab navigation).
    pub focused_index: usize,
    focus_revision: u64,
    scroll: gpui::ScrollHandle,
    /// Focus handle for this form.
    pub focus_handle: FocusHandle,
    /// Output contract for SDK resolution.
    pub output_mode: FormPromptOutputMode,
}

impl FormPromptState {
    /// Aggregate source epochs without hashing field contents or mutating on reads.
    /// Include every field so changing focus cannot exchange one field's epoch for another.
    pub fn dictation_input_revision(&self, cx: &App) -> u64 {
        self.fields
            .iter()
            .fold(self.focus_revision, |revision, (_, field)| {
                let field_revision = match field {
                    FormFieldEntity::TextField(entity) => entity.read(cx).revision(),
                    FormFieldEntity::TextArea(entity) => entity.read(cx).revision(),
                    FormFieldEntity::Checkbox(entity) => entity.read(cx).revision(),
                };
                assert!(
                    revision <= u64::MAX - field_revision,
                    "form input revision exhausted"
                );
                revision + field_revision
            })
    }

    pub fn dictation_input_len(&self, cx: &App) -> Option<usize> {
        match &self.fields.get(self.focused_index)?.1 {
            FormFieldEntity::TextField(entity) => Some(entity.read(cx).value().len()),
            FormFieldEntity::TextArea(entity) => Some(entity.read(cx).value().len()),
            FormFieldEntity::Checkbox(_) => None,
        }
    }

    fn set_focused_index(&mut self, index: usize) {
        self.scroll.scroll_to_item(index);
        if self.focused_index != index {
            assert!(
                self.focus_revision < u64::MAX,
                "form focus revision exhausted"
            );
            self.focus_revision += 1;
            self.focused_index = index;
        }
    }

    pub fn update_theme(&mut self, theme: &crate::theme::Theme, cx: &mut Context<Self>) {
        let colors = FormFieldColors::from_theme(theme);
        self.colors = colors;
        for (_, field) in &self.fields {
            match field {
                FormFieldEntity::TextField(entity) => {
                    entity.update(cx, |field, cx| field.update_colors(colors, cx))
                }
                FormFieldEntity::TextArea(entity) => {
                    entity.update(cx, |field, cx| field.update_colors(colors, cx))
                }
                FormFieldEntity::Checkbox(entity) => {
                    entity.update(cx, |field, cx| field.update_colors(colors, cx))
                }
            }
        }
        cx.notify();
    }

    pub fn field_values(&self, cx: &App) -> Vec<(String, String)> {
        self.fields
            .iter()
            .map(|(field, entity)| (field.name.clone(), Self::field_value(entity, cx)))
            .collect()
    }

    pub fn focused_value(&self, cx: &App) -> Option<String> {
        self.fields
            .get(self.focused_index)
            .map(|(_, entity)| Self::field_value(entity, cx))
    }

    pub fn validated_submit_value(
        &mut self,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        let mut first = None;
        for (index, (_, field)) in self.fields.iter().enumerate() {
            if let FormFieldEntity::TextField(entity) = field {
                if entity.update(cx, |field, cx| field.reveal_submit_issue(cx)) {
                    first.get_or_insert(index);
                }
            }
        }
        if let Some(index) = first {
            if let Some(window) = window {
                self.focus_field_at(index, window, cx);
            } else {
                // The semantic submit path has no window. Render applies this focus.
                self.set_focused_index(index);
                cx.notify();
            }
            return None;
        }
        Some(self.collect_values(cx))
    }

    fn build_values_json(values: impl IntoIterator<Item = (String, String)>) -> String {
        let mut map = serde_json::Map::new();
        for (key, value) in values {
            map.insert(key, serde_json::Value::String(value));
        }
        serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
    }

    fn build_field_entities(
        parsed_fields: Vec<protocol::Field>,
        colors: FormFieldColors,
        cx: &mut App,
    ) -> Vec<(protocol::Field, FormFieldEntity)> {
        let fields: Vec<(protocol::Field, FormFieldEntity)> = parsed_fields
            .into_iter()
            .enumerate()
            .map(|(index, mut field)| {
                if field
                    .label
                    .as_deref()
                    .is_none_or(|label| label.trim().is_empty())
                {
                    let words = field
                        .name
                        .split(['_', '-'])
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let mut chars = words.chars();
                    field.label = Some(match chars.next() {
                        Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
                        None => format!("Field {}", index + 1),
                    });
                }
                let field_type = field
                    .field_type
                    .clone()
                    .unwrap_or_else(|| "text".to_string());
                logging::log(
                    "FORM",
                    &format!("Creating field: {} (type: {})", field.name, field_type),
                );

                let normalized_type = field_type.to_ascii_lowercase();
                let entity = match normalized_type.as_str() {
                    "checkbox" => {
                        let checkbox = FormCheckbox::new(field.clone(), colors, cx);
                        FormFieldEntity::Checkbox(cx.new(|_| checkbox))
                    }
                    "textarea" => {
                        let textarea = FormTextArea::new(field.clone(), colors, 4, cx);
                        FormFieldEntity::TextArea(cx.new(|_| textarea))
                    }
                    _ => {
                        // text, password, email, number all use TextField
                        let textfield = FormTextField::new(field.clone(), colors, cx);
                        FormFieldEntity::TextField(cx.new(|_| textfield))
                    }
                };

                (field, entity)
            })
            .collect();

        fields
    }

    fn from_fields_with_mode(
        id: String,
        html: String,
        parsed_fields: Vec<protocol::Field>,
        colors: FormFieldColors,
        output_mode: FormPromptOutputMode,
        cx: &mut App,
    ) -> Self {
        let fields = Self::build_field_entities(parsed_fields, colors, cx);

        Self {
            id,
            html,
            fields,
            colors,
            focused_index: 0,
            focus_revision: 0,
            scroll: gpui::ScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            output_mode,
        }
    }

    /// Create a new form prompt state from HTML.
    pub fn new(id: String, html: String, colors: FormFieldColors, cx: &mut App) -> Self {
        let parsed_fields = form_parser::parse_form_html(&html);

        logging::log(
            "FORM",
            &format!("Parsed {} form fields from HTML", parsed_fields.len()),
        );

        Self::from_fields_with_mode(
            id,
            html,
            parsed_fields,
            colors,
            FormPromptOutputMode::ObjectByName,
            cx,
        )
    }

    /// Create a form prompt state from SDK fields() definitions.
    pub fn from_fields(
        id: String,
        fields: Vec<protocol::Field>,
        colors: FormFieldColors,
        cx: &mut App,
    ) -> Self {
        logging::log(
            "FORM",
            &format!("Creating fields() prompt with {} fields", fields.len()),
        );

        Self::from_fields_with_mode(
            id,
            String::new(),
            fields,
            colors,
            FormPromptOutputMode::ArrayByOrder,
            cx,
        )
    }

    pub fn prompt_type(&self) -> &'static str {
        match self.output_mode {
            FormPromptOutputMode::ObjectByName => "form",
            FormPromptOutputMode::ArrayByOrder => "fields",
            FormPromptOutputMode::Preset => "preset",
        }
    }

    pub fn semantic_prefix(&self) -> &'static str {
        self.prompt_type()
    }

    pub(crate) fn named_values(&self, cx: &App) -> Vec<(String, String)> {
        self.fields
            .iter()
            .map(|(f, e)| (f.name.clone(), Self::field_value(e, cx)))
            .collect()
    }

    fn field_value(entity: &FormFieldEntity, cx: &App) -> String {
        match entity {
            FormFieldEntity::TextField(e) => e.read(cx).value().to_string(),
            FormFieldEntity::TextArea(e) => e.read(cx).value().to_string(),
            FormFieldEntity::Checkbox(e) => {
                if e.read(cx).is_checked() {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
        }
    }

    /// Get all field values using the SDK contract for this prompt source.
    pub fn collect_values(&self, cx: &App) -> String {
        match self.output_mode {
            FormPromptOutputMode::ObjectByName | FormPromptOutputMode::Preset => {
                Self::build_values_json(self.named_values(cx))
            }
            FormPromptOutputMode::ArrayByOrder => {
                let values: Vec<String> = self
                    .fields
                    .iter()
                    .map(|(_, entity)| Self::field_value(entity, cx))
                    .collect();
                serde_json::to_string(&values).unwrap_or_else(|_| "[]".to_string())
            }
        }
    }

    #[allow(
        dead_code,
        reason = "used by the binary prompt evaluator and field correction tests"
    )]
    pub(crate) fn visible_validation_message(&self, cx: &App) -> Option<&'static str> {
        self.fields.iter().find_map(|(_, entity)| match entity {
            FormFieldEntity::TextField(field) => field.read(cx).visible_submit_hint(),
            _ => None,
        })
    }

    /// Focus the next field (for Tab navigation).
    pub fn focus_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.fields.is_empty() {
            return;
        }
        let next = (self.focused_index + 1) % self.fields.len();
        self.focus_field_at(next, window, cx);
    }

    /// Focus the previous field (for Shift+Tab navigation).
    pub fn focus_previous(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.fields.is_empty() {
            return;
        }
        let previous = if self.focused_index == 0 {
            self.fields.len() - 1
        } else {
            self.focused_index - 1
        };
        self.focus_field_at(previous, window, cx);
    }

    fn focus_handle_at(&self, index: usize, cx: &App) -> Option<FocusHandle> {
        self.fields.get(index).map(|(_, entity)| match entity {
            FormFieldEntity::TextField(e) => e.read(cx).focus_handle(cx),
            FormFieldEntity::TextArea(e) => e.read(cx).focus_handle(cx),
            FormFieldEntity::Checkbox(e) => e.read(cx).focus_handle(cx),
        })
    }

    pub(crate) fn focus_field_at(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.fields.is_empty() {
            return;
        }
        self.visit_field(index.min(self.fields.len() - 1), cx);
        if let Some(focus_handle) = self.focus_handle_at(self.focused_index, cx) {
            focus_handle.focus(window, cx);
        }
        cx.notify();
    }

    fn visit_field(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.focused_index != index {
            if let Some((_, field)) = self.fields.get(self.focused_index) {
                match field {
                    FormFieldEntity::TextField(field) => {
                        field.update(cx, |f, cx| f.finish_visit(cx))
                    }
                    FormFieldEntity::TextArea(field) => field.update(cx, |f, _| f.finish_visit()),
                    FormFieldEntity::Checkbox(_) => (),
                }
            }
        }
        self.set_focused_index(index);
    }

    pub fn focus_field_by_semantic_id(
        &mut self,
        semantic_id: &str,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        let semantic_prefix = self.semantic_prefix();
        let index = self
            .fields
            .iter()
            .enumerate()
            .find_map(|(index, (field, _))| {
                let semantic_name = format!("{semantic_prefix}-{}", field.name);
                let field_semantic_id =
                    protocol::generate_semantic_id_named("input", &semantic_name);
                (field_semantic_id == semantic_id).then_some(index)
            })?;

        self.visit_field(index, cx);
        self.fields
            .get(index)
            .map(|(field, _)| field.label.clone().unwrap_or_else(|| field.name.clone()))
    }

    /// Get the focus handle for the currently focused field.
    pub fn current_focus_handle(&self, cx: &App) -> Option<FocusHandle> {
        self.focus_handle_at(self.focused_index, cx)
    }

    /// Handle keyboard input by forwarding to the currently focused field.
    ///
    /// This forwards key events to the field's unified `handle_key_event` method
    /// which properly handles:
    /// - Char-based cursor positioning (not byte-based)
    /// - Modifier keys (Cmd/Ctrl+C/V/X/A work correctly)
    /// - Selection with Shift+Arrow
    /// - Clipboard operations
    pub fn handle_key_input(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if let Some((_, entity)) = self.fields.get(self.focused_index) {
            match entity {
                FormFieldEntity::TextField(e) => {
                    e.update(cx, |field, cx| {
                        field.handle_key_event(event, cx);
                    });
                }
                FormFieldEntity::TextArea(e) => {
                    e.update(cx, |field, cx| {
                        field.handle_key_event(event, cx);
                    });
                }
                FormFieldEntity::Checkbox(e) => {
                    // Space toggles checkbox
                    let key = event.keystroke.key.as_str();
                    if key == "space" || key == " " {
                        e.update(cx, |checkbox, cx| {
                            checkbox.toggle(cx);
                        });
                    }
                }
            }
        }
    }

    /// Set the current field's input text programmatically.
    pub fn set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if let Some((_, entity)) = self.fields.get(self.focused_index) {
            match entity {
                FormFieldEntity::TextField(e) => {
                    let value = text.clone();
                    e.update(cx, |field, cx| {
                        field.set_value(value);
                        cx.notify();
                    });
                }
                FormFieldEntity::TextArea(e) => {
                    let value = text.clone();
                    e.update(cx, |field, cx| {
                        field.set_value(value);
                        cx.notify();
                    });
                }
                FormFieldEntity::Checkbox(_) => {}
            }
        }
    }
}

impl Render for FormPromptState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tracing::info!(
            surface = %format!("{}_prompt", self.semantic_prefix()),
            field_count = self.fields.len(),
            focused_index = self.focused_index,
            "prompt_surface_rendered"
        );

        // Ensure the currently selected field is the keyboard focus target.
        if let Some(focus_handle) = self.current_focus_handle(cx) {
            if !focus_handle.is_focused(window) {
                focus_handle.focus(window, cx);
            }
        }

        // Build the form fields container
        use gpui_component::scroll::ScrollableElement;
        let metrics = crate::components::FormFieldMetrics::from_colors(self.colors);
        let mut container = div()
            .id("form-fields-scroll")
            .flex()
            .flex_col()
            .gap(px(metrics.field_stack_gap_px))
            .size_full()
            .min_h(px(0.))
            .overflow_y_scroll()
            .track_scroll(&self.scroll);
        for (index, (_field_def, entity)) in self.fields.iter().enumerate() {
            let focus_slot_click = cx.listener(
                move |this: &mut Self,
                      _event: &ClickEvent,
                      window: &mut Window,
                      cx: &mut Context<Self>| {
                    this.focus_field_at(index, window, cx);
                },
            );
            let slot = div()
                .id(ElementId::Name(format!("form-field-slot-{index}").into()))
                .w_full()
                .flex_none()
                .on_click(focus_slot_click);
            container = match entity {
                FormFieldEntity::TextField(e) => container.child(slot.child(e.clone())),
                FormFieldEntity::TextArea(e) => container.child(slot.child(e.clone())),
                FormFieldEntity::Checkbox(e) => container.child(slot.child(e.clone())),
            };
        }

        // If no fields, show an error message
        if self.fields.is_empty() {
            container = container.child(
                div()
                    .p(px(16.))
                    .text_color(self.colors.label)
                    .child("No form fields found in HTML"),
            );
        }

        // Root binds Tab before raw key handlers run. Keep traversal inside this form
        // and update the same field index used by editing, validation and automation.
        container
            .vertical_scrollbar(&self.scroll)
            .on_action(cx.listener(|this, _: &gpui_component::Tab, window, cx| {
                this.focus_next(window, cx);
                cx.stop_propagation();
            }))
            .on_action(
                cx.listener(|this, _: &gpui_component::TabPrev, window, cx| {
                    this.focus_previous(window, cx);
                    cx.stop_propagation();
                }),
            )
    }
}

/// Delegated Focusable implementation for FormPromptState.
///
/// This implements the "delegated focus" pattern from Zed's BufferSearchBar:
/// Instead of returning our own focus_handle, we return the focused field's handle.
/// This prevents the parent container from "stealing" focus from child fields during re-renders.
///
/// When GPUI asks "what should be focused?", we answer with the currently focused
/// text field's handle, so focus stays on the actual input field, not the form container.
impl Focusable for FormPromptState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        // Return the focused field's handle, not our own
        // This delegates focus management to the child field, preventing focus stealing
        if let Some((_, entity)) = self.fields.get(self.focused_index) {
            match entity {
                FormFieldEntity::TextField(e) => e.read(cx).get_focus_handle(),
                FormFieldEntity::TextArea(e) => e.read(cx).get_focus_handle(),
                FormFieldEntity::Checkbox(e) => e.read(cx).focus_handle(cx),
            }
        } else {
            // Fallback to our own handle if no fields exist
            self.focus_handle.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn build_values_json_serializes_string_values() {
        let values = vec![
            ("username".to_string(), "Bob".to_string()),
            ("bio".to_string(), "Hello".to_string()),
            ("subscribe".to_string(), "true".to_string()),
        ];
        let parsed: serde_json::Value =
            serde_json::from_str(&FormPromptState::build_values_json(values))
                .expect("values should be json");
        assert_eq!(
            parsed,
            json!({
                "bio": "Hello",
                "subscribe": "true",
                "username": "Bob"
            })
        );
    }
}

#[cfg(test)]
#[gpui::test]
fn long_form_reveals_the_keyboard_field_and_returns_to_the_invalid_one(
    cx: &mut gpui::TestAppContext,
) {
    let window = cx.add_window(|_, cx| {
        let html = format!("<input name='email' type='email' value='invalid'>{}<input name='enabled' type='checkbox'>",
            (0..18).map(|i| format!("<input name='item_{i}'>")).collect::<String>());
        FormPromptState::new("long".into(), html, FormFieldColors::default(), cx)
    });
    cx.simulate_window_resize(window.into(), gpui::size(px(480.), px(280.)));
    cx.run_until_parked();
    window
        .update(cx, |form, win, cx| form.focus_previous(win, cx))
        .unwrap();
    cx.run_until_parked();
    window
        .update(cx, |form, win, cx| {
            assert_eq!(form.focused_index, form.fields.len() - 1);
            assert!(form.current_focus_handle(cx).unwrap().is_focused(win));
            assert!(
                form.scroll.offset().y < px(0.),
                "Tab must reveal the checkbox offscreen"
            );
            let rect = form.scroll.bounds_for_item(form.focused_index).unwrap();
            assert!(
                rect.bottom() + form.scroll.offset().y <= form.scroll.bounds().bottom() + px(1.)
            );
            assert!(form.validated_submit_value(Some(win), cx).is_none());
        })
        .unwrap();
    cx.run_until_parked();
    window
        .update(cx, |form, win, cx| {
            assert_eq!(form.focused_index, 0);
            assert!(form.current_focus_handle(cx).unwrap().is_focused(win));
            assert!(
                form.scroll.bounds_for_item(0).unwrap().top() + form.scroll.offset().y
                    >= form.scroll.bounds().top() - px(1.)
            );
        })
        .unwrap();
}

#[cfg(test)]
#[gpui::test]
fn rejected_form_focuses_first_invalid_field_and_only_shows_current_issues(
    cx: &mut gpui::TestAppContext,
) {
    use gpui::AppContext as _;
    let colors = FormFieldColors::from_theme(&crate::theme::Theme::default());
    let form = cx.new(|cx| FormPromptState::new("reject".into(),
        "<input name='email' type='email' value='invalid'><input name='count' type='number' value='no'><textarea name='notes'>draft</textarea>".into(), colors, cx));
    form.update(cx, |form, cx| {
        form.set_focused_index(2);
        assert!(form.visible_validation_message(cx).is_none());
        assert!(form.validated_submit_value(None, cx).is_none());
        assert_eq!(form.focused_index, 0);
        form.set_input("reader@example.org".into(), cx);
        assert_eq!(form.visible_validation_message(cx), Some("Enter a number."));
        assert!(form.validated_submit_value(None, cx).is_none());
        assert_eq!(form.focused_index, 1);
        form.set_input("12".into(), cx);
        assert!(form.visible_validation_message(cx).is_none());
        let result = form.validated_submit_value(None, cx).expect("valid draft");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&result).unwrap(),
            serde_json::json!({"email":"reader@example.org","count":"12","notes":"draft"})
        );
    });
}

#[cfg(test)]
#[gpui::test]
fn form_dictation_epoch_tracks_field_and_focus_aba(cx: &mut gpui::TestAppContext) {
    use gpui::AppContext as _;
    let colors = FormFieldColors::from_theme(&crate::theme::Theme::default());
    let form = cx.new(|cx| FormPromptState::new("epoch".into(),
        "<input name='first' value='cat'><textarea name='second'>hé</textarea><input name='check' type='checkbox'>".into(), colors, cx));
    form.update(cx, |form, cx| {
        let initial = form.dictation_input_revision(cx);
        assert_eq!(form.dictation_input_len(cx), Some(3));
        form.set_input("cat".into(), cx);
        form.update_theme(&crate::theme::Theme::default(), cx);
        assert_eq!(form.dictation_input_revision(cx), initial);
        form.set_input("dog".into(), cx);
        let changed = form.dictation_input_revision(cx);
        assert!(changed > initial);
        form.set_input("cat".into(), cx);
        assert!(form.dictation_input_revision(cx) > changed);
        let before_focus = form.dictation_input_revision(cx);
        form.set_focused_index(1);
        form.set_input("hé".into(), cx);
        assert_eq!(form.dictation_input_len(cx), Some("hé".len()));
        form.set_focused_index(0);
        assert_eq!(form.focused_value(cx).as_deref(), Some("cat"));
        assert!(form.dictation_input_revision(cx) > before_focus);
        let focused = form.dictation_input_revision(cx);
        form.set_focused_index(0);
        form.field_values(cx);
        assert_eq!(form.dictation_input_revision(cx), focused);
        form.set_focused_index(2);
        assert_eq!(form.dictation_input_len(cx), None);
        let checkbox_epoch = form.dictation_input_revision(cx);
        let FormFieldEntity::Checkbox(checkbox) = form.fields[2].1.clone() else {
            panic!("checkbox fixture")
        };
        checkbox.update(cx, |checkbox, cx| {
            checkbox.set_checked(false, cx);
            assert_eq!(checkbox.revision(), 0);
            checkbox.toggle(cx);
            checkbox.toggle(cx);
            assert!(!checkbox.is_checked());
        });
        assert!(form.dictation_input_revision(cx) > checkbox_epoch);
    });
}
