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
use super::confirm_modal_catalog::{
    base_confirm_modal_style, confirm_modal_knob_by_id, confirm_modal_knob_id_from_str,
    ConfirmModalKnobId, ConfirmModalStyleDef, CONFIRM_MODAL_KNOBS,
};
use super::copy_catalog::{copy_control_by_id, copy_control_id_from_str, CopyControlId};
use super::theme_catalog::{
    format_theme_color_hex, theme_color_knob_by_id, theme_color_knob_id_from_str, ThemeColorKnobId,
    THEME_COLOR_KNOBS,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedThemeColorChange {
    pub generation: u64,
    pub previous: Option<u32>,
    pub requested: u32,
    pub applied: u32,
}

/// One override map per channel. Every channel shares the same lifecycle
/// (set/reset/current plus the interleaved undo/redo history), so the maps
/// live together and snapshot history entries clone the whole struct.
#[derive(Debug, Clone, Default)]
struct OverrideMaps {
    values: BTreeMap<StyleKnobId, StyleValue>,
    copy_values: BTreeMap<CopyControlId, String>,
    actions_values: BTreeMap<ActionsPopupKnobId, StyleValue>,
    agent_chat_values: BTreeMap<AgentChatKnobId, StyleValue>,
    confirm_modal_values: BTreeMap<ConfirmModalKnobId, StyleValue>,
    theme_color_values: BTreeMap<ThemeColorKnobId, u32>,
}

impl OverrideMaps {
    fn total_len(&self) -> usize {
        self.values
            .len()
            .saturating_add(self.copy_values.len())
            .saturating_add(self.actions_values.len())
            .saturating_add(self.agent_chat_values.len())
            .saturating_add(self.confirm_modal_values.len())
            .saturating_add(self.theme_color_values.len())
    }
}

#[derive(Debug, Clone, Default)]
struct RuntimeStyleOverrides {
    generation: u64,
    maps: OverrideMaps,
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
    ConfirmModalSingle {
        id: ConfirmModalKnobId,
        before: Option<StyleValue>,
        after: Option<StyleValue>,
    },
    ThemeColorSingle {
        id: ThemeColorKnobId,
        before: Option<u32>,
        after: Option<u32>,
    },
    Snapshot {
        before: Box<OverrideMaps>,
        after: Box<OverrideMaps>,
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

fn read_store() -> std::sync::RwLockReadGuard<'static, RuntimeStyleOverrides> {
    store()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write_store() -> std::sync::RwLockWriteGuard<'static, RuntimeStyleOverrides> {
    store()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn generation() -> u64 {
    read_store().generation
}

pub fn history_state() -> StyleHistoryState {
    let guard = read_store();
    StyleHistoryState {
        can_undo: !guard.undo_stack.is_empty(),
        can_redo: !guard.redo_stack.is_empty(),
        override_count: guard.maps.total_len(),
        generation: guard.generation,
    }
}

/// Generate the set/reset/current trio for one override channel.
///
/// Every channel follows the same lifecycle: clamp the requested value, store
/// it in the channel map, push a single-entry undo record, clear the redo
/// stack, and bump the shared generation counter. Only the value type, the
/// clamp rule, and the base-value source differ per channel.
macro_rules! override_channel {
    (
        change: $change_ty:ident,
        value: $value_ty:ty,
        id: $id_ty:ty,
        field: $field:ident,
        variant: $variant:ident,
        lookup: $lookup:path,
        clamp: |$set_knob:pat_param, $requested:ident| $clamp:expr,
        base: |$reset_knob:pat_param| $base:expr,
        set: $set_fn:ident,
        reset: $reset_fn:ident,
        current: $current_fn:ident $(,)?
    ) => {
        #[allow(clippy::clone_on_copy)]
        pub fn $set_fn(id: $id_ty, $requested: $value_ty) -> Option<$change_ty> {
            let $set_knob = $lookup(id)?;
            let applied: $value_ty = $clamp;
            let mut guard = write_store();
            let previous = guard.maps.$field.insert(id, applied.clone());
            guard.undo_stack.push(HistoryEntry::$variant {
                id,
                before: previous.clone(),
                after: Some(applied.clone()),
            });
            guard.redo_stack.clear();
            guard.generation = guard.generation.saturating_add(1);
            Some($change_ty {
                generation: guard.generation,
                previous,
                requested: $requested,
                applied,
            })
        }

        #[allow(clippy::clone_on_copy)]
        pub fn $reset_fn(id: $id_ty) -> Option<$change_ty> {
            let $reset_knob = $lookup(id)?;
            let base_value: $value_ty = $base;
            let mut guard = write_store();
            let previous = guard.maps.$field.remove(&id);
            guard.undo_stack.push(HistoryEntry::$variant {
                id,
                before: previous.clone(),
                after: None,
            });
            guard.redo_stack.clear();
            guard.generation = guard.generation.saturating_add(1);
            Some($change_ty {
                generation: guard.generation,
                previous,
                requested: base_value.clone(),
                applied: base_value,
            })
        }

        pub fn $current_fn(id: $id_ty) -> Option<$value_ty> {
            read_store().maps.$field.get(&id).cloned()
        }
    };
}

override_channel!(
    change: AppliedStyleChange,
    value: StyleValue,
    id: StyleKnobId,
    field: values,
    variant: Single,
    lookup: knob_by_id,
    clamp: |knob, requested| knob.clamp_value(requested),
    base: |knob| (knob.get)(&crate::designs::current_main_menu_theme().base_def()),
    set: set_value,
    reset: reset_value,
    current: current_value,
);

override_channel!(
    change: AppliedCopyChange,
    value: String,
    id: CopyControlId,
    field: copy_values,
    variant: CopySingle,
    lookup: copy_control_by_id,
    clamp: |_control, requested| requested.clone(),
    base: |control| (control.base)(),
    set: set_copy_value,
    reset: reset_copy_value,
    current: current_copy_value,
);

override_channel!(
    change: AppliedStyleChange,
    value: StyleValue,
    id: ActionsPopupKnobId,
    field: actions_values,
    variant: ActionsSingle,
    lookup: actions_popup_knob_by_id,
    clamp: |knob, requested| knob.clamp_value(requested),
    base: |knob| (knob.get)(&crate::designs::base_actions_popup_theme()),
    set: set_actions_popup_value,
    reset: reset_actions_popup_value,
    current: current_actions_popup_value,
);

override_channel!(
    change: AppliedStyleChange,
    value: StyleValue,
    id: AgentChatKnobId,
    field: agent_chat_values,
    variant: AgentChatSingle,
    lookup: agent_chat_knob_by_id,
    clamp: |knob, requested| knob.clamp_value(requested),
    base: |knob| (knob.get)(&base_agent_chat_style()),
    set: set_agent_chat_value,
    reset: reset_agent_chat_value,
    current: current_agent_chat_value,
);

override_channel!(
    change: AppliedStyleChange,
    value: StyleValue,
    id: ConfirmModalKnobId,
    field: confirm_modal_values,
    variant: ConfirmModalSingle,
    lookup: confirm_modal_knob_by_id,
    clamp: |knob, requested| knob.clamp_value(requested),
    base: |knob| (knob.get)(&base_confirm_modal_style()),
    set: set_confirm_modal_value,
    reset: reset_confirm_modal_value,
    current: current_confirm_modal_value,
);

// Theme color resets read the base value fresh from disk so the reported
// value never includes the override that was just removed.
override_channel!(
    change: AppliedThemeColorChange,
    value: u32,
    id: ThemeColorKnobId,
    field: theme_color_values,
    variant: ThemeColorSingle,
    lookup: theme_color_knob_by_id,
    clamp: |_knob, requested| requested & 0xFF_FF_FF,
    base: |knob| (knob.get)(&crate::theme::load_theme()),
    set: set_theme_color_value,
    reset: reset_theme_color_value,
    current: current_theme_color_value,
);

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

pub fn has_theme_color_overrides() -> bool {
    !read_store().maps.theme_color_values.is_empty()
}

/// Drop every theme color override without touching other channels.
///
/// Used after "Save theme to disk": the overrides have just been baked into
/// theme.json, so the channel must be emptied for the inspector to show the
/// saved values as base instead of double-reporting them as overridden.
/// Recorded as a snapshot history entry so undo/redo stays coherent.
pub fn clear_theme_color_values() -> u64 {
    let mut guard = write_store();
    if guard.maps.theme_color_values.is_empty() {
        return guard.generation;
    }
    let before = guard.maps.clone();
    let mut after = guard.maps.clone();
    after.theme_color_values.clear();
    guard.maps.theme_color_values.clear();
    guard.undo_stack.push(HistoryEntry::Snapshot {
        before: Box::new(before),
        after: Box::new(after),
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    guard.generation
}

pub fn reset_all() -> u64 {
    let mut guard = write_store();
    let before = guard.maps.clone();
    guard.maps = OverrideMaps::default();
    guard.undo_stack.push(HistoryEntry::Snapshot {
        before: Box::new(before),
        after: Box::new(OverrideMaps::default()),
    });
    guard.redo_stack.clear();
    guard.generation = guard.generation.saturating_add(1);
    guard.generation
}

pub fn undo_last() -> Option<String> {
    let mut guard = write_store();
    let entry = guard.undo_stack.pop()?;
    apply_history_entry(&mut guard, &entry, HistoryDirection::Undo);
    guard.redo_stack.push(entry.clone());
    guard.generation = guard.generation.saturating_add(1);
    Some(format_history_result("undo", &entry, guard.generation))
}

pub fn redo_last() -> Option<String> {
    let mut guard = write_store();
    let entry = guard.redo_stack.pop()?;
    apply_history_entry(&mut guard, &entry, HistoryDirection::Redo);
    guard.undo_stack.push(entry.clone());
    guard.generation = guard.generation.saturating_add(1);
    Some(format_history_result("redo", &entry, guard.generation))
}

pub fn copy_override_count() -> usize {
    read_store().maps.copy_values.len()
}

pub fn actions_popup_override_count() -> usize {
    read_store().maps.actions_values.len()
}

pub fn agent_chat_override_count() -> usize {
    read_store().maps.agent_chat_values.len()
}

pub fn confirm_modal_override_count() -> usize {
    read_store().maps.confirm_modal_values.len()
}

pub fn theme_color_override_count() -> usize {
    read_store().maps.theme_color_values.len()
}

#[derive(Debug, Clone, Copy)]
enum HistoryDirection {
    Undo,
    Redo,
}

impl HistoryDirection {
    /// Pick the map state this direction restores: `before` for undo,
    /// `after` for redo.
    fn pick<'a, T>(self, before: &'a T, after: &'a T) -> &'a T {
        match self {
            HistoryDirection::Undo => before,
            HistoryDirection::Redo => after,
        }
    }
}

fn apply_history_entry(
    overrides: &mut RuntimeStyleOverrides,
    entry: &HistoryEntry,
    direction: HistoryDirection,
) {
    match entry {
        HistoryEntry::Single { id, before, after } => {
            apply_optional(
                &mut overrides.maps.values,
                *id,
                *direction.pick(before, after),
            );
        }
        HistoryEntry::CopySingle { id, before, after } => {
            apply_optional(
                &mut overrides.maps.copy_values,
                *id,
                direction.pick(before, after).clone(),
            );
        }
        HistoryEntry::ActionsSingle { id, before, after } => {
            apply_optional(
                &mut overrides.maps.actions_values,
                *id,
                *direction.pick(before, after),
            );
        }
        HistoryEntry::AgentChatSingle { id, before, after } => {
            apply_optional(
                &mut overrides.maps.agent_chat_values,
                *id,
                *direction.pick(before, after),
            );
        }
        HistoryEntry::ConfirmModalSingle { id, before, after } => {
            apply_optional(
                &mut overrides.maps.confirm_modal_values,
                *id,
                *direction.pick(before, after),
            );
        }
        HistoryEntry::ThemeColorSingle { id, before, after } => {
            apply_optional(
                &mut overrides.maps.theme_color_values,
                *id,
                *direction.pick(before, after),
            );
        }
        HistoryEntry::Snapshot { before, after } => {
            overrides.maps = direction.pick(before, after).as_ref().clone();
        }
    }
}

fn apply_optional<K: Ord, V>(values: &mut BTreeMap<K, V>, id: K, value: Option<V>) {
    if let Some(value) = value {
        values.insert(id, value);
    } else {
        values.remove(&id);
    }
}

fn format_history_result(action: &str, entry: &HistoryEntry, generation: u64) -> String {
    let id = match entry {
        HistoryEntry::Single { id, .. } => id.as_str(),
        HistoryEntry::CopySingle { id, .. } => id.as_str(),
        HistoryEntry::ActionsSingle { id, .. } => id.as_str(),
        HistoryEntry::AgentChatSingle { id, .. } => id.as_str(),
        HistoryEntry::ConfirmModalSingle { id, .. } => id.as_str(),
        HistoryEntry::ThemeColorSingle { id, .. } => id.as_str(),
        HistoryEntry::Snapshot { before, after } => {
            return format!(
                "{action}:all before={} after={} generation={generation}",
                before.total_len(),
                after.total_len()
            );
        }
    };
    format!("{action}:{id} generation={generation}")
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

/// Shared body for the per-channel numeric devtools setters. Accepts values
/// with a trailing `px` or `%` suffix, mirroring how the sliders report them.
fn set_channel_number_from_devtools<Id: Copy>(
    label: &str,
    control: &str,
    value: &str,
    id_from_str: fn(&str) -> Option<Id>,
    set: fn(Id, StyleValue) -> Option<AppliedStyleChange>,
    id_as_str: fn(Id) -> &'static str,
) -> anyhow::Result<String> {
    let id = id_from_str(control)
        .ok_or_else(|| anyhow::anyhow!("unknown dev style {label} control '{control}'"))?;
    let parsed = value
        .trim()
        .trim_end_matches("px")
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .map_err(|_| anyhow::anyhow!("invalid numeric value '{value}'"))?;
    let change = set(id, StyleValue::Number(parsed))
        .ok_or_else(|| anyhow::anyhow!("unknown dev style {label} control '{control}'"))?;
    let StyleValue::Number(applied) = change.applied;
    Ok(format!("{}={applied}", id_as_str(id)))
}

pub fn set_actions_number_from_devtools(control: &str, value: &str) -> anyhow::Result<String> {
    set_channel_number_from_devtools(
        "actions",
        control,
        value,
        actions_popup_knob_id_from_str,
        set_actions_popup_value,
        ActionsPopupKnobId::as_str,
    )
}

pub fn set_agent_chat_number_from_devtools(control: &str, value: &str) -> anyhow::Result<String> {
    set_channel_number_from_devtools(
        "agent chat",
        control,
        value,
        agent_chat_knob_id_from_str,
        set_agent_chat_value,
        AgentChatKnobId::as_str,
    )
}

pub fn set_confirm_modal_number_from_devtools(
    control: &str,
    value: &str,
) -> anyhow::Result<String> {
    set_channel_number_from_devtools(
        "confirm modal",
        control,
        value,
        confirm_modal_knob_id_from_str,
        set_confirm_modal_value,
        ConfirmModalKnobId::as_str,
    )
}

pub fn set_theme_color_from_devtools(control: &str, value: &str) -> anyhow::Result<String> {
    let id = theme_color_knob_id_from_str(control)
        .ok_or_else(|| anyhow::anyhow!("unknown dev style theme color control '{control}'"))?;
    let parsed = crate::theme::hex_color::hex_color_serde::parse_color_string(value)
        .map_err(|error| anyhow::anyhow!("invalid color value '{value}': {error}"))?;
    let change = set_theme_color_value(id, parsed)
        .ok_or_else(|| anyhow::anyhow!("unknown dev style theme color control '{control}'"))?;
    Ok(format!(
        "{}={}",
        id.as_str(),
        format_theme_color_hex(change.applied)
    ))
}

/// Apply every live theme color override on top of a freshly loaded theme.
///
/// Called from `theme::reload_theme_cache()` so the override layer survives any
/// cache reload (file watcher, appearance flip, devtools) while remaining a
/// no-op when the channel is empty.
pub fn apply_to_theme(mut theme: crate::theme::Theme) -> crate::theme::Theme {
    let guard = read_store();
    if guard.maps.theme_color_values.is_empty() {
        return theme;
    }
    for knob in THEME_COLOR_KNOBS {
        if let Some(value) = guard.maps.theme_color_values.get(&knob.id).copied() {
            (knob.apply)(&mut theme, value);
        }
    }
    theme
}

pub fn apply_to_main_menu_def(mut def: MainMenuThemeDef) -> MainMenuThemeDef {
    let guard = read_store();
    for knob in STYLE_KNOBS {
        if let Some(value) = guard.maps.values.get(&knob.id).copied() {
            (knob.apply)(&mut def, value);
        }
    }
    def
}

pub fn apply_to_actions_popup_def(mut def: ActionsPopupThemeDef) -> ActionsPopupThemeDef {
    let guard = read_store();
    for knob in ACTIONS_POPUP_KNOBS {
        if let Some(value) = guard.maps.actions_values.get(&knob.id).copied() {
            (knob.apply)(&mut def, value);
        }
    }
    def
}

pub fn effective_agent_chat_style() -> AgentChatStyleDef {
    let mut def = base_agent_chat_style();
    let guard = read_store();
    for knob in AGENT_CHAT_KNOBS {
        if let Some(value) = guard.maps.agent_chat_values.get(&knob.id).copied() {
            (knob.apply)(&mut def, value);
        }
    }
    def
}

pub fn effective_confirm_modal_style() -> ConfirmModalStyleDef {
    let mut def = base_confirm_modal_style();
    let guard = read_store();
    for knob in CONFIRM_MODAL_KNOBS {
        if let Some(value) = guard.maps.confirm_modal_values.get(&knob.id).copied() {
            (knob.apply)(&mut def, value);
        }
    }
    def
}
