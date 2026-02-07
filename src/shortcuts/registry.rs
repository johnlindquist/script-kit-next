//! Deterministic shortcut registry with Vec storage.
//!
//! Uses Vec for deterministic iteration order and HashMap for O(1) lookup.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use super::context::ShortcutContext;
use super::types::{ConflictInfo, Shortcut};

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
    effective_shortcut_index: HashMap<ShortcutContext, HashMap<String, usize>>,
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
            effective_shortcut_index: HashMap::new(),
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
        self.rebuild_effective_shortcut_index();
    }

    pub fn unregister(&mut self, id: &str) {
        self.disabled.insert(id.to_string());
        self.rebuild_effective_shortcut_index();
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
        self.rebuild_effective_shortcut_index();
    }

    pub fn clear_override(&mut self, id: &str) {
        self.user_overrides.remove(id);
        self.disabled.remove(id);
        self.rebuild_effective_shortcut_index();
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
        let canonical_keystroke = Self::canonical_keystroke(keystroke);

        for context in contexts {
            if let Some(binding_index) = self
                .effective_shortcut_index
                .get(context)
                .and_then(|by_shortcut| by_shortcut.get(&canonical_keystroke))
            {
                if let Some(binding) = self.bindings.get(*binding_index) {
                    return Some(&binding.id);
                }
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

    /// Get all user overrides as a map of binding_id -> Option<Shortcut>.
    ///
    /// Returns None for disabled shortcuts, Some(shortcut) for overridden shortcuts.
    pub fn get_overrides(&self) -> &HashMap<String, Option<Shortcut>> {
        &self.user_overrides
    }

    /// Export user overrides as canonical strings for persistence.
    ///
    /// Returns a map of binding_id -> Option<String> where:
    /// - Some(string) = override shortcut as canonical string
    /// - None = shortcut is disabled
    pub fn export_overrides(&self) -> HashMap<String, Option<String>> {
        self.user_overrides
            .iter()
            .map(|(id, opt)| (id.clone(), opt.as_ref().map(|s| s.to_canonical_string())))
            .collect()
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

    /// Find a conflict for a proposed shortcut.
    ///
    /// This is the primary API for the ShortcutRecorder UI. It checks if the given
    /// shortcut is already registered (either as a default or user override) and
    /// returns conflict information suitable for displaying to the user.
    ///
    /// # Arguments
    /// * `shortcut` - The shortcut to check for conflicts
    ///
    /// # Returns
    /// * `Some(ConflictInfo)` if the shortcut conflicts with an existing binding
    /// * `None` if the shortcut is available
    ///
    /// # Example
    /// ```ignore
    /// let shortcut = Shortcut::parse("cmd+k").unwrap();
    /// if let Some(conflict) = registry.find_conflict(&shortcut) {
    ///     println!("Conflict with: {} ({})", conflict.command_name, conflict.command_type);
    /// }
    /// ```
    pub fn find_conflict(&self, shortcut: &Shortcut) -> Option<ConflictInfo> {
        let canonical = shortcut.to_canonical_string();

        // Check OS reserved shortcuts first
        let os_reserved = Self::get_os_reserved_shortcuts();
        if os_reserved.contains(&canonical.as_str()) {
            return Some(ConflictInfo::system());
        }

        // Check all registered bindings (both defaults and user overrides)
        for binding in &self.bindings {
            if self.disabled.contains(&binding.id) {
                continue;
            }

            // Get the effective shortcut (user override or default)
            let effective = self
                .user_overrides
                .get(&binding.id)
                .cloned()
                .unwrap_or_else(|| Some(binding.default_shortcut.clone()));

            if let Some(existing_shortcut) = effective {
                if existing_shortcut.to_canonical_string() == canonical {
                    // Found a conflict
                    let command_type = match binding.source {
                        BindingSource::Builtin => "builtin",
                        BindingSource::Script => "script",
                    };

                    return Some(ConflictInfo {
                        conflicting_command_id: binding.id.clone(),
                        command_name: binding.name.clone(),
                        command_type: command_type.to_string(),
                    });
                }
            }
        }

        None
    }

    /// Find a conflict for a shortcut, excluding a specific command ID.
    ///
    /// This is useful when editing an existing shortcut - you want to check
    /// for conflicts but not report a conflict with the command being edited.
    ///
    /// # Arguments
    /// * `shortcut` - The shortcut to check for conflicts
    /// * `exclude_id` - The command ID to exclude from conflict checking
    ///
    /// # Returns
    /// * `Some(ConflictInfo)` if the shortcut conflicts with another binding
    /// * `None` if the shortcut is available (or only conflicts with excluded ID)
    pub fn find_conflict_excluding(
        &self,
        shortcut: &Shortcut,
        exclude_id: &str,
    ) -> Option<ConflictInfo> {
        let canonical = shortcut.to_canonical_string();

        // Check OS reserved shortcuts first
        let os_reserved = Self::get_os_reserved_shortcuts();
        if os_reserved.contains(&canonical.as_str()) {
            return Some(ConflictInfo::system());
        }

        // Check all registered bindings (both defaults and user overrides)
        for binding in &self.bindings {
            // Skip the excluded binding
            if binding.id == exclude_id {
                continue;
            }
            if self.disabled.contains(&binding.id) {
                continue;
            }

            // Get the effective shortcut (user override or default)
            let effective = self
                .user_overrides
                .get(&binding.id)
                .cloned()
                .unwrap_or_else(|| Some(binding.default_shortcut.clone()));

            if let Some(existing_shortcut) = effective {
                if existing_shortcut.to_canonical_string() == canonical {
                    // Found a conflict
                    let command_type = match binding.source {
                        BindingSource::Builtin => "builtin",
                        BindingSource::Script => "script",
                    };

                    return Some(ConflictInfo {
                        conflicting_command_id: binding.id.clone(),
                        command_name: binding.name.clone(),
                        command_type: command_type.to_string(),
                    });
                }
            }
        }

        None
    }

    /// Get the human-readable name for a command ID.
    ///
    /// # Arguments
    /// * `id` - The command ID to look up
    ///
    /// # Returns
    /// * `Some(&str)` with the command name if found
    /// * `None` if the command ID is not registered
    pub fn get_command_name(&self, id: &str) -> Option<&str> {
        self.get(id).map(|binding| binding.name.as_str())
    }

    /// Get full binding info for a command ID, useful for UI display.
    ///
    /// Returns the binding along with its effective shortcut (considering overrides).
    pub fn get_binding_info(&self, id: &str) -> Option<(&ShortcutBinding, Option<Shortcut>)> {
        self.get(id).map(|binding| {
            let effective = self
                .user_overrides
                .get(&binding.id)
                .cloned()
                .unwrap_or_else(|| Some(binding.default_shortcut.clone()));
            (binding, effective)
        })
    }

    fn canonical_keystroke(keystroke: &gpui::Keystroke) -> String {
        Shortcut::new(
            keystroke.key.to_lowercase(),
            super::types::Modifiers {
                cmd: keystroke.modifiers.platform,
                ctrl: keystroke.modifiers.control,
                alt: keystroke.modifiers.alt,
                shift: keystroke.modifiers.shift,
            },
        )
        .to_canonical_string()
    }

    fn effective_shortcut_for_binding(&self, binding: &ShortcutBinding) -> Option<Shortcut> {
        if self.disabled.contains(&binding.id) {
            return None;
        }

        if let Some(override_opt) = self.user_overrides.get(&binding.id) {
            return override_opt.clone();
        }

        Some(binding.default_shortcut.clone())
    }

    fn has_active_user_override(&self, binding_id: &str) -> bool {
        self.user_overrides
            .get(binding_id)
            .is_some_and(|override_shortcut| override_shortcut.is_some())
    }

    fn should_replace_index_winner(&self, candidate_idx: usize, existing_idx: usize) -> bool {
        let candidate = &self.bindings[candidate_idx];
        let existing = &self.bindings[existing_idx];

        let candidate_override = self.has_active_user_override(&candidate.id);
        let existing_override = self.has_active_user_override(&existing.id);

        if candidate_override != existing_override {
            return candidate_override;
        }

        let candidate_priority = candidate.source.priority();
        let existing_priority = existing.source.priority();
        if candidate_priority != existing_priority {
            return candidate_priority < existing_priority;
        }

        candidate_idx < existing_idx
    }

    fn rebuild_effective_shortcut_index(&mut self) {
        self.effective_shortcut_index.clear();

        for (index, binding) in self.bindings.iter().enumerate() {
            let Some(effective_shortcut) = self.effective_shortcut_for_binding(binding) else {
                continue;
            };

            let canonical = effective_shortcut.to_canonical_string();
            let existing_idx = self
                .effective_shortcut_index
                .get(&binding.context)
                .and_then(|by_shortcut| by_shortcut.get(&canonical))
                .copied();

            let should_replace = match existing_idx {
                Some(existing_idx) => self.should_replace_index_winner(index, existing_idx),
                None => true,
            };

            if should_replace {
                self.effective_shortcut_index
                    .entry(binding.context)
                    .or_default()
                    .insert(canonical, index);
            }
        }
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
