//! Terminal Command Bar Data Structures
//!
//! This module defines the data types for the terminal Cmd+K command bar.
//! The types follow patterns established in `src/actions/types.rs` for
//! consistency with the main actions system.
//!
//! # Design
//!
//! - [`TerminalAction`]: Enum of all terminal actions with stable string IDs
//! - [`TerminalCommandItem`]: Display data for command bar items with cached lowercase fields
//! - [`get_terminal_commands`]: Returns the default set of terminal commands
//!
//! # Signal Reference
//!
//! | Action   | Signal   | Shortcut | Description |
//! |----------|----------|----------|-------------|
//! | Kill     | SIGTERM  | -        | Terminate process gracefully |
//! | Interrupt| SIGINT   | Ctrl+C   | Interrupt running command |
//! | Suspend  | SIGTSTP  | Ctrl+Z   | Suspend process (background) |
//! | Quit     | SIGQUIT  | Ctrl+\   | Quit with core dump |
//! | SendEOF  | -        | Ctrl+D   | Send end-of-file |

use std::fmt;

// =============================================================================
// TerminalAction Enum
// =============================================================================

/// Actions that can be performed on a terminal via Cmd+K command bar.
///
/// Each action has a stable string ID (via [`TerminalAction::id`]) used for:
/// - Serialization/deserialization
/// - Logging and debugging
/// - Keyboard shortcut mapping
///
/// # Signal Actions
///
/// Several actions send Unix signals to the terminal process:
/// - [`Kill`](TerminalAction::Kill): SIGTERM - graceful termination
/// - [`Interrupt`](TerminalAction::Interrupt): SIGINT - interrupt (Ctrl+C)
/// - [`Suspend`](TerminalAction::Suspend): SIGTSTP - suspend to background (Ctrl+Z)
/// - [`Quit`](TerminalAction::Quit): SIGQUIT - quit with core dump (Ctrl+\)
/// - [`SendEOF`](TerminalAction::SendEOF): Sends Ctrl+D (end-of-file)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TerminalAction {
    /// Clear the terminal scrollback and visible screen.
    /// Equivalent to `clear` command or Ctrl+L in most shells.
    Clear,

    /// Copy selected text to system clipboard.
    /// If no text is selected, may copy all visible content.
    Copy,

    /// Copy all visible terminal content to clipboard.
    /// Copies every line in the visible terminal buffer.
    CopyAll,

    /// Copy the last command entered in the terminal.
    /// Parses terminal content to find the most recent command prompt.
    CopyLastCommand,

    /// Copy the output of the last command.
    /// Copies everything between the last command and the current prompt.
    CopyLastOutput,

    /// Paste text from system clipboard into terminal.
    /// Text is sent to the PTY as if typed.
    Paste,

    /// Select all visible text in the terminal.
    /// Useful before Copy to capture entire output.
    SelectAll,

    /// Scroll to the top of the scrollback buffer.
    /// Shows the oldest content in history.
    ScrollToTop,

    /// Scroll to the bottom of the scrollback buffer.
    /// Returns to the most recent output.
    ScrollToBottom,

    /// Scroll up by one page (screen height).
    ScrollPageUp,

    /// Scroll down by one page (screen height).
    ScrollPageDown,

    /// Open find/search dialog (placeholder for future).
    /// Will allow searching terminal output.
    Find,

    /// Send SIGINT to the terminal process (Ctrl+C).
    /// Interrupts the currently running command.
    Interrupt,

    /// Send SIGTERM to the terminal process.
    /// Requests graceful termination of the process.
    Kill,

    /// Send SIGTSTP to the terminal process (Ctrl+Z).
    /// Suspends the process and returns to shell.
    Suspend,

    /// Send SIGQUIT to the terminal process (Ctrl+\).
    /// Quits and typically produces a core dump.
    Quit,

    /// Send EOF (end-of-file) to the terminal (Ctrl+D).
    /// Signals end of input, often closes shell.
    SendEOF,

    /// Reset terminal state to defaults.
    /// Clears screen, resets colors, cursor position, etc.
    Reset,

    /// Open a new shell session (placeholder for future).
    /// Will create a new terminal tab or window.
    NewShell,

    /// Restart the terminal process.
    /// Kills current process and starts a fresh shell.
    Restart,

    /// Increase font size (zoom in).
    /// Font size increases by 2px, up to a maximum of 32px.
    ZoomIn,

    /// Decrease font size (zoom out).
    /// Font size decreases by 2px, down to a minimum of 8px.
    ZoomOut,

    /// Reset font size to config default.
    /// Clears any zoom override and uses the configured terminal font size.
    ResetZoom,

    /// Custom action with user-defined identifier.
    /// Used for SDK-provided or script-defined actions.
    Custom(String),
}

impl TerminalAction {
    /// Returns the stable string identifier for this action.
    ///
    /// IDs are lowercase snake_case to match the convention in
    /// `src/actions/types.rs` for built-in action IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// use script_kit_gpui::terminal::command_bar::TerminalAction;
    ///
    /// assert_eq!(TerminalAction::Clear.id(), "clear");
    /// assert_eq!(TerminalAction::ScrollToTop.id(), "scroll_to_top");
    /// assert_eq!(TerminalAction::Custom("my_action".into()).id(), "my_action");
    /// ```
    pub fn id(&self) -> &str {
        match self {
            TerminalAction::Clear => "clear",
            TerminalAction::Copy => "copy",
            TerminalAction::CopyAll => "copy_all",
            TerminalAction::CopyLastCommand => "copy_last_command",
            TerminalAction::CopyLastOutput => "copy_last_output",
            TerminalAction::Paste => "paste",
            TerminalAction::SelectAll => "select_all",
            TerminalAction::ScrollToTop => "scroll_to_top",
            TerminalAction::ScrollToBottom => "scroll_to_bottom",
            TerminalAction::ScrollPageUp => "scroll_page_up",
            TerminalAction::ScrollPageDown => "scroll_page_down",
            TerminalAction::Find => "find",
            TerminalAction::Interrupt => "interrupt",
            TerminalAction::Kill => "kill",
            TerminalAction::Suspend => "suspend",
            TerminalAction::Quit => "quit",
            TerminalAction::SendEOF => "send_eof",
            TerminalAction::Reset => "reset",
            TerminalAction::NewShell => "new_shell",
            TerminalAction::Restart => "restart",
            TerminalAction::ZoomIn => "zoom_in",
            TerminalAction::ZoomOut => "zoom_out",
            TerminalAction::ResetZoom => "reset_zoom",
            TerminalAction::Custom(id) => id.as_str(),
        }
    }

    /// Returns the keyboard shortcut for this action, if any.
    ///
    /// Shortcuts use macOS-style symbols:
    /// - ⌘ = Command
    /// - ⇧ = Shift
    /// - ⌥ = Option/Alt
    /// - ⌃ = Control
    ///
    /// # Examples
    ///
    /// ```
    /// use script_kit_gpui::terminal::command_bar::TerminalAction;
    ///
    /// assert_eq!(TerminalAction::Copy.default_shortcut(), Some("⌘C"));
    /// assert_eq!(TerminalAction::Kill.default_shortcut(), None);
    /// ```
    pub fn default_shortcut(&self) -> Option<&'static str> {
        match self {
            TerminalAction::Clear => Some("⌘K"),
            TerminalAction::Copy => Some("⌘C"),
            TerminalAction::CopyAll => Some("⇧⌘C"),
            TerminalAction::CopyLastCommand => Some("⌥⌘C"),
            TerminalAction::CopyLastOutput => Some("⌃⌘C"),
            TerminalAction::Paste => Some("⌘V"),
            TerminalAction::SelectAll => Some("⌘A"),
            TerminalAction::ScrollToTop => Some("⌘↑"),
            TerminalAction::ScrollToBottom => Some("⌘↓"),
            TerminalAction::ScrollPageUp => Some("⇧↑"),
            TerminalAction::ScrollPageDown => Some("⇧↓"),
            TerminalAction::Find => Some("⌘F"),
            TerminalAction::Interrupt => Some("⌃C"),
            TerminalAction::Suspend => Some("⌃Z"),
            TerminalAction::Quit => Some("⌃\\"),
            TerminalAction::SendEOF => Some("⌃D"),
            TerminalAction::ZoomIn => Some("⌘+"),
            TerminalAction::ZoomOut => Some("⌘-"),
            TerminalAction::ResetZoom => Some("⌘0"),
            _ => None,
        }
    }

    /// Returns true if this action sends a signal to the terminal process.
    pub fn is_signal_action(&self) -> bool {
        matches!(
            self,
            TerminalAction::Kill
                | TerminalAction::Interrupt
                | TerminalAction::Suspend
                | TerminalAction::Quit
        )
    }
}

impl fmt::Display for TerminalAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

// =============================================================================
// TerminalCommandItem Struct
// =============================================================================

/// A single command item in the terminal command bar.
///
/// Follows the pattern of `Action` in `src/actions/types.rs` with
/// cached lowercase fields for efficient filtering during search.
///
/// # Example
///
/// ```
/// use script_kit_gpui::terminal::command_bar::{TerminalAction, TerminalCommandItem};
///
/// let item = TerminalCommandItem::new(
///     "Clear Terminal",
///     "Clear the screen and scrollback buffer",
///     Some("⌘K"),
///     TerminalAction::Clear,
/// );
///
/// assert_eq!(item.name, "Clear Terminal");
/// assert_eq!(item.name_lower, "clear terminal");
/// ```
#[derive(Debug, Clone)]
pub struct TerminalCommandItem {
    /// Display name shown in the command bar list.
    pub name: String,

    /// Description shown below the name (subtitle).
    pub description: String,

    /// Keyboard shortcut hint (e.g., "⌘K", "⌃C").
    /// Displayed as keycap badges on the right side.
    pub shortcut: Option<String>,

    /// The action to execute when this command is selected.
    pub action: TerminalAction,

    // === Cached lowercase fields for fast filtering ===
    // Pre-computed to avoid repeated to_lowercase() during search.
    /// Cached lowercase name for filtering.
    pub name_lower: String,

    /// Cached lowercase description for filtering.
    pub description_lower: String,
}

impl TerminalCommandItem {
    /// Create a new terminal command item.
    ///
    /// Automatically computes lowercase versions of name and description
    /// for efficient filtering during search operations.
    ///
    /// # Arguments
    ///
    /// * `name` - Display name for the command
    /// * `description` - Description/subtitle text
    /// * `shortcut` - Optional keyboard shortcut hint
    /// * `action` - The terminal action to execute
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        shortcut: Option<impl Into<String>>,
        action: TerminalAction,
    ) -> Self {
        let name = name.into();
        let description = description.into();
        Self {
            name_lower: name.to_lowercase(),
            description_lower: description.to_lowercase(),
            name,
            description,
            shortcut: shortcut.map(|s| s.into()),
            action,
        }
    }

    /// Returns the action ID for this command.
    pub fn action_id(&self) -> &str {
        self.action.id()
    }

    /// Checks if this command matches the given search query.
    ///
    /// Matches against both name and description (case-insensitive).
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.name_lower.contains(&query_lower) || self.description_lower.contains(&query_lower)
    }
}

// =============================================================================
// Default Commands
// =============================================================================

/// Returns the default set of terminal commands.
///
/// Commands are ordered by category:
/// 1. Clipboard operations (Copy, Paste, Select All)
/// 2. Display operations (Clear, Reset, Find)
/// 3. Navigation (Scroll operations)
/// 4. Process control (Interrupt, Kill, Suspend, Quit, SendEOF, Restart)
/// 5. Future features (NewShell)
///
/// # Example
///
/// ```
/// use script_kit_gpui::terminal::command_bar::get_terminal_commands;
///
/// let commands = get_terminal_commands();
/// assert!(!commands.is_empty());
///
/// // Find the clear command
/// let clear = commands.iter().find(|c| c.action_id() == "clear");
/// assert!(clear.is_some());
/// ```
pub fn get_terminal_commands() -> Vec<TerminalCommandItem> {
    vec![
        // === Clipboard Operations ===
        TerminalCommandItem::new(
            "Copy",
            "Copy selected text to clipboard",
            Some("⌘C"),
            TerminalAction::Copy,
        ),
        TerminalCommandItem::new(
            "Copy All",
            "Copy all visible terminal content",
            Some("⇧⌘C"),
            TerminalAction::CopyAll,
        ),
        TerminalCommandItem::new(
            "Copy Last Command",
            "Copy the last command entered",
            Some("⌥⌘C"),
            TerminalAction::CopyLastCommand,
        ),
        TerminalCommandItem::new(
            "Copy Last Output",
            "Copy the output of the last command",
            Some("⌃⌘C"),
            TerminalAction::CopyLastOutput,
        ),
        TerminalCommandItem::new(
            "Paste",
            "Paste from clipboard",
            Some("⌘V"),
            TerminalAction::Paste,
        ),
        TerminalCommandItem::new(
            "Select All",
            "Select all visible text",
            Some("⌘A"),
            TerminalAction::SelectAll,
        ),
        // === Display Operations ===
        TerminalCommandItem::new(
            "Clear Terminal",
            "Clear screen and scrollback buffer",
            Some("⌘K"),
            TerminalAction::Clear,
        ),
        TerminalCommandItem::new(
            "Reset Terminal",
            "Reset terminal state to defaults",
            None::<String>,
            TerminalAction::Reset,
        ),
        TerminalCommandItem::new(
            "Find",
            "Search in terminal output",
            Some("⌘F"),
            TerminalAction::Find,
        ),
        // === Navigation ===
        TerminalCommandItem::new(
            "Scroll to Top",
            "Jump to the top of scrollback",
            Some("⌘↑"),
            TerminalAction::ScrollToTop,
        ),
        TerminalCommandItem::new(
            "Scroll to Bottom",
            "Jump to the bottom (latest output)",
            Some("⌘↓"),
            TerminalAction::ScrollToBottom,
        ),
        TerminalCommandItem::new(
            "Scroll Page Up",
            "Scroll up one page",
            Some("⇧↑"),
            TerminalAction::ScrollPageUp,
        ),
        TerminalCommandItem::new(
            "Scroll Page Down",
            "Scroll down one page",
            Some("⇧↓"),
            TerminalAction::ScrollPageDown,
        ),
        // === Process Control ===
        TerminalCommandItem::new(
            "Interrupt",
            "Send SIGINT to stop running command (Ctrl+C)",
            Some("⌃C"),
            TerminalAction::Interrupt,
        ),
        TerminalCommandItem::new(
            "Kill Process",
            "Send SIGTERM to terminate process",
            None::<String>,
            TerminalAction::Kill,
        ),
        TerminalCommandItem::new(
            "Suspend",
            "Send SIGTSTP to suspend process (Ctrl+Z)",
            Some("⌃Z"),
            TerminalAction::Suspend,
        ),
        TerminalCommandItem::new(
            "Quit",
            "Send SIGQUIT to quit with core dump (Ctrl+\\)",
            Some("⌃\\"),
            TerminalAction::Quit,
        ),
        TerminalCommandItem::new(
            "Send EOF",
            "Send end-of-file signal (Ctrl+D)",
            Some("⌃D"),
            TerminalAction::SendEOF,
        ),
        TerminalCommandItem::new(
            "Restart",
            "Restart the terminal session",
            None::<String>,
            TerminalAction::Restart,
        ),
        // === Zoom Controls ===
        TerminalCommandItem::new(
            "Zoom In",
            "Increase font size",
            Some("⌘+"),
            TerminalAction::ZoomIn,
        ),
        TerminalCommandItem::new(
            "Zoom Out",
            "Decrease font size",
            Some("⌘-"),
            TerminalAction::ZoomOut,
        ),
        TerminalCommandItem::new(
            "Reset Zoom",
            "Reset font size to default",
            Some("⌘0"),
            TerminalAction::ResetZoom,
        ),
        // === Future Features ===
        TerminalCommandItem::new(
            "New Shell",
            "Open a new shell session (coming soon)",
            None::<String>,
            TerminalAction::NewShell,
        ),
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_action_ids() {
        // Test all built-in action IDs
        assert_eq!(TerminalAction::Clear.id(), "clear");
        assert_eq!(TerminalAction::Copy.id(), "copy");
        assert_eq!(TerminalAction::Paste.id(), "paste");
        assert_eq!(TerminalAction::SelectAll.id(), "select_all");
        assert_eq!(TerminalAction::ScrollToTop.id(), "scroll_to_top");
        assert_eq!(TerminalAction::ScrollToBottom.id(), "scroll_to_bottom");
        assert_eq!(TerminalAction::ScrollPageUp.id(), "scroll_page_up");
        assert_eq!(TerminalAction::ScrollPageDown.id(), "scroll_page_down");
        assert_eq!(TerminalAction::Find.id(), "find");
        assert_eq!(TerminalAction::Interrupt.id(), "interrupt");
        assert_eq!(TerminalAction::Kill.id(), "kill");
        assert_eq!(TerminalAction::Suspend.id(), "suspend");
        assert_eq!(TerminalAction::Quit.id(), "quit");
        assert_eq!(TerminalAction::SendEOF.id(), "send_eof");
        assert_eq!(TerminalAction::Reset.id(), "reset");
        assert_eq!(TerminalAction::NewShell.id(), "new_shell");
        assert_eq!(TerminalAction::Restart.id(), "restart");
        assert_eq!(TerminalAction::ZoomIn.id(), "zoom_in");
        assert_eq!(TerminalAction::ZoomOut.id(), "zoom_out");
        assert_eq!(TerminalAction::ResetZoom.id(), "reset_zoom");

        // Test custom action ID
        assert_eq!(TerminalAction::Custom("my_action".into()).id(), "my_action");
    }

    #[test]
    fn test_terminal_action_shortcuts() {
        assert_eq!(TerminalAction::Clear.default_shortcut(), Some("⌘K"));
        assert_eq!(TerminalAction::Copy.default_shortcut(), Some("⌘C"));
        assert_eq!(TerminalAction::Paste.default_shortcut(), Some("⌘V"));
        assert_eq!(TerminalAction::Interrupt.default_shortcut(), Some("⌃C"));
        assert_eq!(TerminalAction::Suspend.default_shortcut(), Some("⌃Z"));
        assert_eq!(TerminalAction::SendEOF.default_shortcut(), Some("⌃D"));
        assert_eq!(TerminalAction::ZoomIn.default_shortcut(), Some("⌘+"));
        assert_eq!(TerminalAction::ZoomOut.default_shortcut(), Some("⌘-"));
        assert_eq!(TerminalAction::ResetZoom.default_shortcut(), Some("⌘0"));

        // Actions without default shortcuts
        assert_eq!(TerminalAction::Kill.default_shortcut(), None);
        assert_eq!(TerminalAction::Reset.default_shortcut(), None);
        assert_eq!(TerminalAction::NewShell.default_shortcut(), None);
    }

    #[test]
    fn test_terminal_action_is_signal() {
        assert!(TerminalAction::Kill.is_signal_action());
        assert!(TerminalAction::Interrupt.is_signal_action());
        assert!(TerminalAction::Suspend.is_signal_action());
        assert!(TerminalAction::Quit.is_signal_action());

        assert!(!TerminalAction::Clear.is_signal_action());
        assert!(!TerminalAction::Copy.is_signal_action());
        assert!(!TerminalAction::SendEOF.is_signal_action());
        assert!(!TerminalAction::ZoomIn.is_signal_action());
        assert!(!TerminalAction::ZoomOut.is_signal_action());
        assert!(!TerminalAction::ResetZoom.is_signal_action());
    }

    #[test]
    fn test_terminal_action_display() {
        assert_eq!(format!("{}", TerminalAction::Clear), "clear");
        assert_eq!(format!("{}", TerminalAction::ScrollToTop), "scroll_to_top");
    }

    #[test]
    fn test_command_item_creation() {
        let item = TerminalCommandItem::new(
            "Clear Terminal",
            "Clear the screen",
            Some("⌘K"),
            TerminalAction::Clear,
        );

        assert_eq!(item.name, "Clear Terminal");
        assert_eq!(item.description, "Clear the screen");
        assert_eq!(item.shortcut, Some("⌘K".to_string()));
        assert_eq!(item.action, TerminalAction::Clear);
        assert_eq!(item.name_lower, "clear terminal");
        assert_eq!(item.description_lower, "clear the screen");
        assert_eq!(item.action_id(), "clear");
    }

    #[test]
    fn test_command_item_without_shortcut() {
        let item = TerminalCommandItem::new(
            "Kill Process",
            "Terminate the running process",
            None::<String>,
            TerminalAction::Kill,
        );

        assert!(item.shortcut.is_none());
        assert_eq!(item.action_id(), "kill");
    }

    #[test]
    fn test_command_item_matches() {
        let item = TerminalCommandItem::new(
            "Clear Terminal",
            "Clear the screen and scrollback",
            Some("⌘K"),
            TerminalAction::Clear,
        );

        // Matches name
        assert!(item.matches("clear"));
        assert!(item.matches("CLEAR"));
        assert!(item.matches("terminal"));

        // Matches description
        assert!(item.matches("screen"));
        assert!(item.matches("scrollback"));

        // Partial matches
        assert!(item.matches("cle"));
        assert!(item.matches("scr"));

        // No match
        assert!(!item.matches("paste"));
        assert!(!item.matches("xyz"));
    }

    #[test]
    fn test_get_terminal_commands() {
        let commands = get_terminal_commands();

        // Should have all required commands
        assert!(commands.len() >= 17);

        // Check essential commands exist
        let action_ids: Vec<&str> = commands.iter().map(|c| c.action_id()).collect();

        assert!(action_ids.contains(&"clear"));
        assert!(action_ids.contains(&"copy"));
        assert!(action_ids.contains(&"paste"));
        assert!(action_ids.contains(&"select_all"));
        assert!(action_ids.contains(&"scroll_to_top"));
        assert!(action_ids.contains(&"scroll_to_bottom"));
        assert!(action_ids.contains(&"find"));
        assert!(action_ids.contains(&"interrupt"));
        assert!(action_ids.contains(&"kill"));
        assert!(action_ids.contains(&"suspend"));
        assert!(action_ids.contains(&"quit"));
        assert!(action_ids.contains(&"send_eof"));
        assert!(action_ids.contains(&"reset"));
        assert!(action_ids.contains(&"new_shell"));
        assert!(action_ids.contains(&"restart"));
    }

    #[test]
    fn test_commands_have_descriptions() {
        let commands = get_terminal_commands();

        for cmd in &commands {
            assert!(
                !cmd.description.is_empty(),
                "Command {} should have a description",
                cmd.name
            );
        }
    }

    #[test]
    fn test_commands_have_lowercase_cache() {
        let commands = get_terminal_commands();

        for cmd in &commands {
            assert_eq!(
                cmd.name_lower,
                cmd.name.to_lowercase(),
                "name_lower should be lowercase version of name"
            );
            assert_eq!(
                cmd.description_lower,
                cmd.description.to_lowercase(),
                "description_lower should be lowercase version of description"
            );
        }
    }
}
