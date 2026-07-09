use super::{main_window_result_action_label, paste_into_frontmost_app_label};
use crate::scripts::{MatchIndices, Scriptlet, ScriptletMatch};
use std::sync::Arc;

fn make_scriptlet_result(tool: &str) -> crate::scripts::SearchResult {
    crate::scripts::SearchResult::Scriptlet(ScriptletMatch {
        scriptlet: Arc::new(Scriptlet {
            name: "Test Scriptlet".to_string(),
            description: None,
            code: "echo test".to_string(),
            tool: tool.to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            plugin_id: String::new(),
            plugin_title: None,
            file_path: None,
            command: None,
            alias: None,
            icon: None,
        }),
        score: 100,
        display_file_path: None,
        match_indices: MatchIndices::default(),
        match_evidence: None,
    })
}

#[test]
fn paste_into_frontmost_app_label_uses_app_name() {
    assert_eq!(
        paste_into_frontmost_app_label(Some("Safari")),
        "Paste into Safari"
    );
}

#[test]
fn paste_into_frontmost_app_label_falls_back_to_active_app() {
    assert_eq!(
        paste_into_frontmost_app_label(None),
        "Paste into Active App"
    );
}

#[test]
fn main_window_result_action_label_uses_frontmost_app_for_paste_scriptlets() {
    let result = make_scriptlet_result("paste");
    assert_eq!(
        main_window_result_action_label(&result, Some("TextEdit")),
        "Paste into TextEdit"
    );
}

#[test]
fn main_window_result_action_label_keeps_default_for_non_paste_scriptlets() {
    let result = make_scriptlet_result("bash");
    assert_eq!(
        main_window_result_action_label(&result, Some("TextEdit")),
        "Run Command"
    );
}
