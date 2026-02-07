                                if let Message::FileSearch {
                                    request_id,
                                    query,
                                    only_in,
                                } = &msg
                                {
                                    logging::log(
                                        "EXEC",
                                        &format!(
                                            "FileSearch request: query='{}', only_in={:?}",
                                            query, only_in
                                        ),
                                    );

                                    // Check if query looks like a directory path
                                    // If so, list directory contents instead of searching
                                    let results = if file_search::is_directory_path(query) {
                                        logging::log(
                                            "EXEC",
                                            &format!("Detected directory path, listing: {}", query),
                                        );
                                        file_search::list_directory(
                                            query,
                                            file_search::DEFAULT_CACHE_LIMIT,
                                        )
                                    } else {
                                        file_search::search_files(
                                            query,
                                            only_in.as_deref(),
                                            file_search::DEFAULT_SEARCH_LIMIT,
                                        )
                                    };

                                    let file_entries: Vec<protocol::FileSearchResultEntry> =
                                        results
                                            .into_iter()
                                            .map(|f| protocol::FileSearchResultEntry {
                                                path: f.path,
                                                name: f.name,
                                                is_directory: f.file_type
                                                    == file_search::FileType::Directory,
                                                size: Some(f.size),
                                                modified_at: chrono::DateTime::from_timestamp(
                                                    f.modified as i64,
                                                    0,
                                                )
                                                .map(|dt| dt.to_rfc3339()),
                                            })
                                            .collect();

                                    let response = Message::file_search_result(
                                        request_id.clone(),
                                        file_entries,
                                    );

                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!("Failed to send file search response: {}", e),
                                        );
                                    }
                                    continue;
                                }

                                // Handle GetWindowBounds directly (no UI needed)
                                if let Message::GetWindowBounds { request_id } = &msg {
                                    logging::log(
                                        "EXEC",
                                        &format!("GetWindowBounds request: {}", request_id),
                                    );

                                    #[cfg(target_os = "macos")]
                                    let bounds_json = {
                                        if let Some(window) = window_manager::get_main_window() {
                                            unsafe {
                                                // Get the window frame
                                                let frame: NSRect = msg_send![window, frame];

                                                // Get the PRIMARY screen's height for coordinate conversion
                                                // macOS uses bottom-left origin, we convert to top-left
                                                let screens: id =
                                                    msg_send![class!(NSScreen), screens];
                                                let main_screen: id =
                                                    msg_send![screens, firstObject];
                                                let main_screen_frame: NSRect =
                                                    msg_send![main_screen, frame];
                                                let primary_screen_height =
                                                    main_screen_frame.size.height;

                                                // Convert from bottom-left origin (macOS) to top-left origin
                                                let flipped_y = primary_screen_height
                                                    - frame.origin.y
                                                    - frame.size.height;

                                                logging::log("EXEC", &format!(
                                                    "Window bounds: x={:.0}, y={:.0}, width={:.0}, height={:.0}",
                                                    frame.origin.x, flipped_y, frame.size.width, frame.size.height
                                                ));

                                                // Create JSON string with bounds
                                                format!(
                                                    r#"{{"x":{},"y":{},"width":{},"height":{}}}"#,
                                                    frame.origin.x as f64,
                                                    flipped_y as f64,
                                                    frame.size.width as f64,
                                                    frame.size.height as f64
                                                )
                                            }
                                        } else {
                                            logging::log(
                                                "EXEC",
                                                "GetWindowBounds: Main window not registered",
                                            );
                                            r#"{"error":"Main window not found"}"#.to_string()
                                        }
                                    };

                                    #[cfg(not(target_os = "macos"))]
                                    let bounds_json =
                                        r#"{"error":"Not supported on this platform"}"#.to_string();

                                    let response = Message::Submit {
                                        id: request_id.clone(),
                                        value: Some(bounds_json),
                                    };
                                    logging::log(
                                        "EXEC",
                                        &format!("Sending window bounds response: {:?}", response),
                                    );
                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "Failed to send window bounds response: {}",
                                                e
                                            ),
                                        );
                                    }
                                    continue;
                                }

                                // Handle AI SDK messages that can be processed directly
                                if let Some(response) = crate::ai::try_handle_ai_message(&msg) {
                                    logging::log(
                                        "EXEC",
                                        &format!("AI SDK message handled: {:?}", msg),
                                    );
                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!("Failed to send AI SDK response: {}", e),
                                        );
                                    }
                                    continue;
                                }

                                // Handle GetState - needs UI state, forward to UI thread
