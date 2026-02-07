use super::*;

impl NotesApp {
    /// Render the actions panel overlay (Cmd+K)
    ///
    /// IMPORTANT: Uses items_start + fixed top padding to keep the search input
    /// at a stable position. Without this, the panel would re-center when items
    /// are filtered out, causing the search input to jump around.
    pub(super) fn render_actions_panel_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let panel = self
            .actions_panel
            .as_ref()
            .map(|panel| panel.clone().into_any_element())
            .unwrap_or_else(|| div().into_any_element());

        // Fixed top offset so search input stays at same position regardless of item count

        div()
            .id("actions-panel-overlay")
            .absolute()
            .inset_0()
            .bg(Self::get_modal_overlay_background()) // Theme-aware overlay
            .flex()
            .flex_col()
            .items_center() // Horizontally centered
            .justify_start() // Vertically aligned to top (not centered!)
            .pt(px(ACTIONS_PANEL_TOP_OFFSET))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.close_actions_panel(window, cx);
                }),
            )
            .child(
                div()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                        // Stop propagation - don't close when clicking panel
                    })
                    .child(panel),
            )
    }

    /// Render the browse panel overlay (Cmd+P)
    pub(super) fn render_browse_panel_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // If we have a browse panel entity, render it
        // Otherwise render an empty container that will close on click
        if let Some(ref browse_panel) = self.browse_panel {
            div()
                .id("browse-panel-overlay")
                .absolute()
                .inset_0()
                .child(browse_panel.clone())
        } else {
            // Fallback: create inline browse panel
            let note_items: Vec<NoteListItem> = self
                .notes
                .iter()
                .map(|note| NoteListItem::from_note(note, Some(note.id) == self.selected_note_id))
                .collect();

            // We need a simple inline version since we can't create entities in render
            div()
                .id("browse-panel-overlay")
                .absolute()
                .inset_0()
                .bg(Self::get_modal_overlay_background()) // Theme-aware overlay
                .flex()
                .items_center()
                .justify_center()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.close_browse_panel(window, cx);
                }))
                .child(
                    div()
                        .w(px(BROWSE_PANEL_WIDTH))
                        .max_h(px(BROWSE_PANEL_MAX_HEIGHT))
                        // NO .bg() - overlay already provides backdrop, avoid double-layering opacity
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_lg()
                        // Shadow disabled for vibrancy - shadows on transparent elements cause gray fill
                        .p_4()
                        .on_mouse_down(gpui::MouseButton::Left, |_, _, _| {
                            // Stop propagation
                        })
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{} notes available", note_items.len())),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .mt_2()
                                .child("Press Escape to close"),
                        ),
                )
        }
    }
}
