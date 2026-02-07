    #[test]
    fn cross_context_notes_all_titles_and_ids_nonempty() {
        let info = NotesInfo {
            has_selection: true,
            is_trash_view: false,
            auto_sizing_enabled: false,
        };
        let actions = get_notes_command_bar_actions(&info);
        for action in &actions {
            assert!(
                !action.title.is_empty(),
                "Empty title for id: {}",
                action.id
            );
            assert!(!action.id.is_empty(), "Empty id found");
        }
    }
