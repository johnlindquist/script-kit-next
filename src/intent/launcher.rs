//! What the main menu knows about Jev's reading of the current text.
//!
//! Typing never waits on Jev: every keystroke bumps the generation, and a reading that
//! comes back for anything but the current text at the current generation is dropped.
//! A plan is only ever shown for the exact text it was read from.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::catalog::Catalog;
use super::context::LastAction;
use super::engine::Plan;
use super::jev::{JevError, Oracle};

/// How long typing must pause before Jev is asked.
pub const DEBOUNCE: Duration = Duration::from_millis(140);
/// Enter pressed during a reading waits this long for it, then runs what was shown.
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
    /// Nothing to read; any shown plan is gone.
    Skip,
    /// A plan read earlier for this same text and screen is shown again at once.
    Recalled,
    /// Ask Jev after the debounce.
    Read(Ticket),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finish {
    /// For older text or an older keystroke; nothing changes.
    Stale,
    /// A ready plan for the current text; the list should refresh.
    Shown,
    /// Jev answered but the plan isn't ready; nothing is pinned.
    NotReady,
    /// Jev failed; the launcher behaves as if Jev were off.
    Failed,
}

/// Enter pressed while a reading for the same text was in flight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldEnter {
    pub request: String,
    pub pressed_at: Instant,
    /// The row Enter would have run, so a changed selection is noticed.
    pub shown_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeldOutcome {
    /// The selection is still what the user saw: run it.
    Run,
    /// A different row took the selection: show it and wait for another Enter.
    Changed,
}

#[derive(Debug)]
pub struct IntentLauncher {
    oracle: OracleSlot,
    generation: u64,
    epoch: u64,
    in_flight: Option<Ticket>,
    plan: Option<Arc<Plan>>,
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
            plan: None,
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
        if self.plan.as_ref().is_some_and(|plan| plan.request != text) {
            self.plan = None;
        }
        if !eligible || self.is_off() {
            self.plan = None;
            return TextChange::Skip;
        }
        if let Some(plan) = self.recalled(text, context_key) {
            if plan.ready {
                self.show(plan);
            }
            return TextChange::Recalled;
        }
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

    fn show(&mut self, plan: Arc<Plan>) {
        self.plan = Some(plan);
        self.epoch = self.epoch.wrapping_add(1);
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
        match result {
            Ok(plan) if plan.request != ticket.request => {
                self.plan = None;
                Finish::Failed
            }
            Ok(plan) => {
                let plan = Arc::new(plan);
                self.remember(ticket.context_key, plan.clone());
                if plan.ready {
                    self.show(plan);
                    Finish::Shown
                } else {
                    self.plan = None;
                    Finish::NotReady
                }
            }
            Err(error) => {
                self.plan = None;
                if matches!(error, JevError::MissingKey | JevError::Refused(_)) {
                    self.oracle = OracleSlot::Off("unavailable");
                }
                Finish::Failed
            }
        }
    }

    /// The plan to pin for this text, only if it was read from exactly this text.
    pub fn plan_for(&self, text: &str) -> Option<&Arc<Plan>> {
        self.plan
            .as_ref()
            .filter(|plan| plan.ready && plan.request == text)
    }

    /// Part of the list's cache key, so a new plan for the same text rebuilds the list.
    pub fn cache_token(&self, text: &str) -> u64 {
        self.plan_for(text).map_or(0, |_| self.epoch)
    }

    pub fn is_reading(&self, text: &str) -> bool {
        self.in_flight
            .as_ref()
            .is_some_and(|ticket| ticket.request == text)
    }

    /// Enter was pressed. Holds it when a reading for this text is in flight and the
    /// user hasn't picked a row themselves.
    pub fn hold_enter(
        &mut self,
        text: &str,
        anchored: bool,
        shown_key: Option<String>,
        now: Instant,
    ) -> bool {
        if anchored || !self.is_reading(text) {
            return false;
        }
        self.held = Some(HeldEnter {
            request: text.to_string(),
            pressed_at: now,
            shown_key,
        });
        true
    }

    pub fn held_enter(&self) -> Option<&HeldEnter> {
        self.held.as_ref()
    }

    /// Resolve a held Enter once the reading settles or the wait runs out.
    pub fn release_enter(&mut self, text: &str, selected_key: Option<&str>) -> Option<HeldOutcome> {
        let held = self.held.take()?;
        if held.request != text {
            return None;
        }
        if held.shown_key.as_deref() == selected_key {
            Some(HeldOutcome::Run)
        } else {
            Some(HeldOutcome::Changed)
        }
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

    /// Stop waiting on the reading in flight; its answer will be dropped.
    pub fn abandon_reading(&mut self) {
        self.in_flight = None;
    }

    /// Forget the shown plan, e.g. when the launcher hides.
    pub fn reset_reading(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.in_flight = None;
        self.plan = None;
        self.held = None;
    }
}
