//! MCP JSON-RPC 2.0 Protocol Handler
//!
//! Implements the JSON-RPC 2.0 protocol for MCP (Model Context Protocol).
//! Handles request parsing, method routing, and response generation.
//!
//! JSON-RPC 2.0 format:
//! - Request: {"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
//! - Success: {"jsonrpc":"2.0","id":1,"result":{"tools":[]}}
//! - Error: {"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}

// Allow from_str name - we're not implementing FromStr trait as this returns Option, not Result
#![allow(clippy::should_implement_trait)]
// Allow large error variant - JsonRpcResponse needs to carry full error info for JSON-RPC spec
#![allow(clippy::result_large_err)]
// Allow dead code - this module provides complete MCP API surface; some methods for future use
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::mcp_kit_tools;
use crate::mcp_resources;
use crate::mcp_script_tools;
use crate::scripts::Script;
use crate::scripts::Scriptlet;

/// JSON-RPC 2.0 version string
pub const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC 2.0 standard error codes
pub mod error_codes {
    /// Invalid JSON was received
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s)
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// Must be "2.0"
    pub jsonrpc: String,
    /// Request identifier (can be string, number, or null)
    pub id: Value,
    /// Method name to invoke
    pub method: String,
    /// Optional parameters
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// Must be "2.0"
    pub jsonrpc: String,
    /// Request identifier (matches request)
    pub id: Value,
    /// Result on success (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error on failure (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP methods supported by this server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpMethod {
    /// Initialize the MCP session
    Initialize,
    /// List available tools
    ToolsList,
    /// Call a specific tool
    ToolsCall,
    /// List available resources
    ResourcesList,
    /// Read a specific resource
    ResourcesRead,
}

impl McpMethod {
    /// Parse method string to enum variant
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "initialize" => Some(Self::Initialize),
            "tools/list" => Some(Self::ToolsList),
            "tools/call" => Some(Self::ToolsCall),
            "resources/list" => Some(Self::ResourcesList),
            "resources/read" => Some(Self::ResourcesRead),
            _ => None,
        }
    }

    /// Get the method string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::ToolsList => "tools/list",
            Self::ToolsCall => "tools/call",
            Self::ResourcesList => "resources/list",
            Self::ResourcesRead => "resources/read",
        }
    }
}

impl JsonRpcResponse {
    /// Create a success response
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    /// Create an error response with additional data
    pub fn error_with_data(id: Value, code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: Some(data),
            }),
        }
    }
}

/// MCP server capabilities returned by initialize
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCapabilities {
    /// Server information
    pub server_info: ServerInfo,
    /// Supported capabilities
    pub capabilities: CapabilitySet,
}

/// Server identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Set of capabilities the server supports
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilitySet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
}

/// Tools capability settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resources capability settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourcesCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Parse a JSON string into a JsonRpcRequest
pub fn parse_request(json: &str) -> Result<JsonRpcRequest, JsonRpcResponse> {
    // Try to parse the JSON
    let value: Value = serde_json::from_str(json).map_err(|e| {
        JsonRpcResponse::error(
            Value::Null,
            error_codes::PARSE_ERROR,
            format!("Parse error: {}", e),
        )
    })?;

    // Validate jsonrpc version
    let jsonrpc = value
        .get("jsonrpc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            JsonRpcResponse::error(
                value.get("id").cloned().unwrap_or(Value::Null),
                error_codes::INVALID_REQUEST,
                "Missing or invalid 'jsonrpc' field",
            )
        })?;

    if jsonrpc != JSONRPC_VERSION {
        return Err(JsonRpcResponse::error(
            value.get("id").cloned().unwrap_or(Value::Null),
            error_codes::INVALID_REQUEST,
            format!(
                "Invalid jsonrpc version: expected '{}', got '{}'",
                JSONRPC_VERSION, jsonrpc
            ),
        ));
    }

    // Validate required fields
    let id = value.get("id").cloned().unwrap_or(Value::Null);

    let method = value
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            JsonRpcResponse::error(
                id.clone(),
                error_codes::INVALID_REQUEST,
                "Missing 'method' field",
            )
        })?;

    let params = value
        .get("params")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    Ok(JsonRpcRequest {
        jsonrpc: JSONRPC_VERSION.to_string(),
        id,
        method: method.to_string(),
        params,
    })
}

/// Handle an MCP JSON-RPC request and return a response
pub fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    // Use empty scripts list for stateless handler
    handle_request_with_scripts(request, &[])
}

/// Handle an MCP JSON-RPC request with script context
/// This allows script tools to be dynamically included based on loaded scripts
pub fn handle_request_with_scripts(
    request: JsonRpcRequest,
    scripts: &[std::sync::Arc<Script>],
) -> JsonRpcResponse {
    // Use empty scriptlets list for backwards compatibility
    handle_request_with_context(request, scripts, &[], None)
}

/// Handle an MCP JSON-RPC request with full context
/// This allows script tools and resources to be dynamically included
///
/// H1 Optimization: Accepts Arc<Script> and Arc<Scriptlet> to work with Arc storage.
pub fn handle_request_with_context(
    request: JsonRpcRequest,
    scripts: &[std::sync::Arc<Script>],
    scriptlets: &[std::sync::Arc<Scriptlet>],
    app_state: Option<&mcp_resources::AppStateResource>,
) -> JsonRpcResponse {
    // Check for valid jsonrpc version
    if request.jsonrpc != JSONRPC_VERSION {
        return JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_REQUEST,
            format!("Invalid jsonrpc version: {}", request.jsonrpc),
        );
    }

    // Route to appropriate handler based on method
    match McpMethod::from_str(&request.method) {
        Some(McpMethod::Initialize) => handle_initialize(request),
        Some(McpMethod::ToolsList) => handle_tools_list_with_scripts(request, scripts),
        Some(McpMethod::ToolsCall) => handle_tools_call_with_scripts(request, scripts),
        Some(McpMethod::ResourcesList) => handle_resources_list(request),
        Some(McpMethod::ResourcesRead) => {
            handle_resources_read_with_context(request, scripts, scriptlets, app_state)
        }
        None => JsonRpcResponse::error(
            request.id,
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: {}", request.method),
        ),
    }
}

/// Handle initialize request
fn handle_initialize(request: JsonRpcRequest) -> JsonRpcResponse {
    let capabilities = McpCapabilities {
        server_info: ServerInfo {
            name: "script-kit".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        capabilities: CapabilitySet {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(false),
                list_changed: Some(true),
            }),
        },
    };

    JsonRpcResponse::success(
        request.id,
        serde_json::to_value(capabilities).unwrap_or(Value::Null),
    )
}

/// Handle tools/list request (no script context)
#[allow(dead_code)]
fn handle_tools_list(request: JsonRpcRequest) -> JsonRpcResponse {
    // Use empty scripts list for stateless handler
    handle_tools_list_with_scripts(request, &[])
}

/// Handle tools/list request with script context
/// This allows including dynamically loaded script tools
///
/// H1 Optimization: Accepts Arc<Script> to work with Arc storage.
pub fn handle_tools_list_with_scripts(
    request: JsonRpcRequest,
    scripts: &[std::sync::Arc<Script>],
) -> JsonRpcResponse {
    // Get kit/* namespace tools
    let mut all_tools = mcp_kit_tools::get_kit_tool_definitions();

    // Get scripts/* namespace tools (only scripts with schema.input)
    let script_tools = mcp_script_tools::get_script_tool_definitions(scripts);
    all_tools.extend(script_tools);

    // Convert to JSON value
    let tools_json = serde_json::to_value(&all_tools).unwrap_or(serde_json::json!([]));

    JsonRpcResponse::success(
        request.id,
        serde_json::json!({
            "tools": tools_json
        }),
    )
}

/// Handle tools/call request (no script context)
#[allow(dead_code)]
fn handle_tools_call(request: JsonRpcRequest) -> JsonRpcResponse {
    // Use empty scripts list for stateless handler
    handle_tools_call_with_scripts(request, &[])
}

/// Handle tools/call request with script context
/// This allows handling scripts/* namespace tool calls
///
/// H1 Optimization: Accepts Arc<Script> to work with Arc storage.
pub fn handle_tools_call_with_scripts(
    request: JsonRpcRequest,
    scripts: &[std::sync::Arc<Script>],
) -> JsonRpcResponse {
    // Validate params
    let params = request.params.as_object();
    if params.is_none() {
        return JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_PARAMS,
            "Invalid params: expected object",
        );
    }

    let params = params.unwrap();
    let tool_name = params.get("name").and_then(|v| v.as_str());

    if tool_name.is_none() {
        return JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_PARAMS,
            "Missing required parameter: name",
        );
    }

    let tool_name = tool_name.unwrap();
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    // Route kit/* namespace tools
    if mcp_kit_tools::is_kit_tool(tool_name) {
        let result = mcp_kit_tools::handle_kit_tool_call(tool_name, &arguments);
        return JsonRpcResponse::success(
            request.id,
            serde_json::to_value(result).unwrap_or(serde_json::json!({})),
        );
    }

    // Route scripts/* namespace tools
    if mcp_script_tools::is_script_tool(tool_name) {
        let result = mcp_script_tools::handle_script_tool_call(scripts, tool_name, &arguments);
        return JsonRpcResponse::success(
            request.id,
            serde_json::to_value(result).unwrap_or(serde_json::json!({})),
        );
    }

    // Tool not found in any namespace
    JsonRpcResponse::error(
        request.id,
        error_codes::METHOD_NOT_FOUND,
        format!("Tool not found: {}", tool_name),
    )
}

/// Handle resources/list request
fn handle_resources_list(request: JsonRpcRequest) -> JsonRpcResponse {
    let resources = mcp_resources::get_resource_definitions();
    JsonRpcResponse::success(
        request.id,
        mcp_resources::resource_list_to_value(&resources),
    )
}

/// Handle resources/read request (stateless - for backwards compatibility)
#[allow(dead_code)]
fn handle_resources_read(request: JsonRpcRequest) -> JsonRpcResponse {
    handle_resources_read_with_context(request, &[], &[], None)
}

/// Handle resources/read request with full context
fn handle_resources_read_with_context(
    request: JsonRpcRequest,
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    app_state: Option<&mcp_resources::AppStateResource>,
) -> JsonRpcResponse {
    // Validate params
    let params = request.params.as_object();
    if params.is_none() {
        return JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_PARAMS,
            "Invalid params: expected object",
        );
    }

    let params = params.unwrap();
    let uri = params.get("uri").and_then(|v| v.as_str());

    if uri.is_none() {
        return JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_PARAMS,
            "Missing required parameter: uri",
        );
    }

    let uri = uri.unwrap();

    // Read the resource
    match mcp_resources::read_resource(uri, scripts, scriptlets, app_state) {
        Ok(content) => JsonRpcResponse::success(
            request.id,
            mcp_resources::resource_content_to_value(content),
        ),
        Err(err) => JsonRpcResponse::error(request.id, error_codes::METHOD_NOT_FOUND, err),
    }
}

#[cfg(test)]
mod tests {
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
}
