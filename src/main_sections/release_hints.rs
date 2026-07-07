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

    /// Disclose the silent clipboard auto-keep once it has actually fired.
    /// Shown on window-show (never at copy time) so the sediment no-popup
    /// contract stays intact while the behavior stops being invisible.
    pub(crate) fn maybe_show_sediment_disclosure_hint(&mut self, cx: &mut Context<Self>) {
        if script_kit_gpui::nux::sediment_disclosure::already_shown() {
            return;
        }
        if !script_kit_gpui::nux::sediment_disclosure::activity_recorded() {
            return;
        }
        script_kit_gpui::nux::sediment_disclosure::mark_shown();
        tracing::info!(event = "sediment_disclosure_hint_shown");
        self.toast_manager.push(
            Toast::info(
                "Script Kit keeps copied links and repeat copies on Today — press ⌘K on a Clipboard History entry to manage.",
                &self.theme,
            )
            .duration_ms(Some(TOAST_INFO_MS)),
        );
        cx.notify();
    }
}
