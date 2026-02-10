    #[test]
    fn notes_cmd_bar_create_quicklink_icon_star() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: true,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "create_quicklink").unwrap();
        assert_eq!(a.icon, Some(IconName::Star));
    }

    #[test]
    fn notes_cmd_bar_enable_auto_sizing_icon_settings() {
        let info = NotesInfo {
            has_selection: false,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        let a = find_action(&actions, "enable_auto_sizing").unwrap();
        assert_eq!(a.icon, Some(IconName::Settings));
    }

    // ========================================
    // 6. Path context exact shortcut values (7 tests)
    // ========================================

    #[test]
    fn path_dir_primary_shortcut_enter() {
        let info = PathInfo {
            name: "docs".to_string(),
            path: "/home/docs".to_string(),
            is_dir: true,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "open_directory").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn path_file_primary_shortcut_enter() {
        let info = PathInfo {
            name: "readme.md".to_string(),
            path: "/home/readme.md".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "select_file").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("↵"));
    }

    #[test]
    fn path_copy_path_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "copy_path").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⇧C"));
    }

    #[test]
    fn path_open_in_finder_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "open_in_finder").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⇧F"));
    }

    #[test]
    fn path_open_in_editor_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "open_in_editor").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘E"));
    }

    #[test]
    fn path_open_in_terminal_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "open_in_terminal").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘T"));
    }

    #[test]
    fn path_move_to_trash_shortcut() {
        let info = PathInfo {
            name: "f".to_string(),
            path: "/f".to_string(),
            is_dir: false,
        };
        let actions = get_path_context_actions(&info);
        let a = find_action(&actions, "move_to_trash").unwrap();
        assert_eq!(a.shortcut.as_deref(), Some("⌘⌫"));
    }

    // ========================================
    // 7. File context exact description strings (6 tests)
    // ========================================

    #[test]
    fn file_open_file_description() {
        let fi = FileInfo {
            path: "/x/y.txt".to_string(),
            name: "y.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "open_file").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Open with default application")
        );
    }

    #[test]
    fn file_open_directory_description() {
        let fi = FileInfo {
            path: "/x/dir".to_string(),
            name: "dir".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "open_directory").unwrap();
        assert_eq!(a.description.as_deref(), Some("Open this folder"));
    }

    #[test]
    fn file_reveal_in_finder_description() {
        let fi = FileInfo {
            path: "/x/y.txt".to_string(),
            name: "y.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "reveal_in_finder").unwrap();
        assert_eq!(a.description.as_deref(), Some("Reveal in Finder"));
    }

    #[test]
    fn file_copy_path_description() {
        let fi = FileInfo {
            path: "/x/y.txt".to_string(),
            name: "y.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "copy_path").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Copy the full path to clipboard")
        );
    }

    #[test]
    fn file_copy_filename_description() {
        let fi = FileInfo {
            path: "/x/y.txt".to_string(),
            name: "y.txt".to_string(),
            file_type: FileType::File,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "copy_filename").unwrap();
        assert_eq!(
            a.description.as_deref(),
            Some("Copy just the filename to clipboard")
        );
    }

    #[test]
    fn file_open_title_includes_name() {
        let fi = FileInfo {
            path: "/x/report.pdf".to_string(),
            name: "report.pdf".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        };
        let actions = get_file_context_actions(&fi);
        let a = find_action(&actions, "open_file").unwrap();
        assert!(a.title.contains("report.pdf"));
    }

    // ========================================
    // 8. FileType variants have no effect on file actions (7 tests)
    // ========================================

    #[test]
    fn filetype_document_same_actions_as_file() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_image_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Image,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Other,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_audio_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Audio,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_video_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Video,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_application_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Application,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_other_same_actions() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Other,
            is_dir: false,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Document,
            is_dir: false,
        });
        assert_eq!(action_ids(&a), action_ids(&b));
    }

    #[test]
    fn filetype_directory_different_from_file() {
        let a = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::Directory,
            is_dir: true,
        });
        let b = get_file_context_actions(&FileInfo {
            path: "/x".to_string(),
            name: "x".to_string(),
            file_type: FileType::File,
            is_dir: false,
        });
        // is_dir changes actions
        assert_ne!(action_ids(&a), action_ids(&b));
    }

    // ========================================
    // 9. Chat model checkmark logic and ID format (6 tests)
    // ========================================

    #[test]
    fn chat_model_id_format_select_model_prefix() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions[0].id.starts_with("select_model_"));
        assert_eq!(actions[0].id, "select_model_gpt-4");
    }

    #[test]
    fn chat_current_model_gets_checkmark_in_title() {
        let info = ChatPromptInfo {
            current_model: Some("GPT-4".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(actions[0].title.contains("✓"));
        assert_eq!(actions[0].title, "GPT-4 ✓");
    }

    #[test]
    fn chat_non_current_model_no_checkmark() {
        let info = ChatPromptInfo {
            current_model: Some("Claude".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert!(!actions[0].title.contains("✓"));
        assert_eq!(actions[0].title, "GPT-4");
    }

    #[test]
    fn chat_model_description_shows_provider() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![ChatModelInfo {
                id: "claude-3".to_string(),
                display_name: "Claude 3".to_string(),
                provider: "Anthropic".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions[0].description.as_deref(), Some("via Anthropic"));
    }

    #[test]
    fn chat_no_models_only_continue_in_chat() {
        let info = ChatPromptInfo {
            current_model: None,
            available_models: vec![],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "continue_in_chat");
    }

    #[test]
    fn chat_checkmark_exact_match_only() {
        let info = ChatPromptInfo {
            current_model: Some("GPT".to_string()),
            available_models: vec![ChatModelInfo {
                id: "gpt-4".to_string(),
                display_name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
            }],
            has_messages: false,
            has_response: false,
        };
        let actions = get_chat_context_actions(&info);
        // "GPT" != "GPT-4", so no checkmark
        assert!(!actions[0].title.contains("✓"));
    }

    // ========================================
    // 10. New chat provider_display_name propagation (5 tests)
    // ========================================

    #[test]
    fn new_chat_last_used_description_is_provider_display_name() {
        let actions = get_new_chat_actions(
            &[NewChatModelInfo {
                model_id: "m1".to_string(),
                display_name: "Model 1".to_string(),
                provider: "provider-id".to_string(),
                provider_display_name: "My Provider".to_string(),
            }],
            &[],
            &[],
        );
        assert_eq!(actions[0].description.as_deref(), Some("My Provider"));
    }

