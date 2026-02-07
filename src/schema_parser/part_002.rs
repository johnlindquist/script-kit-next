#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_schema() {
        let content = r#"
schema = {
    input: {
        title: { type: "string", required: true, description: "The title" }
    },
    output: {
        result: { type: "string" }
    }
}
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_some(), "Errors: {:?}", result.errors);
        let schema = result.schema.unwrap();

        assert_eq!(schema.input.len(), 1);
        assert_eq!(schema.output.len(), 1);

        let title_field = schema.input.get("title").unwrap();
        assert_eq!(title_field.field_type, FieldType::String);
        assert!(title_field.required);
        assert_eq!(title_field.description, Some("The title".to_string()));
    }

    #[test]
    fn test_parse_all_field_types() {
        let content = r#"
schema = {
    input: {
        name: { type: "string" },
        count: { type: "number" },
        enabled: { type: "boolean" },
        items: { type: "array", items: "string" },
        config: { type: "object" },
        anything: { type: "any" }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();

        assert_eq!(
            schema.input.get("name").unwrap().field_type,
            FieldType::String
        );
        assert_eq!(
            schema.input.get("count").unwrap().field_type,
            FieldType::Number
        );
        assert_eq!(
            schema.input.get("enabled").unwrap().field_type,
            FieldType::Boolean
        );
        assert_eq!(
            schema.input.get("items").unwrap().field_type,
            FieldType::Array
        );
        assert_eq!(
            schema.input.get("config").unwrap().field_type,
            FieldType::Object
        );
        assert_eq!(
            schema.input.get("anything").unwrap().field_type,
            FieldType::Any
        );
    }

    #[test]
    fn test_parse_field_constraints() {
        let content = r#"
schema = {
    input: {
        username: {
            type: "string",
            required: true,
            min: 3,
            max: 20,
            pattern: "^[a-z]+$"
        },
        age: {
            type: "number",
            min: 0,
            max: 150
        },
        status: {
            type: "string",
            enum: ["active", "inactive", "pending"]
        }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();

        let username = schema.input.get("username").unwrap();
        assert_eq!(username.min, Some(3.0));
        assert_eq!(username.max, Some(20.0));
        assert_eq!(username.pattern, Some("^[a-z]+$".to_string()));

        let status = schema.input.get("status").unwrap();
        assert_eq!(
            status.enum_values,
            Some(vec![
                "active".to_string(),
                "inactive".to_string(),
                "pending".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_with_defaults_and_examples() {
        let content = r#"
schema = {
    input: {
        count: {
            type: "number",
            default: 10,
            example: 42
        }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();

        let count = schema.input.get("count").unwrap();
        assert_eq!(count.default, Some(serde_json::json!(10)));
        assert_eq!(count.example, Some(serde_json::json!(42)));
    }

    #[test]
    fn test_parse_no_schema() {
        let content = r#"
// Just a regular script
const x = await arg("Pick");
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_none());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_parse_input_only() {
        let content = r#"
schema = {
    input: {
        name: { type: "string" }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 1);
        assert_eq!(schema.output.len(), 0);
    }

    #[test]
    fn test_parse_output_only() {
        let content = r#"
schema = {
    output: {
        result: { type: "string" }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 0);
        assert_eq!(schema.output.len(), 1);
    }

    #[test]
    fn test_to_json_schema() {
        let content = r#"
schema = {
    input: {
        title: { type: "string", required: true, description: "Title" },
        count: { type: "number", required: false }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();

        let json_schema = schema.to_json_schema_input();

        assert_eq!(json_schema["type"], "object");
        assert!(json_schema["properties"]["title"].is_object());
        assert_eq!(json_schema["properties"]["title"]["type"], "string");

        let required = json_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("title")));
        assert!(!required.contains(&serde_json::json!("count")));
    }

    #[test]
    fn test_parse_trailing_commas() {
        let content = r#"
schema = {
    input: {
        name: { type: "string", },
    },
    output: {
        result: { type: "boolean", },
    },
}
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_some(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_single_quotes() {
        let content = r#"
schema = {
    input: {
        name: { type: 'string', description: 'The name' }
    }
}
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_some());
        let schema = result.schema.unwrap();
        assert_eq!(
            schema.input.get("name").unwrap().description,
            Some("The name".to_string())
        );
    }

    #[test]
    fn test_span_tracking() {
        let content = r#"// Header
schema = { input: { x: { type: "string" } } }
const y = 1;"#;
        let result = extract_schema(content);
        assert!(result.span.is_some());
        let (start, end) = result.span.unwrap();
        let extracted = &content[start..end];
        assert!(extracted.contains("schema"));
        assert!(extracted.contains("input"));
    }

    #[test]
    fn test_invalid_schema_reports_error() {
        let content = r#"
schema = {
    input: {
        bad: { type: "invalid_type" }
    }
}
"#;
        let result = extract_schema(content);
        // serde should error on unknown enum variant
        assert!(result.schema.is_none() || result.schema.as_ref().unwrap().input.is_empty());
    }

    // TDD: Test for defineSchema() function pattern
    // This pattern should also work for MCP tool detection
    #[test]
    fn test_parse_define_schema_function() {
        let content = r#"
import "@johnlindquist/kit"

const { input, output } = defineSchema({
    input: {
        greeting: { type: "string", required: true, description: "Greeting message" },
        count: { type: "number" }
    },
    output: {
        message: { type: "string", description: "Response message" }
    }
} as const)

const { greeting } = await input()
output({ message: `Hello ${greeting}!` })
"#;
        let result = extract_schema(content);
        assert!(
            result.schema.is_some(),
            "defineSchema() should be parseable. Errors: {:?}",
            result.errors
        );

        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 2, "Should have 2 input fields");
        assert_eq!(schema.output.len(), 1, "Should have 1 output field");

        // Verify input fields
        let greeting = schema
            .input
            .get("greeting")
            .expect("Should have greeting field");
        assert!(greeting.required, "greeting should be required");
        assert_eq!(greeting.description, Some("Greeting message".to_string()));

        let count = schema.input.get("count").expect("Should have count field");
        assert!(!count.required, "count should not be required");

        // Verify output fields
        let message = schema
            .output
            .get("message")
            .expect("Should have message field");
        assert_eq!(message.description, Some("Response message".to_string()));
    }

    // TDD: Test that both patterns work (direct assignment and defineSchema)
    #[test]
    fn test_parse_both_schema_patterns() {
        // Direct assignment pattern
        let direct = r#"
schema = {
    input: { name: { type: "string", required: true } }
}
"#;
        let result1 = extract_schema(direct);
        assert!(result1.schema.is_some(), "Direct assignment should work");

        // defineSchema function pattern
        let define_fn = r#"
const { input, output } = defineSchema({
    input: { name: { type: "string", required: true } }
} as const)
"#;
        let result2 = extract_schema(define_fn);
        assert!(result2.schema.is_some(), "defineSchema() should work");

        // Both should produce same schema
        let schema1 = result1.schema.unwrap();
        let schema2 = result2.schema.unwrap();
        assert_eq!(schema1.input.len(), schema2.input.len());
        assert_eq!(
            schema1.input.get("name").unwrap().required,
            schema2.input.get("name").unwrap().required
        );
    }

    #[test]
    fn test_parse_array_items_as_object_schema() {
        let content = r#"
schema = {
  input: {
    transforms: {
      type: "array",
      required: true,
      items: { type: "string", enum: ["uppercase", "lowercase"] }
    }
  }
}
"#;

        let result = extract_schema(content);
        assert!(result.schema.is_some(), "Errors: {:?}", result.errors);

        let schema = result.schema.unwrap();
        let transforms = schema.input.get("transforms").expect("missing transforms");
        assert_eq!(transforms.field_type, FieldType::Array);

        // Verify items parsed as a nested schema object
        let items = transforms.items.as_ref().expect("missing items");
        match items {
            ItemsDef::Schema(def) => {
                assert_eq!(def.field_type, FieldType::String);
                assert_eq!(
                    def.enum_values,
                    Some(vec!["uppercase".to_string(), "lowercase".to_string()])
                );
            }
            _ => panic!("Expected items to be a schema object"),
        }

        // Verify JSON Schema output includes items.enum properly
        let json_schema = schema.to_json_schema_input();
        let items_schema = &json_schema["properties"]["transforms"]["items"];
        assert_eq!(items_schema["type"], "string");
        let enum_vals = items_schema["enum"].as_array().expect("missing enum");
        assert!(enum_vals.contains(&serde_json::json!("uppercase")));
        assert!(enum_vals.contains(&serde_json::json!("lowercase")));
    }

    #[test]
    fn test_array_min_max_become_min_items_max_items() {
        let content = r#"
schema = {
  input: {
    tags: { type: "array", min: 1, max: 5, items: "string" }
  }
}
"#;

        let result = extract_schema(content);
        assert!(result.schema.is_some(), "Errors: {:?}", result.errors);

        let schema = result.schema.unwrap();
        let json_schema = schema.to_json_schema_input();
        let tags = &json_schema["properties"]["tags"];

        assert_eq!(tags["type"], "array");
        assert_eq!(tags["minItems"], 1);
        assert_eq!(tags["maxItems"], 5);
        assert_eq!(tags["items"]["type"], "string");
    }
}
