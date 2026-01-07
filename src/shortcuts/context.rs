//! Shortcut context stack for deterministic keyboard routing.
//!
//! The context stack determines which shortcuts are active based on the current
//! UI state. Contexts are ordered from most-specific to least-specific, and
//! shortcuts are matched against the first context that contains them.
//!
//! This prevents issues like "Global shortcut eats arrow keys in editor" by
//! ensuring editor-specific shortcuts take precedence over global ones.

// Allow dead code during incremental development
#![allow(dead_code)]

/// A context in which shortcuts can be active.
///
/// Contexts are ordered by specificity:
/// - Modal contexts (ActionsDialog) are most specific
/// - View contexts (Editor, ScriptList, etc.) are next
/// - Group contexts (AnyPrompt) catch multiple views
/// - Global is always last
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShortcutContext {
    /// Active when the actions dialog/popup is open (most specific)
    /// Takes precedence over all other contexts
    ActionsDialog,

    /// Active when editor prompt is focused
    Editor,

    /// Active when terminal prompt is focused
    Terminal,

    /// Active when script list is focused (main menu)
    ScriptList,

    /// Active when arg prompt is shown (list selection)
    ArgPrompt,

    /// Active when form prompt is shown
    FormPrompt,

    /// Active when div prompt is shown (HTML content)
    DivPrompt,

    /// Active when path prompt is shown (file picker)
    PathPrompt,

    /// Active in any prompt (catch-all for prompts)
    AnyPrompt,

    /// Always active (least specific)
    /// Should only contain shortcuts that don't conflict with view-specific ones
    Global,
}

impl ShortcutContext {
    /// Get the specificity level (lower = more specific = higher priority).
    ///
    /// Used for sorting context stacks and resolving conflicts.
    pub fn specificity(&self) -> u8 {
        match self {
            // Modal contexts (highest priority)
            Self::ActionsDialog => 0,

            // Specific view contexts
            Self::Editor => 10,
            Self::Terminal => 10,
            Self::ScriptList => 10,
            Self::ArgPrompt => 10,
            Self::FormPrompt => 10,
            Self::DivPrompt => 10,
            Self::PathPrompt => 10,

            // Group context
            Self::AnyPrompt => 20,

            // Global (lowest priority)
            Self::Global => 100,
        }
    }

    /// Check if this context contains another (for shadowing detection).
    ///
    /// Returns true if `self` is a more general context that contains `other`.
    pub fn contains(&self, other: &ShortcutContext) -> bool {
        match self {
            Self::Global => true, // Global contains everything
            Self::AnyPrompt => matches!(
                other,
                Self::ArgPrompt
                    | Self::FormPrompt
                    | Self::DivPrompt
                    | Self::PathPrompt
                    | Self::Editor
                    | Self::Terminal
            ),
            _ => self == other,
        }
    }

    /// Check if this context is a modal (blocks underlying contexts).
    pub fn is_modal(&self) -> bool {
        matches!(self, Self::ActionsDialog)
    }
}

/// An ordered stack of active contexts.
///
/// Built from current UI state, ordered from most-specific to least-specific.
/// Shortcut matching searches contexts in order, stopping at the first match.
#[derive(Clone, Debug, Default)]
pub struct ContextStack {
    /// Contexts in order from most-specific to least-specific
    contexts: Vec<ShortcutContext>,
}

impl ContextStack {
    /// Create an empty context stack.
    pub fn new() -> Self {
        Self { contexts: vec![] }
    }

    /// Create a context stack with just Global.
    pub fn global_only() -> Self {
        Self {
            contexts: vec![ShortcutContext::Global],
        }
    }

    /// Build a context stack from current UI state.
    ///
    /// # Arguments
    /// * `view` - Current view type
    /// * `has_actions_popup` - Whether actions dialog is open
    ///
    /// Returns a properly ordered stack for shortcut matching.
    pub fn from_state(view: ViewType, has_actions_popup: bool) -> Self {
        let mut contexts = Vec::new();

        // Modal takes precedence if open
        if has_actions_popup {
            contexts.push(ShortcutContext::ActionsDialog);
        }

        // Add view-specific context
        match view {
            ViewType::ScriptList => contexts.push(ShortcutContext::ScriptList),
            ViewType::ArgPrompt => {
                contexts.push(ShortcutContext::ArgPrompt);
                contexts.push(ShortcutContext::AnyPrompt);
            }
            ViewType::Editor => {
                contexts.push(ShortcutContext::Editor);
                contexts.push(ShortcutContext::AnyPrompt);
            }
            ViewType::Terminal => {
                contexts.push(ShortcutContext::Terminal);
                contexts.push(ShortcutContext::AnyPrompt);
            }
            ViewType::Form => {
                contexts.push(ShortcutContext::FormPrompt);
                contexts.push(ShortcutContext::AnyPrompt);
            }
            ViewType::Div => {
                contexts.push(ShortcutContext::DivPrompt);
                contexts.push(ShortcutContext::AnyPrompt);
            }
            ViewType::Path => {
                contexts.push(ShortcutContext::PathPrompt);
                contexts.push(ShortcutContext::AnyPrompt);
            }
        }

        // Global is always last (unless modal is blocking)
        // Note: ActionsDialog is modal, so we still add Global for non-navigation keys
        contexts.push(ShortcutContext::Global);

        Self { contexts }
    }

    /// Get the contexts in order.
    pub fn contexts(&self) -> &[ShortcutContext] {
        &self.contexts
    }

    /// Check if a context is in this stack.
    pub fn contains(&self, context: ShortcutContext) -> bool {
        self.contexts.contains(&context)
    }

    /// Check if any modal context is active.
    pub fn has_modal(&self) -> bool {
        self.contexts.iter().any(|c| c.is_modal())
    }

    /// Iterate over contexts in order.
    pub fn iter(&self) -> impl Iterator<Item = &ShortcutContext> {
        self.contexts.iter()
    }
}

/// View type for building context stacks.
///
/// Maps to the current prompt/view being displayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewType {
    /// Main script list
    ScriptList,
    /// Arg prompt (list selection)
    ArgPrompt,
    /// Editor prompt
    Editor,
    /// Terminal prompt
    Terminal,
    /// Form prompt
    Form,
    /// Div prompt (HTML content)
    Div,
    /// Path prompt (file picker)
    Path,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_specificity_ordering() {
        assert!(
            ShortcutContext::ActionsDialog.specificity() < ShortcutContext::Editor.specificity()
        );
        assert!(ShortcutContext::Editor.specificity() < ShortcutContext::AnyPrompt.specificity());
        assert!(ShortcutContext::AnyPrompt.specificity() < ShortcutContext::Global.specificity());
    }

    #[test]
    fn context_contains_relationship() {
        assert!(ShortcutContext::Global.contains(&ShortcutContext::Editor));
        assert!(ShortcutContext::Global.contains(&ShortcutContext::ScriptList));
        assert!(ShortcutContext::Global.contains(&ShortcutContext::Global));

        assert!(ShortcutContext::AnyPrompt.contains(&ShortcutContext::ArgPrompt));
        assert!(ShortcutContext::AnyPrompt.contains(&ShortcutContext::Editor));
        assert!(!ShortcutContext::AnyPrompt.contains(&ShortcutContext::ScriptList));

        assert!(ShortcutContext::Editor.contains(&ShortcutContext::Editor));
        assert!(!ShortcutContext::Editor.contains(&ShortcutContext::Terminal));
    }

    #[test]
    fn context_stack_from_script_list() {
        let stack = ContextStack::from_state(ViewType::ScriptList, false);
        let contexts = stack.contexts();

        assert_eq!(contexts.len(), 2);
        assert_eq!(contexts[0], ShortcutContext::ScriptList);
        assert_eq!(contexts[1], ShortcutContext::Global);
    }

    #[test]
    fn context_stack_from_editor() {
        let stack = ContextStack::from_state(ViewType::Editor, false);
        let contexts = stack.contexts();

        assert_eq!(contexts.len(), 3);
        assert_eq!(contexts[0], ShortcutContext::Editor);
        assert_eq!(contexts[1], ShortcutContext::AnyPrompt);
        assert_eq!(contexts[2], ShortcutContext::Global);
    }

    #[test]
    fn context_stack_with_actions_popup() {
        let stack = ContextStack::from_state(ViewType::Editor, true);
        let contexts = stack.contexts();

        assert_eq!(contexts[0], ShortcutContext::ActionsDialog);
        assert!(stack.has_modal());
    }

    #[test]
    fn context_stack_ordering_is_deterministic() {
        // Same state should always produce same stack
        let stack1 = ContextStack::from_state(ViewType::ArgPrompt, true);
        let stack2 = ContextStack::from_state(ViewType::ArgPrompt, true);

        assert_eq!(stack1.contexts(), stack2.contexts());
    }

    #[test]
    fn modal_contexts_identified() {
        assert!(ShortcutContext::ActionsDialog.is_modal());
        assert!(!ShortcutContext::Editor.is_modal());
        assert!(!ShortcutContext::Global.is_modal());
    }

    #[test]
    fn global_only_stack() {
        let stack = ContextStack::global_only();
        assert_eq!(stack.contexts().len(), 1);
        assert_eq!(stack.contexts()[0], ShortcutContext::Global);
    }
}
