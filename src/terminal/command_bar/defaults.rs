use super::{TerminalAction, TerminalCommandItem};

/// Returns the default set of terminal commands.
///
/// Commands are ordered by category:
/// 1. Clipboard operations (Copy, Paste, Select All)
/// 2. Display operations (Clear, Reset, Find)
/// 3. Navigation (Scroll operations)
/// 4. Process control (Interrupt, Kill, Suspend, Quit, SendEOF, Restart)
/// 5. Future features (NewShell)
pub fn get_terminal_commands() -> Vec<TerminalCommandItem> {
    vec![
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
        TerminalCommandItem::new(
            "New Shell",
            "Open a new shell session (coming soon)",
            None::<String>,
            TerminalAction::NewShell,
        ),
    ]
}
