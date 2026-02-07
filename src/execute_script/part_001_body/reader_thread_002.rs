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
                                        // FIX: Wait for stderr reader to complete (with timeout)
                                        // before reading buffer. This prevents partial error captures
                                        // when stderr flushes late after bun exits.
                                        // 100ms timeout is generous - stderr should drain quickly.
                                        let stderr_output = stderr_capture
                                            .as_ref()
                                            .map(|cap| {
                                                cap.get_contents_with_timeout(
                                                    std::time::Duration::from_millis(100),
                                                )
                                            })
                                            .filter(|s| !s.is_empty());

                                        if let Some(ref stderr_text) = stderr_output {
                                            logging::log(
                                                "EXEC",
                                                &format!(
                                                    "Captured stderr from buffer ({} bytes)",
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

                                // FIX: Wait for stderr reader to complete before reading
                                let stderr_output = stderr_capture
                                    .as_ref()
                                    .map(|cap| {
                                        cap.get_contents_with_timeout(
                                            std::time::Duration::from_millis(100),
                                        )
                                    })
                                    .filter(|s| !s.is_empty());

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
