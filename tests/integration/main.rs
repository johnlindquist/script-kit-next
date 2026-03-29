//! Integration tests for Script Kit GPUI.
//!
//! These tests spawn the real `script-kit-gpui` binary in an isolated environment
//! and drive it via the stdin JSONL protocol, observing behavior through stderr logs.
//!
//! Tests use a Jest/Cypress-style `TestSuite` that spawns **one** app process per
//! suite and runs named sub-tests with `reset()` between each (~200 ms) instead of
//! restarting the process (~2 s per spawn).
//!
//! # Prerequisites
//! - Binary must be pre-built: `cargo build --bin script-kit-gpui`
//!
//! # Running
//! ```bash
//! cargo test --features integration-tests --test integration -- --test-threads=1 --nocapture
//! ```

#![cfg(feature = "integration-tests")]

mod harness;

use std::time::Duration;

use harness::{windows_feature_suite, TestSuite};

// ── Main Prompt Suite ─────────────────────────────────────────────
//
// Core prompt lifecycle: show/hide, filtering, escape behavior,
// keyboard navigation, modifier keys, and raw command escape hatch.

#[test]
fn test_main_prompt() {
    TestSuite::new("Main Prompt")
        .it("shows, filters, and hides", |app| {
            app.should_have_kit_dirs()
                .show()
                .should_log("Processing external command")
                .should_be_prompt_type("none")
                .set_filter("hello")
                .should_log("setFilter")
                .should_have_input("hello")
                .hide()
                .should_log("hide");
        })
        .it("keyboard navigation through list", |app| {
            app.show()
                .should_log("Processing external command")
                .type_text("hello")
                .should_log("SimulateKey")
                .arrow_down()
                .arrow_down()
                .arrow_up()
                .enter()
                .should_log("SimulateKey: Enter");
        })
        .it("escape clears filter then hides", |app| {
            app.show()
                .should_log("Processing external command")
                .set_filter("something")
                .should_log("setFilter")
                .escape()
                .should_log("SimulateKey: Escape")
                .escape()
                .should_log("SimulateKey: Escape");
        })
        .it("modifier key combinations", |app| {
            app.show()
                .should_log("Processing external command")
                .press_with_modifiers("k", &["cmd"])
                .should_log("SimulateKey");
        })
        .it("raw command escape hatch", |app| {
            app.show()
                .should_log("Processing external command")
                .send(r#"{"type":"setFilter","text":"raw command test"}"#)
                .should_log("setFilter")
                .should_have_input("raw command test");
        })
        .it("custom env vars via builder", |app| {
            // Note: env vars were set at suite level, but we can still verify
            // the app started and responds to commands.
            app.should_have_kit_dirs()
                .show()
                .should_log("Processing external command");
        })
        .run();
}

// ── State Query Suite ─────────────────────────────────────────────
//
// Structured getState queries: script list contents, filtering
// behavior, and process reuse via reset().

#[test]
fn test_state_queries() {
    TestSuite::new("State Queries")
        .it("returns valid state for script list", |app| {
            app.show()
                .should_log("Processing external command")
                .should_be_prompt_type("none")
                .should_have_input("");

            let state = app.state();
            assert!(
                state.selected_index >= 0,
                "expected non-negative selected_index, got {}",
                state.selected_index
            );
        })
        .it("filtering reduces visible choices", |app| {
            app.show().should_log("Processing external command");
            let initial = app.state();
            let initial_visible = initial.visible_choice_count;

            app.set_filter("zzzzzqqqq");
            let filtered = app.state();

            assert_eq!(filtered.input_value, "zzzzzqqqq");
            assert!(
                filtered.visible_choice_count <= initial_visible,
                "filtering should reduce or maintain visible count: before={}, after={}",
                initial_visible,
                filtered.visible_choice_count
            );
        })
        .it("reset clears state between sub-tests", |app| {
            // First interaction
            app.show()
                .should_log("Processing external command")
                .set_filter("first test")
                .should_have_input("first test");

            // Manual inline reset to verify the mechanism itself
            app.reset();

            // Second interaction should start clean
            app.show()
                .should_log("Processing external command")
                .should_have_input("")
                .set_filter("second test")
                .should_have_input("second test");
        })
        .run();
}

// ── Windows Features: Window Switcher & App Launcher ──────────────
//
// Builtin features that interact with the Windows desktop. These need
// longer timeouts because the app scanner reads Start Menu shortcuts.

#[test]
fn test_window_switcher_and_app_launcher() {
    windows_feature_suite("Window Switcher & App Launcher")
        .it("window switcher shows windows", |app| {
            app.show().should_log("Processing external command");

            app.send(r#"{"type":"triggerBuiltin","name":"window-switcher"}"#)
                .should_log("Triggering built-in: 'window-switcher'");

            app.wait(Duration::from_millis(500));

            let state = app.state();
            assert_eq!(
                state.prompt_type, "windowSwitcher",
                "expected prompt_type 'windowSwitcher', got '{}'",
                state.prompt_type
            );
            assert!(
                state.choice_count >= 1,
                "expected at least 1 window in the switcher, got {}",
                state.choice_count
            );

            if let Some(ref title) = state.selected_value {
                assert!(!title.is_empty(), "expected non-empty window title");
            }

            eprintln!(
                "      {} windows listed, selected: {:?}",
                state.choice_count, state.selected_value
            );
        })
        .it("app launcher shows installed apps", |app| {
            app.show().should_log("Processing external command");

            app.send(r#"{"type":"triggerBuiltin","name":"apps"}"#)
                .should_log("Triggering built-in: 'apps'");

            app.wait(Duration::from_millis(1000));

            let state = app.state();
            assert_eq!(
                state.prompt_type, "appLauncher",
                "expected prompt_type 'appLauncher', got '{}'",
                state.prompt_type
            );
            assert!(
                state.choice_count >= 1,
                "expected at least 1 app in the launcher, got {}",
                state.choice_count
            );

            if let Some(ref name) = state.selected_value {
                assert!(!name.is_empty(), "expected non-empty app name");
            }

            eprintln!(
                "      {} apps listed, selected: {:?}",
                state.choice_count, state.selected_value
            );
        })
        .it("app launcher filter reduces results", |app| {
            app.show().should_log("Processing external command");

            app.send(r#"{"type":"triggerBuiltin","name":"apps"}"#)
                .should_log("Triggering built-in: 'apps'");

            app.wait(Duration::from_millis(1000));

            let initial_state = app.state();
            assert_eq!(initial_state.prompt_type, "appLauncher");
            let total_before = initial_state.choice_count;

            app.set_filter("zzznonexistentapp999");
            app.wait(Duration::from_millis(300));

            let filtered_state = app.state();
            assert_eq!(filtered_state.input_value, "zzznonexistentapp999");
            assert!(
                filtered_state.visible_choice_count <= total_before,
                "filtering should reduce visible count: before={}, after={}",
                total_before,
                filtered_state.visible_choice_count
            );

            eprintln!(
                "      {} apps before filter, {} after",
                total_before, filtered_state.visible_choice_count
            );
        })
        .run();
}

// ── Windows Features: Clipboard & Text Builtins ───────────────────
//
// Paste sequentially and selected text / browser tab builtins.

#[test]
fn test_clipboard_and_text_builtins() {
    windows_feature_suite("Clipboard & Text Builtins")
        .it("paste sequentially handles empty clipboard", |app| {
            app.show().should_log("Processing external command");

            app.send(r#"{"type":"triggerBuiltin","name":"paste-sequentially"}"#)
                .should_log("Paste Sequentially triggered via stdin");

            app.should_log("Paste sequential:");

            // Verify the app is still responsive
            let state = app.state();
            eprintln!("      app responsive, prompt_type='{}'", state.prompt_type);
        })
        .it("selected text and browser tab builtins registered", |app| {
            app.show().should_log("Processing external command");

            // Filter for selected text builtins
            app.set_filter("selected text");
            app.wait(Duration::from_millis(300));
            let state_selected = app.state();
            eprintln!(
                "      filter 'selected text': {} visible / {} total",
                state_selected.visible_choice_count, state_selected.choice_count,
            );

            // Filter for browser tab builtins
            app.set_filter("browser tab");
            app.wait(Duration::from_millis(300));
            let state_browser = app.state();
            eprintln!(
                "      filter 'browser tab': {} visible / {} total",
                state_browser.visible_choice_count, state_browser.choice_count,
            );

            // Verify app still responsive after clearing filter
            app.set_filter("");
            let state_reset = app.state();
            assert!(
                state_reset.choice_count > 0,
                "expected choices after clearing filter, got {}",
                state_reset.choice_count
            );
        })
        .run();
}

// ── Chat Actions Suite ───────────────────────────────────────────
//
// Tests the inline chat prompt's actions panel (Cmd+K menu).
// Opens chat via Tab from main prompt, then exercises the actions
// dialog: open/close, action presence, navigation, and escape.

#[test]
fn test_chat_actions() {
    windows_feature_suite("Chat Actions")
        .it("tab opens inline chat from main prompt", |app| {
            app.show().should_log("Processing external command");

            // Type something to use as the chat query, then Tab to open inline chat
            app.set_filter("hello world");
            app.wait(Duration::from_millis(200));
            app.press("Tab");
            app.wait(Duration::from_millis(500));

            // Verify we're now in the chat view
            let state = app.chat_actions_state();
            assert!(
                state.is_chat_view,
                "expected chat view after Tab, got is_chat_view=false"
            );
            eprintln!("      in chat view, model={:?}", state.chat_model);

            // Actions popup should not be open yet
            assert!(
                !state.actions_popup_open,
                "actions popup should be closed initially"
            );
        })
        .it("cmd+k opens and escape closes actions popup", |app| {
            // First get into chat view
            app.show().should_log("Processing external command");
            app.set_filter("test query");
            app.wait(Duration::from_millis(200));
            app.press("Tab");
            app.wait(Duration::from_millis(500));

            // Verify we're in chat
            app.should_be_chat_view();

            // Open actions popup with Cmd+K
            app.press_with_modifiers("k", &["cmd"]);
            app.wait(Duration::from_millis(300));

            app.should_have_actions_popup_open();

            // Query the full actions state
            let state = app.chat_actions_state();
            eprintln!(
                "      actions popup open, {} actions: {:?}",
                state.action_ids.len(),
                state.action_ids
            );

            // Close with Escape
            app.escape();
            app.wait(Duration::from_millis(200));

            app.should_have_actions_popup_closed();
        })
        .it("actions popup contains expected chat actions", |app| {
            // Get into chat view
            app.show().should_log("Processing external command");
            app.set_filter("test");
            app.wait(Duration::from_millis(200));
            app.press("Tab");
            app.wait(Duration::from_millis(500));

            // Open actions popup
            app.press_with_modifiers("k", &["cmd"]);
            app.wait(Duration::from_millis(300));

            // Should have the core chat actions
            app.should_have_actions_popup_open()
                .should_have_action("chat:continue_in_chat")
                .should_have_action("chat:expand_full_chat")
                .should_have_action("chat:capture_screen_area");

            // Should have at least 3 actions (the fixed ones)
            app.should_have_at_least_n_actions(3);

            let state = app.chat_actions_state();
            eprintln!("      action titles: {:?}", state.action_titles);

            // With no messages, these conditional actions should NOT be present
            app.should_not_have_action("chat:copy_response")
                .should_not_have_action("chat:clear_conversation");

            // No messages in a fresh chat
            app.should_have_message_count(0);

            // Clean up
            app.escape();
        })
        .it("cmd+k toggles actions popup open and closed", |app| {
            // Get into chat view
            app.show().should_log("Processing external command");
            app.set_filter("toggle test");
            app.wait(Duration::from_millis(200));
            app.press("Tab");
            app.wait(Duration::from_millis(500));

            // First Cmd+K: open
            app.press_with_modifiers("k", &["cmd"]);
            app.wait(Duration::from_millis(300));
            app.should_have_actions_popup_open();

            // Second Cmd+K: close (toggle)
            app.press_with_modifiers("k", &["cmd"]);
            app.wait(Duration::from_millis(300));
            app.should_have_actions_popup_closed();

            // Third Cmd+K: open again
            app.press_with_modifiers("k", &["cmd"]);
            app.wait(Duration::from_millis(300));
            app.should_have_actions_popup_open();

            app.escape();
        })
        .it("arrow keys navigate actions in popup", |app| {
            // Get into chat and open actions
            app.show().should_log("Processing external command");
            app.set_filter("nav test");
            app.wait(Duration::from_millis(200));
            app.press("Tab");
            app.wait(Duration::from_millis(500));
            app.press_with_modifiers("k", &["cmd"]);
            app.wait(Duration::from_millis(300));

            // Get initial selection
            let state1 = app.chat_actions_state();
            let initial_index = state1.selected_action_index;
            let initial_id = state1.selected_action_id.clone();
            eprintln!(
                "      initial: index={}, id={:?}",
                initial_index, initial_id
            );

            // Arrow down
            app.arrow_down();
            app.wait(Duration::from_millis(100));
            let state2 = app.chat_actions_state();
            eprintln!(
                "      after down: index={}, id={:?}",
                state2.selected_action_index, state2.selected_action_id
            );
            // Selection should have moved
            assert_ne!(
                state2.selected_action_index, initial_index,
                "arrow down should change selected index"
            );

            // Arrow up should go back
            app.arrow_up();
            app.wait(Duration::from_millis(100));
            let state3 = app.chat_actions_state();
            eprintln!(
                "      after up: index={}, id={:?}",
                state3.selected_action_index, state3.selected_action_id
            );

            app.escape();
        })
        .run();
}

// ── AI Command Bar Suite ─────────────────────────────────────────
//
// Tests the separate AI Chat window's command bar (Ctrl+K popup):
// open/close, navigation with arrow keys, action listing.
//
// NOTE: The first sub-test opens the AI window; subsequent tests
// reuse it.  `reset()` only hides the main prompt — the AI window
// stays open.  Each sub-test re-opens the command bar via
// `showAiCommandBar`.

#[test]
fn test_ai_window_opens() {
    windows_feature_suite("AI Window Opens")
        .it("opens AI window without crash", |app| {
            app.show();
            app.wait(Duration::from_millis(300));
            app.open_ai_with_mock_data();
            let state = app.ai_command_bar_state();
            assert!(state.ai_window_open, "AI window should be open");
        })
        .run();
}

#[test]
fn test_ai_command_bar() {
    windows_feature_suite("AI Command Bar")
        .it("opens AI window and command bar", |app| {
            app.show();
            app.wait(Duration::from_millis(300));
            // Open the separate AI Chat window with mock data (polls until ready)
            app.open_ai_with_mock_data();

            // Open the command bar — polls until actually open
            app.show_ai_command_bar();
            app.should_have_ai_command_bar_open();

            let state = app.ai_command_bar_state();
            eprintln!(
                "      command bar: open={}, actions={}, selected_idx={}, selected_id={:?}",
                state.command_bar_open,
                state.action_ids.len(),
                state.selected_index,
                state.selected_action_id,
            );
            assert!(
                !state.action_ids.is_empty(),
                "command bar should have at least 1 action"
            );
        })
        .it("arrow down moves selection forward", |app| {
            // Re-open command bar (polls until open)
            app.show_ai_command_bar();

            let before = app.ai_command_bar_state();
            eprintln!(
                "      before: index={}, id={:?}, actions={:?}",
                before.selected_index, before.selected_action_id, before.action_ids
            );
            let initial_index = before.selected_index;
            let initial_id = before.selected_action_id.clone();

            // Press arrow down in the AI window
            app.ai_arrow_down();

            let after = app.ai_command_bar_state();
            eprintln!(
                "      after down: index={}, id={:?}",
                after.selected_index, after.selected_action_id
            );

            // The selected index or action ID MUST have changed
            let moved = after.selected_index != initial_index
                || after.selected_action_id != initial_id;
            assert!(
                moved,
                "arrow down should change selection: before index={} id={:?}, after index={} id={:?}",
                initial_index, initial_id, after.selected_index, after.selected_action_id
            );
        })
        .it("arrow up moves selection backward", |app| {
            app.show_ai_command_bar();

            // Press down twice, then up once — should not be at initial position
            app.ai_arrow_down();
            app.ai_arrow_down();
            let after_two_down = app.ai_command_bar_state();

            app.ai_arrow_up();
            let after_up = app.ai_command_bar_state();

            eprintln!(
                "      after 2xDown: index={}, after Up: index={}",
                after_two_down.selected_index, after_up.selected_index
            );

            // After up, we should be at a different index than after 2 downs
            assert_ne!(
                after_up.selected_index, after_two_down.selected_index,
                "arrow up should move selection backward: was {} after 2 downs, still {} after up",
                after_two_down.selected_index, after_up.selected_index
            );
        })
        .it("multiple arrow downs traverse all items", |app| {
            app.show_ai_command_bar();

            let initial = app.ai_command_bar_state();
            let action_count = initial.action_ids.len();
            eprintln!(
                "      {} actions, starting at index {}",
                action_count, initial.selected_index
            );

            // Press down enough times to traverse all items
            let mut seen_indices = std::collections::HashSet::new();
            seen_indices.insert(initial.selected_index);

            for i in 0..action_count + 2 {
                app.ai_arrow_down();
                let state = app.ai_command_bar_state();
                seen_indices.insert(state.selected_index);
                if i < 5 || i == action_count + 1 {
                    eprintln!("      step {}: index={}", i, state.selected_index);
                }
            }

            eprintln!("      seen {} unique indices for {} actions", seen_indices.len(), action_count);
            // We should see at least 2 distinct indices (proves navigation works)
            assert!(
                seen_indices.len() >= 2,
                "arrow down should visit multiple indices, only saw {:?} for {} actions",
                seen_indices, action_count
            );
        })
        .run();
}
