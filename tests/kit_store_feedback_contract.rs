const KIT_STORE: &str = include_str!("../src/render_builtins/kit_store.rs");

#[test]
fn kit_store_plugin_mutation_feedback_is_named_state() {
    assert!(
        KIT_STORE.contains("enum KitStorePluginMutation"),
        "Kit Store install/update/remove feedback must be owned by a named mutation state"
    );

    for variant in ["Install", "Update", "Remove"] {
        assert!(
            KIT_STORE.contains(&format!("Self::{variant}")),
            "KitStorePluginMutation must keep the {variant} variant"
        );
    }

    for method in [
        "fn action(self) -> &'static str",
        "fn progress_message(self, plugin_name: &str) -> String",
        "fn success_message(self, plugin_name: &str) -> String",
        "fn failure_message(self, plugin_name: Option<&str>, error: &str) -> String",
    ] {
        assert!(
            KIT_STORE.contains(method),
            "KitStorePluginMutation must own {method}"
        );
    }

    assert!(
        !KIT_STORE.contains("fn plugin_mutation_message(action: &str"),
        "Kit Store feedback must not regress to stringly-typed action branching"
    );
}

#[test]
fn kit_store_toasts_and_refresh_use_named_mutation_state() {
    for usage in [
        "KitStorePluginMutation::Install.progress_message(&selected.name)",
        "KitStorePluginMutation::Install.success_message(&installed.name)",
        "KitStorePluginMutation::Install.failure_message(None, &error)",
        "KitStorePluginMutation::Update.progress_message(&kit_name)",
        "KitStorePluginMutation::Update.success_message(&kit_name)",
        "KitStorePluginMutation::Update\n                                .failure_message(Some(&kit_name), &error)",
        "KitStorePluginMutation::Remove.progress_message(&kit_name)",
        "KitStorePluginMutation::Remove.success_message(&kit_name)",
        "KitStorePluginMutation::Remove\n                                .failure_message(Some(&kit_name), &error)",
    ] {
        assert!(
            KIT_STORE.contains(usage),
            "Kit Store operation feedback should route through named mutation state: {usage}"
        );
    }

    for refresh in [
        "this.request_plugin_runtime_refresh(\n                        KitStorePluginMutation::Install,",
        "this.request_plugin_runtime_refresh(KitStorePluginMutation::Update, &kit_name, cx)",
        "this.request_plugin_runtime_refresh(KitStorePluginMutation::Remove, &kit_name, cx)",
    ] {
        assert!(
            KIT_STORE.contains(refresh),
            "Kit Store runtime refresh tracing should use named mutation state: {refresh}"
        );
    }
}

#[test]
fn kit_store_operation_failures_are_named_steps() {
    assert!(
        KIT_STORE.contains("enum KitStoreOperationStep"),
        "Kit Store git/storage/remove failures must be owned by named operation steps"
    );

    for variant in [
        "ReadGitHead",
        "SaveInstalledRegistry",
        "PullRepository",
        "SaveUpdatedRegistry",
        "RemoveDirectory",
        "RemoveRegistry",
    ] {
        assert!(
            KIT_STORE.contains(&format!("Self::{variant}")),
            "KitStoreOperationStep must keep the {variant} variant"
        );
    }

    for method in [
        "fn git_command(self) -> Option<&'static str>",
        "fn git_spawn_failure(self, error: impl std::fmt::Display) -> String",
        "fn git_status_failure(",
        "fn storage_failure(self, error: impl std::fmt::Display) -> String",
        "fn remove_directory_failure(self, error: impl std::fmt::Display) -> String",
        "fn empty_git_hash_message(self) -> String",
    ] {
        assert!(
            KIT_STORE.contains(method),
            "KitStoreOperationStep must own {method}"
        );
    }

    for scattered_call_site in [
        ".map_err(|error| format!(\"Failed to run git rev-parse: {}\", error))",
        ".map_err(|error| format!(\"Failed to update plugin registry: {}\", error))",
        ".map_err(|error| format!(\"Failed to run git pull: {}\", error))",
        ".map_err(|error| format!(\"Failed to save updated kit registry: {}\", error))",
        ".map_err(|error| format!(\"Failed to remove kit directory: {}\", error))",
        ".map_err(|error| format!(\"Failed to update kit registry: {}\", error))",
    ] {
        assert!(
            !KIT_STORE.contains(scattered_call_site),
            "Kit Store operation failures must not regress to scattered call-site formatting: {scattered_call_site}"
        );
    }
}

#[test]
fn kit_store_git_storage_and_remove_paths_use_operation_steps() {
    for usage in [
        "KitStoreOperationStep::ReadGitHead.git_spawn_failure(error)",
        "KitStoreOperationStep::ReadGitHead.git_status_failure(output.status, &output.stderr)",
        "KitStoreOperationStep::ReadGitHead.empty_git_hash_message()",
        "KitStoreOperationStep::SaveInstalledRegistry.storage_failure(error)",
        "KitStoreOperationStep::PullRepository.git_spawn_failure(error)",
        "KitStoreOperationStep::PullRepository\n                .git_status_failure(pull_output.status, &pull_output.stderr)",
        "KitStoreOperationStep::SaveUpdatedRegistry.storage_failure(error)",
        "KitStoreOperationStep::RemoveDirectory.remove_directory_failure(error)",
        "KitStoreOperationStep::RemoveRegistry.storage_failure(error)",
    ] {
        assert!(
            KIT_STORE.contains(usage),
            "Kit Store operation path should route through named step: {usage}"
        );
    }
}
