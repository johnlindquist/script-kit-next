//! Source-audit tests that freeze the current-app route → trace → recipe → prompt
//! contract.  These are deliberately source-scanning tests (not unit tests) so
//! they break loudly when internal wiring is refactored without updating the
//! contract.

use script_kit_gpui::test_utils::read_source;

/// Return the substring of `source` starting at the first occurrence of
/// `needle`, or panic with a structured message naming the invariant.
fn slice_from<'a>(source: &'a str, needle: &str) -> &'a str {
    let idx = source
        .find(needle)
        .unwrap_or_else(|| panic!("expected to find '{needle}' in src/menu_bar/current_app_commands.rs"));
    &source[idx..]
}

// ---------------------------------------------------------------------------
// 1. Recipe builder wires real prompt inputs through to generation
// ---------------------------------------------------------------------------

#[test]
fn recipe_builder_generates_prompt_from_real_inputs() {
    let source = read_source("src/menu_bar/current_app_commands.rs");
    let body = slice_from(&source, "pub fn build_current_app_command_recipe(");

    // The recipe builder must call the snapshot prompt builder with
    // selected_text and browser_url so the generated prompt reflects live
    // desktop context — not just menu entries.
    assert!(
        body.contains("build_generate_script_prompt_from_snapshot(snapshot, request, selected_text, browser_url)"),
        "recipe builder must generate the real prompt from selected_text and browser_url"
    );

    tracing::info!(
        anchor = "build_current_app_command_recipe",
        invariant = "route",
        "audited: recipe builder calls prompt generator with real context inputs"
    );
}

// ---------------------------------------------------------------------------
// 2. Recipe builder keeps nested trace aligned with actual prompt
// ---------------------------------------------------------------------------

#[test]
fn recipe_builder_aligns_trace_with_actual_prompt() {
    let source = read_source("src/menu_bar/current_app_commands.rs");
    let body = slice_from(&source, "pub fn build_current_app_command_recipe(");

    // Only the generate_script action gets prompt alignment — other actions
    // (ExecuteEntry, OpenCommandPalette) have no prompt to align.
    assert!(
        body.contains("if trace.action == \"generate_script\" {"),
        "recipe builder must only overwrite nested trace prompt fields for generate_script traces"
    );
    assert!(
        body.contains("trace.prompt_receipt = Some(prompt_receipt.clone());"),
        "nested trace prompt_receipt must stay aligned with the real recipe prompt_receipt"
    );
    assert!(
        body.contains("trace.prompt_preview = Some(prompt.clone());"),
        "nested trace prompt_preview must stay aligned with the real recipe prompt"
    );

    tracing::info!(
        anchor = "build_current_app_command_recipe",
        invariant = "trace",
        "audited: trace prompt fields are aligned with recipe prompt for generate_script"
    );
}

// ---------------------------------------------------------------------------
// 3. Trace builder has explicit GenerateScript branch with prompt preview
// ---------------------------------------------------------------------------

#[test]
fn trace_builder_has_generate_script_branch_with_prompt_preview() {
    let source = read_source("src/menu_bar/current_app_commands.rs");
    let body = slice_from(&source, "pub fn build_current_app_intent_trace_receipt(");

    assert!(
        body.contains("DoInCurrentAppAction::GenerateScript => {"),
        "trace builder must have an explicit GenerateScript branch"
    );
    // The trace builder generates a deterministic prompt preview by calling
    // the snapshot prompt builder with None for selected_text/browser_url.
    assert!(
        body.contains("build_generate_script_prompt_from_snapshot(snapshot, request, None, None)"),
        "trace builder must generate a deterministic prompt preview for the script-generation fallback"
    );

    tracing::info!(
        anchor = "build_current_app_intent_trace_receipt",
        invariant = "recipe",
        "audited: trace builder generates deterministic prompt preview for GenerateScript"
    );
}

// ---------------------------------------------------------------------------
// 4. Generated script prompt embeds round-trip recipe headers
// ---------------------------------------------------------------------------

#[test]
fn generated_script_prompt_embeds_roundtrip_recipe_headers() {
    let source = read_source("src/menu_bar/current_app_commands.rs");
    let body = slice_from(&source, "pub fn build_generated_script_prompt_from_recipe(");

    assert!(
        body.contains("// Current-App-Recipe-Base64:"),
        "generated prompt must embed a base64 recipe header for replay and audit"
    );
    assert!(
        body.contains("// Current-App-Recipe-Name:"),
        "generated prompt must embed a stable recipe-name header"
    );

    tracing::info!(
        anchor = "build_generated_script_prompt_from_recipe",
        invariant = "prompt",
        "audited: generated prompt embeds base64 recipe and name headers"
    );
}
