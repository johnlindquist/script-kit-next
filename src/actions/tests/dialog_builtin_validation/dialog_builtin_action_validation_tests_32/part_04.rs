
#[test]
fn batch32_all_new_chat_actions_are_script_context() {
    let models = vec![NewChatModelInfo {
        model_id: "m".into(),
        display_name: "M".into(),
        provider: "p".into(),
        provider_display_name: "P".into(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    for a in &actions {
        assert_eq!(
            a.category,
            ActionCategory::ScriptContext,
            "new chat action {} should be ScriptContext",
            a.id
        );
    }
}
