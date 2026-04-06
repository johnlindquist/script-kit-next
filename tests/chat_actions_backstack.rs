// Integration tests for the chat actions back-stack contract.
//
// Verifies that:
// - The root route contains "Change Model" (not flat model rows)
// - get_chat_context_actions and get_chat_model_picker_actions are distinct
// - Model-level actions only exist in the picker, not at root level

use std::fs;

/// The chat builder must produce a "Change Model" action
/// instead of flat model selection rows at the top level.
#[test]
fn chat_root_route_contains_change_model_not_flat_models() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat.rs builders");

    assert!(
        content.contains("\"chat:change_model\""),
        "Root route must use the chat:change_model action ID"
    );
    assert!(
        content.contains("get_chat_context_actions"),
        "Root action builder must exist"
    );
    assert!(
        content.contains("get_chat_model_picker_actions"),
        "Model picker action builder must exist"
    );
}

/// get_chat_context_actions must not produce flat chat:select_model_* rows.
/// Those live exclusively in get_chat_model_picker_actions.
#[test]
fn root_builder_has_no_flat_model_rows() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat.rs builders");

    // Find get_chat_context_actions function body
    let fn_start = content
        .find("pub fn get_chat_context_actions")
        .expect("get_chat_context_actions must exist");
    // Find end: next pub fn or end of file
    let fn_end = content[fn_start + 30..]
        .find("\npub fn ")
        .map(|offset| fn_start + 30 + offset)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    // Must NOT contain select_model pattern in the root builder
    assert!(
        !fn_body.contains("chat:select_model_"),
        "get_chat_context_actions must not produce flat chat:select_model_* rows"
    );

    // Must contain change_model
    assert!(
        fn_body.contains("chat:change_model"),
        "get_chat_context_actions must include chat:change_model"
    );
}

/// get_chat_model_picker_actions must produce chat:select_model_* rows.
#[test]
fn picker_builder_produces_model_rows() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat.rs builders");

    let fn_start = content
        .find("pub fn get_chat_model_picker_actions")
        .expect("get_chat_model_picker_actions must exist");
    let fn_end = content[fn_start + 30..]
        .find("\npub fn ")
        .or_else(|| content[fn_start + 30..].find("\nfn "))
        .map(|offset| fn_start + 30 + offset)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    assert!(
        fn_body.contains("chat:select_model_"),
        "get_chat_model_picker_actions must produce chat:select_model_* rows"
    );
}

/// Both builders must share validation via is_chat_prompt_info_valid.
#[test]
fn both_builders_share_validation() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat.rs builders");

    assert!(
        content.contains("fn is_chat_prompt_info_valid"),
        "Shared validator must exist"
    );

    // get_chat_context_actions uses shared validator
    let ctx_start = content
        .find("pub fn get_chat_context_actions")
        .expect("get_chat_context_actions must exist");
    let ctx_end = content[ctx_start + 30..]
        .find("\npub fn ")
        .map(|offset| ctx_start + 30 + offset)
        .unwrap_or(content.len());
    let ctx_body = &content[ctx_start..ctx_end];
    assert!(
        ctx_body.contains("is_chat_prompt_info_valid"),
        "get_chat_context_actions must use shared validation"
    );

    // get_chat_model_picker_actions uses shared validator
    let picker_start = content
        .find("pub fn get_chat_model_picker_actions")
        .expect("get_chat_model_picker_actions must exist");
    let picker_end = content[picker_start + 30..]
        .find("\npub fn ")
        .or_else(|| content[picker_start + 30..].find("\nfn "))
        .map(|offset| picker_start + 30 + offset)
        .unwrap_or(content.len());
    let picker_body = &content[picker_start..picker_end];
    assert!(
        picker_body.contains("is_chat_prompt_info_valid"),
        "get_chat_model_picker_actions must use shared validation"
    );
}

/// Validation warnings must be emitted by the shared validator,
/// not duplicated across builders.
#[test]
fn validation_emits_warnings_for_invalid_inputs() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat.rs builders");

    // Find is_chat_prompt_info_valid
    let fn_start = content
        .find("fn is_chat_prompt_info_valid")
        .expect("is_chat_prompt_info_valid must exist");
    let fn_end = content[fn_start + 20..]
        .find("\n}")
        .map(|offset| fn_start + 20 + offset + 2)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    assert!(
        fn_body.contains("tracing::warn!"),
        "Validator must emit tracing warnings for invalid inputs"
    );
    assert!(
        fn_body.contains("current model name is blank"),
        "Validator must warn about blank current model"
    );
    assert!(
        fn_body.contains("model metadata missing required fields"),
        "Validator must warn about invalid model metadata"
    );
}

/// The builders module must re-export both builder functions.
#[test]
fn builders_module_exports_both_functions() {
    let content =
        fs::read_to_string("src/actions/builders.rs").expect("Failed to read builders.rs");

    assert!(
        content.contains("get_chat_context_actions"),
        "builders.rs must re-export get_chat_context_actions"
    );
    assert!(
        content.contains("get_chat_model_picker_actions"),
        "builders.rs must re-export get_chat_model_picker_actions"
    );
}

// ── Route builder contract tests ────────────────────────────────────────

/// get_chat_root_route must produce a route with the correct ID and
/// initial selection on the Change Model action.
#[test]
fn chat_root_route_has_correct_id_and_initial_selection() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat.rs builders");

    assert!(
        content.contains("CHAT_ROOT_ROUTE_ID"),
        "Chat root route ID constant must exist"
    );
    assert!(
        content.contains("\"chat:root\""),
        "Chat root route ID must be 'chat:root'"
    );

    let fn_start = content
        .find("pub fn get_chat_root_route")
        .expect("get_chat_root_route must exist");
    let fn_end = content[fn_start + 20..]
        .find("\npub fn ")
        .or_else(|| content[fn_start + 20..].find("\npub const "))
        .map(|offset| fn_start + 20 + offset)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    assert!(
        fn_body.contains("CHAT_ROOT_ROUTE_ID"),
        "get_chat_root_route must use the CHAT_ROOT_ROUTE_ID constant"
    );
    assert!(
        fn_body.contains("CHAT_CHANGE_MODEL_ACTION_ID"),
        "get_chat_root_route must set initial selection to the Change Model action"
    );
}

/// get_chat_model_picker_route must produce a route with the correct ID
/// and initial selection on the current model.
#[test]
fn chat_model_picker_route_has_correct_id_and_initial_selection() {
    let content = fs::read_to_string("src/actions/builders/chat.rs")
        .expect("Failed to read chat.rs builders");

    assert!(
        content.contains("CHAT_MODEL_PICKER_ROUTE_ID"),
        "Chat model picker route ID constant must exist"
    );
    assert!(
        content.contains("\"chat:model_picker\""),
        "Chat model picker route ID must be 'chat:model_picker'"
    );

    let fn_start = content
        .find("pub fn get_chat_model_picker_route")
        .expect("get_chat_model_picker_route must exist");
    let fn_end = content[fn_start + 20..]
        .find("\npub fn ")
        .or_else(|| content[fn_start + 20..].find("\npub const "))
        .map(|offset| fn_start + 20 + offset)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    assert!(
        fn_body.contains("CHAT_MODEL_PICKER_ROUTE_ID"),
        "get_chat_model_picker_route must use the CHAT_MODEL_PICKER_ROUTE_ID constant"
    );
    assert!(
        fn_body.contains("initial_selected_action_id"),
        "get_chat_model_picker_route must set initial_selected_action_id"
    );
    assert!(
        fn_body.contains("chat:select_model_"),
        "get_chat_model_picker_route must derive initial selection from model ID"
    );
}

/// builders.rs must re-export the route builder functions and constants.
#[test]
fn builders_module_exports_route_builders() {
    let content =
        fs::read_to_string("src/actions/builders.rs").expect("Failed to read builders.rs");

    assert!(
        content.contains("get_chat_root_route"),
        "builders.rs must re-export get_chat_root_route"
    );
    assert!(
        content.contains("get_chat_model_picker_route"),
        "builders.rs must re-export get_chat_model_picker_route"
    );
    assert!(
        content.contains("CHAT_CHANGE_MODEL_ACTION_ID"),
        "builders.rs must re-export CHAT_CHANGE_MODEL_ACTION_ID"
    );
}

/// with_chat() must use the route system (set_root_route + register_drill_down_route),
/// not a flat action list.
#[test]
fn with_chat_uses_route_system() {
    let content = fs::read_to_string("src/actions/dialog.rs").expect("Failed to read dialog.rs");

    let fn_start = content
        .find("pub fn with_chat(")
        .expect("with_chat must exist");
    let fn_end = content[fn_start + 10..]
        .find("\n    pub ")
        .map(|offset| fn_start + 10 + offset)
        .unwrap_or(content.len());
    let fn_body = &content[fn_start..fn_end];

    assert!(
        fn_body.contains("set_root_route"),
        "with_chat must call set_root_route to initialize the route stack"
    );
    assert!(
        fn_body.contains("register_drill_down_route"),
        "with_chat must register the model picker as a drill-down route"
    );
    assert!(
        fn_body.contains("get_chat_root_route"),
        "with_chat must use get_chat_root_route for the root route"
    );
    assert!(
        fn_body.contains("get_chat_model_picker_route"),
        "with_chat must use get_chat_model_picker_route for the drill-down"
    );
    assert!(
        fn_body.contains("CHAT_CHANGE_MODEL_ACTION_ID"),
        "with_chat must register drill-down on CHAT_CHANGE_MODEL_ACTION_ID"
    );
}
