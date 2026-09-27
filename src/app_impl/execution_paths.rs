use super::path_action::PathAction;
use super::*;

impl ScriptListApp {
    pub(crate) fn execute_path_action(
        &mut self,
        action_id: &str,
        path_info: &PathInfo,
        path_prompt_entity: &Entity<PathPrompt>,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "UI",
            &format!(
                "Executing path action '{}' for: {} (is_dir={})",
                action_id, path_info.path, path_info.is_dir
            ),
        );

        let Some(action) = PathAction::from_action_id(action_id) else {
            logging::log("UI", &format!("Unknown path action: {}", action_id));
            return;
        };

        match action {
            PathAction::SelectFile | PathAction::OpenDirectory => {
                // For select/open, trigger submission through the path prompt
                // We need to trigger the submit callback with this path
                path_prompt_entity.update(cx, |prompt, cx| {
                    // Find the index of this path in filtered_entries and submit it
                    if let Some(idx) = prompt
                        .filtered_entries
                        .iter()
                        .position(|e| e.path == path_info.path)
                    {
                        prompt.set_selected_index(idx);
                    }
                    // For directories, navigate into them; for files, submit
                    if path_info.is_dir && action == PathAction::OpenDirectory {
                        prompt.navigate_to(&path_info.path, cx);
                    } else {
                        // Submit the selected path
                        let id = prompt.id.clone();
                        let path = path_info.path.clone();
                        (prompt.on_submit)(id, Some(path));
                    }
                });
            }
            PathAction::CopyPath => {
                // Copy full path to clipboard
                self.copy_to_clipboard_with_feedback(
                    &path_info.path,
                    format!("Copied path: {}", path_info.path),
                    false,
                    cx,
                );
            }
            PathAction::CopyFilename => {
                // Copy just the filename to clipboard
                self.copy_to_clipboard_with_feedback(
                    &path_info.name,
                    format!("Copied filename: {}", path_info.name),
                    false,
                    cx,
                );
            }
            PathAction::OpenInFinder => {
                let file_manager = if cfg!(target_os = "macos") {
                    "Finder"
                } else if cfg!(target_os = "windows") {
                    "Explorer"
                } else {
                    "File Manager"
                };

                match crate::file_search::reveal_in_finder(&path_info.path) {
                    Ok(_) => {
                        logging::log(
                            "UI",
                            &format!("Revealed in {}: {}", file_manager, path_info.path),
                        );
                        self.one_window_notice(
                            crate::line_notice::INFO,
                            format!("Opened in {}", file_manager),
                            cx,
                        );
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        logging::log(
                            "ERROR",
                            &format!("Failed to reveal in {}: {}", file_manager, e),
                        );
                        self.notice_failed(
                            format!("Failed to open in {}: {}", file_manager, e),
                            cx,
                        );
                    }
                }
            }
            PathAction::OpenInEditor => {
                // Open in configured editor
                let editor = self.config.get_editor();
                let path_str = path_info.path.clone();
                logging::log(
                    "UI",
                    &format!("Opening in editor '{}': {}", editor, path_str),
                );

                match std::process::Command::new(&editor).arg(&path_str).spawn() {
                    Ok(_) => {
                        logging::log("UI", &format!("Opened in editor: {}", path_str));
                        self.one_window_notice(
                            crate::line_notice::INFO,
                            format!("Opened in {}", editor),
                            cx,
                        );
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open in editor: {}", e));
                        self.notice_failed(format!("Failed to open in {}: {}", editor, e), cx);
                    }
                }
            }
            PathAction::OpenInQuickTerminal => {
                match crate::action_helpers::resolve_quick_terminal_cwd(std::path::Path::new(
                    &path_info.path,
                )) {
                    Ok(cwd) => {
                        logging::log(
                            "UI",
                            &format!("Opened Quick Terminal at: {}", cwd.display()),
                        );
                        self.open_quick_terminal(Some(cwd), cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to open Quick Terminal: {}", e));
                        self.notice_failed(format!("Failed to open Quick Terminal: {}", e), cx);
                    }
                }
            }
            PathAction::MoveToTrash => {
                let path_str = path_info.path.clone();
                let name = path_info.name.clone();
                match crate::file_search::move_to_trash(&path_str) {
                    Ok(trashed) => {
                        logging::log("UI", &format!("Moved to trash: {}", path_str));
                        self.last_output =
                            Some(SharedString::from(format!("Moved to Trash: {}", name)));
                        Self::refresh_path_prompt(path_prompt_entity, cx);

                        let path_prompt_entity = path_prompt_entity.clone();
                        self.one_window_done_undoable(
                            "trash",
                            format!("Moved \"{}\" to Trash", name),
                            Box::new(
                                move |this, cx| match crate::file_search::restore_from_trash(
                                    &trashed,
                                    std::path::Path::new(&path_str),
                                ) {
                                    Ok(()) => Self::refresh_path_prompt(&path_prompt_entity, cx),
                                    Err(e) => {
                                        this.notice_failed(format!("Couldn't restore: {}", e), cx)
                                    }
                                },
                            ),
                            cx,
                        );
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to move to trash: {}", e));
                        self.last_output = Some(SharedString::from("Failed to move to Trash"));
                        self.notice_failed(format!("Failed to move to Trash: {}", e), cx);
                    }
                }
            }
        }

        cx.notify();
    }

    fn refresh_path_prompt(path_prompt_entity: &Entity<PathPrompt>, cx: &mut Context<Self>) {
        path_prompt_entity.update(cx, |prompt, cx| {
            let current = prompt.current_path.clone();
            prompt.navigate_to(&current, cx);
        });
    }
}
