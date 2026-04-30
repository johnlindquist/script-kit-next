//! Source-level contract for case-insensitive `triggerBuiltin` resolution.
//!
//! Case folding now lives in the trigger-builtin registry instead of three
//! duplicated stdin match arms. Dispatchers pass the whole command through the
//! shared dispatcher; the registry trims and lowercases legacy aliases once.

const DISPATCH: &str = include_str!("../src/app_impl/trigger_builtin_dispatch.rs");
const REGISTRY: &str = include_str!("../src/builtins/trigger_registry.rs");
const RUNTIME_STDIN_MATCH_CORE: &str =
    include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");

const DISPATCHERS: &[(&str, &str)] = &[
    (
        RUNTIME_STDIN_MATCH_CORE,
        "src/main_entry/runtime_stdin_match_core.rs",
    ),
    (RUNTIME_STDIN, "src/main_entry/runtime_stdin.rs"),
    (APP_RUN_SETUP, "src/main_entry/app_run_setup.rs"),
];

#[test]
fn registry_lowercases_legacy_aliases_before_lookup() {
    assert!(
        REGISTRY.contains("let normalized = name.trim().to_ascii_lowercase();"),
        "TriggerBuiltinRegistry::lookup_legacy_alias must trim and lowercase incoming legacy names"
    );
    assert!(
        REGISTRY.contains("self.by_legacy_alias.get(normalized.as_str()).copied()"),
        "TriggerBuiltinRegistry::lookup_legacy_alias must use the normalized key"
    );
}

#[test]
fn registry_validates_alias_literals_are_lowercase() {
    assert!(
        REGISTRY.contains("alias != alias.to_ascii_lowercase().as_str()"),
        "TriggerBuiltinRegistry::build must reject uppercase alias literals at startup"
    );
}

#[test]
fn dispatch_resolves_through_registry_not_case_sensitive_match() {
    assert!(
        DISPATCH.contains("trigger_registry().resolve(name)"),
        "dispatch_trigger_builtin_name must resolve through the registry case-folding path"
    );
    assert!(
        !DISPATCH.contains("match name.as_str()")
            && !DISPATCH.contains("match name.to_lowercase().as_str()"),
        "dispatch_trigger_builtin_name must not reintroduce local string-match dispatch"
    );
}

#[test]
fn stdin_dispatchers_do_not_inline_case_sensitive_trigger_matches() {
    for (src, path) in DISPATCHERS {
        let trigger_arm_pos = src
            .find("ExternalCommand::TriggerBuiltin")
            .unwrap_or_else(|| panic!("{path}: missing ExternalCommand::TriggerBuiltin arm"));
        let body = &src[trigger_arm_pos..(trigger_arm_pos + 1200).min(src.len())];
        assert!(
            body.contains("view.dispatch_trigger_builtin(cmd, window, ctx)"),
            "{path}: triggerBuiltin arm must delegate to shared dispatch"
        );
        assert!(
            !body.contains("match name.as_str()") && !body.contains("match name.to_lowercase()"),
            "{path}: triggerBuiltin arm must not inline case-sensitive or case-folding matches"
        );
    }
}
