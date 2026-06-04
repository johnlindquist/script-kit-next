use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

use crate::designs::MainMenuThemeDef;

use super::catalog::{knob_by_id, StyleKnobId, StyleValue, STYLE_KNOBS};

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
}

static STORE: OnceLock<RwLock<RuntimeStyleOverrides>> = OnceLock::new();

fn store() -> &'static RwLock<RuntimeStyleOverrides> {
    STORE.get_or_init(|| RwLock::new(RuntimeStyleOverrides::default()))
}

pub fn generation() -> u64 {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .generation
}

pub fn set_value(id: StyleKnobId, requested: StyleValue) -> Option<AppliedStyleChange> {
    let knob = knob_by_id(id)?;
    let applied = knob.clamp_value(requested);
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.values.insert(id, applied);
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
    guard.values.clear();
    guard.generation = guard.generation.saturating_add(1);
    guard.generation
}

pub fn current_value(id: StyleKnobId) -> Option<StyleValue> {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .values
        .get(&id)
        .copied()
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
