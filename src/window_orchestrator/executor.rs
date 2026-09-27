//! Maps `WindowCommand` variants to platform calls and GPUI operations.
//!
//! All functions in this module must be called from the GPUI foreground executor
//! (i.e., inside `cx.spawn()` or on the main thread). Commands are executed in
//! order because ordering matters for correctness (e.g., save bounds BEFORE
//! concealing main).

use super::{SurfaceId, WindowCommand};

/// Execute a batch of window commands in order.
///
/// Must be called from the main thread (GPUI foreground executor).
#[allow(
    dead_code,
    reason = "the separately compiled application binary owns the orchestrator bridge"
)]
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
            crate::set_main_window_visible(false);
            crate::platform::conceal_main_window();
        }

        WindowCommand::DismissMain => {
            crate::set_main_window_visible(false);
            crate::platform::defer_hide_main_window(cx);
        }

        WindowCommand::RevealMain {
            activate_app,
            make_key,
        } => {
            crate::set_main_window_visible(true);
            if *activate_app {
                crate::platform::activate_main_window();
            } else if *make_key {
                crate::platform::ensure_main_panel_configured(
                    "window_orchestrator::executor::RevealMain/make_key",
                );
                crate::platform::show_main_window_without_activation();
            } else {
                crate::platform::ensure_main_panel_configured(
                    "window_orchestrator::executor::RevealMain/background",
                );
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

        WindowCommand::FocusSurface(SurfaceId::Main) => {
            crate::set_main_window_visible(true);
            crate::platform::ensure_main_panel_configured(
                "window_orchestrator::executor::FocusSurface/Main",
            );
            crate::platform::show_main_window_without_activation();
        }
    }
}
