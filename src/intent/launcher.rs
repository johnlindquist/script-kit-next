//! What an answer from Jev needs to run from the main window.
//!
//! The one-window reducer decides when Jev is asked, which answer the list shows and what Enter
//! runs; this keeps the oracle, the launcher's catalog, the plans behind Jev's rows for the words
//! on screen, and what ran last so "save that as a script" has something to save.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use super::catalog::Catalog;
use super::context::LastAction;
use super::engine::{Pathway, Plan, COMPLETE_THRESHOLD};
use super::jev::{JevError, Oracle};

/// Jev fills this many rows under the input.
pub const READINGS: usize = 3;
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

/// One of Jev's rows: a way it read the words, as a plan the row runs.
#[derive(Debug, Clone)]
pub struct JevRow {
    /// The same reading keeps the same key across answers, so it keeps its row.
    pub key: String,
    pub plan: Arc<Plan>,
}

/// Jev's readings of what was said: the plan it chose, then the alternatives it weighed, each
/// complete enough to run on its own. At most [`READINGS`], never two with the same key.
pub fn readings(plan: Plan) -> Vec<JevRow> {
    let mut alternatives = Vec::new();
    let mut top = plan;
    std::mem::swap(&mut alternatives, &mut top.alternatives);
    let mut plans = Vec::with_capacity(READINGS);
    if top.pathway != Pathway::None {
        plans.push(top.clone());
    }
    for reading in alternatives {
        let ready = reading.missing.is_none()
            && !reading.steps.is_empty()
            && top.complete >= COMPLETE_THRESHOLD;
        plans.push(Plan {
            reading,
            ready,
            clauses: Vec::new(),
            ..top.clone()
        });
    }
    let mut out: Vec<JevRow> = Vec::with_capacity(READINGS);
    for plan in plans {
        let key = format!("jev:{}:{}", plan.pathway.key(), plan.title);
        if out.len() < READINGS && !out.iter().any(|r| r.key == key) {
            out.push(JevRow {
                key,
                plan: Arc::new(plan),
            });
        }
    }
    out
}

#[derive(Debug)]
pub struct IntentLauncher {
    oracle: OracleSlot,
    last_action: Option<LastAction>,
    recent: VecDeque<(String, u64, Vec<JevRow>)>,
    catalog: Option<(u64, Arc<Catalog>)>,
    shown_for: String,
    shown: HashMap<String, Arc<Plan>>,
    /// The first reading the list last showed.
    shown_top: Option<Arc<Plan>>,
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
            last_action: None,
            recent: VecDeque::new(),
            catalog: None,
            shown_for: String::new(),
            shown: HashMap::new(),
            shown_top: None,
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

    pub fn is_off(&self) -> bool {
        matches!(self.oracle, OracleSlot::Off(_))
    }

    /// A failure that means Jev can't be reached at all turns it off for the session.
    pub fn failed(&mut self, error: &JevError) {
        if matches!(error, JevError::MissingKey | JevError::Refused(_)) {
            self.oracle = OracleSlot::Off("unavailable");
        }
    }

    /// Readings of these same words on this same screen, from earlier.
    pub fn recalled(&self, text: &str, context_key: u64) -> Option<Vec<JevRow>> {
        self.recent
            .iter()
            .find(|(request, key, _)| request == text && *key == context_key)
            .map(|(_, _, readings)| readings.clone())
    }

    pub fn remember(&mut self, text: &str, context_key: u64, readings: &[JevRow]) {
        self.recent
            .retain(|(request, key, _)| !(request == text && *key == context_key));
        self.recent
            .push_front((text.to_string(), context_key, readings.to_vec()));
        self.recent.truncate(RECENT_LIMIT);
    }

    /// The list shows these readings for `text`. Plans shown earlier for the same words stay
    /// runnable: a row Jev just replaced still runs for a moment after.
    pub fn show(&mut self, text: &str, readings: &[JevRow]) {
        if self.shown_for != text {
            self.shown_for = text.to_string();
            self.shown.clear();
            self.shown_top = None;
        }
        if let Some(top) = readings.first() {
            self.shown_top = Some(top.plan.clone());
        }
        for reading in readings {
            self.shown.insert(reading.key.clone(), reading.plan.clone());
        }
    }

    /// The plan behind one of Jev's rows for exactly these words.
    pub fn plan(&self, text: &str, key: &str) -> Option<Arc<Plan>> {
        (self.shown_for == text)
            .then(|| self.shown.get(key).cloned())
            .flatten()
    }

    /// Jev's top reading as the list last showed it, with the words it was for.
    pub fn shown_top(&self) -> Option<(&str, &Arc<Plan>)> {
        self.shown_top
            .as_ref()
            .map(|plan| (self.shown_for.as_str(), plan))
    }

    /// The plan behind one of Jev's rows as it was last shown, whatever the input says now.
    pub fn shown_plan(&self, key: &str) -> Option<Arc<Plan>> {
        self.shown.get(key).cloned()
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
}
