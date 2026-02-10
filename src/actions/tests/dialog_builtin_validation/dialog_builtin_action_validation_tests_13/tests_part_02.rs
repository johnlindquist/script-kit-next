    #[test]
    fn cat06_one_char_singular() {
        let actions = get_note_switcher_actions(&[make_note("", "", 1)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "1 char");
    }

    #[test]
    fn cat06_zero_chars_plural() {
        let actions = get_note_switcher_actions(&[make_note("", "", 0)]);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "0 chars");
    }

    // =========================================================================
    // 7. Chat context multi-model ordering and checkmark logic
    // =========================================================================

    fn make_chat_info(
        current: Option<&str>,
        models: &[(&str, &str, &str)],
        has_response: bool,
        has_messages: bool,
    ) -> ChatPromptInfo {
        ChatPromptInfo {
            current_model: current.map(|s| s.to_string()),
            available_models: models
                .iter()
                .map(|(id, name, provider)| ChatModelInfo {
                    id: id.to_string(),
                    display_name: name.to_string(),
                    provider: provider.to_string(),
                })
                .collect(),
            has_response,
            has_messages,
        }
    }

    #[test]
    fn cat07_model_actions_ordered_by_input() {
        let info = make_chat_info(
            None,
            &[("m1", "Model A", "P1"), ("m2", "Model B", "P2")],
            false,
            false,
        );
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].id, "select_model_m1");
        assert_eq!(actions[1].id, "select_model_m2");
    }

    #[test]
    fn cat07_current_model_gets_checkmark() {
        let info = make_chat_info(
            Some("Model A"),
            &[("m1", "Model A", "P1"), ("m2", "Model B", "P2")],
            false,
            false,
        );
        let actions = get_chat_context_actions(&info);
        let m1 = actions.iter().find(|a| a.id == "select_model_m1").unwrap();
        assert!(m1.title.contains('✓'), "Current model should have ✓");
    }

    #[test]
    fn cat07_non_current_model_no_checkmark() {
        let info = make_chat_info(
            Some("Model A"),
            &[("m1", "Model A", "P1"), ("m2", "Model B", "P2")],
            false,
            false,
        );
        let actions = get_chat_context_actions(&info);
        let m2 = actions.iter().find(|a| a.id == "select_model_m2").unwrap();
        assert!(
            !m2.title.contains('✓'),
            "Non-current model should NOT have ✓"
        );
    }

    #[test]
    fn cat07_model_description_is_via_provider() {
        let info = make_chat_info(None, &[("m1", "Claude", "Anthropic")], false, false);
        let actions = get_chat_context_actions(&info);
        let desc = actions[0].description.as_ref().unwrap();
        assert_eq!(desc, "via Anthropic");
    }

    #[test]
    fn cat07_no_models_still_has_continue_in_chat() {
        let info = make_chat_info(None, &[], false, false);
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "continue_in_chat"));
    }

    #[test]
    fn cat07_has_response_adds_copy_response() {
        let info = make_chat_info(None, &[], true, false);
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn cat07_no_response_no_copy_response() {
        let info = make_chat_info(None, &[], false, false);
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "copy_response"));
    }

    #[test]
    fn cat07_has_messages_adds_clear_conversation() {
        let info = make_chat_info(None, &[], false, true);
        let actions = get_chat_context_actions(&info);
        assert!(actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn cat07_no_messages_no_clear_conversation() {
        let info = make_chat_info(None, &[], false, false);
        let actions = get_chat_context_actions(&info);
        assert!(!actions.iter().any(|a| a.id == "clear_conversation"));
    }

    #[test]
    fn cat07_continue_in_chat_shortcut() {
        let info = make_chat_info(None, &[], false, false);
        let actions = get_chat_context_actions(&info);
        let a = actions.iter().find(|a| a.id == "continue_in_chat").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘↵"));
    }

    // =========================================================================
    // 8. AI command bar actions without shortcuts
    // =========================================================================

    #[test]
    fn cat08_branch_from_last_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "branch_from_last").unwrap();
        assert!(
            a.shortcut.is_none(),
            "branch_from_last should have no shortcut"
        );
    }

    #[test]
    fn cat08_change_model_no_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "change_model").unwrap();
        assert!(a.shortcut.is_none(), "change_model should have no shortcut");
    }

    #[test]
    fn cat08_toggle_shortcuts_help_has_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = actions
            .iter()
            .find(|a| a.id == "toggle_shortcuts_help")
            .unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘/"));
    }

    #[test]
    fn cat08_export_markdown_shortcut() {
        let actions = get_ai_command_bar_actions();
        let a = actions.iter().find(|a| a.id == "export_markdown").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⇧⌘E"));
    }

    #[test]
    fn cat08_ai_all_have_icons() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.icon.is_some(),
                "AI action {} should have icon",
                action.id
            );
        }
    }

    #[test]
    fn cat08_ai_all_have_sections() {
        let actions = get_ai_command_bar_actions();
        for action in &actions {
            assert!(
                action.section.is_some(),
                "AI action {} should have section",
                action.id
            );
        }
    }

    // =========================================================================
    // 9. CommandBarConfig close flag defaults
    // =========================================================================

    #[test]
    fn cat09_default_close_on_select_true() {
        let config = super::super::command_bar::CommandBarConfig::default();
        assert!(config.close_on_select);
    }

    #[test]
    fn cat09_default_close_on_click_outside_true() {
        let config = super::super::command_bar::CommandBarConfig::default();
        assert!(config.close_on_click_outside);
    }

    #[test]
    fn cat09_default_close_on_escape_true() {
        let config = super::super::command_bar::CommandBarConfig::default();
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat09_ai_style_close_defaults_preserved() {
        let config = super::super::command_bar::CommandBarConfig::ai_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat09_main_menu_style_close_defaults_preserved() {
        let config = super::super::command_bar::CommandBarConfig::main_menu_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat09_no_search_style_close_defaults_preserved() {
        let config = super::super::command_bar::CommandBarConfig::no_search();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    #[test]
    fn cat09_notes_style_close_defaults_preserved() {
        let config = super::super::command_bar::CommandBarConfig::notes_style();
        assert!(config.close_on_select);
        assert!(config.close_on_click_outside);
        assert!(config.close_on_escape);
    }

    // =========================================================================
    // 10. Cross-builder shortcut/alias action symmetry
    // =========================================================================

    #[test]
    fn cat10_script_no_shortcut_no_alias_has_add_both() {
        let s = ScriptInfo::new("t", "/p");
        let ids = action_ids(&get_script_context_actions(&s));
        assert!(ids.contains(&"add_shortcut".into()));
        assert!(ids.contains(&"add_alias".into()));
        assert!(!ids.contains(&"update_shortcut".into()));
        assert!(!ids.contains(&"update_alias".into()));
    }

    #[test]
    fn cat10_script_has_shortcut_has_alias_has_update_remove_both() {
        let s =
            ScriptInfo::with_shortcut_and_alias("t", "/p", Some("cmd+t".into()), Some("ts".into()));
        let ids = action_ids(&get_script_context_actions(&s));
        assert!(ids.contains(&"update_shortcut".into()));
        assert!(ids.contains(&"remove_shortcut".into()));
        assert!(ids.contains(&"update_alias".into()));
        assert!(ids.contains(&"remove_alias".into()));
        assert!(!ids.contains(&"add_shortcut".into()));
        assert!(!ids.contains(&"add_alias".into()));
    }

    #[test]
    fn cat10_scriptlet_context_same_shortcut_alias_logic() {
        let s = ScriptInfo::scriptlet("t", "/p", Some("cmd+k".into()), Some("tk".into()));
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"update_shortcut".into()));
        assert!(ids.contains(&"remove_shortcut".into()));
        assert!(ids.contains(&"update_alias".into()));
        assert!(ids.contains(&"remove_alias".into()));
    }

    #[test]
    fn cat10_scriptlet_no_shortcut_no_alias_has_add() {
        let s = ScriptInfo::scriptlet("t", "/p", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"add_shortcut".into()));
        assert!(ids.contains(&"add_alias".into()));
    }

    #[test]
    fn cat10_shortcut_and_alias_action_shortcut_values() {
        let s = ScriptInfo::new("t", "/p");
        let actions = get_script_context_actions(&s);
        let add_sc = actions.iter().find(|a| a.id == "add_shortcut").unwrap();
        assert_eq!(add_sc.shortcut.as_deref(), Some("⌘⇧K"));
        let add_al = actions.iter().find(|a| a.id == "add_alias").unwrap();
        assert_eq!(add_al.shortcut.as_deref(), Some("⌘⇧A"));
    }

    // =========================================================================
    // 11. Scriptlet context action verb propagation
    // =========================================================================

    #[test]
    fn cat11_scriptlet_run_title_uses_action_verb() {
        let s = ScriptInfo::scriptlet("My Script", "/p", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(
            run.title.starts_with("Run "),
            "Title should start with 'Run ': {}",
            run.title
        );
        assert!(
            run.title.contains("My Script"),
            "Title should contain name: {}",
            run.title
        );
    }

    #[test]
    fn cat11_scriptlet_run_description_uses_verb() {
        let s = ScriptInfo::scriptlet("T", "/p", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        let desc = run.description.as_ref().unwrap();
        assert!(
            desc.contains("Run"),
            "Description should contain verb: {}",
            desc
        );
    }

    #[test]
    fn cat11_script_context_custom_verb_propagates() {
        let s = ScriptInfo::with_action_verb("Safari", "/app", false, "Launch");
        let actions = get_script_context_actions(&s);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(
            run.title.starts_with("Launch "),
            "Title should start with 'Launch ': {}",
            run.title
        );
    }

    #[test]
    fn cat11_script_context_switch_to_verb() {
        let s = ScriptInfo::with_action_verb("Window", "win:1", false, "Switch to");
        let actions = get_script_context_actions(&s);
        let run = actions.iter().find(|a| a.id == "run_script").unwrap();
        assert!(run.title.starts_with("Switch to "), "Title: {}", run.title);
    }

    // =========================================================================
    // 12. Agent context exact action IDs
    // =========================================================================

    #[test]
    fn cat12_agent_has_edit_script_not_edit_agent_id() {
        // Agent uses "edit_script" as ID but "Edit Agent" as title
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        let ids = action_ids(&actions);
        assert!(ids.contains(&"edit_script".into()));
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert_eq!(edit.title, "Edit Agent");
    }

    #[test]
    fn cat12_agent_has_reveal_in_finder() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        assert!(actions.iter().any(|a| a.id == "reveal_in_finder"));
    }

    #[test]
    fn cat12_agent_has_copy_path() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        assert!(actions.iter().any(|a| a.id == "copy_path"));
    }

    #[test]
    fn cat12_agent_has_copy_content() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        assert!(actions.iter().any(|a| a.id == "copy_content"));
    }

    #[test]
    fn cat12_agent_no_view_logs() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        assert!(!actions.iter().any(|a| a.id == "view_logs"));
    }

    #[test]
    fn cat12_agent_descriptions_mention_agent() {
        let mut s = ScriptInfo::new("My Agent", "/agent");
        s.is_agent = true;
        s.is_script = false;
        let actions = get_script_context_actions(&s);
        let edit = actions.iter().find(|a| a.id == "edit_script").unwrap();
        assert!(edit.description.as_ref().unwrap().contains("agent"));
        let reveal = actions.iter().find(|a| a.id == "reveal_in_finder").unwrap();
        assert!(reveal.description.as_ref().unwrap().contains("agent"));
    }

    // =========================================================================
    // 13. Deeplink URL in description for scriptlet context
    // =========================================================================

    #[test]
    fn cat13_scriptlet_deeplink_description_contains_url() {
        let s = ScriptInfo::scriptlet("My Script", "/p", None, None);
        let actions = get_scriptlet_context_actions_with_custom(&s, None);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(desc.contains("scriptkit://run/my-script"), "Desc: {}", desc);
    }

    #[test]
    fn cat13_script_deeplink_description_format() {
        let s = ScriptInfo::new("Hello World", "/p");
        let actions = get_script_context_actions(&s);
        let dl = actions.iter().find(|a| a.id == "copy_deeplink").unwrap();
        let desc = dl.description.as_ref().unwrap();
        assert!(
            desc.contains("scriptkit://run/hello-world"),
            "Desc: {}",
            desc
        );
    }

