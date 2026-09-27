#[path = "notes_browse_elements.rs"]
mod notes_browse_elements;

impl ScriptListApp {
    fn append_footer_elements(
        &self,
        outcome: &mut ElementCollectionOutcome,
        limit: usize,
    ) {
        let target = crate::get_main_window_handle().and_then(|handle| {
            crate::windows::list_automation_windows()
                .into_iter()
                .find(|target| {
                    target.generation.is_some_and(|generation| {
                        crate::windows::get_runtime_window_handle_for_generation(
                            &target.id,
                            generation,
                        ) == Some(handle)
                    })
                })
        });
        let Some(target) = target else {
            outcome
                .warnings
                .push("footer_target_unregistered".to_string());
            return;
        };
        let footer = self.active_footer_snapshot(&target);
        let row_kind = match footer.owner.as_str() {
            "native" => Some("nativeFooterRow"),
            "prompt" => Some("promptFooterRow"),
            "content" => Some("contentFooterRow"),
            _ => None,
        };
        let Some(row_kind) = row_kind else {
            return;
        };

        outcome.total_count += 1 + footer.buttons.len();

        if outcome.elements.len() >= limit {
            outcome
                .warnings
                .push("footer_elements_truncated_by_limit".to_string());
            return;
        }

        outcome.elements.push(protocol::ElementInfo {
            semantic_id: format!("footer:{}:row", footer.owner),
            element_type: protocol::ElementType::Panel,
            text: Some(footer.owner.clone()),
            value: footer.expected_surface.clone(),
            content: None,
            selected: None,
            focused: None,
            index: None,
            role: Some("footer".to_string()),
            kind: Some(row_kind.to_string()),
            source: None,
            source_name: None,
            selectable: Some(false),
            status_kind: footer.mismatch.clone(),
            action_disabled: None,
            style: None,
        });

        for (index, button) in footer.buttons.iter().enumerate() {
            if outcome.elements.len() >= limit {
                outcome
                    .warnings
                    .push("footer_elements_truncated_by_limit".to_string());
                break;
            }

            let kind = match footer.owner.as_str() {
                "native" => "nativeFooterButton",
                "prompt" => "promptFooterButton",
                _ => "contentFooterButton",
            };
            let text = if button.shortcut_routable {
                format!("{} {}", button.key, button.label)
            } else {
                button.label.clone()
            };
            outcome.elements.push(protocol::ElementInfo {
                semantic_id: button.id.clone(),
                element_type: protocol::ElementType::Button,
                text: Some(text),
                value: Some(button.action.clone()),
                content: None,
                selected: Some(button.selected),
                focused: None,
                index: Some(index),
                role: Some("footer".to_string()),
                kind: Some(kind.to_string()),
                source: footer.expected_surface.clone(),
                source_name: footer.requested_surface.clone(),
                selectable: Some(button.enabled),
                status_kind: button.action_disabled.clone(),
                action_disabled: button.action_disabled.clone(),
                style: None,
            });
        }
    }

    fn collect_choice_view_elements(
        &self,
        input_name: &str,
        input_value: String,
        choices: &[Choice],
        selected_index: usize,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let filtered = self.get_filtered_arg_choices(choices);
        let total_count = filtered.len() + 2;

        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::input(
                input_name,
                Some(input_value.as_str()),
                self.focused_input != FocusedInput::None,
            ),
        );

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list("choices", filtered.len()),
        );

        for (display_index, choice) in filtered.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            elements.push(Self::keyed_choice_element(
                display_index,
                choice,
                display_index == selected_index,
            ));
        }

        (elements, total_count)
    }

    fn collect_notes_browse_elements(
        &self,
        search: &crate::notes::search_model::NoteSearchHostState,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        notes_browse_elements::notes_browse_elements(
            search,
            self.focused_input != FocusedInput::None,
            limit,
        )
    }

    fn collect_named_rows(
        &self,
        input_name: &str,
        input_value: String,
        list_name: &str,
        rows: &[String],
        selected_index: usize,
        limit: usize,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let total_count = rows.len() + 2;

        let mut elements = Vec::with_capacity(limit.min(total_count));

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::input(
                input_name,
                Some(input_value.as_str()),
                self.focused_input != FocusedInput::None,
            ),
        );

        Self::push_limited_element(
            &mut elements,
            limit,
            protocol::ElementInfo::list(list_name, rows.len()),
        );

        let content_kind = match list_name {
            "clipboard-history" | "agent_chat-history" | "dictation-history" => {
                Some(protocol::ElementContentKind::UserContent)
            }
            "windows" | "browser-tabs" | "browser-history" | "processes" | "kit-results"
            | "installed-kits" => Some(protocol::ElementContentKind::ExternalContent),
            "file-results" | "migrate-v1-results" => Some(protocol::ElementContentKind::FilePath),
            "apps" | "settings" | "menu-commands" | "sdk-functions" | "tips"
            | "script-templates" | "emoji-results" => None,
            _ => Some(protocol::ElementContentKind::ExternalContent),
        };

        for (index, row) in rows.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            let element = if let Some(content_kind) = content_kind {
                protocol::ElementInfo::redacted_choice(
                    index,
                    row,
                    row,
                    index == selected_index,
                    content_kind,
                )
            } else {
                protocol::ElementInfo::product_static_choice(
                    index,
                    row,
                    row,
                    index == selected_index,
                )
            };
            elements.push(element);
        }

        (elements, total_count)
    }
}
