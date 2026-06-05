use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

use crate::designs::MainMenuThemeDef;

use super::catalog::{knob_by_id, knob_id_from_str, StyleKnobId, StyleValue, STYLE_KNOBS};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppliedStyleChange {
    pub generation: u64,
    pub previous: Option<StyleValue>,
    pub requested: StyleValue,
    pub applied: StyleValue,
}

#[derive(Debug, Clone, Default)]
struct RuntimeStyleOverrides {
    generation: u64,
    values: BTreeMap<StyleKnobId, StyleValue>,
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
}

static STORE: OnceLock<RwLock<RuntimeStyleOverrides>> = OnceLock::new();

#[derive(Debug, Clone)]
enum HistoryEntry {
    Single {
        id: StyleKnobId,
        before: Option<StyleValue>,
        after: Option<StyleValue>,
    },
    Snapshot {
        before: BTreeMap<StyleKnobId, StyleValue>,
        after: BTreeMap<StyleKnobId, StyleValue>,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleHistoryState {
    pub can_undo: bool,
    pub can_redo: bool,
    pub override_count: usize,
    pub generation: u64,
}

fn store() -> &'static RwLock<RuntimeStyleOverrides> {
    STORE.get_or_init(|| RwLock::new(RuntimeStyleOverrides::default()))
}

pub fn generation() -> u64 {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .generation
}

pub fn history_state() -> StyleHistoryState {
    let guard = store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    StyleHistoryState {
        can_undo: !guard.undo_stack.is_empty(),
        can_redo: !guard.redo_stack.is_empty(),
        override_count: guard.values.len(),
        generation: guard.generation,
    }
}

pub fn set_value(id: StyleKnobId, requested: StyleValue) -> Option<AppliedStyleChange> {
    let knob = knob_by_id(id)?;
    let applied = knob.clamp_value(requested);
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.values.insert(id, applied);
    guard.undo_stack.push(HistoryEntry::Single {
        id,
        before: previous,
        after: Some(applied),
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    Some(AppliedStyleChange {
        generation: guard.generation,
        previous,
        requested,
        applied,
    })
}

pub fn reset_value(id: StyleKnobId) -> Option<AppliedStyleChange> {
    let knob = knob_by_id(id)?;
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.values.remove(&id);
    guard.undo_stack.push(HistoryEntry::Single {
        id,
        before: previous,
        after: None,
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    let base_value = (knob.get)(&crate::designs::current_main_menu_theme().base_def());
    Some(AppliedStyleChange {
        generation: guard.generation,
        previous,
        requested: base_value,
        applied: base_value,
    })
}

pub fn reset_all() -> u64 {
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let before = guard.values.clone();
    guard.values.clear();
    guard.undo_stack.push(HistoryEntry::Snapshot {
        before,
        after: BTreeMap::new(),
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    guard.generation
}

pub fn undo_last() -> Option<String> {
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let entry = guard.undo_stack.pop()?;
    apply_history_entry(&mut guard.values, &entry, HistoryDirection::Undo);
    guard.redo_stack.push(entry.clone());
    guard.generation = guard.generation.saturating_add(1);
    Some(format_history_result("undo", &entry, guard.generation))
}

pub fn redo_last() -> Option<String> {
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let entry = guard.redo_stack.pop()?;
    apply_history_entry(&mut guard.values, &entry, HistoryDirection::Redo);
    guard.undo_stack.push(entry.clone());
    guard.generation = guard.generation.saturating_add(1);
    Some(format_history_result("redo", &entry, guard.generation))
}

pub fn current_value(id: StyleKnobId) -> Option<StyleValue> {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .values
        .get(&id)
        .copied()
}

#[derive(Debug, Clone, Copy)]
enum HistoryDirection {
    Undo,
    Redo,
}

fn apply_history_entry(
    values: &mut BTreeMap<StyleKnobId, StyleValue>,
    entry: &HistoryEntry,
    direction: HistoryDirection,
) {
    match (entry, direction) {
        (HistoryEntry::Single { id, before, .. }, HistoryDirection::Undo) => {
            apply_optional_value(values, *id, *before);
        }
        (HistoryEntry::Single { id, after, .. }, HistoryDirection::Redo) => {
            apply_optional_value(values, *id, *after);
        }
        (HistoryEntry::Snapshot { before, .. }, HistoryDirection::Undo) => {
            *values = before.clone();
        }
        (HistoryEntry::Snapshot { after, .. }, HistoryDirection::Redo) => {
            *values = after.clone();
        }
    }
}

fn apply_optional_value(
    values: &mut BTreeMap<StyleKnobId, StyleValue>,
    id: StyleKnobId,
    value: Option<StyleValue>,
) {
    if let Some(value) = value {
        values.insert(id, value);
    } else {
        values.remove(&id);
    }
}

fn format_history_result(action: &str, entry: &HistoryEntry, generation: u64) -> String {
    match entry {
        HistoryEntry::Single { id, .. } => {
            format!("{action}:{} generation={generation}", id.as_str())
        }
        HistoryEntry::Snapshot { before, after } => format!(
            "{action}:all before={} after={} generation={generation}",
            before.len(),
            after.len()
        ),
    }
}

pub fn set_number_from_devtools(control: &str, value: &str) -> anyhow::Result<String> {
    let id = knob_id_from_str(control)
        .ok_or_else(|| anyhow::anyhow!("unknown dev style control '{control}'"))?;
    let parsed = value
        .trim()
        .trim_end_matches("px")
        .trim()
        .parse::<f32>()
        .map_err(|_| anyhow::anyhow!("invalid numeric value '{value}'"))?;
    let change = set_value(id, StyleValue::Number(parsed))
        .ok_or_else(|| anyhow::anyhow!("unknown dev style control '{control}'"))?;
    let StyleValue::Number(applied) = change.applied;
    Ok(format!("{}={applied}", id.as_str()))
}

pub fn apply_to_main_menu_def(mut def: MainMenuThemeDef) -> MainMenuThemeDef {
    let guard = store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for knob in STYLE_KNOBS {
        if let Some(value) = guard.values.get(&knob.id).copied() {
            (knob.apply)(&mut def, value);
        }
    }
    def
}
