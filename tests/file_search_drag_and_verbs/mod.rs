// Integration tests for file-search drag-out support and verb availability.
//
// These tests verify:
// 1. The drag payload struct exists and can be constructed from FileResult
// 2. The native drag function exists with the correct platform signature
// 3. The shared secondary-command contract advertises the right verbs
//    for files vs directories
// 4. The file search list renders drag-capable rows

use script_kit_gpui::file_search::{FileDragPayload, FileInfo, FileResult, FileType};

// ──────────────────────────────────────────────────────────────────────
// Drag payload creation
// ──────────────────────────────────────────────────────────────────────

#[test]
fn drag_payload_from_file_result() {
    let result = FileResult {
        path: "/Users/test/Documents/report.pdf".to_string(),
        name: "report.pdf".to_string(),
        size: 1024,
        modified: 1700000000,
        file_type: FileType::Document,
    };
    let payload = FileDragPayload::from_result(&result);
    assert_eq!(payload.name, "report.pdf");
}

#[test]
fn drag_payload_from_directory_result() {
    let result = FileResult {
        path: "/Users/test/Projects".to_string(),
        name: "Projects".to_string(),
        size: 0,
        modified: 1700000000,
        file_type: FileType::Directory,
    };
    let payload = FileDragPayload::from_result(&result);
    assert_eq!(payload.name, "Projects");
}

// ──────────────────────────────────────────────────────────────────────
// Native drag function exists with correct platform signature
// ──────────────────────────────────────────────────────────────────────

/// Source text of path_actions.rs for verifying the native drag function.
const PATH_ACTIONS_SOURCE: &str = include_str!("../../src/platform/path_actions.rs");

#[test]
fn native_file_drag_function_exists() {
    assert!(
        PATH_ACTIONS_SOURCE.contains("pub fn begin_native_file_drag(path: &str)"),
        "begin_native_file_drag must exist in platform/path_actions.rs"
    );
}

#[test]
fn native_file_drag_has_macos_implementation() {
    assert!(
        PATH_ACTIONS_SOURCE.contains("beginDraggingSessionWithItems"),
        "macOS implementation must use beginDraggingSessionWithItems"
    );
}

#[test]
fn native_file_drag_has_non_macos_stub() {
    assert!(
        PATH_ACTIONS_SOURCE.contains(
            r#"cfg(not(target_os = "macos"))]
pub fn begin_native_file_drag"#
        ),
        "must have a non-macOS stub for begin_native_file_drag"
    );
}

#[test]
fn native_file_drag_registers_dragging_source_protocol() {
    assert!(
        PATH_ACTIONS_SOURCE.contains("ensure_dragging_source_protocol"),
        "must register NSDraggingSource protocol before starting drag"
    );
    assert!(
        PATH_ACTIONS_SOURCE.contains("sourceOperationMaskForDraggingContext"),
        "must implement the required NSDraggingSource method"
    );
}

#[test]
fn native_file_drag_uses_file_url_pasteboard_type() {
    assert!(
        PATH_ACTIONS_SOURCE.contains("public.file-url"),
        "must write file URL to pasteboard with public.file-url UTI"
    );
}

// ──────────────────────────────────────────────────────────────────────
// File search list renders drag-capable rows
// ──────────────────────────────────────────────────────────────────────

const FILE_SEARCH_SOURCE: &str = include_str!("../../src/render_builtins/file_search.rs");

#[test]
fn file_search_list_has_on_drag_handler() {
    assert!(
        FILE_SEARCH_SOURCE.contains(".on_drag(drag_payload"),
        "file search list rows must have an on_drag handler"
    );
}

#[test]
fn file_search_drag_triggers_native_drag() {
    assert!(
        FILE_SEARCH_SOURCE.contains("begin_native_file_drag"),
        "on_drag handler must call begin_native_file_drag"
    );
}

#[test]
fn file_search_drag_uses_platform_module_path() {
    // Since path_actions.rs is include!()'d into platform/mod.rs,
    // the call must go through crate::platform::, not crate::platform::path_actions::
    assert!(
        FILE_SEARCH_SOURCE.contains("crate::platform::begin_native_file_drag"),
        "must call through crate::platform:: (path_actions is include!() into platform)"
    );
}

#[test]
fn file_search_drag_creates_drag_preview_entity() {
    assert!(
        FILE_SEARCH_SOURCE.contains("cx.new(|_| file_search::FileDragPayload"),
        "on_drag constructor must create a FileDragPayload entity for preview"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Verb availability: shared contract for files vs directories
// ──────────────────────────────────────────────────────────────────────

const FILE_PATH_SOURCE: &str = include_str!("../../src/actions/builders/file_path.rs");

#[test]
fn secondary_commands_contract_exists() {
    assert!(
        FILE_PATH_SOURCE.contains("FILE_SEARCH_SECONDARY_COMMANDS"),
        "must define FILE_SEARCH_SECONDARY_COMMANDS constant"
    );
    assert!(
        FILE_PATH_SOURCE.contains("FileSearchSecondaryCommand"),
        "must define FileSearchSecondaryCommand struct"
    );
}

#[test]
fn all_expected_verbs_are_in_contract() {
    let expected_action_ids = [
        "rename_path",
        "move_path",
        "copy_filename",
        "open_in_editor",
        "copy_path",
        "move_to_trash",
        "open_in_terminal",
        "quick_look",
        "show_info",
    ];
    for action_id in &expected_action_ids {
        assert!(
            FILE_PATH_SOURCE.contains(&format!("action_id: \"{}\"", action_id)),
            "secondary commands must include action_id: {}",
            action_id
        );
    }
}

#[test]
fn quick_look_is_files_only() {
    // Find the quick_look entry and verify files_only: true
    let ql_pos = FILE_PATH_SOURCE
        .find("action_id: \"quick_look\"")
        .expect("quick_look must exist in secondary commands");
    let ql_section = &FILE_PATH_SOURCE[ql_pos..ql_pos + 500];
    assert!(
        ql_section.contains("files_only: true"),
        "quick_look must be files_only: true"
    );
}

#[test]
fn show_info_is_macos_only() {
    let info_pos = FILE_PATH_SOURCE
        .find("action_id: \"show_info\"")
        .expect("show_info must exist in secondary commands");
    let info_section = &FILE_PATH_SOURCE[info_pos..info_pos + 500];
    assert!(
        info_section.contains("macos_only: true"),
        "show_info must be macos_only: true"
    );
}

#[test]
fn rename_and_move_work_for_both_files_and_directories() {
    for action_id in &["rename_path", "move_path"] {
        let pos = FILE_PATH_SOURCE
            .find(&format!("action_id: \"{}\"", action_id))
            .unwrap_or_else(|| panic!("{} must exist", action_id));
        let section = &FILE_PATH_SOURCE[pos..pos + 500];
        assert!(
            section.contains("files_only: false"),
            "{} must have files_only: false (works for both files and dirs)",
            action_id
        );
    }
}

#[test]
fn key_resolver_uses_shared_contract() {
    assert!(
        FILE_PATH_SOURCE.contains("fn resolve_file_search_secondary_action_id("),
        "must define resolve_file_search_secondary_action_id"
    );
    assert!(
        FILE_PATH_SOURCE.contains("FILE_SEARCH_SECONDARY_COMMANDS"),
        "key resolver must reference the shared command array"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Key handler uses the shared contract (not hardcoded)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn file_search_key_handler_reads_from_shared_contract() {
    assert!(
        FILE_SEARCH_SOURCE.contains("resolve_file_search_secondary_action_id"),
        "file_search.rs key handler must use the shared key resolver"
    );
}

#[test]
fn file_search_footer_advertises_actions_with_selection_or_directory() {
    // Selected-file branch must advertise ⌘K Actions.
    let selected_branch = FILE_SEARCH_SOURCE
        .split("if let Some(file) = selected_file.as_ref()")
        .nth(1)
        .expect("selected file footer branch must exist");
    let selected_branch = selected_branch
        .split("} else if self.file_search_current_dir.is_some()")
        .next()
        .expect("selected file footer branch must end before directory branch");

    assert!(
        selected_branch.contains("\\u{2318}K Actions"),
        "selected file footer branch must advertise actions"
    );

    // Directory-browse branch (no selection but browsing a directory) must
    // also advertise ⌘K Actions for current-directory verbs.
    let dir_branch = FILE_SEARCH_SOURCE
        .split("} else if self.file_search_current_dir.is_some()")
        .nth(1)
        .expect("directory-browse footer branch must exist");
    let dir_branch = dir_branch
        .split("} else if is_loading {")
        .next()
        .expect("directory-browse branch must end before loading branch");

    assert!(
        dir_branch.contains("\\u{2318}K Actions"),
        "directory-browse footer branch must advertise actions for current-directory verbs"
    );

    // Non-directory no-selection branches (loading, empty, fallback) must NOT
    // advertise ⌘K Actions.  Split from the directory branch onward to isolate
    // the footer's loading/empty/fallback arms (not the preview panel's).
    let after_dir_branch = FILE_SEARCH_SOURCE
        .split("} else if self.file_search_current_dir.is_some()")
        .nth(1)
        .expect("directory-browse footer branch must exist for tail split");
    let no_dir_branch = after_dir_branch
        .split("} else if is_loading {")
        .nth(1)
        .expect("no-selection footer branches must exist after directory branch");

    assert!(
        !no_dir_branch.contains("\\u{2318}K Actions"),
        "file search footer must not advertise actions when no selection and no directory context"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Verb availability via FileInfo (pure logic tests)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn file_info_from_result_preserves_is_dir() {
    let file_result = FileResult {
        path: "/tmp/test.txt".to_string(),
        name: "test.txt".to_string(),
        size: 100,
        modified: 1700000000,
        file_type: FileType::File,
    };
    let info = FileInfo::from_result(&file_result);
    assert!(!info.is_dir);
    assert_eq!(info.path, "/tmp/test.txt");

    let dir_result = FileResult {
        path: "/tmp/folder".to_string(),
        name: "folder".to_string(),
        size: 0,
        modified: 1700000000,
        file_type: FileType::Directory,
    };
    let info = FileInfo::from_result(&dir_result);
    assert!(info.is_dir);
}

// ──────────────────────────────────────────────────────────────────────
// Action builder uses shared contract
// ──────────────────────────────────────────────────────────────────────

#[test]
fn action_builder_uses_shared_loop() {
    assert!(
        FILE_PATH_SOURCE.contains("for command in FILE_SEARCH_SECONDARY_COMMANDS.iter()"),
        "get_file_context_actions must loop over the shared contract"
    );
}

#[test]
fn action_builder_filters_by_supports() {
    assert!(
        FILE_PATH_SOURCE.contains("command.supports(file_info.is_dir)"),
        "action builder must filter commands by supports(is_dir)"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Drag/Escape regression: native drag handoff clears GPUI drag state
// ──────────────────────────────────────────────────────────────────────

#[test]
fn file_search_native_drag_stops_gpui_drag_state() {
    assert!(
        FILE_SEARCH_SOURCE.contains("cx.stop_active_drag(window)"),
        "native file drag handoff must stop GPUI drag state so Escape keeps dismissing the view"
    );
}

// ──────────────────────────────────────────────────────────────────────
// File-search actions use context builder (not file-only builder)
// ──────────────────────────────────────────────────────────────────────

const ACTIONS_SOURCE: &str = include_str!("../../src/render_builtins/actions.rs");

#[test]
fn file_search_actions_use_context_builder_not_file_only_builder() {
    assert!(
        ACTIONS_SOURCE.contains("ActionsDialog::with_file_search_context"),
        "file search actions should be built from file-search context, not only ActionsDialog::with_file"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Current-directory action IDs exist
// ──────────────────────────────────────────────────────────────────────

#[test]
fn current_directory_action_ids_exist() {
    for action_id in [
        "refresh_directory",
        "reveal_current_directory",
        "open_current_directory_in_terminal",
        "copy_current_directory_path",
        "sort_name_asc",
        "sort_name_desc",
        "sort_modified_desc",
        "sort_modified_asc",
    ] {
        assert!(
            FILE_PATH_SOURCE.contains(&format!("\"file:{action_id}\"")),
            "missing current-directory action: {action_id}"
        );
    }
}

// ──────────────────────────────────────────────────────────────────────
// App state tracks file_search_sort_mode
// ──────────────────────────────────────────────────────────────────────

#[test]
fn app_state_tracks_file_search_sort_mode() {
    let source = include_str!("../../src/main_sections/app_state.rs");
    assert!(
        source.contains("file_search_sort_mode"),
        "ScriptListApp state must track file_search_sort_mode for directory actions"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Sort mode is reset when opening a fresh file search session
// ──────────────────────────────────────────────────────────────────────

#[test]
fn sort_mode_resets_on_fresh_file_search_open() {
    let source = include_str!("../../src/app_impl/filter_input_core.rs");
    let open_fn_start = source
        .find("fn open_file_search_view(")
        .expect("open_file_search_view must exist");
    let open_fn = &source[open_fn_start..];
    assert!(
        open_fn.contains("file_search_sort_mode"),
        "open_file_search_view must reset file_search_sort_mode"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Toggle function accepts optional file (directory-only mode)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn toggle_file_search_actions_accepts_optional_file() {
    assert!(
        ACTIONS_SOURCE.contains("selected_file: Option<&file_search::FileResult>"),
        "toggle_file_search_actions must accept Option<&FileResult> for directory-only mode"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Sort mode preserved when browsing into subdirectory
// ──────────────────────────────────────────────────────────────────────

#[test]
fn sort_mode_preserved_on_internal_browse() {
    let source = include_str!("../../src/app_impl/filter_input_core.rs");
    assert!(
        source.contains(
            "let preserve_sort_mode = matches!(self.current_view, AppView::FileSearchView"
        ),
        "open_file_search_view must detect when already in file search to preserve sort mode"
    );
    assert!(
        source.contains("if !preserve_sort_mode {"),
        "open_file_search_view must conditionally reset sort mode"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Stream completion uses active sort mode, not hardcoded name sort
// ──────────────────────────────────────────────────────────────────────

#[test]
fn directory_stream_completion_uses_active_sort_mode() {
    let source = include_str!("../../src/app_impl/filter_input_change.rs");
    assert!(
        source.contains("self.apply_file_search_sort_mode();"),
        "directory stream completion must honor the active file_search_sort_mode"
    );
    assert!(
        !source.contains("self.sort_directory_results();"),
        "stream completion must not use hardcoded sort_directory_results"
    );
}

// ──────────────────────────────────────────────────────────────────────
// refresh_directory restores focus and shows correct HUD
// ──────────────────────────────────────────────────────────────────────

const FILES_ACTION_SOURCE: &str = include_str!("../../src/app_actions/handle_action/files.rs");

#[test]
fn refresh_directory_shows_hud_and_restores_focus() {
    let handler_start = FILES_ACTION_SOURCE
        .find("\"refresh_directory\" =>")
        .expect("refresh_directory handler must exist");
    let handler_section =
        &FILES_ACTION_SOURCE[handler_start..(handler_start + 1400).min(FILES_ACTION_SOURCE.len())];

    assert!(
        handler_section.contains("\"Refreshed Directory\""),
        "refresh_directory must show 'Refreshed Directory' HUD"
    );
    assert!(
        handler_section.contains("restore_file_search_input_focus"),
        "refresh_directory must restore focus to the main filter input"
    );
    assert!(
        handler_section.contains("restart_file_search_stream_for_query"),
        "refresh_directory must restart the directory stream"
    );
}

#[test]
fn refresh_directory_returns_error_when_no_directory_active() {
    let handler_start = FILES_ACTION_SOURCE
        .find("\"refresh_directory\" =>")
        .expect("refresh_directory handler must exist");
    let handler_section =
        &FILES_ACTION_SOURCE[handler_start..(handler_start + 400).min(FILES_ACTION_SOURCE.len())];

    assert!(
        handler_section.contains("current_file_search_directory_abs()"),
        "refresh_directory must check for active browsed directory"
    );
    assert!(
        handler_section.contains("DispatchOutcome::error("),
        "refresh_directory must return error when no directory is active"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Filtered directory views obey the selected sort mode
// ──────────────────────────────────────────────────────────────────────

#[test]
fn filtered_directory_recompute_reapplies_active_sort_mode() {
    let source = include_str!("../../src/app_execute/utility_views.rs");
    let recompute_start = source
        .find("pub fn recompute_file_search_display_indices(")
        .expect("recompute_file_search_display_indices must exist");
    let recompute_section = &source[recompute_start..(recompute_start + 2600).min(source.len())];

    assert!(
        recompute_section.contains("let (filter_pattern, is_directory_query)"),
        "recompute_file_search_display_indices must detect directory queries separately from filter text"
    );
    assert!(
        recompute_section
            .contains("self.sort_file_search_display_indices_for_directory(&mut indices);"),
        "filtered directory results must be re-sorted with the active file_search_sort_mode"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Directory stream batches reapply the active sort mode before recompute
// ──────────────────────────────────────────────────────────────────────

#[test]
fn directory_stream_batches_reapply_active_sort_mode_before_recompute() {
    let source = include_str!("../../src/app_impl/filter_input_change.rs");
    let block_start = source
        .find("if needs_recompute {")
        .expect("needs_recompute block must exist");
    let block = &source[block_start..(block_start + 1200).min(source.len())];

    assert!(
        block.contains("let is_directory_query = matches!("),
        "stream batch handling must identify directory queries before recompute"
    );
    assert!(
        block.contains("self.apply_file_search_sort_mode();"),
        "stream batch handling must reapply the active sort mode before recomputing indices"
    );
}

#[test]
fn directory_stream_batches_keep_auto_selection_pinned_to_first_row() {
    let source = include_str!("../../src/app_impl/filter_input_change.rs");

    assert!(
        source.contains("self.file_search_selection_mode == FileSearchSelectionMode::AutoFirst"),
        "selection restore must branch on file-search selection mode"
    );
    assert!(
        source.contains("let next_index = if pin_to_first_row {\n            0"),
        "auto selection mode must keep streamed directory updates pinned to the first visible row"
    );
}

#[test]
fn file_search_user_navigation_locks_selection_mode() {
    let arrow_source = include_str!("../../src/app_impl/startup_new_arrow.rs");
    let render_source = include_str!("../../src/render_builtins/file_search.rs");

    assert!(
        arrow_source.contains("this.lock_file_search_selection_to_user_choice();"),
        "file-search arrow navigation must mark selection as user-owned"
    );
    assert!(
        render_source.contains("this.lock_file_search_selection_to_user_choice();"),
        "file-search row clicks must mark selection as user-owned"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Sort mode paths emit verification logs
// ──────────────────────────────────────────────────────────────────────

#[test]
fn sort_mode_paths_emit_verification_logs() {
    let source = include_str!("../../src/app_actions/handle_action/files.rs");
    assert!(
        source.contains("event = \"sort_action_selected\""),
        "sort action handler must emit a structured log when a sort action is chosen"
    );
    assert!(
        source.contains("event = \"apply_file_search_sort_mode\""),
        "sort application must emit a structured log for runtime verification"
    );
}
