//! Integration tests for generic ACP onboarding, recovery, and auth-capability hooks.
//!
//! These are source-level contract tests that verify the launch path and setup
//! surfaces no longer rely on Claude-specific copy or loaders.

const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");
const ACP_VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const SETUP_RENDER_SOURCE: &str = include_str!("../src/ai/window/render_setup.rs");
const SETUP_SOURCE: &str = include_str!("../src/ai/window/setup.rs");
const CLIENT_SOURCE: &str = include_str!("../src/ai/acp/client.rs");
const ACP_CONFIG_SOURCE: &str = include_str!("../src/ai/acp/config.rs");
const CONFIG_TYPES_SOURCE: &str = include_str!("../src/config/types.rs");

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

// ── AiPreferences includes selected_acp_agent_id ──────────────────────

#[test]
fn ai_preferences_include_selected_acp_agent_id() {
    assert!(
        CONFIG_TYPES_SOURCE.contains("selected_acp_agent_id"),
        "AiPreferences must persist selected_acp_agent_id"
    );
}

// ── tab_ai_mode passes preferred agent to preflight ───────────────────

#[test]
fn tab_ai_mode_passes_preferred_agent_to_preflight() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("preferred_agent_id.as_deref()"),
        "tab_ai_mode must pass the persisted preferred agent into resolve_default_acp_launch"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("load_preferred_acp_agent_id"),
        "tab_ai_mode must load the preferred agent from user preferences"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("persist_preferred_acp_agent_id"),
        "tab_ai_mode must persist the resolved preferred agent after launch"
    );
}

// ── ACP config exposes preference helpers ─────────────────────────────

#[test]
fn acp_config_exposes_agent_preference_helpers() {
    assert!(
        ACP_CONFIG_SOURCE.contains("load_preferred_acp_agent_id"),
        "acp config must expose a preferred-agent loader"
    );
    assert!(
        ACP_CONFIG_SOURCE.contains("persist_preferred_acp_agent_id"),
        "acp config must expose a preferred-agent persistence helper"
    );
}

// ── Catalog loader classifies built-in agents ─────────────────────────

#[test]
fn catalog_loader_classifies_builtin_agents() {
    assert!(
        ACP_CONFIG_SOURCE.contains("AcpAgentSource::BuiltIn"),
        "catalog loader must classify built-in ACP agents distinctly"
    );
    assert!(
        ACP_CONFIG_SOURCE.contains("classify_agent_source"),
        "catalog loader should centralize source classification"
    );
}

// ── Built-in classification distinguishes legacy Claude from built-ins ─

#[test]
fn classify_agent_source_distinguishes_legacy_from_builtin() {
    // The classifier must assign LegacyClaudeCode to "claude-code" and
    // BuiltIn to the well-known auto-detected agents.
    assert!(
        ACP_CONFIG_SOURCE.contains(r#""claude-code" => super::catalog::AcpAgentSource::LegacyClaudeCode"#),
        "claude-code must be classified as LegacyClaudeCode"
    );
    assert!(
        ACP_CONFIG_SOURCE.contains(r#""opencode" | "codex-acp" => super::catalog::AcpAgentSource::BuiltIn"#),
        "opencode and codex-acp must be classified as BuiltIn"
    );
}

// ── Catalog builder emits per-entry structured logs ──────────────────

#[test]
fn catalog_builder_emits_per_entry_logs() {
    assert!(
        ACP_CONFIG_SOURCE.contains("acp_agent_catalog_entry_built"),
        "catalog builder must emit per-entry structured log events"
    );
    assert!(
        ACP_CONFIG_SOURCE.contains("install_state = ?install_state"),
        "per-entry log must include install_state"
    );
    assert!(
        ACP_CONFIG_SOURCE.contains("config_state = ?config_state"),
        "per-entry log must include config_state"
    );
}

// ── AcpThreadInit carries selected_agent ─────────────────────────────

const ACP_THREAD_SOURCE: &str = include_str!("../src/ai/acp/thread.rs");
const SETUP_STATE_SOURCE: &str = include_str!("../src/ai/acp/setup_state.rs");

#[test]
fn acp_thread_init_includes_selected_agent() {
    assert!(
        ACP_THREAD_SOURCE.contains("pub selected_agent: Option<super::catalog::AcpAgentCatalogEntry>"),
        "AcpThreadInit must carry the selected agent catalog entry"
    );
}

// ── Runtime SetupRequired arms inline setup state ────────────────────

#[test]
fn runtime_setup_required_arms_inline_setup_state() {
    assert!(
        ACP_THREAD_SOURCE.contains("from_runtime_setup_required"),
        "AcpThread must convert SetupRequired events into inline setup state"
    );
    assert!(
        ACP_THREAD_SOURCE.contains("acp_runtime_setup_session_armed"),
        "runtime setup recovery must be logged"
    );
}

#[test]
fn setup_state_has_runtime_constructor() {
    assert!(
        SETUP_STATE_SOURCE.contains("fn from_runtime_setup_required"),
        "AcpInlineSetupState must have from_runtime_setup_required constructor"
    );
    assert!(
        SETUP_STATE_SOURCE.contains("auth_required"),
        "runtime setup constructor must handle auth_required reason"
    );
}

// ── View renders runtime setup state ─────────────────────────────────

#[test]
fn acp_view_renders_runtime_setup_state() {
    assert!(
        ACP_VIEW_SOURCE.contains("thread_ref.setup_state()"),
        "AcpChatView render must check thread setup_state for runtime recovery"
    );
}
