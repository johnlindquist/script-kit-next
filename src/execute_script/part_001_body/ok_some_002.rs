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
                                    tile_position,
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
                                        protocol::WindowActionType::Tile => {
                                            if let (Some(id), Some(pos)) =
                                                (window_id, tile_position)
                                            {
                                                let wc_pos = protocol_tile_to_window_control(pos);
                                                window_control::tile_window(*id, wc_pos)
                                            } else {
                                                Err(anyhow::anyhow!(
                                                    "Missing window_id or tile_position"
                                                ))
                                            }
                                        }
                                        protocol::WindowActionType::MoveToNextDisplay => {
                                            if let Some(id) = window_id {
                                                window_control::move_to_next_display(*id)
                                            } else {
                                                Err(anyhow::anyhow!("Missing window_id"))
                                            }
                                        }
                                        protocol::WindowActionType::MoveToPreviousDisplay => {
                                            if let Some(id) = window_id {
                                                window_control::move_to_previous_display(*id)
                                            } else {
                                                Err(anyhow::anyhow!("Missing window_id"))
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

                                // Handle DisplayList directly (no UI needed)
                                if let Message::DisplayList { request_id } = &msg {
                                    logging::log(
                                        "EXEC",
                                        &format!("DisplayList request: {}", request_id),
                                    );

                                    let response = match get_displays() {
                                        Ok(displays) => Message::display_list_result(
                                            request_id.clone(),
                                            displays,
                                        ),
                                        Err(e) => {
                                            logging::log(
                                                "ERROR",
                                                &format!("Failed to get displays: {}", e),
                                            );
                                            // Return empty list on error
                                            Message::display_list_result(request_id.clone(), vec![])
                                        }
                                    };

                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!("Failed to send display list response: {}", e),
                                        );
                                    }
                                    continue;
                                }

                                // Handle FrontmostWindow directly (no UI needed)
                                if let Message::FrontmostWindow { request_id } = &msg {
                                    logging::log(
                                        "EXEC",
                                        &format!("FrontmostWindow request: {}", request_id),
                                    );

                                    let response =
                                        match window_control::get_frontmost_window_of_previous_app()
                                        {
                                            Ok(Some(window)) => {
                                                let window_info = protocol::SystemWindowInfo {
                                                    window_id: window.id,
                                                    title: window.title,
                                                    app_name: window.app,
                                                    bounds: Some(protocol::TargetWindowBounds {
                                                        x: window.bounds.x,
                                                        y: window.bounds.y,
                                                        width: window.bounds.width,
                                                        height: window.bounds.height,
                                                    }),
                                                    is_minimized: None,
                                                    is_active: Some(true),
                                                };
                                                Message::frontmost_window_result(
                                                    request_id.clone(),
                                                    Some(window_info),
                                                    None,
                                                )
                                            }
                                            Ok(None) => Message::frontmost_window_result(
                                                request_id.clone(),
                                                None,
                                                Some("No frontmost window found".to_string()),
                                            ),
                                            Err(e) => Message::frontmost_window_result(
                                                request_id.clone(),
                                                None,
                                                Some(e.to_string()),
                                            ),
                                        };

                                    if let Err(e) = reader_response_tx.send(response) {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "Failed to send frontmost window response: {}",
                                                e
                                            ),
                                        );
                                    }
                                    continue;
                                }

                                // Handle FileSearch directly (no UI needed)
