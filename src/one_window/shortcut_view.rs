//! Giving a script a shortcut happens in the main window: a view under the input says "Press
//! the keys for …", the keys you press show as keycaps, Enter saves them and Esc backs out.
//! A script's `hotkey()` asks through the same view: Enter hands the keys to the script and Esc
//! cancels its prompt.

use gpui::Context;
use sk_one_window::{EnterTarget, Event, Key, Row};

use super::learns::OwnView;
use crate::components::shortcut_recorder::RecordedShortcut;
use crate::prompt_completion::{PromptCompletionBinding, PromptOutcome};
use crate::protocol::SubmitValue;
use crate::ScriptListApp;

/// Who the recorded keys are for.
#[derive(Clone, Debug)]
pub(crate) enum KeysFor {
    /// A command's shortcut: Enter saves it to config.ts.
    Command,
    /// A script's `hotkey()`: Enter answers its prompt with the keys.
    Script(PromptCompletionBinding),
}

/// The keys pressed so far, and the words the view leads with.
#[derive(Clone, Debug)]
pub(crate) struct Recording {
    pub(crate) head: String,
    pub(crate) keys: RecordedShortcut,
    pub(crate) keys_for: KeysFor,
}

fn press_the_keys(name: &str) -> String {
    format!("Press the keys for {name}")
}

/// The keys a keystroke records, or `None` for a lone modifier.
fn recorded(keystroke: &gpui::Keystroke) -> RecordedShortcut {
    let m = &keystroke.modifiers;
    let key = keystroke.key.as_str();
    let modifier_only = matches!(
        key,
        "shift" | "control" | "alt" | "platform" | "function" | ""
    );
    RecordedShortcut {
        cmd: m.platform,
        ctrl: m.control,
        alt: m.alt,
        shift: m.shift,
        key: (!modifier_only).then(|| key.to_string()),
    }
}

impl ScriptListApp {
    /// Opens the shortcut view for the script or saved run called `name`.
    pub(crate) fn one_window_open_shortcut_view(&mut self, name: String, cx: &mut Context<Self>) {
        let script = self
            .scripts
            .iter()
            .find(|script| script.name == name)
            .or_else(|| {
                let saved = super::saved_scripts::saved_path();
                self.scripts.iter().find(|script| script.path == saved)
            })
            .cloned();
        let Some(script) = script else {
            tracing::warn!(target: "script_kit::one_window", "shortcut_target_missing");
            return;
        };
        self.show_shortcut_recorder(script.launcher_command_id(), name, cx);
    }

    /// The shortcut view for a command: its keys fill in as they're pressed.
    pub(crate) fn show_shortcut_recorder(
        &mut self,
        command_id: String,
        command_name: String,
        cx: &mut Context<Self>,
    ) {
        self.shortcut_recorder_state = Some(crate::ShortcutRecorderState {
            command_id,
            command_name: command_name.clone(),
        });
        self.one_window_record(press_the_keys(&command_name), KeysFor::Command, cx);
    }

    /// The shortcut view for a script's `hotkey()`, leading with the script's words.
    pub(crate) fn one_window_record_hotkey(
        &mut self,
        head: String,
        prompt: PromptCompletionBinding,
        cx: &mut Context<Self>,
    ) {
        self.one_window_record(head, KeysFor::Script(prompt), cx);
    }

    fn one_window_record(&mut self, head: String, keys_for: KeysFor, cx: &mut Context<Self>) {
        self.one_window_host.learns.shortcut = Some(Recording {
            head: head.clone(),
            keys: RecordedShortcut::new(),
            keys_for,
        });
        self.one_window_open_view(
            OwnView::Shortcut,
            head,
            Some(EnterTarget::new("shortcut:empty", "Press the keys first")),
            true,
            cx,
        );
    }

    /// The keys a script's `hotkey()` has recorded so far.
    pub(crate) fn one_window_hotkey_keys(&self) -> Option<&RecordedShortcut> {
        self.one_window_host
            .learns
            .shortcut
            .as_ref()
            .filter(|recording| matches!(recording.keys_for, KeysFor::Script(_)))
            .map(|recording| &recording.keys)
    }

    /// Leaves the shortcut view without saving.
    pub fn close_shortcut_recorder(&mut self, cx: &mut Context<Self>) {
        self.shortcut_recorder_state = None;
        self.one_window_host.learns.shortcut = None;
        if self.one_window_own_view() == Some(OwnView::Shortcut) {
            self.one_window_handle(Event::View(sk_one_window::ViewOp::Back), cx);
            self.one_window_republish_own(cx);
        }
        cx.notify();
    }

    /// Every key while the shortcut view is open is recorded, except Enter (save) and Esc
    /// (back out). Returns whether the key was the view's.
    pub(crate) fn one_window_shortcut_keystroke(
        &mut self,
        keystroke: &gpui::Keystroke,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(recording) = self.one_window_host.learns.shortcut.as_ref() else {
            return false;
        };
        if !matches!(self.current_view, crate::AppView::ScriptList)
            || self.one_window_own_view() != Some(OwnView::Shortcut)
        {
            return false;
        }
        let for_script = matches!(recording.keys_for, KeysFor::Script(_));
        let bare = !keystroke.modifiers.modified();
        match keystroke.key.as_str() {
            "escape" if bare => self.one_window_shortcut_key(Key::Escape, cx),
            "enter" if bare => self.one_window_shortcut_key(Key::Enter, cx),
            _ => {
                let keys = recorded(keystroke);
                let caps = keys.to_keycaps().concat();
                let conflict = (keys.is_complete() && !for_script)
                    .then(|| self.one_window_shortcut_conflict(&keys))
                    .flatten();
                let (id, label) = match (keys.is_complete(), conflict) {
                    (true, Some(owner)) => {
                        ("shortcut:conflict", format!("Already used by {owner}"))
                    }
                    (true, None) if for_script => ("shortcut:use", format!("Use {caps}")),
                    (true, None) => ("shortcut:save", format!("Save {caps}")),
                    (false, _) => ("shortcut:empty", "Press the keys first".to_string()),
                };
                if let Some(recording) = self.one_window_host.learns.shortcut.as_mut() {
                    recording.keys = keys;
                }
                self.one_window_handle(Event::ViewEnter(EnterTarget::new(id, label)), cx);
                self.one_window_republish_own(cx);
            }
        }
        true
    }

    /// The command that already answers to these keys, if another one does.
    fn one_window_shortcut_conflict(&self, keys: &RecordedShortcut) -> Option<String> {
        let command_id = &self.shortcut_recorder_state.as_ref()?.command_id;
        let shortcut = crate::shortcuts::Shortcut {
            key: keys.key.clone().unwrap_or_default().to_lowercase(),
            modifiers: crate::shortcuts::Modifiers {
                cmd: keys.cmd,
                ctrl: keys.ctrl,
                alt: keys.alt,
                shift: keys.shift,
            },
        };
        crate::hotkeys::shortcut_conflict_for_recording(command_id, &shortcut.to_canonical_string())
            .map(|conflict| conflict.command_name)
    }

    /// Enter saves the recorded keys (or hands them to the script); Esc goes back (or cancels
    /// the script's prompt).
    pub(super) fn one_window_shortcut_key(&mut self, key: Key, cx: &mut Context<Self>) {
        let Some(recording) = self.one_window_host.learns.shortcut.clone() else {
            return;
        };
        let outcome = match key {
            Key::Escape => PromptOutcome::Cancelled,
            Key::Enter if recording.keys.is_complete() => {
                PromptOutcome::Submitted(SubmitValue::Text(recording.keys.to_hotkey_info_json()))
            }
            _ => return,
        };
        match recording.keys_for {
            KeysFor::Command => match outcome {
                PromptOutcome::Submitted(_) => self.handle_shortcut_save(&recording.keys, cx),
                _ => self.close_shortcut_recorder(cx),
            },
            KeysFor::Script(prompt) => {
                if !prompt.observation().retired {
                    if let Err(error) = prompt.try_complete(outcome) {
                        self.notice_failed(error.to_string(), cx);
                    }
                }
                self.mark_main_data_changed();
                self.close_shortcut_recorder(cx);
            }
        }
    }

    /// The shortcut view's list: what to press as the header, the keys pressed so far as its
    /// one row.
    pub(crate) fn one_window_shortcut_rows(&self) -> Option<(String, Vec<Row>)> {
        let recording = self.one_window_host.learns.shortcut.as_ref()?;
        let caps = recording.keys.to_keycaps();
        let title = if caps.is_empty() {
            "Waiting for keys…".to_string()
        } else {
            caps.concat()
        };
        let sub = match recording.keys_for {
            KeysFor::Command => "Enter saves them · Esc backs out",
            KeysFor::Script(_) => "Enter uses them · Esc cancels",
        };
        let row = Row::new("shortcut:keys", title)
            .with_sub(sub)
            .with_icon("shortcut");
        Some((recording.head.clone(), vec![row]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lone_modifier_is_not_a_shortcut() {
        let shift = gpui::Keystroke::parse("shift").unwrap();
        assert!(!recorded(&shift).is_complete());
        let cmd_k = gpui::Keystroke::parse("cmd-shift-k").unwrap();
        let keys = recorded(&cmd_k);
        assert!(keys.is_complete() && keys.cmd && keys.shift);
    }
}
