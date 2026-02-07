    // =======================================================
    // MCP Resources Integration Tests
    // =======================================================

    mod resources_integration_tests {
        use super::*;
        use crate::scripts::Scriptlet;
        use std::path::PathBuf;

        /// Helper to create a test script
        fn test_script(name: &str, description: Option<&str>) -> Script {
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
                schema: None,
                kit_name: None,
            }
        }

        /// Helper to create a test scriptlet
        fn test_scriptlet(name: &str, tool: &str) -> Scriptlet {
            Scriptlet {
                name: name.to_string(),
                description: None,
                code: "echo test".to_string(),
                tool: tool.to_string(),
                shortcut: None,
                keyword: None,
                group: None,
                file_path: None,
                command: None,
                alias: None,
            }
        }

        #[test]
        fn test_resources_read_scripts() {
            let scripts = wrap_scripts(vec![
                test_script("Script One", Some("First script")),
                test_script("Script Two", None),
            ]);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(1),
                method: "resources/read".to_string(),
                params: serde_json::json!({"uri": "scripts://"}),
            };

            let response = handle_request_with_context(request, &scripts, &[], None);

            assert!(response.error.is_none(), "Should succeed");
            assert!(response.result.is_some());

            let result = response.result.unwrap();
            let contents = result.get("contents").and_then(|c| c.as_array());
            assert!(contents.is_some());

            let contents = contents.unwrap();
            assert_eq!(contents.len(), 1);

            let content = &contents[0];
            assert_eq!(
                content.get("uri").and_then(|u| u.as_str()),
                Some("scripts://")
            );

            // Parse the text as JSON
            let text = content.get("text").and_then(|t| t.as_str()).unwrap();
            let parsed: Vec<serde_json::Value> = serde_json::from_str(text).unwrap();
            assert_eq!(parsed.len(), 2);
        }

        #[test]
        fn test_resources_read_scriptlets() {
            let scriptlets = wrap_scriptlets(vec![
                test_scriptlet("Open URL", "open"),
                test_scriptlet("Paste", "paste"),
            ]);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(2),
                method: "resources/read".to_string(),
                params: serde_json::json!({"uri": "scriptlets://"}),
            };

            let response = handle_request_with_context(request, &[], &scriptlets, None);

            assert!(response.error.is_none(), "Should succeed");

            let result = response.result.unwrap();
            let contents = result.get("contents").and_then(|c| c.as_array()).unwrap();
            let text = contents[0].get("text").and_then(|t| t.as_str()).unwrap();
            let parsed: Vec<serde_json::Value> = serde_json::from_str(text).unwrap();
            assert_eq!(parsed.len(), 2);
        }

        #[test]
        fn test_resources_read_app_state() {
            let app_state = mcp_resources::AppStateResource {
                visible: true,
                focused: true,
                script_count: 5,
                scriptlet_count: 3,
                filter_text: Some("test".to_string()),
                selected_index: Some(2),
            };

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(3),
                method: "resources/read".to_string(),
                params: serde_json::json!({"uri": "kit://state"}),
            };

            let response = handle_request_with_context(request, &[], &[], Some(&app_state));

            assert!(response.error.is_none(), "Should succeed");

            let result = response.result.unwrap();
            let contents = result.get("contents").and_then(|c| c.as_array()).unwrap();
            let text = contents[0].get("text").and_then(|t| t.as_str()).unwrap();
            let parsed: mcp_resources::AppStateResource = serde_json::from_str(text).unwrap();

            assert!(parsed.visible);
            assert!(parsed.focused);
            assert_eq!(parsed.script_count, 5);
            assert_eq!(parsed.scriptlet_count, 3);
            assert_eq!(parsed.filter_text, Some("test".to_string()));
        }

        #[test]
        fn test_resources_read_unknown_uri() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::json!(4),
                method: "resources/read".to_string(),
                params: serde_json::json!({"uri": "unknown://resource"}),
            };

            let response = handle_request_with_context(request, &[], &[], None);

            assert!(
                response.error.is_some(),
                "Unknown resource should return error"
            );
            assert_eq!(
                response.error.as_ref().unwrap().code,
                error_codes::METHOD_NOT_FOUND
            );
            assert!(response
                .error
                .as_ref()
                .unwrap()
                .message
                .contains("Resource not found"));
        }

        #[test]
        fn test_resources_read_with_full_context() {
            let scripts = wrap_scripts(vec![test_script("Test Script", None)]);
            let scriptlets = wrap_scriptlets(vec![test_scriptlet("Test Snippet", "bash")]);
            let app_state = mcp_resources::AppStateResource {
                visible: true,
                focused: false,
                script_count: 1,
                scriptlet_count: 1,
                filter_text: None,
                selected_index: None,
            };

            // Test all three resources work with full context
            for uri in &["kit://state", "scripts://", "scriptlets://"] {
                let request = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::json!(uri),
                    method: "resources/read".to_string(),
                    params: serde_json::json!({"uri": uri}),
                };

                let response =
                    handle_request_with_context(request, &scripts, &scriptlets, Some(&app_state));

                assert!(response.error.is_none(), "Should succeed for {}", uri);
                assert!(response.result.is_some());
            }
        }
    }
