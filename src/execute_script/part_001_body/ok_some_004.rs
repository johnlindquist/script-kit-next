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

