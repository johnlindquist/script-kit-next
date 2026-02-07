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

mod actions;
mod command_item;
mod defaults;
#[cfg(test)]
mod tests;

pub use actions::TerminalAction;
pub use command_item::TerminalCommandItem;
pub use defaults::get_terminal_commands;
