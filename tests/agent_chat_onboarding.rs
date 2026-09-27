//! Integration tests for generic Agent Chat onboarding, recovery, and auth-capability hooks.
//!
//! These are source-level contract tests that verify the launch path and setup
//! surfaces no longer rely on Claude-specific copy or loaders.

const AGENT_CHAT_CONFIG_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/config.rs");
const AGENT_CHAT_CONFIG_TESTS_SOURCE: &str =
    include_str!("../src/ai/agent_chat/ui/config_tests.rs");
const CONFIG_TYPES_SOURCE: &str = include_str!("../src/config/types.rs");
const RELEASE_WORKFLOW_SOURCE: &str = include_str!("../.github/workflows/release.yml");
const VERIFY_MACOS_BUNDLE_SOURCE: &str = include_str!("../scripts/verify-macos-bundle.sh");

// ── Launch path uses catalog, not Claude-only loader ───────────────────

// ── AgentChatView supports setup constructor ─────────────────────────────

// (Removed: legacy Agent Chat-client auth-capability tests — they referenced a
// `CLIENT_SOURCE` include of the deleted Agent Chat client. The Agent Chat backend/client
// was removed; all sessions use the Pi backend.)

// ── AiPreferences Pi-only backend ──────────────────────

#[test]
fn agent_chat_profile_config_keeps_legacy_ai_keys() {
    for needed in [
        "selected_model_id",
        "selected_profile_name",
        "profiles: Vec<AgentChatProfile>",
        "system_prompt",
    ] {
        assert!(
            CONFIG_TYPES_SOURCE.contains(needed),
            "Agent Chat profile config must keep legacy ai key/source `{needed}`"
        );
    }

    for needed in [
        "pub enum AgentChatBackend",
        "selected_profile_id",
        "selected_backend",
        "provider",
        "append_system_prompt",
        "disable_extensions",
        "disable_skills",
        "disable_prompt_templates",
    ] {
        assert!(
            CONFIG_TYPES_SOURCE.contains(needed),
            "Agent Chat profile config must expose Pi extension `{needed}`"
        );
    }
}

// (Removed: legacy per-agent preference load/persist tests — the Agent Chat backend
// was removed, so all sessions use the Pi backend with no agent preference.)

// ── Catalog loader classifies built-in agents ─────────────────────────

#[test]
fn catalog_loader_classifies_builtin_agents() {
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("AgentChatAgentSource::BuiltIn"),
        "catalog loader must classify built-in Agent Chat agents distinctly"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("classify_agent_source"),
        "catalog loader should centralize source classification"
    );
}

// ── Built-in classification distinguishes legacy Claude from built-ins ─

#[test]
fn classify_agent_source_distinguishes_legacy_from_builtin() {
    // The classifier must assign LegacyClaudeCode to "claude-code" and
    // BuiltIn to the well-known auto-detected agents.
    assert!(
        AGENT_CHAT_CONFIG_SOURCE
            .contains(r#""claude-code" => super::catalog::AgentChatAgentSource::LegacyClaudeCode"#),
        "claude-code must be classified as LegacyClaudeCode"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains(
            r#""opencode" | "codex-agent_chat" => super::catalog::AgentChatAgentSource::BuiltIn"#
        ),
        "opencode and codex-agent_chat must be classified as BuiltIn"
    );
}

// ── Codex Agent Chat appears in the selectable catalog ───────────────────────

#[test]
fn catalog_loader_merges_codex_starter_for_agent_selector() {
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("merge_catalog_with_starter_agents(&mut file)"),
        "catalog loading must merge starter Agent Chat agents so Codex appears in the selector"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("codex_agent_chat_agent_config()"),
        "starter Agent Chat agents must include the Codex Agent Chat adapter config"
    );
}

#[test]
fn codex_detection_uses_local_codex_cli_not_adapter_binary_only() {
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("codex_agent_chat_default_probe_state"),
        "catalog loading must expose a Codex default probe"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("command_exists(\"codex\")"),
        "Codex diagnostics must probe the local codex CLI"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("npx_runtime_fallback_enabled: false"),
        "Codex must record that npx runtime fallback is disabled"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("let launch_ready = adapter_ready && codex_cli_ready;")
            && AGENT_CHAT_CONFIG_SOURCE.contains("should_be_implicit_codex_default: launch_ready"),
        "Codex launch readiness must require both the resolved adapter and local codex CLI, \
         never npx or codex CLI alone"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("sibling_repo_codex_agent_chat_search_roots")
            && AGENT_CHAT_CONFIG_SOURCE.contains("std::env::current_exe()")
            && AGENT_CHAT_CONFIG_SOURCE.contains("std::env::current_dir()")
            && AGENT_CHAT_CONFIG_SOURCE.contains("parent_name == Some(\"target-agent\")"),
        "Codex adapter discovery must work in the launched app, where CARGO_MANIFEST_DIR is \
         not a reliable source of the sibling dev checkout"
    );
    assert!(
        !AGENT_CHAT_CONFIG_SOURCE.contains(
            "if command_exists(\"codex-agent_chat\") && !agents.iter().any(|a| a.id == \"codex-agent_chat\")"
        ),
        "Codex must not be discoverable only when codex-agent_chat itself is on PATH"
    );
}

#[test]
fn codex_adapter_discovery_excludes_app_bundled_binary() {
    assert!(
        !AGENT_CHAT_CONFIG_SOURCE.contains("BundledApp"),
        "Codex Agent Chat adapter discovery must not treat the app bundle as an adapter source"
    );
    assert!(
        !AGENT_CHAT_CONFIG_SOURCE.contains("bundled_codex_agent_chat_path"),
        "Codex Agent Chat adapter discovery must not probe Contents/MacOS/codex-agent_chat"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("EnvOverride")
            && AGENT_CHAT_CONFIG_SOURCE.contains("SiblingRepo")
            && AGENT_CHAT_CONFIG_SOURCE.contains("RepoLocal")
            && AGENT_CHAT_CONFIG_SOURCE.contains("Path"),
        "Codex Agent Chat adapter discovery must keep external env, dev, and PATH adapter sources"
    );
}

#[test]
fn release_workflow_does_not_embed_or_sign_codex_agent_chat() {
    for forbidden in [
        "CODEX_AGENT_CHAT_VERSION",
        "Fetch and embed codex-agent_chat binary",
        "scripts/fetch-codex-agent_chat.sh",
        "Contents/MacOS/codex-agent_chat",
    ] {
        assert!(
            !RELEASE_WORKFLOW_SOURCE.contains(forbidden),
            "release workflow must not contain bundled codex-agent_chat hook: {forbidden}"
        );
    }
}

#[test]
fn bundle_verifier_rejects_extra_macos_payloads() {
    assert!(
        VERIFY_MACOS_BUNDLE_SOURCE.contains("EXPECTED_BIN="),
        "bundle verifier must still require the main executable"
    );
    assert!(
        !VERIFY_MACOS_BUNDLE_SOURCE.contains("CODEX_AGENT_CHAT_BIN"),
        "bundle verifier must not require a codex-agent_chat executable"
    );
    assert!(
        VERIFY_MACOS_BUNDLE_SOURCE.contains("! -name 'script-kit-gpui'")
            && !VERIFY_MACOS_BUNDLE_SOURCE.contains("! -name 'codex-agent_chat'"),
        "bundle verifier must allow only the main executable in Contents/MacOS"
    );
}

#[test]
fn codex_setup_normalizes_stale_absolute_adapter_paths() {
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("looks_like_codex_agent_chat_adapter_command"),
        "Codex catalog normalization must recognize adapter commands by basename"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains(".file_name()"),
        "absolute stale codex-agent_chat paths must be detected by filename"
    );
    assert!(
        AGENT_CHAT_CONFIG_TESTS_SOURCE
            .contains("fn missing_adapter_does_not_normalize_to_npx_runtime()"),
        "config tests must pin stale absolute codex-agent_chat migration away from npx"
    );
}

// ── Catalog builder emits per-entry structured logs ──────────────────

#[test]
fn catalog_builder_emits_per_entry_logs() {
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("agent_chat_agent_catalog_entry_built"),
        "catalog builder must emit per-entry structured log events"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("install_state = ?install_state"),
        "per-entry log must include install_state"
    );
    assert!(
        AGENT_CHAT_CONFIG_SOURCE.contains("config_state = ?config_state"),
        "per-entry log must include config_state"
    );
}

// ── AgentChatThreadInit carries selected_agent ─────────────────────────────

const AGENT_CHAT_THREAD_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/thread.rs");
const SETUP_STATE_SOURCE: &str = include_str!("../src/ai/agent_chat/ui/setup_state.rs");

#[test]
fn agent_chat_thread_init_includes_selected_agent() {
    assert!(
        AGENT_CHAT_THREAD_SOURCE
            .contains("pub selected_agent: Option<super::catalog::AgentChatAgentCatalogEntry>"),
        "AgentChatThreadInit must carry the selected agent catalog entry"
    );
}

// ── Runtime SetupRequired arms inline setup state ────────────────────

#[test]
fn runtime_setup_required_arms_inline_setup_state() {
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("from_runtime_setup_required"),
        "AgentChatThread must convert SetupRequired events into inline setup state"
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("agent_chat_runtime_setup_session_armed"),
        "runtime setup recovery must be logged"
    );
}

#[test]
fn setup_state_has_runtime_constructor() {
    assert!(
        SETUP_STATE_SOURCE.contains("fn from_runtime_setup_required"),
        "AgentChatInlineSetupState must have from_runtime_setup_required constructor"
    );
    assert!(
        SETUP_STATE_SOURCE.contains("auth_required"),
        "runtime setup constructor must handle auth_required reason"
    );
}

// ── View renders runtime setup state ─────────────────────────────────

// ── Launch requirements threaded into thread init ────────────────────

#[test]
fn agent_chat_thread_init_carries_launch_requirements() {
    assert!(
        AGENT_CHAT_THREAD_SOURCE
            .contains("pub launch_requirements: super::preflight::AgentChatLaunchRequirements"),
        "AgentChatThreadInit must carry launch_requirements field"
    );
}

// ── Runtime recovery preserves launch requirements ──────────────────

#[test]
fn runtime_setup_required_preserves_launch_requirements() {
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("self.launch_requirements"),
        "runtime setup recovery must preserve launch requirements instead of resetting to default"
    );
    assert!(
        AGENT_CHAT_THREAD_SOURCE.contains("agent_chat_runtime_setup_requirements_preserved"),
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
        SETUP_STATE_SOURCE.contains("agent_chat_setup_state_from_resolution"),
        "from_resolution must emit agent_chat_setup_state_from_resolution log event"
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

// ── Setup picker confirmation updates live thread ───────────────────

// (Removed: legacy Agent Chat agent-preference persistence tests — the per-agent
// preference load/persist flow was deleted when the Agent Chat backend was removed.
// All sessions now use the Pi backend with provider/model selection.)
