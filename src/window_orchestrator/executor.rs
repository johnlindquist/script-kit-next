//! Maps `WindowCommand` variants to platform calls and GPUI operations.
//!
//! All functions in this module must be called from the GPUI foreground executor
//! (i.e., inside `cx.spawn()` or on the main thread). Commands are executed in
//! order because ordering matters for correctness (e.g., conceal main BEFORE
//! opening the dictation overlay).

use super::{FocusToken, SurfaceId, WindowCommand};

/// Execute a batch of window commands in order.
///
/// Must be called from the main thread (GPUI foreground executor).
pub(crate) fn execute_commands(commands: &[WindowCommand], cx: &mut gpui::App) {
    for command in commands {
        execute_single(command, cx);
    }
}

fn execute_single(command: &WindowCommand, cx: &mut gpui::App) {
    match command {
        // -----------------------------------------------------------------
        // Main window
        // -----------------------------------------------------------------
        WindowCommand::SaveMainBounds => {
            // Save is handled inline by the existing show/hide helpers when
            // they read get_main_window_bounds(). This is a placeholder for
            // future explicit save-before-hide if the helpers are refactored.
            tracing::debug!(
                category = "ORCHESTRATOR",
                "SaveMainBounds (no-op — bounds saved by hide helpers)"
            );
        }

        WindowCommand::ConcealMain => {
            // Synchronous — the executor already runs inside cx.update()
            // on the main thread, so we can call orderOut: directly.
            // Using the deferred variant would delay the hide by one tick,
            // causing the overlay to open while the main window is still
            // visible (and macOS may push the overlay behind other apps).
            crate::platform::conceal_main_window();
        }

        WindowCommand::DismissMain => {
            crate::platform::defer_hide_main_window(cx);
        }

        WindowCommand::RevealMain {
            activate_app,
            make_key,
        } => {
            if *activate_app {
                crate::platform::activate_main_window();
            } else if *make_key {
                crate::platform::show_main_window_without_activation();
            } else {
                crate::platform::show_main_window_background();
            }
        }

        WindowCommand::FocusMain(token) => {
            tracing::debug!(
                category = "ORCHESTRATOR",
                ?token,
                "FocusMain — delegated to caller's entity update"
            );
            // Focus restoration requires access to the ScriptListApp entity
            // and its focus handles, which the executor doesn't own. The
            // dispatch_window_event() method on ScriptListApp handles this
            // after running execute_commands().
        }

        // -----------------------------------------------------------------
        // Actions dialog
        // -----------------------------------------------------------------
        WindowCommand::OpenActions => {
            tracing::debug!(
                category = "ORCHESTRATOR",
                "OpenActions — delegated to caller"
            );
        }

        WindowCommand::CloseActions => {
            tracing::debug!(
                category = "ORCHESTRATOR",
                "CloseActions — delegated to caller"
            );
        }

        // -----------------------------------------------------------------
        // Notes
        // -----------------------------------------------------------------
        WindowCommand::OpenNotesWindow => {
            if let Err(e) = crate::notes::open_notes_window(cx) {
                tracing::warn!(category = "ORCHESTRATOR", error = %e, "Failed to open notes window");
            }
        }

        WindowCommand::CloseNotesWindow => {
            crate::notes::close_notes_window(cx);
        }

        WindowCommand::FocusSurface(surface) => {
            match surface {
                SurfaceId::Notes => {
                    // Notes window manages its own key-window status via AppKit.
                    tracing::debug!(
                        category = "ORCHESTRATOR",
                        "FocusSurface(Notes) — notes window self-manages key status"
                    );
                }
                SurfaceId::DetachedAiChat => {
                    tracing::debug!(
                        category = "ORCHESTRATOR",
                        "FocusSurface(DetachedAiChat) — chat window self-manages key status"
                    );
                }
                SurfaceId::DictationOverlay => {
                    tracing::debug!(
                        category = "ORCHESTRATOR",
                        "FocusSurface(DictationOverlay) — overlay self-manages key status"
                    );
                }
                SurfaceId::Main => {
                    crate::platform::show_main_window_without_activation();
                }
            }
        }

        // -----------------------------------------------------------------
        // Detached AI Chat
        // -----------------------------------------------------------------
        WindowCommand::OpenDetachedAiChatWindow => {
            if let Err(e) = crate::ai::acp::chat_window::open_chat_window(cx) {
                tracing::warn!(category = "ORCHESTRATOR", error = %e, "Failed to open detached AI chat window");
            }
        }

        WindowCommand::CloseDetachedAiChatWindow => {
            crate::ai::acp::chat_window::close_chat_window(cx);
        }

        // -----------------------------------------------------------------
        // Dictation overlay
        // -----------------------------------------------------------------
        WindowCommand::OpenDictationOverlay { phase: _ } => {
            // During migration: the dictation session opens the overlay directly
            // via start_dictation_overlay_session(). This command is a state
            // signal only — the overlay is already open by the time it runs.
            tracing::debug!(
                category = "ORCHESTRATOR",
                "OpenDictationOverlay — overlay managed by dictation session"
            );
        }

        WindowCommand::UpdateDictationOverlay { phase: _ } => {
            // The overlay state update requires the full DictationOverlayState
            // (bars, elapsed, transcript), not just the phase. The dictation
            // session manager pushes state directly via update_dictation_overlay().
            tracing::debug!(
                category = "ORCHESTRATOR",
                "UpdateDictationOverlay — state pushed by dictation session"
            );
        }

        WindowCommand::CloseDictationOverlay => {
            // During migration: the dictation session manages overlay close
            // with its own delay scheduling (schedule_dictation_overlay_close).
            // This command is a state signal only — do not close immediately.
            tracing::debug!(
                category = "ORCHESTRATOR",
                "CloseDictationOverlay — overlay close managed by dictation session"
            );
        }
    }
}

/// Convert a `FocusToken` to the corresponding `FocusTarget` used by `ScriptListApp`.
///
/// Returns `None` for tokens that don't map to a main-window focus target
/// (e.g., `NotesEditor`, `DetachedAiComposer`, `None`).
pub(crate) fn focus_token_to_focus_target(token: &FocusToken) -> Option<&'static str> {
    match token {
        FocusToken::MainFilter => Some("MainFilter"),
        FocusToken::PromptInput => Some("PromptInput"),
        FocusToken::ChatComposer => Some("ChatComposer"),
        FocusToken::TermInput => Some("TermInput"),
        FocusToken::NotesEditor | FocusToken::DetachedAiComposer | FocusToken::None => None,
    }
}

/// Convert from the dictation module's `DictationTarget` to the orchestrator's
/// `DictationTarget`.  The two types are structurally identical but live in
/// different modules to keep the orchestrator free of dictation-module dependencies.
pub(crate) fn to_orchestrator_target(
    target: &crate::dictation::DictationTarget,
) -> super::DictationTarget {
    match target {
        crate::dictation::DictationTarget::MainWindowPrompt => {
            super::DictationTarget::MainWindowPrompt
        }
        crate::dictation::DictationTarget::NotesEditor => super::DictationTarget::NotesEditor,
        crate::dictation::DictationTarget::AiChatComposer => super::DictationTarget::AiChatComposer,
        crate::dictation::DictationTarget::TabAiHarness => super::DictationTarget::TabAiHarness,
        crate::dictation::DictationTarget::ExternalApp => super::DictationTarget::ExternalApp,
    }
}
