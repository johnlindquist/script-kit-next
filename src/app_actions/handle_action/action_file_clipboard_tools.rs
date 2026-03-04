            "open_file" | "open_directory" | "quick_look" | "open_with" | "show_info" => {
                if let Some(path) = self.file_search_actions_path.clone() {
                    logging::log("UI", &format!("File action '{}': {}", action_id, path));

                    let result = match action_id.as_str() {
                        "open_file" | "open_directory" => crate::file_search::open_file(&path),
                        "quick_look" => crate::file_search::quick_look(&path),
                        "open_with" => crate::file_search::open_with(&path),
                        "show_info" => crate::file_search::show_info(&path),
                        _ => Ok(()),
                    };

                    match result {
                        Ok(()) => {
                            if let Some(message) = file_search_action_success_hud(&action_id) {
                                self.show_hud(message.to_string(), Some(HUD_SHORT_MS), cx);
                            }
                            self.file_search_actions_path = None;
                            if action_id == "open_file" || action_id == "open_directory" {
                                self.hide_main_and_reset(cx);
                            }
                        }
                        Err(e) => {
                            logging::log(
                                "ERROR",
                                &format!(
                                    "File search action '{}' failed for '{}': {}",
                                    action_id, path, e
                                ),
                            );
                            let prefix = file_search_action_error_hud_prefix(&action_id)
                                .unwrap_or("Action failed");
                            self.show_hud(format!("{}: {}", prefix, e), Some(HUD_LONG_MS), cx);
                            self.file_search_actions_path = None;
                        }
                    }
                }
            }
            "copy_filename" => {
                if let Some(ref path) = self.file_search_actions_path {
                    let filename = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");
                    logging::log("UI", &format!("Copy filename: {}", filename));
                    #[cfg(target_os = "macos")]
                    {
                        let _ = self.pbcopy(filename);
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        use arboard::Clipboard;
                        let _ = Clipboard::new().and_then(|mut c| c.set_text(filename));
                    }
                    self.show_hud(format!("Copied: {}", filename), Some(HUD_MEDIUM_MS), cx);
                    self.file_search_actions_path = None;
                    self.hide_main_and_reset(cx);
                }
            }
            "clipboard_open_with" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                    self.show_hud(
                        "Failed to load clipboard content".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                };

                let full_entry = clipboard_history::ClipboardEntry {
                    id: entry.id.clone(),
                    content,
                    content_type: entry.content_type,
                    timestamp: entry.timestamp,
                    pinned: entry.pinned,
                    ocr_text: entry.ocr_text.clone(),
                };

                let temp_path = match clipboard_history::save_entry_to_temp_file(&full_entry) {
                    Ok(path) => path,
                    Err(e) => {
                        logging::log("ERROR", &format!("Failed to save temp file: {}", e));
                        self.show_hud("Failed to save temp file".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    }
                };

                #[cfg(target_os = "macos")]
                {
                    let path_str = temp_path.to_string_lossy().to_string();
                    if let Err(e) = crate::file_search::open_with(&path_str) {
                        logging::log("ERROR", &format!("Open With failed: {}", e));
                        self.show_hud("Failed to open \"Open With\"".to_string(), Some(HUD_MEDIUM_MS), cx);
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    let _ = temp_path;
                    self.show_hud(
                        "\"Open With\" is only supported on macOS".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "clipboard_annotate_cleanshot" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "CleanShot actions are only available for images".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                }

                #[cfg(target_os = "macos")]
                {
                    if let Err(e) = clipboard_history::copy_entry_to_clipboard(&entry.id) {
                        logging::log("ERROR", &format!("Failed to copy image: {}", e));
                        self.show_hud("Failed to copy image".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    }

                    let url = "cleanshot://open-from-clipboard";
                    match std::process::Command::new("open").arg(url).spawn() {
                        Ok(_) => {
                            self.show_hud("Opening CleanShot X…".to_string(), Some(HUD_SHORT_MS), cx);
                            self.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open CleanShot X: {}", e));
                            self.show_hud("Failed to open CleanShot X".to_string(), Some(HUD_MEDIUM_MS), cx);
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.show_hud(
                        "CleanShot actions are only supported on macOS".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "clipboard_upload_cleanshot" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "CleanShot actions are only available for images".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                }

                #[cfg(target_os = "macos")]
                {
                    let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                        self.show_hud("Failed to load image content".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    };

                    let Some(png_bytes) = clipboard_history::content_to_png_bytes(&content) else {
                        self.show_hud("Failed to decode image".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    };

                    let temp_path = std::env::temp_dir()
                        .join(format!("script-kit-clipboard-{}.png", uuid::Uuid::new_v4()));

                    if let Err(e) = std::fs::write(&temp_path, png_bytes) {
                        logging::log("ERROR", &format!("Failed to write temp image: {}", e));
                        self.show_hud("Failed to save image".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    }

                    let path_str = temp_path.to_string_lossy();
                    let encoded_path = self.percent_encode_for_url(&path_str);
                    let url = format!(
                        "cleanshot://open-annotate?filepath={}&action=upload",
                        encoded_path
                    );

                    match std::process::Command::new("open").arg(&url).spawn() {
                        Ok(_) => {
                            self.show_hud(
                                "Opening CleanShot X upload…".to_string(),
                                Some(HUD_SHORT_MS),
                                cx,
                            );
                            self.hide_main_and_reset(cx);
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("Failed to open CleanShot X: {}", e));
                            self.show_hud("Failed to open CleanShot X".to_string(), Some(HUD_MEDIUM_MS), cx);
                        }
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.show_hud(
                        "CleanShot actions are only supported on macOS".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                }
            }
            "clipboard_ocr" => {
                let Some(entry) = selected_clipboard_entry else {
                    self.show_hud("No clipboard entry selected".to_string(), Some(HUD_MEDIUM_MS), cx);
                    return;
                };

                if entry.content_type != clipboard_history::ContentType::Image {
                    self.show_hud(
                        "OCR is only available for images".to_string(),
                        Some(HUD_MEDIUM_MS),
                        cx,
                    );
                    return;
                }

                // Check if we already have cached OCR text
                if let Some(ref cached_text) = entry.ocr_text {
                    if !cached_text.trim().is_empty() {
                        logging::log("OCR", "Using cached OCR text");
                        #[cfg(target_os = "macos")]
                        {
                            let _ = self.pbcopy(cached_text);
                        }
                        #[cfg(not(target_os = "macos"))]
                        {
                            use arboard::Clipboard;
                            let _ =
                                Clipboard::new().and_then(|mut c| c.set_text(cached_text.clone()));
                        }
                        self.show_hud("Copied text from image".to_string(), Some(HUD_SHORT_MS), cx);
                        self.hide_main_and_reset(cx);
                        return;
                    }
                }

                #[cfg(all(target_os = "macos", feature = "ocr"))]
                {
                    // Get image content
                    let Some(content) = clipboard_history::get_entry_content(&entry.id) else {
                        self.show_hud("Failed to load image content".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    };

                    // Decode to RGBA bytes for OCR
                    let Some((width, height, rgba_bytes)) =
                        clipboard_history::decode_to_rgba_bytes(&content)
                    else {
                        self.show_hud("Failed to decode image".to_string(), Some(HUD_MEDIUM_MS), cx);
                        return;
                    };

                    logging::log(
                        "OCR",
                        &format!("Starting OCR on {}x{} image", width, height),
                    );
                    self.show_hud("Extracting text...".to_string(), Some(HUD_SHORT_MS), cx);

                    // Perform OCR synchronously (it runs on a background thread internally)
                    // For a truly async approach, we'd need to integrate with GPUI's async system
                    let entry_id = entry.id.clone();
                    match script_kit_gpui::ocr::extract_text_from_rgba(width, height, &rgba_bytes) {
                        Ok(text) => {
                            if text.trim().is_empty() {
                                logging::log("OCR", "No text found in image");
                                self.show_hud("No text found in image".to_string(), Some(HUD_MEDIUM_MS), cx);
                            } else {
                                logging::log(
                                    "OCR",
                                    &format!("Extracted {} characters", text.len()),
                                );

                                // Cache the OCR result
                                let _ = clipboard_history::update_ocr_text(&entry_id, &text);

                                // Copy to clipboard
                                #[cfg(target_os = "macos")]
                                {
                                    let _ = self.pbcopy(&text);
                                }
                                #[cfg(not(target_os = "macos"))]
                                {
                                    use arboard::Clipboard;
                                    let _ =
                                        Clipboard::new().and_then(|mut c| c.set_text(text.clone()));
                                }

                                self.show_hud("Copied text from image".to_string(), Some(HUD_SHORT_MS), cx);
                                self.hide_main_and_reset(cx);
                            }
                        }
                        Err(e) => {
                            logging::log("ERROR", &format!("OCR failed: {}", e));
                            self.show_hud(format!("OCR failed: {}", e), Some(HUD_LONG_MS), cx);
                        }
                    }
                }

                #[cfg(not(all(target_os = "macos", feature = "ocr")))]
                {
                    self.show_hud("OCR is only supported on macOS".to_string(), Some(HUD_MEDIUM_MS), cx);
                }
            }
            // Clipboard delete actions
