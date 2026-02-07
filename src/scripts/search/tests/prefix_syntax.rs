use std::path::PathBuf;

use super::super::*;

#[test]
fn test_parse_query_prefix_tag() {
    let parsed = parse_query_prefix("tag:productivity");
    assert_eq!(parsed.filter_kind.as_deref(), Some("tag"));
    assert_eq!(parsed.filter_value.as_deref(), Some("productivity"));
    assert_eq!(parsed.remainder, "");
}

#[test]
fn test_parse_query_prefix_tag_with_remainder() {
    let parsed = parse_query_prefix("tag:productivity notes");
    assert_eq!(parsed.filter_kind.as_deref(), Some("tag"));
    assert_eq!(parsed.filter_value.as_deref(), Some("productivity"));
    assert_eq!(parsed.remainder, "notes");
}

#[test]
fn test_parse_query_prefix_no_prefix() {
    let parsed = parse_query_prefix("hello world");
    assert_eq!(parsed.filter_kind, None);
    assert_eq!(parsed.filter_value, None);
    assert_eq!(parsed.remainder, "hello world");
}

#[test]
fn test_parse_query_prefix_empty_value() {
    let parsed = parse_query_prefix("tag:");
    assert_eq!(parsed.filter_kind, None); // empty value = not a filter
}

#[test]
fn test_parse_query_prefix_is_cron() {
    let parsed = parse_query_prefix("is:cron");
    assert_eq!(parsed.filter_kind.as_deref(), Some("is"));
    assert_eq!(parsed.filter_value.as_deref(), Some("cron"));
}

#[test]
fn test_parse_query_prefix_type_script() {
    let parsed = parse_query_prefix("type:script");
    assert_eq!(parsed.filter_kind.as_deref(), Some("type"));
    assert_eq!(parsed.filter_value.as_deref(), Some("script"));
}

#[test]
fn test_parse_query_prefix_author() {
    let parsed = parse_query_prefix("author:john search term");
    assert_eq!(parsed.filter_kind.as_deref(), Some("author"));
    assert_eq!(parsed.filter_value.as_deref(), Some("john"));
    assert_eq!(parsed.remainder, "search term");
}

#[test]
fn test_script_passes_tag_filter() {
    use crate::metadata_parser::TypedMetadata;
    let mut script = Script {
        name: "Test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        ..Default::default()
    };
    let meta = TypedMetadata {
        tags: vec!["productivity".to_string(), "notes".to_string()],
        ..Default::default()
    };
    script.typed_metadata = Some(meta);

    let parsed = parse_query_prefix("tag:prod");
    assert!(script_passes_prefix_filter(&script, &parsed));

    let parsed_no = parse_query_prefix("tag:gaming");
    assert!(!script_passes_prefix_filter(&script, &parsed_no));
}

#[test]
fn test_script_passes_author_filter() {
    use crate::metadata_parser::TypedMetadata;
    let mut script = Script {
        name: "Test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        ..Default::default()
    };
    let meta = TypedMetadata {
        author: Some("John Lindquist".to_string()),
        ..Default::default()
    };
    script.typed_metadata = Some(meta);

    let parsed = parse_query_prefix("author:john");
    assert!(script_passes_prefix_filter(&script, &parsed));
}

#[test]
fn test_script_passes_kit_filter() {
    let script = Script {
        name: "Test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        kit_name: Some("cleanshot".to_string()),
        ..Default::default()
    };

    let parsed = parse_query_prefix("kit:cleanshot");
    assert!(script_passes_prefix_filter(&script, &parsed));

    let parsed_no = parse_query_prefix("kit:main");
    assert!(!script_passes_prefix_filter(&script, &parsed_no));
}

#[test]
fn test_script_passes_is_cron_filter() {
    use crate::metadata_parser::TypedMetadata;
    let mut script = Script {
        name: "Backup".to_string(),
        path: PathBuf::from("/backup.ts"),
        extension: "ts".to_string(),
        ..Default::default()
    };
    let meta = TypedMetadata {
        cron: Some("0 0 * * *".to_string()),
        ..Default::default()
    };
    script.typed_metadata = Some(meta);

    let parsed = parse_query_prefix("is:cron");
    assert!(script_passes_prefix_filter(&script, &parsed));

    let parsed_sched = parse_query_prefix("is:scheduled");
    assert!(script_passes_prefix_filter(&script, &parsed_sched));
}

#[test]
fn test_script_passes_is_bg_filter() {
    use crate::metadata_parser::TypedMetadata;
    let mut script = Script {
        name: "Monitor".to_string(),
        path: PathBuf::from("/monitor.ts"),
        extension: "ts".to_string(),
        ..Default::default()
    };
    let meta = TypedMetadata {
        background: true,
        ..Default::default()
    };
    script.typed_metadata = Some(meta);

    let parsed = parse_query_prefix("is:bg");
    assert!(script_passes_prefix_filter(&script, &parsed));

    let parsed_full = parse_query_prefix("is:background");
    assert!(script_passes_prefix_filter(&script, &parsed_full));
}

#[test]
fn test_script_fails_wrong_is_filter() {
    let script = Script {
        name: "Test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        ..Default::default()
    };

    let parsed = parse_query_prefix("is:cron");
    assert!(!script_passes_prefix_filter(&script, &parsed));
}

#[test]
fn test_scriptlet_passes_group_filter() {
    let scriptlet = Scriptlet {
        name: "Deploy".to_string(),
        code: "echo deploy".to_string(),
        tool: "bash".to_string(),
        group: Some("Development".to_string()),
        description: None,
        shortcut: None,
        keyword: None,
        file_path: None,
        command: None,
        alias: None,
    };

    let parsed = parse_query_prefix("group:dev");
    assert!(scriptlet_passes_prefix_filter(&scriptlet, &parsed));
}

#[test]
fn test_scriptlet_passes_tool_filter() {
    let scriptlet = Scriptlet {
        name: "Deploy".to_string(),
        code: "echo deploy".to_string(),
        tool: "bash".to_string(),
        group: None,
        description: None,
        shortcut: None,
        keyword: None,
        file_path: None,
        command: None,
        alias: None,
    };

    let parsed = parse_query_prefix("tool:bash");
    assert!(scriptlet_passes_prefix_filter(&scriptlet, &parsed));

    let parsed_display = parse_query_prefix("tool:shell");
    assert!(scriptlet_passes_prefix_filter(&scriptlet, &parsed_display));
}

#[test]
fn test_type_filter_script_excludes_scriptlets() {
    let parsed = parse_query_prefix("type:script");
    // Scripts should pass
    let script = Script {
        name: "Test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        ..Default::default()
    };
    assert!(script_passes_prefix_filter(&script, &parsed));

    // Scriptlets should not pass
    let scriptlet = Scriptlet {
        name: "Snippet".to_string(),
        code: "echo hi".to_string(),
        tool: "bash".to_string(),
        group: None,
        description: None,
        shortcut: None,
        keyword: None,
        file_path: None,
        command: None,
        alias: None,
    };
    assert!(!scriptlet_passes_prefix_filter(&scriptlet, &parsed));
}

#[test]
fn test_type_filter_snippet_excludes_scripts() {
    let parsed = parse_query_prefix("type:snippet");
    let script = Script {
        name: "Test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        ..Default::default()
    };
    assert!(!script_passes_prefix_filter(&script, &parsed));
}

#[test]
fn test_builtin_prefix_filter_allows_command_type_and_rejects_non_builtin_types() {
    let command_filter = parse_query_prefix("type:command");
    assert!(
        builtin_passes_prefix_filter(&command_filter),
        "type:command should include built-ins"
    );

    let builtin_filter = parse_query_prefix("type:builtins");
    assert!(
        builtin_passes_prefix_filter(&builtin_filter),
        "type:builtins should include built-ins"
    );

    let script_filter = parse_query_prefix("type:script");
    assert!(
        !builtin_passes_prefix_filter(&script_filter),
        "type:script should exclude built-ins"
    );
}

#[test]
fn test_no_filter_passes_everything() {
    let parsed = parse_query_prefix("hello");
    let script = Script {
        name: "Test".to_string(),
        path: PathBuf::from("/test.ts"),
        extension: "ts".to_string(),
        ..Default::default()
    };
    assert!(script_passes_prefix_filter(&script, &parsed));
}
