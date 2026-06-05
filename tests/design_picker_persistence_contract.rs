//! Phase 4 — Design Picker persistence contract.
//!
//! Asserts the wiring contract between the in-memory preview helpers, the
//! commit helper, and the on-disk `DesignsConfig.activeId` write path. A
//! full GPUI startup harness is too heavy for a unit test, so this is a
//! source audit that pins the call sites exactly where the goal requires
//! them — preview-only paths must NOT write to disk, commit paths must.

use std::fs;

fn section<'a>(source: &'a str, start_needle: &str, end_needle: &str) -> &'a str {
    let start = source
        .find(start_needle)
        .unwrap_or_else(|| panic!("missing start marker `{start_needle}`"));
    let after = &source[start..];
    let end_rel = after
        .find(end_needle)
        .unwrap_or_else(|| panic!("missing end marker `{end_needle}`"));
    &after[..end_rel]
}

#[test]
fn design_picker_commit_paths_persist_but_preview_paths_do_not() {
    let source = fs::read_to_string("src/render_builtins/design_picker.rs")
        .expect("design_picker.rs must be readable");

    let preview_fn = section(
        &source,
        "fn preview_design_picker_id(",
        "fn restore_design_picker_original(",
    );
    assert!(
        !preview_fn.contains("save_active_design_id"),
        "preview_design_picker_id must remain preview-only (no config writes)"
    );
    assert!(
        !preview_fn.contains("save_user_preferences"),
        "preview_design_picker_id must not touch user-preferences write paths"
    );

    let filtered_preview_fn = section(
        &source,
        "fn preview_design_picker_filtered_index(",
        "fn design_picker_id_for_filtered_index(",
    );
    assert!(
        !filtered_preview_fn.contains("save_active_design_id"),
        "preview_design_picker_filtered_index must remain preview-only"
    );

    assert!(
        source.contains("fn persist_design_picker_selection("),
        "commit helper persist_design_picker_selection must exist"
    );
    assert!(
        source.contains("save_active_design_id("),
        "commit path must call save_active_design_id"
    );

    let submit_fn = section(
        &source,
        "fn submit_design_picker_from_input_enter(",
        "fn render_design_picker(",
    );
    assert!(
        submit_fn.contains("persist_design_picker_selection"),
        "Enter must commit through persist_design_picker_selection"
    );
    assert!(
        submit_fn.contains("design_picker_done"),
        "Enter commit must use the design_picker_done reason"
    );

    assert!(
        source.contains("design_picker_mouse_click"),
        "row click must use the design_picker_mouse_click reason"
    );

    let render_fn = &source[source
        .find("fn render_design_picker(")
        .expect("render_design_picker must exist")..];
    assert!(
        render_fn.contains("persist_design_picker_selection"),
        "row click handler must commit through persist_design_picker_selection"
    );
}

#[test]
fn startup_hydrates_design_from_config_instead_of_default_variant() {
    let startup =
        fs::read_to_string("src/app_impl/startup.rs").expect("startup.rs must be readable");
    let startup_new_state = fs::read_to_string("src/app_impl/startup_new_state.rs")
        .expect("startup_new_state.rs must be readable");

    assert!(
        startup.contains("fn current_design_from_config("),
        "startup must define current_design_from_config helper"
    );
    assert!(
        startup.contains("Self::current_design_from_config(&config)"),
        "startup ScriptListApp::new must hydrate current_design via current_design_from_config"
    );
    assert!(
        startup_new_state.contains("Self::current_design_from_config(&config)"),
        "startup_new_state must hydrate current_design via current_design_from_config"
    );

    assert!(
        !startup.contains("current_design: DesignVariant::default()"),
        "startup.rs must not seed current_design with DesignVariant::default()"
    );
    assert!(
        !startup_new_state.contains("current_design: DesignVariant::default()"),
        "startup_new_state.rs must not seed current_design with DesignVariant::default()"
    );
}

#[test]
fn save_active_design_id_helper_exists_and_is_exported() {
    let loader =
        fs::read_to_string("src/config/loader.rs").expect("config/loader.rs must be readable");
    assert!(
        loader.contains("pub fn save_active_design_id(id: &str)"),
        "save_active_design_id helper must exist with the expected signature"
    );
    assert!(
        loader.contains("CONFIG_PREFERENCE_WRITE_LOCK"),
        "save_active_design_id must reuse the preference write lock"
    );
    assert!(
        loader.contains("write_preference_group("),
        "save_active_design_id must persist via write_preference_group"
    );
    assert!(
        loader.contains("resolve_possibly_legacy_id"),
        "save_active_design_id must run input through legacy migration"
    );

    let module = fs::read_to_string("src/config/mod.rs").expect("config/mod.rs must be readable");
    assert!(
        module.contains("save_active_design_id"),
        "config module must re-export save_active_design_id"
    );
}

#[test]
fn cmd1_design_picker_honors_configured_behavior() {
    let source = fs::read_to_string("src/app_execute/builtin_execution.rs")
        .expect("builtin_execution.rs must be readable");

    let branch_start = source
        .find("builtins::BuiltInFeature::DesignPicker =>")
        .expect("DesignPicker dispatch arm must exist");
    let after = &source[branch_start..];
    // Skip past the arm head itself, then cut before the next BuiltInFeature variant
    // (the next match arm). Storybook's variant is gated by a #[cfg] attribute so we
    // anchor on the literal arm head to stay robust to those attributes.
    let body = &after[64..];
    let next = body
        .find("builtins::BuiltInFeature::")
        .or_else(|| body.find("BuiltInFeature::"))
        .unwrap_or(body.len());
    let branch = &after[..64 + next];

    assert!(
        branch.contains("effective_cmd1_behavior"),
        "DesignPicker dispatch must consult effective_cmd1_behavior"
    );
    assert!(
        branch.contains("Cmd1Behavior::Picker"),
        "DesignPicker dispatch must handle the Picker arm"
    );
    assert!(
        branch.contains("Cmd1Behavior::Cycle"),
        "DesignPicker dispatch must handle the Cycle arm"
    );
    assert!(
        branch.contains("open_design_picker_view"),
        "Picker arm must call open_design_picker_view"
    );
    assert!(
        branch.contains("cycle_design"),
        "Cycle arm must call cycle_design"
    );
    assert!(
        branch.contains("save_active_design_id"),
        "Cycle arm must persist via save_active_design_id"
    );
}

#[test]
fn save_design_overrides_helper_exists_and_is_exported() {
    let loader =
        fs::read_to_string("src/config/loader.rs").expect("config/loader.rs must be readable");
    assert!(
        loader.contains("pub fn save_design_overrides(\n"),
        "save_design_overrides helper must exist with the expected signature"
    );
    assert!(
        loader.contains("DesignOverrides"),
        "save_design_overrides must accept DesignOverrides"
    );

    let overrides_section = section(
        &loader,
        "pub fn save_design_overrides(",
        "/// Load configuration from",
    );
    assert!(
        overrides_section.contains("CONFIG_PREFERENCE_WRITE_LOCK"),
        "save_design_overrides must reuse the preference write lock"
    );
    assert!(
        overrides_section.contains("write_preference_group("),
        "save_design_overrides must persist via write_preference_group"
    );
    assert!(
        overrides_section.contains("resolve_possibly_legacy_id"),
        "save_design_overrides must run id through legacy migration"
    );
    assert!(
        overrides_section.contains("designs.overrides"),
        "save_design_overrides must mutate designs.overrides"
    );

    let module = fs::read_to_string("src/config/mod.rs").expect("config/mod.rs must be readable");
    assert!(
        module.contains("save_design_overrides"),
        "config module must re-export save_design_overrides"
    );
}

#[test]
fn design_override_action_commits_through_config_helper() {
    let source = fs::read_to_string("src/app_impl/actions_dialog.rs")
        .expect("actions_dialog.rs must be readable");
    assert!(
        source.contains("save_design_overrides"),
        "at least one Design Picker action must commit through save_design_overrides"
    );
    assert!(
        source.contains("ActionsDialogHost::DesignPicker"),
        "the commit site must live in the DesignPicker action host branch"
    );
}

#[test]
fn cycle_design_remains_in_memory_only() {
    let source =
        fs::read_to_string("src/app_impl/theme_focus.rs").expect("theme_focus.rs must be readable");
    let cycle_fn = section(
        &source,
        "pub(crate) fn cycle_design(",
        "pub(crate) fn update_theme(",
    );
    assert!(
        !cycle_fn.contains("save_active_design_id"),
        "cycle_design must remain in-memory only; persist outside the helper"
    );
    assert!(
        !cycle_fn.contains("save_design_overrides"),
        "cycle_design must not write override state"
    );
}
