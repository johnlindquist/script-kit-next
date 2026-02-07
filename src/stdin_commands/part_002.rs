// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_read_stdin_line_bounded_skips_oversized_line_and_recovers() {
        let oversized_payload = "x".repeat(20_000);
        let input = format!(
            r#"{{"type":"setFilter","text":"{}"}}
{{"type":"show"}}
"#,
            oversized_payload
        );

        let mut reader = Cursor::new(input);
        let mut byte_buffer = Vec::new();

        let first = read_stdin_line_bounded(&mut reader, &mut byte_buffer, MAX_STDIN_COMMAND_BYTES)
            .expect("Expected bounded line reader to process input");
        match first {
            StdinLineRead::TooLong { raw_len, .. } => {
                assert!(raw_len > MAX_STDIN_COMMAND_BYTES);
            }
            _ => panic!("Expected first line to be marked too long"),
        }

        let second =
            read_stdin_line_bounded(&mut reader, &mut byte_buffer, MAX_STDIN_COMMAND_BYTES)
                .expect("Expected second line to be readable");
        match second {
            StdinLineRead::Line(line) => {
                assert_eq!(line.trim_end(), r#"{"type":"show"}"#);
            }
            _ => panic!("Expected second line to be a valid command"),
        }
    }

    #[test]
    fn test_external_command_run_deserialization() {
        let json = r#"{"type": "run", "path": "/path/to/script.ts"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::Run { path, request_id } => {
                assert_eq!(path, "/path/to/script.ts");
                assert!(request_id.is_none());
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_external_command_run_with_request_id() {
        let json = r#"{"type": "run", "path": "/path/to/script.ts", "requestId": "req-123"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::Run { path, request_id } => {
                assert_eq!(path, "/path/to/script.ts");
                assert_eq!(request_id, Some("req-123".to_string().into()));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_external_command_show_deserialization() {
        let json = r#"{"type": "show"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, ExternalCommand::Show { request_id: None }));
    }

    #[test]
    fn test_external_command_show_with_request_id() {
        let json = r#"{"type": "show", "requestId": "req-456"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::Show { request_id } => {
                assert_eq!(request_id, Some("req-456".to_string().into()));
            }
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_external_command_hide_deserialization() {
        let json = r#"{"type": "hide"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, ExternalCommand::Hide { request_id: None }));
    }

    #[test]
    fn test_external_command_set_filter_deserialization() {
        let json = r#"{"type": "setFilter", "text": "hello world"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::SetFilter { text, request_id } => {
                assert_eq!(text, "hello world");
                assert!(request_id.is_none());
            }
            _ => panic!("Expected SetFilter command"),
        }
    }

    #[test]
    fn test_external_command_set_filter_with_request_id() {
        let json = r#"{"type": "setFilter", "text": "hello", "requestId": "req-789"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::SetFilter { text, request_id } => {
                assert_eq!(text, "hello");
                assert_eq!(request_id, Some("req-789".to_string().into()));
            }
            _ => panic!("Expected SetFilter command"),
        }
    }

    #[test]
    fn test_external_command_trigger_builtin_deserialization() {
        let json = r#"{"type": "triggerBuiltin", "name": "clipboardHistory"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::TriggerBuiltin { name } => assert_eq!(name, "clipboardHistory"),
            _ => panic!("Expected TriggerBuiltin command"),
        }
    }

    #[test]
    fn test_external_command_simulate_key_deserialization() {
        let json = r#"{"type": "simulateKey", "key": "enter", "modifiers": ["cmd", "shift"]}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::SimulateKey { key, modifiers } => {
                assert_eq!(key, "enter");
                assert_eq!(modifiers, vec![KeyModifier::Cmd, KeyModifier::Shift]);
            }
            _ => panic!("Expected SimulateKey command"),
        }
    }

    #[test]
    fn test_external_command_simulate_key_no_modifiers() {
        let json = r#"{"type": "simulateKey", "key": "escape"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::SimulateKey { key, modifiers } => {
                assert_eq!(key, "escape");
                assert!(modifiers.is_empty());
            }
            _ => panic!("Expected SimulateKey command"),
        }
    }

    #[test]
    fn test_external_command_simulate_key_modifier_aliases() {
        let json = r#"{"type":"simulateKey","key":"k","modifiers":["meta","option","control"]}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::SimulateKey { modifiers, .. } => {
                assert_eq!(
                    modifiers,
                    vec![KeyModifier::Cmd, KeyModifier::Alt, KeyModifier::Ctrl]
                );
            }
            _ => panic!("Expected SimulateKey command"),
        }
    }

    #[test]
    fn test_external_command_simulate_key_unknown_modifier_rejected() {
        let json = r#"{"type":"simulateKey","key":"enter","modifiers":["capslock"]}"#;
        let result = serde_json::from_str::<ExternalCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_invalid_json_fails() {
        let json = r#"{"type": "unknown"}"#;
        let result = serde_json::from_str::<ExternalCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_rejects_unknown_fields() {
        let json = r#"{"type":"show","unexpected":"field"}"#;
        let result = serde_json::from_str::<ExternalCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_missing_required_field_fails() {
        // Run command requires path field
        let json = r#"{"type": "run"}"#;
        let result = serde_json::from_str::<ExternalCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_external_command_clone() {
        let cmd = ExternalCommand::Run {
            path: "/test".to_string(),
            request_id: None,
        };
        let cloned = cmd.clone();
        match cloned {
            ExternalCommand::Run { path, .. } => assert_eq!(path, "/test"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_external_command_debug() {
        let cmd = ExternalCommand::Show { request_id: None };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Show"));
    }

    #[test]
    fn test_external_command_request_id_accessor() {
        let cmd = ExternalCommand::SetFilter {
            text: "hello".to_string(),
            request_id: Some("req-42".to_string().into()),
        };
        assert_eq!(cmd.request_id(), Some("req-42"));
    }

    #[test]
    fn test_external_command_type_accessor() {
        let cmd = ExternalCommand::Show { request_id: None };
        assert_eq!(cmd.command_type(), "show");
    }

    #[test]
    fn test_external_command_open_notes_deserialization() {
        let json = r#"{"type": "openNotes"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, ExternalCommand::OpenNotes));
    }

    #[test]
    fn test_external_command_open_ai_deserialization() {
        let json = r#"{"type": "openAi"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, ExternalCommand::OpenAi));
    }

    #[test]
    fn test_external_command_open_ai_with_mock_data_deserialization() {
        let json = r#"{"type": "openAiWithMockData"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, ExternalCommand::OpenAiWithMockData));
    }

    #[test]
    fn test_external_command_capture_window_deserialization() {
        let json =
            r#"{"type": "captureWindow", "title": "Script Kit AI", "path": "/tmp/screenshot.png"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::CaptureWindow { title, path } => {
                assert_eq!(title, "Script Kit AI");
                assert_eq!(path, "/tmp/screenshot.png");
            }
            _ => panic!("Expected CaptureWindow command"),
        }
    }

    #[test]
    fn test_external_command_show_grid_defaults() {
        let json = r#"{"type": "showGrid"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::ShowGrid {
                grid_size,
                show_bounds,
                show_box_model,
                show_alignment_guides,
                show_dimensions,
                depth,
            } => {
                assert_eq!(grid_size, 8); // default
                assert!(!show_bounds); // default false
                assert!(!show_box_model); // default false
                assert!(!show_alignment_guides); // default false
                assert!(!show_dimensions); // default false
                assert!(matches!(depth, GridDepthOption::Preset(_))); // default
            }
            _ => panic!("Expected ShowGrid command"),
        }
    }

    #[test]
    fn test_external_command_show_grid_with_options() {
        let json = r#"{"type": "showGrid", "gridSize": 16, "showBounds": true, "showBoxModel": true, "showAlignmentGuides": true, "showDimensions": true, "depth": "all"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::ShowGrid {
                grid_size,
                show_bounds,
                show_box_model,
                show_alignment_guides,
                show_dimensions,
                depth,
            } => {
                assert_eq!(grid_size, 16);
                assert!(show_bounds);
                assert!(show_box_model);
                assert!(show_alignment_guides);
                assert!(show_dimensions);
                match depth {
                    GridDepthOption::Preset(s) => assert_eq!(s, "all"),
                    _ => panic!("Expected Preset depth"),
                }
            }
            _ => panic!("Expected ShowGrid command"),
        }
    }

    #[test]
    fn test_external_command_show_grid_with_components() {
        let json = r#"{"type": "showGrid", "depth": ["header", "footer"]}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::ShowGrid { depth, .. } => match depth {
                GridDepthOption::Components(components) => {
                    assert_eq!(components, vec!["header", "footer"]);
                }
                _ => panic!("Expected Components depth"),
            },
            _ => panic!("Expected ShowGrid command"),
        }
    }

    #[test]
    fn test_external_command_hide_grid_deserialization() {
        let json = r#"{"type": "hideGrid"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, ExternalCommand::HideGrid));
    }

    #[test]
    fn test_external_command_execute_fallback_deserialization() {
        let json =
            r#"{"type": "executeFallback", "fallbackId": "search-google", "input": "hello world"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::ExecuteFallback { fallback_id, input } => {
                assert_eq!(fallback_id, "search-google");
                assert_eq!(input, "hello world");
            }
            _ => panic!("Expected ExecuteFallback command"),
        }
    }

    #[test]
    fn test_external_command_execute_fallback_copy() {
        let json = r#"{"type": "executeFallback", "fallbackId": "copy-to-clipboard", "input": "test text"}"#;
        let cmd: ExternalCommand = serde_json::from_str(json).unwrap();
        match cmd {
            ExternalCommand::ExecuteFallback { fallback_id, input } => {
                assert_eq!(fallback_id, "copy-to-clipboard");
                assert_eq!(input, "test text");
            }
            _ => panic!("Expected ExecuteFallback command"),
        }
    }

    #[test]
    fn test_validate_capture_window_output_path_allows_dot_test_screenshots() {
        let temp = TempDir::new().expect("create temp dir");
        let cwd = std::fs::canonicalize(temp.path()).expect("canonicalize temp dir");
        let kit_root = cwd.join("kit-root");
        std::fs::create_dir_all(&kit_root).expect("create kit root");

        let resolved = validate_capture_window_output_path_with_roots(
            ".test-screenshots/shot.png",
            &cwd,
            &kit_root,
        )
        .expect("path should be accepted");

        assert_eq!(resolved, cwd.join(".test-screenshots/shot.png"));
    }

    #[test]
    fn test_validate_capture_window_output_path_rejects_traversal() {
        let temp = TempDir::new().expect("create temp dir");
        let cwd = temp.path();
        let kit_root = cwd.join("kit-root");
        std::fs::create_dir_all(&kit_root).expect("create kit root");

        let error = validate_capture_window_output_path_with_roots(
            ".test-screenshots/../escape.png",
            cwd,
            &kit_root,
        )
        .expect_err("path traversal should be rejected");

        assert!(matches!(
            error,
            CaptureWindowPathPolicyError::PathOutsideAllowedRoots { .. }
        ));
    }

    #[test]
    fn test_validate_capture_window_output_path_rejects_symlink_parent() {
        let temp = TempDir::new().expect("create temp dir");
        let cwd = temp.path();
        let kit_root = cwd.join("kit-root");
        std::fs::create_dir_all(&kit_root).expect("create kit root");

        let screenshots_root = cwd.join(".test-screenshots");
        std::fs::create_dir_all(&screenshots_root).expect("create screenshots root");

        let outside = cwd.join("outside");
        std::fs::create_dir_all(&outside).expect("create outside dir");

        let symlink_path = screenshots_root.join("linked");
        create_symlink(&outside, &symlink_path);

        let error = validate_capture_window_output_path_with_roots(
            ".test-screenshots/linked/shot.png",
            cwd,
            &kit_root,
        )
        .expect_err("symlink target should be rejected");

        assert!(matches!(
            error,
            CaptureWindowPathPolicyError::SymlinkInPath { .. }
        ));
    }

    #[test]
    fn test_validate_capture_window_output_path_allows_scriptkit_screenshots_root() {
        let temp = TempDir::new().expect("create temp dir");
        let cwd = std::fs::canonicalize(temp.path()).expect("canonicalize temp dir");
        let kit_root = cwd.join("kit-root");
        let screenshots_root = kit_root.join("screenshots");
        std::fs::create_dir_all(&screenshots_root).expect("create screenshots root");

        let target = screenshots_root.join("shot.png");
        let resolved = validate_capture_window_output_path_with_roots(
            target.to_string_lossy().as_ref(),
            &cwd,
            &kit_root,
        )
        .expect("path should be accepted");

        assert_eq!(resolved, target);
    }

    #[cfg(unix)]
    fn create_symlink(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("create symlink");
    }

    #[cfg(windows)]
    fn create_symlink(target: &Path, link: &Path) {
        std::os::windows::fs::symlink_dir(target, link).expect("create symlink");
    }
}
