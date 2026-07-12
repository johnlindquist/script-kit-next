fn paint_measurement_component_type(stable_id: &str) -> protocol::LayoutComponentType {
    use protocol::LayoutComponentType;

    if stable_id.contains("transcript-row-") {
        return LayoutComponentType::ListItem;
    }

    match stable_id {
        "main-view-context-cwd-button"
        | "main-view-context-model-button"
        | "agent-chat-send-button" => LayoutComponentType::Button,
        "main-view-input-shell"
        | "main-view-input-body"
        | "focused-text-mini-input-row"
        | "focused-text-mini-scope-row" => LayoutComponentType::Input,
        "agent-chat-transcript-viewport" => LayoutComponentType::List,
        "native-main-window-footer-spacer" => LayoutComponentType::Panel,
        "main-view-header" => LayoutComponentType::Header,
        "main-view-shell" | "main-view-context-zone" | "main-view-main" => {
            LayoutComponentType::Container
        }
        _ => LayoutComponentType::Other,
    }
}

fn fidelity_layout_bounds(bounds: gpui::Bounds<gpui::Pixels>) -> protocol::LayoutBounds {
    protocol::LayoutBounds {
        x: bounds.origin.x.as_f32(),
        y: bounds.origin.y.as_f32(),
        width: bounds.size.width.as_f32(),
        height: bounds.size.height.as_f32(),
    }
}

impl ScriptListApp {
    /// Append bounds recorded by GPUI's current rendered frame.
    ///
    /// These nodes intentionally use their debug-selector IDs rather than the
    /// formula component names above. Missing selectors remain missing so
    /// fidelity comparisons fail closed instead of silently using estimates.
    fn append_paint_measurements(layout: &mut protocol::LayoutInfo, window: &gpui::Window) {
        use protocol::LayoutComponentInfo;

        let mut measurements: Vec<_> = window.debug_bounds_entries().iter().collect();
        measurements.sort_by(|left, right| left.selector.cmp(&right.selector));
        let frame_generation = window.rendered_frame_generation();

        for measurement in measurements {
            let component_type = paint_measurement_component_type(measurement.selector.as_str());
            let bounds = measurement.bounds;
            let visible = measurement.visible_bounds;
            let clip = measurement.clip_bounds;

            layout.components.push(
                LayoutComponentInfo::new(measurement.selector.clone(), component_type)
                    .with_bounds(
                        bounds.origin.x.as_f32(),
                        bounds.origin.y.as_f32(),
                        bounds.size.width.as_f32(),
                        bounds.size.height.as_f32(),
                    )
                    .with_measurement("paint-time", "window")
                    .with_paint_visibility(
                        visible.origin.x.as_f32(),
                        visible.origin.y.as_f32(),
                        visible.size.width.as_f32(),
                        visible.size.height.as_f32(),
                        clip.origin.x.as_f32(),
                        clip.origin.y.as_f32(),
                        clip.size.width.as_f32(),
                        clip.size.height.as_f32(),
                    )
                    .with_measurement_frame(frame_generation),
            );
        }

        if window.fidelity_capture_active() {
            use sha2::{Digest as _, Sha256};

            let mut summaries: Vec<_> = window.fidelity_scope_summaries().iter().collect();
            summaries.sort_by(|left, right| {
                left.first_paint_order
                    .unwrap_or(u64::MAX)
                    .cmp(&right.first_paint_order.unwrap_or(u64::MAX))
                    .then_with(|| left.id.cmp(&right.id))
            });
            let nodes = summaries
                .into_iter()
                .enumerate()
                .map(|(paint_order, summary)| protocol::FidelityLayoutNode {
                    id: summary.id.clone(),
                    kind: summary.kind.as_str().to_string(),
                    parent_id: summary.parent_id.clone(),
                    bounds: fidelity_layout_bounds(summary.bounds),
                    visible_bounds: fidelity_layout_bounds(summary.visible_bounds),
                    clip_bounds: fidelity_layout_bounds(summary.clip_bounds),
                    union_paint_bounds: fidelity_layout_bounds(summary.union_paint_bounds),
                    primitive_count: summary.primitive_count,
                    primitive_digest: summary.primitive_digest.clone(),
                    paint_order: paint_order as u64,
                    first_paint_order: summary.first_paint_order,
                    last_paint_order: summary.last_paint_order,
                    measurement_frame_generation: frame_generation,
                    measurement_provenance: "paint-time".to_string(),
                    coordinate_space: "window".to_string(),
                    text_hash: None,
                    text_layout_hash: None,
                    metadata: None,
                })
                .collect();

            let unscoped_atoms: Vec<_> = window
                .fidelity_paint_atoms()
                .iter()
                .filter(|atom| atom.scope_id == "__unscoped__")
                .collect();
            let mut hasher = Sha256::new();
            let mut primitive_kinds = Vec::new();
            let mut union_paint_bounds: Option<gpui::Bounds<gpui::Pixels>> = None;
            for atom in &unscoped_atoms {
                hasher.update(atom.primitive_kind.as_bytes());
                hasher.update([0]);
                hasher.update(atom.payload_hash.as_bytes());
                hasher.update(atom.paint_order.to_le_bytes());
                hasher.update(atom.bounds.origin.x.as_f32().to_bits().to_le_bytes());
                hasher.update(atom.bounds.origin.y.as_f32().to_bits().to_le_bytes());
                hasher.update(atom.bounds.size.width.as_f32().to_bits().to_le_bytes());
                hasher.update(atom.bounds.size.height.as_f32().to_bits().to_le_bytes());
                hasher.update(
                    atom.clipped_bounds
                        .origin
                        .x
                        .as_f32()
                        .to_bits()
                        .to_le_bytes(),
                );
                hasher.update(
                    atom.clipped_bounds
                        .origin
                        .y
                        .as_f32()
                        .to_bits()
                        .to_le_bytes(),
                );
                hasher.update(
                    atom.clipped_bounds
                        .size
                        .width
                        .as_f32()
                        .to_bits()
                        .to_le_bytes(),
                );
                hasher.update(
                    atom.clipped_bounds
                        .size
                        .height
                        .as_f32()
                        .to_bits()
                        .to_le_bytes(),
                );
                hasher.update(atom.opacity.to_bits().to_le_bytes());
                if !primitive_kinds.contains(&atom.primitive_kind) {
                    primitive_kinds.push(atom.primitive_kind.clone());
                }
                union_paint_bounds = Some(match union_paint_bounds {
                    Some(current) => current.union(&atom.bounds),
                    None => atom.bounds,
                });
            }

            layout.fidelity = Some(protocol::FidelityLayoutSnapshot {
                capture_target: "agent-chat".to_string(),
                frame_generation,
                nodes,
                unscoped: protocol::FidelityUnscopedPaintSummary {
                    primitive_count: unscoped_atoms.len(),
                    primitive_digest: format!("{:x}", hasher.finalize()),
                    primitive_kinds,
                    union_paint_bounds: fidelity_layout_bounds(
                        union_paint_bounds.unwrap_or_default(),
                    ),
                },
            });
        }
    }
}
