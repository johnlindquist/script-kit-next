use script_kit_gpui::test_utils::read_source;

/// The AI generation contract must not disagree with the default script template
/// about the canonical metadata shape. Both must accept `export const metadata = { name, ... }`.
#[test]
fn test_ai_generator_contract_does_not_disagree_with_default_script_template() {
    let generator = read_source("src/ai/script_generation.rs");
    let template = read_source("src/script_creation/mod.rs");

    let template_uses_metadata_export = template.contains("export const metadata = {");
    let generator_requires_only_comment_headers =
        generator.contains("FIRST line must be: // Name:")
            || generator.contains("SECOND line must be: // Description:");
    let generator_mentions_metadata_export = generator.contains("export const metadata");

    assert!(
        !template_uses_metadata_export || !generator_requires_only_comment_headers || generator_mentions_metadata_export,
        "AI generation contract disagrees with the default script template. \
         The template uses `export const metadata` but the generator only accepts comment headers. \
         Pick one canonical metadata format and make both files accept it."
    );

    // Positive check: if template uses metadata export, generator must mention it
    if template_uses_metadata_export {
        assert!(
            generator_mentions_metadata_export,
            "Template uses `export const metadata` but generator contract does not mention it"
        );
    }
}

/// The slug derivation path must have a fallback for metadata export names
/// so scripts using `export const metadata = { name: "..." }` get proper slugs.
#[test]
fn test_slug_derivation_supports_metadata_export_fallback() {
    let generator = read_source("src/ai/script_generation.rs");

    assert!(
        generator.contains("extract_metadata_name"),
        "prepare_script_from_ai_response must call extract_metadata_name as a fallback slug source"
    );

    // The slug resolution chain should be: comment header → metadata export → normalized prompt
    assert!(
        generator.contains("extract_name_comment")
            && generator.contains("extract_metadata_name"),
        "Slug derivation must try comment header first, then metadata export"
    );
}

/// The generator must emit structured log lines reporting which slug source won.
#[test]
fn test_slug_source_is_logged() {
    let generator = read_source("src/ai/script_generation.rs");

    assert!(
        generator.contains("slug_source_resolved"),
        "prepare_script_from_ai_response must emit a slug_source_resolved log line"
    );

    // Should report which source was used
    assert!(
        generator.contains("comment_header")
            && generator.contains("metadata_export")
            && generator.contains("normalized_prompt"),
        "slug_source_resolved log must identify the winning source: comment_header, metadata_export, or normalized_prompt"
    );
}

/// The system prompt must present both metadata formats as valid options,
/// with metadata export mentioned as preferred for new scripts.
#[test]
fn test_system_prompt_accepts_both_metadata_formats() {
    let generator = read_source("src/ai/script_generation.rs");

    // Must mention comment headers as valid
    assert!(
        generator.contains("// Name:") && generator.contains("// Description:"),
        "System prompt must still accept comment header metadata"
    );

    // Must mention metadata export as valid
    assert!(
        generator.contains("export const metadata"),
        "System prompt must accept metadata export format"
    );
}

/// Before/after contract markers: the generator instructions must bracket
/// the two accepted formats so an AI agent can verify which contract is active.
#[test]
fn test_generator_contract_has_format_markers() {
    let generator = read_source("src/ai/script_generation.rs");

    assert!(
        generator.contains("Format A") && generator.contains("Format B"),
        "Generator contract must label the two accepted metadata formats as Format A and Format B"
    );
}

// ---------------------------------------------------------------------------
// Receipt-oriented API exports: these must remain visible from src/ai/mod.rs
// so agentic tooling can consume receipts without reaching into internals.
// ---------------------------------------------------------------------------

/// The receipt API must be re-exported from the ai module facade.
#[test]
fn test_receipt_api_exports_visible_from_ai_module() {
    let mod_rs = read_source("src/ai/mod.rs");

    assert!(
        mod_rs.contains("generate_script_from_prompt_with_receipt"),
        "invariant_receipt_api_export: generate_script_from_prompt_with_receipt must be re-exported from src/ai/mod.rs"
    );
    assert!(
        mod_rs.contains("GeneratedScriptReceipt"),
        "invariant_receipt_type_export: GeneratedScriptReceipt must be re-exported from src/ai/mod.rs"
    );
    assert!(
        mod_rs.contains("GeneratedScriptContractAudit"),
        "invariant_contract_audit_export: GeneratedScriptContractAudit must be re-exported from src/ai/mod.rs"
    );
    assert!(
        mod_rs.contains("GeneratedScriptMetadataStyle"),
        "invariant_metadata_style_export: GeneratedScriptMetadataStyle must be re-exported from src/ai/mod.rs"
    );
    assert!(
        mod_rs.contains("AI_GENERATED_SCRIPT_RECEIPT_SCHEMA_VERSION"),
        "invariant_schema_version_export: AI_GENERATED_SCRIPT_RECEIPT_SCHEMA_VERSION must be re-exported from src/ai/mod.rs"
    );
    assert!(
        mod_rs.contains("generated_script_receipt_path"),
        "invariant_receipt_path_export: generated_script_receipt_path must be re-exported from src/ai/mod.rs"
    );
}

/// The generator must define contract audit types so receipts carry machine-readable diagnostics.
#[test]
fn test_generator_defines_contract_audit_infrastructure() {
    let generator = read_source("src/ai/script_generation.rs");

    assert!(
        generator.contains("GeneratedScriptContractAudit"),
        "invariant_contract_audit_struct: script_generation.rs must define GeneratedScriptContractAudit"
    );
    assert!(
        generator.contains("GeneratedScriptMetadataStyle"),
        "invariant_metadata_style_enum: script_generation.rs must define GeneratedScriptMetadataStyle"
    );
    assert!(
        generator.contains("audit_generated_script_contract"),
        "invariant_audit_function: script_generation.rs must define audit_generated_script_contract"
    );
    assert!(
        generator.contains("write_generated_script_receipt"),
        "invariant_receipt_writer: script_generation.rs must define write_generated_script_receipt"
    );
}

/// The receipt must include contract audit and slug provenance fields.
#[test]
fn test_receipt_struct_has_observable_fields() {
    let generator = read_source("src/ai/script_generation.rs");

    // Receipt must carry slug provenance for replay/audit
    assert!(
        generator.contains("slug_source_kind"),
        "invariant_receipt_slug_provenance: GeneratedScriptReceipt must include slug_source_kind"
    );
    assert!(
        generator.contains("slug_source"),
        "invariant_receipt_slug_source: GeneratedScriptReceipt must include slug_source"
    );

    // Receipt must carry contract audit
    assert!(
        generator.contains("contract: GeneratedScriptContractAudit")
            || generator.contains("contract: prepared.contract"),
        "invariant_receipt_contract_field: GeneratedScriptReceipt must include a contract field"
    );
}

/// Legacy save paths must also emit receipts so chat-driven saves do not bypass
/// the sidecar contract introduced for generated scripts.
#[test]
fn test_legacy_save_path_writes_receipt_sidecar() {
    let generator = read_source("src/ai/script_generation.rs");

    assert!(
        generator.contains("pub(crate) fn save_generated_script_from_response("),
        "invariant_legacy_save_exists: script_generation.rs must keep save_generated_script_from_response"
    );
    assert!(
        generator.contains("let receipt_path = generated_script_receipt_path(&script_path);"),
        "invariant_legacy_save_receipt_path: legacy save path must derive a receipt path beside the script"
    );
    assert!(
        generator.contains("model_id: \"unknown\".to_string()"),
        "invariant_legacy_save_model_provenance: legacy save path must record explicit unknown model provenance when none is available"
    );
    assert!(
        generator.contains("provider_id: \"unknown\".to_string()"),
        "invariant_legacy_save_provider_provenance: legacy save path must record explicit unknown provider provenance when none is available"
    );
    assert!(
        generator.contains("write_generated_script_receipt(&receipt_path, &receipt)?;"),
        "invariant_legacy_save_writes_receipt: legacy save path must persist the receipt sidecar"
    );
}
