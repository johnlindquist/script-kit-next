//! Tests for action_helpers module.
//!
//! NOTE: Some tests are limited because SearchResult variants require
//! types with complex constructors (WindowInfo, AgentMatch).
//! The core functionality is tested via extract_path_for_* with Script/Scriptlet.

use super::*;
use crate::builtins::{BuiltInEntry, BuiltInFeature, BuiltInGroup};
use crate::scripts::{BuiltInMatch, MatchIndices, Script, ScriptMatch, Scriptlet, ScriptletMatch};
use std::path::PathBuf;
use std::sync::Arc;

fn make_script(name: &str, path: &str) -> Arc<Script> {
    Arc::new(Script {
        name: name.to_string(),
        path: PathBuf::from(path),
        extension: "ts".to_string(),
        description: None,
        icon: None,
        alias: None,
        shortcut: None,
        typed_metadata: None,
        schema: None,
    })
}

fn make_script_match(name: &str, path: &str) -> ScriptMatch {
    ScriptMatch {
        script: make_script(name, path),
        score: 100,
        filename: format!("{}.ts", name),
        match_indices: MatchIndices::default(),
    }
}

fn make_scriptlet_match() -> ScriptletMatch {
    ScriptletMatch {
        scriptlet: Arc::new(Scriptlet {
            name: "test-scriptlet".to_string(),
            description: None,
            code: "console.log('test')".to_string(),
            tool: "ts".to_string(),
            shortcut: None,
            expand: None,
            group: None,
            file_path: None,
            command: None,
            alias: None,
        }),
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
    }
}

fn make_builtin_match() -> BuiltInMatch {
    BuiltInMatch {
        entry: BuiltInEntry {
            id: "clipboard_history".to_string(),
            name: "Clipboard History".to_string(),
            description: "View clipboard history".to_string(),
            keywords: vec!["clipboard".to_string()],
            feature: BuiltInFeature::ClipboardHistory,
            icon: None,
            group: BuiltInGroup::Core,
        },
        score: 100,
    }
}

// Tests for extract_path_for_reveal

#[test]
fn test_extract_path_for_reveal_none() {
    let result = extract_path_for_reveal(None);
    assert!(matches!(result, Err(PathExtractionError::NoSelection)));
    assert_eq!(result.unwrap_err().message().as_ref(), "No item selected");
}

#[test]
fn test_extract_path_for_reveal_script() {
    let script_match = make_script_match("test", "/path/to/test.ts");
    let result = extract_path_for_reveal(Some(&SearchResult::Script(script_match)));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("/path/to/test.ts"));
}

#[test]
fn test_extract_path_for_reveal_scriptlet() {
    let scriptlet_match = make_scriptlet_match();
    let result = extract_path_for_reveal(Some(&SearchResult::Scriptlet(scriptlet_match)));
    assert!(matches!(
        result,
        Err(PathExtractionError::UnsupportedType(_))
    ));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot reveal scriptlets in Finder"
    );
}

#[test]
fn test_extract_path_for_reveal_builtin() {
    let builtin_match = make_builtin_match();
    let result = extract_path_for_reveal(Some(&SearchResult::BuiltIn(builtin_match)));
    assert!(matches!(
        result,
        Err(PathExtractionError::UnsupportedType(_))
    ));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot reveal built-in features"
    );
}

// Tests for extract_path_for_copy

#[test]
fn test_extract_path_for_copy_none() {
    let result = extract_path_for_copy(None);
    assert!(matches!(result, Err(PathExtractionError::NoSelection)));
}

#[test]
fn test_extract_path_for_copy_script() {
    let script_match = make_script_match("test", "/path/to/test.ts");
    let result = extract_path_for_copy(Some(&SearchResult::Script(script_match)));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("/path/to/test.ts"));
}

#[test]
fn test_extract_path_for_copy_scriptlet() {
    let scriptlet_match = make_scriptlet_match();
    let result = extract_path_for_copy(Some(&SearchResult::Scriptlet(scriptlet_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot copy scriptlet path"
    );
}

// Tests for extract_path_for_edit

#[test]
fn test_extract_path_for_edit_none() {
    let result = extract_path_for_edit(None);
    assert!(matches!(result, Err(PathExtractionError::NoSelection)));
}

#[test]
fn test_extract_path_for_edit_script() {
    let script_match = make_script_match("test", "/path/to/test.ts");
    let result = extract_path_for_edit(Some(&SearchResult::Script(script_match)));
    assert!(result.is_ok());
}

#[test]
fn test_extract_path_for_edit_scriptlet() {
    let scriptlet_match = make_scriptlet_match();
    let result = extract_path_for_edit(Some(&SearchResult::Scriptlet(scriptlet_match)));
    assert_eq!(
        result.unwrap_err().message().as_ref(),
        "Cannot edit scriptlets"
    );
}

// Tests for reserved action IDs

#[test]
fn test_is_reserved_action_id() {
    assert!(is_reserved_action_id("copy_path"));
    assert!(is_reserved_action_id("edit_script"));
    assert!(is_reserved_action_id("copy_deeplink"));
    assert!(is_reserved_action_id("__cancel__"));

    assert!(!is_reserved_action_id("custom_action"));
    assert!(!is_reserved_action_id("quit")); // quit is no longer reserved (main menu only)
    assert!(!is_reserved_action_id(""));
}

#[test]
fn test_find_sdk_action_none() {
    let result = find_sdk_action(None, "test", false);
    assert!(result.is_none());
}

#[test]
fn test_find_sdk_action_found() {
    let actions = vec![
        ProtocolAction {
            name: "test_action".to_string(),
            description: Some("Test".to_string()),
            shortcut: None,
            value: Some("value".to_string()),
            has_action: true,
            visible: None,
            close: None,
        },
        ProtocolAction {
            name: "other_action".to_string(),
            description: None,
            shortcut: None,
            value: None,
            has_action: false,
            visible: None,
            close: None,
        },
    ];

    let result = find_sdk_action(Some(&actions), "test_action", false);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "test_action");

    let result = find_sdk_action(Some(&actions), "not_found", false);
    assert!(result.is_none());
}

// Tests for trigger_sdk_action

#[test]
fn test_trigger_sdk_action_no_sender() {
    let action = ProtocolAction {
        name: "test".to_string(),
        description: None,
        shortcut: None,
        value: Some("value".to_string()),
        has_action: true,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("test", &action, "", None);
    assert!(!result);
}

#[test]
fn test_trigger_sdk_action_with_handler() {
    use std::sync::mpsc;

    let (sender, receiver) = mpsc::sync_channel::<protocol::Message>(10);

    let action = ProtocolAction {
        name: "test".to_string(),
        description: None,
        shortcut: None,
        value: Some("value".to_string()),
        has_action: true,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("test", &action, "current input", Some(&sender));
    assert!(result);

    let msg = receiver.try_recv().unwrap();
    match msg {
        protocol::Message::ActionTriggered {
            action,
            value,
            input,
        } => {
            assert_eq!(action, "test");
            assert_eq!(value, Some("value".to_string()));
            assert_eq!(input, "current input");
        }
        _ => panic!("Expected ActionTriggered message"),
    }
}

#[test]
fn test_trigger_sdk_action_without_handler_with_value() {
    use std::sync::mpsc;

    let (sender, receiver) = mpsc::sync_channel::<protocol::Message>(10);

    let action = ProtocolAction {
        name: "test".to_string(),
        description: None,
        shortcut: None,
        value: Some("submit_value".to_string()),
        has_action: false,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("test", &action, "", Some(&sender));
    assert!(result);

    let msg = receiver.try_recv().unwrap();
    match msg {
        protocol::Message::Submit { id, value } => {
            assert_eq!(id, "action");
            assert_eq!(value, Some("submit_value".to_string()));
        }
        _ => panic!("Expected Submit message"),
    }
}

#[test]
fn test_trigger_sdk_action_without_handler_without_value() {
    use std::sync::mpsc;

    let (sender, _receiver) = mpsc::sync_channel::<protocol::Message>(10);

    let action = ProtocolAction {
        name: "test".to_string(),
        description: None,
        shortcut: None,
        value: None,
        has_action: false,
        visible: None,
        close: None,
    };

    let result = trigger_sdk_action("test", &action, "", Some(&sender));
    assert!(!result); // No message sent when has_action=false and value=None
}

// Tests for pbcopy (macOS only)

#[cfg(target_os = "macos")]
#[test]
fn test_pbcopy_basic() {
    let result = pbcopy("test clipboard content");
    assert!(result.is_ok());
}

#[cfg(target_os = "macos")]
#[test]
fn test_pbcopy_empty_string() {
    let result = pbcopy("");
    assert!(result.is_ok());
}

#[cfg(target_os = "macos")]
#[test]
fn test_pbcopy_unicode() {
    let result = pbcopy("Hello 🌍 世界");
    assert!(result.is_ok());
}
