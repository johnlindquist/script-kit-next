    #[test]
    fn test_tools_call_kit_state() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(3),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "kit/state",
                "arguments": {}
            }),
        };

        let response = handle_request(request);

        assert!(response.error.is_none(), "kit/state call should succeed");
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert!(result.get("content").is_some());

        // Verify the content is valid JSON with state fields
        let content = result.get("content").and_then(|c| c.as_array());
        assert!(content.is_some());

        let content = content.unwrap();
        assert!(!content.is_empty());

        let text = content[0].get("text").and_then(|t| t.as_str());
        assert!(text.is_some());

        // Should be parseable as AppState JSON
        let state: Result<serde_json::Value, _> = serde_json::from_str(text.unwrap());
        assert!(state.is_ok(), "kit/state should return valid JSON");

        let state = state.unwrap();
        assert!(state.get("visible").is_some());
        assert!(state.get("focused").is_some());
    }
    #[test]
    fn test_tools_call_unknown_kit_tool() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(4),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "kit/unknown",
                "arguments": {}
            }),
        };

        let response = handle_request(request);

        // Should succeed but with isError flag in result
        assert!(
            response.error.is_none(),
            "Should return result, not protocol error"
        );
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        // isError should be true for unknown kit tools
        assert_eq!(result.get("isError").and_then(|e| e.as_bool()), Some(true));
    }
    #[test]
    fn test_tools_call_non_kit_tool_not_found() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(5),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "scripts/run",
                "arguments": {}
            }),
        };

        let response = handle_request(request);

        // scripts/* tools now go through script handler which returns isError: true
        // instead of a protocol error, because it's a valid namespace
        assert!(
            response.error.is_none(),
            "Should return result, not protocol error"
        );
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert_eq!(result.get("isError").and_then(|e| e.as_bool()), Some(true));
    }
    #[test]
    fn test_tools_call_unknown_namespace_returns_error() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(5),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "unknown/tool",
                "arguments": {}
            }),
        };

        let response = handle_request(request);

        // Unknown namespace should return method not found error
        assert!(response.error.is_some());
        assert_eq!(
            response.error.as_ref().unwrap().code,
            error_codes::METHOD_NOT_FOUND
        );
    }
    // =======================================================
    // Script Tools Integration Tests
    // =======================================================

    mod script_tools_tests {
        use super::*;
        use crate::schema_parser::{FieldDef, FieldType, Schema};
        use std::collections::HashMap;
        use std::path::PathBuf;

        /// Helper to create a test script with schema
        fn test_script_with_schema(name: &str, description: Option<&str>) -> Script {
            let mut input = HashMap::new();
            input.insert(
                "title".to_string(),
                FieldDef {
                    field_type: FieldType::String,
                    required: true,
                    description: Some("The title".to_string()),
                    ..Default::default()
                },
            );
            let schema = Schema {
                input,
                output: HashMap::new(),
            };

            Script {
                name: name.to_string(),
                path: PathBuf::from(format!(
                    "/test/{}.ts",
                    name.to_lowercase().replace(' ', "-")
                )),
                extension: "ts".to_string(),
                description: description.map(|s| s.to_string()),
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: Some(schema),
                kit_name: None,
            }
        }

        #[test]
        fn test_tools_list_includes_script_tools() {
            let scripts = wrap_scripts(vec![
                test_script_with_schema("Create Note", Some("Creates a new note")),
                test_script_with_schema("Git Commit", Some("Commits changes")),
            ]);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(1),
                method: "tools/list".to_string(),
                params: serde_json::json!({}),
            };

            let response = handle_request_with_scripts(request, &scripts);

            assert!(response.result.is_some());
            let result = response.result.unwrap();
            let tools = result.get("tools").and_then(|v| v.as_array()).unwrap();

            // Collect tool names
            let tool_names: Vec<&str> = tools
                .iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .collect();

            // Should include kit/* tools
            assert!(tool_names.contains(&"kit/show"));
            assert!(tool_names.contains(&"kit/hide"));

            // Should include scripts/* tools
            assert!(
                tool_names.contains(&"scripts/create-note"),
                "Should include scripts/create-note"
            );
            assert!(
                tool_names.contains(&"scripts/git-commit"),
                "Should include scripts/git-commit"
            );
        }

        #[test]
        fn test_tools_list_script_tool_has_correct_schema() {
            let scripts = wrap_scripts(vec![test_script_with_schema(
                "Test Script",
                Some("Test description"),
            )]);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(1),
                method: "tools/list".to_string(),
                params: serde_json::json!({}),
            };

            let response = handle_request_with_scripts(request, &scripts);
            let result = response.result.unwrap();
            let tools = result.get("tools").and_then(|v| v.as_array()).unwrap();

            // Find the script tool
            let script_tool = tools
                .iter()
                .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("scripts/test-script"));

            assert!(script_tool.is_some(), "Script tool should be in list");
            let tool = script_tool.unwrap();

            // Check description
            assert_eq!(
                tool.get("description").and_then(|d| d.as_str()),
                Some("Test description")
            );

            // Check inputSchema
            let input_schema = tool.get("inputSchema");
            assert!(input_schema.is_some());
            assert_eq!(input_schema.unwrap()["type"], "object");
            assert!(input_schema.unwrap()["properties"]["title"].is_object());
        }

        #[test]
        fn test_tools_call_script_tool() {
            let scripts = wrap_scripts(vec![test_script_with_schema(
                "Create Note",
                Some("Creates notes"),
            )]);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(1),
                method: "tools/call".to_string(),
                params: serde_json::json!({
                    "name": "scripts/create-note",
                    "arguments": {"title": "My Note"}
                }),
            };

            let response = handle_request_with_scripts(request, &scripts);

            // Should succeed (return result, not error)
            assert!(response.error.is_none(), "Script tool call should succeed");
            assert!(response.result.is_some());

            let result = response.result.unwrap();
            // Should have content
            assert!(result.get("content").is_some());
        }

        #[test]
        fn test_tools_call_unknown_script_tool() {
            let scripts = wrap_scripts(vec![test_script_with_schema("Create Note", None)]);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(1),
                method: "tools/call".to_string(),
                params: serde_json::json!({
                    "name": "scripts/unknown-script",
                    "arguments": {}
                }),
            };

            let response = handle_request_with_scripts(request, &scripts);

            // Should succeed but with isError flag
            assert!(response.error.is_none());
            let result = response.result.unwrap();
            assert_eq!(result.get("isError").and_then(|e| e.as_bool()), Some(true));
        }

        #[test]
        fn test_tools_list_empty_scripts() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(1),
                method: "tools/list".to_string(),
                params: serde_json::json!({}),
            };

            let response = handle_request_with_scripts(request, &[]);

            assert!(response.result.is_some());
            let result = response.result.unwrap();
            let tools = result.get("tools").and_then(|v| v.as_array()).unwrap();

            // Should still have kit/* tools
            let tool_names: Vec<&str> = tools
                .iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .collect();

            assert!(tool_names.contains(&"kit/show"));
            assert!(tool_names.contains(&"kit/hide"));
            assert!(tool_names.contains(&"kit/state"));

            // Should NOT have any scripts/* tools
            let script_tools: Vec<&&str> = tool_names
                .iter()
                .filter(|n| n.starts_with("scripts/"))
                .collect();
            assert!(
                script_tools.is_empty(),
                "No script tools when scripts list is empty"
            );
        }

        #[test]
        fn test_scripts_without_schema_not_in_tools_list() {
            // Script without schema
            let script_no_schema = Script {
                name: "Simple Script".to_string(),
                path: PathBuf::from("/test/simple-script.ts"),
                extension: "ts".to_string(),
                description: Some("No schema".to_string()),
                icon: None,
                alias: None,
                shortcut: None,
                typed_metadata: None,
                schema: None, // No schema!
                kit_name: None,
            };

            let scripts = wrap_scripts(vec![
                script_no_schema,
                test_script_with_schema("With Schema", Some("Has schema")),
            ]);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(1),
                method: "tools/list".to_string(),
                params: serde_json::json!({}),
            };

            let response = handle_request_with_scripts(request, &scripts);
            let result = response.result.unwrap();
            let tools = result.get("tools").and_then(|v| v.as_array()).unwrap();

            let tool_names: Vec<&str> = tools
                .iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .collect();

            // Should have the script with schema
            assert!(tool_names.contains(&"scripts/with-schema"));

            // Should NOT have the script without schema
            assert!(
                !tool_names.contains(&"scripts/simple-script"),
                "Script without schema should not be in tools list"
            );
        }
    }
