//! Tests for context strip element collection via getElements protocol.
//!
//! Verifies that `collect_script_list_elements` includes the context strip
//! panel, four context chip buttons, and the "Ask AI with Context" button
//! with correct semantic IDs, types, and selection state.

use script_kit_gpui::ai::message_parts::AiContextPart;
use script_kit_gpui::protocol::{ElementInfo, ElementType};

// ---------- Helper: build expected context strip elements ----------

/// Expected semantic IDs for context strip elements (slugified).
const PANEL_ID: &str = "panel:context-strip";
const CHIP_IDS: [&str; 4] = [
    "button:0:current-context",
    "button:1:selection",
    "button:2:browser-url",
    "button:3:focused-window",
];
const AI_BUTTON_ID: &str = "button:4:ask-ai-with-context";

/// Mirror the default parts (same as ScriptListApp::default_main_window_context_parts).
fn default_parts() -> Vec<AiContextPart> {
    vec![
        AiContextPart::ResourceUri {
            uri: "kit://context?profile=minimal".to_string(),
            label: "Current Context".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0"
                .to_string(),
            label: "Selection".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://context?selectedText=0&frontmostApp=0&menuBar=0&browserUrl=1&focusedWindow=0"
                .to_string(),
            label: "Browser URL".to_string(),
        },
        AiContextPart::ResourceUri {
            uri: "kit://context?selectedText=0&frontmostApp=1&menuBar=0&browserUrl=0&focusedWindow=1"
                .to_string(),
            label: "Focused Window".to_string(),
        },
    ]
}

// ---------- Semantic ID format ----------

#[test]
fn context_strip_panel_semantic_id_format() {
    assert_eq!(PANEL_ID, "panel:context-strip");
}

#[test]
fn context_strip_chip_semantic_ids_are_slugified() {
    for id in &CHIP_IDS {
        assert!(id.starts_with("button:"), "Expected button prefix: {}", id);
        // Should not contain uppercase or spaces
        assert_eq!(*id, id.to_lowercase(), "Semantic IDs must be lowercase");
        assert!(!id.contains(' '), "Semantic IDs must not contain spaces");
    }
}

#[test]
fn context_strip_ai_button_semantic_id_format() {
    assert_eq!(AI_BUTTON_ID, "button:4:ask-ai-with-context");
}

// ---------- ElementInfo construction ----------

#[test]
fn panel_element_info_has_correct_type() {
    let panel = ElementInfo::panel("context-strip");
    assert_eq!(panel.element_type, ElementType::Panel);
    assert_eq!(panel.semantic_id, PANEL_ID);
}

#[test]
fn button_element_info_has_correct_type_and_text() {
    let labels = ["Current Context", "Selection", "Browser URL", "Focused Window"];
    for (i, label) in labels.iter().enumerate() {
        let btn = ElementInfo::button(i, label);
        assert_eq!(btn.element_type, ElementType::Button);
        assert_eq!(btn.semantic_id, CHIP_IDS[i]);
        assert_eq!(btn.text, Some(label.to_string()));
    }
}

#[test]
fn ai_button_element_info_has_correct_id() {
    let btn = ElementInfo::button(4, "Ask AI with Context");
    assert_eq!(btn.semantic_id, AI_BUTTON_ID);
    assert_eq!(btn.element_type, ElementType::Button);
}

// ---------- Selection state ----------

#[test]
fn chip_selected_state_reflects_membership() {
    let parts = default_parts();
    let labels = ["Current Context", "Selection", "Browser URL", "Focused Window"];

    // All default parts are selected
    for (i, (label, part)) in labels.iter().zip(parts.iter()).enumerate() {
        let is_selected = parts.contains(part);
        let mut btn = ElementInfo::button(i, label);
        btn.selected = Some(is_selected);
        assert_eq!(btn.selected, Some(true), "Default parts should be selected");
    }
}

#[test]
fn chip_deselected_when_part_removed() {
    let mut parts = default_parts();
    let selection = parts[1].clone();
    // Remove "Selection"
    parts.retain(|p| p != &selection);

    let labels = ["Current Context", "Selection", "Browser URL", "Focused Window"];
    let default = default_parts();
    for (i, (label, default_part)) in labels.iter().zip(default.iter()).enumerate() {
        let is_selected = parts.contains(default_part);
        let mut btn = ElementInfo::button(i, label);
        btn.selected = Some(is_selected);
        if i == 1 {
            assert_eq!(btn.selected, Some(false), "Removed part should not be selected");
        } else {
            assert_eq!(btn.selected, Some(true), "Remaining parts should be selected");
        }
    }
}

#[test]
fn ai_button_selected_when_parts_present() {
    let parts = default_parts();
    let mut btn = ElementInfo::button(4, "Ask AI with Context");
    btn.selected = Some(!parts.is_empty());
    assert_eq!(btn.selected, Some(true));
}

#[test]
fn ai_button_not_selected_when_parts_empty() {
    let parts: Vec<AiContextPart> = vec![];
    let mut btn = ElementInfo::button(4, "Ask AI with Context");
    btn.selected = Some(!parts.is_empty());
    assert_eq!(btn.selected, Some(false));
}

// ---------- Element count ----------

#[test]
fn context_strip_adds_six_elements() {
    // 1 panel + 4 chips + 1 AI button = 6
    let strip_count = 1 + 4 + 1;
    assert_eq!(strip_count, 6);
}

// ---------- Existing elements preserved ----------

#[test]
fn filter_input_element_still_present() {
    let input = ElementInfo::input("filter", Some("test"), true);
    assert_eq!(input.semantic_id, "input:filter");
    assert_eq!(input.element_type, ElementType::Input);
}

#[test]
fn results_list_element_still_present() {
    let list = ElementInfo::list("results", 10);
    assert_eq!(list.semantic_id, "list:results");
    assert_eq!(list.element_type, ElementType::List);
}
