use script_kit_gpui::test_utils::read_source;

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"));
    let after_start = &source[start_index..];
    let end_index = after_start
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"));
    &after_start[..end_index]
}

#[test]
fn naming_prompt_scripts_write_receipt_after_template_body_before_editor() {
    let source = read_source("src/app_impl/naming_dialog.rs");
    let body = source_between(
        &source,
        "match create_result {\n            Ok(path) => {",
        "self.open_creation_feedback_payload(",
    );

    let template_write = body
        .find("render_script_template_file")
        .expect("script-template body should be rendered before the editor opens");
    let receipt_write = body
        .find("write_script_creation_receipt_for_path")
        .expect("script creation must write a generated-script receipt");
    let editor_open = body
        .find("script_creation::open_in_editor")
        .expect("created script should still open in the editor");

    assert!(
        template_write < receipt_write && receipt_write < editor_open,
        "receipt must describe the final template-written file before editor/CreationFeedback"
    );
    assert!(
        source.contains("self.open_creation_feedback_payload("),
        "script creation should continue to hand off to CreationFeedback"
    );
}

#[test]
fn receipt_plumbing_is_script_only_and_scriptlets_stay_unverified() {
    let source = read_source("src/app_impl/naming_dialog.rs");
    let body = source_between(
        &source,
        "let create_result = match result.target {",
        "self.open_creation_feedback_payload(",
    );

    let receipt_guard = "if result.target == prompts::NamingTarget::Script";
    let first_guard = body
        .find(receipt_guard)
        .expect("template overwrite must be guarded to NamingTarget::Script");
    let after_first = &body[first_guard + receipt_guard.len()..];
    let second_guard = after_first
        .find(receipt_guard)
        .expect("receipt plumbing must have its own NamingTarget::Script guard");
    let template_arm = &after_first[..second_guard];
    let receipt_arm = &after_first[second_guard..];
    assert!(
        template_arm.contains("find_script_template"),
        "first NamingTarget::Script guard must own the template-overwrite plumbing"
    );
    assert!(
        receipt_arm.contains("write_script_creation_receipt_for_path"),
        "second NamingTarget::Script guard must own the receipt plumbing"
    );
    assert!(
        body.contains("prompts::NamingTarget::Extension => script_creation::create_new_scriptlet"),
        "extension/scriptlet creation should keep the existing create_new_scriptlet path"
    );
    let extension_arm = source_between(
        body,
        "prompts::NamingTarget::Extension => script_creation::create_new_scriptlet",
        "};",
    );
    assert!(
        !extension_arm.contains("write_script_creation_receipt_for_path"),
        "scriptlet creation must not pretend to have TypeScript generated-script verification"
    );
}

#[test]
fn script_creation_receipts_use_existing_generated_script_receipt_schema() {
    let generator = read_source("src/ai/script_generation.rs");
    let helper = source_between(
        &generator,
        "pub(crate) fn write_script_creation_receipt_for_path(",
        "pub fn extract_current_app_recipe_from_script(",
    );

    for required in [
        "GeneratedScriptReceipt",
        "AI_GENERATED_SCRIPT_RECEIPT_SCHEMA_VERSION",
        "audit_generated_script_contract(&source)",
        "verify_generated_script_with_bun_build(script_path)",
        "generated_script_receipt_path(script_path)",
        "write_generated_script_receipt(&receipt_path, &receipt)?",
        "current_app_recipe: None",
    ] {
        assert!(
            helper.contains(required),
            "script-creation receipt helper must reuse existing receipt contract: {required}"
        );
    }
    assert!(
        helper.contains("file_stem()"),
        "receipt slug must derive from the actual created file stem to preserve collision suffixes"
    );
}

#[test]
fn generated_script_build_verification_externalizes_scriptkit_sdk() {
    let generator = read_source("src/ai/script_generation.rs");
    let verifier = source_between(
        &generator,
        "fn verify_generated_script_with_bun_build(",
        "pub(crate) fn write_script_creation_receipt_for_path(",
    );

    assert!(
        verifier.contains("\"--external\".to_string()")
            && verifier.contains("SCRIPT_KIT_SDK_IMPORT_MODULE.to_string()"),
        "receipt verification command must externalize @scriptkit/sdk"
    );
    assert!(
        verifier.contains(".arg(\"--external\")")
            && verifier.contains(".arg(SCRIPT_KIT_SDK_IMPORT_MODULE)"),
        "spawned verification command must pass the externalization args"
    );
}
