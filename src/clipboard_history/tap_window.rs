//! Pure post-copy modifier tap window state machine (T12).
//!
//! After a stored clipboard capture, watches for a bare trigger-modifier tap
//! (down+up with no other key) within a configurable window. Any non-modifier
//! key-down while the modifier is held cancels the pending window (⌘V after ⌘C).

use std::time::{Duration, Instant};

/// Inputs to the tap-window state machine. Timestamps are injected by callers
/// (no internal clock) so unit tests stay deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapWindowInput {
    CopyStored { at: Instant },
    ModifierDown { at: Instant },
    ModifierUp { at: Instant },
    KeyDown { at: Instant },
    Tick { at: Instant },
}

/// Outputs emitted when the machine transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TapWindowOutput {
    /// Open the post-copy quick menu for the pending entry.
    OpenMenu,
    /// Pending window ended without opening the menu.
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Pending {
        expires_at: Instant,
        modifier_down: bool,
        chord_seen: bool,
    },
}

/// Post-copy ⌘-tap detector. Thread-safe usage is the caller's responsibility;
/// the type itself is a pure state container.
#[derive(Debug, Clone)]
pub struct TapWindowMachine {
    window: Duration,
    phase: Phase,
}

impl TapWindowMachine {
    pub fn new(window_ms: u64) -> Self {
        Self {
            window: Duration::from_millis(window_ms),
            phase: Phase::Idle,
        }
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.phase, Phase::Pending { .. })
    }

    pub fn apply(&mut self, input: TapWindowInput) -> Vec<TapWindowOutput> {
        match (self.phase, input) {
            (Phase::Pending { expires_at, .. }, TapWindowInput::Tick { at })
                if at >= expires_at =>
            {
                self.phase = Phase::Idle;
                vec![TapWindowOutput::Cancelled]
            }
            (Phase::Pending { .. }, TapWindowInput::Tick { .. }) => Vec::new(),
            (Phase::Idle, TapWindowInput::CopyStored { at }) => {
                self.phase = Phase::Pending {
                    expires_at: at + self.window,
                    modifier_down: false,
                    chord_seen: false,
                };
                Vec::new()
            }

            (
                Phase::Pending {
                    expires_at: _,
                    modifier_down,
                    ..
                },
                TapWindowInput::CopyStored { at },
            ) => {
                self.phase = Phase::Pending {
                    expires_at: at + self.window,
                    modifier_down,
                    chord_seen: false,
                };
                Vec::new()
            }

            (
                Phase::Pending {
                    expires_at,
                    chord_seen,
                    ..
                },
                TapWindowInput::ModifierDown { at },
            ) if at < expires_at => {
                self.phase = Phase::Pending {
                    expires_at,
                    modifier_down: true,
                    chord_seen,
                };
                Vec::new()
            }

            (
                Phase::Pending {
                    expires_at,
                    modifier_down: true,
                    chord_seen: false,
                },
                TapWindowInput::ModifierUp { at },
            ) if at < expires_at => {
                self.phase = Phase::Idle;
                vec![TapWindowOutput::OpenMenu]
            }

            (Phase::Pending { expires_at, .. }, TapWindowInput::ModifierUp { at })
                if at < expires_at =>
            {
                self.phase = Phase::Pending {
                    expires_at,
                    modifier_down: false,
                    chord_seen: false,
                };
                Vec::new()
            }

            (
                Phase::Pending {
                    expires_at,
                    modifier_down: true,
                    ..
                },
                TapWindowInput::KeyDown { at },
            ) if at < expires_at => {
                self.phase = Phase::Pending {
                    expires_at,
                    modifier_down: true,
                    chord_seen: true,
                };
                Vec::new()
            }

            (Phase::Pending { .. }, TapWindowInput::KeyDown { .. })
            | (Phase::Pending { .. }, TapWindowInput::ModifierDown { .. })
            | (Phase::Pending { .. }, TapWindowInput::ModifierUp { .. }) => Vec::new(),

            (Phase::Idle, _) => Vec::new(),
        }
    }

    /// Force-clear any pending window (menu opened, entry rejected, etc.).
    pub fn reset(&mut self) {
        self.phase = Phase::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Instant {
        Instant::now()
    }

    #[test]
    fn bare_modifier_tap_after_copy_opens_menu() {
        let mut machine = TapWindowMachine::new(2500);
        let start = base();

        assert!(machine
            .apply(TapWindowInput::CopyStored { at: start })
            .is_empty());
        assert!(machine
            .apply(TapWindowInput::ModifierDown {
                at: start + Duration::from_millis(100)
            })
            .is_empty());
        let outputs = machine.apply(TapWindowInput::ModifierUp {
            at: start + Duration::from_millis(150),
        });
        assert_eq!(outputs, vec![TapWindowOutput::OpenMenu]);
        assert!(!machine.is_pending());
    }

    #[test]
    fn modifier_release_after_copy_without_prior_down_opens_menu() {
        let mut machine = TapWindowMachine::new(2500);
        let start = base();

        machine.apply(TapWindowInput::CopyStored { at: start });
        // Simulate copy completing while ⌘ still held from ⌘C, then released.
        assert!(machine
            .apply(TapWindowInput::ModifierDown { at: start })
            .is_empty());
        let outputs = machine.apply(TapWindowInput::ModifierUp {
            at: start + Duration::from_millis(50),
        });
        assert_eq!(outputs, vec![TapWindowOutput::OpenMenu]);
    }

    #[test]
    fn chord_while_modifier_down_cancels_menu_on_release() {
        let mut machine = TapWindowMachine::new(2500);
        let start = base();

        machine.apply(TapWindowInput::CopyStored { at: start });
        machine.apply(TapWindowInput::ModifierDown {
            at: start + Duration::from_millis(10),
        });
        machine.apply(TapWindowInput::KeyDown {
            at: start + Duration::from_millis(20),
        });
        let outputs = machine.apply(TapWindowInput::ModifierUp {
            at: start + Duration::from_millis(30),
        });
        assert!(outputs.is_empty());
        assert!(machine.is_pending());
    }

    #[test]
    fn timeout_cancels_pending_window() {
        let mut machine = TapWindowMachine::new(2500);
        let start = base();

        machine.apply(TapWindowInput::CopyStored { at: start });
        let outputs = machine.apply(TapWindowInput::Tick {
            at: start + Duration::from_millis(2501),
        });
        assert_eq!(outputs, vec![TapWindowOutput::Cancelled]);
        assert!(!machine.is_pending());
    }

    #[test]
    fn no_pending_entry_ignores_modifier_tap() {
        let mut machine = TapWindowMachine::new(2500);
        let start = base();

        assert!(machine
            .apply(TapWindowInput::ModifierDown { at: start })
            .is_empty());
        assert!(machine
            .apply(TapWindowInput::ModifierUp { at: start })
            .is_empty());
    }

    #[test]
    fn fresh_copy_rearms_pending_window() {
        let mut machine = TapWindowMachine::new(2500);
        let start = base();

        machine.apply(TapWindowInput::CopyStored { at: start });
        machine.apply(TapWindowInput::ModifierDown {
            at: start + Duration::from_millis(10),
        });
        machine.apply(TapWindowInput::KeyDown {
            at: start + Duration::from_millis(20),
        });

        let rearms = start + Duration::from_millis(500);
        machine.apply(TapWindowInput::CopyStored { at: rearms });
        machine.apply(TapWindowInput::ModifierDown {
            at: rearms + Duration::from_millis(40),
        });
        let outputs = machine.apply(TapWindowInput::ModifierUp {
            at: rearms + Duration::from_millis(60),
        });
        assert_eq!(outputs, vec![TapWindowOutput::OpenMenu]);
    }
}
