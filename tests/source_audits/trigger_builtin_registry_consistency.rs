//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR1:
//! cross-check the canonical [`TriggerBuiltin`] registry against the
//! `BuiltInFeature` launcher registration.
//!
//! The `triggerBuiltin` stdin verb resolves a name into a
//! [`TriggerBuiltin`] variant and then opens the corresponding view.
//! The view is ultimately a `BuiltInFeature` — or, for intentionally
//! hidden routes, a `hidden_builtin_entry`. If the two tables ever
//! drift (someone adds a `TriggerBuiltin` variant but forgets to
//! register the `builtin/...` id in [`get_builtin_entries`] /
//! `hidden_builtin_entry`), the bug surfaces as a silent no-op at
//! runtime. This test fails loudly at compile / `cargo test` time.

use script_kit_gpui::builtins::resolve_builtin_entry;
use script_kit_gpui::builtins::trigger_registry::{
    all_trigger_builtin_command_ids, TriggerBuiltin,
};
use script_kit_gpui::config::BuiltInConfig;

#[test]
fn every_trigger_builtin_command_id_resolves_as_builtin_entry() {
    let config = BuiltInConfig::default();
    let mut missing = Vec::new();

    for &id in TriggerBuiltin::ALL {
        if !id.requires_builtin_feature_entry() {
            continue;
        }
        let command_id = id.command_id();
        if resolve_builtin_entry(command_id, &config).is_none() {
            missing.push(format!(
                "{id:?} ({command_id}) has no BuiltInEntry registration"
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "TriggerBuiltin variants missing BuiltInFeature registration:\n{}\n\n\
         Every canonical `TriggerBuiltin` command id that claims \
         `requires_builtin_feature_entry = true` must resolve via \
         `resolve_builtin_entry` (either through `get_builtin_entries` or via \
         `hidden_builtin_entry`). If a route is intentionally internal-only, \
         set `requires_builtin_feature_entry` to `false` in \
         `src/builtins/trigger_registry.rs` so the registry and the launcher \
         stay in lockstep.",
        missing.join("\n")
    );
}

#[test]
fn internal_only_trigger_variants_have_no_launcher_entry() {
    let config = BuiltInConfig::default();
    let mut unexpected = Vec::new();

    for &id in TriggerBuiltin::ALL {
        if id.requires_builtin_feature_entry() {
            continue;
        }
        let command_id = id.command_id();
        if resolve_builtin_entry(command_id, &config).is_some() {
            unexpected.push(format!(
                "{id:?} ({command_id}) is marked internal-only but DOES register a BuiltInEntry"
            ));
        }
    }

    assert!(
        unexpected.is_empty(),
        "TriggerBuiltin variants claim `requires_builtin_feature_entry = false` but still \
         resolve via `resolve_builtin_entry`:\n{}\n\n\
         Either add the entry and flip the flag to `true`, or remove the launcher \
         registration.",
        unexpected.join("\n")
    );
}

#[test]
fn exported_command_id_slice_matches_variant_enumeration() {
    let from_accessor: Vec<&'static str> = all_trigger_builtin_command_ids().to_vec();
    let from_enum: Vec<&'static str> = TriggerBuiltin::ALL
        .iter()
        .map(|id| id.command_id())
        .collect();

    assert_eq!(
        from_accessor, from_enum,
        "all_trigger_builtin_command_ids() must list exactly the same ids \
         as TriggerBuiltin::ALL.iter().map(|id| id.command_id())"
    );
}
