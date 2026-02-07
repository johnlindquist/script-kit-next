use std::fmt;

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
