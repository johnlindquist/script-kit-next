use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{Context, FocusHandle};

use crate::logging;
use crate::theme::Theme;

use super::types::{
    compute_overlay_appear_style, ConflictChecker, OnCancelCallback, OnSaveCallback,
    OverlayAppearStyle, RecordedShortcut, RecorderAction, ShortcutConflict, ShortcutRecorderColors,
};

/// Shortcut Recorder Modal Component
///
/// A modal dialog for recording keyboard shortcuts with visual feedback.
pub struct ShortcutRecorder {
    /// Focus handle for keyboard input
    pub focus_handle: FocusHandle,
    /// Theme for styling
    pub theme: Arc<Theme>,
    /// Pre-computed colors
    pub colors: ShortcutRecorderColors,
    /// Name of the command being configured
    pub command_name: Option<String>,
    /// Description of the command
    pub command_description: Option<String>,
    /// Currently recorded shortcut (final result with key)
    pub shortcut: RecordedShortcut,
    /// Currently held modifiers (for live display before final key)
    pub current_modifiers: gpui::Modifiers,
    /// Current conflict if any
    pub conflict: Option<ShortcutConflict>,
    /// Callback when save is pressed
    pub on_save: Option<OnSaveCallback>,
    /// Callback when cancel is pressed
    pub on_cancel: Option<OnCancelCallback>,
    /// Function to check for conflicts
    pub conflict_checker: Option<ConflictChecker>,
    /// Whether recording is active (listening for keys)
    pub is_recording: bool,
    /// Pending action for the parent to handle (polled after render)
    pub pending_action: Option<RecorderAction>,
    /// Timestamp for enter animation start (fade/slide-in)
    pub(super) overlay_animation_started_at: Instant,
    /// Ensures we schedule at most one animation tick task at a time
    pub(super) overlay_animation_tick_scheduled: bool,
}

impl ShortcutRecorder {
    /// Create a new shortcut recorder
    /// The focus_handle MUST be created from the entity's own context (cx.focus_handle())
    /// for keyboard events to work properly.
    pub fn new(cx: &mut Context<Self>, theme: Arc<Theme>) -> Self {
        let colors = ShortcutRecorderColors::from_theme(&theme);
        // Create focus handle from THIS entity's context - critical for keyboard events
        let focus_handle = cx.focus_handle();
        logging::log("SHORTCUT", "Created ShortcutRecorder with new focus handle");
        Self {
            focus_handle,
            theme,
            colors,
            command_name: None,
            command_description: None,
            shortcut: RecordedShortcut::new(),
            current_modifiers: gpui::Modifiers::default(),
            conflict: None,
            on_save: None,
            on_cancel: None,
            conflict_checker: None,
            is_recording: true,
            pending_action: None,
            overlay_animation_started_at: Instant::now(),
            overlay_animation_tick_scheduled: false,
        }
    }

    /// Set the command name
    pub fn with_command_name(mut self, name: impl Into<String>) -> Self {
        self.command_name = Some(name.into());
        self
    }

    /// Set the command description
    pub fn with_command_description(mut self, description: impl Into<String>) -> Self {
        self.command_description = Some(description.into());
        self
    }

    /// Set the save callback
    pub fn on_save(mut self, callback: impl Fn(RecordedShortcut) + 'static) -> Self {
        self.on_save = Some(Box::new(callback));
        self
    }

    /// Set the cancel callback
    pub fn on_cancel(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_cancel = Some(Box::new(callback));
        self
    }

    /// Set the conflict checker
    pub fn with_conflict_checker(
        mut self,
        checker: impl Fn(&RecordedShortcut) -> Option<ShortcutConflict> + 'static,
    ) -> Self {
        self.conflict_checker = Some(Box::new(checker));
        self
    }

    /// Set command name (mutable version)
    pub fn set_command_name(&mut self, name: Option<String>) {
        self.command_name = name;
    }

    /// Set command description (mutable version)
    pub fn set_command_description(&mut self, description: Option<String>) {
        self.command_description = description;
    }

    /// Clear the recorded shortcut
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.shortcut = RecordedShortcut::new();
        self.conflict = None;
        self.is_recording = true;
        logging::log("SHORTCUT", "Shortcut cleared");
        cx.notify();
    }

    /// Handle save button press
    pub fn save(&mut self) {
        if self.shortcut.is_complete() && self.conflict.is_none() {
            logging::log(
                "SHORTCUT",
                &format!("Saving shortcut: {}", self.shortcut.to_config_string()),
            );
            // Set pending action for parent to poll
            self.pending_action = Some(RecorderAction::Save(self.shortcut.clone()));
            // Also call legacy callback if set
            if let Some(ref callback) = self.on_save {
                callback(self.shortcut.clone());
            }
        }
    }

    /// Handle cancel button press
    pub fn cancel(&mut self) {
        logging::log("SHORTCUT", "Shortcut recording cancelled");
        // Set pending action for parent to poll
        self.pending_action = Some(RecorderAction::Cancel);
        // Also call legacy callback if set
        if let Some(ref callback) = self.on_cancel {
            callback();
        }
    }

    /// Take the pending action (returns it and clears the field)
    pub fn take_pending_action(&mut self) -> Option<RecorderAction> {
        self.pending_action.take()
    }

    /// Handle a key down event
    pub fn handle_key_down(
        &mut self,
        key: &str,
        modifiers: gpui::Modifiers,
        cx: &mut Context<Self>,
    ) {
        if !self.is_recording {
            return;
        }

        // ALWAYS update current_modifiers for live display
        // This provides feedback even if on_modifiers_changed doesn't fire
        self.current_modifiers = modifiers;

        // Update shortcut modifiers
        self.shortcut.cmd = modifiers.platform;
        self.shortcut.ctrl = modifiers.control;
        self.shortcut.alt = modifiers.alt;
        self.shortcut.shift = modifiers.shift;

        // Check if this is a modifier-only key press
        let is_modifier_key = matches!(
            key.to_lowercase().as_str(),
            "shift"
                | "control"
                | "alt"
                | "meta"
                | "command"
                | "cmd"
                | "super"
                | "win"
                | "ctrl"
                | "opt"
                | "option"
        );

        if !is_modifier_key && !key.is_empty() {
            // Got a real key, record it
            self.shortcut.key = Some(key.to_uppercase());
            self.is_recording = false;

            logging::log(
                "SHORTCUT",
                &format!(
                    "Recorded shortcut: {} (config: {})",
                    self.shortcut.to_display_string(),
                    self.shortcut.to_config_string()
                ),
            );

            // Check for conflicts
            self.check_conflict();
        } else if is_modifier_key {
            // For modifier-only keypresses, log that we're showing live feedback
            logging::log(
                "SHORTCUT",
                &format!(
                    "Modifier key pressed (live feedback): key='{}' cmd={} ctrl={} alt={} shift={}",
                    key, modifiers.platform, modifiers.control, modifiers.alt, modifiers.shift
                ),
            );
        }

        cx.notify();
    }

    /// Handle escape key
    pub fn handle_escape(&mut self, cx: &mut Context<Self>) {
        if self.shortcut.is_empty() {
            // If nothing recorded, cancel
            self.cancel();
        } else {
            // Otherwise, clear the recording
            self.clear(cx);
        }
    }

    /// Check for shortcut conflicts
    fn check_conflict(&mut self) {
        if let Some(ref checker) = self.conflict_checker {
            self.conflict = checker(&self.shortcut);
            if let Some(ref conflict) = self.conflict {
                logging::log(
                    "SHORTCUT",
                    &format!(
                        "Conflict detected with '{}' (shortcut: {})",
                        conflict.command_name, conflict.shortcut
                    ),
                );
            }
        }
    }

    /// Update theme
    pub fn update_theme(&mut self, theme: Arc<Theme>) {
        self.colors = ShortcutRecorderColors::from_theme(&theme);
        self.theme = theme;
    }

    pub(super) fn overlay_appear_style(&self) -> OverlayAppearStyle {
        compute_overlay_appear_style(self.overlay_animation_started_at.elapsed())
    }

    pub(super) fn schedule_overlay_animation_tick_if_needed(
        &mut self,
        animation_complete: bool,
        cx: &mut Context<Self>,
    ) {
        if animation_complete || self.overlay_animation_tick_scheduled {
            return;
        }

        self.overlay_animation_tick_scheduled = true;
        cx.spawn(async move |this, cx| {
            gpui::Timer::after(Duration::from_millis(16)).await;
            let _ = cx.update(|cx| {
                let _ = this.update(cx, |recorder, cx| {
                    recorder.overlay_animation_tick_scheduled = false;
                    cx.notify();
                });
            });
        })
        .detach();
    }
}
