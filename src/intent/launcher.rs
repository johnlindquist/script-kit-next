//! What the main menu knows about Jev's reading of the current text.
//!
//! Typing never waits on Jev: every keystroke bumps the generation, and a reading that
//! comes back for anything but the current text at the current generation is dropped.
//! The intent row exists for a sentence from its first frame, reading until Jev answers,
//! and only ever shows a reading of the exact text in the input.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::catalog::Catalog;
use super::context::LastAction;
use super::engine::Plan;
use super::jev::{JevError, Oracle};

/// How long typing must pause before Jev is asked.
pub const DEBOUNCE: Duration = Duration::from_millis(140);
/// Enter on the intent row during a reading waits this long for it.
pub const HELD_ENTER_LIMIT: Duration = Duration::from_millis(1500);
const RECENT_LIMIT: usize = 24;

#[derive(Clone)]
pub enum OracleSlot {
    Unloaded,
    Off(&'static str),
    Ready(Arc<dyn Oracle>),
}

impl std::fmt::Debug for OracleSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleSlot::Unloaded => f.write_str("Unloaded"),
            OracleSlot::Off(reason) => write!(f, "Off({reason})"),
            OracleSlot::Ready(_) => f.write_str("Ready"),
        }
    }
}

/// A reading in flight: which text, at which generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
    pub generation: u64,
    pub request: String,
    pub context_key: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextChange {
    /// No intent row for this text.
    Skip,
    /// A plan read earlier for this same text and screen is shown again at once.
    Recalled,
    /// The row reads until Jev answers, asked after the debounce.
    Read(Ticket),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finish {
    /// For older text or an older keystroke; nothing changes.
    Stale,
    /// A ready plan for the current text.
    Shown,
    /// Jev answered but the plan needs more; the row says what.
    NotReady,
    /// Jev failed; the row says so and never runs.
    Failed,
}

/// What the intent row says.
#[derive(Debug, Clone)]
pub enum RowState {
    Reading,
    /// Runs only when the plan is ready.
    Read(Arc<Plan>),
    Failed,
}

#[derive(Debug, Clone)]
pub struct IntentRow {
    pub request: String,
    pub state: RowState,
}

impl IntentRow {
    /// The plan Enter runs, if Jev's reading is ready.
    pub fn runnable(&self) -> Option<&Arc<Plan>> {
        match &self.state {
            RowState::Read(plan) if plan.ready => Some(plan),
            _ => None,
        }
    }
}

/// Enter pressed on the intent row while it was still reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldEnter {
    pub request: String,
    pub pressed_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeldOutcome {
    /// The reading is ready and the intent row is still highlighted: run it.
    Run,
    /// The reading needs more, failed, or hasn't come back in time.
    Wait,
    /// The user moved off the intent row while it read.
    Moved,
}

#[derive(Debug)]
pub struct IntentLauncher {
    oracle: OracleSlot,
    generation: u64,
    epoch: u64,
    in_flight: Option<Ticket>,
    row: Option<IntentRow>,
    held: Option<HeldEnter>,
    last_action: Option<LastAction>,
    recent: VecDeque<(String, u64, Arc<Plan>)>,
    catalog: Option<(u64, Arc<Catalog>)>,
}

impl Default for IntentLauncher {
    fn default() -> Self {
        Self::with_slot(OracleSlot::Unloaded)
    }
}

impl IntentLauncher {
    pub fn with_slot(oracle: OracleSlot) -> Self {
        Self {
            oracle,
            generation: 0,
            epoch: 0,
            in_flight: None,
            row: None,
            held: None,
            last_action: None,
            recent: VecDeque::new(),
            catalog: None,
        }
    }

    pub fn with_oracle(oracle: Arc<dyn Oracle>) -> Self {
        Self::with_slot(OracleSlot::Ready(oracle))
    }

    pub fn off(reason: &'static str) -> Self {
        Self::with_slot(OracleSlot::Off(reason))
    }

    pub fn oracle_slot(&self) -> &OracleSlot {
        &self.oracle
    }

    pub fn set_oracle_slot(&mut self, oracle: OracleSlot) {
        self.oracle = oracle;
    }

    pub fn oracle(&self) -> Option<Arc<dyn Oracle>> {
        match &self.oracle {
            OracleSlot::Ready(oracle) => Some(oracle.clone()),
            _ => None,
        }
    }

    pub fn is_off(&self) -> bool {
        matches!(self.oracle, OracleSlot::Off(_))
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The committed text changed. `eligible` is false for launcher syntax, one word,
    /// or any surface that owns the list; `context_key` fingerprints the screen.
    pub fn text_changed(&mut self, text: &str, eligible: bool, context_key: u64) -> TextChange {
        self.generation = self.generation.wrapping_add(1);
        self.in_flight = None;
        if self.held.as_ref().is_some_and(|held| held.request != text) {
            self.held = None;
        }
        if !eligible || self.is_off() {
            if self.row.is_some() {
                self.set_row(None);
            }
            return TextChange::Skip;
        }
        if let Some(plan) = self.recalled(text, context_key) {
            self.set_row(Some(IntentRow {
                request: text.to_string(),
                state: RowState::Read(plan),
            }));
            return TextChange::Recalled;
        }
        self.set_row(Some(IntentRow {
            request: text.to_string(),
            state: RowState::Reading,
        }));
        let ticket = Ticket {
            generation: self.generation,
            request: text.to_string(),
            context_key,
        };
        self.in_flight = Some(ticket.clone());
        TextChange::Read(ticket)
    }

    fn recalled(&self, text: &str, context_key: u64) -> Option<Arc<Plan>> {
        self.recent
            .iter()
            .find(|(request, key, _)| request == text && *key == context_key)
            .map(|(_, _, plan)| plan.clone())
    }

    fn remember(&mut self, context_key: u64, plan: Arc<Plan>) {
        self.recent
            .retain(|(request, key, _)| !(request == &plan.request && *key == context_key));
        self.recent
            .push_front((plan.request.clone(), context_key, plan));
        self.recent.truncate(RECENT_LIMIT);
    }

    fn set_row(&mut self, row: Option<IntentRow>) {
        self.row = row;
        self.epoch = self.epoch.wrapping_add(1).max(1);
    }

    /// Whether a ticket is still the one the launcher is waiting on.
    pub fn is_current(&self, ticket: &Ticket) -> bool {
        self.in_flight.as_ref() == Some(ticket)
    }

    pub fn finish(
        &mut self,
        ticket: &Ticket,
        current_text: &str,
        result: Result<Plan, JevError>,
    ) -> Finish {
        if !self.is_current(ticket) || ticket.request != current_text {
            return Finish::Stale;
        }
        self.in_flight = None;
        let (state, finish) = match result {
            Ok(plan) if plan.request != ticket.request => (RowState::Failed, Finish::Failed),
            Ok(plan) => {
                let plan = Arc::new(plan);
                self.remember(ticket.context_key, plan.clone());
                let finish = if plan.ready {
                    Finish::Shown
                } else {
                    Finish::NotReady
                };
                (RowState::Read(plan), finish)
            }
            Err(error) => {
                if matches!(error, JevError::MissingKey | JevError::Refused(_)) {
                    self.oracle = OracleSlot::Off("unavailable");
                }
                (RowState::Failed, Finish::Failed)
            }
        };
        self.set_row(Some(IntentRow {
            request: ticket.request.clone(),
            state,
        }));
        finish
    }

    /// The intent row for exactly this text.
    pub fn row_for(&self, text: &str) -> Option<&IntentRow> {
        self.row.as_ref().filter(|row| row.request == text)
    }

    /// The plan Enter on the intent row runs, only if read from exactly this text.
    pub fn plan_for(&self, text: &str) -> Option<&Arc<Plan>> {
        self.row_for(text).and_then(IntentRow::runnable)
    }

    /// Whether the row shows Jev's words for this text rather than a placeholder.
    pub fn has_reading_for(&self, text: &str) -> bool {
        self.row_for(text)
            .is_some_and(|row| matches!(row.state, RowState::Read(_)))
    }

    /// Part of the list's cache key, so a changed row for the same text rebuilds the list.
    pub fn cache_token(&self, text: &str) -> u64 {
        self.row_for(text).map_or(0, |_| self.epoch)
    }

    pub fn is_reading(&self, text: &str) -> bool {
        self.in_flight
            .as_ref()
            .is_some_and(|ticket| ticket.request == text)
    }

    /// Enter on the intent row. Holds it when a reading for this text is in flight.
    pub fn hold_enter(&mut self, text: &str, now: Instant) -> bool {
        if !self.is_reading(text) {
            return false;
        }
        self.held = Some(HeldEnter {
            request: text.to_string(),
            pressed_at: now,
        });
        true
    }

    pub fn held_enter(&self) -> Option<&HeldEnter> {
        self.held.as_ref()
    }

    /// Resolve a held Enter once the reading settles or the wait runs out.
    pub fn release_enter(&mut self, text: &str, on_intent_row: bool) -> Option<HeldOutcome> {
        let held = self.held.take()?;
        if held.request != text {
            return None;
        }
        Some(if !on_intent_row {
            HeldOutcome::Moved
        } else if self.plan_for(text).is_some() {
            HeldOutcome::Run
        } else {
            HeldOutcome::Wait
        })
    }

    pub fn record(&mut self, action: LastAction) {
        self.last_action = Some(action);
    }

    pub fn last_action(&self) -> Option<&LastAction> {
        self.last_action.as_ref()
    }

    /// The catalog built for this launcher data revision, if still current.
    pub fn catalog_for(&self, revision: u64) -> Option<Arc<Catalog>> {
        self.catalog
            .as_ref()
            .filter(|(key, _)| *key == revision)
            .map(|(_, catalog)| catalog.clone())
    }

    pub fn store_catalog(&mut self, revision: u64, catalog: Arc<Catalog>) {
        self.catalog = Some((revision, catalog));
    }

    /// Forget the row, e.g. when the launcher hides.
    pub fn reset_reading(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.in_flight = None;
        self.held = None;
        self.set_row(None);
    }
}
