// Main-hotkey gesture routing: classifier listener + surface morph handlers.

use std::time::Instant;

use gpui::AsyncApp;

use crate::hotkeys::gesture::GestureEvent;
use crate::hotkeys::gesture_routing::merge_launcher_query_into_day_page_content;
use crate::hotkeys::process_main_hotkey_physical_event;

/// True while the in-flight gesture began from the closed (window hidden) state,
/// i.e. its key-down emitted `ShowImmediate`. The classifier does not carry this
/// distinction, but routing needs it: the opening tap's deferred `Tap` must NOT
/// toggle to Day Page, and `HoldStart` only acts for hold-from-closed
/// (hold-while-open is intentionally dead until a later task).
static MAIN_GESTURE_BEGAN_CLOSED: AtomicBool = AtomicBool::new(false);

/// Start the async listener that classifies main-hotkey key-down/key-up pairs.
///
/// Single-task design: ONE loop owns every classifier mutation. It waits for
/// either the next physical key event or the classifier's next deadline, then
/// drains all queued physical events (their timestamps come from injection
/// time) BEFORE applying time-based transitions. A separate poll task would
/// race the listener — e.g. firing `HoldStart` from wall-clock time while a
/// sub-250ms key-up was still queued behind a busy main thread.
pub(crate) fn spawn_main_hotkey_gesture_listener(
    cx: &mut App,
    app_entity: Entity<ScriptListApp>,
    window: WindowHandle<Root>,
) {
    let executor = cx.background_executor().clone();
    cx.spawn(async move |cx: &mut AsyncApp| {
        logging::log("HOTKEY", "Main hotkey gesture listener started");
        let receiver = hotkeys::main_hotkey_physical_channel().1.clone();
        loop {
            // Wait for the next physical event, or wake at the classifier
            // deadline (hold threshold / double-tap window expiry).
            let waited: Option<Result<hotkeys::MainHotkeyPhysicalEvent, async_channel::RecvError>> =
                match hotkeys::main_gesture_next_deadline() {
                    Some(deadline) => {
                        let wait = deadline.saturating_duration_since(Instant::now());
                        let recv = async { Some(receiver.recv().await) };
                        let timer = async {
                            executor.timer(wait).await;
                            None
                        };
                        smol::future::or(recv, timer).await
                    }
                    None => Some(receiver.recv().await),
                };

            let mut events = Vec::new();
            let mut correlation_id = None;
            match waited {
                Some(Ok(physical)) => {
                    correlation_id = Some(physical.correlation_id.clone());
                    events.extend(process_main_hotkey_physical_event(physical));
                    // Drain everything already queued so ordering follows the
                    // injected timestamps, not poll wall-clock time.
                    while let Ok(more) = receiver.try_recv() {
                        events.extend(process_main_hotkey_physical_event(more));
                    }
                }
                Some(Err(_)) => break,
                None => {}
            }
            while let Some(gesture) = hotkeys::poll_main_gesture_classifier(Instant::now()) {
                events.push(gesture);
            }
            if events.is_empty() {
                continue;
            }

            let _guard = correlation_id.map(logging::set_correlation_id);
            let app_entity_inner = app_entity.clone();
            let window_inner = window;
            let _ = cx.update(move |cx: &mut App| {
                for gesture in events {
                    dispatch_main_gesture_event(
                        gesture,
                        window_inner,
                        app_entity_inner.clone(),
                        cx,
                    );
                }
            });
        }
        logging::log(
            "HOTKEY",
            "Main hotkey gesture listener exiting (channel closed)",
        );
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
            MAIN_GESTURE_BEGAN_CLOSED.store(true, Ordering::SeqCst);
            if !script_kit_gpui::is_main_window_visible() {
                show_main_window_helper(window, app_entity, cx);
            }
        }
        GestureEvent::Tap => {
            let began_closed = MAIN_GESTURE_BEGAN_CLOSED.swap(false, Ordering::SeqCst);
            if began_closed {
                // The tap that opened the window resolves to the launcher
                // steady state — it must not also toggle to Day Page.
                logging::log(
                    "GESTURE",
                    "Tap — opening tap resolved (launcher steady state)",
                );
                return;
            }
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
            MAIN_GESTURE_BEGAN_CLOSED.store(false, Ordering::SeqCst);
            if !script_kit_gpui::is_main_window_visible() {
                show_main_window_helper(window, app_entity.clone(), cx);
            }
            app_entity.update(cx, |view, cx| {
                view.mark_opened_directly("gesture_double_tap");
                view.open_tab_ai_agent_chat_with_entry_intent(None, cx);
            });
        }
        GestureEvent::HoldStart => {
            let began_closed = MAIN_GESTURE_BEGAN_CLOSED.load(Ordering::SeqCst);
            if !began_closed {
                // Hold-while-open semantics are intentionally dead (T8 scope).
                logging::log("GESTURE", "HoldStart — ignored (hold-while-open is dead)");
                return;
            }
            logging::log("GESTURE", "HoldStart — show Day Page");
            if !script_kit_gpui::is_main_window_visible() {
                show_main_window_helper(window, app_entity.clone(), cx);
            }
            let _ = window.update(cx, |_root, window, cx| {
                app_entity.update(cx, |view, cx| {
                    view.show_day_page_view(window, cx);
                });
            });
        }
        GestureEvent::HoldEnd => {
            // Hold release is inert: the Day Page is already showing.
            // Dictation is owned by the dedicated dictation shortcut/window.
            MAIN_GESTURE_BEGAN_CLOSED.store(false, Ordering::SeqCst);
        }
    }
}

impl ScriptListApp {
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
            editor.load_value_with_cursor_at_end(merged.clone(), window, cx);
        });
        self.session.apply_editor_content(&merged);
        self.refresh_fragment_open_targets(&merged);
        self.focus_editor(window, cx);
        self.sync_footer(window, cx);
        cx.notify();
    }
}

#[cfg(test)]
mod gesture_routing_tests {
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
