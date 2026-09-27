use super::*;

fn shortcut_config_script_path(script_name: &str) -> anyhow::Result<std::path::PathBuf> {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let sdk_path = std::path::PathBuf::from(home_dir)
        .join(".scriptkit")
        .join("sdk")
        .join(script_name);

    if sdk_path.exists() {
        return Ok(sdk_path);
    }

    let dev_path = std::env::current_dir()?.join("scripts").join(script_name);
    if dev_path.exists() {
        return Ok(dev_path);
    }

    Err(anyhow::anyhow!(
        "Could not find {} in .scriptkit/sdk or repo scripts/",
        script_name
    ))
}

fn shortcut_config_bun_path() -> String {
    crate::config::load_config()
        .bun_path
        .as_ref()
        .filter(|path| std::path::Path::new(path.as_str()).exists())
        .cloned()
        .unwrap_or_else(|| "bun".to_string())
}

impl ScriptListApp {
    pub(crate) fn write_config_command_shortcut(
        &self,
        command_id: &str,
        key: &str,
        cmd: bool,
        ctrl: bool,
        alt: bool,
        shift: bool,
    ) -> anyhow::Result<()> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)?;
        let script_path = shortcut_config_script_path("update-config-shortcut.ts")?;
        let output = std::process::Command::new(shortcut_config_bun_path())
            .arg(script_path)
            .arg(command_id)
            .arg(key)
            .arg(cmd.to_string())
            .arg(ctrl.to_string())
            .arg(alt.to_string())
            .arg(shift.to_string())
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(anyhow::anyhow!(
                "config shortcut update failed: {}{}",
                stderr.trim(),
                stdout.trim()
            ))
        }
    }

    pub(crate) fn remove_config_command_shortcut(&self, command_id: &str) -> anyhow::Result<()> {
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)?;
        let script_path = shortcut_config_script_path("remove-config-shortcut.ts")?;
        let output = std::process::Command::new(shortcut_config_bun_path())
            .arg(script_path)
            .arg(command_id)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(anyhow::anyhow!(
                "config shortcut removal failed: {}{}",
                stderr.trim(),
                stdout.trim()
            ))
        }
    }
}

impl ScriptListApp {
    pub(crate) fn main_window_modal_owns_keyboard(&self) -> bool {
        self.shortcut_recorder_state.is_some()
    }

    pub(crate) fn edit_script(&mut self, path: &std::path::Path) {
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
    #[allow(dead_code)]
    pub(crate) fn open_config_for_shortcut(&mut self, command_id: &str) {
        let config_path = shellexpand::tilde("~/.scriptkit/config.ts").to_string();
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
                tracing::error!(error = %e, "Failed to create config.ts");
            }
        }

        // Copy command_id to clipboard as a hint
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = self.pbcopy(command_id) {
                tracing::error!(error = %e, "Failed to copy command ID to clipboard");
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
                Err(e) => tracing::error!(error = %e, "Failed to open config.ts in editor"),
            }
        });
    }

    /// Create config.ts template with keyboard shortcut documentation
    #[allow(dead_code)]
    pub(crate) fn create_config_template(path: &std::path::Path) -> std::io::Result<()> {
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

    /// Handle saving a shortcut from the recorder.
    ///
    /// This saves the shortcut to config.ts and updates the live hotkey registry.
    pub(crate) fn handle_shortcut_save(
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

        let recorded_key = recorded.key.clone().unwrap_or_default();
        let shortcut_str = shortcut.to_canonical_string();
        if let MainServices::OwnedFixtures(sources) = &mut self.main_services {
            Arc::make_mut(sources).shortcut_overrides.insert(command_id, shortcut_str);
            self.mark_main_data_changed();
            crate::runtime_policy::record_completed_fixture_effect();
            self.close_shortcut_recorder(cx);
            return;
        }

        if let Some(conflict) =
            crate::hotkeys::shortcut_conflict_for_recording(&command_id, &shortcut_str)
        {
            tracing::warn!(
                command_id = %command_id,
                conflicting_command_id = %conflict.command_id,
                shortcut = %conflict.shortcut,
                "Shortcut save blocked by live route conflict"
            );
            self.notice_failed(
                format!("Shortcut already used by {}", conflict.command_name),
                cx,
            );
            return;
        }

        // Save to config.ts via the same CLI path used by script-side tooling.
        match self.write_config_command_shortcut(
            &command_id,
            &recorded_key,
            recorded.cmd,
            recorded.ctrl,
            recorded.alt,
            recorded.shift,
        ) {
            Ok(()) => {
                logging::log("SHORTCUT", "Shortcut saved to config.ts commands");

                // Register the hotkey immediately so it works without restart
                match crate::hotkeys::update_script_hotkey(&command_id, None, Some(&shortcut_str)) {
                    Ok(()) => {
                        logging::log("SHORTCUT", "Registered config shortcut immediately");
                        self.one_window_notice(crate::line_notice::INFO, format!("Shortcut set: {} (active now)", shortcut.display()), cx);
                    }
                    Err(e) => {
                        // Shortcut saved but couldn't register, so the config remains durable while
                        // the live binding stays inactive until the user resolves the OS/global conflict.
                        logging::log(
                            "SHORTCUT",
                            &format!(
                                "Shortcut saved but registration failed: {} - not active now",
                                e
                            ),
                        );
                        self.one_window_notice(crate::line_notice::INFO, format!("Shortcut saved: {} (not active now)", shortcut.display()), cx);
                    }
                }
                self.refresh_scripts(cx);
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to save shortcut");
                self.notice_failed(format!("Failed to save shortcut: {}", e), cx);
            }
        }

        // Close the recorder and restore focus
        self.close_shortcut_recorder(cx);
    }
}
