use super::*;

impl ScriptListApp {
    /// Dispatch a window event through the orchestrator state machine,
    /// then execute the resulting commands.
    ///
    /// Must be called from an entity update context (i.e., inside
    /// `app_entity.update(cx, |view, cx| { ... })`).
    ///
    /// Platform calls that trigger AppKit delegate callbacks are deferred
    /// via `cx.spawn()` to avoid `RefCell` reentrancy panics.
    pub(crate) fn dispatch_window_event(
        &mut self,
        event: crate::window_orchestrator::WindowEvent,
        cx: &mut Context<Self>,
    ) {
        let commands = self.window_orchestrator.dispatch(event);
        if commands.is_empty() {
            return;
        }

        tracing::debug!(
            category = "ORCHESTRATOR",
            count = commands.len(),
            "Dispatching window commands"
        );

        // Spawn command execution to avoid RefCell conflicts — platform calls
        // like orderOut:/makeKeyWindow trigger synchronous delegate callbacks
        // that re-enter GPUI.
        cx.spawn({
            let commands = commands.clone();
            async move |_this, cx| {
                cx.update(|cx| {
                    crate::window_orchestrator::executor::execute_commands(&commands, cx);
                });
            }
        })
        .detach();
    }
}
