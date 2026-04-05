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
        TAB_AI_MODE_SOURCE.contains("resolve_acp_launch_with_requirements"),
        "tab_ai_mode must use capability-aware preflight resolution"
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
        ACP_CONFIG_SOURCE
            .contains(r#""claude-code" => super::catalog::AcpAgentSource::LegacyClaudeCode"#),
        "claude-code must be classified as LegacyClaudeCode"
    );
    assert!(
        ACP_CONFIG_SOURCE
            .contains(r#""opencode" | "codex-acp" => super::catalog::AcpAgentSource::BuiltIn"#),
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
        ACP_THREAD_SOURCE
            .contains("pub selected_agent: Option<super::catalog::AcpAgentCatalogEntry>"),
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

// ── Launch requirements threaded into thread init ────────────────────

#[test]
fn tab_ai_mode_threads_launch_requirements_into_acp_thread_init() {
    assert!(
        TAB_AI_MODE_SOURCE.contains("launch_requirements: requirements"),
        "tab_ai_mode must pass launch requirements into AcpThreadInit"
    );
}

#[test]
fn acp_thread_init_carries_launch_requirements() {
    assert!(
        ACP_THREAD_SOURCE
            .contains("pub launch_requirements: super::preflight::AcpLaunchRequirements"),
        "AcpThreadInit must carry launch_requirements field"
    );
}

// ── Runtime recovery preserves launch requirements ──────────────────

#[test]
fn runtime_setup_required_preserves_launch_requirements() {
    assert!(
        ACP_THREAD_SOURCE.contains("self.launch_requirements"),
        "runtime setup recovery must preserve launch requirements instead of resetting to default"
    );
    assert!(
        ACP_THREAD_SOURCE.contains("acp_runtime_setup_requirements_preserved"),
        "runtime setup recovery must emit a structured preservation log"
    );
}

// ── Runtime setup only suggests capable alternatives ────────────────

#[test]
fn runtime_setup_state_only_suggests_capable_alternatives() {
    assert!(
        SETUP_STATE_SOURCE.contains("has_launchable_capable_alternative"),
        "runtime setup must only suggest switching to alternatives that satisfy launch requirements"
    );
}

// ── from_resolution is fully capability-aware ──────────────────────

#[test]
fn from_resolution_uses_can_switch_capable_for_install_branch() {
    assert!(
        SETUP_STATE_SOURCE.contains("AgentNotInstalled) if can_switch_capable"),
        "AgentNotInstalled must gate on can_switch_capable, not can_switch"
    );
}

#[test]
fn from_resolution_uses_can_switch_capable_for_auth_branch() {
    assert!(
        SETUP_STATE_SOURCE.contains("AuthenticationRequired) if can_switch_capable"),
        "AuthenticationRequired must gate on can_switch_capable, not can_switch"
    );
}

#[test]
fn from_resolution_uses_can_switch_capable_for_misconfig_branch() {
    assert!(
        SETUP_STATE_SOURCE.contains("AgentMisconfigured) if can_switch_capable"),
        "AgentMisconfigured must gate on can_switch_capable, not can_switch"
    );
}

#[test]
fn from_resolution_emits_structured_log() {
    assert!(
        SETUP_STATE_SOURCE.contains("acp_setup_state_from_resolution"),
        "from_resolution must emit acp_setup_state_from_resolution log event"
    );
    assert!(
        SETUP_STATE_SOURCE.contains("can_switch_capable"),
        "from_resolution log must include can_switch_capable field"
    );
}

#[test]
fn from_resolution_has_capability_gap_message_helper() {
    assert!(
        SETUP_STATE_SOURCE.contains("fn capability_gap_message"),
        "setup_state must have capability_gap_message helper"
    );
}

// ── Picker surfaces auth + capability labels ──────────────────────

#[test]
fn picker_row_includes_auth_label() {
    assert!(
        ACP_VIEW_SOURCE.contains("setup_agent_auth_label"),
        "picker must render auth state labels"
    );
}

#[test]
fn picker_row_includes_capability_label() {
    assert!(
        ACP_VIEW_SOURCE.contains("setup_agent_capability_label"),
        "picker must render capability compatibility labels"
    );
}

#[test]
fn picker_log_includes_compatible_count() {
    assert!(
        ACP_VIEW_SOURCE.contains("compatible_count"),
        "picker opened log must include compatible_count"
    );
}

// ── Setup picker confirmation updates live thread ───────────────────

#[test]
fn setup_picker_confirm_updates_live_thread_selected_agent() {
    assert!(
        ACP_VIEW_SOURCE.contains("replace_selected_agent"),
        "setup picker confirmation must update the live thread selected agent"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("acp_setup_agent_confirmed_for_runtime_recovery"),
        "setup picker confirmation must emit a structured log"
    );
}

// ── Setup picker uses synchronous persistence before retry ──────────

#[test]
fn setup_picker_uses_sync_persistence_before_retry() {
    // The confirm path must call the synchronous helper so the persisted
    // preference is already on disk when a retry reloads it.
    assert!(
        ACP_VIEW_SOURCE.contains("persist_preferred_acp_agent_id_sync"),
        "confirm_setup_agent_picker must use the synchronous persistence helper"
    );
    assert!(
        !ACP_VIEW_SOURCE.contains("persist_preferred_acp_agent_id(Some(agent.id"),
        "confirm_setup_agent_picker must NOT call the async persistence helper directly"
    );
}

#[test]
fn setup_picker_gates_retry_on_persistence_success() {
    // Auto-retry must depend on both resolution readiness AND sync persistence.
    assert!(
        ACP_VIEW_SOURCE.contains("resolution.is_ready() && persist_result.is_ok()"),
        "auto-retry must be gated on both resolution readiness and sync persistence success"
    );
}

#[test]
fn setup_picker_emits_persist_before_retry_log() {
    assert!(
        ACP_VIEW_SOURCE.contains("acp_setup_agent_persist_before_retry"),
        "confirm path must emit acp_setup_agent_persist_before_retry log"
    );
    assert!(
        ACP_VIEW_SOURCE.contains("persisted = persist_result.is_ok()"),
        "persist-before-retry log must include the persisted outcome"
    );
}

#[test]
fn acp_config_exposes_sync_persistence_helper() {
    assert!(
        ACP_CONFIG_SOURCE.contains("fn persist_preferred_acp_agent_id_sync"),
        "acp config must expose a synchronous preferred-agent persistence helper"
    );
}

#[test]
fn async_persistence_delegates_to_sync_helper() {
    // The async helper must delegate to the sync helper to avoid duplicating
    // the write logic.
    let async_fn_start = ACP_CONFIG_SOURCE
        .find("fn persist_preferred_acp_agent_id(agent_id")
        .expect("async persistence helper must exist");
    let async_fn_body = &ACP_CONFIG_SOURCE[async_fn_start..];
    let next_fn = async_fn_body[1..]
        .find("\npub(crate) fn ")
        .unwrap_or(async_fn_body.len());
    let async_fn_body = &async_fn_body[..next_fn];

    assert!(
        async_fn_body.contains("persist_preferred_acp_agent_id_sync"),
        "async persist helper must delegate to the sync helper internally"
    );
}

// ── Post-launch persistence rule: fallback must not overwrite preference ──

#[test]
fn post_launch_persist_decision_is_conditional() {
    // The open path must NOT unconditionally persist the selected agent.
    // It must check whether the launch was explicit (retry), first-run,
    // or already aligned with the saved preference.
    assert!(
        TAB_AI_MODE_SOURCE.contains("should_persist_selected_agent"),
        "tab_ai_mode must compute a should_persist_selected_agent decision"
    );
    assert!(
        TAB_AI_MODE_SOURCE.contains("acp_preferred_agent_post_launch_persist_decision"),
        "tab_ai_mode must emit the post-launch persist decision tracing event"
    );
}

#[test]
fn fallback_launch_preserves_existing_preference() {
    // When a capability-driven fallback selects a different agent than the
    // saved preference, the open path must NOT overwrite the preference.
    assert!(
        TAB_AI_MODE_SOURCE.contains("acp_preferred_agent_preserved_during_fallback_launch"),
        "tab_ai_mode must emit a preservation event when fallback skips persistence"
    );
}

#[test]
fn post_launch_persist_gates_on_retry_or_first_run_or_match() {
    // The persistence guard must only persist when:
    // 1. retry_request.is_some() (explicit retry)
    // 2. preferred_agent_id.is_none() (first-run / no prior preference)
    // 3. preferred == selected (already aligned)
    let decision_block_start = TAB_AI_MODE_SOURCE
        .find("should_persist_selected_agent")
        .expect("should_persist_selected_agent must exist in tab_ai_mode");
    let decision_context = &TAB_AI_MODE_SOURCE[decision_block_start..decision_block_start + 300];

    assert!(
        decision_context.contains("retry_request.is_some()"),
        "persist decision must check for explicit retry request"
    );
    assert!(
        decision_context.contains("preferred_agent_id.is_none()"),
        "persist decision must check for absent prior preference (first-run)"
    );
    assert!(
        decision_context.contains("preferred_agent_id.as_deref() == selected_agent_id.as_deref()"),
        "persist decision must check whether preference already matches selection"
    );
}
