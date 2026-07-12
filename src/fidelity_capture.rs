//! Shared conversion from GPUI's completed-frame fidelity telemetry into the
//! finite protocol snapshots consumed by DevTools and capture receipts.

use sha2::{Digest as _, Sha256};

use crate::protocol::{
    FidelityLayoutNode, FidelityPaintTargetSnapshot, FidelityUnscopedPaintSummary, LayoutBounds,
};

pub(crate) fn layout_bounds(bounds: gpui::Bounds<gpui::Pixels>) -> LayoutBounds {
    LayoutBounds {
        x: bounds.origin.x.as_f32(),
        y: bounds.origin.y.as_f32(),
        width: bounds.size.width.as_f32(),
        height: bounds.size.height.as_f32(),
    }
}

/// Convert one completed GPUI window frame into a finite paint-target
/// snapshot. The caller owns target identity; geometry and primitive evidence
/// always come from the supplied window's rendered frame.
pub(crate) fn paint_target_snapshot(
    window: &gpui::Window,
    target_id: impl Into<String>,
    target_kind: impl Into<String>,
    parent_target_id: Option<String>,
) -> FidelityPaintTargetSnapshot {
    let frame_generation = window.rendered_frame_generation();
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
        .map(|(paint_order, summary)| FidelityLayoutNode {
            id: summary.id.clone(),
            kind: summary.kind.as_str().to_string(),
            parent_id: summary.parent_id.clone(),
            bounds: layout_bounds(summary.bounds),
            visible_bounds: layout_bounds(summary.visible_bounds),
            clip_bounds: layout_bounds(summary.clip_bounds),
            union_paint_bounds: layout_bounds(summary.union_paint_bounds),
            primitive_count: summary.primitive_count,
            primitive_digest: summary.primitive_digest.clone(),
            paint_order: paint_order as u64,
            first_paint_order: summary.first_paint_order,
            last_paint_order: summary.last_paint_order,
            measurement_frame_generation: frame_generation,
            measurement_provenance: "paint-time".to_string(),
            coordinate_space: "window".to_string(),
            text_hash: summary.text_hash.clone(),
            text_layout_hash: summary.text_layout_hash.clone(),
            metadata: summary.metadata.clone(),
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

    FidelityPaintTargetSnapshot {
        target_id: target_id.into(),
        target_kind: target_kind.into(),
        parent_target_id,
        window_bounds: layout_bounds(window.bounds()),
        frame_generation,
        nodes,
        unscoped: FidelityUnscopedPaintSummary {
            primitive_count: unscoped_atoms.len(),
            primitive_digest: format!("{:x}", hasher.finalize()),
            primitive_kinds,
            union_paint_bounds: layout_bounds(union_paint_bounds.unwrap_or_default()),
        },
    }
}
