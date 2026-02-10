
#[test]
fn all_new_chat_actions_have_nonempty_title_and_id() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Model 1".to_string(),
        provider: "p1".to_string(),
        provider_display_name: "Provider 1".to_string(),
    }];
    let presets = vec![NewChatPresetInfo {
        id: "general".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    for action in get_new_chat_actions(&models, &presets, &models) {
        assert!(!action.id.is_empty());
        assert!(
            !action.title.is_empty(),
            "Action {} should have non-empty title",
            action.id
        );
    }
}
