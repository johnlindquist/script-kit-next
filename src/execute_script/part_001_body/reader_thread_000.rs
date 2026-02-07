                    // These variables keep the process alive - they're dropped when the thread exits
                    let _keep_alive_handle = _process_handle;
                    let mut keep_alive_child = _child;
                    // FIX: Use the stderr capture which includes both buffer and join handle
                    // The buffer is populated by the stderr reader thread, and we wait for it
                    // to complete (with timeout) before reading to prevent partial captures.
                    let stderr_capture = stderr_capture;
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
