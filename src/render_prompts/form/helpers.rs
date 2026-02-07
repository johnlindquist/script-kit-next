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

#[inline]
fn form_footer_status_text(focused_field_is_textarea: bool) -> String {
    if focused_field_is_textarea {
        running_status_text("press ⌘↵ to submit (Enter adds a new line)")
    } else {
        running_status_text("press Enter to submit")
    }
}

#[inline]
fn form_field_value_for_validation(
    field_entity: &crate::form_prompt::FormFieldEntity,
    cx: &App,
) -> String {
    match field_entity {
        crate::form_prompt::FormFieldEntity::TextField(entity) => {
            entity.read(cx).value().to_string()
        }
        crate::form_prompt::FormFieldEntity::TextArea(entity) => {
            entity.read(cx).value().to_string()
        }
        crate::form_prompt::FormFieldEntity::Checkbox(entity) => {
            if entity.read(cx).is_checked() {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
    }
}

#[inline]
fn is_valid_email_submit_value(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }

    if value
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return false;
    }

    let mut parts = value.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();

    if local.is_empty() || domain.is_empty() || parts.next().is_some() {
        return false;
    }

    if domain.starts_with('.') || domain.ends_with('.') {
        return false;
    }

    domain.contains('.')
}

#[inline]
fn is_valid_number_submit_value(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return true;
    }
    trimmed.parse::<f64>().is_ok()
}

#[inline]
fn form_field_value_is_valid_for_submit(field_type: Option<&str>, value: &str) -> bool {
    match field_type {
        Some(field_type) if field_type.eq_ignore_ascii_case("email") => {
            is_valid_email_submit_value(value)
        }
        Some(field_type) if field_type.eq_ignore_ascii_case("number") => {
            is_valid_number_submit_value(value)
        }
        _ => true,
    }
}

#[inline]
fn collect_form_submit_validation_errors(form: &FormPromptState, cx: &App) -> Vec<String> {
    let mut invalid_fields = Vec::new();

    for (field_definition, field_entity) in &form.fields {
        let value = form_field_value_for_validation(field_entity, cx);
        if form_field_value_is_valid_for_submit(field_definition.field_type.as_deref(), &value) {
            continue;
        }

        let field_type = field_definition
            .field_type
            .as_deref()
            .unwrap_or("text")
            .to_string();
        invalid_fields.push(format!("{} ({})", field_definition.name, field_type));
    }

    invalid_fields
}

#[inline]
fn form_submit_validation_message(invalid_fields: &[String]) -> String {
    if invalid_fields.len() == 1 {
        format!("Fix invalid field before submitting: {}", invalid_fields[0])
    } else {
        format!(
            "Fix invalid fields before submitting: {}",
            invalid_fields.join(", ")
        )
    }
}
