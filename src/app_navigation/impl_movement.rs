impl ScriptListApp {
    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        // Switch to keyboard mode and clear hover to prevent dual-highlight
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        // Find the first selectable (non-header) item index
        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));

        // If already at or before first selectable, wrap around to the last selectable item
        if let Some(first) = first_selectable {
            if self.selected_index <= first {
                // Wrap around to the last selectable item
                let last_selectable = grouped_items
                    .iter()
                    .rposition(|item| matches!(item, GroupedListItem::Item(_)));
                if let Some(last) = last_selectable {
                    self.selected_index = last;
                    self.scroll_to_selected_if_needed("keyboard_up_wrap");
                    self.trigger_scroll_activity(cx);
                    cx.notify();
                }
                return;
            }
        }

        if self.selected_index > 0 {
            let mut new_index = self.selected_index - 1;

            // Skip section headers when moving up
            while new_index > 0 {
                if let Some(GroupedListItem::SectionHeader(..)) = grouped_items.get(new_index) {
                    new_index -= 1;
                } else {
                    break;
                }
            }

            // Make sure we didn't land on a section header at index 0
            if let Some(GroupedListItem::SectionHeader(..)) = grouped_items.get(new_index) {
                // Stay at current position if we can't find a valid item
                return;
            }

            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("keyboard_up");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        // Switch to keyboard mode and clear hover to prevent dual-highlight
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        // Get grouped results to check for section headers (cached)
        let (grouped_items, _) = self.get_grouped_results_cached();
        // Clone to avoid borrow issues with self mutation below
        let grouped_items = grouped_items.clone();

        let item_count = grouped_items.len();

        // Find the last selectable (non-header) item index
        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        // If already at or after last selectable, wrap around to the first selectable item
        if let Some(last) = last_selectable {
            if self.selected_index >= last {
                // Wrap around to the first selectable item
                let first_selectable = grouped_items
                    .iter()
                    .position(|item| matches!(item, GroupedListItem::Item(_)));
                if let Some(first) = first_selectable {
                    self.selected_index = first;
                    self.scroll_to_selected_if_needed("keyboard_down_wrap");
                    self.trigger_scroll_activity(cx);
                    cx.notify();
                }
                return;
            }
        }

        if self.selected_index < item_count.saturating_sub(1) {
            let mut new_index = self.selected_index + 1;

            // Skip section headers when moving down
            while new_index < item_count.saturating_sub(1) {
                if let Some(GroupedListItem::SectionHeader(..)) = grouped_items.get(new_index) {
                    new_index += 1;
                } else {
                    break;
                }
            }

            // Make sure we didn't land on a section header at the end
            if let Some(GroupedListItem::SectionHeader(..)) = grouped_items.get(new_index) {
                // Stay at current position if we can't find a valid item
                return;
            }

            self.selected_index = new_index;
            self.scroll_to_selected_if_needed("keyboard_down");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Jump to the first selectable (non-header) item in the list
    fn move_selection_to_first(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();

        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));

        if let Some(first) = first_selectable {
            if self.selected_index != first {
                self.selected_index = first;
                self.scroll_to_selected_if_needed("jump_first");
                self.trigger_scroll_activity(cx);
                cx.notify();
            }
        }
    }

    /// Move selection up by approximately one page (~10 selectable items)
    /// Skips section headers and clamps to the first selectable item
    fn move_selection_page_up(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();

        let first_selectable = grouped_items
            .iter()
            .position(|item| matches!(item, GroupedListItem::Item(_)));

        let Some(first) = first_selectable else {
            return;
        };

        // Already at or before first selectable → no-op (don't wrap)
        if self.selected_index <= first {
            return;
        }

        // Count ~10 selectable items upward from current position
        const PAGE_SIZE: usize = 10;
        let mut remaining = PAGE_SIZE;
        let mut target = self.selected_index;
        for i in (first..self.selected_index).rev() {
            if matches!(grouped_items.get(i), Some(GroupedListItem::Item(_))) {
                target = i;
                remaining -= 1;
                if remaining == 0 {
                    break;
                }
            }
        }

        if target != self.selected_index {
            self.selected_index = target;
            self.scroll_to_selected_if_needed("page_up");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Move selection down by approximately one page (~10 selectable items)
    /// Skips section headers and clamps to the last selectable item
    fn move_selection_page_down(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();

        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        let Some(last) = last_selectable else {
            return;
        };

        // Already at or after last selectable → no-op (don't wrap)
        if self.selected_index >= last {
            return;
        }

        // Count ~10 selectable items downward from current position
        const PAGE_SIZE: usize = 10;
        let target = page_down_target_index(&grouped_items, self.selected_index, PAGE_SIZE);

        if target != self.selected_index {
            self.selected_index = target;
            self.scroll_to_selected_if_needed("page_down");
            self.trigger_scroll_activity(cx);
            cx.notify();
        }
    }

    /// Jump to the last selectable (non-header) item in the list
    fn move_selection_to_last(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);

        let (grouped_items, _) = self.get_grouped_results_cached();
        let grouped_items = grouped_items.clone();

        let last_selectable = grouped_items
            .iter()
            .rposition(|item| matches!(item, GroupedListItem::Item(_)));

        if let Some(last) = last_selectable {
            if self.selected_index != last {
                self.selected_index = last;
                self.scroll_to_selected_if_needed("jump_last");
                self.trigger_scroll_activity(cx);
                cx.notify();
            }
        }
    }
}
