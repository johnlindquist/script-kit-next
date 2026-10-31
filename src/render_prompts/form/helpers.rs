use gpui_component::scroll::ScrollableElement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormEnterBehavior {
    Submit,
    ForwardToField,
    Ignore,
}

#[inline]
fn form_enter_behavior(
    key: &str,
    has_cmd: bool,
    focused_field_is_textarea: bool,
) -> FormEnterBehavior {
    if !ui_foundation::is_key_enter(key) {
        return FormEnterBehavior::Ignore;
    }

    if focused_field_is_textarea && !has_cmd {
        return FormEnterBehavior::ForwardToField;
    }

    FormEnterBehavior::Submit
}

#[inline]
fn focused_form_field_is_textarea(form: &FormPromptState) -> bool {
    form.fields
        .get(form.focused_index)
        .and_then(|(field, _)| field.field_type.as_deref())
        .is_some_and(|field_type| field_type.eq_ignore_ascii_case("textarea"))
}

#[allow(dead_code)]
#[inline]
fn form_footer_status_text(focused_field_is_textarea: bool) -> String {
    if focused_field_is_textarea {
        running_status_text("press ⌘↵ to submit (Enter adds a new line)")
    } else {
        running_status_text("press Enter to submit")
    }
}
