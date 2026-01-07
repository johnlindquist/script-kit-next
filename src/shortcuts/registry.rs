//! Deterministic shortcut registry with Vec storage.
//!
//! Uses Vec for deterministic iteration order and HashMap for O(1) lookup.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use super::context::ShortcutContext;
use super::types::Shortcut;

/// Source of a shortcut binding.
///
/// Priority order (highest first): user_override > Builtin > Script
/// This ensures built-in shortcuts aren't silently stolen by scripts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BindingSource {
    /// Built-in app shortcut (highest priority)
    Builtin = 0,
    /// Script-defined shortcut (lower priority)
    Script = 1,
}

impl BindingSource {
    /// Get the priority value (lower = higher priority).
    pub fn priority(&self) -> u8 {
        match self {
            Self::Builtin => 0,
            Self::Script => 1,
        }
    }
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
    ///
    /// Within each context, priority order is:
    /// 1. User overrides (always win if present)
    /// 2. Builtins (win over scripts)
    /// 3. Scripts (lowest priority)
    pub fn find_match(
        &self,
        keystroke: &gpui::Keystroke,
        contexts: &[ShortcutContext],
    ) -> Option<&str> {
        for context in contexts {
            // Collect all matches in this context
            let mut matches: Vec<(&ShortcutBinding, bool)> = Vec::new();

            for binding in &self.bindings {
                if binding.context != *context || self.disabled.contains(&binding.id) {
                    continue;
                }

                let has_user_override = self.user_overrides.contains_key(&binding.id);
                let shortcut = if let Some(override_opt) = self.user_overrides.get(&binding.id) {
                    match override_opt {
                        Some(s) => s.clone(),
                        None => continue, // Disabled via override
                    }
                } else {
                    binding.default_shortcut.clone()
                };

                if shortcut.matches_keystroke(keystroke) {
                    matches.push((binding, has_user_override));
                }
            }

            // If we have matches, return the highest priority one
            if !matches.is_empty() {
                // Sort by: user_override first, then source priority, then registration order
                matches.sort_by(|(a, a_override), (b, b_override)| {
                    // User override always wins
                    match (a_override, b_override) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => {
                            // Same override status: compare source priority
                            a.source.priority().cmp(&b.source.priority())
                        }
                    }
                });

                return Some(&matches[0].0.id);
            }
        }
        None
    }

    /// Check if a script shortcut would conflict with a builtin.
    ///
    /// Returns the ID of the conflicting builtin if one exists.
    pub fn check_builtin_conflict(
        &self,
        shortcut: &Shortcut,
        context: ShortcutContext,
    ) -> Option<&str> {
        for binding in &self.bindings {
            if binding.source != BindingSource::Builtin {
                continue;
            }
            if binding.context != context && binding.context != ShortcutContext::Global {
                continue;
            }
            if self.disabled.contains(&binding.id) {
                continue;
            }

            let effective = self
                .user_overrides
                .get(&binding.id)
                .cloned()
                .unwrap_or_else(|| Some(binding.default_shortcut.clone()));

            if let Some(builtin_shortcut) = effective {
                if builtin_shortcut == *shortcut {
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

    /// Find all conflicts in the registry.
    ///
    /// Returns a list of conflicts, each containing:
    /// - The type of conflict
    /// - The IDs of the conflicting bindings
    /// - The conflicting shortcut
    pub fn find_conflicts(&self) -> Vec<ShortcutConflict> {
        let mut conflicts = Vec::new();

        // Build a map of effective shortcuts to bindings
        let mut shortcut_map: HashMap<(String, ShortcutContext), Vec<&ShortcutBinding>> =
            HashMap::new();

        for binding in &self.bindings {
            if self.disabled.contains(&binding.id) {
                continue;
            }

            let effective = self
                .user_overrides
                .get(&binding.id)
                .cloned()
                .unwrap_or_else(|| Some(binding.default_shortcut.clone()));

            if let Some(shortcut) = effective {
                let key = (shortcut.to_canonical_string(), binding.context);
                shortcut_map.entry(key).or_default().push(binding);
            }
        }

        // Check for hard conflicts (same shortcut + same context)
        for ((shortcut_str, context), bindings) in &shortcut_map {
            if bindings.len() > 1 {
                // Sort by priority to determine winner/loser
                let mut sorted: Vec<_> = bindings.iter().collect();
                sorted.sort_by_key(|b| b.source.priority());

                let winner = sorted[0];
                for loser in &sorted[1..] {
                    let conflict_type = if winner.source == loser.source {
                        ConflictType::Hard
                    } else {
                        // Different sources: higher priority shadows lower
                        ConflictType::Shadowed
                    };

                    conflicts.push(ShortcutConflict {
                        conflict_type,
                        winner_id: winner.id.clone(),
                        loser_id: loser.id.clone(),
                        shortcut: shortcut_str.clone(),
                        context: *context,
                    });
                }
            }
        }

        // Check for shadowing across context specificity
        for binding in &self.bindings {
            if self.disabled.contains(&binding.id) {
                continue;
            }

            let effective = self
                .user_overrides
                .get(&binding.id)
                .cloned()
                .unwrap_or_else(|| Some(binding.default_shortcut.clone()));

            if let Some(shortcut) = effective {
                let shortcut_str = shortcut.to_canonical_string();

                // Check if this binding is shadowed by a more specific context
                for other in &self.bindings {
                    if other.id == binding.id || self.disabled.contains(&other.id) {
                        continue;
                    }

                    // Skip if not same shortcut
                    let other_effective = self
                        .user_overrides
                        .get(&other.id)
                        .cloned()
                        .unwrap_or_else(|| Some(other.default_shortcut.clone()));

                    let other_shortcut = match other_effective {
                        Some(s) => s,
                        None => continue,
                    };

                    if other_shortcut.to_canonical_string() != shortcut_str {
                        continue;
                    }

                    // Check context specificity: other shadows binding if other is more specific
                    if other.context.specificity() > binding.context.specificity()
                        && binding.context.contains(&other.context)
                    {
                        // Already covered by same-context check
                        continue;
                    }
                }
            }
        }

        // Check for OS-reserved shortcuts (unreachable)
        let os_reserved = Self::get_os_reserved_shortcuts();
        for binding in &self.bindings {
            if self.disabled.contains(&binding.id) {
                continue;
            }

            let effective = self
                .user_overrides
                .get(&binding.id)
                .cloned()
                .unwrap_or_else(|| Some(binding.default_shortcut.clone()));

            if let Some(shortcut) = effective {
                let canonical = shortcut.to_canonical_string();
                if os_reserved.contains(&canonical.as_str()) {
                    conflicts.push(ShortcutConflict {
                        conflict_type: ConflictType::Unreachable,
                        winner_id: "system".to_string(),
                        loser_id: binding.id.clone(),
                        shortcut: canonical,
                        context: binding.context,
                    });
                }
            }
        }

        conflicts
    }

    /// Get list of OS-reserved shortcuts that cannot be overridden.
    ///
    /// These are typically system-level shortcuts that apps cannot intercept.
    fn get_os_reserved_shortcuts() -> HashSet<&'static str> {
        let mut reserved = HashSet::new();

        #[cfg(target_os = "macos")]
        {
            // macOS system shortcuts
            reserved.insert("cmd+tab"); // App switcher
            reserved.insert("cmd+shift+tab"); // Reverse app switcher
            reserved.insert("cmd+space"); // Spotlight (often)
            reserved.insert("cmd+ctrl+q"); // Lock screen
            reserved.insert("cmd+shift+3"); // Screenshot full
            reserved.insert("cmd+shift+4"); // Screenshot selection
            reserved.insert("cmd+shift+5"); // Screenshot/record options
            reserved.insert("ctrl+up"); // Mission Control
            reserved.insert("ctrl+down"); // App windows
            reserved.insert("ctrl+left"); // Move space left
            reserved.insert("ctrl+right"); // Move space right
        }

        #[cfg(target_os = "windows")]
        {
            // Windows system shortcuts
            reserved.insert("cmd+tab"); // Alt+Tab (cmd maps to Win)
            reserved.insert("cmd+d"); // Show desktop
            reserved.insert("cmd+l"); // Lock
            reserved.insert("cmd+e"); // File Explorer
            reserved.insert("ctrl+alt+delete"); // Security options
        }

        #[cfg(target_os = "linux")]
        {
            // Common Linux shortcuts (vary by DE)
            reserved.insert("alt+tab"); // Window switcher
            reserved.insert("cmd+tab"); // Super+Tab
            reserved.insert("ctrl+alt+t"); // Terminal (common)
            reserved.insert("ctrl+alt+delete"); // System
        }

        reserved
    }

    /// Get conflicts for a specific binding.
    pub fn conflicts_for(&self, id: &str) -> Vec<ShortcutConflict> {
        self.find_conflicts()
            .into_iter()
            .filter(|c| c.winner_id == id || c.loser_id == id)
            .collect()
    }

    /// Check if adding a shortcut would create conflicts.
    ///
    /// Returns the conflicts that would be created if this shortcut were added.
    pub fn would_conflict(
        &self,
        shortcut: &Shortcut,
        context: ShortcutContext,
        source: BindingSource,
    ) -> Vec<PotentialConflict> {
        let mut conflicts = Vec::new();
        let canonical = shortcut.to_canonical_string();

        // Check OS reserved
        let os_reserved = Self::get_os_reserved_shortcuts();
        if os_reserved.contains(&canonical.as_str()) {
            conflicts.push(PotentialConflict {
                conflict_type: ConflictType::Unreachable,
                existing_id: "system".to_string(),
                existing_name: "System Shortcut".to_string(),
            });
        }

        // Check existing bindings
        for binding in &self.bindings {
            if self.disabled.contains(&binding.id) {
                continue;
            }

            let effective = self
                .user_overrides
                .get(&binding.id)
                .cloned()
                .unwrap_or_else(|| Some(binding.default_shortcut.clone()));

            if let Some(existing_shortcut) = effective {
                if existing_shortcut.to_canonical_string() != canonical {
                    continue;
                }

                // Same shortcut - check context overlap
                let contexts_overlap = binding.context == context
                    || binding.context == ShortcutContext::Global
                    || context == ShortcutContext::Global
                    || binding.context.contains(&context)
                    || context.contains(&binding.context);

                if contexts_overlap {
                    let conflict_type = if binding.context == context
                        && source.priority() == binding.source.priority()
                    {
                        ConflictType::Hard // Same context + same priority = hard conflict
                    } else {
                        ConflictType::Shadowed // Different priority or context = shadowing
                    };

                    conflicts.push(PotentialConflict {
                        conflict_type,
                        existing_id: binding.id.clone(),
                        existing_name: binding.name.clone(),
                    });
                }
            }
        }

        conflicts
    }
}

/// Type of shortcut conflict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictType {
    /// Same shortcut + same context + same priority.
    /// One must be changed or disabled.
    Hard,
    /// Shortcut exists but is intentionally shadowed by context specificity
    /// or source priority. The more specific/higher priority binding wins.
    Shadowed,
    /// Shortcut is reserved by the OS and cannot be overridden.
    Unreachable,
}

/// A conflict between two shortcut bindings.
#[derive(Clone, Debug)]
pub struct ShortcutConflict {
    pub conflict_type: ConflictType,
    pub winner_id: String,
    pub loser_id: String,
    pub shortcut: String,
    pub context: ShortcutContext,
}

/// A potential conflict that would occur if a shortcut were added.
#[derive(Clone, Debug)]
pub struct PotentialConflict {
    pub conflict_type: ConflictType,
    pub existing_id: String,
    pub existing_name: String,
}
