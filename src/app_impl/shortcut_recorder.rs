use super::*;

impl ScriptListApp {
    fn edit_script(&mut self, path: &std::path::Path) {
        let editor = self.config.get_editor();
        logging::log(
            "UI",
            &format!("Opening script in editor '{}': {}", editor, path.display()),
        );
        let path_str = path.to_string_lossy().to_string();

        std::thread::spawn(move || {
            use std::process::Command;
            match Command::new(&editor).arg(&path_str).spawn() {
                Ok(_) => logging::log("UI", &format!("Successfully spawned editor: {}", editor)),
                Err(e) => logging::log(
                    "ERROR",
                    &format!("Failed to spawn editor '{}': {}", editor, e),
                ),
            }
        });
    }

    /// Open config.ts for configuring a keyboard shortcut
    /// Creates the file with documentation if it doesn't exist
    ///
    /// NOTE: This is the legacy approach. For new code, use `show_shortcut_recorder()` instead
    /// which provides an inline modal UI for recording shortcuts.
    #[allow(dead_code)]
    fn open_config_for_shortcut(&mut self, command_id: &str) {
        let config_path = shellexpand::tilde("~/.scriptkit/kit/config.ts").to_string();
        let editor = self.config.get_editor();

        logging::log(
            "UI",
            &format!(
                "Opening config.ts for shortcut configuration: {} (command: {})",
                config_path, command_id
            ),
        );

        // Ensure config.ts exists with documentation
        let config_path_buf = std::path::PathBuf::from(&config_path);
        if !config_path_buf.exists() {
            if let Err(e) = Self::create_config_template(&config_path_buf) {
                logging::log("ERROR", &format!("Failed to create config.ts: {}", e));
            }
        }

        // Copy command_id to clipboard as a hint
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = self.pbcopy(command_id) {
                logging::log("ERROR", &format!("Failed to copy command ID: {}", e));
            } else {
                self.last_output = Some(gpui::SharedString::from(format!(
                    "Copied '{}' to clipboard - paste in config.ts commands section",
                    command_id
                )));
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            use arboard::Clipboard;
            if let Ok(mut clipboard) = Clipboard::new() {
                if clipboard.set_text(command_id).is_ok() {
                    self.last_output = Some(gpui::SharedString::from(format!(
                        "Copied '{}' to clipboard - paste in config.ts commands section",
                        command_id
                    )));
                }
            }
        }

        let config_path_clone = config_path.clone();
        std::thread::spawn(move || {
            use std::process::Command;
            match Command::new(&editor).arg(&config_path_clone).spawn() {
                Ok(_) => logging::log("UI", &format!("Opened config.ts in {}", editor)),
                Err(e) => logging::log("ERROR", &format!("Failed to open config.ts: {}", e)),
            }
        });
    }

    /// Create config.ts template with keyboard shortcut documentation
    #[allow(dead_code)]
    fn create_config_template(path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let template = r#"// Script Kit Configuration
// https://scriptkit.com/docs/config

import type { Config } from "@scriptkit/sdk";

export default {
  // ============================================
  // MAIN HOTKEY
  // ============================================
  // The keyboard shortcut to open Script Kit
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // ============================================
  // KEYBOARD SHORTCUTS
  // ============================================
  // Configure shortcuts for any command (scripts, built-ins, apps, snippets)
  //
  // Command ID formats:
  //   - "script/my-script"           - User scripts (by filename without extension)
  //   - "builtin/clipboard-history"  - Built-in features
  //   - "app/com.apple.Safari"       - Apps (by bundle ID)
  //   - "scriptlet/my-snippet"       - Scriptlets/snippets
  //
  // Modifier keys: "meta" (⌘), "ctrl", "alt" (⌥), "shift"
  // Key names: "KeyA"-"KeyZ", "Digit0"-"Digit9", "Space", "Enter", etc.
  //
  // Example:
  //   commands: {
  //     "builtin/clipboard-history": {
  //       shortcut: { modifiers: ["meta", "shift"], key: "KeyV" }
  //     },
  //     "app/com.apple.Safari": {
  //       shortcut: { modifiers: ["meta", "alt"], key: "KeyS" }
  //     }
  //   }
  commands: {
    // Add your shortcuts here
  },

  // ============================================
  // WINDOW HOTKEYS
  // ============================================
  // notesHotkey: { modifiers: ["meta", "shift"], key: "KeyN" },
  // aiHotkey: { modifiers: ["meta", "shift"], key: "Space" },

  // ============================================
  // APPEARANCE
  // ============================================
  // editorFontSize: 14,
  // terminalFontSize: 14,
  // uiScale: 1.0,

  // ============================================
  // PATHS
  // ============================================
  // bun_path: "/opt/homebrew/bin/bun",
  // editor: "code",
} satisfies Config;
"#;

        let mut file = std::fs::File::create(path)?;
        file.write_all(template.as_bytes())?;
        logging::log(
            "UI",
            &format!("Created config.ts template: {}", path.display()),
        );
        Ok(())
    }

    /// Show the inline shortcut recorder for a command.
    ///
    /// This replaces `open_config_for_shortcut` for non-script commands.
    /// For scripts, we still open the script file directly to edit the // Shortcut: comment.
    ///
    /// # Arguments
    /// * `command_id` - The unique identifier for the command (e.g., "builtin/clipboard-history")
    /// * `command_name` - Human-readable name of the command
    /// * `cx` - The context for UI updates
    fn show_shortcut_recorder(
        &mut self,
        command_id: String,
        command_name: String,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "SHORTCUT",
            &format!(
                "Showing shortcut recorder for '{}' (id: {})",
                command_name, command_id
            ),
        );

        // Store state - the entity will be created in render_shortcut_recorder_overlay
        // when we have window access
        self.shortcut_recorder_state = Some(ShortcutRecorderState {
            command_id,
            command_name,
        });

        // Clear any existing entity so a new one is created with correct focus
        self.shortcut_recorder_entity = None;

        // Close actions popup if open
        self.show_actions_popup = false;
        self.actions_dialog = None;

        cx.notify();
    }

    /// Close the shortcut recorder and clear state.
    /// Returns focus to the main filter input.
    pub fn close_shortcut_recorder(&mut self, cx: &mut Context<Self>) {
        if self.shortcut_recorder_state.is_some() || self.shortcut_recorder_entity.is_some() {
            logging::log(
                "SHORTCUT",
                "Closing shortcut recorder, returning focus to main filter",
            );
            self.shortcut_recorder_state = None;
            self.shortcut_recorder_entity = None;
            // Return focus to the main filter input
            self.pending_focus = Some(FocusTarget::MainFilter);
            cx.notify();
        }
    }

    /// Render the shortcut recorder overlay if state is set.
    ///
    /// Returns None if no recorder is active.
    ///
    /// The recorder is created once and persisted to maintain keyboard focus.
    /// Callbacks use cx.entity() to communicate back to the parent app.
    fn render_shortcut_recorder_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        use crate::components::shortcut_recorder::ShortcutRecorder;

        // Check if we have state but no entity yet - need to create the recorder
        let state = self.shortcut_recorder_state.as_ref()?;

        // Create entity if needed (only once per show)
        if self.shortcut_recorder_entity.is_none() {
            let command_id = state.command_id.clone();
            let command_name = state.command_name.clone();
            let theme = std::sync::Arc::clone(&self.theme);

            // Get a weak reference to the app for callbacks
            let app_entity = cx.entity().downgrade();
            let app_entity_for_cancel = app_entity.clone();

            let recorder = cx.new(move |cx| {
                // Create the recorder with its own focus handle from its own context
                // This is CRITICAL for keyboard events to work
                let mut r = ShortcutRecorder::new(cx, theme);
                r.set_command_name(Some(command_name.clone()));
                r.set_command_description(Some(format!("ID: {}", command_id)));

                // Set save callback - directly updates the app via entity reference
                let app_for_save = app_entity.clone();
                r.on_save = Some(Box::new(move |recorded| {
                    logging::log(
                        "SHORTCUT",
                        &format!(
                            "Recorder on_save triggered: {}",
                            recorded.to_config_string()
                        ),
                    );
                    // Schedule the save on the app - this will be picked up by the app
                    if app_for_save.upgrade().is_some() {
                        // We can't call update() from here directly, so we'll use a different approach
                        // Store the result in the recorder and check it in render
                        logging::log("SHORTCUT", "Save callback - app entity available");
                    }
                }));

                // Set cancel callback
                let app_for_cancel = app_entity_for_cancel.clone();
                r.on_cancel = Some(Box::new(move || {
                    logging::log("SHORTCUT", "Recorder on_cancel triggered");
                    if let Some(_app) = app_for_cancel.upgrade() {
                        logging::log("SHORTCUT", "Cancel callback - app entity available");
                    }
                }));

                r
            });

            self.shortcut_recorder_entity = Some(recorder);
            logging::log("SHORTCUT", "Created new shortcut recorder entity");
        }

        // Get the existing entity
        let recorder = self.shortcut_recorder_entity.as_ref()?;

        // ALWAYS focus the recorder to ensure it captures keyboard input
        // This is critical for modal behavior - the recorder must have focus
        let recorder_fh = recorder.read(cx).focus_handle.clone();
        let was_focused = recorder_fh.is_focused(window);
        window.focus(&recorder_fh, cx);
        if !was_focused {
            logging::log("SHORTCUT", "Focused shortcut recorder (was not focused)");
        }

        // Check for pending actions from the recorder (Save or Cancel)
        // We need to update() the recorder entity to take the pending action
        let pending_action = recorder.update(cx, |r, _cx| r.take_pending_action());

        if let Some(action) = pending_action {
            use crate::components::shortcut_recorder::RecorderAction;
            match action {
                RecorderAction::Save(recorded) => {
                    logging::log(
                        "SHORTCUT",
                        &format!("Handling save action: {}", recorded.to_config_string()),
                    );
                    // Handle the save - need to defer to avoid borrow issues
                    let app_entity = cx.entity().downgrade();
                    cx.spawn(async move |_this, cx| {
                        gpui::Timer::after(std::time::Duration::from_millis(1)).await;
                        let _ = cx.update(|cx| {
                            if let Some(app) = app_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    this.handle_shortcut_save(&recorded, cx);
                                });
                            }
                        });
                    })
                    .detach();
                }
                RecorderAction::Cancel => {
                    logging::log("SHORTCUT", "Handling cancel action");
                    // Handle the cancel - need to defer to avoid borrow issues
                    let app_entity = cx.entity().downgrade();
                    cx.spawn(async move |_this, cx| {
                        gpui::Timer::after(std::time::Duration::from_millis(1)).await;
                        let _ = cx.update(|cx| {
                            if let Some(app) = app_entity.upgrade() {
                                app.update(cx, |this, cx| {
                                    this.close_shortcut_recorder(cx);
                                });
                            }
                        });
                    })
                    .detach();
                }
            }
        }

        // Clone the entity for rendering
        let recorder_clone = recorder.clone();

        // Render the recorder as a child element
        Some(
            div()
                .id("shortcut-recorder-wrapper")
                .absolute()
                .inset_0()
                .child(recorder_clone)
                .into_any_element(),
        )
    }

    /// Handle saving a shortcut from the recorder.
    ///
    /// This saves the shortcut to ~/.scriptkit/shortcuts.json and updates the registry.
    fn handle_shortcut_save(
        &mut self,
        recorded: &crate::components::shortcut_recorder::RecordedShortcut,
        cx: &mut Context<Self>,
    ) {
        let Some(ref state) = self.shortcut_recorder_state else {
            logging::log("SHORTCUT", "No recorder state when trying to save");
            return;
        };

        let command_id = state.command_id.clone();
        let command_name = state.command_name.clone();

        // Convert RecordedShortcut to the persistence Shortcut type
        let shortcut = crate::shortcuts::Shortcut {
            key: recorded.key.clone().unwrap_or_default().to_lowercase(),
            modifiers: crate::shortcuts::Modifiers {
                cmd: recorded.cmd,
                ctrl: recorded.ctrl,
                alt: recorded.alt,
                shift: recorded.shift,
            },
        };

        logging::log(
            "SHORTCUT",
            &format!(
                "Saving shortcut for '{}' ({}): {}",
                command_name,
                command_id,
                shortcut.to_canonical_string()
            ),
        );

        // Save to persistence
        match crate::shortcuts::save_shortcut_override(&command_id, &shortcut) {
            Ok(()) => {
                logging::log("SHORTCUT", "Shortcut saved to shortcuts.json");

                // Register the hotkey immediately so it works without restart
                let shortcut_str = shortcut.to_canonical_string();
                match crate::hotkeys::register_dynamic_shortcut(
                    &command_id,
                    &shortcut_str,
                    &command_name,
                ) {
                    Ok(id) => {
                        logging::log(
                            "SHORTCUT",
                            &format!("Registered hotkey immediately (id: {})", id),
                        );
                        self.show_hud(
                            format!("Shortcut set: {} (active now)", shortcut.display()),
                            Some(2000),
                            cx,
                        );
                    }
                    Err(e) => {
                        // Shortcut saved but couldn't register - will work after restart
                        logging::log(
                            "SHORTCUT",
                            &format!("Shortcut saved but registration failed: {} - will work after restart", e),
                        );
                        self.show_hud(
                            format!("Shortcut set: {} (restart to activate)", shortcut.display()),
                            Some(3000),
                            cx,
                        );
                    }
                }
            }
            Err(e) => {
                logging::log("ERROR", &format!("Failed to save shortcut: {}", e));
                self.show_hud(format!("Failed to save shortcut: {}", e), Some(4000), cx);
            }
        }

        // Close the recorder and restore focus
        self.close_shortcut_recorder(cx);
    }

    /// Show the alias input overlay for configuring a command alias.
    ///
    /// The alias input allows users to set a text alias that can be typed
    /// in the main menu to quickly run a command.
}
