//! Jest/Cypress-style `describe` / `it` test suite for integration tests.
//!
//! Spawns **one** `App` process for the entire suite and runs named sub-tests
//! sequentially, calling `reset()` between each to clear UI state (~200 ms)
//! instead of restarting the process (~2 s).
//!
//! # Usage
//!
//! ```rust,no_run
//! TestSuite::new("Main Prompt")
//!     .it("shows and filters", |app| {
//!         app.show()
//!             .set_filter("hello")
//!             .should_have_input("hello")
//!             .hide();
//!     })
//!     .it("escape clears then hides", |app| {
//!         app.show()
//!             .set_filter("something")
//!             .escape()
//!             .escape();
//!     })
//!     .run();
//! ```

use std::time::Duration;

use super::app::{App, AppBuilder};

/// A named sub-test closure.
struct TestCase {
    name: String,
    body: Box<dyn FnOnce(&mut App)>,
}

/// Result of a single sub-test.
struct TestResult {
    name: String,
    passed: bool,
    error: Option<String>,
}

/// Jest-style test suite that shares one `App` process across many sub-tests.
pub struct TestSuite {
    suite_name: String,
    cases: Vec<TestCase>,
    builder: Option<AppBuilder>,
}

impl TestSuite {
    /// Create a new test suite with the given describe-level name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            suite_name: name.into(),
            cases: Vec::new(),
            builder: None,
        }
    }

    /// Use a custom `AppBuilder` instead of the default `App::builder()`.
    ///
    /// Useful for setting custom env vars, timeouts, etc.
    pub fn with_builder(mut self, builder: AppBuilder) -> Self {
        self.builder = Some(builder);
        self
    }

    /// Add a named sub-test (like Jest's `it("...", () => { ... })`).
    pub fn it(mut self, name: impl Into<String>, body: impl FnOnce(&mut App) + 'static) -> Self {
        self.cases.push(TestCase {
            name: name.into(),
            body: Box::new(body),
        });
        self
    }

    /// Spawn the app once, run all sub-tests with `reset()` between each,
    /// and panic with a summary if any failed.
    pub fn run(self) {
        let total = self.cases.len();
        assert!(
            total > 0,
            "TestSuite '{}' has no test cases",
            self.suite_name
        );

        // Spawn the app once.
        let mut app = match self.builder {
            Some(builder) => builder.spawn(),
            None => App::start(),
        };

        eprintln!("\n  ── {} ({} tests) ──\n", self.suite_name, total);

        let mut results: Vec<TestResult> = Vec::with_capacity(total);

        for (i, case) in self.cases.into_iter().enumerate() {
            let full_name = format!("{} > {}", self.suite_name, case.name);

            // Reset between tests (skip before the first one).
            if i > 0 {
                // Catch reset panics too — if reset fails, subsequent tests are unreliable.
                let reset_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    app.reset();
                }));
                if let Err(e) = reset_result {
                    let msg = panic_message(&e);
                    eprintln!("    ✗ [RESET FAILED before: {}] {}", case.name, msg);
                    // Mark remaining tests as skipped.
                    results.push(TestResult {
                        name: full_name,
                        passed: false,
                        error: Some(format!("reset failed: {}", msg)),
                    });
                    break;
                }
            }

            // Run the sub-test, catching panics.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (case.body)(&mut app);
            }));

            match result {
                Ok(()) => {
                    eprintln!("    ✓ {}", case.name);
                    results.push(TestResult {
                        name: full_name,
                        passed: true,
                        error: None,
                    });
                }
                Err(e) => {
                    let msg = panic_message(&e);
                    eprintln!("    ✗ {} — {}", case.name, msg);
                    results.push(TestResult {
                        name: full_name,
                        passed: false,
                        error: Some(msg),
                    });
                }
            }
        }

        // Summary.
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.iter().filter(|r| !r.passed).count();

        eprintln!();
        eprintln!(
            "  {} passed, {} failed, {} total",
            passed,
            failed,
            results.len()
        );

        if failed > 0 {
            eprintln!();
            eprintln!("  Failures:");
            for r in &results {
                if !r.passed {
                    eprintln!(
                        "    ✗ {} — {}",
                        r.name,
                        r.error.as_deref().unwrap_or("unknown")
                    );
                }
            }
            eprintln!();
            panic!(
                "{}: {} of {} tests failed",
                self.suite_name,
                failed,
                results.len()
            );
        }
    }
}

/// Convenience: create a suite with custom builder settings for Windows feature tests.
///
/// Sets 30s ready timeout and 10s assert timeout, matching the previous per-test config.
pub fn windows_feature_suite(name: impl Into<String>) -> TestSuite {
    TestSuite::new(name).with_builder(
        App::builder()
            .ready_timeout(Duration::from_secs(30))
            .assert_timeout(Duration::from_secs(10)),
    )
}

/// Extract a human-readable message from a caught panic.
fn panic_message(err: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic".to_string()
    }
}
