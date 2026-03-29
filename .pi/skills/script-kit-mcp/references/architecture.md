# MCP Architecture Reference

## Module Dependency Graph

```
                    +------------------+
                    |   mcp_server.rs  |
                    |   (HTTP Server)  |
                    +--------+---------+
                             |
                             v
                    +------------------+
                    | mcp_protocol.rs  |
                    |  (JSON-RPC 2.0)  |
                    +--------+---------+
                             |
         +-------------------+-------------------+
         |                   |                   |
         v                   v                   v
+----------------+  +------------------+  +----------------+
| mcp_kit_tools  |  | mcp_script_tools |  | mcp_resources  |
|  (kit/* tools) |  | (scripts/* tools)|  |  (Resources)   |
+----------------+  +------------------+  +----------------+
                             |
                             v
                    +------------------+
                    |  mcp_streaming   |
                    | (SSE + Auditing) |
                    +------------------+
```

## Request Flow

### 1. HTTP Request Handling (mcp_server.rs)

```
Client -> TcpListener (127.0.0.1:43210)
          |
          v
       handle_connection()
          |
          +-> GET /health     -> 200 {"status":"healthy"}
          +-> GET /           -> 200 Server info (requires auth)
          +-> POST /rpc       -> handle_rpc_request() (requires auth)
          +-> *               -> 404 Not Found
```

### 2. Authentication Flow

```
Request -> Check Authorization header
           |
           +-> Missing/Invalid -> 401 Unauthorized
           +-> Valid Bearer token -> Continue to handler
           
Exception: GET /health bypasses auth
```

### 3. JSON-RPC Request Processing (mcp_protocol.rs)

```
JSON String -> parse_request()
               |
               +-> Parse error -> -32700 PARSE_ERROR
               +-> Invalid jsonrpc -> -32600 INVALID_REQUEST
               +-> Missing method -> -32600 INVALID_REQUEST
               +-> Valid -> handle_request_with_context()
```

### 4. Method Routing

```
handle_request_with_context(request, scripts, scriptlets, app_state)
    |
    +-> "initialize"      -> handle_initialize()
    +-> "tools/list"      -> handle_tools_list_with_scripts()
    +-> "tools/call"      -> handle_tools_call_with_scripts()
    +-> "resources/list"  -> handle_resources_list()
    +-> "resources/read"  -> handle_resources_read_with_context()
    +-> unknown           -> -32601 METHOD_NOT_FOUND
```

### 5. tools/call Routing

```
tools/call with name parameter
    |
    +-> kit/*      -> mcp_kit_tools::handle_kit_tool_call()
    +-> scripts/*  -> mcp_script_tools::handle_script_tool_call()
    +-> unknown    -> -32601 METHOD_NOT_FOUND
```

## Data Flow for Script Tools

```
1. Scripts loaded from ~/.scriptkit/scripts/
                    |
                    v
2. Filter scripts with schema.input
                    |
                    v
3. Generate tool definitions (scripts/{slug})
                    |
                    v
4. Return in tools/list response
                    |
                    v
5. On tools/call:
   - Find script by slug
   - Return pending execution with script path
   - Actual execution handled by caller
```

## Resource Data Flow

```
resources/list:
    get_resource_definitions() -> [kit://state, scripts://, scriptlets://]

resources/read with uri:
    |
    +-> kit://state     -> AppStateResource JSON
    +-> scripts://      -> [ScriptResourceEntry...] JSON
    +-> scriptlets://   -> [ScriptletResourceEntry...] JSON
    +-> unknown         -> Error "Resource not found"
```

## Server Lifecycle

```
1. McpServer::new(port, kit_path)
   - Load or create agent-token
   
2. server.start()
   - Write discovery file (~/.scriptkit/server.json)
   - Bind to port
   - Set running flag
   - Spawn listener thread
   
3. While running:
   - Accept connections (non-blocking)
   - Spawn handler thread per connection
   - Process request
   - Send response
   
4. handle.stop() or Drop
   - Clear running flag
   - Remove discovery file
   - Thread exits gracefully
```

## SSE Streaming Architecture

```
SseStream
    |
    +-> broadcast_event(type, data)
    |       |
    |       v
    |   buffer.push(formatted_sse)
    |
    +-> drain_events()
            |
            v
        Return and clear buffer

Event format:
    event: {type}\n
    data: {json}\n
    \n
```

## Audit Logging Architecture

```
Tool Call
    |
    +-> Start timer
    |
    v
    Execute tool
    |
    +-> Success -> AuditLogEntry::success(method, params, duration)
    +-> Failure -> AuditLogEntry::failure(method, params, duration, error)
            |
            v
    AuditLogger::log(entry)
            |
            v
    Append to ~/.scriptkit/logs/mcp-audit.jsonl
```

## Key Type Relationships

```
JsonRpcRequest
    |
    +-> method: String -> McpMethod enum
    +-> params: Value  -> Tool-specific params
    +-> id: Value      -> Preserved in response

JsonRpcResponse
    |
    +-> Success: result = Some(Value)
    +-> Error: error = Some(JsonRpcError)

ToolDefinition (for tools/list)
    |
    +-> name: String
    +-> description: String
    +-> inputSchema: Value (JSON Schema)

ToolResult (for tools/call)
    |
    +-> content: Vec<ToolContent>
    +-> isError: Option<bool>  // Note: Rust field is is_error, serializes as isError

McpResource (for resources/list)
    |
    +-> uri: String
    +-> name: String
    +-> description: Option<String>
    +-> mimeType: String

ResourceContent (for resources/read)
    |
    +-> uri: String
    +-> mimeType: String  // Note: Rust field is mime_type, serializes as mimeType
    +-> text: String (JSON stringified)
```

## Thread Safety

- `McpServer` uses `Arc<AtomicBool>` for running state
- Handler threads receive cloned token (String is cheap to clone)
- Scripts and scriptlets are `Arc<T>` for shared ownership
- Each connection gets its own handler thread
- No shared mutable state between handlers
