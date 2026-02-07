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
    let params = match request.params.as_object() {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(
                request.id,
                error_codes::INVALID_PARAMS,
                "Invalid params: expected object",
            );
        }
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => {
            return JsonRpcResponse::error(
                request.id,
                error_codes::INVALID_PARAMS,
                "Missing required parameter: name",
            );
        }
    };

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
    let params = match request.params.as_object() {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(
                request.id,
                error_codes::INVALID_PARAMS,
                "Invalid params: expected object",
            );
        }
    };

    let uri = match params.get("uri").and_then(|v| v.as_str()) {
        Some(u) => u,
        None => {
            return JsonRpcResponse::error(
                request.id,
                error_codes::INVALID_PARAMS,
                "Missing required parameter: uri",
            );
        }
    };

    // Read the resource
    match mcp_resources::read_resource(uri, scripts, scriptlets, app_state) {
        Ok(content) => JsonRpcResponse::success(
            request.id,
            mcp_resources::resource_content_to_value(content),
        ),
        Err(err) => JsonRpcResponse::error(request.id, error_codes::METHOD_NOT_FOUND, err),
    }
}
