                            ExternalCommand::SimulateKey { ref key, ref modifiers, ref target, ref request_id } => {
                                // SimulateKey: Enter - accept menu-syntax popup
                                // SimulateKey: Enter - execute selected
                                view.dispatch_simulate_key(
                                    window,
                                    ctx,
                                    crate::simulate_key_dispatch::SimulatedKeyInput {
                                        key,
                                        modifiers,
                                        target: target.as_ref(),
                                    },
                                );
                                if let Some(rid) = request_id {
                                    if let Some(ref sender) = view.response_sender {
                                        let _ = sender.try_send(
                                            crate::protocol::Message::external_command_result(
                                                rid.to_string(),
                                                "simulateKey".to_string(),
                                                true,
                                                None,
                                                None,
                                            ),
                                        );
                                    }
                                }
                            }
