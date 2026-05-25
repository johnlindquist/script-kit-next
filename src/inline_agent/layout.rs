use crate::platform::accessibility::geometry::{DisplayBounds, RectPx};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InlineAgentLayoutDefaults {
    pub compact_width: f64,
    pub compact_min_width: f64,
    pub compact_max_width: f64,
    pub compact_idle_height: f64,
    pub compact_thinking_height: f64,
    pub compact_completed_height: f64,
    pub edge_gutter: f64,
    pub anchor_gap: f64,
    pub expanded_width: f64,
    pub expanded_min_width: f64,
    pub expanded_max_width: f64,
    pub expanded_height: f64,
}

impl Default for InlineAgentLayoutDefaults {
    fn default() -> Self {
        Self {
            compact_width: 420.0,
            compact_min_width: 320.0,
            compact_max_width: 560.0,
            compact_idle_height: 118.0,
            compact_thinking_height: 144.0,
            compact_completed_height: 252.0,
            edge_gutter: 12.0,
            anchor_gap: 8.0,
            expanded_width: 680.0,
            expanded_min_width: 560.0,
            expanded_max_width: 760.0,
            expanded_height: 560.0,
        }
    }
}

pub fn place_compact_overlay(anchor: RectPx, display: DisplayBounds, height: f64) -> RectPx {
    let defaults = InlineAgentLayoutDefaults::default();
    let visible = display.visible;
    let width = defaults
        .compact_width
        .clamp(defaults.compact_min_width, defaults.compact_max_width);
    let below_y = anchor.y + anchor.height + defaults.anchor_gap;
    let above_y = anchor.y - height - defaults.anchor_gap;
    let max_y = visible.y + visible.height - height - defaults.edge_gutter;
    let y = if below_y <= max_y {
        below_y
    } else {
        above_y.max(visible.y + defaults.edge_gutter)
    };
    let max_x = visible.x + visible.width - width - defaults.edge_gutter;
    let x = anchor.x.clamp(visible.x + defaults.edge_gutter, max_x);

    RectPx {
        x,
        y,
        width,
        height,
    }
}

pub fn place_expanded_overlay(anchor: RectPx, display: DisplayBounds) -> RectPx {
    let defaults = InlineAgentLayoutDefaults::default();
    let visible = display.visible;
    let width = defaults
        .expanded_width
        .clamp(defaults.expanded_min_width, defaults.expanded_max_width)
        .min((visible.width - (defaults.edge_gutter * 2.0)).max(1.0));
    let height = defaults
        .expanded_height
        .min((visible.height - (defaults.edge_gutter * 2.0)).max(1.0));
    let max_x = visible.x + visible.width - width - defaults.edge_gutter;
    let max_y = visible.y + visible.height - height - defaults.edge_gutter;

    RectPx {
        x: anchor.x.clamp(visible.x + defaults.edge_gutter, max_x),
        y: anchor.y.clamp(visible.y + defaults.edge_gutter, max_y),
        width,
        height,
    }
}
