//! Isolated test environment: temp dirs + env vars for sandboxed app startup.
//!
//! Creates a temp directory with `SK_PATH` and `HOME` subdirs so the app
//! never touches the real user's `~/.scriptkit`.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// An isolated filesystem + environment for one integration test run.
pub struct TestEnvironment {
    /// Root temp directory. Dropped = cleaned up.
    _root: TempDir,
    /// SK_PATH target — the app's kit root.
    pub sk_path: PathBuf,
    /// Fake HOME — catches subsystems that ignore SK_PATH.
    pub home: PathBuf,
    /// Extra env vars the test wants to set.
    pub extra_env: HashMap<String, String>,
}

impl TestEnvironment {
    /// Create a new isolated environment.
    ///
    /// - `SK_PATH` → `<temp>/sk-root` (empty; `ensure_kit_setup()` will populate it)
    /// - `HOME` → `<temp>/home` (catches leaky subsystems)
    pub fn new() -> anyhow::Result<Self> {
        let root = TempDir::new()?;

        let sk_path = root.path().join("sk-root");
        let home = root.path().join("home");

        std::fs::create_dir_all(&sk_path)?;
        std::fs::create_dir_all(&home)?;

        Ok(Self {
            _root: root,
            sk_path,
            home,
            extra_env: HashMap::new(),
        })
    }

    /// Set an additional environment variable for the spawned process.
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extra_env.insert(key.into(), value.into());
    }

    /// Build the full env map for `Command::envs()`.
    pub fn env_map(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Core isolation
        env.insert("SK_PATH".into(), self.sk_path.display().to_string());
        env.insert("HOME".into(), self.home.display().to_string());
        // Windows equivalent of HOME
        env.insert("USERPROFILE".into(), self.home.display().to_string());

        // Structured stderr logging for test assertions
        env.insert("SCRIPT_KIT_AI_LOG".into(), "1".into());

        // Reasonable default log level
        env.insert("RUST_LOG".into(), "info".into());

        // Full backtraces for debugging crashes
        env.insert("RUST_BACKTRACE".into(), "full".into());

        // Inherit PATH so the app can find bun/node
        if let Ok(path) = std::env::var("PATH") {
            env.insert("PATH".into(), path);
        }

        // Windows needs these for basic Win32 API functionality
        for var in [
            "TEMP",
            "TMP",
            "SystemRoot",
            "SYSTEMDRIVE",
            "APPDATA",
            "LOCALAPPDATA",
            "PROGRAMDATA",
            "COMSPEC",
            "WINDIR",
            "OS",
        ] {
            if let Ok(val) = std::env::var(var) {
                env.insert(var.into(), val);
            }
        }

        // Merge test-specific overrides
        for (k, v) in &self.extra_env {
            env.insert(k.clone(), v.clone());
        }

        env
    }

    /// Path to the test-screenshots directory (allowed by captureWindow path policy).
    pub fn screenshots_dir(&self) -> PathBuf {
        // captureWindow validates relative to cwd, so we use the standard name
        PathBuf::from("test-screenshots")
    }

    /// The kit path root.
    pub fn kit_path(&self) -> &Path {
        &self.sk_path
    }
}
