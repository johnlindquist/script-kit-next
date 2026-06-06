use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

use crate::designs::{ActionsPopupThemeDef, MainMenuThemeDef};

use super::actions_popup_catalog::{
    actions_popup_knob_by_id, actions_popup_knob_id_from_str, ActionsPopupKnobId,
    ACTIONS_POPUP_KNOBS,
};
use super::agent_chat_catalog::{
    agent_chat_knob_by_id, agent_chat_knob_id_from_str, base_agent_chat_style, AgentChatKnobId,
    AgentChatStyleDef, AGENT_CHAT_KNOBS,
};
use super::catalog::{knob_by_id, knob_id_from_str, StyleKnobId, StyleValue, STYLE_KNOBS};
use super::copy_catalog::{copy_control_by_id, copy_control_id_from_str, CopyControlId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppliedStyleChange {
    pub generation: u64,
    pub previous: Option<StyleValue>,
    pub requested: StyleValue,
    pub applied: StyleValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppliedCopyChange {
    pub generation: u64,
    pub previous: Option<String>,
    pub requested: String,
    pub applied: String,
}

#[derive(Debug, Clone, Default)]
struct RuntimeStyleOverrides {
    generation: u64,
    values: BTreeMap<StyleKnobId, StyleValue>,
    copy_values: BTreeMap<CopyControlId, String>,
    actions_values: BTreeMap<ActionsPopupKnobId, StyleValue>,
    agent_chat_values: BTreeMap<AgentChatKnobId, StyleValue>,
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
    CopySingle {
        id: CopyControlId,
        before: Option<String>,
        after: Option<String>,
    },
    ActionsSingle {
        id: ActionsPopupKnobId,
        before: Option<StyleValue>,
        after: Option<StyleValue>,
    },
    AgentChatSingle {
        id: AgentChatKnobId,
        before: Option<StyleValue>,
        after: Option<StyleValue>,
    },
    Snapshot {
        before: BTreeMap<StyleKnobId, StyleValue>,
        copy_before: BTreeMap<CopyControlId, String>,
        actions_before: BTreeMap<ActionsPopupKnobId, StyleValue>,
        agent_chat_before: BTreeMap<AgentChatKnobId, StyleValue>,
        after: BTreeMap<StyleKnobId, StyleValue>,
        copy_after: BTreeMap<CopyControlId, String>,
        actions_after: BTreeMap<ActionsPopupKnobId, StyleValue>,
        agent_chat_after: BTreeMap<AgentChatKnobId, StyleValue>,
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
        override_count: guard
            .values
            .len()
            .saturating_add(guard.copy_values.len())
            .saturating_add(guard.actions_values.len())
            .saturating_add(guard.agent_chat_values.len()),
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

pub fn set_copy_value(id: CopyControlId, requested: String) -> Option<AppliedCopyChange> {
    let _control = copy_control_by_id(id)?;
    let applied = requested;
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.copy_values.insert(id, applied.clone());
    guard.undo_stack.push(HistoryEntry::CopySingle {
        id,
        before: previous.clone(),
        after: Some(applied.clone()),
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    Some(AppliedCopyChange {
        generation: guard.generation,
        previous,
        requested: applied.clone(),
        applied,
    })
}

pub fn reset_copy_value(id: CopyControlId) -> Option<AppliedCopyChange> {
    let control = copy_control_by_id(id)?;
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.copy_values.remove(&id);
    guard.undo_stack.push(HistoryEntry::CopySingle {
        id,
        before: previous.clone(),
        after: None,
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    let base_value = (control.base)();
    Some(AppliedCopyChange {
        generation: guard.generation,
        previous,
        requested: base_value.clone(),
        applied: base_value,
    })
}

pub fn current_copy_value(id: CopyControlId) -> Option<String> {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .copy_values
        .get(&id)
        .cloned()
}

pub fn effective_copy_value(id: CopyControlId) -> String {
    current_copy_value(id).unwrap_or_else(|| {
        copy_control_by_id(id)
            .map(|control| (control.base)())
            .unwrap_or_default()
    })
}

pub fn effective_main_input_placeholder() -> String {
    effective_copy_value(super::copy_catalog::MAIN_INPUT_PLACEHOLDER_COPY_ID)
}

pub fn set_actions_popup_value(
    id: ActionsPopupKnobId,
    requested: StyleValue,
) -> Option<AppliedStyleChange> {
    let knob = actions_popup_knob_by_id(id)?;
    let applied = knob.clamp_value(requested);
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.actions_values.insert(id, applied);
    guard.undo_stack.push(HistoryEntry::ActionsSingle {
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

pub fn reset_actions_popup_value(id: ActionsPopupKnobId) -> Option<AppliedStyleChange> {
    let knob = actions_popup_knob_by_id(id)?;
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.actions_values.remove(&id);
    guard.undo_stack.push(HistoryEntry::ActionsSingle {
        id,
        before: previous,
        after: None,
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    let base_value = (knob.get)(&crate::designs::base_actions_popup_theme());
    Some(AppliedStyleChange {
        generation: guard.generation,
        previous,
        requested: base_value,
        applied: base_value,
    })
}

pub fn current_actions_popup_value(id: ActionsPopupKnobId) -> Option<StyleValue> {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .actions_values
        .get(&id)
        .copied()
}

pub fn set_agent_chat_value(
    id: AgentChatKnobId,
    requested: StyleValue,
) -> Option<AppliedStyleChange> {
    let knob = agent_chat_knob_by_id(id)?;
    let applied = knob.clamp_value(requested);
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.agent_chat_values.insert(id, applied);
    guard.undo_stack.push(HistoryEntry::AgentChatSingle {
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

pub fn reset_agent_chat_value(id: AgentChatKnobId) -> Option<AppliedStyleChange> {
    let knob = agent_chat_knob_by_id(id)?;
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = guard.agent_chat_values.remove(&id);
    guard.undo_stack.push(HistoryEntry::AgentChatSingle {
        id,
        before: previous,
        after: None,
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    let base_value = (knob.get)(&base_agent_chat_style());
    Some(AppliedStyleChange {
        generation: guard.generation,
        previous,
        requested: base_value,
        applied: base_value,
    })
}

pub fn current_agent_chat_value(id: AgentChatKnobId) -> Option<StyleValue> {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .agent_chat_values
        .get(&id)
        .copied()
}

pub fn reset_all() -> u64 {
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let before = guard.values.clone();
    let copy_before = guard.copy_values.clone();
    let actions_before = guard.actions_values.clone();
    let agent_chat_before = guard.agent_chat_values.clone();
    guard.values.clear();
    guard.copy_values.clear();
    guard.actions_values.clear();
    guard.agent_chat_values.clear();
    guard.undo_stack.push(HistoryEntry::Snapshot {
        before,
        copy_before,
        actions_before,
        agent_chat_before,
        after: BTreeMap::new(),
        copy_after: BTreeMap::new(),
        actions_after: BTreeMap::new(),
        agent_chat_after: BTreeMap::new(),
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
    apply_history_entry(&mut guard, &entry, HistoryDirection::Undo);
    guard.redo_stack.push(entry.clone());
    guard.generation = guard.generation.saturating_add(1);
    Some(format_history_result("undo", &entry, guard.generation))
}

pub fn redo_last() -> Option<String> {
    let mut guard = store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let entry = guard.redo_stack.pop()?;
    apply_history_entry(&mut guard, &entry, HistoryDirection::Redo);
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

pub fn copy_override_count() -> usize {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .copy_values
        .len()
}

pub fn actions_popup_override_count() -> usize {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .actions_values
        .len()
}

pub fn agent_chat_override_count() -> usize {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .agent_chat_values
        .len()
}

#[derive(Debug, Clone, Copy)]
enum HistoryDirection {
    Undo,
    Redo,
}

fn apply_history_entry(
    overrides: &mut RuntimeStyleOverrides,
    entry: &HistoryEntry,
    direction: HistoryDirection,
) {
    match (entry, direction) {
        (HistoryEntry::Single { id, before, .. }, HistoryDirection::Undo) => {
            apply_optional_value(&mut overrides.values, *id, *before);
        }
        (HistoryEntry::Single { id, after, .. }, HistoryDirection::Redo) => {
            apply_optional_value(&mut overrides.values, *id, *after);
        }
        (HistoryEntry::CopySingle { id, before, .. }, HistoryDirection::Undo) => {
            apply_optional_copy_value(&mut overrides.copy_values, *id, before.clone());
        }
        (HistoryEntry::CopySingle { id, after, .. }, HistoryDirection::Redo) => {
            apply_optional_copy_value(&mut overrides.copy_values, *id, after.clone());
        }
        (HistoryEntry::ActionsSingle { id, before, .. }, HistoryDirection::Undo) => {
            apply_optional_actions_value(&mut overrides.actions_values, *id, *before);
        }
        (HistoryEntry::ActionsSingle { id, after, .. }, HistoryDirection::Redo) => {
            apply_optional_actions_value(&mut overrides.actions_values, *id, *after);
        }
        (HistoryEntry::AgentChatSingle { id, before, .. }, HistoryDirection::Undo) => {
            apply_optional_agent_chat_value(&mut overrides.agent_chat_values, *id, *before);
        }
        (HistoryEntry::AgentChatSingle { id, after, .. }, HistoryDirection::Redo) => {
            apply_optional_agent_chat_value(&mut overrides.agent_chat_values, *id, *after);
        }
        (
            HistoryEntry::Snapshot {
                before,
                copy_before,
                actions_before,
                agent_chat_before,
                ..
            },
            HistoryDirection::Undo,
        ) => {
            overrides.values = before.clone();
            overrides.copy_values = copy_before.clone();
            overrides.actions_values = actions_before.clone();
            overrides.agent_chat_values = agent_chat_before.clone();
        }
        (
            HistoryEntry::Snapshot {
                after,
                copy_after,
                actions_after,
                agent_chat_after,
                ..
            },
            HistoryDirection::Redo,
        ) => {
            overrides.values = after.clone();
            overrides.copy_values = copy_after.clone();
            overrides.actions_values = actions_after.clone();
            overrides.agent_chat_values = agent_chat_after.clone();
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

fn apply_optional_copy_value(
    values: &mut BTreeMap<CopyControlId, String>,
    id: CopyControlId,
    value: Option<String>,
) {
    if let Some(value) = value {
        values.insert(id, value);
    } else {
        values.remove(&id);
    }
}

fn apply_optional_actions_value(
    values: &mut BTreeMap<ActionsPopupKnobId, StyleValue>,
    id: ActionsPopupKnobId,
    value: Option<StyleValue>,
) {
    if let Some(value) = value {
        values.insert(id, value);
    } else {
        values.remove(&id);
    }
}

fn apply_optional_agent_chat_value(
    values: &mut BTreeMap<AgentChatKnobId, StyleValue>,
    id: AgentChatKnobId,
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
        HistoryEntry::CopySingle { id, .. } => {
            format!("{action}:{} generation={generation}", id.as_str())
        }
        HistoryEntry::ActionsSingle { id, .. } => {
            format!("{action}:{} generation={generation}", id.as_str())
        }
        HistoryEntry::AgentChatSingle { id, .. } => {
            format!("{action}:{} generation={generation}", id.as_str())
        }
        HistoryEntry::Snapshot {
            before,
            copy_before,
            actions_before,
            agent_chat_before,
            after,
            copy_after,
            actions_after,
            agent_chat_after,
        } => {
            let before = before
                .len()
                .saturating_add(copy_before.len())
                .saturating_add(actions_before.len())
                .saturating_add(agent_chat_before.len());
            let after = after
                .len()
                .saturating_add(copy_after.len())
                .saturating_add(actions_after.len())
                .saturating_add(agent_chat_after.len());
            format!("{action}:all before={before} after={after} generation={generation}")
        }
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

pub fn set_copy_from_devtools(control: &str, value: &str) -> anyhow::Result<String> {
    let id = copy_control_id_from_str(control)
        .ok_or_else(|| anyhow::anyhow!("unknown dev style copy control '{control}'"))?;
    let change = set_copy_value(id, value.to_string())
        .ok_or_else(|| anyhow::anyhow!("unknown dev style copy control '{control}'"))?;
    Ok(format!("{}={}", id.as_str(), change.applied))
}

pub fn set_actions_number_from_devtools(control: &str, value: &str) -> anyhow::Result<String> {
    let id = actions_popup_knob_id_from_str(control)
        .ok_or_else(|| anyhow::anyhow!("unknown dev style actions control '{control}'"))?;
    let parsed = value
        .trim()
        .trim_end_matches("px")
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .map_err(|_| anyhow::anyhow!("invalid numeric value '{value}'"))?;
    let change = set_actions_popup_value(id, StyleValue::Number(parsed))
        .ok_or_else(|| anyhow::anyhow!("unknown dev style actions control '{control}'"))?;
    let StyleValue::Number(applied) = change.applied;
    Ok(format!("{}={applied}", id.as_str()))
}

pub fn set_agent_chat_number_from_devtools(control: &str, value: &str) -> anyhow::Result<String> {
    let id = agent_chat_knob_id_from_str(control)
        .ok_or_else(|| anyhow::anyhow!("unknown dev style agent chat control '{control}'"))?;
    let parsed = value
        .trim()
        .trim_end_matches("px")
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .map_err(|_| anyhow::anyhow!("invalid numeric value '{value}'"))?;
    let change = set_agent_chat_value(id, StyleValue::Number(parsed))
        .ok_or_else(|| anyhow::anyhow!("unknown dev style agent chat control '{control}'"))?;
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

pub fn apply_to_actions_popup_def(mut def: ActionsPopupThemeDef) -> ActionsPopupThemeDef {
    let guard = store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for knob in ACTIONS_POPUP_KNOBS {
        if let Some(value) = guard.actions_values.get(&knob.id).copied() {
            (knob.apply)(&mut def, value);
        }
    }
    def
}

pub fn effective_agent_chat_style() -> AgentChatStyleDef {
    let mut def = base_agent_chat_style();
    let guard = store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for knob in AGENT_CHAT_KNOBS {
        if let Some(value) = guard.agent_chat_values.get(&knob.id).copied() {
            (knob.apply)(&mut def, value);
        }
    }
    def
}
