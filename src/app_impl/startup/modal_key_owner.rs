impl ScriptListApp {
    /// While a modal is open it owns every keystroke its window receives.
    ///
    /// Registered before every other keystroke interceptor. GPUI runs
    /// interceptors ahead of key bindings and element listeners, so this is
    /// the one place a modal claims a key before a launcher shortcut, a
    /// focused text input, or a window's Escape ladder can act on it.
    ///
    /// Every modal is the shared confirm popup (confirmations, sign-in, the
    /// alias editor, the Tab AI save offer). It never becomes the key window,
    /// so the keys meant for it arrive at its parent — main, Notes,
    /// or detached Agent Chat — and are routed to it here.
    ///
    /// With no modal open, ⌘Z takes a pending Undo offer (see
    /// `crate::undo_offer`) before it can reach a text field's own undo.
    ///
    /// While the main window records a shortcut, every key it gets is the
    /// shortcut, so no launcher binding acts on it first.
    fn install_modal_key_owner_interceptor(&mut self, cx: &mut Context<Self>) {
        let app = cx.entity().downgrade();
        let interceptor = cx.intercept_keystrokes(move |event, window, cx| {
            let recorded = app
                .upgrade()
                .is_some_and(|app| {
                    app.update(cx, |this, cx| {
                        this.one_window_shortcut_keystroke(&event.keystroke, cx)
                    })
                });
            if recorded
                || crate::confirm::route_window_key_to_confirm_popup(window, &event.keystroke, cx)
                || crate::undo_offer::route_window_key(window, &event.keystroke, cx)
            {
                cx.stop_propagation();
            }
        });
        self.gpui_input_subscriptions.push(interceptor);
    }
}
