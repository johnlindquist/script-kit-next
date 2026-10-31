// Main-hotkey gesture routing: classifier listener + surface morph handlers.

use std::time::Instant;

use gpui::AsyncApp;

use crate::hotkeys::gesture::GestureEvent;
use crate::hotkeys::process_main_hotkey_physical_event;

// Only a key-up commits a tap. A hold must never focus Kit on the way to the microphone.
static MAIN_GESTURE_TODAY: AtomicBool = AtomicBool::new(false);
static MAIN_GESTURE_OUTSIDE: AtomicBool = AtomicBool::new(false);
static MAIN_GESTURE_TAP_HANDLED: AtomicBool = AtomicBool::new(false);

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
                        // Floor the wait: if a classifier state ever reports
                        // an already-elapsed deadline that poll can't clear,
                        // an unfloored timer spins this loop at 100% CPU.
                        let wait = deadline
                            .saturating_duration_since(Instant::now())
                            .max(std::time::Duration::from_millis(4));
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
                    rearm_main_gesture_if_window_unfocused(&physical, &app_entity, cx);
                    events.extend(process_main_hotkey_physical_event(physical));
                    // Drain everything already queued so ordering follows the
                    // injected timestamps, not poll wall-clock time.
                    while let Ok(more) = receiver.try_recv() {
                        rearm_main_gesture_if_window_unfocused(&more, &app_entity, cx);
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
            cx.update(move |cx: &mut App| {
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

/// A press while the window is on screen but another app holds the keyboard
/// (it survived a blur: pinned, a waiting popup, dictation) brings the keyboard
/// back instead of hiding the window.
fn rearm_main_gesture_if_window_unfocused(
    physical: &hotkeys::MainHotkeyPhysicalEvent,
    app_entity: &Entity<ScriptListApp>,
    cx: &mut AsyncApp,
) {
    if physical.phase != hotkeys::MainHotkeyPhase::KeyDown {
        return;
    }
    let unfocused = cx.update(|cx| app_entity.read(cx).main_window_shown_but_unfocused());
    if unfocused && hotkeys::rearm_main_gesture_for_unfocused_window() {
        logging::log(
            "GESTURE",
            "Main window shown but not key — this press refocuses it",
        );
    }
}

fn dispatch_main_gesture_event(
    event: GestureEvent,
    window: WindowHandle<Root>,
    app_entity: Entity<ScriptListApp>,
    cx: &mut App,
) {
    match event {
        GestureEvent::BeginOutside | GestureEvent::BeginInside => {
            let outside =
                !script_kit_gpui::is_main_window_visible() || !platform::is_main_window_focused();
            MAIN_GESTURE_OUTSIDE.store(outside, Ordering::SeqCst);
            MAIN_GESTURE_TAP_HANDLED.store(false, Ordering::SeqCst);
        }
        GestureEvent::TapPreview => {
            let outside = MAIN_GESTURE_OUTSIDE.load(Ordering::SeqCst);
            let handled = if app_entity.read(cx).cursor_dictation_pending() {
                true
            } else if outside {
                let launcher = app_entity.update(cx, |app, cx| app.prepare_shortcut_launcher(cx));
                if !launcher {
                    app_entity.update(cx, |app, cx| {
                        app.dispatch_window_event(
                            crate::window_orchestrator::WindowEvent::ShowMain {
                                activate_app: true,
                            },
                            cx,
                        )
                    });
                    MAIN_GESTURE_TAP_HANDLED.store(true, Ordering::SeqCst);
                    return;
                }
                if script_kit_gpui::is_main_window_visible() {
                    let _ = window.update(cx, |_, win, cx| {
                        app_entity.update(cx, |app, cx| app.refocus_shown_main_window(win, cx))
                    });
                } else {
                    show_main_window_helper(window, app_entity.clone(), cx);
                }
                true
            } else {
                window
                    .update(cx, |_, win, cx| {
                        app_entity.update(cx, |app, cx| {
                            app.try_handle_main_hotkey_tap_preview(win, cx)
                        })
                    })
                    .unwrap_or(false)
            };
            MAIN_GESTURE_TAP_HANDLED.store(handled, Ordering::SeqCst);
        }
        GestureEvent::Tap => {
            if !MAIN_GESTURE_TAP_HANDLED.swap(false, Ordering::SeqCst)
                && script_kit_gpui::is_main_window_visible()
            {
                let _ = window.update(cx, |_, win, cx| {
                    app_entity.update(cx, |app, cx| app.handle_main_hotkey_tap(win, cx))
                });
            }
        }
        GestureEvent::DoubleTap => {
            MAIN_GESTURE_TODAY.store(false, Ordering::SeqCst);
            if app_entity.read(cx).cursor_dictation_pending()
                || app_entity.read(cx).prompt_receiver.is_some()
            {
                return;
            }
            if !script_kit_gpui::is_main_window_visible() {
                show_main_window_helper(window, app_entity.clone(), cx);
            }
            let _ = window.update(cx, |_, win, cx| {
                app_entity.update(cx, |app, cx| {
                    if let Err(error) = app.open_note_in_editor(
                        crate::notes::NoteTarget::Today,
                        crate::DayPageBack::Leave,
                        win,
                        cx,
                    ) {
                        app.notice_failed(format!("Couldn't open Today: {error}"), cx);
                    } else {
                        MAIN_GESTURE_TODAY.store(true, Ordering::SeqCst);
                    }
                })
            });
        }
        GestureEvent::HoldStart | GestureEvent::TodayHoldStart => {
            let today = event == GestureEvent::TodayHoldStart;
            if today && !MAIN_GESTURE_TODAY.load(Ordering::SeqCst) {
                return;
            }
            let external = !today && MAIN_GESTURE_OUTSIDE.load(Ordering::SeqCst);
            let _ = window.update(cx, |_, win, cx| {
                app_entity.update(cx, |app, cx| {
                    app.start_cursor_hold(external, today, win, cx)
                })
            });
        }
        GestureEvent::HoldEnd => app_entity.update(cx, |app, cx| app.finish_cursor_hold(cx)),
    }
}

impl ScriptListApp {
    /// Fast path for taps that are safe to apply before the double-tap window
    /// expires. Hiding resets state (`close_and_reset_window` clears the
    /// filter), so it only previews where there is nothing to lose: an empty
    /// launcher, or the Day Page (whose dirty content is flushed by
    /// `reset_to_script_list`). Taps over typed filter text or prompts stay
    /// on the delayed `Tap` path so a double-tap cannot destroy input.
    pub(crate) fn try_handle_main_hotkey_tap_preview(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match &self.current_view {
            AppView::ScriptList if self.filter_text.trim().is_empty() => {
                self.handle_main_hotkey_tap(window, cx);
                true
            }
            AppView::DayPage { .. } => {
                self.handle_main_hotkey_tap(window, cx);
                true
            }
            _ => false,
        }
    }

    /// True when the window is on screen but not the key window, so typing
    /// goes to another app.
    pub(crate) fn main_window_shown_but_unfocused(&self) -> bool {
        !self.main_services.host_policy().is_hidden()
            && script_kit_gpui::is_main_window_visible()
            && !platform::is_main_window_focused()
    }

    /// Give the keyboard back to the window that is already showing: make it
    /// key without replaying the entry fade, and put the caret back in the
    /// current view if nothing inside the window holds focus.
    pub(crate) fn refocus_shown_main_window(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        logging::log(
            "FOCUS",
            &format!(
                "Main hotkey refocusing the shown window (view={})",
                self.current_view.app_view_variant()
            ),
        );
        window.activate_window();
        self.focus_main_filter(window, cx);
        cx.notify();
    }

    /// Tap-while-open: hide the launcher (the universal hotkey-toggle
    /// convention). Day Page entry lives on the `,` first-character trigger
    /// and hold-from-closed instead. `close_and_reset_window` funnels through
    /// `reset_to_script_list`, which flushes a dirty Day Page before the
    /// entity drops.
    pub(crate) fn handle_main_hotkey_tap(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.close_and_reset_window(cx);
    }
}
