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
// Prompt popup collector (composer picker, history popup)
// ---------------------------------------------------------------------------

/// Collect semantic elements only from the exact registered PromptPopup
/// subtype and lifetime. There is deliberately no "whichever popup is open"
/// fallback: a stale or mismatched target must fail closed.
fn collect_exact_prompt_popup_snapshot(
    resolved: &AutomationWindowInfo,
) -> Option<SurfaceElementSnapshot> {
    collect_footer_snapshot(resolved)
}

