//! PTY (Pseudo-Terminal) management for Script Kit GPUI.
//!
//! This module provides cross-platform PTY creation and lifecycle management
//! using the `portable-pty` crate. It handles spawning shell processes and
//! managing their I/O streams.
//!
//! # Platform Support
//!
//! - **macOS**: Uses native PTY via `/dev/ptmx`
//! - **Linux**: Uses native PTY via `/dev/ptmx` or `/dev/pts`
//! - **Windows**: Uses ConPTY (Windows 10 1809+)

use portable_pty::{Child, MasterPty, PtySize};
use std::io::{Read, Write};
use tracing::{debug, error};

mod io_ops;
mod lifecycle;
#[cfg(test)]
mod tests;

/// Manages a pseudo-terminal session.
///
/// `PtyManager` wraps the portable-pty crate to provide a simplified API
/// for spawning and communicating with shell processes.
pub struct PtyManager {
    /// The master side of the PTY pair
    master: Box<dyn MasterPty + Send>,
    /// The child process running in the PTY
    child: Box<dyn Child + Send + Sync>,
    /// Reader for PTY output (Option to allow taking ownership)
    reader: Option<Box<dyn Read + Send>>,
    /// Writer for PTY input
    writer: Box<dyn Write + Send>,
    /// Current terminal dimensions
    size: PtySize,
}

impl std::fmt::Debug for PtyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyManager")
            .field("size", &self.size)
            .field("master", &"<MasterPty>")
            .field("child", &"<Child>")
            .finish()
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        debug!("PtyManager dropping, cleaning up resources");

        if self.is_running() {
            if let Err(e) = self.kill() {
                error!(error = %e, "Failed to kill child process during cleanup");
            }
        }
    }
}
