// Shared production dispatch for exact automation lifetimes. Included at crate root.

pub(crate) use crate::windows::automation_transaction_provider::apply_registered_root_layer_selection;

#[expect(
    clippy::too_many_arguments,
    reason = "Keep exact window, revision, and dispatch authority explicit across the async batch boundary"
)]
fn select_main_window_semantic_id_for_batch(
    this: &gpui::WeakEntity<ScriptListApp>,
    main_window_handle: Option<gpui::AnyWindowHandle>,
    target: Option<&protocol::AutomationWindowInfo>,
    expected: Option<&protocol::AutomationTargetIdentitySnapshot>,
    guard: &crate::platform::gpui_event_simulator::DispatchTarget,
    semantic_id: &str,
    submit: bool,
    cx: &mut gpui::AsyncApp,
) -> anyhow::Result<String> {
    if semantic_id.starts_with("root-dialog:") || semantic_id.starts_with("root-notification:") {
        let fallback;
        let target = if let Some(target) = target {
            target
        } else {
            fallback = crate::windows::resolve_automation_window(Some(
                &protocol::AutomationWindowTarget::Main,
            ))?;
            &fallback
        };
        let exact_handle = target.generation.and_then(|generation| {
            crate::windows::get_runtime_window_handle_for_generation(&target.id, generation)
        });
        anyhow::ensure!(
            exact_handle.is_some()
                && exact_handle == main_window_handle.or_else(crate::get_main_window_handle),
            "main_root_layer_target_mismatch"
        );
        return cx
            .update(|cx| {
                validate_batch_app_effect(expected, guard, this, cx)?;
                apply_registered_root_layer_selection(target, semantic_id, submit, cx)
            })?
            .ok_or_else(|| protocol::TransactionError::element_not_found(semantic_id).into());
    }
    let semantic_id = semantic_id.to_string();
    if let Some(handle) = main_window_handle.or_else(crate::get_main_window_handle) {
        return handle.update(cx, |_root, window, cx| {
            validate_batch_window_effect(expected, guard, this, window, cx)?;
            this.update(cx, |app, cx| {
                app.select_choice_by_semantic_id_in_window(&semantic_id, submit, window, cx)
            })
        })??;
    }

    this.update(cx, |app, cx| {
        validate_batch_main_effect(app, expected, guard, cx)?;
        app.select_choice_by_semantic_id(&semantic_id, submit, cx)
    })?
}

fn registered_surface_target(
    resolved: &protocol::AutomationWindowInfo,
) -> anyhow::Result<protocol::AutomationWindowInfo> {
    let current = crate::windows::automation_surface_collector::current_surface_metadata(resolved)
        .ok_or_else(|| anyhow::anyhow!("registered_surface_target_stale"))?;
    anyhow::ensure!(
        current.kind != protocol::AutomationWindowKind::Main,
        "registered_surface_requires_secondary_owner"
    );
    let generation = current
        .generation
        .ok_or_else(|| anyhow::anyhow!("registered_surface_generation_missing"))?;
    // Registration verifies the actual native host. Its policy cannot change
    // during this exact lifetime, so reads need no mutable Window borrow.
    let policy = crate::windows::runtime_window_host_policy(&current.id, generation)?;
    policy.validate()?;
    anyhow::ensure!(
        !policy.is_hidden() || (!current.visible && !current.focused),
        "registered_surface_hidden_metadata_mismatch"
    );
    Ok(current)
}

pub(crate) fn registered_surface_ui_snapshot(
    resolved: &protocol::AutomationWindowInfo,
    cx: &gpui::App,
) -> anyhow::Result<protocol::UiStateSnapshot> {
    use crate::windows::automation_surface_collector as collector;
    let target = registered_surface_target(resolved)?;
    target
        .generation
        .ok_or_else(|| anyhow::anyhow!("registered_surface_generation_missing"))?;
    let surface = collector::collect_surface_snapshot(&target, usize::MAX, cx)
        .ok_or_else(|| anyhow::anyhow!("registered_surface_state_unavailable:{}", target.id))?;
    let (elements, focused_semantic_id, selected_semantic_id) = (
        surface.elements,
        surface.focused_semantic_id,
        surface.selected_semantic_id,
    );
    let input_value = elements
        .iter()
        .find(|element| element.element_type == protocol::ElementType::Input)
        .and_then(|element| element.value.clone());
    let selected_value = selected_semantic_id.as_ref().and_then(|id| {
        elements
            .iter()
            .find(|element| &element.semantic_id == id)
            .and_then(|element| element.value.clone().or_else(|| element.text.clone()))
    });
    Ok(protocol::UiStateSnapshot {
        window_visible: target.visible,
        window_focused: target.focused,
        prompt_type: Some(target.kind.as_camel_case().to_string()),
        input_value,
        selected_value,
        choice_count: elements
            .iter()
            .filter(|element| element.element_type == protocol::ElementType::Choice)
            .count(),
        visible_semantic_ids: elements
            .into_iter()
            .map(|element| element.semantic_id)
            .collect(),
        focused_semantic_id,
        ..Default::default()
    })
}

pub(crate) fn registered_surface_wait_satisfied(
    resolved: &protocol::AutomationWindowInfo,
    condition: &protocol::WaitCondition,
    cx: &gpui::App,
) -> anyhow::Result<bool> {
    let target = registered_surface_target(resolved)?;
    let snapshot = registered_surface_ui_snapshot(&target, cx)?;
    Ok(
        protocol::transaction_executor::matches_ui_wait_condition(&snapshot, condition)
            .unwrap_or(false),
    )
}

fn registered_surface_transaction_error(error: anyhow::Error) -> protocol::TransactionError {
    match error.downcast::<protocol::TransactionError>() {
        Ok(error) => error,
        Err(error) => protocol::TransactionError::action_failed(error.to_string()),
    }
}

pub(crate) fn apply_registered_surface_command(
    resolved: &protocol::AutomationWindowInfo,
    command: &protocol::BatchCommand,
    cx: &mut gpui::App,
) -> anyhow::Result<Option<String>> {
    let target = registered_surface_target(resolved)?;
    if let protocol::BatchCommand::SelectBySemanticId {
        semantic_id,
        submit,
    } = command
    {
        if let Some(selected) =
            apply_registered_root_layer_selection(&target, semantic_id, *submit, cx)?
        {
            return Ok(Some(selected));
        }
    }
    target
        .generation
        .ok_or_else(|| anyhow::anyhow!("registered_surface_generation_missing"))?;
    let kind = registered_batch_target_kind(&target);
    Err(unsupported_batch_command_error(kind, command).into())
}

pub(crate) fn registered_surface_state_result(
    request_id: &str,
    resolved: &protocol::AutomationWindowInfo,
    cx: &gpui::App,
) -> anyhow::Result<protocol::Message> {
    let target = registered_surface_target(resolved)?;
    let snapshot = registered_surface_ui_snapshot(&target, cx)?;
    Ok(protocol::Message::StateResult {
        request_id: request_id.to_string(),
        prompt_type: snapshot
            .prompt_type
            .unwrap_or_else(|| target.kind.as_camel_case().to_string()),
        prompt_id: Some(format!("target:{:?}:{}", target.kind, target.id)),
        input_value: snapshot.input_value.unwrap_or_default(),
        choice_count: snapshot.choice_count,
        visible_choice_count: snapshot.choice_count,
        selected_index: -1,
        selected_value: snapshot.selected_value,
        is_focused: snapshot.window_focused,
        window_visible: snapshot.window_visible,
        notes_state: None,
        actions_dialog: None,
        dictation_state: None,
        surface_contract: None,
        active_popup_contract: None,
        active_footer: None,
        submit_diagnostics: None,
        placeholder: None,
        mini_ai: None,
        focused_text_agent_chat: None,
        filter_input_decorations: None,
        filter_input_diagnostics: None,
        menu_syntax_main_hint: None,
        capture_history_picker: None,
        main_window_preflight: None,
        root_file_search: None,
        main_list_scroll: None,
        active_list_scroll: None,
        screenshot_identity: None,
        drop_state: None,
        path_state: None,
        day_page_state: None,
        ghost_prediction: None,
        flow_ux: None,
        backgrounded_sessions: None,
        main_window_animation: None,
        one_window: None,
        permissions: None,
    })
}

impl ScriptListApp {
    fn select_choice_by_semantic_id_in_window(
        &mut self,
        semantic_id: &str,
        submit: bool,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<String> {
        if semantic_id.starts_with("personal-item:") {
            self.pick_personal_view_item(semantic_id, submit, cx)?;
            return Ok(semantic_id.to_string());
        }
        if semantic_id.starts_with("form-field:") {
            anyhow::ensure!(!submit, "select a field, then use the form’s submit action");
            anyhow::ensure!(self.one_window_pick_form_field(semantic_id, cx), "form_field_not_found");
            self.focus_main_filter(window, cx);
            return Ok(semantic_id.to_string());
        }
        if let AppView::CreateAiPresetView { entity } = &self.current_view {
            let entity = entity.clone();
            let selected = entity.update(cx, |form, cx| {
                let selected = form.focus_field_by_semantic_id(semantic_id, cx)?;
                form.focus_field_at(form.focused_index, window, cx);
                Some(selected)
            }).ok_or_else(|| anyhow::anyhow!("preset_field_not_found"))?;
            if submit { self.handle_create_ai_preset_key("enter",window,cx); }
            return Ok(selected);
        }
        if matches!(self.current_view, AppView::ScriptList)
            && self.spine_projection_owns_main_list()
        {
            let selected = self.select_main_menu_choice_by_semantic_id(semantic_id, false, cx)?;
            if submit {
                anyhow::ensure!(
                    self.accept_spine_projection_row(window, cx),
                    "spine_selection_not_accepted"
                );
            }
            return Ok(selected);
        }

        if let AppView::DayPage { entity } = &self.current_view {
            let entity = entity.clone();
            if semantic_id == "day-page-clipboard-shelf-toggle"
                || semantic_id.starts_with("day-page-clipboard-shelf-item-")
            {
                return entity.update(cx, |view, cx| {
                    anyhow::ensure!(
                        !view.clipboard_shelf.is_empty() && view.kit_resource_preview.is_none(),
                        "clipboard_shelf_not_visible"
                    );
                    if semantic_id == "day-page-clipboard-shelf-toggle" {
                        if submit {
                            view.toggle_clipboard_shelf(cx);
                        }
                    } else {
                        let id = semantic_id
                            .strip_prefix("day-page-clipboard-shelf-item-")
                            .ok_or_else(|| {
                                anyhow::anyhow!("day_page_clipboard_shelf_item_id_invalid")
                            })?;
                        anyhow::ensure!(
                            view.clipboard_shelf_expanded
                                && view
                                    .clipboard_shelf
                                    .iter()
                                    .any(|entry| entry.item.entry_id == id),
                            "clipboard_shelf_entry_stale"
                        );
                        if submit {
                            anyhow::ensure!(
                                view.open_clipboard_shelf_entry(id, window, cx),
                                "clipboard_shelf_open_rejected"
                            );
                        }
                    }
                    Ok(semantic_id.to_string())
                });
            }
            if semantic_id == script_kit_gpui::day_page::FRAGMENT_BACK_ID {
                return entity.update(cx, |view, cx| {
                    if !view.session.is_viewing_fragment() {
                        anyhow::bail!("Day Page fragment back is not visible");
                    }
                    if submit {
                        view.return_to_day_page(window, cx);
                    }
                    Ok(semantic_id.to_string())
                });
            }
        }

        self.select_choice_by_semantic_id(semantic_id, submit, cx)
    }
}
