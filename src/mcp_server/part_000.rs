use crate::logging;
use crate::mcp_protocol::{self, JsonRpcResponse};
use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tracing::{debug, error, info, warn};
/// Default port for the MCP server
pub const DEFAULT_PORT: u16 = 43210;
/// MCP Server version for discovery
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Server capabilities advertised in discovery
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerCapabilities {
    pub scripts: bool,
    pub prompts: bool,
    pub tools: bool,
}
impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            scripts: true,
            prompts: true,
            tools: true,
        }
    }
}
/// Discovery file structure written to ~/.scriptkit/server.json
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryInfo {
    pub url: String,
    pub token: String,
    pub version: String,
    pub capabilities: ServerCapabilities,
}
/// MCP HTTP Server
///
/// Lightweight HTTP server for MCP protocol communication.
/// Uses std::net for simplicity (no async runtime required).
pub struct McpServer {
    port: u16,
    token: String,
    running: Arc<AtomicBool>,
    kit_path: PathBuf,
}
impl McpServer {
    /// Create a new MCP server instance
    ///
    /// # Arguments
    /// * `port` - Port to listen on (default: 43210)
    /// * `kit_path` - Path to ~/.scriptkit directory
    pub fn new(port: u16, kit_path: PathBuf) -> Result<Self> {
        let token = Self::load_or_create_token(&kit_path)?;

        Ok(Self {
            port,
            token,
            running: Arc::new(AtomicBool::new(false)),
            kit_path,
        })
    }

    /// Create server with default settings
    pub fn with_defaults() -> Result<Self> {
        let kit_path = dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".scriptkit");

        let port = std::env::var("MCP_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);

        Self::new(port, kit_path)
    }

    /// Load existing token or create a new one
    fn load_or_create_token(kit_path: &PathBuf) -> Result<String> {
        let token_path = kit_path.join("agent-token");

        if token_path.exists() {
            let token = fs::read_to_string(&token_path)
                .context("Failed to read agent-token file")?
                .trim()
                .to_string();

            if !token.is_empty() {
                info!("Loaded existing agent token from {:?}", token_path);
                return Ok(token);
            }
        }

        // Generate new token
        let token = uuid::Uuid::new_v4().to_string();

        // Ensure kit directory exists
        fs::create_dir_all(kit_path).context("Failed to create .kit directory")?;

        // Write token file with restrictive permissions (0o600 - owner read/write only)
        #[cfg(unix)]
        {
            use std::fs::OpenOptions;
            use std::os::unix::fs::OpenOptionsExt;

            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&token_path)
                .and_then(|mut file| {
                    use std::io::Write;
                    file.write_all(token.as_bytes())
                })
                .context("Failed to write agent-token file")?;
        }

        #[cfg(not(unix))]
        {
            fs::write(&token_path, &token).context("Failed to write agent-token file")?;
        }

        info!("Generated new agent token at {:?}", token_path);
        Ok(token)
    }

    /// Get the authentication token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the server URL
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Write discovery file to ~/.scriptkit/server.json
    fn write_discovery_file(&self) -> Result<()> {
        let discovery = DiscoveryInfo {
            url: self.url(),
            token: self.token.clone(),
            version: VERSION.to_string(),
            capabilities: ServerCapabilities::default(),
        };

        let discovery_path = self.kit_path.join("server.json");
        let json = serde_json::to_string_pretty(&discovery)
            .context("Failed to serialize discovery info")?;

        // Write discovery file with restrictive permissions (0o600 - contains token)
        #[cfg(unix)]
        {
            use std::fs::OpenOptions;
            use std::os::unix::fs::OpenOptionsExt;

            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&discovery_path)
                .and_then(|mut file| {
                    use std::io::Write;
                    file.write_all(json.as_bytes())
                })
                .context("Failed to write server.json")?;
        }

        #[cfg(not(unix))]
        {
            fs::write(&discovery_path, json).context("Failed to write server.json")?;
        }

        info!("Wrote discovery file to {:?}", discovery_path);
        Ok(())
    }

    /// Remove discovery file on shutdown
    fn remove_discovery_file(&self) {
        let discovery_path = self.kit_path.join("server.json");
        if discovery_path.exists() {
            if let Err(e) = fs::remove_file(&discovery_path) {
                warn!("Failed to remove discovery file: {}", e);
            } else {
                debug!("Removed discovery file");
            }
        }
    }

    /// Start the HTTP server in a background thread
    ///
    /// Returns a handle that can be used to stop the server.
    pub fn start(&self) -> Result<ServerHandle> {
        if self.is_running() {
            anyhow::bail!("Server is already running");
        }

        // Write discovery file before starting
        self.write_discovery_file()?;

        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .with_context(|| format!("Failed to bind to port {}", self.port))?;

        // Set non-blocking for graceful shutdown
        listener
            .set_nonblocking(true)
            .context("Failed to set non-blocking mode")?;

        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let token = self.token.clone();
        let kit_path = self.kit_path.clone();
        let port = self.port;

        let handle = thread::spawn(move || {
            info!("MCP server started on port {}", port);

            while running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        debug!("Connection from {}", addr);
                        let token = token.clone();
                        thread::spawn(move || {
                            if let Err(e) = handle_connection(stream, &token) {
                                error!("Error handling connection: {}", e);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection available, sleep before next poll.
                        // 100ms is responsive enough for new connections while reducing
                        // CPU wakeups from 100/sec to 10/sec (was 10ms = 100% CPU spin risk)
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        error!("Accept error: {}", e);
                    }
                }
            }

            // Cleanup on shutdown
            let discovery_path = kit_path.join("server.json");
            if discovery_path.exists() {
                let _ = fs::remove_file(&discovery_path);
            }

            info!("MCP server stopped");
        });

        Ok(ServerHandle {
            running: self.running.clone(),
            thread: Some(handle),
        })
    }

    /// Stop the server
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.remove_discovery_file();
    }
}
/// Handle for controlling the running server
pub struct ServerHandle {
    running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}
impl ServerHandle {
    /// Stop the server and wait for it to finish
    pub fn stop(mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }

    /// Check if server is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
impl Drop for ServerHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        // Note: we don't join here to avoid blocking on drop
    }
}
/// Handle a single HTTP connection
fn handle_connection(mut stream: TcpStream, expected_token: &str) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    // Read request line
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let request_line = request_line.trim();

    // Parse method and path
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return send_response(&mut stream, 400, "Bad Request", "Invalid request line");
    }

    let method = parts[0];
    let path = parts[1];
    let path_slug = path.trim_matches('/').replace('/', "_");
    let correlation_id = format!(
        "mcp:{}:{}:{}",
        method.to_lowercase(),
        if path_slug.is_empty() {
            "root".to_string()
        } else {
            path_slug
        },
        uuid::Uuid::new_v4()
    );
    let _request_guard = logging::set_correlation_id(correlation_id.clone());
    debug!(
        category = "MCP",
        event_type = "mcp_http_request",
        method = method,
        path = path,
        correlation_id = %correlation_id,
        "Received MCP HTTP request"
    );

    // Read headers
    let mut headers = std::collections::HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }

    // Check authorization for non-health endpoints
    if path != "/health" {
        let auth_valid = headers
            .get("authorization")
            .map(|auth| {
                auth.strip_prefix("Bearer ")
                    .map(|token| token == expected_token)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        if !auth_valid {
            return send_response(&mut stream, 401, "Unauthorized", "Invalid or missing token");
        }
    }

    // Route request
    match (method, path) {
        ("GET", "/health") => send_response(&mut stream, 200, "OK", r#"{"status":"healthy"}"#),
        ("GET", "/") => {
            let info = serde_json::json!({
                "name": "Script Kit MCP Server",
                "version": VERSION,
                "capabilities": ServerCapabilities::default(),
            });
            send_response(&mut stream, 200, "OK", &info.to_string())
        }
        ("POST", "/rpc") => {
            // Handle JSON-RPC request
            handle_rpc_request(&mut reader, &mut stream, &headers)
        }
        _ => send_response(&mut stream, 404, "Not Found", "Endpoint not found"),
    }
}
/// Handle a JSON-RPC request on the /rpc endpoint
fn handle_rpc_request(
    reader: &mut BufReader<TcpStream>,
    stream: &mut TcpStream,
    headers: &std::collections::HashMap<String, String>,
) -> Result<()> {
    // Get Content-Length
    let content_length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if content_length == 0 {
        let response = JsonRpcResponse::error(
            serde_json::Value::Null,
            mcp_protocol::error_codes::INVALID_REQUEST,
            "Missing or invalid Content-Length header",
        );
        let body = serde_json::to_string(&response)?;
        return send_response(stream, 400, "Bad Request", &body);
    }

    // Read request body
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    let body_str = String::from_utf8_lossy(&body);
    let body_summary = logging::summarize_payload(&body_str);
    debug!(
        category = "MCP",
        event_type = "mcp_rpc_request_body",
        content_length = content_length,
        payload_summary = %body_summary,
        "Received MCP RPC request body"
    );

    // Load scripts and scriptlets for context-aware responses
    // This allows resources/read and tools/list to return actual data
    let scripts = crate::scripts::read_scripts();
    let scriptlets = crate::scripts::load_scriptlets();

    // Parse and handle request with full context
    let response = match mcp_protocol::parse_request(&body_str) {
        Ok(request) => {
            mcp_protocol::handle_request_with_context(request, &scripts, &scriptlets, None)
        }
        Err(error_response) => error_response,
    };

    let response_body = serde_json::to_string(&response)?;
    send_response(stream, 200, "OK", &response_body)
}
/// Send an HTTP response
fn send_response(stream: &mut TcpStream, status: u16, reason: &str, body: &str) -> Result<()> {
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status,
        reason,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
