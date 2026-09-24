//! Scrollbar handle for grouped lists whose row heights the model knows exactly.
//!
//! `ListState`'s own scrollbar offset sums only rows it has measured. After a
//! keyboard reveal jumps past unrendered rows, or a row replacement resets
//! measurements, that offset collapses toward zero while the extent stays
//! complete, so the thumb pins to the top and snaps back as rows re-measure.
//! Here position, extent, and drag all derive from one model (the same exact
//! heights the reveal math uses), so the thumb tracks the true position.
//! Wheel, momentum, and rendering remain owned by `ListState`.

use std::sync::Arc;

use gpui::{point, px, size, Bounds, ListState, Pixels, Point, Size};

use crate::list_item::GroupedListItem;
use crate::scrolling::list_geometry::{
    content_height, offset_for_pixel_top, pixel_top_for_offset, GroupedListRowHeights,
};

#[derive(Clone)]
pub(crate) struct ModelListScrollbarHandle {
    state: ListState,
    items: Arc<[GroupedListItem]>,
    heights: GroupedListRowHeights,
}

impl ModelListScrollbarHandle {
    pub(crate) fn new(
        state: ListState,
        items: Arc<[GroupedListItem]>,
        heights: GroupedListRowHeights,
    ) -> Self {
        Self {
            state,
            items,
            heights,
        }
    }

    /// Exact scrollable content height (rows only; list padding excluded).
    pub(crate) fn content_height(&self) -> Pixels {
        px(content_height(self.items.len(), |ix| {
            self.heights.row_height(&self.items[ix], ix)
        }))
    }
}

impl gpui_component::scroll::ScrollbarHandle for ModelListScrollbarHandle {
    fn viewport_bounds(&self) -> Bounds<Pixels> {
        self.state.viewport_bounds()
    }

    fn offset(&self) -> Point<Pixels> {
        let top = pixel_top_for_offset(self.items.len(), self.state.logical_scroll_top(), |ix| {
            self.heights.row_height(&self.items[ix], ix)
        });
        point(px(0.), px(-top))
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        let target = offset_for_pixel_top(self.items.len(), (-offset.y).as_f32(), |ix| {
            self.heights.row_height(&self.items[ix], ix)
        });
        self.state.scroll_to(target);
    }

    fn content_size(&self) -> Size<Pixels> {
        size(px(0.), self.content_height())
    }

    fn start_drag(&self) {
        self.state.scrollbar_drag_started();
    }

    fn end_drag(&self) {
        self.state.scrollbar_drag_ended();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_component::scroll::ScrollbarHandle;

    fn handle(items: Vec<GroupedListItem>) -> ModelListScrollbarHandle {
        let state = ListState::new(items.len(), gpui::ListAlignment::Top, px(100.));
        ModelListScrollbarHandle::new(
            state,
            items.into(),
            GroupedListRowHeights {
                first_section_header: 20.0,
                section_header: 30.0,
                status: 24.0,
                item: 40.0,
                hide_first_section_header: false,
            },
        )
    }

    /// The regression: a jump past never-rendered rows must still report the
    /// full model offset (ListState's measured sum would read ~0 here).
    #[test]
    fn offset_follows_model_heights_after_jumping_past_unmeasured_rows() {
        let items = std::iter::once(GroupedListItem::SectionHeader("A".into(), None))
            .chain((0..50).map(GroupedListItem::Item))
            .collect::<Vec<_>>();
        let handle = handle(items);
        handle.state.scroll_to(gpui::ListOffset {
            item_ix: 26,
            offset_in_item: px(10.),
        });
        assert_eq!(handle.offset().y, px(-(20. + 25. * 40. + 10.)));
        assert_eq!(handle.content_size().height, px(20. + 50. * 40.));
    }

    /// Dragging to a pixel position lands on the row the model places there.
    #[test]
    fn set_offset_round_trips_through_model_rows() {
        let items = std::iter::once(GroupedListItem::SectionHeader("A".into(), None))
            .chain((0..10).map(GroupedListItem::Item))
            .collect::<Vec<_>>();
        let handle = handle(items);
        handle.set_offset(point(px(0.), px(-(20. + 3. * 40. + 5.))));
        let top = handle.state.logical_scroll_top();
        assert_eq!((top.item_ix, top.offset_in_item), (4, px(5.)));
        assert_eq!(handle.offset().y, px(-(20. + 3. * 40. + 5.)));
    }
}
