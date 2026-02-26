use crate::ui_foundation::{
    is_key_down, is_key_enter, is_key_escape, is_key_left, is_key_right, is_key_space, is_key_up,
};
use gpui_component::PixelsExt as _;

impl ScriptListApp {
    /// Available vibrancy material presets for the theme customizer
    const VIBRANCY_MATERIALS: &[(theme::VibrancyMaterial, &str)] = &[
        (theme::VibrancyMaterial::Hud, "HUD"),
        (theme::VibrancyMaterial::Popover, "Popover"),
        (theme::VibrancyMaterial::Menu, "Menu"),
        (theme::VibrancyMaterial::Sidebar, "Sidebar"),
        (theme::VibrancyMaterial::Content, "Content"),
    ];

    /// Available font size presets for the theme customizer
    const FONT_SIZE_PRESETS: &[(f32, &str)] = &[
        (12.0, "12"),
        (13.0, "13"),
        (14.0, "14"),
        (15.0, "15"),
        (16.0, "16"),
        (18.0, "18"),
        (20.0, "20"),
    ];

    /// Find the index of a vibrancy material in the presets array
    fn find_vibrancy_material_index(material: theme::VibrancyMaterial) -> usize {
        Self::VIBRANCY_MATERIALS
            .iter()
            .position(|(m, _)| *m == material)
            .unwrap_or(0)
    }

    /// Return a human-readable name for a hex accent color
    fn accent_color_name(color: u32) -> &'static str {
        theme::accent_color_name(color)
    }

    /// Compute scrollbar metrics for a tracked uniform list.
    fn builtin_uniform_list_scrollbar_metrics(
        handle: &UniformListScrollHandle,
        total_items: usize,
        fallback_visible_items: usize,
    ) -> Option<(usize, usize, Option<f32>)> {
        if total_items == 0 {
            return None;
        }

        let state = handle.0.borrow();
        let scroll_offset = state
            .deferred_scroll_to_item
            .map(|deferred| deferred.item_index)
            .unwrap_or_else(|| state.base_handle.logical_scroll_top().0);

        let fallback_visible_items = fallback_visible_items.max(1).min(total_items);

        if let Some(item_size) = state.last_item_size {
            let viewport_height = item_size.item.height.as_f32().max(0.0);
            let content_height = item_size.contents.height.as_f32().max(0.0);
            let visible_items = if content_height > 0.0 {
                ((viewport_height / content_height) * total_items as f32)
                    .ceil()
                    .max(1.0) as usize
            } else {
                fallback_visible_items
            };

            Some((
                scroll_offset,
                visible_items.clamp(1, total_items),
                Some(viewport_height),
            ))
        } else {
            Some((scroll_offset, fallback_visible_items, None))
        }
    }

    /// Build a scrollbar overlay for a tracked builtin uniform list.
    fn builtin_uniform_list_scrollbar(
        &self,
        handle: &UniformListScrollHandle,
        total_items: usize,
        fallback_visible_items: usize,
    ) -> AnyElement {
        let Some((scroll_offset, visible_items, container_height)) =
            Self::builtin_uniform_list_scrollbar_metrics(
                handle,
                total_items,
                fallback_visible_items,
            )
        else {
            return div().into_any_element();
        };

        let mut scrollbar = Scrollbar::new(
            total_items,
            visible_items,
            scroll_offset,
            ScrollbarColors::from_theme(&self.theme),
        );
        if let Some(container_height) = container_height {
            scrollbar = scrollbar.container_height(container_height);
        }

        scrollbar.into_any_element()
    }
}
