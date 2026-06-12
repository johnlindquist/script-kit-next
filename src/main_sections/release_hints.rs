// One-shot release hints surfaced from the main window show path.

impl ScriptListApp {
    /// Surface the retired tap-to-dismiss habit change once per install.
    pub(crate) fn maybe_show_tap_dismiss_retired_hint(&mut self, cx: &mut Context<Self>) {
        if script_kit_gpui::nux::tap_dismiss::already_shown() {
            return;
        }
        script_kit_gpui::nux::tap_dismiss::mark_shown();
        tracing::info!(event = "tap_dismiss_retired_hint_shown");
        self.toast_manager.push(
            Toast::info(
                "Tap toggles Day Page — press Esc to dismiss the launcher.",
                &self.theme,
            )
            .duration_ms(Some(TOAST_INFO_MS)),
        );
        cx.notify();
    }
}
