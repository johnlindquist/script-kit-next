use super::*;

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_placeholder() {
        let inputs = TemplatePrompt::parse_template_inputs("Hello {{name}}!");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].name, "name");
    }

    #[test]
    fn test_parse_multiple_placeholders() {
        let inputs =
            TemplatePrompt::parse_template_inputs("Hello {{name}}, your email is {{email}}");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "name");
        assert_eq!(inputs[1].name, "email");
    }

    #[test]
    fn test_parse_duplicate_placeholders() {
        let inputs =
            TemplatePrompt::parse_template_inputs("{{name}} is {{name}}'s name, email: {{email}}");
        assert_eq!(inputs.len(), 2); // Duplicates should be removed
        assert_eq!(inputs[0].name, "name");
        assert_eq!(inputs[1].name, "email");
    }

    #[test]
    fn test_parse_no_placeholders() {
        let inputs = TemplatePrompt::parse_template_inputs("Hello world!");
        assert_eq!(inputs.len(), 0);
    }

    #[test]
    fn test_parse_placeholder_with_underscore() {
        let inputs = TemplatePrompt::parse_template_inputs("Hello {{first_name}} {{last_name}}!");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "first_name");
        assert_eq!(inputs[1].name, "last_name");
    }

    #[test]
    fn test_parse_placeholder_with_numbers() {
        let inputs = TemplatePrompt::parse_template_inputs("Field {{field1}} and {{field2}}");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "field1");
        assert_eq!(inputs[1].name, "field2");
    }

    #[test]
    fn test_parse_dollar_brace_placeholders() {
        let inputs =
            TemplatePrompt::parse_template_inputs("Script ${script_name} by ${author_name}");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "script_name");
        assert_eq!(inputs[1].name, "author_name");
    }

    #[test]
    fn test_parse_brace_placeholders_with_whitespace_and_skip_control_tags() {
        let inputs = TemplatePrompt::parse_template_inputs(
            "Hello {{ first_name }} {{#if cond}}ignored{{/if}} {{ else }} {{last_name}}",
        );
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "first_name");
        assert_eq!(inputs[1].name, "last_name");
    }

    #[test]
    fn test_parse_skips_javascript_expressions_in_dollar_syntax() {
        let inputs =
            TemplatePrompt::parse_template_inputs("${await clipboard.readText()} {{name}}");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].name, "name");
    }

    #[test]
    fn test_template_inputs_use_human_labels_and_groups() {
        let inputs = TemplatePrompt::parse_template_inputs(
            "Name {{script_name}} by {{author}}: {{description}}",
        );
        assert_eq!(inputs.len(), 3);

        assert_eq!(inputs[0].label, "Script Name");
        assert_eq!(inputs[0].placeholder, "my-script-name");
        assert_eq!(inputs[0].group, "Naming");

        assert_eq!(inputs[1].label, "Author");
        assert_eq!(inputs[1].placeholder, "Your Name");
        assert_eq!(inputs[1].group, "Metadata");

        assert_eq!(inputs[2].label, "Description");
        assert_eq!(inputs[2].placeholder, "What this template creates");
        assert_eq!(inputs[2].group, "Metadata");
    }

    #[test]
    fn test_template_prompt_substitute_single_pass_does_not_rewrite_user_literal_placeholders() {
        let rendered = TemplatePrompt::render_template_single_pass(
            "{{first}} and {{second}}",
            |name, raw_placeholder| match name {
                "first" => "{{second}}".to_string(),
                "second" => "done".to_string(),
                _ => raw_placeholder.to_string(),
            },
        );

        assert_eq!(rendered, "{{second}} and done");
    }

    #[test]
    fn test_single_pass_substitution_skips_javascript_style_expressions() {
        let rendered = TemplatePrompt::render_template_single_pass(
            "${await clipboard.readText()} {{name}}",
            |name, raw_placeholder| match name {
                "name" => "Alice".to_string(),
                _ => raw_placeholder.to_string(),
            },
        );

        assert_eq!(rendered, "${await clipboard.readText()} Alice");
    }

    #[test]
    fn test_validate_name_inputs_require_slug_like_values() {
        let input = TemplateInput {
            name: "script_name".to_string(),
            label: "Script Name".to_string(),
            placeholder: "my-script-name".to_string(),
            group: "Naming".to_string(),
            required: true,
        };

        let err = TemplatePrompt::validate_input_value(&input, "My Cool Script")
            .expect_err("spaces should fail slug validation for script naming");
        assert!(err.contains("letters, numbers, and hyphens"));

        assert!(TemplatePrompt::validate_input_value(&input, "my-cool-script").is_ok());
    }

    #[test]
    fn test_validate_required_inputs_reject_empty_trimmed_values() {
        let input = TemplateInput {
            name: "script_name".to_string(),
            label: "Script Name".to_string(),
            placeholder: "my-script-name".to_string(),
            group: "Naming".to_string(),
            required: true,
        };

        let err = TemplatePrompt::validate_input_value(&input, "   ")
            .expect_err("required inputs should reject empty trimmed values");
        assert_eq!(err, "Script Name is required");
    }

    #[test]
    fn test_validate_optional_non_name_inputs_allow_empty_values() {
        let input = TemplateInput {
            name: "description".to_string(),
            label: "Description".to_string(),
            placeholder: "What this template creates".to_string(),
            group: "Metadata".to_string(),
            required: false,
        };

        assert!(TemplatePrompt::validate_input_value(&input, "").is_ok());
        assert!(TemplatePrompt::validate_input_value(&input, "   ").is_ok());
    }

    #[test]
    fn test_validate_slug_like_values_reject_invalid_hyphen_patterns() {
        let input = TemplateInput {
            name: "extension_name".to_string(),
            label: "Scriptlet Bundle Name".to_string(),
            placeholder: "my-scriptlet-bundle".to_string(),
            group: "Naming".to_string(),
            required: true,
        };

        for invalid in ["-starts", "ends-", "double--hyphen"] {
            let err = TemplatePrompt::validate_input_value(&input, invalid)
                .expect_err("invalid slug-like values should fail validation");
            assert!(err.contains("letters, numbers, and hyphens"));
        }
    }
}
