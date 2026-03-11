// Shortcut and alias action handlers for handle_action dispatch.
//
// Contains: configure_shortcut, add_shortcut, update_shortcut, remove_shortcut,
// add_alias, update_alias, remove_alias.

impl ScriptListApp {
    /// Handle shortcut and alias actions. Returns `true` if handled.
    fn handle_shortcut_alias_action(
        &mut self,
        action_id: &str,
        trace_id: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        let _ = trace_id; // Reserved for future async path logging
        match action_id {
            // Handle both legacy "configure_shortcut" and new dynamic actions
            // "add_shortcut" and "update_shortcut" open the shortcut recorder
            "configure_shortcut" | "add_shortcut" | "update_shortcut" => {
                tracing::info!(category = "UI", action = action_id, "action triggered");
                if let Some(result) = self.get_selected_result() {
                    match result {
                        // Scripts: open the script file to edit // Shortcut: comment
                        scripts::SearchResult::Script(m) => {
                            self.edit_script(&m.script.path);
                            self.hide_main_and_reset(cx);
                        }
                        scripts::SearchResult::Agent(m) => {
                            self.edit_script(&m.agent.path);
                            self.hide_main_and_reset(cx);
                        }
                        // Non-scripts: show inline shortcut recorder
                        scripts::SearchResult::Scriptlet(m) => {
                            let command_id = format!("scriptlet/{}", m.scriptlet.name);
                            let command_name = m.scriptlet.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::BuiltIn(m) => {
                            let command_id = format!("builtin/{}", m.entry.id);
                            let command_name = m.entry.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::App(m) => {
                            // Use bundle ID if available, otherwise use name
                            let command_id = if let Some(ref bundle_id) = m.app.bundle_id {
                                format!("app/{}", bundle_id)
                            } else {
                                format!(
                                    "app/{}",
                                    m.app.name.to_lowercase().replace(' ', "-")
                                )
                            };
                            let command_name = m.app.name.clone();
                            self.show_shortcut_recorder(command_id, command_name, cx);
                        }
                        scripts::SearchResult::Window(_) => {
                            self.show_error_toast(
                                "Window shortcuts not supported - windows are transient",
                                cx,
                            );
                        }
                        scripts::SearchResult::Fallback(m) => match &m.fallback {
                            crate::fallbacks::collector::FallbackItem::Builtin(b) => {
                                let command_id =
                                    format!("fallback/{}", m.fallback.name());
                                let command_name = b.name.to_string();
                                self.show_shortcut_recorder(command_id, command_name, cx);
                            }
                            crate::fallbacks::collector::FallbackItem::Script(s) => {
                                // Script-based fallback - open the script
                                self.edit_script(&s.script.path);
                                self.hide_main_and_reset(cx);
                            }
                        },
                    }
                } else {
                    self.show_error_toast(
                        selection_required_message_for_action(action_id),
                        cx,
                    );
                }
                true
            }
            // "remove_shortcut" removes the existing shortcut from the registry
            "remove_shortcut" => {
                tracing::info!(category = "UI", "remove shortcut action");
                if let Some(result) = self.get_selected_result() {
                    let command_id_opt = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(format!("script/{}", m.script.name))
                        }
                        scripts::SearchResult::Scriptlet(m) => {
                            Some(format!("scriptlet/{}", m.scriptlet.name))
                        }
                        scripts::SearchResult::BuiltIn(m) => {
                            Some(format!("builtin/{}", m.entry.id))
                        }
                        scripts::SearchResult::App(m) => {
                            if let Some(ref bundle_id) = m.app.bundle_id {
                                Some(format!("app/{}", bundle_id))
                            } else {
                                Some(format!(
                                    "app/{}",
                                    m.app.name.to_lowercase().replace(' ', "-")
                                ))
                            }
                        }
                        scripts::SearchResult::Agent(m) => {
                            Some(format!("agent/{}", m.agent.name))
                        }
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(m) => {
                            Some(format!("fallback/{}", m.fallback.name()))
                        }
                    };

                    if let Some(command_id) = command_id_opt {
                        // Remove the shortcut override from persistence
                        match crate::shortcuts::remove_shortcut_override(&command_id) {
                            Ok(()) => {
                                tracing::info!(
                                    category = "SHORTCUT",
                                    command_id = %command_id,
                                    "Removed shortcut override"
                                );
                                self.show_hud(
                                    "Shortcut removed".to_string(),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                                // Refresh scripts to update shortcut display
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "failed to remove shortcut");
                                self.show_error_toast(
                                    format!("Failed to remove shortcut: {}", e),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_error_toast(
                            "Cannot remove shortcut for this item type",
                            cx,
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_error_toast(
                        selection_required_message_for_action(action_id),
                        cx,
                    );
                }
                true
            }
            // Alias actions: add_alias, update_alias open the alias input
            "add_alias" | "update_alias" => {
                tracing::info!(category = "UI", action = action_id, "action triggered");
                if let Some(result) = self.get_selected_result() {
                    let (command_id, command_name) = match result {
                        scripts::SearchResult::Script(m) => {
                            (format!("script/{}", m.script.name), m.script.name.clone())
                        }
                        scripts::SearchResult::Scriptlet(m) => (
                            format!("scriptlet/{}", m.scriptlet.name),
                            m.scriptlet.name.clone(),
                        ),
                        scripts::SearchResult::BuiltIn(m) => {
                            (format!("builtin/{}", m.entry.id), m.entry.name.clone())
                        }
                        scripts::SearchResult::App(m) => {
                            let id = if let Some(ref bundle_id) = m.app.bundle_id {
                                format!("app/{}", bundle_id)
                            } else {
                                format!(
                                    "app/{}",
                                    m.app.name.to_lowercase().replace(' ', "-")
                                )
                            };
                            (id, m.app.name.clone())
                        }
                        scripts::SearchResult::Agent(m) => {
                            (format!("agent/{}", m.agent.name), m.agent.name.clone())
                        }
                        scripts::SearchResult::Window(_) => {
                            self.show_error_toast(
                                "Window aliases not supported - windows are transient",
                                cx,
                            );
                            return true;
                        }
                        scripts::SearchResult::Fallback(m) => (
                            format!("fallback/{}", m.fallback.name()),
                            m.fallback.name().to_string(),
                        ),
                    };
                    self.show_alias_input(command_id, command_name, cx);
                } else {
                    self.show_error_toast(
                        selection_required_message_for_action(action_id),
                        cx,
                    );
                }
                true
            }
            // "remove_alias" removes the existing alias from persistence
            "remove_alias" => {
                tracing::info!(category = "UI", "remove alias action");
                if let Some(result) = self.get_selected_result() {
                    let command_id_opt = match result {
                        scripts::SearchResult::Script(m) => {
                            Some(format!("script/{}", m.script.name))
                        }
                        scripts::SearchResult::Scriptlet(m) => {
                            Some(format!("scriptlet/{}", m.scriptlet.name))
                        }
                        scripts::SearchResult::BuiltIn(m) => {
                            Some(format!("builtin/{}", m.entry.id))
                        }
                        scripts::SearchResult::App(m) => {
                            if let Some(ref bundle_id) = m.app.bundle_id {
                                Some(format!("app/{}", bundle_id))
                            } else {
                                Some(format!(
                                    "app/{}",
                                    m.app.name.to_lowercase().replace(' ', "-")
                                ))
                            }
                        }
                        scripts::SearchResult::Agent(m) => {
                            Some(format!("agent/{}", m.agent.name))
                        }
                        scripts::SearchResult::Window(_) => None,
                        scripts::SearchResult::Fallback(m) => {
                            Some(format!("fallback/{}", m.fallback.name()))
                        }
                    };

                    if let Some(command_id) = command_id_opt {
                        // Remove the alias override from persistence
                        match crate::aliases::remove_alias_override(&command_id) {
                            Ok(()) => {
                                tracing::info!(
                                    category = "ALIAS",
                                    command_id = %command_id,
                                    "Removed alias override"
                                );
                                self.show_hud(
                                    "Alias removed".to_string(),
                                    Some(HUD_MEDIUM_MS),
                                    cx,
                                );
                                // Refresh scripts to update alias display and registry
                                self.refresh_scripts(cx);
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "failed to remove alias");
                                self.show_error_toast(
                                    format!("Failed to remove alias: {}", e),
                                    cx,
                                );
                            }
                        }
                    } else {
                        self.show_error_toast(
                            "Cannot remove alias for this item type",
                            cx,
                        );
                    }
                    self.hide_main_and_reset(cx);
                } else {
                    self.show_error_toast(
                        selection_required_message_for_action(action_id),
                        cx,
                    );
                }
                true
            }
            _ => false,
        }
    }
}
