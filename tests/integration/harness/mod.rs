//! Integration test harness for Script Kit GPUI.
//!
//! Two API layers:
//!
//! 1. **`App`** (recommended) — Cypress-style fluent API with implicit waits.
//!    ```rust,no_run
//!    App::start()
//!        .show()
//!        .type_text("hello")
//!        .press("enter")
//!        .should_log("Matched")
//!        .hide();
//!    ```
//!
//! 2. **`TestHarness`** — lower-level builder returning raw `AppProcess`.
//!    ```rust,no_run
//!    let mut app = TestHarness::builder()
//!        .env("AUTO_SUBMIT", "true")
//!        .spawn_and_wait()?;
//!    app.send_show()?;
//!    ```

#![allow(dead_code)]

pub mod app;
pub mod environment;
pub mod log_observer;
pub mod process;
pub mod response_reader;
pub mod suite;

// Re-export the Cypress-style API at the top level.
// `App` is available for direct use, but most tests should use `TestSuite`.
#[allow(unused_imports)]
pub use app::App;
pub use suite::windows_feature_suite;
pub use suite::TestSuite;

use std::collections::HashMap;
use std::time::Duration;

use environment::TestEnvironment;
use process::AppProcess;

/// Default timeout for waiting for the app to become ready.
const DEFAULT_READY_TIMEOUT: Duration = Duration::from_secs(30);

/// Builder for constructing an integration test harness.
pub struct TestHarnessBuilder {
    extra_env: HashMap<String, String>,
    ready_timeout: Duration,
}

impl TestHarnessBuilder {
    fn new() -> Self {
        Self {
            extra_env: HashMap::new(),
            ready_timeout: DEFAULT_READY_TIMEOUT,
        }
    }

    /// Internal constructor used by `AppBuilder`.
    pub(crate) fn new_internal() -> Self {
        Self::new()
    }

    /// Set an environment variable for the spawned app process.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_env.insert(key.into(), value.into());
        self
    }

    /// Override the readiness timeout (default: 30s).
    pub fn ready_timeout(mut self, timeout: Duration) -> Self {
        self.ready_timeout = timeout;
        self
    }

    /// Create the isolated environment, spawn the app, and wait for readiness.
    ///
    /// Returns an `AppProcess` ready to receive commands.
    pub fn spawn_and_wait(self) -> anyhow::Result<AppProcess> {
        let mut env = TestEnvironment::new()?;
        for (k, v) in self.extra_env {
            env.set_env(k, v);
        }

        let app = AppProcess::spawn(env)?;
        app.wait_for_ready(self.ready_timeout)?;
        Ok(app)
    }
}

/// Entry point for building a lower-level integration test.
pub struct TestHarness;

impl TestHarness {
    /// Start building a test harness (lower-level API).
    pub fn builder() -> TestHarnessBuilder {
        TestHarnessBuilder::new()
    }
}
