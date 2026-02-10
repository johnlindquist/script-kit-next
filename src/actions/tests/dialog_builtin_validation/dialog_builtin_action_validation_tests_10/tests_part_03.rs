    #[test]
    fn new_chat_models_section_description_is_provider_display_name() {
        let actions = get_new_chat_actions(
            &[],
            &[],
            &[NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "pid".to_string(),
                provider_display_name: "Anthropic AI".to_string(),
            }],
        );
        assert_eq!(actions[0].description.as_deref(), Some("Anthropic AI"));
    }

    #[test]
    fn new_chat_presets_have_no_description() {
        let actions = get_new_chat_actions(
            &[],
            &[NewChatPresetInfo {
                id: "general".to_string(),
                name: "General".to_string(),
                icon: IconName::Settings,
            }],
            &[],
        );
        assert!(actions[0].description.is_none());
    }

    #[test]
    fn new_chat_preset_uses_its_icon() {
        let actions = get_new_chat_actions(
            &[],
            &[NewChatPresetInfo {
                id: "code".to_string(),
                name: "Code".to_string(),
                icon: IconName::Code,
            }],
            &[],
        );
        assert_eq!(actions[0].icon, Some(IconName::Code));
    }

    #[test]
    fn new_chat_mixed_sections_in_order() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "lu1".to_string(),
                display_name: "LU1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            }],
            &[NewChatPresetInfo {
                id: "gen".to_string(),
                name: "Gen".to_string(),
                icon: IconName::File,
            }],
            &[NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "M1".to_string(),
                provider: "p".to_string(),
                provider_display_name: "P".to_string(),
            }],
        );
        assert_eq!(actions[0].section.as_deref(), Some("Last Used Settings"));
        assert_eq!(actions[1].section.as_deref(), Some("Presets"));
        assert_eq!(actions[2].section.as_deref(), Some("Models"));
    }

    // ========================================
    // 11. Clipboard exact description strings (8 tests)
    // ========================================

    #[test]
    fn clipboard_paste_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_paste").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Copy to clipboard and paste to focused app")
        );
    }

    #[test]
    fn clipboard_copy_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_copy").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Copy entry to clipboard without pasting")
        );
    }

    #[test]
    fn clipboard_paste_keep_open_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_paste_keep_open").unwrap();
        assert!(a.description.as_ref().unwrap().contains("keep"));
    }

    #[test]
    fn clipboard_pin_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_pin").unwrap();
        assert!(a.description.as_ref().unwrap().contains("Pin"));
    }

    #[test]
    fn clipboard_unpin_description() {
        let entry = make_clipboard_entry(ContentType::Text, true, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_unpin").unwrap();
        assert!(a.description.as_ref().unwrap().contains("pin"));
    }

    #[test]
    fn clipboard_delete_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_delete").unwrap();
        assert!(a.description.as_ref().unwrap().contains("Remove"));
    }

    #[test]
    fn clipboard_delete_multiple_description() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_delete_multiple").unwrap();
        assert!(a.description.as_ref().unwrap().contains("filter"));
    }

    #[test]
    fn clipboard_delete_all_description_mentions_pinned() {
        let entry = make_clipboard_entry(ContentType::Text, false, None);
        let actions = get_clipboard_history_context_actions(&entry);
        let a = find_action(&actions, "clipboard_delete_all").unwrap();
        assert!(a.description.as_ref().unwrap().contains("pinned"));
    }

    // ========================================
    // 12. Script context with custom verbs (5 tests)
    // ========================================

    #[test]
    fn custom_verb_launch_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("Safari", "/Applications/Safari.app", false, "Launch");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.title.starts_with("Launch"));
        assert!(run.title.contains("Safari"));
    }

    #[test]
    fn custom_verb_switch_to_in_primary_title() {
        let script = ScriptInfo::with_action_verb("My Window", "window:123", false, "Switch to");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.title.starts_with("Switch to"));
    }

    #[test]
    fn custom_verb_open_in_primary_title() {
        let script =
            ScriptInfo::with_action_verb("App Launcher", "builtin:launcher", false, "Open");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert_eq!(run.title, "Open \"App Launcher\"");
    }

    #[test]
    fn custom_verb_execute_in_description() {
        let script = ScriptInfo::with_action_verb("Task", "/path/task.ts", true, "Execute");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.description.as_ref().unwrap().contains("Execute"));
    }

    #[test]
    fn default_verb_is_run() {
        let script = ScriptInfo::new("test", "/path/test.ts");
        assert_eq!(script.action_verb, "Run");
        let actions = get_script_context_actions(&script);
        let run = find_action(&actions, "run_script").unwrap();
        assert!(run.title.starts_with("Run"));
    }

    // ========================================
    // 13. ActionsDialogConfig defaults (5 tests)
    // ========================================

    #[test]
    fn actions_dialog_config_default_search_bottom() {
        let config = ActionsDialogConfig::default();
        assert_eq!(config.search_position, SearchPosition::Bottom);
    }

    #[test]
    fn actions_dialog_config_default_section_separators() {
        let config = ActionsDialogConfig::default();
        assert_eq!(config.section_style, SectionStyle::Separators);
    }

    #[test]
    fn actions_dialog_config_default_anchor_bottom() {
        let config = ActionsDialogConfig::default();
        assert_eq!(config.anchor, AnchorPosition::Bottom);
    }

    #[test]
    fn actions_dialog_config_default_no_icons() {
        let config = ActionsDialogConfig::default();
        assert!(!config.show_icons);
    }

    #[test]
    fn actions_dialog_config_default_no_footer() {
        let config = ActionsDialogConfig::default();
        assert!(!config.show_footer);
    }

    // ========================================
    // 14. ActionCategory PartialEq (4 tests)
    // ========================================

    #[test]
    fn action_category_eq_same() {
        assert_eq!(ActionCategory::ScriptContext, ActionCategory::ScriptContext);
        assert_eq!(ActionCategory::ScriptOps, ActionCategory::ScriptOps);
        assert_eq!(ActionCategory::GlobalOps, ActionCategory::GlobalOps);
        assert_eq!(ActionCategory::Terminal, ActionCategory::Terminal);
    }

    #[test]
    fn action_category_ne_different() {
        assert_ne!(ActionCategory::ScriptContext, ActionCategory::ScriptOps);
        assert_ne!(ActionCategory::ScriptContext, ActionCategory::GlobalOps);
        assert_ne!(ActionCategory::ScriptContext, ActionCategory::Terminal);
    }

    #[test]
    fn action_category_ne_script_ops_vs_global() {
        assert_ne!(ActionCategory::ScriptOps, ActionCategory::GlobalOps);
    }

    #[test]
    fn action_category_ne_terminal_vs_global() {
        assert_ne!(ActionCategory::Terminal, ActionCategory::GlobalOps);
    }

    // ========================================
    // 15. Agent description content keywords (5 tests)
    // ========================================

    #[test]
    fn agent_edit_description_mentions_agent_file() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "edit_script").unwrap();
        assert!(a.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn agent_reveal_description_mentions_agent() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "reveal_in_finder").unwrap();
        assert!(a.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn agent_copy_path_description_mentions_agent() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "copy_path").unwrap();
        assert!(a.description.as_ref().unwrap().contains("agent"));
    }

    #[test]
    fn agent_copy_content_description() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "copy_content").unwrap();
        assert!(a.description.as_ref().unwrap().contains("content"));
    }

    #[test]
    fn agent_edit_title_says_edit_agent() {
        let mut agent = ScriptInfo::new("MyAgent", "/path/agent.md");
        agent.is_script = false;
        agent.is_agent = true;
        let actions = get_script_context_actions(&agent);
        let a = find_action(&actions, "edit_script").unwrap();
        assert_eq!(a.title, "Edit Agent");
    }

    // ========================================
    // 16. Cross-context frecency reset consistency (3 tests)
    // ========================================

    #[test]
    fn frecency_reset_present_for_script() {
        let script = ScriptInfo::new("s", "/p").with_frecency(true, Some("/p".to_string()));
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn frecency_reset_present_for_scriptlet() {
        let script = ScriptInfo::scriptlet("s", "/p.md", None, None)
            .with_frecency(true, Some("x".to_string()));
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    #[test]
    fn frecency_reset_present_for_builtin() {
        let script = ScriptInfo::builtin("B").with_frecency(true, Some("b".to_string()));
        let actions = get_script_context_actions(&script);
        assert!(actions.iter().any(|a| a.id == "reset_ranking"));
    }

    // ========================================
    // 17. Script context exact shortcut values (5 tests)
    // ========================================

    #[test]
    fn script_edit_shortcut_cmd_e() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "edit_script").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn script_view_logs_shortcut_cmd_l() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "view_logs").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘L"));
    }

    #[test]
    fn script_reveal_shortcut_cmd_shift_f() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "reveal_in_finder").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⇧F"));
    }

    #[test]
    fn script_copy_path_shortcut_cmd_shift_c() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "copy_path").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn script_copy_content_shortcut_cmd_opt_c() {
        let script = ScriptInfo::new("t", "/p/t.ts");
        let actions = get_script_context_actions(&script);
        let a = find_action(&actions, "copy_content").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⌥C"));
    }

    // ========================================
    // 18. CommandBarConfig factory methods (5 tests)
    // ========================================

    #[test]
    fn command_bar_ai_style_search_top_headers_icons_footer() {
        let c = CommandBarConfig::ai_style();
        assert_eq!(c.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(c.dialog_config.section_style, SectionStyle::Headers);
        assert!(c.dialog_config.show_icons);
        assert!(c.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_main_menu_search_bottom_separators() {
        let c = CommandBarConfig::main_menu_style();
        assert_eq!(c.dialog_config.search_position, SearchPosition::Bottom);
        assert_eq!(c.dialog_config.section_style, SectionStyle::Separators);
        assert!(!c.dialog_config.show_icons);
        assert!(!c.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_no_search_hidden() {
        let c = CommandBarConfig::no_search();
        assert_eq!(c.dialog_config.search_position, SearchPosition::Hidden);
    }

    #[test]
    fn command_bar_notes_style_search_top_separators_icons_footer() {
        let c = CommandBarConfig::notes_style();
        assert_eq!(c.dialog_config.search_position, SearchPosition::Top);
        assert_eq!(c.dialog_config.section_style, SectionStyle::Separators);
        assert!(c.dialog_config.show_icons);
        assert!(c.dialog_config.show_footer);
    }

    #[test]
    fn command_bar_default_close_flags_all_true() {
        let c = CommandBarConfig::default();
        assert!(c.close_on_select);
        assert!(c.close_on_click_outside);
        assert!(c.close_on_escape);
    }

    // ========================================
    // 19. Notes command bar exact shortcuts (6 tests)
    // ========================================

    #[test]
    fn notes_cmd_bar_new_note_shortcut() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "new_note").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘N"));
    }

