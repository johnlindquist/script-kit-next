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
