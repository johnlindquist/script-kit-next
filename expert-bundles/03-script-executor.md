🧩 Packing 2 file(s)...
📝 Files selected:
  • src/execute_script.rs
  • src/builtins.rs
This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 2
</notes>
</file_summary>

<directory_structure>
src/execute_script.rs
src/builtins.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/execute_script.rs">
impl ScriptListApp {
    fn execute_interactive(&mut self, script: &scripts::Script, cx: &mut Context<Self>) {
        logging::log(
            "EXEC",
            &format!("Starting interactive execution: {}", script.name),
        );

        // Store script path for error reporting in reader thread
        let script_path_for_errors = script.path.to_string_lossy().to_string();

        match executor::execute_script_interactive(&script.path) {
            Ok(session) => {
                logging::log("EXEC", "Interactive session started successfully");

                // Store PID for explicit cleanup (belt-and-suspenders approach)
                let pid = session.pid();
                self.current_script_pid = Some(pid);
                logging::log("EXEC", &format!("Stored script PID {} for cleanup", pid));

                *self.script_session.lock() = Some(session);

                // Create async_channel for script thread to send prompt messages to UI (event-driven)
                // P1-6: Use bounded channel to prevent unbounded memory growth from slow UI
                // Capacity of 100 is generous (scripts rarely send > 10 messages/sec)
                let (tx, rx) = async_channel::bounded(100);
                let rx_for_listener = rx.clone();
                self.prompt_receiver = Some(rx);

                // Spawn event-driven listener for prompt messages (replaces 50ms polling)
                cx.spawn(async move |this, cx| {
                    logging::log("EXEC", "Prompt message listener started (event-driven)");

                    // Event-driven: recv().await yields until a message arrives
                    while let Ok(msg) = rx_for_listener.recv().await {
                        logging::log("EXEC", &format!("Prompt message received: {:?}", msg));
                        let _ = cx.update(|cx| {
                            this.update(cx, |app, cx| {
                                app.handle_prompt_message(msg, cx);
                            })
                        });
                    }

                    logging::log("EXEC", "Prompt message listener exiting (channel closed)");
                })
                .detach();

                // We need separate threads for reading and writing to avoid deadlock
                // The read thread blocks on receive_message(), so we can't check for responses in the same loop

                // Take ownership of the session and split it
                let session = self.script_session.lock().take().unwrap();
                let split = session.split();

                let mut stdin = split.stdin;
                let mut stdout_reader = split.stdout_reader;
                // Capture stderr for error reporting - we'll read it in real-time for debugging
                let stderr_handle = split.stderr;
                // CRITICAL: Keep process_handle and child alive - they kill the process on drop!
                // We move them into the reader thread so they live until the script exits.
                let _process_handle = split.process_handle;
                let mut _child = split.child;

                // Stderr reader thread - forwards script stderr to logs in real-time
                if let Some(stderr) = stderr_handle {
                    std::thread::spawn(move || {
                        use std::io::BufRead;
                        let reader = std::io::BufReader::new(stderr);
                        for line in reader.lines() {
                            match line {
                                Ok(l) => logging::log("SCRIPT", &l),
                                Err(e) => {
                                    logging::log("SCRIPT", &format!("stderr read error: {}", e));
                                    break;
                                }
                            }
                        }
                        logging::log("SCRIPT", "stderr reader exiting");
                    });
                }

                // Now stderr_handle is consumed, we pass None to reader thread
                let stderr_handle: Option<std::process::ChildStderr> = None;

                // Channel for sending responses from UI to writer thread
                let (response_tx, response_rx) = mpsc::channel::<Message>();

                // Clone response_tx for the reader thread to handle direct responses
                // (e.g., getSelectedText, setSelectedText, checkAccessibility)
                let reader_response_tx = response_tx.clone();

                // Writer thread - handles sending responses to script
                std::thread::spawn(move || {
                    use std::io::Write;
                    use std::os::unix::io::AsRawFd;

                    // Log the stdin file descriptor for debugging
                    let fd = stdin.as_raw_fd();
                    logging::log("EXEC", &format!("Writer thread started, stdin fd={}", fd));

                    // Check if fd is a valid pipe
                    #[cfg(unix)]
                    {
                        let stat_result = unsafe {
                            let mut stat: libc::stat = std::mem::zeroed();
                            libc::fstat(fd, &mut stat)
                        };
                        if stat_result == 0 {
                            logging::log("EXEC", &format!("fd={} fstat succeeded", fd));
                        } else {
                            logging::log(
                                "EXEC",
                                &format!(
                                    "fd={} fstat FAILED: errno={}",
                                    fd,
                                    std::io::Error::last_os_error()
                                ),
                            );
                        }
                    }

                    loop {
                        match response_rx.recv() {
                            Ok(response) => {
                                let json = match protocol::serialize_message(&response) {
                                    Ok(j) => j,
                                    Err(e) => {
                                        logging::log(
                                            "EXEC",
                                            &format!("Failed to serialize: {}", e),
                                        );
                                        continue;
                                    }
                                };
                                logging::log(
                                    "EXEC",
                                    &format!("Writing to stdin fd={}: {}", fd, json),
                                );
                                let bytes = format!("{}\n", json);
                                let bytes_len = bytes.len();

                                // Check fd validity before write
                                let fcntl_result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
                                logging::log(
                                    "EXEC",
                                    &format!(
                                        "Pre-write fcntl(F_GETFD) on fd={}: {}",
                                        fd, fcntl_result
                                    ),
                                );

                                match stdin.write_all(bytes.as_bytes()) {
                                    Ok(()) => {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "Write succeeded: {} bytes to fd={}",
                                                bytes_len, fd
                                            ),
                                        );
                                    }
                                    Err(e) => {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "Failed to write {} bytes: {} (kind={:?})",
                                                bytes_len,
                                                e,
                                                e.kind()
                                            ),
                                        );
                                        break;
                                    }
                                }
                                if let Err(e) = stdin.flush() {
                                    logging::log(
                                        "EXEC",
                                        &format!("Failed to flush fd={}: {}", fd, e),
                                    );
                                    break;
                                }
                                logging::log("EXEC", &format!("Flush succeeded for fd={}", fd));
                            }
                            Err(_) => {
                                logging::log("EXEC", "Response channel closed, writer exiting");
                                break;
                            }
                        }
                    }
                    logging::log("EXEC", "Writer thread exiting");
                });

                // Reader thread - handles receiving messages from script (blocking is OK here)
                // CRITICAL: Move _process_handle and _child into this thread to keep them alive!
                // When the reader thread exits, they'll be dropped and the process killed.
                let script_path_clone = script_path_for_errors.clone();
                std::thread::spawn(move || {
                    // These variables keep the process alive - they're dropped when the thread exits
                    let _keep_alive_handle = _process_handle;
                    let mut keep_alive_child = _child;
                    let mut stderr_for_errors = stderr_handle;
                    let script_path = script_path_clone;

                    loop {
                        // Use next_message_graceful_with_handler to skip non-JSON lines and report parse issues
                        match stdout_reader.next_message_graceful_with_handler(|issue| {
                            let should_report = matches!(
                                issue.kind,
                                protocol::ParseIssueKind::InvalidPayload
                                    | protocol::ParseIssueKind::UnknownType
                            );
                            if !should_report {
                                return;
                            }

                            let summary = match issue.kind {
                                protocol::ParseIssueKind::InvalidPayload => issue
                                    .message_type
                                    .as_deref()
                                    .map(|message_type| {
                                        format!(
                                            "Invalid '{}' message payload from script",
                                            message_type
                                        )
                                    })
                                    .unwrap_or_else(|| {
                                        "Invalid message payload from script".to_string()
                                    }),
                                protocol::ParseIssueKind::UnknownType => issue
                                    .message_type
                                    .as_deref()
                                    .map(|message_type| {
                                        format!(
                                            "Unknown '{}' message type from script",
                                            message_type
                                        )
                                    })
                                    .unwrap_or_else(|| {
                                        "Unknown message type from script".to_string()
                                    }),
                                _ => "Protocol message issue from script".to_string(),
                            };

                            let mut details_lines = Vec::new();
                            details_lines.push(format!("Script: {}", script_path));
                            if let Some(ref message_type) = issue.message_type {
                                details_lines.push(format!("Type: {}", message_type));
                            }
                            if let Some(ref error) = issue.error {
                                details_lines.push(format!("Error: {}", error));
                            }
                            if !issue.raw_preview.is_empty() {
                                details_lines.push(format!("Preview: {}", issue.raw_preview));
                            }
                            let details = Some(details_lines.join("\n"));

                            let severity = match issue.kind {
                                protocol::ParseIssueKind::InvalidPayload => ErrorSeverity::Error,
                                protocol::ParseIssueKind::UnknownType => ErrorSeverity::Warning,
                                _ => ErrorSeverity::Warning,
                            };

                            let correlation_id = issue.correlation_id.clone();
                            let prompt_msg = PromptMessage::ProtocolError {
                                correlation_id: issue.correlation_id,
                                summary,
                                details,
                                severity,
                                script_path: script_path.clone(),
                            };

                            if tx.send_blocking(prompt_msg).is_err() {
                                tracing::warn!(
                                    correlation_id = %correlation_id,
                                    script_path = %script_path,
                                    "Prompt channel closed, dropping protocol error"
                                );
                            }
                        }) {
                            Ok(Some(msg)) => {
                                logging::log("EXEC", &format!("Received message: {:?}", msg));

                                // First, try to handle selected text messages directly (no UI needed)
                                match executor::handle_selected_text_message(&msg) {
                                    executor::SelectedTextHandleResult::Handled(response) => {
                                        logging::log("EXEC", &format!("Handled selected text message, sending response: {:?}", response));
                                        if let Err(e) = reader_response_tx.send(response) {
                                            logging::log(
                                                "EXEC",
                                                &format!(
                                                    "Failed to send selected text response: {}",
                                                    e
                                                ),
                                            );
                                        }
                                        continue;
                                    }
                                    executor::SelectedTextHandleResult::NotHandled => {
                                        // Fall through to other message handling
                                    }
                                }

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
                                if let Message::WindowList { request_id } = &msg {
                                    logging::log(
                                        "EXEC",
                                        &format!("WindowList request: {}", request_id),
                                    );

                                    let response = match window_control::list_windows() {
                                        Ok(windows) => {
                                            let window_infos: Vec<protocol::SystemWindowInfo> =
                                                windows
                                                    .into_iter()
                                                    .map(|w| protocol::SystemWindowInfo {
                                                        window_id: w.id,
                                                        title: w.title,
                                                        app_name: w.app,
                                                        bounds: Some(
                                                            protocol::TargetWindowBounds {
                                                                x: w.bounds.x,
                                                                y: w.bounds.y,
                                                                width: w.bounds.width,
                                                                height: w.bounds.height,
                                                            },
                                                        ),
                                                        is_minimized: None,
                                                        is_active: None,
                                                    })
                                                    .collect();
                                            Message::window_list_result(
                                                request_id.clone(),
                                                window_infos,
                                            )
                                        }
                                        Err(e) => {
                                            logging::log(
                                                "EXEC",
                                                &format!("WindowList error: {}", e),
                                            );
                                            // Return empty list on error
                                            Message::window_list_result(request_id.clone(), vec![])
                                        }
                                    };

                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!("Failed to send window list response: {}", e),
                                        );
                                    }
                                    continue;
                                }

                                // Handle WindowAction directly (no UI needed)
                                if let Message::WindowAction {
                                    request_id,
                                    action,
                                    window_id,
                                    bounds,
                                } = &msg
                                {
                                    logging::log(
                                        "EXEC",
                                        &format!(
                                            "WindowAction request: {:?} for window {:?}",
                                            action, window_id
                                        ),
                                    );

                                    let result = match action {
                                        protocol::WindowActionType::Focus => {
                                            if let Some(id) = window_id {
                                                window_control::focus_window(*id)
                                            } else {
                                                Err(anyhow::anyhow!("Missing window_id"))
                                            }
                                        }
                                        protocol::WindowActionType::Close => {
                                            if let Some(id) = window_id {
                                                window_control::close_window(*id)
                                            } else {
                                                Err(anyhow::anyhow!("Missing window_id"))
                                            }
                                        }
                                        protocol::WindowActionType::Minimize => {
                                            if let Some(id) = window_id {
                                                window_control::minimize_window(*id)
                                            } else {
                                                Err(anyhow::anyhow!("Missing window_id"))
                                            }
                                        }
                                        protocol::WindowActionType::Maximize => {
                                            if let Some(id) = window_id {
                                                window_control::maximize_window(*id)
                                            } else {
                                                Err(anyhow::anyhow!("Missing window_id"))
                                            }
                                        }
                                        protocol::WindowActionType::Resize => {
                                            if let (Some(id), Some(b)) = (window_id, bounds) {
                                                window_control::resize_window(
                                                    *id, b.width, b.height,
                                                )
                                            } else {
                                                Err(anyhow::anyhow!("Missing window_id or bounds"))
                                            }
                                        }
                                        protocol::WindowActionType::Move => {
                                            if let (Some(id), Some(b)) = (window_id, bounds) {
                                                window_control::move_window(*id, b.x, b.y)
                                            } else {
                                                Err(anyhow::anyhow!("Missing window_id or bounds"))
                                            }
                                        }
                                    };

                                    let response = match result {
                                        Ok(()) => {
                                            Message::window_action_success(request_id.clone())
                                        }
                                        Err(e) => Message::window_action_error(
                                            request_id.clone(),
                                            e.to_string(),
                                        ),
                                    };

                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "Failed to send window action response: {}",
                                                e
                                            ),
                                        );
                                    }
                                    continue;
                                }

                                // Handle FileSearch directly (no UI needed)
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

                                    let results = file_search::search_files(
                                        query,
                                        only_in.as_deref(),
                                        file_search::DEFAULT_LIMIT,
                                    );
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

                                // Handle GetState - needs UI state, forward to UI thread
                                if let Message::GetState { request_id } = &msg {
                                    logging::log(
                                        "EXEC",
                                        &format!("GetState request: {}", request_id),
                                    );
                                    let prompt_msg = PromptMessage::GetState {
                                        request_id: request_id.clone(),
                                    };
                                    if tx.send_blocking(prompt_msg).is_err() {
                                        logging::log(
                                            "EXEC",
                                            "Prompt channel closed, reader exiting",
                                        );
                                        break;
                                    }
                                    continue;
                                }

                                // Handle GetLayoutInfo - needs UI state, forward to UI thread
                                if let Message::GetLayoutInfo { request_id } = &msg {
                                    logging::log(
                                        "EXEC",
                                        &format!("GetLayoutInfo request: {}", request_id),
                                    );
                                    let prompt_msg = PromptMessage::GetLayoutInfo {
                                        request_id: request_id.clone(),
                                    };
                                    if tx.send_blocking(prompt_msg).is_err() {
                                        logging::log(
                                            "EXEC",
                                            "Prompt channel closed, reader exiting",
                                        );
                                        break;
                                    }
                                    continue;
                                }

                                // Handle CaptureScreenshot directly (no UI needed)
                                if let Message::CaptureScreenshot { request_id, hi_dpi } = &msg {
                                    let hi_dpi_mode = hi_dpi.unwrap_or(false);
                                    tracing::info!(request_id = %request_id, hi_dpi = hi_dpi_mode, "Capturing screenshot");

                                    let response = match capture_app_screenshot(hi_dpi_mode) {
                                        Ok((png_data, width, height)) => {
                                            use base64::Engine;
                                            let base64_data =
                                                base64::engine::general_purpose::STANDARD
                                                    .encode(&png_data);
                                            tracing::info!(
                                                request_id = %request_id,
                                                width = width,
                                                height = height,
                                                hi_dpi = hi_dpi_mode,
                                                data_len = base64_data.len(),
                                                "Screenshot captured successfully"
                                            );
                                            Message::screenshot_result(
                                                request_id.clone(),
                                                base64_data,
                                                width,
                                                height,
                                            )
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                request_id = %request_id,
                                                error = %e,
                                                "Screenshot capture failed"
                                            );
                                            // Send empty result on error
                                            Message::screenshot_result(
                                                request_id.clone(),
                                                String::new(),
                                                0,
                                                0,
                                            )
                                        }
                                    };

                                    if let Err(e) = reader_response_tx.send(response) {
                                        tracing::error!(error = %e, "Failed to send screenshot response");
                                    }
                                    continue;
                                }

                                let prompt_msg = match msg {
                                    Message::Arg {
                                        id,
                                        placeholder,
                                        choices,
                                        actions,
                                    } => Some(PromptMessage::ShowArg {
                                        id,
                                        placeholder,
                                        choices,
                                        actions,
                                    }),
                                    Message::Div {
                                        id,
                                        html,
                                        container_classes,
                                        actions,
                                        placeholder,
                                        hint,
                                        footer,
                                        container_bg,
                                        container_padding,
                                        opacity,
                                    } => Some(PromptMessage::ShowDiv {
                                        id,
                                        html,
                                        container_classes,
                                        actions,
                                        placeholder,
                                        hint,
                                        footer,
                                        container_bg,
                                        container_padding,
                                        opacity,
                                    }),
                                    Message::Form { id, html, actions } => {
                                        Some(PromptMessage::ShowForm { id, html, actions })
                                    }
                                    Message::Term {
                                        id,
                                        command,
                                        actions,
                                    } => Some(PromptMessage::ShowTerm {
                                        id,
                                        command,
                                        actions,
                                    }),
                                    Message::Editor {
                                        id,
                                        content,
                                        language,
                                        template,
                                        actions,
                                        ..
                                    } => Some(PromptMessage::ShowEditor {
                                        id,
                                        content,
                                        language,
                                        template,
                                        actions,
                                    }),
                                    // New prompt types (scaffolding)
                                    Message::Path {
                                        id,
                                        start_path,
                                        hint,
                                    } => Some(PromptMessage::ShowPath {
                                        id,
                                        start_path,
                                        hint,
                                    }),
                                    Message::Env { id, key, secret } => {
                                        Some(PromptMessage::ShowEnv {
                                            id,
                                            key,
                                            prompt: None,
                                            secret: secret.unwrap_or(false),
                                        })
                                    }
                                    Message::Drop { id } => Some(PromptMessage::ShowDrop {
                                        id,
                                        placeholder: None,
                                        hint: None,
                                    }),
                                    Message::Template { id, template } => {
                                        Some(PromptMessage::ShowTemplate { id, template })
                                    }
                                    Message::Select {
                                        id,
                                        placeholder,
                                        choices,
                                        multiple,
                                    } => Some(PromptMessage::ShowSelect {
                                        id,
                                        placeholder: Some(placeholder),
                                        choices,
                                        multiple: multiple.unwrap_or(false),
                                    }),
                                    Message::Exit { .. } => Some(PromptMessage::ScriptExit),
                                    Message::ForceSubmit { value } => {
                                        Some(PromptMessage::ForceSubmit { value })
                                    }
                                    Message::Hide {} => Some(PromptMessage::HideWindow),
                                    Message::Browse { url } => {
                                        Some(PromptMessage::OpenBrowser { url })
                                    }
                                    Message::Hud { text, duration_ms } => {
                                        Some(PromptMessage::ShowHud { text, duration_ms })
                                    }
                                    Message::SetActions { actions } => {
                                        Some(PromptMessage::SetActions { actions })
                                    }
                                    Message::SetInput { text } => {
                                        Some(PromptMessage::SetInput { text })
                                    }
                                    Message::ShowGrid { options } => {
                                        Some(PromptMessage::ShowGrid { options })
                                    }
                                    Message::HideGrid => Some(PromptMessage::HideGrid),
                                    other => {
                                        // Get the message type name for user feedback
                                        let msg_type = format!("{:?}", other);
                                        // Extract just the variant name (before any {})
                                        let type_name = msg_type
                                            .split('{')
                                            .next()
                                            .unwrap_or(&msg_type)
                                            .trim()
                                            .to_string();
                                        logging::log(
                                            "WARN",
                                            &format!("Unhandled message type: {}", type_name),
                                        );
                                        Some(PromptMessage::UnhandledMessage {
                                            message_type: type_name,
                                        })
                                    }
                                };

                                if let Some(prompt_msg) = prompt_msg {
                                    if tx.send_blocking(prompt_msg).is_err() {
                                        logging::log(
                                            "EXEC",
                                            "Prompt channel closed, reader exiting",
                                        );
                                        break;
                                    }
                                }
                            }
                            Ok(None) => {
                                logging::log("EXEC", "Script stdout closed (EOF)");

                                // Check if process exited with error
                                let exit_code = match keep_alive_child.try_wait() {
                                    Ok(Some(status)) => status.code(),
                                    Ok(None) => {
                                        // Process still running, wait for it
                                        match keep_alive_child.wait() {
                                            Ok(status) => status.code(),
                                            Err(_) => None,
                                        }
                                    }
                                    Err(_) => None,
                                };

                                logging::log("EXEC", &format!("Script exit code: {:?}", exit_code));

                                // If non-zero exit code, capture stderr and send error
                                if let Some(code) = exit_code {
                                    if code != 0 {
                                        // Read stderr if available
                                        let stderr_output =
                                            if let Some(mut stderr) = stderr_for_errors.take() {
                                                use std::io::Read;
                                                let mut stderr_str = String::new();
                                                if stderr.read_to_string(&mut stderr_str).is_ok()
                                                    && !stderr_str.is_empty()
                                                {
                                                    Some(stderr_str)
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            };

                                        if let Some(ref stderr_text) = stderr_output {
                                            logging::log(
                                                "EXEC",
                                                &format!(
                                                    "Captured stderr ({} bytes)",
                                                    stderr_text.len()
                                                ),
                                            );

                                            // Parse error info and generate suggestions
                                            let error_message =
                                                executor::extract_error_message(stderr_text);
                                            let stack_trace =
                                                executor::parse_stack_trace(stderr_text);
                                            let suggestions = executor::generate_suggestions(
                                                stderr_text,
                                                Some(code),
                                            );

                                            // Send script error message
                                            let _ = tx.send_blocking(PromptMessage::ScriptError {
                                                error_message,
                                                stderr_output: Some(stderr_text.clone()),
                                                exit_code: Some(code),
                                                stack_trace,
                                                script_path: script_path.clone(),
                                                suggestions,
                                            });
                                        } else {
                                            // No stderr, send generic error
                                            let _ = tx.send_blocking(PromptMessage::ScriptError {
                                                error_message: format!(
                                                    "Script exited with code {}",
                                                    code
                                                ),
                                                stderr_output: None,
                                                exit_code: Some(code),
                                                stack_trace: None,
                                                script_path: script_path.clone(),
                                                suggestions: vec![
                                                    "Check the script for errors".to_string()
                                                ],
                                            });
                                        }
                                    }
                                }

                                let _ = tx.send_blocking(PromptMessage::ScriptExit);
                                break;
                            }
                            Err(e) => {
                                logging::log("EXEC", &format!("Error reading from script: {}", e));

                                // Try to read stderr for error details
                                let stderr_output =
                                    if let Some(mut stderr) = stderr_for_errors.take() {
                                        use std::io::Read;
                                        let mut stderr_str = String::new();
                                        if stderr.read_to_string(&mut stderr_str).is_ok()
                                            && !stderr_str.is_empty()
                                        {
                                            Some(stderr_str)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                if let Some(ref stderr_text) = stderr_output {
                                    let error_message =
                                        executor::extract_error_message(stderr_text);
                                    let stack_trace = executor::parse_stack_trace(stderr_text);
                                    let suggestions =
                                        executor::generate_suggestions(stderr_text, None);

                                    let _ = tx.send_blocking(PromptMessage::ScriptError {
                                        error_message,
                                        stderr_output: Some(stderr_text.clone()),
                                        exit_code: None,
                                        stack_trace,
                                        script_path: script_path.clone(),
                                        suggestions,
                                    });
                                }

                                let _ = tx.send_blocking(PromptMessage::ScriptExit);
                                break;
                            }
                        }
                    }
                    logging::log(
                        "EXEC",
                        "Reader thread exited, process handle will now be dropped",
                    );
                });

                // Store the response sender for the UI to use
                self.response_sender = Some(response_tx);
            }
            Err(e) => {
                logging::log(
                    "EXEC",
                    &format!("Failed to start interactive session: {}", e),
                );
                self.last_output = Some(SharedString::from(format!("✗ Error: {}", e)));
                cx.notify();
            }
        }
    }
}

</file>

<file path="src/builtins.rs">
//! Built-in Features Registry
//!
//! Provides a registry of built-in features that appear in the main search
//! alongside scripts. Features like Clipboard History and App Launcher are
//! configurable and can be enabled/disabled via config.
//!
//! ## Command Types
//!
//! The registry supports various command types organized by category:
//! - **System Actions**: Power management, UI controls, volume/brightness
//! - **Window Actions**: Window tiling and management for the frontmost window
//! - **Notes Commands**: Notes window operations
//! - **AI Commands**: AI chat window operations  
//! - **Script Commands**: Create new scripts and scriptlets
//! - **Permission Commands**: Accessibility permission management
//!

use crate::config::BuiltInConfig;
use crate::menu_bar::MenuBarItem;
use tracing::debug;

// ============================================================================
// Command Type Enums
// ============================================================================

/// System action types for macOS system commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemActionType {
    // Power management
    EmptyTrash,
    LockScreen,
    Sleep,
    Restart,
    ShutDown,
    LogOut,

    // UI controls
    ToggleDarkMode,
    ShowDesktop,
    MissionControl,
    Launchpad,
    ForceQuitApps,

    // Volume controls (preset levels)
    Volume0,
    Volume25,
    Volume50,
    Volume75,
    Volume100,
    VolumeMute,

    // Brightness controls (preset levels)
    Brightness0,
    Brightness25,
    Brightness50,
    Brightness75,
    Brightness100,

    // Dev/test actions (only available in debug builds)
    #[cfg(debug_assertions)]
    TestConfirmation,

    // App control
    QuitScriptKit,

    // System utilities
    ToggleDoNotDisturb,
    StartScreenSaver,

    // System Preferences
    OpenSystemPreferences,
    OpenPrivacySettings,
    OpenDisplaySettings,
    OpenSoundSettings,
    OpenNetworkSettings,
    OpenKeyboardSettings,
    OpenBluetoothSettings,
    OpenNotificationsSettings,
}

/// Window action types for window management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowActionType {
    TileLeft,
    TileRight,
    TileTop,
    TileBottom,
    Maximize,
    Minimize,
}

/// Notes window command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NotesCommandType {
    OpenNotes,
    NewNote,
    SearchNotes,
    QuickCapture,
}

/// AI window command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AiCommandType {
    OpenAi,
    NewConversation,
    ClearConversation,
}

/// Script creation command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptCommandType {
    NewScript,
    NewScriptlet,
}

/// Permission management command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionCommandType {
    CheckPermissions,
    RequestAccessibility,
    OpenAccessibilitySettings,
}

/// Frecency/suggested items command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrecencyCommandType {
    ClearSuggested,
}

/// Menu bar action details for executing menu commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarActionInfo {
    /// The bundle ID of the app (e.g., "com.apple.Safari")
    pub bundle_id: String,
    /// The path to the menu item (e.g., ["File", "New Window"])
    pub menu_path: Vec<String>,
    /// Whether the menu item is enabled
    pub enabled: bool,
    /// Keyboard shortcut if any (e.g., "⌘N")
    pub shortcut: Option<String>,
}

/// Groups for categorizing built-in entries in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // MenuBar variant will be used when menu bar integration is complete
pub enum BuiltInGroup {
    /// Core built-in features (Clipboard History, Window Switcher, etc.)
    #[default]
    Core,
    /// Menu bar items from the frontmost application
    MenuBar,
}

/// Types of built-in features
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Some variants reserved for future use
pub enum BuiltInFeature {
    /// Clipboard history viewer/manager
    ClipboardHistory,
    /// Application launcher for opening installed apps (legacy, apps now in main search)
    AppLauncher,
    /// Individual application entry (for future use when apps appear in search)
    App(String),
    /// Window switcher for managing and tiling windows
    WindowSwitcher,
    /// Design gallery for viewing separator and icon variations
    DesignGallery,
    /// AI Chat window for conversing with AI assistants
    AiChat,
    /// Notes window for quick notes and scratchpad
    Notes,
    /// Menu bar action from the frontmost application
    MenuBarAction(MenuBarActionInfo),

    // === New Command Types ===
    /// System actions (power, UI controls, volume, brightness, settings)
    SystemAction(SystemActionType),
    /// Window actions for the frontmost window (tile, maximize, minimize)
    WindowAction(WindowActionType),
    /// Notes window commands
    NotesCommand(NotesCommandType),
    /// AI window commands
    AiCommand(AiCommandType),
    /// Script creation commands
    ScriptCommand(ScriptCommandType),
    /// Permission management commands
    PermissionCommand(PermissionCommandType),
    /// Frecency/suggested items commands
    FrecencyCommand(FrecencyCommandType),
}

/// A built-in feature entry that appears in the main search
#[derive(Debug, Clone)]
pub struct BuiltInEntry {
    /// Unique identifier for the entry
    pub id: String,
    /// Display name shown in search results
    pub name: String,
    /// Description shown below the name
    pub description: String,
    /// Keywords for fuzzy matching in search
    pub keywords: Vec<String>,
    /// The actual feature this entry represents
    pub feature: BuiltInFeature,
    /// Optional icon (emoji) to display
    pub icon: Option<String>,
    /// Group for categorization in the UI (will be used when menu bar integration is complete)
    #[allow(dead_code)]
    pub group: BuiltInGroup,
}

impl BuiltInEntry {
    /// Create a new built-in entry (Core group, no icon)
    #[allow(dead_code)]
    fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        keywords: Vec<&str>,
        feature: BuiltInFeature,
    ) -> Self {
        BuiltInEntry {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            keywords: keywords.into_iter().map(String::from).collect(),
            feature,
            icon: None,
            group: BuiltInGroup::Core,
        }
    }

    /// Create a new built-in entry with an icon (Core group)
    fn new_with_icon(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        keywords: Vec<&str>,
        feature: BuiltInFeature,
        icon: impl Into<String>,
    ) -> Self {
        BuiltInEntry {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            keywords: keywords.into_iter().map(String::from).collect(),
            feature,
            icon: Some(icon.into()),
            group: BuiltInGroup::Core,
        }
    }

    /// Create a new built-in entry with icon and group
    #[allow(dead_code)]
    pub fn new_with_group(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        keywords: Vec<String>,
        feature: BuiltInFeature,
        icon: Option<String>,
        group: BuiltInGroup,
    ) -> Self {
        BuiltInEntry {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            keywords,
            feature,
            icon,
            group,
        }
    }
}

/// Get the list of enabled built-in entries based on configuration
///
/// # Arguments
/// * `config` - The built-in features configuration
///
/// # Returns
/// A vector of enabled built-in entries that should appear in the main search
///
/// Note: AppLauncher built-in is no longer used since apps now appear directly
/// in the main search results. The config option is retained for future use
/// (e.g., to control whether apps are included in search at all).
pub fn get_builtin_entries(config: &BuiltInConfig) -> Vec<BuiltInEntry> {
    let mut entries = Vec::new();

    if config.clipboard_history {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-clipboard-history",
            "Clipboard History",
            "View and manage your clipboard history",
            vec!["clipboard", "history", "paste", "copy"],
            BuiltInFeature::ClipboardHistory,
            "📋",
        ));
        debug!("Added Clipboard History built-in entry");
    }

    // Note: AppLauncher built-in removed - apps now appear directly in main search
    // The app_launcher config flag is kept for future use (e.g., to disable app search entirely)
    if config.app_launcher {
        debug!("app_launcher enabled - apps will appear in main search");
    }

    if config.window_switcher {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-window-switcher",
            "Window Switcher",
            "Switch, tile, and manage open windows",
            vec!["window", "switch", "tile", "focus", "manage", "switcher"],
            BuiltInFeature::WindowSwitcher,
            "🪟",
        ));
        debug!("Added Window Switcher built-in entry");
    }

    // AI Chat is always available
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-ai-chat",
        "AI Chat",
        "Chat with AI assistants (Claude, GPT)",
        vec![
            "ai",
            "chat",
            "assistant",
            "claude",
            "gpt",
            "openai",
            "anthropic",
            "llm",
        ],
        BuiltInFeature::AiChat,
        "🤖",
    ));
    debug!("Added AI Chat built-in entry");

    // Notes is always available
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-notes",
        "Notes",
        "Quick notes and scratchpad",
        vec![
            "notes",
            "note",
            "scratch",
            "scratchpad",
            "memo",
            "markdown",
            "write",
            "text",
        ],
        BuiltInFeature::Notes,
        "📝",
    ));
    debug!("Added Notes built-in entry");

    // Design Gallery is only available in debug builds (developer tool)
    #[cfg(debug_assertions)]
    {
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-design-gallery",
            "Design Gallery",
            "Browse separator styles and icon variations",
            vec![
                "design",
                "gallery",
                "separator",
                "icon",
                "style",
                "theme",
                "variations",
            ],
            BuiltInFeature::DesignGallery,
            "🎨",
        ));
        debug!("Added Design Gallery built-in entry");

        // Test Confirmation entry for testing confirmation UI
        entries.push(BuiltInEntry::new_with_icon(
            "builtin-test-confirmation",
            "Test Confirmation",
            "Test the confirmation dialog (dev only)",
            vec!["test", "confirmation", "dev", "debug"],
            BuiltInFeature::SystemAction(SystemActionType::TestConfirmation),
            "🧪",
        ));
        debug!("Added Test Confirmation built-in entry");
    }

    // =========================================================================
    // System Actions
    // =========================================================================

    // Power management
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-empty-trash",
        "Empty Trash",
        "Empty the macOS Trash",
        vec!["empty", "trash", "delete", "clean"],
        BuiltInFeature::SystemAction(SystemActionType::EmptyTrash),
        "🗑️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-lock-screen",
        "Lock Screen",
        "Lock the screen",
        vec!["lock", "screen", "security"],
        BuiltInFeature::SystemAction(SystemActionType::LockScreen),
        "🔒",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-sleep",
        "Sleep",
        "Put the system to sleep",
        vec!["sleep", "suspend", "power"],
        BuiltInFeature::SystemAction(SystemActionType::Sleep),
        "😴",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-restart",
        "Restart",
        "Restart the system",
        vec!["restart", "reboot", "power"],
        BuiltInFeature::SystemAction(SystemActionType::Restart),
        "🔄",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-shut-down",
        "Shut Down",
        "Shut down the system",
        vec!["shut", "down", "shutdown", "power", "off"],
        BuiltInFeature::SystemAction(SystemActionType::ShutDown),
        "⏻",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-log-out",
        "Log Out",
        "Log out the current user",
        vec!["log", "out", "logout", "user"],
        BuiltInFeature::SystemAction(SystemActionType::LogOut),
        "🚪",
    ));

    // UI controls
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-toggle-dark-mode",
        "Toggle Dark Mode",
        "Switch between light and dark appearance",
        vec!["dark", "mode", "light", "appearance", "theme", "toggle"],
        BuiltInFeature::SystemAction(SystemActionType::ToggleDarkMode),
        "🌙",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-show-desktop",
        "Show Desktop",
        "Hide all windows to reveal the desktop",
        vec!["show", "desktop", "hide", "windows"],
        BuiltInFeature::SystemAction(SystemActionType::ShowDesktop),
        "🖥️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-mission-control",
        "Mission Control",
        "Show all windows and desktops",
        vec!["mission", "control", "expose", "spaces", "windows"],
        BuiltInFeature::SystemAction(SystemActionType::MissionControl),
        "🪟",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-launchpad",
        "Launchpad",
        "Open Launchpad to show all applications",
        vec!["launchpad", "apps", "applications"],
        BuiltInFeature::SystemAction(SystemActionType::Launchpad),
        "🚀",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-force-quit",
        "Force Quit Apps",
        "Open the Force Quit Applications dialog",
        vec!["force", "quit", "kill", "apps", "unresponsive"],
        BuiltInFeature::SystemAction(SystemActionType::ForceQuitApps),
        "⚠️",
    ));

    // Volume controls (preset levels)
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-0",
        "Volume 0%",
        "Set system volume to 0% (mute)",
        vec!["volume", "mute", "0", "percent", "zero", "off"],
        BuiltInFeature::SystemAction(SystemActionType::Volume0),
        "🔇",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-25",
        "Volume 25%",
        "Set system volume to 25%",
        vec!["volume", "25", "percent", "low", "quiet"],
        BuiltInFeature::SystemAction(SystemActionType::Volume25),
        "🔈",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-50",
        "Volume 50%",
        "Set system volume to 50%",
        vec!["volume", "50", "percent", "half", "medium"],
        BuiltInFeature::SystemAction(SystemActionType::Volume50),
        "🔉",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-75",
        "Volume 75%",
        "Set system volume to 75%",
        vec!["volume", "75", "percent", "high", "loud"],
        BuiltInFeature::SystemAction(SystemActionType::Volume75),
        "🔉",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-100",
        "Volume 100%",
        "Set system volume to 100% (max)",
        vec!["volume", "100", "percent", "max", "full"],
        BuiltInFeature::SystemAction(SystemActionType::Volume100),
        "🔊",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-volume-mute",
        "Toggle Mute",
        "Toggle audio mute",
        vec!["mute", "unmute", "volume", "sound", "audio", "toggle"],
        BuiltInFeature::SystemAction(SystemActionType::VolumeMute),
        "🔇",
    ));

    // Brightness controls (preset levels)
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-brightness-0",
        "Brightness 0%",
        "Set display brightness to 0% (dark)",
        vec![
            "brightness",
            "0",
            "percent",
            "dark",
            "off",
            "display",
            "screen",
        ],
        BuiltInFeature::SystemAction(SystemActionType::Brightness0),
        "🌑",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-brightness-25",
        "Brightness 25%",
        "Set display brightness to 25%",
        vec![
            "brightness",
            "25",
            "percent",
            "dim",
            "low",
            "display",
            "screen",
        ],
        BuiltInFeature::SystemAction(SystemActionType::Brightness25),
        "🌘",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-brightness-50",
        "Brightness 50%",
        "Set display brightness to 50%",
        vec![
            "brightness",
            "50",
            "percent",
            "half",
            "medium",
            "display",
            "screen",
        ],
        BuiltInFeature::SystemAction(SystemActionType::Brightness50),
        "🌗",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-brightness-75",
        "Brightness 75%",
        "Set display brightness to 75%",
        vec![
            "brightness",
            "75",
            "percent",
            "bright",
            "high",
            "display",
            "screen",
        ],
        BuiltInFeature::SystemAction(SystemActionType::Brightness75),
        "🌖",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-brightness-100",
        "Brightness 100%",
        "Set display brightness to 100% (max)",
        vec![
            "brightness",
            "100",
            "percent",
            "max",
            "full",
            "display",
            "screen",
        ],
        BuiltInFeature::SystemAction(SystemActionType::Brightness100),
        "☀️",
    ));

    // App control
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-quit-script-kit",
        "Quit Script Kit",
        "Quit the Script Kit application",
        vec!["quit", "exit", "close", "script", "kit", "app"],
        BuiltInFeature::SystemAction(SystemActionType::QuitScriptKit),
        "🚪",
    ));

    // System utilities
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-toggle-dnd",
        "Toggle Do Not Disturb",
        "Toggle Focus/Do Not Disturb mode",
        vec![
            "do",
            "not",
            "disturb",
            "dnd",
            "focus",
            "notifications",
            "toggle",
        ],
        BuiltInFeature::SystemAction(SystemActionType::ToggleDoNotDisturb),
        "🔕",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-screen-saver",
        "Start Screen Saver",
        "Activate the screen saver",
        vec!["screen", "saver", "screensaver"],
        BuiltInFeature::SystemAction(SystemActionType::StartScreenSaver),
        "🖼️",
    ));

    // System Preferences
    entries.push(BuiltInEntry::new_with_icon(
        "builtin-system-preferences",
        "Open System Settings",
        "Open System Settings (System Preferences)",
        vec!["system", "settings", "preferences", "prefs"],
        BuiltInFeature::SystemAction(SystemActionType::OpenSystemPreferences),
        "⚙️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-privacy-settings",
        "Privacy & Security Settings",
        "Open Privacy & Security settings",
        vec!["privacy", "security", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenPrivacySettings),
        "🔐",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-display-settings",
        "Display Settings",
        "Open Display settings",
        vec!["display", "monitor", "screen", "resolution", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenDisplaySettings),
        "🖥️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-sound-settings",
        "Sound Settings",
        "Open Sound settings",
        vec!["sound", "audio", "volume", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenSoundSettings),
        "🔊",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-network-settings",
        "Network Settings",
        "Open Network settings",
        vec!["network", "wifi", "ethernet", "internet", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenNetworkSettings),
        "📡",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-keyboard-settings",
        "Keyboard Settings",
        "Open Keyboard settings",
        vec!["keyboard", "shortcuts", "input", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenKeyboardSettings),
        "⌨️",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-bluetooth-settings",
        "Bluetooth Settings",
        "Open Bluetooth settings",
        vec!["bluetooth", "wireless", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenBluetoothSettings),
        "🔵",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-notifications-settings",
        "Notification Settings",
        "Open Notifications settings",
        vec!["notifications", "alerts", "banners", "settings"],
        BuiltInFeature::SystemAction(SystemActionType::OpenNotificationsSettings),
        "🔔",
    ));

    // =========================================================================
    // Window Actions (for frontmost window)
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-tile-left",
        "Tile Window Left",
        "Tile the frontmost window to the left half",
        vec!["tile", "left", "window", "half", "snap"],
        BuiltInFeature::WindowAction(WindowActionType::TileLeft),
        "◧",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-tile-right",
        "Tile Window Right",
        "Tile the frontmost window to the right half",
        vec!["tile", "right", "window", "half", "snap"],
        BuiltInFeature::WindowAction(WindowActionType::TileRight),
        "◨",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-tile-top",
        "Tile Window Top",
        "Tile the frontmost window to the top half",
        vec!["tile", "top", "window", "half", "snap"],
        BuiltInFeature::WindowAction(WindowActionType::TileTop),
        "⬒",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-tile-bottom",
        "Tile Window Bottom",
        "Tile the frontmost window to the bottom half",
        vec!["tile", "bottom", "window", "half", "snap"],
        BuiltInFeature::WindowAction(WindowActionType::TileBottom),
        "⬓",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-maximize-window",
        "Maximize Window",
        "Maximize the frontmost window",
        vec!["maximize", "window", "fullscreen", "expand"],
        BuiltInFeature::WindowAction(WindowActionType::Maximize),
        "⬜",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-minimize-window",
        "Minimize Window",
        "Minimize the frontmost window",
        vec!["minimize", "window", "dock", "hide"],
        BuiltInFeature::WindowAction(WindowActionType::Minimize),
        "➖",
    ));

    // =========================================================================
    // Notes Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-note",
        "New Note",
        "Create a new note",
        vec!["new", "note", "create"],
        BuiltInFeature::NotesCommand(NotesCommandType::NewNote),
        "📝",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-search-notes",
        "Search Notes",
        "Search through your notes",
        vec!["search", "notes", "find"],
        BuiltInFeature::NotesCommand(NotesCommandType::SearchNotes),
        "🔍",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-quick-capture",
        "Quick Capture",
        "Quickly capture a note",
        vec!["quick", "capture", "note", "fast"],
        BuiltInFeature::NotesCommand(NotesCommandType::QuickCapture),
        "⚡",
    ));

    // =========================================================================
    // AI Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-conversation",
        "New AI Conversation",
        "Start a new AI conversation",
        vec!["new", "conversation", "chat", "ai"],
        BuiltInFeature::AiCommand(AiCommandType::NewConversation),
        "💬",
    ));

    // =========================================================================
    // Script Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-script",
        "New Script",
        "Create a new Script Kit script",
        vec!["new", "script", "create", "code"],
        BuiltInFeature::ScriptCommand(ScriptCommandType::NewScript),
        "📜",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-new-scriptlet",
        "New Scriptlet",
        "Create a new Script Kit scriptlet",
        vec!["new", "scriptlet", "create", "snippet"],
        BuiltInFeature::ScriptCommand(ScriptCommandType::NewScriptlet),
        "✨",
    ));

    // =========================================================================
    // Permission Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-check-permissions",
        "Check Permissions",
        "Check all required macOS permissions",
        vec!["check", "permissions", "accessibility", "privacy"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::CheckPermissions),
        "✅",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-request-accessibility",
        "Request Accessibility Permission",
        "Request accessibility permission for Script Kit",
        vec!["request", "accessibility", "permission"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::RequestAccessibility),
        "🔑",
    ));

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-accessibility-settings",
        "Open Accessibility Settings",
        "Open Accessibility settings in System Preferences",
        vec!["accessibility", "settings", "permission", "open"],
        BuiltInFeature::PermissionCommand(PermissionCommandType::OpenAccessibilitySettings),
        "♿",
    ));

    // =========================================================================
    // Frecency/Suggested Commands
    // =========================================================================

    entries.push(BuiltInEntry::new_with_icon(
        "builtin-clear-suggested",
        "Clear Suggested",
        "Clear all suggested/recently used items",
        vec![
            "clear",
            "suggested",
            "recent",
            "frecency",
            "reset",
            "history",
        ],
        BuiltInFeature::FrecencyCommand(FrecencyCommandType::ClearSuggested),
        "🧹",
    ));

    debug!(count = entries.len(), "Built-in entries loaded");
    entries
}

// ============================================================================
// Menu Bar Item Conversion
// ============================================================================

/// Convert menu bar items to built-in entries for search
///
/// This flattens the menu hierarchy into searchable entries, skipping the
/// Apple menu (first item) and only including leaf items (no submenus).
///
/// # Arguments
/// * `items` - The menu bar items from the frontmost application
/// * `bundle_id` - The bundle identifier of the application (e.g., "com.apple.Safari")
/// * `app_name` - The display name of the application (e.g., "Safari")
///
/// # Returns
/// A vector of `BuiltInEntry` items that can be added to search results
#[allow(dead_code)] // Will be used when menu bar integration is complete
pub fn menu_bar_items_to_entries(
    items: &[MenuBarItem],
    bundle_id: &str,
    app_name: &str,
) -> Vec<BuiltInEntry> {
    let mut entries = Vec::new();

    // Skip first item (Apple menu)
    for item in items.iter().skip(1) {
        flatten_menu_item(item, bundle_id, app_name, &[], &mut entries);
    }

    debug!(
        count = entries.len(),
        bundle_id = bundle_id,
        app_name = app_name,
        "Menu bar items converted to entries"
    );
    entries
}

/// Recursively flatten a menu item and its children into entries
#[allow(dead_code)] // Will be used when menu bar integration is complete
fn flatten_menu_item(
    item: &MenuBarItem,
    bundle_id: &str,
    app_name: &str,
    parent_path: &[String],
    entries: &mut Vec<BuiltInEntry>,
) {
    // Skip separators and disabled items
    if item.title.is_empty() || item.title == "-" || item.is_separator() || !item.enabled {
        return;
    }

    let mut current_path = parent_path.to_vec();
    current_path.push(item.title.clone());

    // Only add leaf items (items without children) as entries
    if item.children.is_empty() {
        let id = format!(
            "menubar-{}-{}",
            bundle_id,
            current_path.join("-").to_lowercase().replace(' ', "-")
        );
        let name = current_path.join(" → ");
        let description = if let Some(ref shortcut) = item.shortcut {
            format!("{}  {}", app_name, shortcut.to_display_string())
        } else {
            app_name.to_string()
        };
        let keywords: Vec<String> = current_path.iter().map(|s| s.to_lowercase()).collect();
        let icon = get_menu_icon(&current_path[0]);

        entries.push(BuiltInEntry {
            id,
            name,
            description,
            keywords,
            feature: BuiltInFeature::MenuBarAction(MenuBarActionInfo {
                bundle_id: bundle_id.to_string(),
                menu_path: current_path,
                enabled: item.enabled,
                shortcut: item.shortcut.as_ref().map(|s| s.to_display_string()),
            }),
            icon: Some(icon.to_string()),
            group: BuiltInGroup::MenuBar,
        });
    } else {
        // Recurse into children
        for child in &item.children {
            flatten_menu_item(child, bundle_id, app_name, &current_path, entries);
        }
    }
}

/// Get an appropriate icon for a top-level menu
#[allow(dead_code)] // Will be used when menu bar integration is complete
fn get_menu_icon(top_menu: &str) -> &'static str {
    match top_menu.to_lowercase().as_str() {
        "file" => "📁",
        "edit" => "📋",
        "view" => "👁",
        "window" => "🪟",
        "help" => "❓",
        "format" => "🎨",
        "tools" => "🔧",
        "go" => "➡️",
        "bookmarks" | "favorites" => "⭐",
        "history" => "🕐",
        "develop" | "developer" => "🛠",
        _ => "📌",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BuiltInConfig;

    #[test]
    fn test_builtin_config_default() {
        let config = BuiltInConfig::default();
        assert!(config.clipboard_history);
        assert!(config.app_launcher);
        assert!(config.window_switcher);
    }

    #[test]
    fn test_builtin_config_custom() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: true,
            window_switcher: false,
        };
        assert!(!config.clipboard_history);
        assert!(config.app_launcher);
        assert!(!config.window_switcher);
    }

    #[test]
    fn test_get_builtin_entries_all_enabled() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Core built-ins: Clipboard history, window switcher, AI chat, Notes, design gallery
        // Plus: system actions (28), window actions (6), notes commands (3), AI commands (1),
        // script commands (2), permission commands (3) = 43 new entries
        // Total: 5 + 43 = 48
        assert!(entries.len() >= 5); // At minimum the core built-ins should exist

        // Check clipboard history entry
        let clipboard = entries.iter().find(|e| e.id == "builtin-clipboard-history");
        assert!(clipboard.is_some());
        let clipboard = clipboard.unwrap();
        assert_eq!(clipboard.name, "Clipboard History");
        assert_eq!(clipboard.feature, BuiltInFeature::ClipboardHistory);
        assert!(clipboard.keywords.contains(&"clipboard".to_string()));
        assert!(clipboard.keywords.contains(&"history".to_string()));
        assert!(clipboard.keywords.contains(&"paste".to_string()));
        assert!(clipboard.keywords.contains(&"copy".to_string()));

        // Check window switcher entry
        let window_switcher = entries.iter().find(|e| e.id == "builtin-window-switcher");
        assert!(window_switcher.is_some());
        let window_switcher = window_switcher.unwrap();
        assert_eq!(window_switcher.name, "Window Switcher");
        assert_eq!(window_switcher.feature, BuiltInFeature::WindowSwitcher);
        assert!(window_switcher.keywords.contains(&"window".to_string()));
        assert!(window_switcher.keywords.contains(&"switch".to_string()));
        assert!(window_switcher.keywords.contains(&"tile".to_string()));
        assert!(window_switcher.keywords.contains(&"focus".to_string()));
        assert!(window_switcher.keywords.contains(&"manage".to_string()));
        assert!(window_switcher.keywords.contains(&"switcher".to_string()));

        // Check AI chat entry
        let ai_chat = entries.iter().find(|e| e.id == "builtin-ai-chat");
        assert!(ai_chat.is_some());
        let ai_chat = ai_chat.unwrap();
        assert_eq!(ai_chat.name, "AI Chat");
        assert_eq!(ai_chat.feature, BuiltInFeature::AiChat);
        assert!(ai_chat.keywords.contains(&"ai".to_string()));
        assert!(ai_chat.keywords.contains(&"chat".to_string()));
        assert!(ai_chat.keywords.contains(&"claude".to_string()));
        assert!(ai_chat.keywords.contains(&"gpt".to_string()));

        // Note: App Launcher built-in removed - apps now appear directly in main search
    }

    #[test]
    fn test_get_builtin_entries_clipboard_only() {
        let config = BuiltInConfig {
            clipboard_history: true,
            app_launcher: false,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // Check that core entries exist (plus all the new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-clipboard-history"));
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Window switcher should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-window-switcher"));
    }

    #[test]
    fn test_get_builtin_entries_app_launcher_only() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: true,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // App launcher no longer creates a built-in entry (apps appear in main search)
        // But AI Chat, Notes and Design Gallery are always enabled (plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Clipboard history should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
    }

    #[test]
    fn test_get_builtin_entries_none_enabled() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: false,
            window_switcher: false,
        };
        let entries = get_builtin_entries(&config);

        // AI Chat, Notes, and Design Gallery are always enabled (plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Clipboard history and window switcher should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
        assert!(!entries.iter().any(|e| e.id == "builtin-window-switcher"));
    }

    #[test]
    fn test_get_builtin_entries_window_switcher_only() {
        let config = BuiltInConfig {
            clipboard_history: false,
            app_launcher: false,
            window_switcher: true,
        };
        let entries = get_builtin_entries(&config);

        // Window switcher + AI Chat + Notes + Design Gallery (always enabled, plus new command entries)
        assert!(entries.iter().any(|e| e.id == "builtin-window-switcher"));
        assert!(entries.iter().any(|e| e.id == "builtin-ai-chat"));
        assert!(entries.iter().any(|e| e.id == "builtin-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-design-gallery"));

        // Verify window switcher has correct properties
        let window_switcher = entries
            .iter()
            .find(|e| e.id == "builtin-window-switcher")
            .unwrap();
        assert_eq!(window_switcher.feature, BuiltInFeature::WindowSwitcher);
        assert_eq!(window_switcher.icon, Some("🪟".to_string()));

        // Clipboard history should NOT be present
        assert!(!entries.iter().any(|e| e.id == "builtin-clipboard-history"));
    }

    #[test]
    fn test_builtin_feature_equality() {
        assert_eq!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::ClipboardHistory
        );
        assert_eq!(BuiltInFeature::AppLauncher, BuiltInFeature::AppLauncher);
        assert_eq!(
            BuiltInFeature::WindowSwitcher,
            BuiltInFeature::WindowSwitcher
        );
        assert_eq!(BuiltInFeature::DesignGallery, BuiltInFeature::DesignGallery);
        assert_eq!(BuiltInFeature::AiChat, BuiltInFeature::AiChat);
        assert_ne!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::AppLauncher
        );
        assert_ne!(
            BuiltInFeature::ClipboardHistory,
            BuiltInFeature::WindowSwitcher
        );
        assert_ne!(BuiltInFeature::AppLauncher, BuiltInFeature::WindowSwitcher);
        assert_ne!(
            BuiltInFeature::DesignGallery,
            BuiltInFeature::ClipboardHistory
        );
        assert_ne!(BuiltInFeature::AiChat, BuiltInFeature::ClipboardHistory);
        assert_ne!(BuiltInFeature::AiChat, BuiltInFeature::DesignGallery);

        // Test App variant
        assert_eq!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::App("Safari".to_string())
        );
        assert_ne!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::App("Chrome".to_string())
        );
        assert_ne!(
            BuiltInFeature::App("Safari".to_string()),
            BuiltInFeature::AppLauncher
        );
    }

    #[test]
    fn test_builtin_entry_new() {
        let entry = BuiltInEntry::new(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test", "keyword"],
            BuiltInFeature::ClipboardHistory,
        );

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.name, "Test Entry");
        assert_eq!(entry.description, "Test description");
        assert_eq!(
            entry.keywords,
            vec!["test".to_string(), "keyword".to_string()]
        );
        assert_eq!(entry.feature, BuiltInFeature::ClipboardHistory);
        assert_eq!(entry.icon, None);
    }

    #[test]
    fn test_builtin_entry_new_with_icon() {
        let entry = BuiltInEntry::new_with_icon(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test"],
            BuiltInFeature::ClipboardHistory,
            "📋",
        );

        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.name, "Test Entry");
        assert_eq!(entry.icon, Some("📋".to_string()));
    }

    #[test]
    fn test_builtin_entry_clone() {
        let entry = BuiltInEntry::new_with_icon(
            "test-id",
            "Test Entry",
            "Test description",
            vec!["test"],
            BuiltInFeature::AppLauncher,
            "🚀",
        );

        let cloned = entry.clone();
        assert_eq!(entry.id, cloned.id);
        assert_eq!(entry.name, cloned.name);
        assert_eq!(entry.description, cloned.description);
        assert_eq!(entry.keywords, cloned.keywords);
        assert_eq!(entry.feature, cloned.feature);
        assert_eq!(entry.icon, cloned.icon);
    }

    #[test]
    fn test_builtin_config_clone() {
        let config = BuiltInConfig {
            clipboard_history: true,
            app_launcher: false,
            window_switcher: true,
        };

        let cloned = config.clone();
        assert_eq!(config.clipboard_history, cloned.clipboard_history);
        assert_eq!(config.app_launcher, cloned.app_launcher);
        assert_eq!(config.window_switcher, cloned.window_switcher);
    }

    #[test]
    fn test_system_action_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that system action entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-empty-trash"));
        assert!(entries.iter().any(|e| e.id == "builtin-lock-screen"));
        assert!(entries.iter().any(|e| e.id == "builtin-toggle-dark-mode"));
        // Volume presets
        assert!(entries.iter().any(|e| e.id == "builtin-volume-0"));
        assert!(entries.iter().any(|e| e.id == "builtin-volume-50"));
        assert!(entries.iter().any(|e| e.id == "builtin-volume-100"));
        // Brightness presets
        assert!(entries.iter().any(|e| e.id == "builtin-brightness-0"));
        assert!(entries.iter().any(|e| e.id == "builtin-brightness-50"));
        assert!(entries.iter().any(|e| e.id == "builtin-brightness-100"));
        assert!(entries.iter().any(|e| e.id == "builtin-system-preferences"));
    }

    #[test]
    fn test_window_action_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that window action entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-tile-left"));
        assert!(entries.iter().any(|e| e.id == "builtin-tile-right"));
        assert!(entries.iter().any(|e| e.id == "builtin-maximize-window"));
        assert!(entries.iter().any(|e| e.id == "builtin-minimize-window"));
    }

    #[test]
    fn test_notes_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that notes command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-new-note"));
        assert!(entries.iter().any(|e| e.id == "builtin-search-notes"));
        assert!(entries.iter().any(|e| e.id == "builtin-quick-capture"));
    }

    #[test]
    fn test_script_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that script command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-new-script"));
        assert!(entries.iter().any(|e| e.id == "builtin-new-scriptlet"));
    }

    #[test]
    fn test_permission_command_entries_exist() {
        let config = BuiltInConfig::default();
        let entries = get_builtin_entries(&config);

        // Check that permission command entries exist
        assert!(entries.iter().any(|e| e.id == "builtin-check-permissions"));
        assert!(entries
            .iter()
            .any(|e| e.id == "builtin-request-accessibility"));
        assert!(entries
            .iter()
            .any(|e| e.id == "builtin-accessibility-settings"));
    }

    #[test]
    fn test_system_action_type_equality() {
        assert_eq!(SystemActionType::EmptyTrash, SystemActionType::EmptyTrash);
        assert_ne!(SystemActionType::EmptyTrash, SystemActionType::LockScreen);
    }

    #[test]
    fn test_window_action_type_equality() {
        assert_eq!(WindowActionType::TileLeft, WindowActionType::TileLeft);
        assert_ne!(WindowActionType::TileLeft, WindowActionType::TileRight);
    }

    #[test]
    fn test_builtin_feature_system_action() {
        let feature = BuiltInFeature::SystemAction(SystemActionType::ToggleDarkMode);
        assert_eq!(
            feature,
            BuiltInFeature::SystemAction(SystemActionType::ToggleDarkMode)
        );
        assert_ne!(
            feature,
            BuiltInFeature::SystemAction(SystemActionType::Sleep)
        );
    }

    #[test]
    fn test_builtin_feature_window_action() {
        let feature = BuiltInFeature::WindowAction(WindowActionType::Maximize);
        assert_eq!(
            feature,
            BuiltInFeature::WindowAction(WindowActionType::Maximize)
        );
        assert_ne!(
            feature,
            BuiltInFeature::WindowAction(WindowActionType::Minimize)
        );
    }
}

</file>

</files>
📊 Pack Summary:
────────────────
  Total Files: 2 files
  Search Mode: ripgrep (fast)
  Total Tokens: ~18.3K (18,327 exact)
  Total Chars: 122,240 chars
       Output: -

📁 Extensions Found:
────────────────────
  .rs

📂 Top 10 Files (by tokens):
──────────────────────────
     10.4K - src/builtins.rs
      7.9K - src/execute_script.rs
