use super::*;

impl ScriptListApp {
    pub(crate) fn queue_filter_compute(&mut self, value: String, cx: &mut Context<Self>) {
        // P3: Debounce expensive search/window resize work.
        // Use 8ms debounce (half a frame) to batch rapid keystrokes.
        logging::log(
            "FILTER_PERF",
            &format!("[2/5] QUEUE_FILTER value='{}' len={}", value, value.len()),
        );
        if self.filter_coalescer.queue(value) {
            cx.spawn(async move |this, cx| {
                // Wait 8ms for coalescing window (half frame at 60fps)
                Timer::after(std::time::Duration::from_millis(8)).await;

                let _ = cx.update(|cx| {
                    this.update(cx, |app, cx| {
                        if let Some(latest) = app.filter_coalescer.take_latest() {
                            if app.computed_filter_text != latest {
                                let coalesce_start = std::time::Instant::now();
                                logging::log(
                                    "FILTER_PERF",
                                    &format!(
                                        "[3/5] COALESCE_PROCESS value='{}' (after 8ms debounce)",
                                        latest
                                    ),
                                );
                                app.computed_filter_text = latest.clone();
                                // Sync list component state and validate selection
                                // This moves state mutation OUT of render() (anti-pattern fix)
                                app.sync_list_state();
                                app.selected_index = 0;
                                app.validate_selection_bounds(cx);
                                app.main_list_state
                                    .scroll_to_reveal_item(app.selected_index);
                                app.last_scrolled_index = Some(app.selected_index);
                                // This will trigger window resize
                                app.update_window_size();
                                let coalesce_elapsed = coalesce_start.elapsed();
                                logging::log(
                                    "FILTER_PERF",
                                    &format!(
                                        "[3/5] COALESCE_DONE in {:.2}ms for '{}'",
                                        coalesce_elapsed.as_secs_f64() * 1000.0,
                                        latest
                                    ),
                                );
                                cx.notify();
                            }
                        }
                    })
                });
            })
            .detach();
        }
    }

    pub(crate) fn set_filter_text_immediate(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.suppress_filter_events = true;
        self.filter_text = text.clone();
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(text.clone(), window, cx);
            // Ensure cursor is at end with no selection after programmatic set_value
            let len = text.len();
            state.set_selection(len, len, window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;

        // Menu bar items are now pre-fetched by frontmost_app_tracker
        // No lazy loading needed - items are already in cache when we open

        self.computed_filter_text = text.clone();
        self.filter_coalescer.reset();

        // Sync list component state and validate selection
        // This moves state mutation OUT of render() (anti-pattern fix)
        self.sync_list_state();
        self.selected_index = 0;
        self.validate_selection_bounds(cx);
        self.main_list_state
            .scroll_to_reveal_item(self.selected_index);
        self.last_scrolled_index = Some(self.selected_index);

        // Update fallback mode immediately based on filter results
        // This ensures SimulateKey commands can check fallback_mode correctly
        // NOTE: validate_selection_bounds already clears fallback_mode and cached_fallbacks,
        // but we need special handling for legacy SimulateKey compatibility
        if !text.is_empty() {
            let results = self.get_filtered_results_cached();
            if results.is_empty() {
                // No matches - check if we should enter fallback mode
                use crate::fallbacks::collect_fallbacks;
                let fallbacks = collect_fallbacks(&text, self.scripts.as_slice());
                if !fallbacks.is_empty() {
                    self.fallback_mode = true;
                    self.cached_fallbacks = fallbacks;
                    self.fallback_selected_index = 0;
                }
            }
        }

        self.update_window_size_deferred(window, cx);
        cx.notify();
    }

    pub(crate) fn clear_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_filter_text_immediate(String::new(), window, cx);
    }

    pub(crate) fn sync_filter_input_if_needed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Sync placeholder if pending
        if let Some(placeholder) = self.pending_placeholder.take() {
            self.gpui_input_state.update(cx, |state, cx| {
                state.set_placeholder(placeholder, window, cx);
            });
        }

        if !self.pending_filter_sync {
            return;
        }

        let desired = self.filter_text.clone();
        let current = self.gpui_input_state.read(cx).value().to_string();
        if current == desired {
            self.pending_filter_sync = false;
            return;
        }

        self.suppress_filter_events = true;
        self.gpui_input_state.update(cx, |state, cx| {
            state.set_value(desired.clone(), window, cx);
            // Ensure cursor is at end with no selection after programmatic set_value
            let len = desired.len();
            state.set_selection(len, len, window, cx);
        });
        self.suppress_filter_events = false;
        self.pending_filter_sync = false;
    }

}
