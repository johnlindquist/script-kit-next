//! HTML Form Parser Module
//!
//! Parses HTML strings and extracts form elements (input, textarea, select, label)
//! converting them to Vec<Field> for native GPUI rendering.

use crate::protocol::Field;
use regex::Regex;
use tracing::warn;

type SmallStringMap = Vec<(String, String)>;

fn pair_get<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a String> {
    pairs
        .iter()
        .find_map(|(name, value)| (name.as_str() == key).then_some(value))
}

fn pair_upsert(pairs: &mut SmallStringMap, key: String, value: String) {
    if let Some((_, existing)) = pairs
        .iter_mut()
        .find(|(name, _)| name.as_str() == key.as_str())
    {
        *existing = value;
    } else {
        pairs.push((key, value));
    }
}

fn pair_insert_if_absent(pairs: &mut SmallStringMap, key: String, value: String) {
    if !pairs.iter().any(|(name, _)| name.as_str() == key.as_str()) {
        pairs.push((key, value));
    }
}

fn compile_regex(pattern: &str, context: &str) -> Option<Regex> {
    match Regex::new(pattern) {
        Ok(regex) => Some(regex),
        Err(error) => {
            warn!(
                category = "FORM_PARSER",
                context,
                pattern,
                ?error,
                "Failed to compile HTML form parser regex"
            );
            None
        }
    }
}

/// Parse an HTML string and extract form fields.
///
/// Supported elements:
/// - `<input>` with types: text, password, email, number, checkbox
/// - `<textarea>` elements
/// - `<select>` elements (extracts options)
/// - `<label>` elements (associates with inputs via `for` attribute or wrapping)
///
/// # Arguments
/// * `html` - HTML string containing form elements
///
/// # Returns
/// Vec<Field> with extracted form field definitions
pub fn parse_form_html(html: &str) -> Vec<Field> {
    let labels = extract_labels(html);

    // Collect all form elements with their positions to maintain document order
    let mut elements: Vec<(usize, Field)> = Vec::new();

    // Parse input elements
    if let Some(input_regex) = compile_regex(r#"<input\s+([^>]*)/?>"#, "input") {
        for cap in input_regex.captures_iter(html) {
            if let Some(attrs_str) = cap.get(1) {
                let attrs = parse_attributes(attrs_str.as_str());
                if let Some(field) = input_to_field(&attrs, &labels) {
                    let pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
                    elements.push((pos, field));
                }
            }
        }
    }

    // Parse textarea elements (handles both empty and with content)
    if let Some(textarea_regex) =
        compile_regex(r#"<textarea\s+([^>]*)>([\s\S]*?)</textarea>"#, "textarea")
    {
        for cap in textarea_regex.captures_iter(html) {
            if let Some(attrs_str) = cap.get(1) {
                let attrs = parse_attributes(attrs_str.as_str());
                let content = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                if let Some(field) = textarea_to_field(&attrs, content, &labels) {
                    let pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
                    elements.push((pos, field));
                }
            }
        }
    }

    // Parse select elements
    if let Some(select_regex) = compile_regex(r#"<select\s+([^>]*)>[\s\S]*?</select>"#, "select") {
        for cap in select_regex.captures_iter(html) {
            if let Some(attrs_str) = cap.get(1) {
                let attrs = parse_attributes(attrs_str.as_str());
                if let Some(field) = select_to_field(&attrs, &labels) {
                    let pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
                    elements.push((pos, field));
                }
            }
        }
    }

    // Sort by position in document and extract fields
    elements.sort_by_key(|(pos, _)| *pos);
    elements.into_iter().map(|(_, field)| field).collect()
}

/// Extract label elements and their text content, indexed by `for` attribute.
fn extract_labels(html: &str) -> SmallStringMap {
    let mut labels = SmallStringMap::new();

    if let Some(label_regex) = compile_regex(
        r#"<label\s+[^>]*for\s*=\s*["']([^"']+)["'][^>]*>([^<]*)</label>"#,
        "label_with_attrs",
    ) {
        for cap in label_regex.captures_iter(html) {
            if let (Some(for_attr), Some(text)) = (cap.get(1), cap.get(2)) {
                pair_upsert(
                    &mut labels,
                    for_attr.as_str().to_string(),
                    text.as_str().trim().to_string(),
                );
            }
        }
    }

    if let Some(simple_label_regex) = compile_regex(
        r#"<label\s+for\s*=\s*["']([^"']+)["']\s*>([^<]*)</label>"#,
        "simple_label",
    ) {
        for cap in simple_label_regex.captures_iter(html) {
            if let (Some(for_attr), Some(text)) = (cap.get(1), cap.get(2)) {
                pair_insert_if_absent(
                    &mut labels,
                    for_attr.as_str().to_string(),
                    text.as_str().trim().to_string(),
                );
            }
        }
    }

    labels
}

/// Parse HTML attributes from an attribute string.
fn parse_attributes(attrs_str: &str) -> SmallStringMap {
    let mut attrs = SmallStringMap::new();

    if let Some(attr_regex) = compile_regex(r#"(\w+)\s*=\s*["']([^"']*)["']"#, "attributes") {
        for cap in attr_regex.captures_iter(attrs_str) {
            if let (Some(name), Some(value)) = (cap.get(1), cap.get(2)) {
                pair_upsert(
                    &mut attrs,
                    name.as_str().to_lowercase(),
                    value.as_str().to_string(),
                );
            }
        }
    }

    if attrs_str.contains("checked") && pair_get(&attrs, "checked").is_none() {
        attrs.push(("checked".to_string(), "true".to_string()));
    }

    attrs
}

/// Convert input element attributes to a Field.
fn input_to_field(attrs: &[(String, String)], labels: &[(String, String)]) -> Option<Field> {
    let name = pair_get(attrs, "name")?.clone();
    let field_type = pair_get(attrs, "type")
        .cloned()
        .unwrap_or_else(|| "text".to_string());

    if field_type == "hidden" || field_type == "submit" || field_type == "button" {
        return None;
    }

    let mut field = Field::new(name.clone());

    field.field_type = Some(field_type.clone());

    let label = pair_get(attrs, "id")
        .and_then(|id| pair_get(labels, id).cloned())
        .or_else(|| pair_get(labels, &name).cloned());
    field.label = label;

    if let Some(placeholder) = pair_get(attrs, "placeholder") {
        field.placeholder = Some(placeholder.clone());
    }

    if field_type == "checkbox" {
        if pair_get(attrs, "checked").is_some() {
            field.value = Some("true".to_string());
        } else {
            field.value = pair_get(attrs, "value").cloned();
        }
    } else if let Some(value) = pair_get(attrs, "value") {
        field.value = Some(value.clone());
    }

    Some(field)
}

/// Convert textarea element to a Field.
fn textarea_to_field(
    attrs: &[(String, String)],
    content: &str,
    labels: &[(String, String)],
) -> Option<Field> {
    let name = pair_get(attrs, "name")?.clone();

    let mut field = Field::new(name.clone());
    field.field_type = Some("textarea".to_string());

    let label = pair_get(attrs, "id")
        .and_then(|id| pair_get(labels, id).cloned())
        .or_else(|| pair_get(labels, &name).cloned());
    field.label = label;

    if let Some(placeholder) = pair_get(attrs, "placeholder") {
        field.placeholder = Some(placeholder.clone());
    }

    // Set value from content if not empty
    if !content.trim().is_empty() {
        field.value = Some(content.trim().to_string());
    }

    Some(field)
}

/// Convert select element to a Field.
fn select_to_field(attrs: &[(String, String)], labels: &[(String, String)]) -> Option<Field> {
    let name = pair_get(attrs, "name")?.clone();

    let mut field = Field::new(name.clone());
    field.field_type = Some("select".to_string());

    let label = pair_get(attrs, "id")
        .and_then(|id| pair_get(labels, id).cloned())
        .or_else(|| pair_get(labels, &name).cloned());
    field.label = label;

    Some(field)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        let fields = parse_form_html("");
        assert!(fields.is_empty());
    }

    #[test]
    fn test_parse_text_input() {
        let html = r#"<input type="text" name="username" placeholder="Enter username" />"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "username");
        assert_eq!(fields[0].field_type, Some("text".to_string()));
        assert_eq!(fields[0].placeholder, Some("Enter username".to_string()));
    }

    #[test]
    fn test_parse_password_input() {
        let html = r#"<input type="password" name="password" />"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "password");
        assert_eq!(fields[0].field_type, Some("password".to_string()));
    }

    #[test]
    fn test_parse_email_input() {
        let html = r#"<input type="email" name="email" placeholder="you@example.com" />"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "email");
        assert_eq!(fields[0].field_type, Some("email".to_string()));
        assert_eq!(fields[0].placeholder, Some("you@example.com".to_string()));
    }

    #[test]
    fn test_parse_number_input() {
        let html = r#"<input type="number" name="age" value="25" />"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "age");
        assert_eq!(fields[0].field_type, Some("number".to_string()));
        assert_eq!(fields[0].value, Some("25".to_string()));
    }

    #[test]
    fn test_parse_checkbox() {
        let html = r#"<input type="checkbox" name="subscribe" value="yes" />"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "subscribe");
        assert_eq!(fields[0].field_type, Some("checkbox".to_string()));
        assert_eq!(fields[0].value, Some("yes".to_string()));
    }

    #[test]
    fn test_parse_checkbox_checked() {
        let html = r#"<input type="checkbox" name="agree" checked />"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "agree");
        assert_eq!(fields[0].field_type, Some("checkbox".to_string()));
        assert_eq!(fields[0].value, Some("true".to_string()));
    }

    #[test]
    fn test_parse_textarea() {
        let html =
            r#"<textarea name="bio" placeholder="Tell us about yourself">Hello world</textarea>"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "bio");
        assert_eq!(fields[0].field_type, Some("textarea".to_string()));
        assert_eq!(
            fields[0].placeholder,
            Some("Tell us about yourself".to_string())
        );
        assert_eq!(fields[0].value, Some("Hello world".to_string()));
    }

    #[test]
    fn test_parse_empty_textarea() {
        let html = r#"<textarea name="notes"></textarea>"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "notes");
        assert_eq!(fields[0].field_type, Some("textarea".to_string()));
        assert_eq!(fields[0].value, None);
    }

    #[test]
    fn test_parse_select() {
        let html = r#"<select name="country"><option value="us">USA</option></select>"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "country");
        assert_eq!(fields[0].field_type, Some("select".to_string()));
    }

    #[test]
    fn test_parse_label_with_for() {
        let html = r#"
            <label for="username">Username:</label>
            <input type="text" name="username" id="username" />
        "#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "username");
        assert_eq!(fields[0].label, Some("Username:".to_string()));
    }

    #[test]
    fn test_parse_multiple_fields() {
        let html = r#"
            <input type="text" name="username" placeholder="Username" />
            <input type="password" name="password" />
            <textarea name="bio"></textarea>
            <input type="checkbox" name="subscribe" value="yes" />
        "#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0].name, "username");
        assert_eq!(fields[1].name, "password");
        assert_eq!(fields[2].name, "bio");
        assert_eq!(fields[3].name, "subscribe");
    }

    #[test]
    fn test_skip_hidden_inputs() {
        let html = r#"
            <input type="hidden" name="csrf_token" value="abc123" />
            <input type="text" name="username" />
        "#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "username");
    }

    #[test]
    fn test_skip_submit_buttons() {
        let html = r#"
            <input type="text" name="username" />
            <input type="submit" value="Submit" />
            <input type="button" value="Cancel" />
        "#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "username");
    }

    #[test]
    fn test_default_input_type() {
        // Input without type should default to text
        let html = r#"<input name="field1" />"#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "field1");
        assert_eq!(fields[0].field_type, Some("text".to_string()));
    }

    #[test]
    fn test_parse_with_class_attributes() {
        // Real-world example with Tailwind classes
        let html = r#"
            <input type="text" name="username" class="px-4 py-2 border rounded" />
            <input type="password" name="password" class="px-4 py-2 border rounded" />
        "#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "username");
        assert_eq!(fields[1].name, "password");
    }

    #[test]
    fn test_parse_real_world_form() {
        // Example from the task description
        let html = r#"
            <input type="text" name="username" class="..." />
            <input type="password" name="password" />
            <textarea name="bio"></textarea>
            <input type="checkbox" name="subscribe" value="yes" />
        "#;
        let fields = parse_form_html(html);

        assert_eq!(fields.len(), 4);

        assert_eq!(fields[0].name, "username");
        assert_eq!(fields[0].field_type, Some("text".to_string()));

        assert_eq!(fields[1].name, "password");
        assert_eq!(fields[1].field_type, Some("password".to_string()));

        assert_eq!(fields[2].name, "bio");
        assert_eq!(fields[2].field_type, Some("textarea".to_string()));

        assert_eq!(fields[3].name, "subscribe");
        assert_eq!(fields[3].field_type, Some("checkbox".to_string()));
        assert_eq!(fields[3].value, Some("yes".to_string()));
    }
}
