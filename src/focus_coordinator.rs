//! Focus Coordinator - Centralized focus management for Script Kit GPUI
//!
//! This module provides a single control plane for focus management, replacing
//! the scattered `focused_input` + `pending_focus` pattern with a unified system.
//!
//! # Architecture
//!
//! The FocusCoordinator owns:
//! - **pending**: The next focus request to apply (applied once in render, then cleared)
//! - **restore_stack**: Stack of focus states for overlay push/pop semantics
//! - **current_cursor_owner**: Single source of truth for cursor blink ownership
//!
//! # Key Concepts
//!
//! - **FocusTarget**: Where focus should go (MainFilter, ActionsDialog, specific prompts)
//! - **CursorOwner**: Which input gets the blinking cursor (for text input UX)
//! - **FocusRequest**: Complete focus intent (target + cursor owner)
//!
//! # Usage Patterns
//!
//! ```rust,ignore
//! // Request focus to main filter with cursor
//! coordinator.request(FocusRequest::main_filter());
//!
//! // Push overlay (actions dialog) - saves current state for restore
//! coordinator.push_overlay(FocusRequest::actions_dialog());
//!
//! // Pop overlay - restores previous focus state
//! coordinator.pop_overlay();
//!
//! // Apply pending focus (called once per render when appropriate)
//! if let Some(request) = coordinator.take_pending() {
//!     // ... apply focus based on request.target
//! }
//! ```

use crate::logging;

/// Tracks which input field currently owns the blinking cursor.
///
/// This is separate from GPUI focus - an input can be "focused" for GPUI purposes
/// (receiving keyboard events) but not be the cursor owner (not showing a blinking cursor).
///
/// Replaces the old `FocusedInput` enum with clearer semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorOwner {
    /// Main script list filter input
    MainFilter,
    /// Actions dialog search input
    ActionsSearch,
    /// Arg prompt input (when running a script)
    ArgPrompt,
    /// Chat prompt input
    ChatPrompt,
    /// No input owns the cursor (e.g., terminal, editor with own cursor)
    #[default]
    None,
}

/// Identifies the target element that should receive GPUI focus.
///
/// Unlike the old FocusTarget, this enum does NOT include "AppRoot" as a magic
/// indirection. Each variant maps directly to the actual focus destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // All variants are part of the API for gradual migration
pub enum FocusTarget {
    /// Focus the main filter input (gpui_input_state)
    MainFilter,
    /// Focus the actions dialog's search input
    ActionsDialog,
    /// Focus the arg prompt (uses main app's focus_handle + ArgPrompt cursor)
    ArgPrompt,
    /// Focus the path prompt's focus handle
    PathPrompt,
    /// Focus the form prompt (delegates to active field)
    FormPrompt,
    /// Focus the editor prompt
    EditorPrompt,
    /// Focus the select prompt
    SelectPrompt,
    /// Focus the env prompt
    EnvPrompt,
    /// Focus the drop prompt
    DropPrompt,
    /// Focus the template prompt
    TemplatePrompt,
    /// Focus the term prompt
    TermPrompt,
    /// Focus the chat prompt
    ChatPrompt,
    /// Focus the div prompt (uses app root, no cursor)
    DivPrompt,
    /// Focus the scratchpad editor
    ScratchPad,
    /// Focus the quick terminal
    QuickTerminal,
}

impl FocusTarget {
    /// Returns the default cursor owner for this focus target.
    ///
    /// Most prompts own their own cursor (None), but some delegate to the
    /// main app's input system.
    pub fn default_cursor_owner(self) -> CursorOwner {
        match self {
            FocusTarget::MainFilter => CursorOwner::MainFilter,
            FocusTarget::ActionsDialog => CursorOwner::ActionsSearch,
            FocusTarget::ArgPrompt => CursorOwner::ArgPrompt,
            FocusTarget::ChatPrompt => CursorOwner::ChatPrompt,
            // These prompts have their own cursor management
            FocusTarget::PathPrompt
            | FocusTarget::FormPrompt
            | FocusTarget::EditorPrompt
            | FocusTarget::SelectPrompt
            | FocusTarget::EnvPrompt
            | FocusTarget::DropPrompt
            | FocusTarget::TemplatePrompt
            | FocusTarget::TermPrompt
            | FocusTarget::DivPrompt
            | FocusTarget::ScratchPad
            | FocusTarget::QuickTerminal => CursorOwner::None,
        }
    }
}

/// A complete focus request: where to focus + who owns the cursor.
///
/// This replaces the old pattern of setting `focused_input` and `pending_focus`
/// separately with hidden coupling between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusRequest {
    /// The element that should receive GPUI focus
    pub target: FocusTarget,
    /// Which input gets the blinking cursor (can differ from focus target)
    pub cursor: CursorOwner,
}

impl FocusRequest {
    /// Create a new focus request with explicit target and cursor owner.
    #[allow(dead_code)]
    pub fn new(target: FocusTarget, cursor: CursorOwner) -> Self {
        Self { target, cursor }
    }

    /// Create a focus request with default cursor owner for the target.
    pub fn with_default_cursor(target: FocusTarget) -> Self {
        Self {
            target,
            cursor: target.default_cursor_owner(),
        }
    }

    // === Convenience constructors ===

    /// Focus main filter with cursor
    pub fn main_filter() -> Self {
        Self::with_default_cursor(FocusTarget::MainFilter)
    }

    /// Focus actions dialog with cursor in search
    pub fn actions_dialog() -> Self {
        Self::with_default_cursor(FocusTarget::ActionsDialog)
    }

    /// Focus arg prompt with cursor
    pub fn arg_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::ArgPrompt)
    }

    /// Focus chat prompt with cursor
    pub fn chat_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::ChatPrompt)
    }

    /// Focus div prompt (no cursor)
    #[allow(dead_code)]
    pub fn div_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::DivPrompt)
    }

    /// Focus form prompt (delegates to active field)
    #[allow(dead_code)]
    pub fn form_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::FormPrompt)
    }

    /// Focus path prompt
    #[allow(dead_code)]
    pub fn path_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::PathPrompt)
    }

    /// Focus editor prompt
    #[allow(dead_code)]
    pub fn editor_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::EditorPrompt)
    }

    /// Focus select prompt
    #[allow(dead_code)]
    pub fn select_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::SelectPrompt)
    }

    /// Focus env prompt
    #[allow(dead_code)]
    pub fn env_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::EnvPrompt)
    }

    /// Focus drop prompt
    #[allow(dead_code)]
    pub fn drop_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::DropPrompt)
    }

    /// Focus template prompt
    #[allow(dead_code)]
    pub fn template_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::TemplatePrompt)
    }

    /// Focus term prompt
    #[allow(dead_code)]
    pub fn term_prompt() -> Self {
        Self::with_default_cursor(FocusTarget::TermPrompt)
    }

    /// Focus scratchpad
    #[allow(dead_code)]
    pub fn scratchpad() -> Self {
        Self::with_default_cursor(FocusTarget::ScratchPad)
    }

    /// Focus quick terminal
    #[allow(dead_code)]
    pub fn quick_terminal() -> Self {
        Self::with_default_cursor(FocusTarget::QuickTerminal)
    }
}

/// Centralized focus coordinator for the application.
///
/// This is the single source of truth for:
/// - What should be focused next (pending request)
/// - What was focused before an overlay opened (restore stack)
/// - Which input currently owns the cursor
///
/// # Focus Application
///
/// Focus is applied at a single choke point: `render()` calls `take_pending()`
/// and applies focus exactly once, then the pending request is cleared.
/// This prevents "perpetual focus enforcement" thrash.
///
/// # Overlay Semantics
///
/// Overlays (actions dialog, shortcut recorder, etc.) use push/pop:
/// - `push_overlay()`: Save current state, set new focus
/// - `pop_overlay()`: Restore previous state
///
/// This eliminates scattered restoration logic and "forgot to restore" bugs.
#[derive(Debug, Default)]
pub struct FocusCoordinator {
    /// Next focus request to apply (consumed once, then None)
    pending: Option<FocusRequest>,
    /// Stack of saved focus states for overlay restoration
    restore_stack: Vec<FocusRequest>,
    /// Current cursor owner (single source of truth)
    current_cursor_owner: CursorOwner,
}

impl FocusCoordinator {
    /// Create a new focus coordinator with default state.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a coordinator with initial focus on main filter.
    pub fn with_main_filter_focus() -> Self {
        Self {
            pending: Some(FocusRequest::main_filter()),
            restore_stack: Vec::new(),
            current_cursor_owner: CursorOwner::MainFilter,
        }
    }

    // === Request Management ===

    /// Request focus to a target with its default cursor owner.
    ///
    /// This is the primary API for non-overlay focus changes.
    pub fn request(&mut self, request: FocusRequest) {
        logging::log(
            "FOCUS",
            &format!(
                "Focus request: target={:?}, cursor={:?}",
                request.target, request.cursor
            ),
        );
        self.pending = Some(request);
    }

    /// Request focus to a target with default cursor owner.
    #[allow(dead_code)]
    pub fn request_target(&mut self, target: FocusTarget) {
        self.request(FocusRequest::with_default_cursor(target));
    }

    /// Take the pending focus request (called once per render).
    ///
    /// Returns the request and clears pending. The caller is responsible
    /// for actually applying focus based on the request.
    #[allow(dead_code)]
    pub fn take_pending(&mut self) -> Option<FocusRequest> {
        let request = self.pending.take();
        if let Some(ref req) = request {
            // Update cursor owner when focus is applied
            self.current_cursor_owner = req.cursor;
            logging::log(
                "FOCUS",
                &format!(
                    "Applying pending focus: target={:?}, cursor={:?}",
                    req.target, req.cursor
                ),
            );
        }
        request
    }

    /// Check if there's a pending focus request.
    #[allow(dead_code)]
    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Peek at the pending request without consuming it.
    pub fn peek_pending(&self) -> Option<&FocusRequest> {
        self.pending.as_ref()
    }

    // === Overlay Push/Pop ===

    /// Push an overlay onto the stack and request its focus.
    ///
    /// Saves the current focus state (based on cursor owner) so it can be
    /// restored when `pop_overlay()` is called.
    pub fn push_overlay(&mut self, overlay_request: FocusRequest) {
        // Save current state for restoration
        let saved = self.infer_current_request();
        logging::log(
            "FOCUS",
            &format!(
                "Pushing overlay: {:?} (saving: {:?})",
                overlay_request.target, saved.target
            ),
        );
        self.restore_stack.push(saved);
        self.request(overlay_request);
    }

    /// Pop the overlay and restore previous focus.
    ///
    /// If the stack is empty, falls back to main filter focus.
    pub fn pop_overlay(&mut self) {
        let restored = self.restore_stack.pop().unwrap_or_else(|| {
            logging::log("FOCUS", "Restore stack empty, falling back to MainFilter");
            FocusRequest::main_filter()
        });
        logging::log(
            "FOCUS",
            &format!("Popping overlay, restoring to: {:?}", restored.target),
        );
        self.request(restored);
    }

    /// Clear all overlays and restore to main filter.
    ///
    /// Useful for "escape all" or error recovery.
    pub fn clear_overlays(&mut self) {
        if !self.restore_stack.is_empty() {
            logging::log(
                "FOCUS",
                &format!(
                    "Clearing {} overlay(s) from stack",
                    self.restore_stack.len()
                ),
            );
        }
        self.restore_stack.clear();
        self.request(FocusRequest::main_filter());
    }

    /// Get the current overlay depth.
    #[allow(dead_code)]
    pub fn overlay_depth(&self) -> usize {
        self.restore_stack.len()
    }

    /// Check if any overlay is active.
    #[allow(dead_code)]
    pub fn has_overlay(&self) -> bool {
        !self.restore_stack.is_empty()
    }

    // === Cursor Management ===

    /// Get the current cursor owner.
    pub fn cursor_owner(&self) -> CursorOwner {
        self.current_cursor_owner
    }

    /// Directly set cursor owner without changing focus target.
    ///
    /// Use sparingly - prefer `request()` which sets both consistently.
    #[allow(dead_code)]
    pub fn set_cursor_owner(&mut self, owner: CursorOwner) {
        self.current_cursor_owner = owner;
    }

    // === Internal Helpers ===

    /// Infer the current focus request from cursor owner.
    ///
    /// Used to save state when pushing overlays.
    fn infer_current_request(&self) -> FocusRequest {
        match self.current_cursor_owner {
            CursorOwner::MainFilter => FocusRequest::main_filter(),
            CursorOwner::ActionsSearch => FocusRequest::actions_dialog(),
            CursorOwner::ArgPrompt => FocusRequest::arg_prompt(),
            CursorOwner::ChatPrompt => FocusRequest::chat_prompt(),
            CursorOwner::None => {
                // When no cursor owner, we can't infer the exact target.
                // Default to main filter as the safest fallback.
                FocusRequest::main_filter()
            }
        }
    }
}

// === Legacy Compatibility ===
// These types help with gradual migration from the old system.

/// Maps old FocusedInput to new CursorOwner.
///
/// Use this during migration, then remove.
impl CursorOwner {
    /// Convert from old FocusedInput string representation.
    #[allow(dead_code)]
    pub fn from_legacy(s: &str) -> Self {
        match s {
            "MainFilter" => CursorOwner::MainFilter,
            "ActionsSearch" => CursorOwner::ActionsSearch,
            "ArgPrompt" => CursorOwner::ArgPrompt,
            "None" => CursorOwner::None,
            _ => CursorOwner::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_request_defaults() {
        let req = FocusRequest::main_filter();
        assert_eq!(req.target, FocusTarget::MainFilter);
        assert_eq!(req.cursor, CursorOwner::MainFilter);

        let req = FocusRequest::div_prompt();
        assert_eq!(req.target, FocusTarget::DivPrompt);
        assert_eq!(req.cursor, CursorOwner::None);
    }

    #[test]
    fn test_coordinator_request() {
        let mut coord = FocusCoordinator::new();
        assert!(!coord.has_pending());

        coord.request(FocusRequest::main_filter());
        assert!(coord.has_pending());

        let req = coord.take_pending();
        assert!(req.is_some());
        assert!(!coord.has_pending());
        assert_eq!(coord.cursor_owner(), CursorOwner::MainFilter);
    }

    #[test]
    fn test_overlay_push_pop() {
        let mut coord = FocusCoordinator::with_main_filter_focus();

        // Apply initial focus
        coord.take_pending();
        assert_eq!(coord.cursor_owner(), CursorOwner::MainFilter);
        assert_eq!(coord.overlay_depth(), 0);

        // Push overlay
        coord.push_overlay(FocusRequest::actions_dialog());
        assert_eq!(coord.overlay_depth(), 1);

        // Apply overlay focus
        let req = coord.take_pending().unwrap();
        assert_eq!(req.target, FocusTarget::ActionsDialog);
        assert_eq!(coord.cursor_owner(), CursorOwner::ActionsSearch);

        // Pop overlay
        coord.pop_overlay();
        let req = coord.take_pending().unwrap();
        assert_eq!(req.target, FocusTarget::MainFilter);
        assert_eq!(coord.overlay_depth(), 0);
    }

    #[test]
    fn test_overlay_clear() {
        let mut coord = FocusCoordinator::with_main_filter_focus();
        coord.take_pending();

        // Push multiple overlays
        coord.push_overlay(FocusRequest::actions_dialog());
        coord.take_pending();
        coord.push_overlay(FocusRequest::arg_prompt());
        coord.take_pending();

        assert_eq!(coord.overlay_depth(), 2);

        // Clear all
        coord.clear_overlays();
        assert_eq!(coord.overlay_depth(), 0);

        let req = coord.take_pending().unwrap();
        assert_eq!(req.target, FocusTarget::MainFilter);
    }

    #[test]
    fn test_pop_empty_stack_fallback() {
        let mut coord = FocusCoordinator::new();
        coord.pop_overlay();

        let req = coord.take_pending().unwrap();
        assert_eq!(req.target, FocusTarget::MainFilter);
    }
}
