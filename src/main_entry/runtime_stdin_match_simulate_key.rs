                            ExternalCommand::SimulateKey { ref key, ref modifiers, ref target, .. } => {
                                view.dispatch_simulate_key(
                                    window,
                                    ctx,
                                    crate::simulate_key_dispatch::SimulatedKeyInput {
                                        key,
                                        modifiers,
                                        target: target.as_ref(),
                                    },
                                );
                            }
