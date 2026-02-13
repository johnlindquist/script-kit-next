use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tracing::{debug, info, instrument};

use super::*;

impl PtyManager {
    /// Creates a new PTY manager with the default shell.
    #[instrument(level = "info", name = "pty_spawn_default")]
    pub fn new() -> Result<Self> {
        let shell = Self::detect_shell();
        info!(shell = %shell, "Detected default shell");
        Self::with_command(&shell, &[])
    }

    /// Creates a new PTY manager with specified dimensions.
    #[instrument(level = "info", name = "pty_spawn_sized", fields(cols, rows))]
    pub fn with_size(cols: u16, rows: u16) -> Result<Self> {
        let shell = Self::detect_shell();
        info!(shell = %shell, cols, rows, "Spawning shell with custom size");
        Self::spawn_internal(&shell, &[], cols, rows)
    }

    /// Creates a new PTY manager running a specific command.
    #[instrument(level = "info", name = "pty_spawn_command", fields(cmd = %cmd))]
    pub fn with_command(cmd: &str, args: &[&str]) -> Result<Self> {
        Self::spawn_internal(cmd, args, 80, 24)
    }

    /// Creates a new PTY manager running a specific command with custom dimensions.
    #[instrument(level = "info", name = "pty_spawn_full", fields(cmd = %cmd, cols, rows))]
    pub fn with_command_and_size(cmd: &str, args: &[&str], cols: u16, rows: u16) -> Result<Self> {
        Self::spawn_internal(cmd, args, cols, rows)
    }

    /// Internal spawn implementation.
    fn spawn_internal(cmd: &str, args: &[&str], cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        debug!(
            cols = size.cols,
            rows = size.rows,
            "Creating PTY with dimensions"
        );

        let pair = pty_system
            .openpty(size)
            .context("Failed to open PTY pair")?;

        let mut command = CommandBuilder::new(cmd);
        for arg in args {
            command.arg(*arg);
        }

        #[cfg(unix)]
        {
            let env_vars = Self::unix_spawn_env_allowlist();
            debug!(
                allowlisted_env_count = env_vars.len(),
                "Scrubbing inherited PTY environment before spawn"
            );
            command.env_clear();
            for (key, value) in env_vars {
                command.env(key, value);
            }
        }

        info!(cmd = %cmd, args = ?args, "Spawning child process");

        let child = pair
            .slave
            .spawn_command(command)
            .context("Failed to spawn child process in PTY")?;

        let reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;
        let writer = pair
            .master
            .take_writer()
            .context("Failed to take PTY writer")?;

        info!("PTY spawned successfully");

        Ok(Self {
            master: pair.master,
            child,
            reader: Some(reader),
            writer,
            size,
        })
    }

    #[cfg(unix)]
    fn unix_spawn_env_allowlist() -> Vec<(&'static str, String)> {
        let mut env_vars = vec![
            ("TERM", "xterm-256color".to_string()),
            ("COLORTERM", "truecolor".to_string()),
            ("CLICOLOR_FORCE", "1".to_string()),
        ];

        for key in ["HOME", "USER", "PATH", "SHELL", "TMPDIR", "LANG"] {
            if let Ok(value) = std::env::var(key) {
                env_vars.push((key, value));
            }
        }

        env_vars
    }

    /// Takes ownership of the PTY reader for use in a background thread.
    ///
    /// After calling this, `read()` will return an error.
    pub fn take_reader(&mut self) -> Option<Box<dyn std::io::Read + Send>> {
        self.reader.take()
    }

    /// Detects the default shell for the current platform.
    pub(crate) fn detect_shell() -> String {
        #[cfg(unix)]
        {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        }
        #[cfg(windows)]
        {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::PtyManager;

    #[test]
    fn test_unix_spawn_env_allowlist_contains_expected_keys_when_environment_present() {
        let env_vars = PtyManager::unix_spawn_env_allowlist();
        let env_map: HashMap<&str, &str> = env_vars
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect();

        assert_eq!(env_map.get("TERM"), Some(&"xterm-256color"));
        assert_eq!(env_map.get("COLORTERM"), Some(&"truecolor"));
        assert_eq!(env_map.get("CLICOLOR_FORCE"), Some(&"1"));

        let allowed: HashSet<&str> = [
            "TERM",
            "COLORTERM",
            "CLICOLOR_FORCE",
            "HOME",
            "USER",
            "PATH",
            "SHELL",
            "TMPDIR",
            "LANG",
        ]
        .into_iter()
        .collect();

        for key in env_map.keys() {
            assert!(
                allowed.contains(key),
                "found unexpected env key in PTY allowlist: {key}"
            );
        }

        for key in ["HOME", "USER", "PATH", "SHELL", "TMPDIR", "LANG"] {
            match std::env::var(key) {
                Ok(expected) => assert_eq!(env_map.get(key), Some(&expected.as_str())),
                Err(_) => assert!(!env_map.contains_key(key)),
            }
        }
    }
}
