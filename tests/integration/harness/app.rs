//! Cypress-style fluent API for driving Script Kit integration tests.
//!
//! Wraps `AppProcess` with a high-level, chainable interface that handles
//! implicit waiting, log assertions, and input simulation.
//!
//! # Usage
//!
//! ```rust,no_run
//! App::start()
//!     .show()
//!     .type_text("hello world")
//!     .press("enter")
//!     .should_log("Matched hello world")
//!     .hide()
//!     .screenshot("after-hello.png");
//! ```

#![allow(dead_code)]

use std::time::Duration;

use super::process::AppProcess;
use super::response_reader::StateSnapshot;
use super::TestHarnessBuilder;

/// Default timeout for implicit waits on log assertions.
const DEFAULT_ASSERT_TIMEOUT: Duration = Duration::from_secs(5);

/// Small delay between individual keystrokes when typing.
const KEYSTROKE_DELAY: Duration = Duration::from_millis(30);

/// Short delay after sending a command, giving the app time to process.
const COMMAND_SETTLE: Duration = Duration::from_millis(100);

// ---------------------------------------------------------------------------
// App — the main Cypress-style test driver
// ---------------------------------------------------------------------------

/// Fluent test driver for Script Kit GPUI.
///
/// Every method returns `&mut Self` so calls can be chained.
/// Methods panic on failure — this is intentional: integration tests should
/// fail fast with clear diagnostics, not propagate `Result` chains.
pub struct App {
    process: AppProcess,
    /// Timeout applied to `should_*` assertions.
    assert_timeout: Duration,
}

impl App {
    // ── Construction ────────────────────────────────────────────────

    /// Start the app with default settings in an isolated environment.
    ///
    /// Blocks until the app reports "Application ready".
    /// Panics if the app fails to start.
    pub fn start() -> Self {
        Self::builder().spawn()
    }

    /// Start building with custom settings.
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    fn from_process(process: AppProcess) -> Self {
        Self {
            process,
            assert_timeout: DEFAULT_ASSERT_TIMEOUT,
        }
    }

    // ── Configuration ──────────────────────────────────────────────

    /// Set the timeout for all subsequent `should_*` assertions.
    pub fn set_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.assert_timeout = timeout;
        self
    }

    // ── Process reuse ──────────────────────────────────────────────

    /// Reset the app to a clean state without restarting the process.
    ///
    /// Hides the window, clears the filter, and waits for settle.
    /// Use between test cases that share a single app process for speed.
    ///
    /// ```rust,no_run
    /// app.reset().show(); // start fresh
    /// ```
    pub fn reset(&mut self) -> &mut Self {
        self.process.send_hide().ok();
        self.process.send_filter("").ok();
        self.settle();
        self
    }

    // ── Window control ─────────────────────────────────────────────

    /// Show the main prompt window.
    pub fn show(&mut self) -> &mut Self {
        self.process.send_show().expect("send_show failed");
        self.settle();
        self
    }

    /// Hide the main prompt window.
    pub fn hide(&mut self) -> &mut Self {
        self.process.send_hide().expect("send_hide failed");
        self.settle();
        self
    }

    // ── Text input ─────────────────────────────────────────────────

    /// Type text character-by-character into the focused input.
    ///
    /// Each character is sent as a separate `simulateKey` command with a small
    /// delay between keystrokes, just like a real user typing.
    ///
    /// ```rust,no_run
    /// app.type_text("hello world");
    /// ```
    pub fn type_text(&mut self, text: &str) -> &mut Self {
        for ch in text.chars() {
            let key = ch.to_string();
            self.process
                .send_key(&key, &[])
                .unwrap_or_else(|e| panic!("type_text: failed to send key '{}': {}", ch, e));
            std::thread::sleep(KEYSTROKE_DELAY);
        }
        self.settle();
        self
    }

    /// Set the filter/input text directly (replaces current content).
    ///
    /// Unlike `type_text`, this sets the full value at once via `setFilter`.
    /// Faster, but doesn't trigger per-keystroke events.
    ///
    /// ```rust,no_run
    /// app.set_filter("exact search text");
    /// ```
    pub fn set_filter(&mut self, text: &str) -> &mut Self {
        self.process
            .send_filter(text)
            .unwrap_or_else(|e| panic!("set_filter: failed: {}", e));
        self.settle();
        self
    }

    /// Clear the current filter/input text.
    pub fn clear(&mut self) -> &mut Self {
        self.set_filter("")
    }

    // ── Keyboard ───────────────────────────────────────────────────

    /// Press a single key (e.g. "enter", "escape", "tab", "up", "down").
    ///
    /// ```rust,no_run
    /// app.press("enter");
    /// app.press("escape");
    /// app.press("tab");
    /// ```
    pub fn press(&mut self, key: &str) -> &mut Self {
        self.process
            .send_key(key, &[])
            .unwrap_or_else(|e| panic!("press: failed to send key '{}': {}", key, e));
        self.settle();
        self
    }

    /// Press a key with modifiers.
    ///
    /// Modifiers: "cmd"/"meta", "shift", "alt"/"option", "ctrl"/"control"
    ///
    /// ```rust,no_run
    /// app.press_with_modifiers("k", &["cmd"]);       // Cmd+K
    /// app.press_with_modifiers("enter", &["shift"]);  // Shift+Enter
    /// app.press_with_modifiers("a", &["ctrl"]);       // Ctrl+A
    /// ```
    pub fn press_with_modifiers(&mut self, key: &str, modifiers: &[&str]) -> &mut Self {
        self.process.send_key(key, modifiers).unwrap_or_else(|e| {
            panic!(
                "press_with_modifiers: failed to send {}+{}: {}",
                modifiers.join("+"),
                key,
                e
            )
        });
        self.settle();
        self
    }

    /// Press arrow up.
    pub fn arrow_up(&mut self) -> &mut Self {
        self.press("up")
    }

    /// Press arrow down.
    pub fn arrow_down(&mut self) -> &mut Self {
        self.press("down")
    }

    /// Press Enter (confirm/submit).
    pub fn enter(&mut self) -> &mut Self {
        self.press("enter")
    }

    /// Press Escape (cancel/back).
    pub fn escape(&mut self) -> &mut Self {
        self.press("escape")
    }

    /// Press Tab.
    pub fn tab(&mut self) -> &mut Self {
        self.press("tab")
    }

    /// Press Backspace.
    pub fn backspace(&mut self) -> &mut Self {
        self.press("backspace")
    }

    // ── Script execution ───────────────────────────────────────────

    /// Run a script by path.
    ///
    /// ```rust,no_run
    /// app.run_script("path/to/script.ts");
    /// ```
    pub fn run_script(&mut self, path: &str) -> &mut Self {
        self.process
            .send_run(path)
            .unwrap_or_else(|e| panic!("run_script: failed to run '{}': {}", path, e));
        self.settle();
        self
    }

    // ── Assertions ─────────────────────────────────────────────────

    /// Assert that a log line matching `pattern` appears within the assert timeout.
    ///
    /// Searches both the raw line and parsed message portion.
    ///
    /// ```rust,no_run
    /// app.should_log("Processing external command");
    /// ```
    pub fn should_log(&mut self, pattern: &str) -> &mut Self {
        self.process
            .logs
            .wait_for_log(pattern, self.assert_timeout)
            .unwrap_or_else(|e| panic!("should_log({:?}): {}", pattern, e));
        self
    }

    /// Assert that a log line matching `pattern` appears within a custom timeout.
    ///
    /// ```rust,no_run
    /// app.should_log_within("Script finished", Duration::from_secs(30));
    /// ```
    pub fn should_log_within(&mut self, pattern: &str, timeout: Duration) -> &mut Self {
        self.process
            .logs
            .wait_for_log(pattern, timeout)
            .unwrap_or_else(|e| panic!("should_log_within({:?}, {:?}): {}", pattern, timeout, e));
        self
    }

    /// Assert that the kit directory structure was created.
    pub fn should_have_kit_dirs(&mut self) -> &mut Self {
        assert!(
            self.process.env.kit_path().join("kit").exists(),
            "Expected kit/ directory under SK_PATH"
        );
        assert!(
            self.process.env.kit_path().join("sdk").exists(),
            "Expected sdk/ directory under SK_PATH"
        );
        self
    }

    /// Assert that a file exists relative to SK_PATH.
    pub fn should_have_file(&mut self, relative_path: &str) -> &mut Self {
        let path = self.process.env.kit_path().join(relative_path);
        assert!(path.exists(), "Expected file: {}", path.display());
        self
    }

    // ── State queries (structured) ───────────────────────────────────

    /// Query the app's current state snapshot.
    ///
    /// Sends `getState` via stdin and waits for a structured JSON response
    /// on stdout. Returns the parsed `StateSnapshot`.
    ///
    /// ```rust,no_run
    /// let state = app.state();
    /// assert_eq!(state.prompt_type, "none");
    /// ```
    pub fn state(&mut self) -> StateSnapshot {
        self.process
            .get_state(self.assert_timeout)
            .expect("state query failed")
    }

    /// Sends `getChatActionsState` via stdin and waits for a structured JSON
    /// response. Returns the parsed `ChatActionsSnapshot`.
    pub fn chat_actions_state(&mut self) -> super::response_reader::ChatActionsSnapshot {
        self.process
            .get_chat_actions_state(self.assert_timeout)
            .expect("chat actions state query failed")
    }

    // ── Chat actions assertions ──────────────────────────────────────

    /// Assert that the current view is the ChatPrompt.
    pub fn should_be_chat_view(&mut self) -> &mut Self {
        let state = self.chat_actions_state();
        assert!(state.is_chat_view, "expected to be in chat view");
        self
    }

    /// Assert the actions popup is open.
    pub fn should_have_actions_popup_open(&mut self) -> &mut Self {
        let state = self.chat_actions_state();
        assert!(
            state.actions_popup_open,
            "expected actions popup to be open"
        );
        self
    }

    /// Assert the actions popup is closed.
    pub fn should_have_actions_popup_closed(&mut self) -> &mut Self {
        let state = self.chat_actions_state();
        assert!(
            !state.actions_popup_open,
            "expected actions popup to be closed"
        );
        self
    }

    /// Assert that a specific action ID is present in the actions list.
    pub fn should_have_action(&mut self, action_id: &str) -> &mut Self {
        let state = self.chat_actions_state();
        assert!(
            state.action_ids.iter().any(|id| id == action_id),
            "expected action {:?} in list, got: {:?}",
            action_id,
            state.action_ids
        );
        self
    }

    /// Assert that a specific action ID is NOT present in the actions list.
    pub fn should_not_have_action(&mut self, action_id: &str) -> &mut Self {
        let state = self.chat_actions_state();
        assert!(
            !state.action_ids.iter().any(|id| id == action_id),
            "expected action {:?} to NOT be in list, but found it in: {:?}",
            action_id,
            state.action_ids
        );
        self
    }

    /// Assert at least N actions are available.
    pub fn should_have_at_least_n_actions(&mut self, n: usize) -> &mut Self {
        let state = self.chat_actions_state();
        assert!(
            state.action_ids.len() >= n,
            "expected at least {} actions, got {} ({:?})",
            n,
            state.action_ids.len(),
            state.action_ids
        );
        self
    }

    /// Assert message count.
    pub fn should_have_message_count(&mut self, expected: usize) -> &mut Self {
        let state = self.chat_actions_state();
        assert_eq!(
            state.message_count, expected,
            "expected {} messages, got {}",
            expected, state.message_count
        );
        self
    }

    // ── Prompt assertions (Script Kit terminology) ─────────────────

    /// Assert the current prompt type (e.g. "none" for script list, "arg", "div").
    ///
    /// ```rust,no_run
    /// app.should_be_prompt_type("none"); // main script list
    /// app.should_be_prompt_type("arg");  // arg prompt from a script
    /// ```
    pub fn should_be_prompt_type(&mut self, expected: &str) -> &mut Self {
        let state = self.state();
        assert_eq!(
            state.prompt_type, expected,
            "expected prompt_type {:?}, got {:?}",
            expected, state.prompt_type
        );
        self
    }

    /// Assert the current input/filter text matches exactly.
    ///
    /// ```rust,no_run
    /// app.set_filter("hello").should_have_input("hello");
    /// ```
    pub fn should_have_input(&mut self, expected: &str) -> &mut Self {
        let state = self.state();
        assert_eq!(
            state.input_value, expected,
            "expected input {:?}, got {:?}",
            expected, state.input_value
        );
        self
    }

    /// Assert the input/filter text contains a substring.
    pub fn should_have_input_containing(&mut self, substring: &str) -> &mut Self {
        let state = self.state();
        assert!(
            state.input_value.contains(substring),
            "expected input to contain {:?}, got {:?}",
            substring,
            state.input_value
        );
        self
    }

    /// Assert the total number of choices/items available (before filtering).
    ///
    /// ```rust,no_run
    /// app.should_have_choices(42);
    /// ```
    pub fn should_have_choices(&mut self, expected: usize) -> &mut Self {
        let state = self.state();
        assert_eq!(
            state.choice_count, expected,
            "expected {} total choices, got {}",
            expected, state.choice_count
        );
        self
    }

    /// Assert the total number of choices is at least `min`.
    pub fn should_have_choices_at_least(&mut self, min: usize) -> &mut Self {
        let state = self.state();
        assert!(
            state.choice_count >= min,
            "expected at least {} total choices, got {}",
            min,
            state.choice_count
        );
        self
    }

    /// Assert the number of visible choices after filtering.
    ///
    /// ```rust,no_run
    /// app.set_filter("git").should_have_visible_choices(3);
    /// ```
    pub fn should_have_visible_choices(&mut self, expected: usize) -> &mut Self {
        let state = self.state();
        assert_eq!(
            state.visible_choice_count, expected,
            "expected {} visible choices, got {}",
            expected, state.visible_choice_count
        );
        self
    }

    /// Assert that filtering reduced the visible choice count below the total.
    pub fn should_have_filtered_results(&mut self) -> &mut Self {
        let state = self.state();
        assert!(
            state.visible_choice_count < state.choice_count,
            "expected filtered results (visible < total), got visible={} total={}",
            state.visible_choice_count,
            state.choice_count
        );
        self
    }

    /// Assert the currently selected item's index.
    ///
    /// ```rust,no_run
    /// app.arrow_down().should_have_selected_index(1);
    /// ```
    pub fn should_have_selected_index(&mut self, expected: i32) -> &mut Self {
        let state = self.state();
        assert_eq!(
            state.selected_index, expected,
            "expected selected_index {}, got {}",
            expected, state.selected_index
        );
        self
    }

    /// Assert the currently selected item's name/value matches exactly.
    ///
    /// ```rust,no_run
    /// app.should_have_selected("Google Chrome");
    /// ```
    pub fn should_have_selected(&mut self, expected: &str) -> &mut Self {
        let state = self.state();
        let actual = state.selected_value.as_deref().unwrap_or("<none>");
        assert_eq!(
            actual, expected,
            "expected selected value {:?}, got {:?}",
            expected, actual
        );
        self
    }

    /// Assert the selected item's name contains a substring.
    pub fn should_have_selected_containing(&mut self, substring: &str) -> &mut Self {
        let state = self.state();
        let actual = state.selected_value.as_deref().unwrap_or("");
        assert!(
            actual.contains(substring),
            "expected selected value to contain {:?}, got {:?}",
            substring,
            actual
        );
        self
    }

    /// Assert the main window is visible.
    pub fn should_be_visible(&mut self) -> &mut Self {
        let state = self.state();
        assert!(
            state.window_visible,
            "expected window to be visible, but it is hidden"
        );
        self
    }

    /// Assert the main window is hidden.
    pub fn should_be_hidden(&mut self) -> &mut Self {
        let state = self.state();
        assert!(
            !state.window_visible,
            "expected window to be hidden, but it is visible"
        );
        self
    }

    /// Assert the input is focused.
    pub fn should_be_focused(&mut self) -> &mut Self {
        let state = self.state();
        assert!(
            state.is_focused,
            "expected input to be focused, but it is not"
        );
        self
    }

    /// Assert no input is focused.
    pub fn should_not_be_focused(&mut self) -> &mut Self {
        let state = self.state();
        assert!(
            !state.is_focused,
            "expected input to not be focused, but it is"
        );
        self
    }

    // ── Screenshots ────────────────────────────────────────────────

    /// Capture a screenshot of the app window.
    ///
    /// The file is saved relative to the working directory (must be under
    /// `test-screenshots/` per the app's path validation policy).
    ///
    /// ```rust,no_run
    /// app.screenshot("test-screenshots/after-filter.png");
    /// ```
    pub fn screenshot(&mut self, path: &str) -> &mut Self {
        let cmd = serde_json::json!({
            "type": "captureWindow",
            "title": "Script Kit",
            "path": path,
        });
        self.process
            .send_raw(&cmd.to_string())
            .unwrap_or_else(|e| panic!("screenshot: failed: {}", e));

        // Wait for the screenshot to be saved
        self.process
            .logs
            .wait_for_log("Screenshot saved", self.assert_timeout)
            .unwrap_or_else(|e| panic!("screenshot: never got confirmation: {}", e));
        self
    }

    // ── Raw access (escape hatch) ──────────────────────────────────

    /// Send a raw JSON command. Escape hatch for commands not covered by the API.
    ///
    /// ```rust,no_run
    /// app.send(r#"{"type":"triggerBuiltin","name":"clipboardHistory"}"#);
    /// ```
    pub fn send(&mut self, json: &str) -> &mut Self {
        self.process
            .send_raw(json)
            .unwrap_or_else(|e| panic!("send: failed: {}", e));
        self.settle();
        self
    }

    /// Access the underlying log observer for advanced assertions.
    pub fn logs(&self) -> &super::log_observer::LogObserver {
        &self.process.logs
    }

    /// Access the test environment (paths, etc).
    pub fn env(&self) -> &super::environment::TestEnvironment {
        &self.process.env
    }

    // ── AI Window ───────────────────────────────────────────────────

    /// Open the separate AI Chat window with mock data.
    ///
    /// Waits up to 5 seconds for the AI window to become ready (the window
    /// creation, mock-data insertion, and first render cycle can take ~1.5 s
    /// on Windows).
    pub fn open_ai_with_mock_data(&mut self) -> &mut Self {
        self.process
            .send_open_ai_with_mock_data()
            .expect("open_ai_with_mock_data failed");
        // Poll until the AI window reports as open
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            std::thread::sleep(Duration::from_millis(200));
            if let Ok(state) = self
                .process
                .get_ai_command_bar_state(Duration::from_secs(2))
            {
                if state.ai_window_open {
                    break;
                }
            }
            if std::time::Instant::now() > deadline {
                panic!("AI window did not become ready within 5 s");
            }
        }
        // Extra settle for render to complete
        std::thread::sleep(Duration::from_millis(300));
        self
    }

    /// Open the AI window's command bar (Ctrl+K equivalent).
    ///
    /// If the command bar is already open, closes it first (via Escape) so
    /// that a fresh dialog with reset selection is created.
    /// Polls until the command bar actually reports as open (the
    /// `ShowCommandBar` stdin command is deferred to the next render cycle).
    pub fn show_ai_command_bar(&mut self) -> &mut Self {
        // Close first if already open (CommandBar::open() is a no-op when open)
        if let Ok(state) = self
            .process
            .get_ai_command_bar_state(Duration::from_secs(2))
        {
            if state.command_bar_open {
                self.ai_key("escape", &[]);
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        self.process
            .send_show_ai_command_bar()
            .expect("show_ai_command_bar failed");
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            std::thread::sleep(Duration::from_millis(100));
            if let Ok(state) = self
                .process
                .get_ai_command_bar_state(Duration::from_secs(2))
            {
                if state.command_bar_open {
                    return self;
                }
            }
            if std::time::Instant::now() > deadline {
                panic!("AI command bar did not open within 3 s after showAiCommandBar");
            }
        }
    }

    /// Simulate a key press in the AI window.
    pub fn ai_key(&mut self, key: &str, modifiers: &[&str]) -> &mut Self {
        self.process
            .send_ai_key(key, modifiers)
            .unwrap_or_else(|e| panic!("ai_key: failed: {}", e));
        self.settle();
        self
    }

    /// Simulate arrow down in the AI window.
    pub fn ai_arrow_down(&mut self) -> &mut Self {
        self.ai_key("down", &[])
    }

    /// Simulate arrow up in the AI window.
    pub fn ai_arrow_up(&mut self) -> &mut Self {
        self.ai_key("up", &[])
    }

    /// Query the AI window's command bar state.
    pub fn ai_command_bar_state(&mut self) -> super::response_reader::AiCommandBarSnapshot {
        self.process
            .get_ai_command_bar_state(self.assert_timeout)
            .expect("ai command bar state query failed")
    }

    // ── AI Command Bar assertions ──────────────────────────────────

    /// Assert the AI command bar is open.
    pub fn should_have_ai_command_bar_open(&mut self) -> &mut Self {
        let state = self.ai_command_bar_state();
        assert!(state.ai_window_open, "expected AI window to be open");
        assert!(state.command_bar_open, "expected AI command bar to be open");
        self
    }

    /// Assert the AI command bar is closed.
    pub fn should_have_ai_command_bar_closed(&mut self) -> &mut Self {
        let state = self.ai_command_bar_state();
        assert!(
            !state.command_bar_open,
            "expected AI command bar to be closed"
        );
        self
    }

    /// Assert the AI command bar has at least N actions.
    pub fn should_have_ai_command_bar_actions_at_least(&mut self, n: usize) -> &mut Self {
        let state = self.ai_command_bar_state();
        assert!(
            state.action_ids.len() >= n,
            "expected at least {} command bar actions, got {} ({:?})",
            n,
            state.action_ids.len(),
            state.action_ids
        );
        self
    }

    /// Assert the AI command bar's selected index equals `expected`.
    pub fn should_have_ai_command_bar_selected_index(&mut self, expected: i32) -> &mut Self {
        let state = self.ai_command_bar_state();
        assert_eq!(
            state.selected_index, expected,
            "expected AI command bar selected_index {}, got {} (action_ids: {:?})",
            expected, state.selected_index, state.action_ids
        );
        self
    }

    /// Assert the AI command bar's selected action ID matches exactly.
    pub fn should_have_ai_command_bar_selected_action(&mut self, expected: &str) -> &mut Self {
        let state = self.ai_command_bar_state();
        let actual = state.selected_action_id.as_deref().unwrap_or("<none>");
        assert_eq!(
            actual, expected,
            "expected AI command bar selected action {:?}, got {:?}",
            expected, actual
        );
        self
    }

    // ── Timing ─────────────────────────────────────────────────────

    /// Wait for a fixed duration. Use sparingly — prefer `should_log` assertions.
    pub fn wait(&mut self, duration: Duration) -> &mut Self {
        std::thread::sleep(duration);
        self
    }

    // ── Internal ───────────────────────────────────────────────────

    /// Small delay after a command, giving the app time to process the event loop.
    fn settle(&self) {
        std::thread::sleep(COMMAND_SETTLE);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // AppProcess::drop handles killing the process
        eprintln!("[App] Tearing down.");
    }
}

// ---------------------------------------------------------------------------
// AppBuilder — configure before starting
// ---------------------------------------------------------------------------

/// Builder for `App` with custom environment and timeouts.
///
/// ```rust,no_run
/// let mut app = App::builder()
///     .env("AUTO_SUBMIT", "true")
///     .ready_timeout(Duration::from_secs(15))
///     .assert_timeout(Duration::from_secs(10))
///     .spawn();
/// ```
pub struct AppBuilder {
    harness_builder: TestHarnessBuilder,
    assert_timeout: Duration,
}

impl AppBuilder {
    fn new() -> Self {
        Self {
            harness_builder: TestHarnessBuilder::new_internal(),
            assert_timeout: DEFAULT_ASSERT_TIMEOUT,
        }
    }

    /// Set an environment variable for the spawned app process.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.harness_builder = self.harness_builder.env(key, value);
        self
    }

    /// Override the readiness timeout (default: 30s).
    pub fn ready_timeout(mut self, timeout: Duration) -> Self {
        self.harness_builder = self.harness_builder.ready_timeout(timeout);
        self
    }

    /// Override the default assertion timeout for `should_*` methods (default: 5s).
    pub fn assert_timeout(mut self, timeout: Duration) -> Self {
        self.assert_timeout = timeout;
        self
    }

    /// Build, spawn, wait for readiness, and return the fluent `App`.
    ///
    /// Panics if the app fails to start.
    pub fn spawn(self) -> App {
        let assert_timeout = self.assert_timeout;
        let process = self
            .harness_builder
            .spawn_and_wait()
            .expect("App failed to start");
        let mut app = App::from_process(process);
        app.assert_timeout = assert_timeout;
        app
    }
}
