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
