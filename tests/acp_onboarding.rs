//! Integration tests for generic ACP onboarding, recovery, and auth-capability hooks.
//!
//! These are source-level contract tests that verify the launch path and setup
//! surfaces no longer rely on Claude-specific copy or loaders.

const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const SETUP_RENDER_SOURCE: &str = include_str!("../src/ai/window/render_setup.rs");
const SETUP_SOURCE: &str = include_str!("../src/ai/window/setup.rs");
const CLIENT_SOURCE: &str = include_str!("../src/ai/acp/client.rs");

// ── Launch path uses catalog, not Claude-only loader ───────────────────

#[test]
fn tab_ai_mode_uses_catalog_loader_not_claude_only_loader() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("load_acp_agent_catalog_entries"),
        "tab_ai_mode must use the multi-agent catalog loader"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("resolve_default_acp_launch"),
        "tab_ai_mode must use preflight resolution"
    );
}

// ── AcpChatView supports setup constructor ─────────────────────────────

#[test]
fn acp_view_supports_setup_constructor() {
    assert!(
        ACP_VIEW_SOURCE.contains("fn new_setup"),
        "AcpChatView must have a new_setup constructor"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("AcpChatSession::Setup"),
        "AcpChatView must support Setup session state"
    );
}

// ── Setup surface uses generic ACP copy ────────────────────────────────

#[test]
fn ai_setup_surface_no_longer_mentions_claude_only_copy() {
    assert!(
        SETUP_RENDER_SOURCE.contains("ACP Agent Required"),
        "setup card title must say ACP Agent Required"
    );
    assert!(
        SETUP_RENDER_SOURCE.contains("Open ACP Agent Catalog"),
        "setup card must offer Open ACP Agent Catalog"
    );
    assert!(
        !SETUP_RENDER_SOURCE.contains("Connect to Claude Code"),
        "setup card must NOT mention Claude Code specifically"
    );
}

#[test]
fn setup_button_id_is_generic_not_claude_specific() {
    assert!(
        SETUP_RENDER_SOURCE.contains("open-acp-catalog-btn"),
        "catalog button ID must be generic"
    );
    assert!(
        !SETUP_RENDER_SOURCE.contains("connect-claude-code-btn"),
        "catalog button must NOT use Claude-specific ID"
    );
}

// ── Setup.rs has catalog opener ────────────────────────────────────────

#[test]
fn setup_has_open_acp_agents_catalog() {
    assert!(
        SETUP_SOURCE.contains("fn open_acp_agents_catalog"),
        "setup.rs must have open_acp_agents_catalog method"
    );
    assert!(
        SETUP_SOURCE.contains("default_acp_agents_path"),
        "open_acp_agents_catalog must use the default catalog path"
    );
}

// ── Client advertises terminal auth capability ─────────────────────────

#[test]
fn client_advertises_auth_capability() {
    assert!(
        CLIENT_SOURCE.contains("AuthCapabilities"),
        "client must use AuthCapabilities in initialize request"
    );
    assert!(
        CLIENT_SOURCE.contains(".auth("),
        "client must chain .auth() on ClientCapabilities"
    );
}

#[test]
fn client_records_auth_methods_from_initialize() {
    assert!(
        CLIENT_SOURCE.contains("auth_method_count"),
        "client must log auth_method_count from initialize response"
    );
    assert!(
        CLIENT_SOURCE.contains("auth_methods"),
        "client must record auth_methods from initialize response"
    );
}

// ── Client handles auth_required as structured setup event ─────────────

#[test]
fn client_emits_setup_required_on_auth_failure() {
    assert!(
        CLIENT_SOURCE.contains("auth_required"),
        "client must detect auth_required condition"
    );
    assert!(
        CLIENT_SOURCE.contains("AcpEvent::SetupRequired"),
        "client must emit SetupRequired event on auth failure"
    );
    assert!(
        CLIENT_SOURCE.contains("acp_auth_required"),
        "client must log acp_auth_required event"
    );
}
