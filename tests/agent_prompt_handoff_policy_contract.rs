//! Source contracts for Agent Chat prompt handoff policy.
//!
//! These keep the handoff action aligned with the normal Agent Chat prompt-builder
//! submit gate while the broader equivalence matrix is built out.

#[test]
fn handoff_blocks_non_submittable_prompt_builder_plans() {
    let source = include_str!("../src/ai/agent_prompt_handoff.rs");
    let method = source
        .split("pub(crate) fn compile_handoff_payload_from_spine_plan")
        .nth(1)
        .and_then(|tail| tail.split("fn launch_cmux_codex").next())
        .expect("compile_handoff_payload_from_spine_plan body");

    for expected in [
        "plan.prompt_builder_segment_count > 0 && !plan.should_submit_to_chat()",
        "AgentPromptHandoffError::UnsupportedPrompt",
        "prompt builder input is not submittable",
    ] {
        assert!(
            method.contains(expected),
            "handoff must block prompt-builder drafts that normal Agent Chat prompt-builder submit would not submit: {expected}"
        );
    }
}

#[test]
fn handoff_preserves_plain_text_composer_fallback() {
    let source = include_str!("../src/ai/agent_prompt_handoff.rs");
    let method = source
        .split("pub(crate) fn compile_handoff_payload_from_spine_plan")
        .nth(1)
        .and_then(|tail| tail.split("fn launch_cmux_codex").next())
        .expect("compile_handoff_payload_from_spine_plan body");

    assert!(
        method.contains("if plan.prompt_builder_segment_count > 0")
            && method.contains("raw_input.trim().to_string()"),
        "plain Agent Chat composer text has no prompt-builder segments and must still hand off as raw prompt text"
    );
}

#[test]
fn agent_chat_view_uses_shared_handoff_compiler() {
    let source = include_str!("../src/ai/agent_chat/ui/view.rs");
    let method = source
        .split("pub(crate) fn current_prompt_handoff_payload")
        .nth(1)
        .and_then(|tail| tail.split("fn handle_agent_chat_spine_key_down").next())
        .expect("current_prompt_handoff_payload body");

    assert!(
        method.contains("compile_handoff_payload_from_spine_plan"),
        "Agent Chat view should delegate payload semantics to the shared handoff compiler"
    );
}

#[test]
fn prompt_targets_are_config_backed_actions_and_shortcut_command_ids() {
    let handoff = include_str!("../src/ai/agent_prompt_handoff.rs");
    let actions = include_str!("../src/actions/builders/script_context.rs");
    let execution = include_str!("../src/app_impl/execution_scripts.rs");
    let command_ids = include_str!("../src/config/command_ids.rs");
    let config_types = include_str!("../src/config/types.rs");
    let schema = include_str!("../scripts/config-schema.ts");
    let sdk = include_str!("../scripts/kit-sdk.ts");

    assert!(
        handoff.contains("PROMPT_TARGET_ACTION_PREFIX")
            && handoff.contains("\"prompt-target/\"")
            && handoff.contains("configured_prompt_targets"),
        "handoff layer must resolve prompt-target/<id> actions from config"
    );
    assert!(
        actions.contains("get_prompt_target_actions")
            && actions.contains("all_prompt_targets")
            && actions.contains("get_command_shortcut"),
        "Actions menu must surface prompt targets and display config.commands shortcuts"
    );
    assert!(
        execution.contains("CommandCategory::PromptTarget")
            && execution.contains("launch_prompt_target_from_main_prompt"),
        "global command-id execution must route prompt-target shortcuts through the main prompt handoff path"
    );
    assert!(
        command_ids.contains("PromptTarget") && command_ids.contains("\"prompt-target\""),
        "command id parser must accept prompt-target/<id> ids"
    );
    assert!(
        config_types.contains("PromptTargetConfig")
            && config_types.contains("prompt_targets")
            && config_types.contains("rename = \"promptTargets\""),
        "Rust config must load promptTargets"
    );
    assert!(
        schema.contains("PromptTargetConfig")
            && schema.contains("PromptTargetCommandId")
            && schema.contains("\"prompt-target\""),
        "config-schema.ts must document and validate prompt-target command ids"
    );
    assert!(
        sdk.contains("PromptTargetConfig")
            && sdk.contains("PromptTargetCommandId")
            && sdk.contains("promptTargets?: Record<string, PromptTargetConfig>"),
        "kit-sdk.ts must expose promptTargets and prompt-target command ids"
    );
}

#[test]
fn prompt_export_actions_are_shortcut_command_ids() {
    let handoff = include_str!("../src/ai/agent_prompt_handoff.rs");
    let actions = include_str!("../src/actions/builders/script_context.rs");
    let execution = include_str!("../src/app_impl/execution_scripts.rs");
    let command_ids = include_str!("../src/config/command_ids.rs");
    let schema = include_str!("../scripts/config-schema.ts");
    let sdk = include_str!("../scripts/kit-sdk.ts");

    for expected in [
        "PROMPT_ACTION_PREFIX",
        "EXPORT_FILE_ACTION_ID",
        "EXPORT_GIST_ACTION_ID",
        "COPY_PROMPT_ACTION_ID",
        "export_prompt",
    ] {
        assert!(
            handoff.contains(expected),
            "handoff layer must expose built prompt export actions: {expected}"
        );
    }
    assert!(
        actions.contains("get_prompt_export_actions")
            && actions.contains("builtin_prompt_actions")
            && actions.contains("get_command_shortcut"),
        "Actions menu must surface prompt export actions and display config.commands shortcuts"
    );
    assert!(
        execution.contains("CommandCategory::PromptAction")
            && execution.contains("export_prompt_from_main_prompt"),
        "global command-id execution must route prompt-action shortcuts through the main prompt export path"
    );
    assert!(
        command_ids.contains("PromptAction") && command_ids.contains("\"prompt-action\""),
        "command id parser must accept prompt-action/<id> ids"
    );
    assert!(
        schema.contains("PromptActionCommandId") && schema.contains("\"prompt-action\""),
        "config-schema.ts must document and validate prompt-action command ids"
    );
    assert!(
        sdk.contains("PromptActionCommandId")
            && sdk.contains("prompt-action/export-file")
            && sdk.contains("prompt-action/export-gist")
            && sdk.contains("prompt-action/copy-prompt"),
        "kit-sdk.ts must expose prompt-action command ids"
    );
}

#[test]
fn main_prompt_actions_use_shared_handoff_compiler() {
    let source = include_str!("../src/app_actions/handle_action/mod.rs");
    let method = source
        .split("fn current_main_prompt_handoff_payload")
        .nth(1)
        .and_then(|tail| {
            tail.split("pub(crate) fn launch_prompt_target_from_main_prompt")
                .next()
        })
        .expect("current_main_prompt_handoff_payload body");

    for expected in [
        "filter_text().to_string()",
        "set_spine_parse_from_filter_and_cursor",
        "build_spine_prompt_plan",
        "spine_cwd_for_agent_chat_launch",
        "compile_handoff_payload_from_spine_plan",
    ] {
        assert!(
            method.contains(expected),
            "main prompt handoff must share Agent Chat prompt-builder semantics: {expected}"
        );
    }
}

#[test]
fn prompt_target_actions_bypass_pre_dispatch_actions_dialog_transition() {
    let source = include_str!("../src/app_actions/handle_action/mod.rs");
    let method = source
        .split("fn handle_action(&mut self, action_id: String")
        .nth(1)
        .and_then(|tail| tail.split("fn log_dispatch_outcome").next())
        .expect("handle_action body");

    for expected in [
        "let is_prompt_action",
        "is_prompt_action_id(&action_id_stripped)",
        "&& !is_prompt_action",
    ] {
        assert!(
            method.contains(expected),
            "prompt-target and prompt-action actions must dispatch before ActionsDialog is normalized back to ScriptList: {expected}"
        );
    }
}
