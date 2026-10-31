use gpui::*;

use super::super::{render_form_field_shell, FormFieldMetrics, FormFieldShellSpec};
use super::FormTextArea;

impl Focusable for FormTextArea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FormTextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors;
        let metrics = FormFieldMetrics::from_colors(colors);
        let is_focused = self.focus_handle.is_focused(window);
        let display_text = self.value.clone();
        let placeholder = self.field.placeholder.clone().unwrap_or_default();
        let label = self.field.label.clone().map(SharedString::from);
        let rows = self.rows;
        let cursor_pos = self.cursor_position;
        let field_name = self.field.name.clone();
        let field_height = metrics.text_area_height_px(rows);
        let shell_spec = FormFieldShellSpec::neutral(
            format!("form-textarea-{field_name}"),
            label,
            is_focused,
            true,
            field_height,
            Some(field_height),
        );

        let selection = self.selection_range();
        let (text_content, hit) = self.viewport.text(
            display_text,
            &placeholder,
            cursor_pos,
            selection,
            is_focused,
            true,
            self.input_revision,
            colors,
        );
        let focus_handle_for_click = self.focus_handle.clone();
        let handle_click = cx.listener(
            move |this: &mut Self,
                  event: &ClickEvent,
                  window: &mut Window,
                  cx: &mut Context<Self>| {
                if let Some(p) = event.mouse_position() {
                    let old = this.selection_state();
                    this.cursor_position = hit.at(p);
                    this.clear_selection();
                    this.record_selection_change(old);
                    cx.notify();
                }
                focus_handle_for_click.focus(window, cx);
            },
        );
        let handle_key = cx.listener(
            |this: &mut Self,
             event: &KeyDownEvent,
             _window: &mut Window,
             cx: &mut Context<Self>| {
                this.handle_key_event(event, cx);
            },
        );

        let body = div()
            .id(ElementId::Name(format!("textarea-{field_name}").into()))
            .track_focus(&self.focus_handle)
            .on_key_down(handle_key)
            .on_click(handle_click)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .cursor_text()
            .overflow_x_hidden()
            .overflow_y_scroll()
            .track_scroll(&self.viewport.scroll)
            .child(text_content)
            .into_any_element();

        render_form_field_shell(&shell_spec, colors, metrics, body)
    }
}
