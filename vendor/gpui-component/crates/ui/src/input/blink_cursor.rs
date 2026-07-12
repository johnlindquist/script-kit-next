use std::time::{Duration, Instant};

use gpui::{Context, Pixels, Task, px};

static FRAME_INTERVAL: Duration = Duration::from_millis(33);
static PULSE_PERIOD: Duration = Duration::from_millis(1100);
static PAUSE_DELAY: Duration = Duration::from_millis(300);
/// Fraction of each pulse cycle spent holding at an extreme: fully visible
/// at the start of the cycle, fully invisible at the midpoint.
const PULSE_DWELL: f32 = 0.12;

/// Ease-in-out caret pulse with a brief dwell at both extremes.
///
/// Keep in sync with the app-side copy in
/// src/components/text_input/render.rs (`caret_pulse_alpha`).
fn pulse_alpha(phase: f32) -> f32 {
    let fade = 0.5 - PULSE_DWELL;
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    if phase < PULSE_DWELL {
        1.0
    } else if phase < 0.5 {
        1.0 - smooth((phase - PULSE_DWELL) / fade)
    } else if phase < 0.5 + PULSE_DWELL {
        0.0
    } else {
        smooth((phase - 0.5 - PULSE_DWELL) / fade)
    }
}

// On Windows, Linux, we should use integer to avoid blurry cursor.
#[cfg(not(target_os = "macos"))]
pub(super) const CURSOR_WIDTH: Pixels = px(2.);
#[cfg(target_os = "macos")]
pub(super) const CURSOR_WIDTH: Pixels = px(2.0);

/// To manage the Input cursor blinking.
///
/// It pulses smoothly while active.
/// Every loop will notify the view to update the opacity, and Input will observe this update to touch repaint.
///
/// The input painter will check if this in visible state, then it will draw the cursor.
pub(crate) struct BlinkCursor {
    opacity: f32,
    paused: bool,
    epoch: usize,

    _task: Task<()>,
}

impl BlinkCursor {
    pub fn new() -> Self {
        Self {
            opacity: 1.0,
            paused: false,
            epoch: 0,
            _task: Task::ready(()),
        }
    }

    /// Start the blinking
    pub fn start(&mut self, cx: &mut Context<Self>) {
        let epoch = self.next_epoch();
        self.pulse(epoch, Instant::now(), cx);
    }

    pub fn stop(&mut self, cx: &mut Context<Self>) {
        self.epoch = 0;
        cx.notify();
    }

    fn next_epoch(&mut self) -> usize {
        self.epoch += 1;
        self.epoch
    }

    fn pulse(&mut self, epoch: usize, started_at: Instant, cx: &mut Context<Self>) {
        if self.paused || epoch != self.epoch {
            self.opacity = 1.0;
            return;
        }

        let phase = (started_at.elapsed().as_secs_f32() / PULSE_PERIOD.as_secs_f32()) % 1.0;
        self.opacity = pulse_alpha(phase);
        cx.notify();

        // Schedule the next pulse frame without changing the epoch.
        self._task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(FRAME_INTERVAL).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.pulse(epoch, started_at, cx));
            }
        });
    }

    pub fn visible(&self) -> bool {
        // Keep showing the cursor if paused
        self.paused || self.opacity >= 0.5
    }

    pub fn opacity(&self) -> f32 {
        if self.paused { 1.0 } else { self.opacity }
    }

    /// Pause the blinking, and delay 500ms to resume the blinking.
    pub fn pause(&mut self, cx: &mut Context<Self>) {
        self.paused = true;
        self.opacity = 1.0;
        cx.notify();

        // delay 500ms to start the blinking
        let epoch = self.next_epoch();
        self._task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(PAUSE_DELAY).await;

            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    this.paused = false;
                    this.pulse(epoch, Instant::now(), cx);
                });
            }
        });
    }
}
