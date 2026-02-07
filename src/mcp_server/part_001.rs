#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use tempfile::TempDir;

    /// Helper to create a server with a temporary kit directory
    fn create_test_server(port: u16) -> (McpServer, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let server = McpServer::new(port, temp_dir.path().to_path_buf()).unwrap();
        (server, temp_dir)
    }

    /// Helper to send an HTTP request and get the response
    fn http_request(port: u16, method: &str, path: &str, token: Option<&str>) -> (u16, String) {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let mut request = format!("{} {} HTTP/1.1\r\nHost: localhost\r\n", method, path);
        if let Some(token) = token {
            request.push_str(&format!("Authorization: Bearer {}\r\n", token));
        }
        request.push_str("\r\n");

        stream.write_all(request.as_bytes()).unwrap();
        stream.flush().unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        // Parse status code from response
        let status_line = response.lines().next().unwrap_or("");
        let status_code = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Get body (after blank line)
        let body = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();

        (status_code, body)
    }

    #[test]
    fn test_server_starts_and_stops() {
        let (server, _temp_dir) = create_test_server(43211);

        // Server should not be running initially
        assert!(!server.is_running());

        // Start server
        let handle = server.start().unwrap();

        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(100));

        // Server should be running
        assert!(handle.is_running());

        // Stop server
        handle.stop();

        // Server should stop
        assert!(!server.is_running());
    }

    #[test]
    fn test_health_endpoint_returns_200() {
        let (server, _temp_dir) = create_test_server(43212);
        let _handle = server.start().unwrap();

        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(100));

        let (status, body) = http_request(43212, "GET", "/health", None);

        assert_eq!(status, 200);
        assert!(body.contains("healthy"));
    }

    #[test]
    fn test_auth_rejects_invalid_token() {
        let (server, _temp_dir) = create_test_server(43213);
        let _handle = server.start().unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        // Request to root without token should fail
        let (status, _) = http_request(43213, "GET", "/", None);
        assert_eq!(status, 401);

        // Request with wrong token should fail
        let (status, _) = http_request(43213, "GET", "/", Some("wrong-token"));
        assert_eq!(status, 401);
    }

    #[test]
    fn test_auth_accepts_valid_token() {
        let (server, _temp_dir) = create_test_server(43214);
        let token = server.token().to_string();
        let _handle = server.start().unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        let (status, body) = http_request(43214, "GET", "/", Some(&token));

        assert_eq!(status, 200);
        assert!(body.contains("Script Kit MCP Server"));
    }

    #[test]
    fn test_discovery_file_created() {
        let (server, temp_dir) = create_test_server(43215);
        let token = server.token().to_string();

        // Discovery file should not exist before start
        let discovery_path = temp_dir.path().join("server.json");
        assert!(!discovery_path.exists());

        // Start server
        let handle = server.start().unwrap();
        thread::sleep(std::time::Duration::from_millis(100));

        // Discovery file should exist
        assert!(discovery_path.exists());

        // Verify contents
        let content = fs::read_to_string(&discovery_path).unwrap();
        let discovery: DiscoveryInfo = serde_json::from_str(&content).unwrap();

        assert!(discovery.url.contains("43215"));
        assert_eq!(discovery.token, token);
        assert_eq!(discovery.version, VERSION);
        assert!(discovery.capabilities.scripts);

        // Stop server
        handle.stop();

        // Discovery file should be removed after stop
        thread::sleep(std::time::Duration::from_millis(100));
        assert!(!discovery_path.exists());
    }

    #[test]
    fn test_generates_token_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let token_path = temp_dir.path().join("agent-token");

        // Token file should not exist
        assert!(!token_path.exists());

        // Create server - should generate token
        let server = McpServer::new(43216, temp_dir.path().to_path_buf()).unwrap();

        // Token file should now exist
        assert!(token_path.exists());

        // Token should be a valid UUID-like string
        let token = server.token();
        assert!(!token.is_empty());
        assert!(token.len() >= 32); // UUID v4 format

        // Token should match file contents
        let file_token = fs::read_to_string(&token_path).unwrap();
        assert_eq!(token, file_token.trim());

        // Creating another server should use the same token
        let server2 = McpServer::new(43217, temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(server.token(), server2.token());
    }

    #[test]
    fn test_url_format() {
        let (server, _temp_dir) = create_test_server(43218);
        assert_eq!(server.url(), "http://localhost:43218");
    }

    /// Helper to send a POST request with a JSON body
    fn http_post_json(port: u16, path: &str, token: &str, body: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let request = format!(
            "POST {} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Authorization: Bearer {}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             \r\n\
             {}",
            path,
            token,
            body.len(),
            body
        );

        stream.write_all(request.as_bytes()).unwrap();
        stream.flush().unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        // Parse status code from response
        let status_line = response.lines().next().unwrap_or("");
        let status_code = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Get body (after blank line)
        let body = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();

        (status_code, body)
    }

    #[test]
    fn test_rpc_endpoint_tools_list() {
        let (server, _temp_dir) = create_test_server(43219);
        let token = server.token().to_string();
        let _handle = server.start().unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        let (status, body) = http_post_json(43219, "/rpc", &token, request);

        assert_eq!(status, 200);

        // Parse response
        let response: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert!(response["result"]["tools"].is_array());
    }

    #[test]
    fn test_rpc_endpoint_initialize() {
        let (server, _temp_dir) = create_test_server(43220);
        let token = server.token().to_string();
        let _handle = server.start().unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        let request = r#"{"jsonrpc":"2.0","id":"init-1","method":"initialize","params":{}}"#;
        let (status, body) = http_post_json(43220, "/rpc", &token, request);

        assert_eq!(status, 200);

        let response: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "init-1");
        assert!(response["result"]["serverInfo"]["name"].is_string());
        assert!(response["result"]["capabilities"].is_object());
    }

    #[test]
    fn test_rpc_endpoint_method_not_found() {
        let (server, _temp_dir) = create_test_server(43221);
        let token = server.token().to_string();
        let _handle = server.start().unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        let request = r#"{"jsonrpc":"2.0","id":99,"method":"unknown/method","params":{}}"#;
        let (status, body) = http_post_json(43221, "/rpc", &token, request);

        assert_eq!(status, 200);

        let response: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 99);
        assert_eq!(response["error"]["code"], -32601);
        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Method not found"));
    }

    #[test]
    fn test_rpc_endpoint_invalid_json() {
        let (server, _temp_dir) = create_test_server(43222);
        let token = server.token().to_string();
        let _handle = server.start().unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        let request = r#"{"jsonrpc":"2.0", invalid}"#;
        let (status, body) = http_post_json(43222, "/rpc", &token, request);

        assert_eq!(status, 200);

        let response: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["error"]["code"], -32700); // Parse error
    }

    #[test]
    fn test_rpc_endpoint_requires_auth() {
        let (server, _temp_dir) = create_test_server(43223);
        let _handle = server.start().unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        // Try POST /rpc without token - should fail auth
        let mut stream = TcpStream::connect("127.0.0.1:43223").unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let request = format!(
            "POST /rpc HTTP/1.1\r\n\
             Host: localhost\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             \r\n\
             {}",
            body.len(),
            body
        );

        stream.write_all(request.as_bytes()).unwrap();
        stream.flush().unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        let status_line = response.lines().next().unwrap_or("");
        let status_code: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        assert_eq!(status_code, 401);
    }

    #[test]
    fn test_rpc_endpoint_resources_list() {
        let (server, _temp_dir) = create_test_server(43224);
        let token = server.token().to_string();
        let _handle = server.start().unwrap();

        thread::sleep(std::time::Duration::from_millis(100));

        let request = r#"{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}"#;
        let (status, body) = http_post_json(43224, "/rpc", &token, request);

        assert_eq!(status, 200);

        let response: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 2);
        assert!(response["result"]["resources"].is_array());
    }

    #[test]
    #[cfg(unix)]
    fn test_token_file_has_secure_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let token_path = temp_dir.path().join("agent-token");

        // Create server - should generate token with secure permissions
        let _server = McpServer::new(43225, temp_dir.path().to_path_buf()).unwrap();

        // Token file should exist
        assert!(token_path.exists());

        // Check file permissions (should be 0o600 - owner read/write only)
        let metadata = fs::metadata(&token_path).unwrap();
        let mode = metadata.permissions().mode();
        let file_perms = mode & 0o777;

        assert_eq!(
            file_perms, 0o600,
            "Token file should have 0o600 permissions, got 0o{:o}",
            file_perms
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_discovery_file_has_secure_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let (server, temp_dir) = create_test_server(43226);
        let discovery_path = temp_dir.path().join("server.json");

        // Start server - should create discovery file with secure permissions
        let _handle = server.start().unwrap();
        thread::sleep(std::time::Duration::from_millis(100));

        // Discovery file should exist
        assert!(discovery_path.exists());

        // Check file permissions (should be 0o600 - contains token)
        let metadata = fs::metadata(&discovery_path).unwrap();
        let mode = metadata.permissions().mode();
        let file_perms = mode & 0o777;

        assert_eq!(
            file_perms, 0o600,
            "Discovery file should have 0o600 permissions, got 0o{:o}",
            file_perms
        );
    }
}
