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

fn collect_dictation_snapshot(
    resolved: &AutomationWindowInfo,
    cx: &gpui::App,
) -> Option<SurfaceElementSnapshot> {
    let state =
        crate::dictation::get_dictation_overlay_state_for_instance(resolved.generation?, cx)?;
    let phase = format!("{:?}", state.phase);
    let destination = crate::dictation::destination_selector_spec(state.target);

    let mut panel = element(
        "panel:dictation-overlay",
        ElementType::Panel,
        resolved
            .title
            .clone()
            .or_else(|| Some("Dictation".to_string())),
        None,
        None,
        Some(resolved.focused),
        None,
    );
    panel.kind = Some("overlay".to_string());
    panel.status_kind = Some(phase.clone());

    let mut signal = element(
        "panel:dictation-signal-band",
        ElementType::Panel,
        Some(phase),
        None,
        None,
        None,
        None,
    );
    signal.kind = Some("signal".to_string());

    let mut target_badge = collect_semantic_chip_element(&destination);
    target_badge.selectable = Some(false);
    target_badge.selected = Some(true);
    target_badge.kind = Some("destination-indicator".to_string());

    let interactive = matches!(
        state.phase,
        crate::dictation::DictationSessionPhase::Recording
            | crate::dictation::DictationSessionPhase::Confirming
    );
    let mut elements = vec![panel, signal, target_badge];
    for descriptor in crate::dictation::DictationTarget::quick_chip_descriptors() {
        let Some(label) = descriptor.quick_chip_label else {
            tracing::warn!(
                target: "script_kit::automation",
                destination = descriptor.stable_id,
                "skipping_dictation_destination_without_quick_chip_label"
            );
            continue;
        };
        let mut spec = crate::components::main_view_chrome::SemanticChipSpec::destination_selector(
            format!("dictation-destination:{}", descriptor.stable_id),
            label,
        );
        if !interactive {
            spec.enabled = false;
            spec.disabled_reason = Some("Destination is locked while Dictation processes".into());
        }
        let mut chip = collect_semantic_chip_element(&spec);
        chip.selected = Some(descriptor.target == state.target);
        chip.source = Some("DictationTargetDescriptor".to_string());
        elements.push(chip);
    }
    if interactive {
        if let Some(actions) =
            crate::dictation::dictation_recording_actions_for_instance(resolved.generation?, cx)
        {
            for action in actions.into_iter().filter(|action| {
                matches!(
                    action.id.as_str(),
                    "dictation_microphone" | "dictation_jump_latest"
                )
            }) {
                let mut button = ElementInfo::button(elements.len(), &action.title);
                button.semantic_id = action.id.clone();
                button.source = Some(action.id);
                button.value = action.description;
                button.selectable = Some(true);
                elements.push(button);
            }
        }
    }
    if let crate::dictation::DictationSessionPhase::Failed(failure) = &state.phase {
        let mut message = ElementInfo::panel("dictation.failure");
        message.text = Some(failure.safe_message().to_string());
        message.status_kind = Some("failed".to_string());
        elements.push(message);
    }
    let total_count = elements.len();

    Some(SurfaceElementSnapshot {
        elements,
        total_count,
        focused_semantic_id: Some("panel:dictation-overlay".to_string()),
        selected_semantic_id: None,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
}

// ---------------------------------------------------------------------------
// Prompt popup collector (composer picker, history popup)
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
        crate::dictation::DICTATION_MICROPHONE_POPUP_AUTOMATION_ID => {
            collect_cached_prompt_popup_snapshot(&resolved.id, resolved.generation?)
        }
        _ => None,
    }
}

fn collect_cached_prompt_popup_snapshot(
    window_id: &str,
    generation: u64,
) -> Option<SurfaceElementSnapshot> {
    let cached = prompt_popup_semantic_cache()
        .lock()
        .ok()?
        .get(window_id)
        .filter(|snapshot| snapshot.generation == Some(generation))
        .cloned()?;
    Some(SurfaceElementSnapshot {
        total_count: cached.elements.len(),
        elements: cached.elements,
        focused_semantic_id: cached.focused_semantic_id,
        selected_semantic_id: cached.selected_semantic_id,
        warnings: Vec::new(),
        quality: SnapshotQuality::Full,
    })
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
