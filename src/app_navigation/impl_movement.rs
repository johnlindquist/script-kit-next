impl ScriptListApp {
    #[inline]
    fn enter_keyboard_mode(&mut self, cx: &mut Context<Self>) {
        self.input_mode = InputMode::Keyboard;
        self.hovered_index = None;
        self.hide_mouse_cursor(cx);
    }

    #[inline]
    fn set_selected_index(&mut self, ix: usize, reason: &str, cx: &mut Context<Self>) {
        if ix == self.selected_index {
            return;
        }

        self.selected_index = ix;
        self.scroll_to_selected_if_needed(reason);
        self.trigger_scroll_activity(cx);
        cx.notify();
    }

    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        self.enter_keyboard_mode(cx);

        let (target_index, reason) = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            if grouped_items.is_empty() {
                return;
            }

            let clamped_index = self
                .selected_index
                .min(grouped_items.len().saturating_sub(1));
            let first_selectable = grouped_items
                .iter()
                .position(|item| matches!(item, GroupedListItem::Item(_)));

            if let Some(first) = first_selectable {
                if clamped_index <= first {
                    let last_selectable = grouped_items
                        .iter()
                        .rposition(|item| matches!(item, GroupedListItem::Item(_)));
                    if let Some(last) = last_selectable {
                        (last, "keyboard_up_wrap")
                    } else {
                        (clamped_index, "keyboard_up_clamp")
                    }
                } else if clamped_index > 0 {
                    let mut new_index = clamped_index - 1;
                    while new_index > 0 {
                        if let Some(GroupedListItem::SectionHeader(..)) =
                            grouped_items.get(new_index)
                        {
                            new_index -= 1;
                        } else {
                            break;
                        }
                    }

                    if matches!(
                        grouped_items.get(new_index),
                        Some(GroupedListItem::SectionHeader(..))
                    ) {
                        (clamped_index, "keyboard_up_clamp")
                    } else {
                        (new_index, "keyboard_up")
                    }
                } else {
                    (clamped_index, "keyboard_up_clamp")
                }
            } else {
                (clamped_index, "keyboard_up_clamp")
            }
        };

        self.set_selected_index(target_index, reason, cx);
    }

    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        self.enter_keyboard_mode(cx);

        let (target_index, reason) = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            if grouped_items.is_empty() {
                return;
            }

            let clamped_index = self
                .selected_index
                .min(grouped_items.len().saturating_sub(1));
            let item_count = grouped_items.len();
            let last_selectable = grouped_items
                .iter()
                .rposition(|item| matches!(item, GroupedListItem::Item(_)));

            if let Some(last) = last_selectable {
                if clamped_index >= last {
                    let first_selectable = grouped_items
                        .iter()
                        .position(|item| matches!(item, GroupedListItem::Item(_)));
                    if let Some(first) = first_selectable {
                        (first, "keyboard_down_wrap")
                    } else {
                        (clamped_index, "keyboard_down_clamp")
                    }
                } else if clamped_index < item_count.saturating_sub(1) {
                    let mut new_index = clamped_index + 1;
                    while new_index < item_count.saturating_sub(1) {
                        if let Some(GroupedListItem::SectionHeader(..)) =
                            grouped_items.get(new_index)
                        {
                            new_index += 1;
                        } else {
                            break;
                        }
                    }

                    if matches!(
                        grouped_items.get(new_index),
                        Some(GroupedListItem::SectionHeader(..))
                    ) {
                        (clamped_index, "keyboard_down_clamp")
                    } else {
                        (new_index, "keyboard_down")
                    }
                } else {
                    (clamped_index, "keyboard_down_clamp")
                }
            } else {
                (clamped_index, "keyboard_down_clamp")
            }
        };

        self.set_selected_index(target_index, reason, cx);
    }

    /// Jump to the first selectable (non-header) item in the list
    fn move_selection_to_first(&mut self, cx: &mut Context<Self>) {
        self.enter_keyboard_mode(cx);

        let (target_index, reason) = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            if grouped_items.is_empty() {
                return;
            }

            let clamped_index = self
                .selected_index
                .min(grouped_items.len().saturating_sub(1));
            let first_selectable = grouped_items
                .iter()
                .position(|item| matches!(item, GroupedListItem::Item(_)));

            if let Some(first) = first_selectable {
                (first, "jump_first")
            } else {
                (clamped_index, "jump_first_clamp")
            }
        };

        self.set_selected_index(target_index, reason, cx);
    }

    /// Move selection up by approximately one page (~10 selectable items)
    /// Skips section headers and clamps to the first selectable item
    fn move_selection_page_up(&mut self, cx: &mut Context<Self>) {
        self.enter_keyboard_mode(cx);

        let (target_index, reason) = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            if grouped_items.is_empty() {
                return;
            }

            let clamped_index = self
                .selected_index
                .min(grouped_items.len().saturating_sub(1));
            let first_selectable = grouped_items
                .iter()
                .position(|item| matches!(item, GroupedListItem::Item(_)));

            if let Some(first) = first_selectable {
                if clamped_index <= first {
                    (clamped_index, "page_up_clamp")
                } else {
                    const PAGE_SIZE: usize = 10;
                    let mut remaining = PAGE_SIZE;
                    let mut target = clamped_index;
                    for i in (first..clamped_index).rev() {
                        if matches!(grouped_items.get(i), Some(GroupedListItem::Item(_))) {
                            target = i;
                            remaining -= 1;
                            if remaining == 0 {
                                break;
                            }
                        }
                    }

                    if target != clamped_index {
                        (target, "page_up")
                    } else {
                        (clamped_index, "page_up_clamp")
                    }
                }
            } else {
                (clamped_index, "page_up_clamp")
            }
        };

        self.set_selected_index(target_index, reason, cx);
    }

    /// Move selection down by approximately one page (~10 selectable items)
    /// Skips section headers and clamps to the last selectable item
    fn move_selection_page_down(&mut self, cx: &mut Context<Self>) {
        self.enter_keyboard_mode(cx);

        let (target_index, reason) = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            if grouped_items.is_empty() {
                return;
            }

            let clamped_index = self
                .selected_index
                .min(grouped_items.len().saturating_sub(1));
            let last_selectable = grouped_items
                .iter()
                .rposition(|item| matches!(item, GroupedListItem::Item(_)));

            if let Some(last) = last_selectable {
                if clamped_index >= last {
                    (clamped_index, "page_down_clamp")
                } else {
                    const PAGE_SIZE: usize = 10;
                    let target = page_down_target_index(&grouped_items, clamped_index, PAGE_SIZE);
                    if target != clamped_index {
                        (target, "page_down")
                    } else {
                        (clamped_index, "page_down_clamp")
                    }
                }
            } else {
                (clamped_index, "page_down_clamp")
            }
        };

        self.set_selected_index(target_index, reason, cx);
    }

    /// Jump to the last selectable (non-header) item in the list
    fn move_selection_to_last(&mut self, cx: &mut Context<Self>) {
        self.enter_keyboard_mode(cx);

        let (target_index, reason) = {
            let (grouped_items, _) = self.get_grouped_results_cached();
            if grouped_items.is_empty() {
                return;
            }

            let clamped_index = self
                .selected_index
                .min(grouped_items.len().saturating_sub(1));
            let last_selectable = grouped_items
                .iter()
                .rposition(|item| matches!(item, GroupedListItem::Item(_)));

            if let Some(last) = last_selectable {
                (last, "jump_last")
            } else {
                (clamped_index, "jump_last_clamp")
            }
        };

        self.set_selected_index(target_index, reason, cx);
    }
}
