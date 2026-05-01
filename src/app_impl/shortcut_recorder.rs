use super::*;

use std::sync::{Mutex, OnceLock};

use gpui::{
    div, AnyWindowHandle, Bounds, DisplayId, Entity, FocusHandle, Pixels, Point, Render, Size,
    WeakEntity, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind, WindowOptions,
};

const SHORTCUT_RECORDER_POPUP_WIDTH: f32 = 360.0;
const SHORTCUT_RECORDER_POPUP_HEIGHT: f32 = 196.0;
#[cfg(target_os = "macos")]
const NS_WINDOW_ABOVE: i64 = 1;

static SHORTCUT_RECORDER_WINDOW: OnceLock<
    Mutex<Option<WindowHandle<ShortcutRecorderPopupWindow>>>,
> = OnceLock::new();

struct ShortcutRecorderPopupWindow {
    recorder: Entity<crate::components::shortcut_recorder::ShortcutRecorder>,
    app: WeakEntity<ScriptListApp>,
    focus_handle: FocusHandle,
}

impl ShortcutRecorderPopupWindow {
    fn new(
        command_id: String,
        command_name: String,
        theme: std::sync::Arc<theme::Theme>,
        app: WeakEntity<ScriptListApp>,
        cx: &mut Context<Self>,
    ) -> Self {
        let recorder_theme = std::sync::Arc::clone(&theme);
        let recorder = cx.new(move |cx| {
            crate::components::shortcut_recorder::ShortcutRecorder::new(cx, recorder_theme)
                .with_detached_window(true)
                .with_command_name(command_name)
                .with_command_description(format!("ID: {}", command_id))
        });

        Self {
            recorder,
            app,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for ShortcutRecorderPopupWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let recorder_fh = self.recorder.read(cx).focus_handle.clone();
        if !recorder_fh.is_focused(window) {
            window.focus(&recorder_fh, cx);
        }

        let pending_action = self
            .recorder
            .update(cx, |recorder, _cx| recorder.take_pending_action());

        if let Some(action) = pending_action {
            let app = self.app.clone();
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    if let Some(app) = app.upgrade() {
                        app.update(cx, |app, cx| match action {
                            crate::components::shortcut_recorder::RecorderAction::Save(
                                recorded,
                            ) => {
                                app.handle_shortcut_save(&recorded, cx);
                            }
                            crate::components::shortcut_recorder::RecorderAction::Cancel => {
                                app.close_shortcut_recorder(cx);
                            }
                        });
                    } else {
                        close_shortcut_recorder_window(cx);
                    }
                });
            })
            .detach();
        }

        div()
            .id("shortcut-recorder-window")
            .relative()
            .w_full()
            .h_full()
            .track_focus(&self.focus_handle)
            .child(self.recorder.clone())
    }
}

fn shortcut_recorder_window_bounds(parent_bounds: Bounds<Pixels>) -> Bounds<Pixels> {
    let width = px(SHORTCUT_RECORDER_POPUP_WIDTH).min(parent_bounds.size.width);
    let height = px(SHORTCUT_RECORDER_POPUP_HEIGHT).min(parent_bounds.size.height);
    let x = parent_bounds.origin.x + ((parent_bounds.size.width - width) / 2.0);
    let y = parent_bounds.origin.y + ((parent_bounds.size.height - height) / 2.0);

    Bounds {
        origin: Point { x, y },
        size: Size { width, height },
    }
}

fn close_shortcut_recorder_window(cx: &mut App) {
    crate::windows::remove_automation_window("shortcut-recorder-popup");

    if let Some(storage) = SHORTCUT_RECORDER_WINDOW.get() {
        if let Ok(mut guard) = storage.lock() {
            if let Some(handle) = guard.take() {
                let _ = handle.update(cx, |_popup, window, _cx| {
                    window.remove_window();
                });
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn shortcut_recorder_ns_window(window: &mut Window) -> Option<cocoa::base::id> {
    if let Ok(window_handle) = raw_window_handle::HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::AppKit(appkit) = window_handle.as_raw() {
            use cocoa::base::nil;
            use objc::{msg_send, sel, sel_impl};

            let ns_view = appkit.ns_view.as_ptr() as cocoa::base::id;
            unsafe {
                let ns_window: cocoa::base::id = msg_send![ns_view, window];
                if ns_window != nil {
                    return Some(ns_window);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn attach_shortcut_recorder_to_parent_window(
    cx: &mut App,
    parent_window_handle: AnyWindowHandle,
    child_ns_window: cocoa::base::id,
) {
    let _ = cx.update_window(parent_window_handle, move |_, parent_window, _cx| {
        let Some(parent_ns_window) = shortcut_recorder_ns_window(parent_window) else {
            return;
        };

        unsafe {
            use cocoa::base::nil;
            use objc::{msg_send, sel, sel_impl};

            if parent_ns_window == nil
                || child_ns_window == nil
                || parent_ns_window == child_ns_window
            {
                return;
            }

            let _: () =
                msg_send![parent_ns_window, addChildWindow:child_ns_window ordered:NS_WINDOW_ABOVE];
            let _: () = msg_send![child_ns_window, orderFrontRegardless];
            let _: () = msg_send![child_ns_window, makeKeyWindow];
        }
    });
}

fn open_shortcut_recorder_window(
    cx: &mut App,
    app: WeakEntity<ScriptListApp>,
    command_id: String,
    command_name: String,
    theme: std::sync::Arc<theme::Theme>,
    parent_window_handle: AnyWindowHandle,
    parent_bounds: Bounds<Pixels>,
    display_id: Option<DisplayId>,
) -> anyhow::Result<WindowHandle<ShortcutRecorderPopupWindow>> {
    close_shortcut_recorder_window(cx);

    let window_background = if theme.is_vibrancy_enabled() {
        WindowBackgroundAppearance::Blurred
    } else {
        WindowBackgroundAppearance::Opaque
    };
    let is_dark_vibrancy = theme.should_use_dark_vibrancy();
    let bounds = shortcut_recorder_window_bounds(parent_bounds);

    let window_theme = std::sync::Arc::clone(&theme);
    // Intentionally not Root-wrapped: this popup is fixed compact capture chrome.
    // Keep focus/root behavior unchanged unless capture dismissal is retested.
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            window_background,
            focus: true,
            show: true,
            kind: WindowKind::PopUp,
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            display_id,
            ..Default::default()
        },
        move |_window, cx| {
            cx.new(|cx| {
                ShortcutRecorderPopupWindow::new(command_id, command_name, window_theme, app, cx)
            })
        },
    )?;

    #[cfg(target_os = "macos")]
    {
        let _ = handle.update(cx, move |_popup, window, cx| {
            window.defer(cx, move |window, cx| {
                if let Some(ns_window) = shortcut_recorder_ns_window(window) {
                    unsafe {
                        crate::platform::configure_actions_popup_window(
                            ns_window,
                            is_dark_vibrancy,
                        );
                    }
                    attach_shortcut_recorder_to_parent_window(cx, parent_window_handle, ns_window);
                }
            });
        });
    }

    let storage = SHORTCUT_RECORDER_WINDOW.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = storage.lock() {
        *guard = Some(handle);
    }

    if let Some(parent_automation_id) = crate::windows::focused_automation_window_id() {
        let popup_bounds = crate::protocol::AutomationWindowBounds {
            x: f32::from(bounds.origin.x) as f64,
            y: f32::from(bounds.origin.y) as f64,
            width: f32::from(bounds.size.width) as f64,
            height: f32::from(bounds.size.height) as f64,
        };
        if let Err(error) = crate::windows::register_attached_popup(
            "shortcut-recorder-popup".to_string(),
            crate::protocol::AutomationWindowKind::PromptPopup,
            Some("Shortcut Recorder".to_string()),
            Some("shortcutRecorder".to_string()),
            Some(popup_bounds),
            Some(parent_automation_id.as_str()),
        ) {
            tracing::warn!(
                target: "script_kit::shortcut",
                error = %error,
                "Failed to register shortcut recorder popup"
            );
        }
    }

    logging::log(
        "SHORTCUT",
        "Shortcut recorder popup window opened with vibrancy",
    );

    Ok(handle)
}

impl ScriptListApp {
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
    /// NOTE: This is the legacy approach. For new code, use `show_shortcut_recorder()` instead
    /// which opens the detached shortcut recorder popup.
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

    /// Show the detached shortcut recorder popup for a command.
    ///
    /// This replaces `open_config_for_shortcut` for non-script commands.
    /// For scripts, we still open the script file directly to edit the // Shortcut: comment.
    ///
    /// # Arguments
    /// * `command_id` - The unique identifier for the command (e.g., "builtin/clipboard-history")
    /// * `command_name` - Human-readable name of the command
    /// * `cx` - The context for UI updates
    pub(crate) fn show_shortcut_recorder(
        &mut self,
        command_id: String,
        command_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "SHORTCUT",
            &format!(
                "Showing shortcut recorder for '{}' (id: {})",
                command_name, command_id
            ),
        );

        // Store state so parent key handlers treat the recorder as modal while
        // the native popup owns actual input.
        self.shortcut_recorder_state = Some(ShortcutRecorderState {
            command_id: command_id.clone(),
            command_name: command_name.clone(),
        });
        self.shortcut_recorder_entity = None;

        // Close actions popup if open
        self.show_actions_popup = false;
        self.actions_dialog = None;

        let app = cx.entity().downgrade();
        let theme = std::sync::Arc::clone(&self.theme);
        let parent_window_handle = window.window_handle();
        let parent_bounds = window.bounds();
        let display_id = window.display(cx).map(|display| display.id());

        cx.spawn(async move |this, cx| {
            cx.update(|cx| {
                if let Err(error) = open_shortcut_recorder_window(
                    cx,
                    app,
                    command_id,
                    command_name,
                    theme,
                    parent_window_handle,
                    parent_bounds,
                    display_id,
                ) {
                    tracing::error!(
                        target: "script_kit::shortcut",
                        error = %error,
                        "Failed to open shortcut recorder popup"
                    );
                    let _ = this.update(cx, |app, cx| {
                        app.shortcut_recorder_state = None;
                        app.shortcut_recorder_entity = None;
                        app.show_error_toast(
                            format!("Failed to open shortcut recorder: {}", error),
                            cx,
                        );
                    });
                }
            });
        })
        .detach();

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
            cx.spawn(async move |_this, cx| {
                cx.update(|cx| {
                    close_shortcut_recorder_window(cx);
                });
            })
            .detach();
            // Return focus to the main filter input
            self.pending_focus = Some(FocusTarget::MainFilter);
            cx.notify();
        }
    }

    /// Legacy inline overlay path for the shortcut recorder.
    ///
    /// Returns None if no recorder is active.
    ///
    /// The current native popup path returns None while shortcut_recorder_state
    /// is set so the parent is not dimmed. The remaining inline path is kept for
    /// legacy state/entity handling.
    pub(crate) fn render_shortcut_recorder_overlay(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        if self.shortcut_recorder_state.is_some() {
            return None;
        }

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
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(1))
                            .await;
                        cx.update(|cx| {
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
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(1))
                            .await;
                        cx.update(|cx| {
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
                            Some(HUD_MEDIUM_MS),
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
                            Some(HUD_LONG_MS),
                            cx,
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to save shortcut");
                self.show_error_toast(format!("Failed to save shortcut: {}", e), cx);
            }
        }

        // Close the recorder and restore focus
        self.close_shortcut_recorder(cx);
    }
}
