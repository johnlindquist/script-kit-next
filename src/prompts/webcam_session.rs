//! The photo session behind the webcam view: the live preview, an optional countdown so you can
//! pose, and the review of the saved photo with its copy actions.
//!
//! Pure state with no clock: the app owns the one-second timer and hands every tick back with the
//! generation it started, so a cancelled, finished, or restarted countdown can never fire a shot.

use std::path::{Path, PathBuf};

/// Countdown lengths offered from ⌘K.
pub const COUNTDOWN_SHORT_SECONDS: u8 = 3;
pub const COUNTDOWN_LONG_SECONDS: u8 = 10;

/// Copy Path's shortcut, shown in the footer and on its ⌘K row while a photo is reviewed.
pub const COPY_PATH_SHORTCUT: &str = "⌘⇧C";

/// Action ids shared by the ⌘K list, the footer, and `triggerAction` (host `webcamPrompt`).
pub mod action_id {
    pub const CAPTURE: &str = "capture";
    pub const COUNTDOWN_SHORT: &str = "countdown_3";
    pub const COUNTDOWN_LONG: &str = "countdown_10";
    pub const CANCEL_COUNTDOWN: &str = "cancel_countdown";
    pub const COPY_IMAGE: &str = "copy_image";
    pub const COPY_PATH: &str = "copy_path";
    pub const REVEAL: &str = "reveal";
    pub const OPEN: &str = "open";
    pub const RETAKE: &str = "retake";
    pub const CLOSE: &str = "close";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebcamPhase {
    /// The camera preview; ↵ takes the photo.
    Live,
    /// Counting down to the shot; `remaining` is the number on screen.
    Countdown { remaining: u8 },
    /// The photo was saved to `path` and is shown frozen until you copy it, retake, or leave.
    Review { path: PathBuf },
}

/// What one second of the countdown did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownTick {
    /// The countdown this timer started is gone (cancelled, shot, or restarted): stop the timer.
    Stale,
    /// Still counting; the number now on screen.
    Remaining(u8),
    /// The countdown reached zero: take the photo now.
    Shoot,
}

/// What Escape means in the current phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebcamEscape {
    CancelledCountdown,
    Retake,
    /// Nothing to step back from; the view's normal dismissal applies.
    Leave,
}

/// One row of the ⌘K list for the current phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebcamActionRow {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub shortcut: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WebcamSession {
    phase: WebcamPhase,
    generation: u64,
}

impl Default for WebcamPhase {
    fn default() -> Self {
        Self::Live
    }
}

impl WebcamSession {
    pub fn phase(&self) -> &WebcamPhase {
        &self.phase
    }

    /// The saved photo while it is being reviewed.
    pub fn photo_path(&self) -> Option<&Path> {
        match &self.phase {
            WebcamPhase::Review { path } => Some(path),
            _ => None,
        }
    }

    /// Starts a countdown from the live preview. Returns the generation the timer must hand back
    /// with every tick, or `None` when a countdown can't start here.
    pub fn start_countdown(&mut self, seconds: u8) -> Option<u64> {
        if seconds == 0 || self.phase != WebcamPhase::Live {
            return None;
        }
        self.generation += 1;
        self.phase = WebcamPhase::Countdown { remaining: seconds };
        Some(self.generation)
    }

    /// One second passed for the countdown started at `generation`.
    pub fn tick(&mut self, generation: u64) -> CountdownTick {
        if generation != self.generation {
            return CountdownTick::Stale;
        }
        match self.phase {
            WebcamPhase::Countdown { remaining } if remaining > 1 => {
                self.phase = WebcamPhase::Countdown {
                    remaining: remaining - 1,
                };
                CountdownTick::Remaining(remaining - 1)
            }
            WebcamPhase::Countdown { .. } => {
                self.end_countdown();
                CountdownTick::Shoot
            }
            _ => CountdownTick::Stale,
        }
    }

    /// Ends a running countdown without a shot (the caller may shoot at once). Returns whether
    /// one was running.
    pub fn end_countdown(&mut self) -> bool {
        if !matches!(self.phase, WebcamPhase::Countdown { .. }) {
            return false;
        }
        self.generation += 1;
        self.phase = WebcamPhase::Live;
        true
    }

    /// The photo was saved; show it for review.
    pub fn captured(&mut self, path: PathBuf) {
        self.generation += 1;
        self.phase = WebcamPhase::Review { path };
    }

    /// Back to the live preview from a review. Returns whether there was one.
    pub fn retake(&mut self) -> bool {
        if !matches!(self.phase, WebcamPhase::Review { .. }) {
            return false;
        }
        self.generation += 1;
        self.phase = WebcamPhase::Live;
        true
    }

    /// Escape steps back one phase: a countdown is cancelled, a review goes back to the camera.
    pub fn escape(&mut self) -> WebcamEscape {
        if self.end_countdown() {
            WebcamEscape::CancelledCountdown
        } else if self.retake() {
            WebcamEscape::Retake
        } else {
            WebcamEscape::Leave
        }
    }

    /// The ↵ label in the footer.
    pub fn primary_label(&self) -> &'static str {
        match self.phase {
            WebcamPhase::Live => "Capture Photo",
            WebcamPhase::Countdown { .. } => "Capture Now",
            WebcamPhase::Review { .. } => "Copy Image",
        }
    }

    /// The action ↵ runs.
    pub fn primary_action_id(&self) -> &'static str {
        match self.phase {
            WebcamPhase::Live | WebcamPhase::Countdown { .. } => action_id::CAPTURE,
            WebcamPhase::Review { .. } => action_id::COPY_IMAGE,
        }
    }

    /// The ⌘K rows for the current phase, ↵'s action first.
    pub fn action_rows(&self) -> Vec<WebcamActionRow> {
        let row = |id, title, description, shortcut| WebcamActionRow {
            id,
            title,
            description,
            shortcut,
        };
        match self.phase {
            WebcamPhase::Live => vec![
                row(action_id::CAPTURE, "Capture Photo", "Take a photo now", Some("↵")),
                row(
                    action_id::COUNTDOWN_SHORT,
                    "Capture in 3 Seconds",
                    "Count down so you can pose",
                    None,
                ),
                row(
                    action_id::COUNTDOWN_LONG,
                    "Capture in 10 Seconds",
                    "Count down so you can step back and pose",
                    None,
                ),
                row(action_id::CLOSE, "Close", "Turn off the camera", Some("⎋")),
            ],
            WebcamPhase::Countdown { .. } => vec![
                row(
                    action_id::CAPTURE,
                    "Capture Now",
                    "Skip the rest of the countdown",
                    Some("↵"),
                ),
                row(
                    action_id::CANCEL_COUNTDOWN,
                    "Cancel Countdown",
                    "Back to the camera without a photo",
                    Some("⎋"),
                ),
            ],
            WebcamPhase::Review { .. } => vec![
                row(
                    action_id::COPY_IMAGE,
                    "Copy Image",
                    "Copy the photo to paste into any app",
                    Some("↵"),
                ),
                row(
                    action_id::COPY_PATH,
                    "Copy Path",
                    "Copy the full path of the saved photo",
                    Some(COPY_PATH_SHORTCUT),
                ),
                row(action_id::REVEAL, "Show in Finder", "Reveal the saved photo", None),
                row(action_id::OPEN, "Open", "Open the photo in its default app", None),
                row(
                    action_id::RETAKE,
                    "Retake",
                    "Back to the camera; this photo stays saved",
                    Some("⎋"),
                ),
                row(action_id::CLOSE, "Close", "Turn off the camera", None),
            ],
        }
    }

    /// A one-word phase for automation (`getElements`): `live`, `countdown`, or `review`.
    pub fn phase_name(&self) -> &'static str {
        match self.phase {
            WebcamPhase::Live => "live",
            WebcamPhase::Countdown { .. } => "countdown",
            WebcamPhase::Review { .. } => "review",
        }
    }
}

/// Large countdown numeral. No display-size typography token exists; this one surface needs a
/// number readable from a few steps away while you pose.
const COUNTDOWN_NUMERAL_SIZE: f32 = 96.0;
const COUNTDOWN_DISC_SIZE: f32 = 168.0;

/// What the session draws over the camera picture: the countdown number, or the saved file's name
/// while reviewing. `None` on the plain live preview.
pub fn render_overlay(
    session: &WebcamSession,
    dc: &super::base::DesignContext<'_>,
) -> Option<gpui::AnyElement> {
    use crate::ui::chrome as tokens;
    use gpui::{div, prelude::*, px, FontWeight};

    match session.phase() {
        WebcamPhase::Live => None,
        WebcamPhase::Countdown { remaining } => {
            let mut disc_bg = dc.bg_main();
            disc_bg.a = crate::theme::opacity::OPACITY_STRONG;
            Some(
                div()
                    .id("webcam-countdown")
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(COUNTDOWN_DISC_SIZE))
                            .rounded_full()
                            .bg(disc_bg)
                            .text_size(px(COUNTDOWN_NUMERAL_SIZE))
                            .font_weight(FontWeight::BOLD)
                            .text_color(dc.text_primary())
                            .child(remaining.to_string()),
                    )
                    .into_any_element(),
            )
        }
        WebcamPhase::Review { path } => {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            let mut chip_bg = dc.bg_main();
            chip_bg.a = crate::theme::opacity::OPACITY_STRONG;
            Some(
                div()
                    .id("webcam-review")
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom(px(tokens::LIQUID_GLASS_PANEL_PADDING_PX))
                    .flex()
                    .justify_center()
                    .child(
                        div()
                            .px(px(tokens::LIQUID_GLASS_PANEL_PADDING_PX))
                            .py(px(tokens::LIQUID_GLASS_DENSE_GAP_PX))
                            .rounded(px(tokens::LIQUID_GLASS_COMPACT_RADIUS_PX))
                            .bg(chip_bg)
                            .text_sm()
                            .text_color(dc.text_primary())
                            .text_ellipsis()
                            .overflow_hidden()
                            .child(format!("Saved {name}")),
                    )
                    .into_any_element(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reviewing() -> WebcamSession {
        let mut session = WebcamSession::default();
        session.captured(PathBuf::from("/tmp/webcam-photo-1.png"));
        session
    }

    #[test]
    fn a_countdown_shoots_exactly_once_after_its_seconds() {
        let mut session = WebcamSession::default();
        let generation = session.start_countdown(3).expect("starts from live");
        assert_eq!(session.phase(), &WebcamPhase::Countdown { remaining: 3 });
        assert_eq!(session.tick(generation), CountdownTick::Remaining(2));
        assert_eq!(session.tick(generation), CountdownTick::Remaining(1));
        assert_eq!(session.tick(generation), CountdownTick::Shoot);
        assert_eq!(session.phase(), &WebcamPhase::Live);
        assert_eq!(session.tick(generation), CountdownTick::Stale);
    }

    #[test]
    fn a_cancelled_countdown_never_shoots_even_if_restarted() {
        let mut session = WebcamSession::default();
        let first = session.start_countdown(1).unwrap();
        assert_eq!(session.escape(), WebcamEscape::CancelledCountdown);
        assert_eq!(session.tick(first), CountdownTick::Stale);

        // A new countdown's timer owns the shot; the old timer's late tick is ignored.
        let second = session.start_countdown(2).unwrap();
        assert_eq!(session.tick(first), CountdownTick::Stale);
        assert_eq!(session.phase(), &WebcamPhase::Countdown { remaining: 2 });
        assert_eq!(session.tick(second), CountdownTick::Remaining(1));
    }

    #[test]
    fn capture_now_ends_the_countdown_so_its_timer_cannot_shoot_again() {
        let mut session = WebcamSession::default();
        let generation = session.start_countdown(3).unwrap();
        assert!(session.end_countdown());
        session.captured(PathBuf::from("/tmp/a.png"));
        assert_eq!(session.tick(generation), CountdownTick::Stale);
        assert_eq!(session.photo_path(), Some(Path::new("/tmp/a.png")));
    }

    #[test]
    fn countdowns_start_only_from_the_live_camera() {
        let mut session = WebcamSession::default();
        assert_eq!(session.start_countdown(0), None);
        session.start_countdown(3).unwrap();
        assert_eq!(session.start_countdown(3), None, "no second countdown over the first");
        assert_eq!(reviewing().start_countdown(3), None, "a review must be left first");
    }

    #[test]
    fn escape_steps_back_one_phase_at_a_time() {
        let mut session = reviewing();
        assert_eq!(session.escape(), WebcamEscape::Retake);
        assert_eq!(session.phase(), &WebcamPhase::Live);
        assert_eq!(session.photo_path(), None);
        assert_eq!(session.escape(), WebcamEscape::Leave);
    }

    #[test]
    fn enter_and_its_first_action_row_always_agree() {
        let mut countdown = WebcamSession::default();
        countdown.start_countdown(3).unwrap();
        for session in [WebcamSession::default(), countdown, reviewing()] {
            let rows = session.action_rows();
            assert_eq!(rows[0].id, session.primary_action_id());
            assert_eq!(rows[0].title, session.primary_label());
            assert_eq!(rows[0].shortcut, Some("↵"));
            let mut ids: Vec<_> = rows.iter().map(|row| row.id).collect();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), rows.len(), "{:?}", session.phase());
        }
    }

    #[test]
    fn copy_actions_exist_only_while_a_saved_photo_is_shown() {
        let has_copy = |session: &WebcamSession| {
            session
                .action_rows()
                .iter()
                .any(|row| row.id == action_id::COPY_IMAGE || row.id == action_id::COPY_PATH)
        };
        assert!(!has_copy(&WebcamSession::default()));
        assert!(has_copy(&reviewing()));
    }
}
