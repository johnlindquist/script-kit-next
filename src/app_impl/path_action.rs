//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR5a:
//! typed handle for the `path_prompt` action set.
//!
//! `execute_path_action` was previously a stringly-typed dispatcher
//! that fell into a silent "unknown action" log line when an id was
//! typoed or renamed. This module introduces [`PathAction`] — an
//! exhaustive enum whose variants are the only ids the dispatcher
//! will ever execute — plus a single round-trip table [`PathAction::action_id`].
//! The path dispatcher parses `&str` once at the boundary and then
//! operates on the enum, so a typo at a call site fails at parse
//! time instead of vanishing into the `_` arm.
//!
//! This is deliberately a data-only module: no UI, no GPUI context,
//! no logging, so it is trivially unit-testable.

/// Every path-prompt action the launcher dispatches. Variants are
/// `Copy`-able so callers can thread them through closures without
/// juggling lifetimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathAction {
    /// Submit the currently selected file path via the path prompt's
    /// `on_submit` callback.
    SelectFile,
    /// Navigate into a directory (path prompt stays open).
    OpenDirectory,
    /// Copy the full path to the system clipboard.
    CopyPath,
    /// Copy the trailing filename (no directory) to the clipboard.
    CopyFilename,
    /// Reveal the path in the OS file manager (Finder / Explorer).
    OpenInFinder,
    /// Launch the user-configured editor with the path as argv[1].
    OpenInEditor,
    /// Open Quick Terminal rooted at the path (or the parent for files).
    OpenInQuickTerminal,
    /// Confirm + move the path to the system trash.
    MoveToTrash,
}

impl PathAction {
    /// All variants in declaration order. Used by the round-trip
    /// parser and by uniqueness tests.
    pub const ALL: &'static [PathAction] = &[
        PathAction::SelectFile,
        PathAction::OpenDirectory,
        PathAction::CopyPath,
        PathAction::CopyFilename,
        PathAction::OpenInFinder,
        PathAction::OpenInEditor,
        PathAction::OpenInQuickTerminal,
        PathAction::MoveToTrash,
    ];

    /// Canonical action-id as produced by the action-menu wiring.
    /// Matches the match arms that previously lived in
    /// `execute_path_action` — do not change these strings without
    /// also updating the Bun-side action definitions.
    pub const fn action_id(self) -> &'static str {
        match self {
            PathAction::SelectFile => "select_file",
            PathAction::OpenDirectory => "open_directory",
            PathAction::CopyPath => "copy_path",
            PathAction::CopyFilename => "copy_filename",
            PathAction::OpenInFinder => "open_in_finder",
            PathAction::OpenInEditor => "open_in_editor",
            PathAction::OpenInQuickTerminal => "open_in_quick_terminal",
            PathAction::MoveToTrash => "move_to_trash",
        }
    }

    /// Parse a raw action-id. Accepts the optional `file:` prefix
    /// that some action-menu callers emit (the previous dispatcher
    /// stripped it once at the top, so we preserve that behaviour).
    /// Returns `None` for unknown ids so the caller can log exactly
    /// once and keep going, instead of dispatching into a silent
    /// no-op branch.
    pub fn from_action_id(raw: &str) -> Option<Self> {
        let id = raw.strip_prefix("file:").unwrap_or(raw);
        Self::ALL.iter().copied().find(|a| a.action_id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_round_trips() {
        for &action in PathAction::ALL {
            assert_eq!(
                PathAction::from_action_id(action.action_id()),
                Some(action),
                "round-trip failed for {action:?}"
            );
        }
    }

    #[test]
    fn file_prefix_is_stripped() {
        assert_eq!(
            PathAction::from_action_id("file:copy_path"),
            Some(PathAction::CopyPath)
        );
        assert_eq!(
            PathAction::from_action_id("file:move_to_trash"),
            Some(PathAction::MoveToTrash)
        );
    }

    #[test]
    fn unknown_is_none() {
        assert_eq!(PathAction::from_action_id("totally_unknown"), None);
        assert_eq!(PathAction::from_action_id(""), None);
        // A bare `file:` prefix with nothing after is also unknown.
        assert_eq!(PathAction::from_action_id("file:"), None);
    }

    #[test]
    fn action_ids_are_unique() {
        use std::collections::BTreeSet;
        let ids: BTreeSet<&'static str> = PathAction::ALL.iter().map(|a| a.action_id()).collect();
        assert_eq!(
            ids.len(),
            PathAction::ALL.len(),
            "duplicate action_id between variants: {ids:?}"
        );
    }

    #[test]
    fn action_ids_are_snake_case() {
        for &action in PathAction::ALL {
            let id = action.action_id();
            assert!(
                id.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{action:?} action_id `{id}` must be snake_case",
            );
            assert!(
                !id.starts_with('_') && !id.ends_with('_'),
                "{action:?} action_id `{id}` must not start or end with underscore",
            );
        }
    }

    #[test]
    fn legacy_select_and_open_dir_still_parse() {
        // Belt-and-braces: pin the exact ids the old dispatcher
        // accepted (`select_file`, `open_directory`) so a future
        // rename lands with a compile break, not a runtime surprise.
        assert_eq!(
            PathAction::from_action_id("select_file"),
            Some(PathAction::SelectFile)
        );
        assert_eq!(
            PathAction::from_action_id("open_directory"),
            Some(PathAction::OpenDirectory)
        );
    }
}
