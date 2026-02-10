    use super::*;
    use crate::config::BuiltInConfig;
    #[test]
    fn test_builtin_config_default() {
        let config = BuiltInConfig::default();
        assert!(config.clipboard_history);
        assert!(config.app_launcher);
        assert!(config.window_switcher);
    }
    #[test]
    fn test_builtin_config_custom() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: true,
            window_switcher: false,
        };
        assert!(!config.clipboard_history);
        assert!(config.app_launcher);
        assert!(!config.window_switcher);
    }
    #[test]
    fn test_get_builtin_entries_all_enabled() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Core built-ins: Clipboard history, window switcher, AI chat, Notes, design gallery
        // Plus: system actions (28), window actions (6), notes commands (3), AI commands (1),
        // script commands (2), permission commands (3) = 43 new entries
        // Total: 5 + 43 = 48
        assert!(entries.len() >= 5); // At minimum the core built-ins should exist

        // Check clipboard history entry
        let clipboard = entries.iter().find(|e| e.id == "builtin-clipboard-history");
        assert!(clipboard.is_some());
        let clipboard = clipboard.unwrap();
        assert_eq!(clipboard.name, "Clipboard History");
        assert_eq!(clipboard.feature, BuiltInFeature::ClipboardHistory);
        assert!(clipboard.keywords.contains(&"clipboard".to_string()));
        assert!(clipboard.keywords.contains(&"history".to_string()));
        assert!(clipboard.keywords.contains(&"paste".to_string()));
        assert!(clipboard.keywords.contains(&"copy".to_string()));

        // Check window switcher entry
        let window_switcher = entries.iter().find(|e| e.id == "builtin-window-switcher");
        assert!(window_switcher.is_some());
        let window_switcher = window_switcher.unwrap();
        assert_eq!(window_switcher.name, "Window Switcher");
        assert_eq!(window_switcher.feature, BuiltInFeature::WindowSwitcher);
        assert!(window_switcher.keywords.contains(&"window".to_string()));
        assert!(window_switcher.keywords.contains(&"switch".to_string()));
        assert!(window_switcher.keywords.contains(&"tile".to_string()));
        assert!(window_switcher.keywords.contains(&"focus".to_string()));
        assert!(window_switcher.keywords.contains(&"manage".to_string()));
        assert!(window_switcher.keywords.contains(&"switcher".to_string()));

        // Check AI chat entry
        let ai_chat = entries.iter().find(|e| e.id == "builtin-ai-chat");
        assert!(ai_chat.is_some());
        let ai_chat = ai_chat.unwrap();
        assert_eq!(ai_chat.name, "AI Chat");
        assert_eq!(ai_chat.feature, BuiltInFeature::AiChat);
        assert!(ai_chat.keywords.contains(&"ai".to_string()));
        assert!(ai_chat.keywords.contains(&"chat".to_string()));
        assert!(ai_chat.keywords.contains(&"claude".to_string()));
        assert!(ai_chat.keywords.contains(&"gpt".to_string()));

        // Check Emoji Picker entry
        let emoji_picker = entries.iter().find(|e| e.id == "builtin-emoji-picker");
        assert!(emoji_picker.is_some());
        let emoji_picker = emoji_picker.unwrap();
        assert_eq!(emoji_picker.name, "Emoji Picker");
        assert_eq!(emoji_picker.feature, BuiltInFeature::EmojiPicker);
        assert!(emoji_picker.keywords.contains(&"emoji".to_string()));
        assert!(emoji_picker.keywords.contains(&"picker".to_string()));

        // Check Quicklinks entry
        let quicklinks = entries.iter().find(|e| e.id == "builtin-quicklinks");
        assert!(quicklinks.is_some());
        let quicklinks = quicklinks.unwrap();
        assert_eq!(quicklinks.name, "Quicklinks");
        assert_eq!(quicklinks.feature, BuiltInFeature::Quicklinks);
        assert!(quicklinks.keywords.contains(&"quicklinks".to_string()));
        assert!(quicklinks.keywords.contains(&"url".to_string()));

        // Note: App Launcher built-in removed - apps now appear directly in main search
    }
    #[test]
    fn test_get_builtin_entries_clipboard_only() {
        let config = BuiltInConfig {
            clipboard_history: true,
            app_launcher: false,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // Check that core entries exist (plus all the new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-clipboard-history"));
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Window switcher should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-window-switcher"));
    }
    #[test]
    fn test_get_builtin_entries_app_launcher_only() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: true,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // App launcher no longer creates a built-in entry (apps appear in main search)
        // But AI Chat, Notes and Design Gallery are always enabled (plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Clipboard history should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
    }
    #[test]
    fn test_get_builtin_entries_none_enabled() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: false,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // AI Chat, Notes, and Design Gallery are always enabled (plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Clipboard history and window switcher should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
        assert!(!entries.iter().any(|e| e.id == "builtin-window-switcher"));
    }
    #[test]
    fn test_get_builtin_entries_window_switcher_only() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: false,
            window_switcher: true,
        };
        let entries = get_builtin_entries(&config);

        // Window switcher + AI Chat + Notes + Design Gallery (always enabled, plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-window-switcher"));
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Verify window switcher has correct properties
        let window_switcher = entries
            .iter()
            .find(|e| e.id == "builtin-window-switcher")
            .unwrap();
        assert_eq!(window_switcher.feature, BuiltInFeature::WindowSwitcher);
        assert_eq!(window_switcher.icon, Some("🪟".to_string()));

        // Clipboard history should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
    }
    #[test]
    fn test_builtin_feature_equality() {
        assert_eq!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::ClipboardHistory
        );
        assert_eq!(BuiltInFeature::AppLauncher, BuiltInFeature::AppLauncher);
        assert_eq!(
            BuiltInFeature::WindowSwitcher,
            BuiltInFeature::WindowSwitcher
        );
        assert_eq!(BuiltInFeature::DesignGallery, BuiltInFeature::DesignGallery);
        assert_eq!(BuiltInFeature::AiChat, BuiltInFeature::AiChat);
        assert_eq!(BuiltInFeature::Favorites, BuiltInFeature::Favorites);
        assert_eq!(BuiltInFeature::EmojiPicker, BuiltInFeature::EmojiPicker);
        assert_eq!(BuiltInFeature::Quicklinks, BuiltInFeature::Quicklinks);
        assert_ne!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::AppLauncher
        );
        assert_ne!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::WindowSwitcher
        );
        assert_ne!(BuiltInFeature::AppLauncher, BuiltInFeature::WindowSwitcher);
        assert_ne!(
            BuiltInFeature::DesignGallery,
            BuiltInFeature::ClipboardHistory
        );
        assert_ne!(BuiltInFeature::AiChat, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::AiChat, BuiltInFeature::DesignGallery);
        assert_ne!(BuiltInFeature::Favorites, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::EmojiPicker, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::Quicklinks, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::EmojiPicker, BuiltInFeature::Quicklinks);

        // Test App variant
        assert_eq!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::App("Safari".to_string())
        );
        assert_ne!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::App("Chrome".to_string())
        );
        assert_ne!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::AppLauncher
        );
    }
    #[test]
    fn test_builtin_entry_new() {
        let entry = BuiltInEntry::new(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test", "keyword"],
            BuiltInFeature::ClipboardHistory,
        );

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.name, "Test Entry");
        assert_eq!(entry.description, "Test description");
        assert_eq!(
            entry.keywords,
            vec!["test".to_string(), "keyword".to_string()]
        );
        assert_eq!(entry.feature, BuiltInFeature::ClipboardHistory);
        assert_eq!(entry.icon, None);
    }
    #[test]
    fn test_builtin_entry_new_with_icon() {
        let entry = BuiltInEntry::new_with_icon(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test"],
            BuiltInFeature::ClipboardHistory,
            "📋",
        );

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.name, "Test Entry");
        assert_eq!(entry.icon, Some("📋".to_string()));
    }
    #[test]
    fn test_builtin_entry_clone() {
        let entry = BuiltInEntry::new_with_icon(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test"],
            BuiltInFeature::AppLauncher,
            "🚀",
        );

        let cloned = entry.clone();
        assert_eq!(entry.id, cloned.id);
        assert_eq!(entry.name, cloned.name);
        assert_eq!(entry.description, cloned.description);
        assert_eq!(entry.keywords, cloned.keywords);
        assert_eq!(entry.feature, cloned.feature);
        assert_eq!(entry.icon, cloned.icon);
    }
    #[test]
    fn test_builtin_config_clone() {
        let config = BuiltInConfig {
            clipboard_history: true,
            app_launcher: false,
            window_switcher: true,
        };

        let cloned = config.clone();
        assert_eq!(config.clipboard_history, cloned.clipboard_history);
        assert_eq!(config.app_launcher, cloned.app_launcher);
        assert_eq!(config.window_switcher, cloned.window_switcher);
    }
    #[test]
    fn test_system_action_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that system action entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-empty-trash"));
        assert!(entries.iter().any(|e| e.id == "builtin-lock-screen"));
        assert!(entries.iter().any(|e| e.id == "builtin-toggle-dark-mode"));
        // Volume presets
        assert!(entries.iter().any(|e| e.id == "builtin-volume-0"));
        assert!(entries.iter().any(|e| e.id == "builtin-volume-50"));
        assert!(entries.iter().any(|e| e.id == "builtin-volume-100"));
        assert!(entries.iter().any(|e| e.id == "builtin-system-preferences"));
    }
    // NOTE: test_window_action_entries_exist removed - window actions now in extension

    #[test]
    fn test_notes_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that notes command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-open-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-new-note"));
        assert!(entries.iter().any(|e| e.id == "builtin-search-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-quick-capture"));
    }
    #[test]
    fn test_get_builtin_entries_includes_open_notes_and_open_ai_commands() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let open_notes = entries.iter().find(|e| e.id == "builtin-open-notes");
        assert!(open_notes.is_some(), "builtin-open-notes should exist");
        assert_eq!(
            open_notes.unwrap().feature,
            BuiltInFeature::NotesCommand(NotesCommandType::OpenNotes)
        );

        let open_ai = entries.iter().find(|e| e.id == "builtin-open-ai");
        assert!(open_ai.is_some(), "builtin-open-ai should exist");
        assert_eq!(
            open_ai.unwrap().feature,
            BuiltInFeature::AiCommand(AiCommandType::OpenAi)
        );

        let generate_script = entries
            .iter()
            .find(|e| e.id == "builtin-generate-script-with-ai");
        assert!(
            generate_script.is_some(),
            "builtin-generate-script-with-ai should exist"
        );
        let generate_script = generate_script.unwrap();
        assert_eq!(
            generate_script.feature,
            BuiltInFeature::AiCommand(AiCommandType::GenerateScript)
        );
        assert!(
            generate_script
                .keywords
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case("shift")),
            "Generate Script command should be discoverable via Shift+Tab wording"
        );
    }
    #[test]
    fn test_get_builtin_entries_hides_preview_ai_commands() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        assert!(
            !entries
                .iter()
                .any(|e| e.id == "builtin-send-screen-area-to-ai"),
            "Preview command should be hidden from built-in entries"
        );
        assert!(
            !entries.iter().any(|e| e.id == "builtin-create-ai-preset"),
            "Preview command should be hidden from built-in entries"
        );
        assert!(
            !entries.iter().any(|e| e.id == "builtin-import-ai-presets"),
            "Preview command should be hidden from built-in entries"
        );
        assert!(
            !entries.iter().any(|e| e.id == "builtin-search-ai-presets"),
            "Preview command should be hidden from built-in entries"
        );
    }

    #[test]
    fn test_get_builtin_entries_includes_favorites_command() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let favorites = entries.iter().find(|e| e.id == "builtin-favorites");
        assert!(favorites.is_some(), "builtin-favorites should exist");

        let favorites = favorites.unwrap();
        assert_eq!(favorites.name, "Favorites");
        assert_eq!(favorites.feature, BuiltInFeature::Favorites);
        assert!(
            favorites
                .keywords
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case("star")),
            "Favorites command should be discoverable with 'star'"
        );
    }
    #[test]
    fn test_script_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that script command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-new-script"));
        assert!(entries.iter().any(|e| e.id == "builtin-new-extension"));
    }
    #[test]
    fn test_new_creation_commands_are_discoverable() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let new_script = entries
            .iter()
            .find(|e| e.id == "builtin-new-script")
            .expect("builtin-new-script should exist");
        assert!(
            new_script.name.to_lowercase().contains("new"),
            "New Script entry name should prominently include 'new'"
        );
        assert!(
            new_script
                .keywords
                .iter()
                .any(|k| k.eq_ignore_ascii_case("template")),
            "New Script entry should be discoverable via 'template'"
        );
        assert!(
            new_script
                .keywords
                .iter()
                .any(|k| k.eq_ignore_ascii_case("starter")),
            "New Script entry should be discoverable via 'starter'"
        );

        let new_extension = entries
            .iter()
            .find(|e| e.id == "builtin-new-extension")
            .expect("builtin-new-extension should exist");
        assert!(
            new_extension
                .keywords
                .iter()
                .any(|k| k.eq_ignore_ascii_case("scriptlet")),
            "New Extension entry should be discoverable via 'scriptlet'"
        );
        assert!(
            new_extension
                .keywords
                .iter()
                .any(|k| k.eq_ignore_ascii_case("frontmatter")),
            "New Extension entry should be discoverable via 'frontmatter'"
        );
    }
    #[test]
    fn test_permission_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that permission command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-check-permissions"));
        assert!(entries
            .iter()
            .any(|e| e.id == "builtin-request-accessibility"));
        assert!(entries
            .iter()
            .any(|e| e.id == "builtin-accessibility-settings"));
    }
    #[test]
    fn test_system_action_type_equality() {
        assert_eq!(SystemActionType::EmptyTrash, SystemActionType::EmptyTrash);
        assert_ne!(SystemActionType::EmptyTrash, SystemActionType::LockScreen);
    }
    // NOTE: test_window_action_type_equality removed - WindowActionType no longer in builtins

    #[test]
    fn test_builtin_feature_system_action() {
        let feature = BuiltInFeature::SystemAction(SystemActionType::ToggleDarkMode);
        assert_eq!(
            feature,
            BuiltInFeature::SystemAction(SystemActionType::ToggleDarkMode)
        );
        assert_ne!(
            feature,
            BuiltInFeature::SystemAction(SystemActionType::Sleep)
        );
    }
