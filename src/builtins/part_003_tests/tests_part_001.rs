    // NOTE: test_builtin_feature_window_action removed - WindowAction no longer in BuiltInFeature

    #[test]
    fn test_file_search_builtin_exists() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that FileSearch entry exists
        let file_search = entries.iter().find(|e| e.id == "builtin-file-search");
        assert!(
            file_search.is_some(),
            "FileSearch builtin should exist in the main menu"
        );

        let file_search = file_search.unwrap();
        assert_eq!(file_search.name, "Search Files");
        assert_eq!(file_search.feature, BuiltInFeature::FileSearch);
        assert!(file_search.keywords.contains(&"file".to_string()));
        assert!(file_search.keywords.contains(&"search".to_string()));
        assert!(file_search.keywords.contains(&"find".to_string()));
        assert!(file_search.keywords.contains(&"directory".to_string()));
        assert!(file_search.icon.is_some());
    }
    #[test]
    fn test_get_builtin_entries_includes_process_manager_command() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let process_manager = entries.iter().find(|e| e.id == "builtin-process-manager");
        assert!(
            process_manager.is_some(),
            "Process Manager builtin should exist in the main menu"
        );

        let process_manager = process_manager.unwrap();
        assert_eq!(
            process_manager.feature,
            BuiltInFeature::UtilityCommand(UtilityCommandType::ProcessManager)
        );
        assert!(process_manager.keywords.iter().any(|k| k == "process"));
        assert!(process_manager.keywords.iter().any(|k| k == "running"));
        assert!(process_manager.keywords.iter().any(|k| k == "kill"));
    }
    #[test]
    fn test_get_builtin_entries_includes_stop_all_processes_command() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let stop_all = entries
            .iter()
            .find(|e| e.id == "builtin-stop-all-processes");
        assert!(
            stop_all.is_some(),
            "Stop all running scripts builtin should exist in the main menu"
        );

        let stop_all = stop_all.unwrap();
        assert_eq!(
            stop_all.feature,
            BuiltInFeature::UtilityCommand(UtilityCommandType::StopAllProcesses)
        );
        assert!(stop_all.keywords.iter().any(|k| k == "stop"));
        assert!(stop_all.keywords.iter().any(|k| k == "kill"));
        assert!(stop_all.keywords.iter().any(|k| k == "terminate"));
    }
    #[test]
    fn test_builtin_descriptions_use_clear_action_phrasing() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        let notes = entries.iter().find(|e| e.id == "builtin-notes").unwrap();
        assert_eq!(
            notes.description,
            "Open quick notes and a scratchpad editor"
        );

        let quick_capture = entries
            .iter()
            .find(|e| e.id == "builtin-quick-capture")
            .unwrap();
        assert_eq!(
            quick_capture.description,
            "Capture a new note without opening the full Notes window"
        );

        let file_search = entries
            .iter()
            .find(|e| e.id == "builtin-file-search")
            .unwrap();
        assert_eq!(
            file_search.description,
            "Browse directories, search files, and open results"
        );

        let webcam = entries.iter().find(|e| e.id == "builtin-webcam").unwrap();
        assert_eq!(
            webcam.description,
            "Open the webcam prompt and capture a photo"
        );
    }
    #[test]
    fn test_file_search_feature_equality() {
        assert_eq!(BuiltInFeature::FileSearch, BuiltInFeature::FileSearch);
        assert_ne!(BuiltInFeature::FileSearch, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::FileSearch, BuiltInFeature::Notes);
    }
    #[test]
    fn test_system_action_hud_message_volume_presets() {
        assert_eq!(
            system_action_hud_message(SystemActionType::Volume0, None),
            Some("Volume 0%".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::Volume50, None),
            Some("Volume 50%".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::Volume100, None),
            Some("Volume 100%".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::VolumeMute, None),
            Some("Volume Muted".to_string())
        );
    }
    #[test]
    fn test_system_action_hud_message_dark_mode() {
        assert_eq!(
            system_action_hud_message(SystemActionType::ToggleDarkMode, Some(true)),
            Some("Dark Mode On".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::ToggleDarkMode, Some(false)),
            Some("Dark Mode Off".to_string())
        );
        assert_eq!(
            system_action_hud_message(SystemActionType::ToggleDarkMode, None),
            Some("Dark Mode Toggled".to_string())
        );
    }
