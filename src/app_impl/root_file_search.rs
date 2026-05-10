use super::*;

impl ScriptListApp {
    fn cancel_root_file_search(&mut self) {
        if let Some(cancel) = self.root_file_search_cancel.take() {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub(crate) fn maybe_start_root_file_search(&mut self, query: &str, cx: &mut Context<Self>) {
        let trimmed = query.trim();
        let eligible = matches!(self.current_view, AppView::ScriptList)
            && self
                .menu_syntax_mode
                .advanced_query_for(&self.filter_text)
                .is_none()
            && !self.menu_syntax_trigger_popup_state.owns_main_list()
            && !self
                .menu_syntax_mode
                .capture_composer_owns_input_for(trimmed)
            && !self.menu_syntax_mode.command_owns_input_for(trimmed)
            && crate::file_search::should_search_root_files(trimmed);

        if !eligible {
            self.cancel_root_file_search();
            let had_results = !self.root_file_results.is_empty()
                || !self.root_file_search_query.is_empty()
                || self.root_file_search_loading;
            self.root_file_results.clear();
            self.root_file_search_query.clear();
            self.root_file_search_loading = false;
            if had_results {
                self.root_file_search_generation = self.root_file_search_generation.wrapping_add(1);
                self.invalidate_grouped_cache();
                cx.notify();
            }
            return;
        }

        if self.root_file_search_query == trimmed {
            return;
        }

        self.cancel_root_file_search();
        self.root_file_search_generation = self.root_file_search_generation.wrapping_add(1);
        let generation = self.root_file_search_generation;
        self.root_file_search_query = trimmed.to_string();
        self.root_file_results.clear();
        self.root_file_search_loading = true;
        self.invalidate_grouped_cache();

        let cancel = crate::file_search::new_cancel_token();
        self.root_file_search_cancel = Some(cancel.clone());
        let query_for_task = trimmed.to_string();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(120))
                .await;

            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn({
                let cancel = cancel.clone();
                let query_for_task = query_for_task.clone();
                move || {
                    crate::file_search::search_files_streaming_with_options(
                        &query_for_task,
                        None,
                        crate::file_search::ROOT_FILE_SOURCE_LIMIT,
                        cancel,
                        crate::file_search::SearchFilesStreamingOptions::root_search(),
                        |event| {
                            let _ = tx.send(event);
                        },
                    );
                }
            });

            let mut batch = Vec::new();
            loop {
                if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                match rx.try_recv() {
                    Ok(crate::file_search::SearchEvent::Result(result)) => batch.push(result),
                    Ok(crate::file_search::SearchEvent::Done) => break,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(16))
                            .await;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                }
            }

            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    if app.root_file_search_generation != generation
                        || app.root_file_search_query != query_for_task
                    {
                        return;
                    }

                    app.root_file_results = batch;
                    app.root_file_search_loading = false;
                    app.root_file_search_cancel = None;
                    app.invalidate_grouped_cache();
                    if matches!(app.current_view, AppView::ScriptList) {
                        app.sync_list_state_for_filter_replacement();
                        app.validate_selection_bounds(cx);
                        app.rebuild_main_window_preflight_if_needed();
                    }
                    cx.notify();
                })
            });
        })
        .detach();
    }
}
