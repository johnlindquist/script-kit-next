use super::*;

impl ScriptListApp {
    pub(crate) fn menu_syntax_main_hint_snapshot(
        &self,
        raw_filter_text: &str,
        advanced_query_results_empty: bool,
    ) -> Option<crate::menu_syntax::MenuSyntaxMainHintSnapshot> {
        let mut snapshot = crate::menu_syntax::build_menu_syntax_main_hint(
            crate::menu_syntax::MenuSyntaxMainHintContext {
                raw_filter_text,
                mode: &self.menu_syntax_mode,
                popup_snapshot: self.menu_syntax_trigger_popup_state.snapshot.as_ref(),
                popup_selected_row_id: self
                    .menu_syntax_trigger_popup_state
                    .selected_row_id
                    .as_deref(),
                scripts: &self.scripts,
                scriptlets: &self.scriptlets,
                advanced_query_results_empty,
                menu_syntax_ai_proposal: self.pending_menu_syntax_ai_proposal.as_ref(),
            },
        )?;

        if matches!(
            snapshot.kind,
            crate::menu_syntax::MenuSyntaxMainHintKind::CaptureComposer
        ) {
            if let Some(target) = self.capture_target_for(raw_filter_text) {
                let store = crate::menu_syntax::history::HistoryStore::from_env();
                if let Ok(pool) = store.try_read_tag_pool(&target) {
                    let recent: Vec<String> = pool
                        .iter()
                        .take(5)
                        .map(|tf| format!("#{}", tf.tag))
                        .collect();
                    if !recent.is_empty() {
                        let value = recent.join(" ");
                        if let Some(existing) = snapshot
                            .rows
                            .iter_mut()
                            .find(|row| row.label == "Tags")
                        {
                            existing.value = value;
                        } else {
                            snapshot
                                .rows
                                .push(crate::menu_syntax::MenuSyntaxMainHintRow {
                                    label: "Recent tags".to_string(),
                                    value,
                                    chips: Vec::new(),
                                });
                        }
                    }
                }

                if let Some(invocation) = self.menu_syntax_mode.capture_for(raw_filter_text) {
                    let mut seen: std::collections::HashSet<&str> =
                        std::collections::HashSet::new();
                    for (key, _) in invocation.kv.iter().take(3) {
                        if !seen.insert(key.as_str()) {
                            continue;
                        }
                        if let Ok(pool) = store.try_read_key_pool(&target, key) {
                            let recent: Vec<String> =
                                pool.iter().take(3).map(|vf| vf.value.clone()).collect();
                            if !recent.is_empty() {
                                snapshot.rows.push(
                                    crate::menu_syntax::MenuSyntaxMainHintRow {
                                        label: format!("Recent {key}"),
                                        value: recent.join(", "),
                                        chips: Vec::new(),
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }

        Some(snapshot)
    }

    fn capture_target_for(&self, raw_filter_text: &str) -> Option<String> {
        if let Some(invocation) = self.menu_syntax_mode.capture_for(raw_filter_text) {
            return Some(invocation.target.clone());
        }
        if let Some(incomplete) = self.menu_syntax_mode.incomplete_for(raw_filter_text) {
            if let crate::menu_syntax::payload::IncompleteKind::MissingCaptureBody(target) =
                &incomplete.kind
            {
                return Some(target.clone());
            }
        }
        None
    }
}
