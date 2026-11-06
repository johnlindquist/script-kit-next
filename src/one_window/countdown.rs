//! The one app-local countdown. A row carries the timer's generation; neither a late
//! wake-up nor an old Undo can stop a different timer. Never opens an OS timer or a window.
use crate::{AppView, ScriptListApp};
use gpui::Context;
use sk_one_window::countdown::{self, Timer};
use std::time::Duration;

impl ScriptListApp {
    fn forget_countdown_undo(&mut self) {
        if let Some(id) = self.one_window_host.timer_undo.take() {
            self.forget_one_window_undo(id);
        }
    }
    fn refresh_countdown_rows(&mut self, cx: &mut Context<Self>) {
        if matches!(self.current_view, AppView::ScriptList)
            && !self.one_window.in_view()
            && countdown::request(&self.filter_text).is_some()
        {
            self.one_window_republish("countdown", cx);
        }
    }
    fn watch_countdown(&mut self, timer: Timer, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let mut delay = 1000;
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(delay))
                    .await;
                let Ok(Some(next)) = this.update(cx, |app, cx| {
                    let current = app.one_window_host.timers.active()?;
                    if current.id != timer.id {
                        return None;
                    }
                    let now = app.one_window_now(cx);
                    if app.one_window_host.timers.finish(timer.id, now).is_some() {
                        app.forget_countdown_undo();
                        app.one_window_notice(
                            "timer",
                            format!(
                                "Timer finished · {}",
                                countdown::duration_words(timer.seconds)
                            ),
                            cx,
                        );
                        app.refresh_countdown_rows(cx);
                        return None;
                    }
                    app.refresh_countdown_rows(cx);
                    Some(current.due.saturating_sub(now).clamp(1, 1000))
                }) else {
                    break;
                };
                delay = next;
            }
        })
        .detach();
    }
    fn arm_countdown(&mut self, seconds: u64, remaining: u64, cx: &mut Context<Self>) {
        let now = self.one_window_now(cx);
        let Some(timer) = self.one_window_host.timers.arm(seconds, remaining, now) else {
            self.one_window_notice("timer", "A timer is already running", cx);
            return;
        };
        self.forget_countdown_undo();
        let kept = self.record_one_window_undo(
            "timer",
            format!("Started timer · {}", countdown::duration_words(seconds)),
            Box::new(move |app, cx| {
                if app.one_window_host.timers.stop(timer.id).is_some() {
                    app.one_window_host.timer_undo = None;
                    app.refresh_countdown_rows(cx);
                }
            }),
            cx,
        );
        self.one_window_host.timer_undo = kept;
        self.watch_countdown(timer, cx);
        self.refresh_countdown_rows(cx);
    }
    pub(super) fn run_countdown_row(&mut self, key: &str, cx: &mut Context<Self>) -> bool {
        let Some(key) = key.strip_prefix("quick-timer:") else {
            return false;
        };
        if key == "idle" {
            self.one_window_notice("timer", "No timer is running", cx);
            return true;
        }
        let Some((verb, raw)) = key.split_once(':') else {
            return false;
        };
        let Ok(value) = raw.parse::<u64>() else {
            return false;
        };
        match verb {
            "start" if (1..=countdown::MAX_SECONDS).contains(&value) => {
                self.arm_countdown(value, value * 1000, cx)
            }
            "status" | "stop" => {
                let now = self.one_window_now(cx);
                let Some(timer) = self
                    .one_window_host
                    .timers
                    .active()
                    .filter(|t| t.id == value && t.due > now)
                else {
                    self.one_window_notice("timer", "That timer is no longer running", cx);
                    return true;
                };
                let remaining = timer.due - now;
                if verb == "status" {
                    self.one_window_open_local_answer(
                        "Time left",
                        countdown::remaining_words(remaining),
                        "Remaining when checked",
                        sk_one_window::form::Source::Known("timer".into()),
                        cx,
                    );
                } else if self.one_window_host.timers.stop(timer.id).is_some() {
                    self.forget_countdown_undo();
                    let kept = self.record_one_window_undo(
                        "timer",
                        "Stopped timer",
                        Box::new(move |app, cx| {
                            if app.one_window_host.timers.active().is_none() {
                                app.arm_countdown(timer.seconds, remaining, cx);
                            }
                        }),
                        cx,
                    );
                    self.one_window_host.timer_undo = kept;
                    self.refresh_countdown_rows(cx);
                }
            }
            _ => return false,
        }
        true
    }
}
