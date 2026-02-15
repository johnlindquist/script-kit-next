            "clipboard_save_file" => {
                let Some(entry) = selected_clipboard_entry.clone() else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud("Clipboard content unavailable".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                // Determine filename and content based on type
                let (file_content, extension) = match entry.content_type {
                    clipboard_history::ContentType::Text => (content.into_bytes(), "txt"),
                    clipboard_history::ContentType::Image => {
                        let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content)
                        else {
                            self.show_hud("Failed to decode image".to_string(), Some(HUD_MEDIUM_MS), cx);
                            return;
                        };
                        (png_bytes, "png")
                    }
                };

                // Get save location (Desktop or home)
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
                let desktop = home.join("Desktop");
                let save_dir = if desktop.exists() { desktop } else { home };

                // Generate unique filename with timestamp
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let filename = format!("clipboard-{}.{}", timestamp, extension);
                let save_path = save_dir.join(&filename);

                match std::fs::write(&save_path, &file_content) {
                    Ok(()) => {
                        logging::log("UI", &format!("Saved clipboard to: {:?}", save_path));
                        self.show_hud(format!("Saved to: {}", save_path.display()), Some(HUD_LONG_MS), cx);
                        self.reveal_in_finder(&save_path);
                        self.hide_main_and_reset(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to save file: {}", e));
                        self.show_hud(format!("Save failed: {}", e), Some(HUD_LONG_MS), cx);
                    }
                }
                return;
            }
            "clipboard_save_snippet" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Text {
                    self.show_hud(
                        "Only text can be saved as snippet".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                }

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud("Clipboard content unavailable".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                // Generate a default keyword from the first few words
                let default_keyword: String = content
                    .chars()
                    .take(20)
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
                    .to_lowercase();
                let default_keyword = if default_keyword.is_empty() {
                    "snippet".to_string()
                } else {
                    default_keyword
                };

                // Create snippet file in extensions directory
                let kenv = dirs::home_dir()
                    .map(|h| h.join(".kenv"))
                    .unwrap_or_else(|| std::path::PathBuf::from("/"));
                let extensions_dir = kenv.join("extensions");
                let snippets_file = extensions_dir.join("clipboard-snippets.md");

                // Ensure extensions directory exists
                if !extensions_dir.exists() {
                    if let Err(e) = std::fs::create_dir_all(&extensions_dir) {
                        logging::log("ERROR", &format!("Failed to create extensions dir: {}", e));
                        self.show_hud(format!("Failed to create snippets: {}", e), Some(HUD_LONG_MS), cx);
                        return;
                    }
                }

                // Generate unique keyword with timestamp suffix
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() % 10000)
                    .unwrap_or(0);
                let keyword = format!("{}-{}", default_keyword, timestamp);

                // Create snippet entry with proper fence handling
                let fence = if content.contains("```") {
                    "~~~~"
                } else {
                    "```"
                };
                let snippet_entry = format!(
                    "\n## {}\n\n{}\nname: {}\ntool: paste\nkeyword: {}\n{}\n\n{}paste\n{}\n{}\n",
                    keyword, fence, keyword, keyword, fence, fence, content, fence
                );

                // Append to snippets file
                let result = if snippets_file.exists() {
                    std::fs::OpenOptions::new()
                        .append(true)
                        .open(&snippets_file)
                        .and_then(|mut f| {
                            use std::io::Write;
                            f.write_all(snippet_entry.as_bytes())
                        })
                } else {
                    let header =
                        "# Clipboard Snippets\n\nSnippets created from clipboard history.\n";
                    std::fs::write(&snippets_file, format!("{}{}", header, snippet_entry))
                };

                match result {
                    Ok(()) => {
                        logging::log("UI", &format!("Created snippet with keyword: {}", keyword));
                        self.show_hud(
                            format!("Snippet created: type '{}' to paste", keyword),
                            Some(HUD_LONG_MS),
                            cx,
                        );
                        // Refresh scripts to pick up new snippet
                        self.refresh_scripts(cx);
                    }
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to save snippet: {}", e));
                        self.show_hud(format!("Save failed: {}", e), Some(HUD_LONG_MS), cx);
                    }
                }
                return;
            }

            // Scriptlet-specific actions
            "edit_scriptlet" => {
                logging::log("UI", "Edit scriptlet action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor (e.g., "/path/to/file.md#slug" -> "/path/to/file.md")
                            let path_str = file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::PathBuf::from(path_str);
                            self.edit_script(&path);
                            self.hide_main_and_reset(cx);
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "reveal_scriptlet_in_finder" => {
                logging::log("UI", "Reveal scriptlet in Finder action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str = file_path.split('#').next().unwrap_or(file_path);
                            let path = std::path::Path::new(path_str);
                            self.reveal_in_finder(path);
                            self.show_hud("Opened in Finder".to_string(), Some(HUD_SHORT_MS), cx);
                            self.hide_main_and_reset(cx);
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "copy_scriptlet_path" => {
                logging::log("UI", "Copy scriptlet path action");
                if let Some(result) = self.get_selected_result() {
                    if let scripts::SearchResult::Scriptlet(m) = result {
                        if let Some(ref file_path) = m.scriptlet.file_path {
                            // Extract just the path without the anchor
                            let path_str = file_path.split('#').next().unwrap_or(file_path);

                            #[cfg(target_os = "macos")]
                            {
                                match self.pbcopy(path_str) {
                                    Ok(_) => {
                                        logging::log(
                                            "UI",
                                            &format!(
                                                "Copied scriptlet path to clipboard: {}",
                                                path_str
                                            ),
                                        );
                                        self.show_hud(
                                            format!("Copied: {}", path_str),
                                            Some(HUD_MEDIUM_MS),
                                            cx,
                                        );
                                    }
                                    Err(e) => {
                                        logging::log("ERROR", &format!("pbcopy failed: {}", e));
                                        self.show_hud(
                                            "Failed to copy path".to_string(),
                                            Some(HUD_LONG_MS),
                                            cx,
                                        );
                                    }
                                }
                            }

                            #[cfg(not(target_os = "macos"))]
                            {
                                use arboard::Clipboard;
                                match Clipboard::new().and_then(|mut c| c.set_text(path_str)) {
                                    Ok(_) => {
                                        logging::log(
                                            "UI",
                                            &format!(
                                                "Copied scriptlet path to clipboard: {}",
                                                path_str
                                            ),
                                        );
                                        self.show_hud(
                                            format!("Copied: {}", path_str),
                                            Some(HUD_MEDIUM_MS),
                                            cx,
                                        );
                                    }
                                    Err(e) => {
                                        logging::log(
                                            "ERROR",
                                            &format!("Failed to copy path: {}", e),
                                        );
                                        self.show_hud(
                                            "Failed to copy path".to_string(),
                                            Some(HUD_LONG_MS),
                                            cx,
                                        );
                                    }
                                }
                            }
                            self.hide_main_and_reset(cx);
                        } else {
                            self.show_hud(
                                "Scriptlet has no source file path".to_string(),
                                Some(HUD_MEDIUM_MS),
                                cx,
                            );
                        }
                    } else {
                        self.show_hud(
                            "Selected item is not a scriptlet".to_string(),
                            Some(HUD_MEDIUM_MS),
                            cx,
                        );
                    }
                } else {
                    self.show_hud(
                        selection_required_message_for_action(&action_id).to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
