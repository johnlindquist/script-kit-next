#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierEvent {
    CommandDown { at_ms: u64 },
    CommandUp { at_ms: u64 },
    OtherModifier,
    NonModifierKey,
    CombinedShortcut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoubleCommandOutcome {
    Idle,
    Armed,
    Trigger,
}

#[derive(Debug, Clone)]
pub struct DoubleCommandState {
    last_command_tap_ms: Option<u64>,
    threshold_ms: u64,
}

impl Default for DoubleCommandState {
    fn default() -> Self {
        Self {
            last_command_tap_ms: None,
            threshold_ms: 420,
        }
    }
}

impl DoubleCommandState {
    pub fn observe(&mut self, event: ModifierEvent) -> DoubleCommandOutcome {
        match event {
            ModifierEvent::CommandUp { at_ms } => {
                let triggered = self
                    .last_command_tap_ms
                    .is_some_and(|last| at_ms.saturating_sub(last) <= self.threshold_ms);
                self.last_command_tap_ms = Some(at_ms);
                if triggered {
                    self.last_command_tap_ms = None;
                    DoubleCommandOutcome::Trigger
                } else {
                    DoubleCommandOutcome::Armed
                }
            }
            ModifierEvent::CommandDown { .. } | ModifierEvent::OtherModifier => {
                DoubleCommandOutcome::Idle
            }
            ModifierEvent::NonModifierKey | ModifierEvent::CombinedShortcut => {
                self.last_command_tap_ms = None;
                DoubleCommandOutcome::Idle
            }
        }
    }
}
