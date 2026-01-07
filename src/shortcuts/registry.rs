//! Deterministic shortcut registry with Vec storage.
//!
//! Uses Vec for deterministic iteration order and HashMap for O(1) lookup.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use super::context::ShortcutContext;
use super::types::Shortcut;

/// Source of a shortcut binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindingSource {
    Builtin,
    Script,
}

/// Scope in which a shortcut operates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    #[default]
    App,
    Global,
}

/// Category for organizing shortcuts in UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShortcutCategory {
    Navigation,
    Actions,
    Edit,
    View,
    Scripts,
    System,
}

/// A shortcut binding with metadata.
#[derive(Clone, Debug)]
pub struct ShortcutBinding {
    pub id: String,
    pub name: String,
    pub default_shortcut: Shortcut,
    pub context: ShortcutContext,
    pub scope: ShortcutScope,
    pub category: ShortcutCategory,
    pub source: BindingSource,
    pub customizable: bool,
}

impl ShortcutBinding {
    pub fn builtin(
        id: impl Into<String>,
        name: impl Into<String>,
        shortcut: Shortcut,
        context: ShortcutContext,
        category: ShortcutCategory,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            default_shortcut: shortcut,
            context,
            scope: ShortcutScope::App,
            category,
            source: BindingSource::Builtin,
            customizable: true,
        }
    }

    pub fn script(id: impl Into<String>, name: impl Into<String>, shortcut: Shortcut) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            default_shortcut: shortcut,
            context: ShortcutContext::Global,
            scope: ShortcutScope::App,
            category: ShortcutCategory::Scripts,
            source: BindingSource::Script,
            customizable: false,
        }
    }

    pub fn non_customizable(mut self) -> Self {
        self.customizable = false;
        self
    }

    pub fn global(mut self) -> Self {
        self.scope = ShortcutScope::Global;
        self
    }
}

/// Central registry of all keyboard shortcuts.
pub struct ShortcutRegistry {
    bindings: Vec<ShortcutBinding>,
    id_to_index: HashMap<String, usize>,
    user_overrides: HashMap<String, Option<Shortcut>>,
    disabled: HashSet<String>,
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutRegistry {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            id_to_index: HashMap::new(),
            user_overrides: HashMap::new(),
            disabled: HashSet::new(),
        }
    }

    pub fn register(&mut self, binding: ShortcutBinding) {
        let id = binding.id.clone();
        if let Some(&existing_index) = self.id_to_index.get(&id) {
            self.bindings[existing_index] = binding;
        } else {
            let index = self.bindings.len();
            self.bindings.push(binding);
            self.id_to_index.insert(id, index);
        }
    }

    pub fn unregister(&mut self, id: &str) {
        self.disabled.insert(id.to_string());
    }

    pub fn get(&self, id: &str) -> Option<&ShortcutBinding> {
        self.id_to_index.get(id).and_then(|&i| self.bindings.get(i))
    }

    pub fn get_shortcut(&self, id: &str) -> Option<Shortcut> {
        if self.disabled.contains(id) {
            return None;
        }
        if let Some(override_opt) = self.user_overrides.get(id) {
            return override_opt.clone();
        }
        self.get(id).map(|b| b.default_shortcut.clone())
    }

    pub fn set_override(&mut self, id: &str, shortcut: Option<Shortcut>) {
        if shortcut.is_none() {
            self.disabled.insert(id.to_string());
        } else {
            self.disabled.remove(id);
        }
        self.user_overrides.insert(id.to_string(), shortcut);
    }

    pub fn clear_override(&mut self, id: &str) {
        self.user_overrides.remove(id);
        self.disabled.remove(id);
    }

    pub fn is_disabled(&self, id: &str) -> bool {
        self.disabled.contains(id)
    }

    /// Find a matching binding for a keystroke in the given context stack.
    pub fn find_match(
        &self,
        keystroke: &gpui::Keystroke,
        contexts: &[ShortcutContext],
    ) -> Option<&str> {
        for context in contexts {
            for binding in &self.bindings {
                if binding.context != *context || self.disabled.contains(&binding.id) {
                    continue;
                }
                let shortcut = if let Some(override_opt) = self.user_overrides.get(&binding.id) {
                    match override_opt {
                        Some(s) => s.clone(),
                        None => continue,
                    }
                } else {
                    binding.default_shortcut.clone()
                };
                if shortcut.matches_keystroke(keystroke) {
                    return Some(&binding.id);
                }
            }
        }
        None
    }

    pub fn bindings(&self) -> &[ShortcutBinding] {
        &self.bindings
    }

    pub fn bindings_by_category(&self, category: ShortcutCategory) -> Vec<&ShortcutBinding> {
        self.bindings
            .iter()
            .filter(|b| b.category == category && !self.disabled.contains(&b.id))
            .collect()
    }

    pub fn bindings_by_context(&self, context: ShortcutContext) -> Vec<&ShortcutBinding> {
        self.bindings
            .iter()
            .filter(|b| b.context == context && !self.disabled.contains(&b.id))
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.bindings
            .iter()
            .filter(|b| !self.disabled.contains(&b.id))
            .count()
    }
}
