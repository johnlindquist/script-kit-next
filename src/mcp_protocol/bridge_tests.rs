//! The HTTP server and real Notes store run in a separate process with a new HOME. No app launch,
//! global clipboard, personal scripts, desktop permission or agent provider is needed.
#[test]
fn external_agent_journey() {
    let scratch = tempfile::tempdir().unwrap();
    let home = scratch.path().join("home");
    let kit = home.join(".scriptkit");
    std::fs::create_dir_all(&kit).unwrap();
    let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
    cmd.args([
        "--exact",
        "mcp_protocol::bridge_tests::private_server",
        "--nocapture",
    ])
    .env("SK_BRIDGE_FIXTURE", scratch.path())
    .env("HOME", &home)
    .env("SK_PATH", &kit)
    .env("SCRIPT_KIT_TEST_NOTES_DB_PATH", kit.join("db/notes.sqlite"))
    .env("SCRIPT_KIT_TEST_NOTES_BRAIN_PATH", kit.join("brain"))
    .env_remove("SCRIPT_KIT_MCP_TOKEN")
    .env_remove("SCRIPT_KIT_MCP_SERVER_JSON")
    .env_remove("SCRIPT_KIT_MCP_ENDPOINT")
    .env_remove("SCRIPT_KIT_AGENT_CREDENTIAL");
    if std::env::var_os("SK_TEST_BRIDGE_JUDGE").is_some_and(|v| v == "1") {
        assert!(cmd.status().unwrap().success());
        return;
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success()
            && String::from_utf8_lossy(&out.stdout)
                .contains("private external agent journey passed"),
        "{} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // No shared open connection or in-memory note cache: start a new Kit server process.
    let cold = cmd.env("SK_BRIDGE_COLD_READ", "1").output().unwrap();
    assert!(
        cold.status.success()
            && String::from_utf8_lossy(&cold.stdout).contains("Cold readback passed"),
        "{} {}",
        String::from_utf8_lossy(&cold.stdout),
        String::from_utf8_lossy(&cold.stderr)
    );
}
#[test]
fn private_server() {
    let Some(root) = std::env::var_os("SK_BRIDGE_FIXTURE").map(std::path::PathBuf::from) else {
        return;
    };
    let kit = root.join("home/.scriptkit");
    assert_eq!(crate::setup::get_kit_path(), kit);
    assert_eq!(
        std::env::var_os("HOME").map(std::path::PathBuf::from),
        Some(root.join("home"))
    );
    let installed = crate::setup::ensure_kit_setup();
    assert_eq!(installed.kit_path, kit);
    assert!(
        installed.warnings.is_empty(),
        "isolated setup reported: {:?}",
        installed.warnings
    );
    let port = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let server = crate::mcp_server::McpServer::new(port, kit).unwrap();
    let handle = server.start().unwrap();
    let test = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts/agentic/external-agent-journey.ts");
    let hc = tempfile::Builder::new()
        .prefix("sk-bridge-")
        .tempdir_in("/tmp")
        .unwrap();
    let socket = hc.path().join("peer.sock");
    super::bridge::herdr::TEST_SOCKET
        .set(socket.clone())
        .unwrap();
    let result = std::process::Command::new("bun")
        .arg(test)
        .current_dir(&root)
        .env("HC_SOCKET", socket)
        .env("HC_SESSION", "bridge")
        .env_remove("HC_TERMINAL_ID")
        .env("XDG_STATE_HOME", root.join("state"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_RUNTIME_DIR", hc.path())
        .status()
        .unwrap();
    handle.stop();
    assert!(result.success());
    println!("private external agent journey passed");
}
