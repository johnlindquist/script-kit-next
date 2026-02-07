    use super::*;
    use std::sync::Arc;
    /// Helper to wrap Vec<Script> into Vec<Arc<Script>> for tests
    fn wrap_scripts(scripts: Vec<Script>) -> Vec<Arc<Script>> {
        scripts.into_iter().map(Arc::new).collect()
    }
    /// Helper to wrap Vec<Scriptlet> into Vec<Arc<Scriptlet>> for tests
    fn wrap_scriptlets(scriptlets: Vec<Scriptlet>) -> Vec<Arc<Scriptlet>> {
        scriptlets.into_iter().map(Arc::new).collect()
    }
    // =======================================================
    // TDD Tests - Written FIRST per spec requirements
    // =======================================================

    #[test]
    fn test_parse_valid_jsonrpc_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        let result = parse_request(json);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, serde_json::json!(1));
        assert_eq!(request.method, "tools/list");
        assert_eq!(request.params, serde_json::json!({}));
    }
    #[test]
    fn test_parse_invalid_jsonrpc_returns_error() {
        // Test 1: Invalid JSON
        let json = r#"{"jsonrpc":"2.0", invalid}"#;
        let result = parse_request(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.error.as_ref().unwrap().code, error_codes::PARSE_ERROR);

        // Test 2: Missing jsonrpc field
        let json = r#"{"id":1,"method":"test"}"#;
        let result = parse_request(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.error.as_ref().unwrap().code,
            error_codes::INVALID_REQUEST
        );

        // Test 3: Wrong jsonrpc version
        let json = r#"{"jsonrpc":"1.0","id":1,"method":"test"}"#;
        let result = parse_request(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.error.as_ref().unwrap().code,
            error_codes::INVALID_REQUEST
        );

        // Test 4: Missing method field
        let json = r#"{"jsonrpc":"2.0","id":1}"#;
        let result = parse_request(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.error.as_ref().unwrap().code,
            error_codes::INVALID_REQUEST
        );
    }
    #[test]
    fn test_method_not_found_error() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "unknown/method".to_string(),
            params: serde_json::json!({}),
        };

        let response = handle_request(request);

        assert!(response.error.is_some());
        assert!(response.result.is_none());
        let err = response.error.unwrap();
        assert_eq!(err.code, error_codes::METHOD_NOT_FOUND);
        assert!(err.message.contains("Method not found"));
        assert!(err.message.contains("unknown/method"));
    }
    #[test]
    fn test_initialize_returns_capabilities() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "initialize".to_string(),
            params: serde_json::json!({}),
        };

        let response = handle_request(request);

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();

        // Check serverInfo
        assert!(result.get("serverInfo").is_some());
        let server_info = result.get("serverInfo").unwrap();
        assert_eq!(
            server_info.get("name").and_then(|v| v.as_str()),
            Some("script-kit")
        );
        assert!(server_info.get("version").is_some());

        // Check capabilities
        assert!(result.get("capabilities").is_some());
        let caps = result.get("capabilities").unwrap();
        assert!(caps.get("tools").is_some());
        assert!(caps.get("resources").is_some());
    }
    #[test]
    fn test_tools_list_returns_kit_tools() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(42),
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
        };

        let response = handle_request(request);

        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, serde_json::json!(42));

        let result = response.result.unwrap();
        let tools = result.get("tools").and_then(|v| v.as_array());
        assert!(tools.is_some());

        let tools = tools.unwrap();
        // Should have at least the kit/* tools
        assert!(!tools.is_empty(), "tools/list should return kit tools");

        // Verify kit tools are present
        let tool_names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();

        assert!(tool_names.contains(&"kit/show"), "Should include kit/show");
        assert!(tool_names.contains(&"kit/hide"), "Should include kit/hide");
        assert!(
            tool_names.contains(&"kit/state"),
            "Should include kit/state"
        );
    }
    #[test]
    fn test_resources_list_returns_all_resources() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!("req-123"),
            method: "resources/list".to_string(),
            params: serde_json::json!({}),
        };

        let response = handle_request(request);

        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, serde_json::json!("req-123"));

        let result = response.result.unwrap();
        let resources = result.get("resources").and_then(|v| v.as_array());
        assert!(resources.is_some());

        let resources = resources.unwrap();
        assert_eq!(resources.len(), 3, "Should have 3 resources");

        // Verify expected resources are present
        let uris: Vec<&str> = resources
            .iter()
            .filter_map(|r| r.get("uri").and_then(|u| u.as_str()))
            .collect();

        assert!(uris.contains(&"kit://state"), "Should include kit://state");
        assert!(uris.contains(&"scripts://"), "Should include scripts://");
        assert!(
            uris.contains(&"scriptlets://"),
            "Should include scriptlets://"
        );
    }
    // =======================================================
    // Additional tests for completeness
    // =======================================================

    #[test]
    fn test_mcp_method_from_str() {
        assert_eq!(
            McpMethod::from_str("initialize"),
            Some(McpMethod::Initialize)
        );
        assert_eq!(
            McpMethod::from_str("tools/list"),
            Some(McpMethod::ToolsList)
        );
        assert_eq!(
            McpMethod::from_str("tools/call"),
            Some(McpMethod::ToolsCall)
        );
        assert_eq!(
            McpMethod::from_str("resources/list"),
            Some(McpMethod::ResourcesList)
        );
        assert_eq!(
            McpMethod::from_str("resources/read"),
            Some(McpMethod::ResourcesRead)
        );
        assert_eq!(McpMethod::from_str("unknown"), None);
    }
    #[test]
    fn test_mcp_method_as_str() {
        assert_eq!(McpMethod::Initialize.as_str(), "initialize");
        assert_eq!(McpMethod::ToolsList.as_str(), "tools/list");
        assert_eq!(McpMethod::ToolsCall.as_str(), "tools/call");
        assert_eq!(McpMethod::ResourcesList.as_str(), "resources/list");
        assert_eq!(McpMethod::ResourcesRead.as_str(), "resources/read");
    }
    #[test]
    fn test_jsonrpc_response_success() {
        let response =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"key": "value"}));

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, serde_json::json!(1));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }
    #[test]
    fn test_jsonrpc_response_error() {
        let response = JsonRpcResponse::error(
            serde_json::json!(1),
            error_codes::METHOD_NOT_FOUND,
            "Method not found",
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, serde_json::json!(1));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let err = response.error.unwrap();
        assert_eq!(err.code, error_codes::METHOD_NOT_FOUND);
        assert_eq!(err.message, "Method not found");
    }
    #[test]
    fn test_tools_call_requires_name_param() {
        // Missing name param
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/call".to_string(),
            params: serde_json::json!({}),
        };

        let response = handle_request(request);
        assert!(response.error.is_some());
        assert_eq!(
            response.error.as_ref().unwrap().code,
            error_codes::INVALID_PARAMS
        );
    }
    #[test]
    fn test_resources_read_requires_uri_param() {
        // Missing uri param
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "resources/read".to_string(),
            params: serde_json::json!({}),
        };

        let response = handle_request(request);
        assert!(response.error.is_some());
        assert_eq!(
            response.error.as_ref().unwrap().code,
            error_codes::INVALID_PARAMS
        );
    }
    #[test]
    fn test_parse_request_with_string_id() {
        let json = r#"{"jsonrpc":"2.0","id":"request-123","method":"initialize","params":{}}"#;
        let result = parse_request(json);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.id, serde_json::json!("request-123"));
    }
    #[test]
    fn test_parse_request_with_null_id() {
        // Notifications have null id (or id is omitted)
        let json = r#"{"jsonrpc":"2.0","id":null,"method":"initialize","params":{}}"#;
        let result = parse_request(json);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.id, Value::Null);
    }
    #[test]
    fn test_parse_request_without_params() {
        // params is optional
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let result = parse_request(json);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.params, serde_json::json!({}));
    }
    #[test]
    fn test_response_serialization() {
        let response =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"tools": []}));

        let json = serde_json::to_string(&response).unwrap();

        // Should not contain "error" field when it's None
        assert!(!json.contains("error"));
        assert!(json.contains("result"));
        assert!(json.contains("jsonrpc"));
        assert!(json.contains("2.0"));
    }
    #[test]
    fn test_error_response_serialization() {
        let response = JsonRpcResponse::error(
            serde_json::json!(1),
            error_codes::METHOD_NOT_FOUND,
            "Not found",
        );

        let json = serde_json::to_string(&response).unwrap();

        // Should not contain "result" field when it's None
        assert!(!json.contains("result"));
        assert!(json.contains("error"));
        assert!(json.contains("-32601"));
    }
    // =======================================================
    // Kit Tools Integration Tests
    // =======================================================

    #[test]
    fn test_tools_call_kit_show() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "kit/show",
                "arguments": {}
            }),
        };

        let response = handle_request(request);

        // Should succeed (not return an error)
        assert!(response.error.is_none(), "kit/show call should succeed");
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        // Should have content array
        assert!(result.get("content").is_some());
    }
    #[test]
    fn test_tools_call_kit_hide() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(2),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": "kit/hide",
                "arguments": {}
            }),
        };

        let response = handle_request(request);

        assert!(response.error.is_none(), "kit/hide call should succeed");
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert!(result.get("content").is_some());
    }
