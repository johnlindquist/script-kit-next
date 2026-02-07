                                // Handle ClipboardHistory directly (no UI needed)
                                if let Message::ClipboardHistory {
                                    request_id,
                                    action,
                                    entry_id,
                                } = &msg
                                {
                                    logging::log(
                                        "EXEC",
                                        &format!("ClipboardHistory request: {:?}", action),
                                    );

                                    let response = match action {
                                        protocol::ClipboardHistoryAction::List => {
                                            let entries =
                                                clipboard_history::get_clipboard_history(100);
                                            let entry_data: Vec<protocol::ClipboardHistoryEntryData> = entries
                                                .into_iter()
                                                .map(|e| {
                                                    // Truncate large content to avoid pipe buffer issues
                                                    // Images are stored as base64 which can be huge
                                                    let content = match e.content_type {
                                                        clipboard_history::ContentType::Image => {
                                                            // For images, send a placeholder with metadata
                                                            format!("[image:{}]", e.id)
                                                        }
                                                        clipboard_history::ContentType::Text => {
                                                            // Truncate very long text entries
                                                            if e.content.len() > 1000 {
                                                                format!("{}...", &e.content[..1000])
                                                            } else {
                                                                e.content
                                                            }
                                                        }
                                                    };
                                                    protocol::ClipboardHistoryEntryData {
                                                        entry_id: e.id,
                                                        content,
                                                        content_type: match e.content_type {
                                                            clipboard_history::ContentType::Text => protocol::ClipboardEntryType::Text,
                                                            clipboard_history::ContentType::Image => protocol::ClipboardEntryType::Image,
                                                        },
                                                        timestamp: chrono::DateTime::from_timestamp(e.timestamp, 0)
                                                            .map(|dt| dt.to_rfc3339())
                                                            .unwrap_or_default(),
                                                        pinned: e.pinned,
                                                    }
                                                })
                                                .collect();
                                            Message::clipboard_history_list_response(
                                                request_id.clone(),
                                                entry_data,
                                            )
                                        }
                                        protocol::ClipboardHistoryAction::Pin => {
                                            if let Some(id) = entry_id {
                                                match clipboard_history::pin_entry(id) {
                                                    Ok(()) => Message::clipboard_history_success(
                                                        request_id.clone(),
                                                    ),
                                                    Err(e) => Message::clipboard_history_error(
                                                        request_id.clone(),
                                                        e.to_string(),
                                                    ),
                                                }
                                            } else {
                                                Message::clipboard_history_error(
                                                    request_id.clone(),
                                                    "Missing entry_id".to_string(),
                                                )
                                            }
                                        }
                                        protocol::ClipboardHistoryAction::Unpin => {
                                            if let Some(id) = entry_id {
                                                match clipboard_history::unpin_entry(id) {
                                                    Ok(()) => Message::clipboard_history_success(
                                                        request_id.clone(),
                                                    ),
                                                    Err(e) => Message::clipboard_history_error(
                                                        request_id.clone(),
                                                        e.to_string(),
                                                    ),
                                                }
                                            } else {
                                                Message::clipboard_history_error(
                                                    request_id.clone(),
                                                    "Missing entry_id".to_string(),
                                                )
                                            }
                                        }
                                        protocol::ClipboardHistoryAction::Remove => {
                                            if let Some(id) = entry_id {
                                                match clipboard_history::remove_entry(id) {
                                                    Ok(()) => Message::clipboard_history_success(
                                                        request_id.clone(),
                                                    ),
                                                    Err(e) => Message::clipboard_history_error(
                                                        request_id.clone(),
                                                        e.to_string(),
                                                    ),
                                                }
                                            } else {
                                                Message::clipboard_history_error(
                                                    request_id.clone(),
                                                    "Missing entry_id".to_string(),
                                                )
                                            }
                                        }
                                        protocol::ClipboardHistoryAction::Clear => {
                                            match clipboard_history::clear_history() {
                                                Ok(()) => Message::clipboard_history_success(
                                                    request_id.clone(),
                                                ),
                                                Err(e) => Message::clipboard_history_error(
                                                    request_id.clone(),
                                                    e.to_string(),
                                                ),
                                            }
                                        }
                                        protocol::ClipboardHistoryAction::TrimOversize => {
                                            match clipboard_history::trim_oversize_text_entries() {
                                                Ok(_) => Message::clipboard_history_success(
                                                    request_id.clone(),
                                                ),
                                                Err(e) => Message::clipboard_history_error(
                                                    request_id.clone(),
                                                    e.to_string(),
                                                ),
                                            }
                                        }
                                    };

                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "Failed to send clipboard history response: {}",
                                                e
                                            ),
                                        );
                                    }
                                    continue;
                                }

                                // Handle Clipboard read/write directly (no UI needed)
                                if let Message::Clipboard {
                                    id,
                                    action,
                                    format,
                                    content,
                                } = &msg
                                {
                                    logging::log(
                                        "EXEC",
                                        &format!(
                                            "Clipboard request: {:?} format: {:?}",
                                            action, format
                                        ),
                                    );

                                    // If no request ID, we can't send a response, so just handle and continue
                                    let req_id = match id {
                                        Some(rid) => rid.clone(),
                                        None => {
                                            // Handle clipboard operation without response
                                            if let protocol::ClipboardAction::Write = action {
                                                if let Some(text) = content {
                                                    use arboard::Clipboard;
                                                    if let Ok(mut clipboard) = Clipboard::new() {
                                                        let _ = clipboard.set_text(text.clone());
                                                    }
                                                }
                                            }
                                            continue;
                                        }
                                    };

                                    let response = match action {
                                        protocol::ClipboardAction::Read => {
                                            // Read from clipboard
                                            match format {
                                                Some(protocol::ClipboardFormat::Text) | None => {
                                                    use arboard::Clipboard;
                                                    match Clipboard::new() {
                                                        Ok(mut clipboard) => {
                                                            match clipboard.get_text() {
                                                                Ok(text) => Message::Submit {
                                                                    id: req_id,
                                                                    value: Some(text),
                                                                },
                                                                Err(e) => {
                                                                    logging::log("EXEC", &format!("Clipboard read error: {}", e));
                                                                    Message::Submit {
                                                                        id: req_id,
                                                                        value: Some(String::new()),
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            logging::log(
                                                                "EXEC",
                                                                &format!(
                                                                    "Clipboard init error: {}",
                                                                    e
                                                                ),
                                                            );
                                                            Message::Submit {
                                                                id: req_id,
                                                                value: Some(String::new()),
                                                            }
                                                        }
                                                    }
                                                }
                                                Some(protocol::ClipboardFormat::Image) => {
                                                    use arboard::Clipboard;
                                                    match Clipboard::new() {
                                                        Ok(mut clipboard) => {
                                                            match clipboard.get_image() {
                                                                Ok(img) => {
                                                                    // Convert image to base64
                                                                    use base64::Engine;
                                                                    let bytes = img.bytes.to_vec();
                                                                    let base64_str = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                                                    Message::Submit {
                                                                        id: req_id,
                                                                        value: Some(base64_str),
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    logging::log("EXEC", &format!("Clipboard read image error: {}", e));
                                                                    Message::Submit {
                                                                        id: req_id,
                                                                        value: Some(String::new()),
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            logging::log(
                                                                "EXEC",
                                                                &format!(
                                                                    "Clipboard init error: {}",
                                                                    e
                                                                ),
                                                            );
                                                            Message::Submit {
                                                                id: req_id,
                                                                value: Some(String::new()),
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        protocol::ClipboardAction::Write => {
                                            // Write to clipboard
                                            use arboard::Clipboard;
                                            match Clipboard::new() {
                                                Ok(mut clipboard) => {
                                                    if let Some(text) = content {
                                                        match clipboard.set_text(text.clone()) {
                                                            Ok(()) => {
                                                                logging::log("EXEC", &format!("Clipboard write success: {} bytes", text.len()));
                                                                Message::Submit {
                                                                    id: req_id,
                                                                    value: Some("ok".to_string()),
                                                                }
                                                            }
                                                            Err(e) => {
                                                                logging::log(
                                                                    "EXEC",
                                                                    &format!(
                                                                        "Clipboard write error: {}",
                                                                        e
                                                                    ),
                                                                );
                                                                Message::Submit {
                                                                    id: req_id,
                                                                    value: Some(String::new()),
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        logging::log(
                                                            "EXEC",
                                                            "Clipboard write: no content provided",
                                                        );
                                                        Message::Submit {
                                                            id: req_id,
                                                            value: Some(String::new()),
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    logging::log(
                                                        "EXEC",
                                                        &format!("Clipboard init error: {}", e),
                                                    );
                                                    Message::Submit {
                                                        id: req_id,
                                                        value: Some(String::new()),
                                                    }
                                                }
                                            }
                                        }
                                    };

                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!("Failed to send clipboard response: {}", e),
                                        );
                                    }
                                    continue;
                                }

                                // Handle WindowList directly (no UI needed)
