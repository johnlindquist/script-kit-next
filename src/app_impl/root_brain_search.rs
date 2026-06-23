//! Async semantic pass for the root launcher "From Your Brain" section.
//!
//! The sync lexical pass (`crate::brain::search_root_brain_direct`, invoked
//! per keystroke from `filtering_cache.rs`) is the instant first paint. This
//! module mirrors `root_file_search.rs`: a debounced background task embeds
//! the query on the warm indexer thread (bounded ~200ms budget) and runs the
//! hybrid FTS+cosine search; results are applied by generation and preferred
//! over lexical hits while their stored query matches the live query.
//!
//! Staleness contract: applying (or clearing) semantic results bumps
//! `root_brain_semantic_epoch`, which is part of `RootPassiveFrameKey`, and
//! invalidates the passive frame + grouped cache — a cached frame holding
//! lexical-only brain hits can never be served after semantic results land.

use super::*;

const ROOT_BRAIN_SEMANTIC_DEBOUNCE_MS: u64 = 60;

impl ScriptListApp {
    /// Brain section options for `query`, mirroring the sync passive pass in
    /// `filtering_cache.rs` (explicit `@brain` source filter force-enables the
    /// section and widens its caps).
    fn root_brain_semantic_options_for_query(
        &self,
        source_filters: &crate::menu_syntax::RootUnifiedSourceFilterSet,
    ) -> crate::brain::RootBrainSectionOptions {
        let unified_search = self.config.get_unified_search();
        let mut brain_options = unified_search.brain_section_options();
        if source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Brain) {
            brain_options.enabled = true;
            brain_options.min_query_chars = 0;
            brain_options.max_results = brain_options
                .max_results
                .max(unified_search.passive_result_limits().max_total_results);
        }
        brain_options
    }

    /// Drop in-flight + stored semantic state. Invalidates caches only when
    /// stored results actually existed.
    fn clear_root_brain_semantic_state(&mut self, cx: &mut Context<Self>) {
        // Orphan any in-flight batch.
        self.root_brain_search_generation = self.root_brain_search_generation.wrapping_add(1);
        self.root_brain_search_request = None;
        if self.root_brain_semantic_results.is_some() {
            self.root_brain_semantic_results = None;
            self.root_brain_semantic_epoch = self.root_brain_semantic_epoch.wrapping_add(1);
            self.invalidate_root_passive_and_grouped_cache();
            cx.notify();
        }
    }

    /// Kick the debounced async semantic brain search for the current filter
    /// text. Hooked at the same call sites as `maybe_start_root_file_search`
    /// so it runs exactly when the filter text changes. Never blocks: all
    /// embedding/search work happens on a dedicated background thread.
    pub(crate) fn maybe_start_root_brain_semantic_search(
        &mut self,
        query: &str,
        cx: &mut Context<Self>,
    ) {
        let search_text =
            crate::menu_syntax::free_text_for_search(&self.menu_syntax_mode, query).to_string();
        let trimmed = search_text.trim();
        let advanced_query_owned = self.menu_syntax_mode.advanced_query_for(query).cloned();
        let source_filters = advanced_query_owned
            .as_ref()
            .map(|advanced_query| advanced_query.source_filters.clone())
            .unwrap_or_default();
        let advanced_predicate_active = advanced_query_owned
            .as_ref()
            .is_some_and(|advanced_query| advanced_query.has_predicates());
        let brain_options = self.root_brain_semantic_options_for_query(&source_filters);

        // Eligibility gates identical to the sync lexical pass, plus the
        // main-list ownership checks the root file search applies.
        let explicit_brain =
            source_filters.includes(crate::menu_syntax::RootUnifiedSourceFilter::Brain);
        let effective_brain_query = if explicit_brain {
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        } else {
            crate::brain::root_brain_passive_search_text(trimmed, brain_options)
        };
        let can_collect = matches!(self.current_view, AppView::ScriptList)
            && !self.menu_syntax_object_selector_state.owns_main_list()
            && !self.menu_syntax_trigger_picker_state.owns_main_list()
            && !self
                .menu_syntax_mode
                .capture_composer_owns_input_for(trimmed)
            && !self.menu_syntax_mode.command_owns_input_for(trimmed);
        let eligible = can_collect
            && !advanced_predicate_active
            && source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Brain)
            && brain_options.max_results > 0
            && effective_brain_query.is_some();

        if !eligible {
            tracing::debug!(
                target: "script_kit::brain",
                query = %trimmed,
                can_collect,
                advanced_predicate_active,
                allows = source_filters.allows(crate::menu_syntax::RootUnifiedSourceFilter::Brain),
                max_results = brain_options.max_results,
                query_eligible =
                    crate::brain::root_brain_query_is_eligible(trimmed, brain_options),
                effective_query = effective_brain_query.as_deref().unwrap_or(""),
                "brain semantic pass ineligible"
            );
            self.clear_root_brain_semantic_state(cx);
            return;
        }
        let query_owned = effective_brain_query.expect("eligible brain query");
        tracing::debug!(
            target: "script_kit::brain",
            query = %query_owned,
            raw_query = %trimmed,
            "brain semantic pass starting"
        );

        // Already have (or are fetching) this exact request — nothing to do.
        if self
            .root_brain_search_request
            .as_ref()
            .is_some_and(|(query, options)| query == &query_owned && *options == brain_options)
        {
            return;
        }

        self.root_brain_search_generation = self.root_brain_search_generation.wrapping_add(1);
        let generation = self.root_brain_search_generation;
        self.root_brain_search_request = Some((query_owned.clone(), brain_options));

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(
                    ROOT_BRAIN_SEMANTIC_DEBOUNCE_MS,
                ))
                .await;

            // Debounce: bail before embedding when a newer request started.
            let still_current = cx
                .update(|cx| {
                    this.update(cx, |app, _| app.root_brain_search_generation == generation)
                })
                .unwrap_or(false);
            if !still_current {
                return;
            }

            // Embed + hybrid search on a dedicated thread: the embed call
            // blocks up to its 200ms budget and the search hits sqlite.
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn({
                let query_owned = query_owned.clone();
                move || {
                    let _ = tx.send(crate::brain::search_root_brain_semantic(
                        &query_owned,
                        &brain_options,
                    ));
                }
            });

            let outcome = loop {
                match rx.try_recv() {
                    Ok(outcome) => break outcome,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        cx.background_executor()
                            .timer(std::time::Duration::from_millis(16))
                            .await;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => break None,
                }
            };

            // None => no warm embedding model (or worker died): lexical stays.
            let Some(hits) = outcome else {
                tracing::debug!(
                    target: "script_kit::brain",
                    query = %query_owned,
                    "brain semantic pass skipped (no warm embed model)"
                );
                return;
            };
            tracing::debug!(
                target: "script_kit::brain",
                query = %query_owned,
                hits = hits.len(),
                "brain semantic results ready"
            );

            let _ = cx.update(|cx| {
                this.update(cx, |app, cx| {
                    app.apply_root_brain_semantic_results_for_generation(
                        generation,
                        query_owned,
                        hits,
                        cx,
                    );
                })
            });
        })
        .detach();
    }

    /// Publish a semantic batch if it's still the newest request. Bumps the
    /// semantic epoch (part of `RootPassiveFrameKey`) and invalidates the
    /// passive frame + grouped cache so the next paint re-merges brain hits.
    fn apply_root_brain_semantic_results_for_generation(
        &mut self,
        generation: u64,
        query: String,
        hits: Vec<crate::brain::RootBrainSearchHit>,
        cx: &mut Context<Self>,
    ) {
        if self.root_brain_search_generation != generation {
            return;
        }
        self.root_brain_semantic_results = Some((query, hits));
        self.root_brain_semantic_epoch = self.root_brain_semantic_epoch.wrapping_add(1);
        self.invalidate_root_passive_and_grouped_cache();
        if matches!(self.current_view, AppView::ScriptList) {
            self.sync_list_state_for_filter_replacement();
            self.validate_selection_bounds(cx);
            self.rebuild_main_window_preflight_if_needed();
        }
        cx.notify();
    }
}
