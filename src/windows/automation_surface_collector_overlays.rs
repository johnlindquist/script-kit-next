fn collect_hud_snapshot(
    resolved: &AutomationWindowInfo,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    let entity = exact_root::<crate::hud_manager::HudView>(resolved, cx)?;
    let view = entity.read(cx);
    let (text, action_label, error, completed) = view.semantic_state();
    let mut elements = vec![
        element(
            "panel:hud",
            ElementType::Panel,
            Some(text.to_string()),
            None,
            None,
            None,
            None,
        ),
        element(
            "hud:dismiss",
            ElementType::Button,
            Some("Dismiss".into()),
            None,
            None,
            None,
            None,
        ),
    ];
    if let Some(label) = action_label {
        let mut action = element(
            "hud:primary-action",
            ElementType::Button,
            Some(label.into()),
            None,
            None,
            None,
            None,
        );
        action.selectable = Some(true);
        action.status_kind = Some(
            if completed {
                "completed"
            } else if error.is_some() {
                "refused"
            } else {
                "ready"
            }
            .into(),
        );
        elements.push(action);
    }
    if let Some(error) = error {
        let mut status = element(
            "hud:action-error",
            ElementType::Panel,
            Some(error.into()),
            None,
            None,
            None,
            None,
        );
        status.status_kind = Some("refused".into());
        elements.push(status);
    }
    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: None,
        selected_semantic_id: None,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
}

fn collect_snap_snapshot(
    resolved: &AutomationWindowInfo,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    let entity = exact_root::<crate::window_control::SnapOverlayView>(resolved, cx)?;
    let view = entity.read(cx);
    let mut elements = vec![element(
        "panel:snap-overlay",
        ElementType::Panel,
        Some("Snap preview".into()),
        None,
        None,
        None,
        None,
    )];
    let mut selected_semantic_id = None;
    if let Some(model) = view.model() {
        elements[0].status_kind = Some(format!("{:?}", model.mode));
        for (index, target) in model.targets.iter().enumerate() {
            let id = format!("snap:target:{:?}", target.tile);
            let mut target_element = element(
                &id,
                ElementType::Panel,
                Some(format!("{:?}", target.tile)),
                None,
                Some(target.active),
                None,
                Some(index),
            );
            target_element.selectable = Some(false);
            target_element.status_kind =
                Some(if target.active { "active" } else { "inactive" }.into());
            if target.active {
                selected_semantic_id = Some(id);
            }
            elements.push(target_element);
        }
    }
    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: None,
        selected_semantic_id,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
}

fn collect_footer_snapshot(resolved: &AutomationWindowInfo) -> Option<SurfaceElementSnapshot> {
    let state = crate::footer_popup::footer_runtime_state(&resolved.id, resolved.generation?)?;
    let mut elements = vec![element(
        "panel:footer-overlay",
        ElementType::Panel,
        Some(state.config.surface.to_string()),
        None,
        None,
        None,
        None,
    )];
    for (index, descriptor) in state.config.buttons.iter().enumerate() {
        let mut button = element(
            descriptor.id.as_ref(),
            ElementType::Button,
            Some(descriptor.label.to_string()),
            Some(descriptor.key.to_string()),
            Some(descriptor.selected),
            None,
            Some(index),
        );
        button.selectable = Some(descriptor.enabled);
        button.action_disabled = descriptor.disabled_reason.as_ref().map(ToString::to_string);
        elements.push(button);
    }
    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: None,
        selected_semantic_id: None,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
}

// ---------------------------------------------------------------------------
// Prompt popup collector (composer picker, history popup, confirm)
// ---------------------------------------------------------------------------

/// Collect semantic elements only from the exact registered PromptPopup
/// subtype and lifetime. There is deliberately no "whichever popup is open"
/// fallback: a stale or mismatched target must fail closed.
fn collect_exact_prompt_popup_snapshot(
    resolved: &AutomationWindowInfo,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    if let Some(footer) = collect_footer_snapshot(resolved) {
        return Some(footer);
    }
    match resolved.id.as_str() {
        crate::ai::agent_chat::ui::history_popup::AGENT_CHAT_HISTORY_POPUP_AUTOMATION_ID => {
            let generation = resolved.generation?;
            if crate::ai::agent_chat::ui::history_popup::history_popup_generation()
                != Some(generation)
            {
                return None;
            }
            collect_history_popup_snapshot(cx, generation)
        }
        "confirm-popup" => collect_confirm_popup_snapshot(cx, resolved.generation?),
        _ => None,
    }
}

fn collect_history_popup_snapshot(
    cx: &gpui::App,
    generation: u64,
) -> Option<SurfaceElementSnapshot> {
    let snap = crate::ai::agent_chat::ui::history_popup::get_history_popup_snapshot_for_generation(
        generation, cx,
    )?;

    let mut elements = vec![element(
        "panel:history-popup",
        ElementType::Panel,
        Some(snap.title.to_string()),
        Some(snap.query.to_string()),
        None,
        None,
        None,
    )];

    let entry_count = snap.entries.len();
    elements.push(element(
        "list:history-entries",
        ElementType::List,
        Some(format!("{entry_count} sessions")),
        None,
        None,
        None,
        None,
    ));

    let mut selected_semantic_id = None;
    for (idx, entry) in snap.entries.iter().enumerate() {
        let is_selected = idx == snap.selected_index;
        let semantic_id = format!("choice:{}:{}", idx, entry.hit.entry.session_id);

        if is_selected {
            selected_semantic_id = Some(semantic_id.clone());
        }

        elements.push(element(
            &semantic_id,
            ElementType::Choice,
            Some(entry.title.to_string()),
            Some(entry.hit.entry.session_id.clone()),
            Some(is_selected),
            None,
            Some(idx),
        ));
    }

    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: selected_semantic_id.clone(),
        selected_semantic_id,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
}

fn collect_confirm_popup_snapshot(
    cx: &gpui::App,
    generation: u64,
) -> Option<SurfaceElementSnapshot> {
    let snap = crate::confirm::get_confirm_popup_snapshot(cx, generation, None)?;

    let confirm_focused = snap.focused_button == "confirm";
    let secondary_focused = snap.focused_button == "secondary";
    let cancel_focused = snap.focused_button == "cancel";
    let has_secondary = snap.secondary_text.is_some();

    let mut panel = element(
        "panel:confirm-dialog",
        ElementType::Panel,
        Some(snap.title),
        Some(snap.body),
        None,
        None,
        None,
    );
    panel.kind = Some(
        match snap.risk {
            crate::confirm::ConfirmRisk::Safe => "safe",
            crate::confirm::ConfirmRisk::Destructive => "destructive",
            crate::confirm::ConfirmRisk::Irreversible => "irreversible",
        }
        .to_string(),
    );
    let waiting = crate::confirm::queued_modal_count();
    if waiting > 0 {
        panel.status_kind = Some(format!("{waiting} waiting"));
    }
    let danger = snap.risk != crate::confirm::ConfirmRisk::Safe;
    let mut elements = vec![panel];
    if let Some(text) = snap.input_text {
        // The field owns typing for as long as the modal is open.
        let mut field = element(
            "input:confirm-field",
            ElementType::Input,
            None,
            Some(text),
            None,
            Some(true),
            None,
        );
        field.status_kind = snap.input_problem.as_ref().map(|_| "error".to_string());
        field.action_disabled = snap.input_problem;
        elements.push(field);
    }
    let mut confirm = element(
        "button:0:confirm",
        ElementType::Button,
        Some(snap.confirm_text),
        Some("confirm".to_string()),
        None,
        Some(confirm_focused),
        Some(0),
    );
    if danger {
        confirm.status_kind = Some("danger".to_string());
    }
    if !snap.primary_enabled {
        confirm.action_disabled = Some("input_invalid".to_string());
        confirm.selectable = Some(false);
    }
    elements.push(confirm);
    if let Some(secondary_text) = snap.secondary_text {
        elements.push(element(
            "button:1:secondary",
            ElementType::Button,
            Some(secondary_text),
            Some("secondary".to_string()),
            None,
            Some(secondary_focused),
            Some(1),
        ));
    }
    let cancel_index = if has_secondary { 2 } else { 1 };
    let cancel_semantic_id = if has_secondary {
        "button:2:cancel"
    } else {
        "button:1:cancel"
    };
    elements.push(element(
        cancel_semantic_id,
        ElementType::Button,
        Some(snap.cancel_text),
        Some("cancel".to_string()),
        None,
        Some(cancel_focused),
        Some(cancel_index),
    ));

    let focused_semantic_id = if confirm_focused {
        "button:0:confirm"
    } else if secondary_focused {
        "button:1:secondary"
    } else {
        cancel_semantic_id
    };

    Some(SurfaceElementSnapshot {
        total_count: elements.len(),
        elements,
        focused_semantic_id: Some(focused_semantic_id.to_string()),
        selected_semantic_id: None,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
}
