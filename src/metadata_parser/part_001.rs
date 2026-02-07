#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_metadata() {
        let content = r#"
metadata = {
    name: "My Script",
    description: "Does something cool"
}

const result = await arg("Pick one");
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("My Script".to_string()));
        assert_eq!(meta.description, Some("Does something cool".to_string()));
    }

    #[test]
    fn test_parse_all_fields() {
        let content = r#"
metadata = {
    name: "Full Script",
    description: "A script with all fields",
    author: "John Doe",
    enter: "Execute",
    alias: "fs",
    icon: "Star",
    shortcut: "cmd shift f",
    tags: ["productivity", "utility"],
    hidden: false,
    placeholder: "Type something...",
    cron: "0 9 * * *",
    watch: ["*.ts", "*.js"],
    background: false,
    system: false
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();

        assert_eq!(meta.name, Some("Full Script".to_string()));
        assert_eq!(
            meta.description,
            Some("A script with all fields".to_string())
        );
        assert_eq!(meta.author, Some("John Doe".to_string()));
        assert_eq!(meta.enter, Some("Execute".to_string()));
        assert_eq!(meta.alias, Some("fs".to_string()));
        assert_eq!(meta.icon, Some("Star".to_string()));
        assert_eq!(meta.shortcut, Some("cmd shift f".to_string()));
        assert_eq!(meta.tags, vec!["productivity", "utility"]);
        assert!(!meta.hidden);
        assert_eq!(meta.placeholder, Some("Type something...".to_string()));
        assert_eq!(meta.cron, Some("0 9 * * *".to_string()));
        assert_eq!(meta.watch, vec!["*.ts", "*.js"]);
        assert!(!meta.background);
        assert!(!meta.system);
    }

    #[test]
    fn test_parse_with_single_quotes() {
        let content = r#"
metadata = {
    name: 'Single Quoted',
    description: 'Uses single quotes'
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Single Quoted".to_string()));
    }

    #[test]
    fn test_parse_with_trailing_comma() {
        let content = r#"
metadata = {
    name: "Trailing Comma",
    description: "Has a trailing comma",
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Trailing Comma".to_string()));
    }

    #[test]
    fn test_parse_no_metadata() {
        let content = r#"
// Name: Old Style
// Description: Uses comments

const result = await arg("Pick");
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_none());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_parse_metadata_no_spaces() {
        let content = r#"metadata={name:"NoSpaces"}"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("NoSpaces".to_string()));
    }

    #[test]
    fn test_parse_extra_fields() {
        let content = r#"
metadata = {
    name: "With Extras",
    customField: "custom value",
    anotherField: 42
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("With Extras".to_string()));
        assert!(meta.extra.contains_key("customField"));
    }

    #[test]
    fn test_parse_nested_objects_in_string() {
        let content = r#"
metadata = {
    name: "Has JSON in string",
    description: "Contains {nested: \"json\"} in description"
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert!(meta.description.unwrap().contains("{nested:"));
    }

    #[test]
    fn test_span_tracking() {
        let content = r#"// Comment
metadata = {
    name: "Test"
}
const x = 1;"#;
        let result = extract_typed_metadata(content);
        assert!(result.span.is_some());
        let (start, end) = result.span.unwrap();
        let extracted = &content[start..end];
        assert!(extracted.contains("metadata"));
        assert!(extracted.contains("name"));
    }

    #[test]
    fn test_invalid_json_reports_error() {
        let content = r#"
metadata = {
    name: "Bad JSON,
    description: missing closing quote
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_none());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_defaults_for_missing_optional_fields() {
        let content = r#"metadata = { name: "Minimal" }"#;
        let result = extract_typed_metadata(content);
        let meta = result.metadata.unwrap();

        assert_eq!(meta.name, Some("Minimal".to_string()));
        assert_eq!(meta.description, None);
        assert_eq!(meta.tags, Vec::<String>::new());
        assert!(!meta.hidden);
        assert!(!meta.background);
    }

    #[test]
    fn test_parse_schedule_field() {
        let content = r#"
metadata = {
    name: "Scheduled Script",
    description: "Runs on a schedule",
    schedule: "every tuesday at 2pm"
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Scheduled Script".to_string()));
        assert_eq!(meta.schedule, Some("every tuesday at 2pm".to_string()));
    }

    #[test]
    fn test_parse_cron_and_schedule_together() {
        let content = r#"
metadata = {
    name: "Dual Scheduled",
    cron: "0 14 * * 2",
    schedule: "every tuesday at 2pm"
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();
        assert_eq!(meta.cron, Some("0 14 * * 2".to_string()));
        assert_eq!(meta.schedule, Some("every tuesday at 2pm".to_string()));
    }

    #[test]
    fn test_parse_fallback_fields() {
        let content = r#"
metadata = {
    name: "Search Docs",
    description: "Search documentation for a term",
    fallback: true,
    fallbackLabel: "Search docs for {input}"
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Search Docs".to_string()));
        assert!(meta.fallback, "fallback should be true");
        assert_eq!(
            meta.fallback_label,
            Some("Search docs for {input}".to_string())
        );
    }

    #[test]
    fn test_parse_fallback_without_label() {
        let content = r#"
metadata = {
    name: "Web Search",
    fallback: true
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();
        assert!(meta.fallback, "fallback should be true");
        assert_eq!(
            meta.fallback_label, None,
            "fallback_label should be None when not provided"
        );
    }

    #[test]
    fn test_fallback_defaults_to_false() {
        let content = r#"
metadata = {
    name: "Regular Script"
}
"#;
        let result = extract_typed_metadata(content);
        let meta = result.metadata.unwrap();
        assert!(!meta.fallback, "fallback should default to false");
        assert_eq!(meta.fallback_label, None);
    }
}
