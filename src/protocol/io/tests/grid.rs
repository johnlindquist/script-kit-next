use super::*;

// ============================================================
// Debug Grid Message Tests
// ============================================================

#[test]
fn test_show_grid_default_options() {
    let json = r#"{"type":"showGrid"}"#;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::ShowGrid { options }) => {
            assert_eq!(options.grid_size, 8); // default
            assert!(!options.show_bounds);
            assert!(!options.show_box_model);
            assert!(!options.show_alignment_guides);
        }
        other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
    }
}

#[test]
fn test_grid_options_default_matches_serde_default() {
    use crate::protocol::types::GridOptions;

    // GridOptions::default() should match deserializing an empty showGrid message
    let rust_default = GridOptions::default();

    // Deserialize from JSON with no fields (just the type)
    let json = r#"{"type":"showGrid"}"#;
    let serde_default = match parse_message_graceful(json) {
        ParseResult::Ok(Message::ShowGrid { options }) => options,
        other => panic!("Expected ShowGrid, got {:?}", other),
    };

    // Both should have grid_size = 8, not 0
    assert_eq!(
        rust_default.grid_size, 8,
        "Rust default grid_size should be 8"
    );
    assert_eq!(
        serde_default.grid_size, 8,
        "Serde default grid_size should be 8"
    );
    assert_eq!(
        rust_default.grid_size, serde_default.grid_size,
        "Rust Default and serde default must match for grid_size"
    );

    // Verify all other fields match too
    assert_eq!(rust_default.show_bounds, serde_default.show_bounds);
    assert_eq!(rust_default.show_box_model, serde_default.show_box_model);
    assert_eq!(
        rust_default.show_alignment_guides,
        serde_default.show_alignment_guides
    );
    assert_eq!(rust_default.show_dimensions, serde_default.show_dimensions);
    assert_eq!(rust_default.color_scheme, serde_default.color_scheme);
}

#[test]
fn test_show_grid_with_options() {
    let json = r#"{"type":"showGrid","gridSize":16,"showBounds":true,"showBoxModel":true}"#;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::ShowGrid { options }) => {
            assert_eq!(options.grid_size, 16);
            assert!(options.show_bounds);
            assert!(options.show_box_model);
            assert!(!options.show_alignment_guides);
        }
        other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
    }
}

#[test]
fn test_show_grid_with_depth_preset() {
    use crate::protocol::types::GridDepthOption;

    let json = r#"{"type":"showGrid","depth":"all"}"#;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::ShowGrid { options }) => match options.depth {
            GridDepthOption::Preset(s) => assert_eq!(s, "all"),
            _ => panic!("Expected Preset depth"),
        },
        other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
    }
}

#[test]
fn test_show_grid_with_depth_components() {
    use crate::protocol::types::GridDepthOption;

    let json = r#"{"type":"showGrid","depth":["header","list","footer"]}"#;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::ShowGrid { options }) => match options.depth {
            GridDepthOption::Components(components) => {
                assert_eq!(components.len(), 3);
                assert!(components.contains(&"header".to_string()));
                assert!(components.contains(&"list".to_string()));
                assert!(components.contains(&"footer".to_string()));
            }
            _ => panic!("Expected Components depth"),
        },
        other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
    }
}

#[test]
fn test_show_grid_with_color_scheme() {
    let json =
        r##"{"type":"showGrid","colorScheme":{"gridLines":"#FF0000AA","promptBounds":"#00FF00"}}"##;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::ShowGrid { options }) => {
            let colors = options.color_scheme.expect("Expected color scheme");
            assert_eq!(colors.grid_lines, Some("#FF0000AA".to_string()));
            assert_eq!(colors.prompt_bounds, Some("#00FF00".to_string()));
            assert!(colors.input_bounds.is_none());
        }
        other => panic!("Expected ParseResult::Ok with ShowGrid, got {:?}", other),
    }
}

#[test]
fn test_hide_grid() {
    let json = r#"{"type":"hideGrid"}"#;
    match parse_message_graceful(json) {
        ParseResult::Ok(Message::HideGrid) => {}
        other => panic!("Expected ParseResult::Ok with HideGrid, got {:?}", other),
    }
}

#[test]
fn test_show_grid_roundtrip() {
    use crate::protocol::types::{GridColorScheme, GridDepthOption, GridOptions};

    let options = GridOptions {
        grid_size: 16,
        show_bounds: true,
        show_box_model: false,
        show_alignment_guides: true,
        show_dimensions: true,
        depth: GridDepthOption::Components(vec!["header".to_string(), "list".to_string()]),
        color_scheme: Some(GridColorScheme {
            grid_lines: Some("#FF0000".to_string()),
            prompt_bounds: None,
            input_bounds: None,
            button_bounds: None,
            list_bounds: None,
            padding_fill: Some("#00FF0040".to_string()),
            margin_fill: None,
            alignment_guide: None,
        }),
    };

    let msg = Message::show_grid_with_options(options);
    let serialized = serde_json::to_string(&msg).expect("Failed to serialize");

    // Verify the serialized JSON has the expected type
    assert!(serialized.contains(r##""type":"showGrid""##));
    assert!(serialized.contains(r##""gridSize":16"##));
    assert!(serialized.contains(r##""showBounds":true"##));
    assert!(serialized.contains(r##""showAlignmentGuides":true"##));

    // Deserialize back and verify
    let deserialized: Message = serde_json::from_str(&serialized).expect("Failed to deserialize");
    match deserialized {
        Message::ShowGrid { options } => {
            assert_eq!(options.grid_size, 16);
            assert!(options.show_bounds);
            assert!(!options.show_box_model);
            assert!(options.show_alignment_guides);
        }
        _ => panic!("Expected ShowGrid message"),
    }
}
