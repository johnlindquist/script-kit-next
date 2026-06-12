// Main-hotkey gesture routing: classifier listener + surface morph handlers.

use std::sync::atomic::AtomicU64;
use std::time::Instant;

use gpui::AsyncApp;

use crate::hotkeys::gesture::GestureEvent;
use crate::hotkeys::gesture_routing::merge_launcher_query_into_day_page_content;
use crate::hotkeys::process_main_hotkey_physical_event;

static MAIN_GESTURE_POLL_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Start the async listener that classifies main-hotkey key-down/key-up pairs.
pub(crate) fn spawn_main_hotkey_gesture_listener(
    cx: &mut App,
    app_entity: Entity<ScriptListApp>,
    window: WindowHandle<Root>,
) {
    cx.spawn(async move |cx: &mut AsyncApp| {
        logging::log("HOTKEY", "Main hotkey gesture listener started");
        schedule_main_gesture_poll(cx, app_entity.clone(), window);
        while let Ok(physical) = hotkeys::main_hotkey_physical_channel().1.recv().await {
            let correlation_id = physical.correlation_id.clone();
            let _guard = logging::set_correlation_id(correlation_id);
            let events = process_main_hotkey_physical_event(physical);
            let app_entity_inner = app_entity.clone();
            let window_inner = window;
            let _ = cx.update(move |cx: &mut App| {
                for gesture in events {
                    dispatch_main_gesture_event(gesture, window_inner, app_entity_inner.clone(), cx);
                }
            });
            schedule_main_gesture_poll(cx, app_entity.clone(), window);
        }
        logging::log("HOTKEY", "Main hotkey gesture listener exiting (channel closed)");
    })
    .detach();
}

fn schedule_main_gesture_poll(
    cx: &mut AsyncApp,
    app_entity: Entity<ScriptListApp>,
    window: WindowHandle<Root>,
) {
    let Some(deadline) = hotkeys::main_gesture_next_deadline() else {
        return;
    };
    let wait = deadline.saturating_duration_since(Instant::now());
    let generation = MAIN_GESTURE_POLL_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    cx.spawn(async move |cx: &mut AsyncApp| {
        cx.background_executor().timer(wait).await;
        if MAIN_GESTURE_POLL_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }
        let mut polled_any = false;
        while let Some(gesture) = hotkeys::poll_main_gesture_classifier(Instant::now()) {
            polled_any = true;
            let app_entity_inner = app_entity.clone();
            let _ = cx.update(move |cx: &mut App| {
                dispatch_main_gesture_event(gesture, window, app_entity_inner.clone(), cx);
            });
        }
        if polled_any {
            schedule_main_gesture_poll(cx, app_entity, window);
        } else if hotkeys::main_gesture_next_deadline().is_some() {
            schedule_main_gesture_poll(cx, app_entity, window);
        }
    })
    .detach();
}

fn dispatch_main_gesture_event(
    event: GestureEvent,
    window: WindowHandle<Root>,
    app_entity: Entity<ScriptListApp>,
    cx: &mut App,
) {
    match event {
        GestureEvent::ShowImmediate => {
            logging::log("GESTURE", "ShowImmediate — key-down show");
            if !script_kit_gpui::is_main_window_visible() {
                show_main_window_helper(window, app_entity, cx);
            }
        }
        GestureEvent::Tap => {
            logging::log("GESTURE", "Tap — toggle launcher ↔ Day Page");
            if !script_kit_gpui::is_main_window_visible() {
                return;
            }
            let _ = window.update(cx, |_root, window, cx| {
                app_entity.update(cx, |view, cx| {
                    view.handle_main_hotkey_tap(window, cx);
                });
            });
        }
        GestureEvent::DoubleTap => {
            logging::log("GESTURE", "DoubleTap — Agent Chat entry intent");
            if !script_kit_gpui::is_main_window_visible() {
                show_main_window_helper(window, app_entity.clone(), cx);
            }
            app_entity.update(cx, |view, cx| {
                view.mark_opened_directly("gesture_double_tap");
                view.open_tab_ai_agent_chat_with_entry_intent(None, cx);
            });
        }
        GestureEvent::HoldStart => {
            logging::log("GESTURE", "HoldStart — Day Page + push-to-talk dictation");
            if !script_kit_gpui::is_main_window_visible() {
                show_main_window_helper(window, app_entity.clone(), cx);
            }
            let _ = window.update(cx, |_root, window, cx| {
                app_entity.update(cx, |view, cx| {
                    view.handle_main_hotkey_hold_start(window, cx);
                });
            });
        }
        GestureEvent::HoldEnd => {
            logging::log("GESTURE", "HoldEnd — stop dictation and commit to day page");
            if !script_kit_gpui::is_main_window_visible() {
                return;
            }
            let _ = window.update(cx, |_root, _window, cx| {
                app_entity.update(cx, |view, cx| {
                    view.handle_main_hotkey_hold_end(cx);
                });
            });
        }
    }
}

impl ScriptListApp {
    /// Hold-from-closed: show Day Page and start inline push-to-talk capture.
    pub(crate) fn handle_main_hotkey_hold_start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_day_page_view(window, cx);

        let readiness = self.evaluate_day_page_dictation_readiness();
        if !readiness.ready {
            self.set_day_page_dictation_chrome(
                DayPageDictationChrome::Unavailable {
                    message: readiness.message,
                },
                cx,
            );
            if readiness.open_setup {
                self.open_dictation_model_prompt(cx);
            }
            return;
        }

        match crate::dictation::toggle_dictation(crate::dictation::DictationTarget::DayPage) {
            Ok(crate::dictation::DictationToggleOutcome::Started) => {
                self.set_day_page_dictation_chrome(
                    DayPageDictationChrome::Listening {
                        display_bars: crate::dictation::silent_bars(),
                    },
                    cx,
                );
            }
            Ok(crate::dictation::DictationToggleOutcome::Stopped(_)) => {}
            Err(error) => {
                tracing::error!(category = "DICTATION", error = %error, "Hold dictation failed to start");
                self.set_day_page_dictation_chrome(
                    DayPageDictationChrome::Unavailable {
                        message: format!("Dictation unavailable: {error}"),
                    },
                    cx,
                );
            }
        }
    }

    /// Hold release: stop capture, transcribe, and deliver to today's page.
    pub(crate) fn handle_main_hotkey_hold_end(&mut self, cx: &mut Context<Self>) {
        let active_target = crate::dictation::get_dictation_target();
        if !matches!(
            active_target,
            Some(crate::dictation::DictationTarget::DayPage)
        ) {
            return;
        }

        self.set_day_page_dictation_chrome(DayPageDictationChrome::Transcribing, cx);
        self.request_active_dictation_stop(
            crate::dictation::DictationStopReason::Hotkey,
            crate::dictation::DictationTarget::DayPage,
            true,
            cx,
        );
    }

    fn evaluate_day_page_dictation_readiness(&self) -> DayPageDictationReadiness {
        if !crate::dictation::is_parakeet_model_available() {
            return DayPageDictationReadiness {
                ready: false,
                message: "Dictation model not ready — download it in Dictation Setup.".to_string(),
                open_setup: true,
            };
        }

        let permission = crate::dictation::microphone_permission_status();
        let prefs = crate::config::load_user_preferences();
        let config = crate::config::load_config();
        let devices = match permission {
            crate::dictation::DictationMicrophonePermissionStatus::Granted
            | crate::dictation::DictationMicrophonePermissionStatus::Unknown => {
                crate::dictation::list_input_devices().map_err(|error| error.to_string())
            }
            crate::dictation::DictationMicrophonePermissionStatus::Denied
            | crate::dictation::DictationMicrophonePermissionStatus::NotDetermined => {
                Ok(Vec::new())
            }
        };
        let setup_state = crate::dictation::build_dictation_setup_state(
            crate::dictation::DictationModelStatus::Available,
            permission,
            devices,
            prefs.dictation.selected_device_id.as_deref(),
            config.get_dictation_hotkey().as_ref(),
            config.is_dictation_hotkey_enabled(),
        );

        if setup_state.ready {
            return DayPageDictationReadiness {
                ready: true,
                message: String::new(),
                open_setup: false,
            };
        }

        let message = match setup_state.microphone_status {
            crate::dictation::DictationMicrophoneStatus::PermissionNeeded(_) => {
                "Microphone access is needed for hold-to-talk dictation.".to_string()
            }
            crate::dictation::DictationMicrophoneStatus::SavedDeviceMissing { .. } => {
                "Saved microphone unavailable — choose a device in Dictation Setup.".to_string()
            }
            crate::dictation::DictationMicrophoneStatus::NoDevices => {
                "No microphone found — connect an input device to dictate.".to_string()
            }
            crate::dictation::DictationMicrophoneStatus::EnumerationFailed(error) => {
                format!("Could not list microphones: {error}")
            }
            _ => "Dictation is not ready — finish setup to dictate.".to_string(),
        };

        DayPageDictationReadiness {
            ready: false,
            message,
            open_setup: true,
        }
    }

    pub(crate) fn set_day_page_dictation_chrome(
        &mut self,
        chrome: DayPageDictationChrome,
        cx: &mut Context<Self>,
    ) {
        if let AppView::DayPage { entity } = &self.current_view {
            let entity = entity.clone();
            entity.update(cx, |view, cx| view.set_dictation_chrome(chrome, cx));
        }
    }

    pub(crate) fn clear_day_page_dictation_chrome(&mut self, cx: &mut Context<Self>) {
        if let AppView::DayPage { entity } = &self.current_view {
            let entity = entity.clone();
            entity.update(cx, |view, cx| view.clear_dictation_chrome(cx));
        }
    }

    /// Tap-while-open: toggle launcher ↔ Day Page; carry launcher query on entry.
    pub(crate) fn handle_main_hotkey_tap(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match &self.current_view {
            AppView::ScriptList => {
                let carry = self.filter_text.trim().to_string();
                if !carry.is_empty() {
                    self.set_filter_text_immediate(String::new(), window, cx);
                }
                self.show_day_page_view(window, cx);
                if !carry.is_empty() {
                    if let AppView::DayPage { entity } = &self.current_view {
                        let entity = entity.clone();
                        entity.update(cx, |view, cx| {
                            view.insert_capture_start(carry, window, cx);
                        });
                    }
                }
            }
            AppView::DayPage { .. } => {
                self.reset_to_script_list(cx);
                self.request_script_list_main_filter_focus(cx);
                self.sync_main_footer_popup(window, cx);
                cx.notify();
            }
            _ => {
                self.reset_to_script_list(cx);
                self.request_script_list_main_filter_focus(cx);
                self.sync_main_footer_popup(window, cx);
                cx.notify();
            }
        }
    }
}

impl DayPageView {
    /// Insert launcher carry-over text as the start of a new capture line.
    pub(crate) fn insert_capture_start(
        &mut self,
        query: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing = self.notes_editor.read(cx).content(cx);
        let merged = merge_launcher_query_into_day_page_content(&existing, &query);
        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value(merged.clone(), window, cx);
        });
        self.session.apply_editor_content(&merged);
        self.focus_editor(window, cx);
        self.sync_footer(window, cx);
        cx.notify();
    }
}

struct DayPageDictationReadiness {
    ready: bool,
    message: String,
    open_setup: bool,
}

#[cfg(test)]
mod tests {
    use crate::hotkeys::gesture_routing::merge_launcher_query_into_day_page_content;

    #[test]
    fn tap_toggle_carry_over_contract() {
        assert_eq!(
            merge_launcher_query_into_day_page_content("", "capture me"),
            "capture me"
        );
        assert_eq!(
            merge_launcher_query_into_day_page_content("09:00 line", "next"),
            "09:00 line\nnext"
        );
    }
}
