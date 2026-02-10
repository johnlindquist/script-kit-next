
// =========================================================================
// 34. New chat icons per section
// =========================================================================

#[test]
fn new_chat_last_used_icon_is_bolt() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Recent".to_string(),
        provider: "p".to_string(),
        provider_display_name: "P".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].icon, Some(IconName::BoltFilled));
}

#[test]
fn new_chat_preset_icon_is_custom() {
    let presets = vec![NewChatPresetInfo {
        id: "gen".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].icon, Some(IconName::Star));
}

#[test]
fn new_chat_model_icon_is_settings() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "Claude".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].icon, Some(IconName::Settings));
}

// =========================================================================
// 35. New chat descriptions
// =========================================================================

#[test]
fn new_chat_last_used_has_provider_display_name_description() {
    let last_used = vec![NewChatModelInfo {
        model_id: "lu1".to_string(),
        display_name: "Claude 3.5".to_string(),
        provider: "anthropic".to_string(),
        provider_display_name: "Anthropic".to_string(),
    }];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].description.as_deref(), Some("Anthropic"));
}

#[test]
fn new_chat_preset_has_no_description() {
    let presets = vec![NewChatPresetInfo {
        id: "gen".to_string(),
        name: "General".to_string(),
        icon: IconName::Star,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].description, None);
}

#[test]
fn new_chat_model_has_provider_display_name_description() {
    let models = vec![NewChatModelInfo {
        model_id: "m1".to_string(),
        display_name: "GPT-4".to_string(),
        provider: "openai".to_string(),
        provider_display_name: "OpenAI".to_string(),
    }];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].description.as_deref(), Some("OpenAI"));
}

// =========================================================================
// 36. New chat action ID format
// =========================================================================

#[test]
fn new_chat_last_used_id_format() {
    let last_used = vec![
        NewChatModelInfo {
            model_id: "a".to_string(),
            display_name: "A".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        },
        NewChatModelInfo {
            model_id: "b".to_string(),
            display_name: "B".to_string(),
            provider: "p".to_string(),
            provider_display_name: "P".to_string(),
        },
    ];
    let actions = get_new_chat_actions(&last_used, &[], &[]);
    assert_eq!(actions[0].id, "last_used_0");
    assert_eq!(actions[1].id, "last_used_1");
}

#[test]
fn new_chat_preset_id_format() {
    let presets = vec![NewChatPresetInfo {
        id: "code-review".to_string(),
        name: "Code Review".to_string(),
        icon: IconName::Code,
    }];
    let actions = get_new_chat_actions(&[], &presets, &[]);
    assert_eq!(actions[0].id, "preset_code-review");
}

#[test]
fn new_chat_model_id_format() {
    let models = vec![
        NewChatModelInfo {
            model_id: "claude".to_string(),
            display_name: "Claude".to_string(),
            provider: "anthropic".to_string(),
            provider_display_name: "Anthropic".to_string(),
        },
        NewChatModelInfo {
            model_id: "gpt4".to_string(),
            display_name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_display_name: "OpenAI".to_string(),
        },
    ];
    let actions = get_new_chat_actions(&[], &[], &models);
    assert_eq!(actions[0].id, "model_0");
    assert_eq!(actions[1].id, "model_1");
}
