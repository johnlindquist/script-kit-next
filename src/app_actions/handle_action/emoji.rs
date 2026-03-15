// Emoji action handlers for handle_action dispatch.
//
// Contains all `emoji_*` action handling: paste, copy, paste-keep-open, pin, unpin.

impl ScriptListApp {
    /// Build an EmojiActionInfo for the currently selected emoji in the picker view.
    fn selected_emoji_action_info(&self) -> Option<crate::actions::EmojiActionInfo> {
        let (filter, selected_index, selected_category) = match &self.current_view {
            AppView::EmojiPickerView {
                filter,
                selected_index,
                selected_category,
            } => (filter.as_str(), *selected_index, *selected_category),
            _ => return None,
        };

        let (ordered_emojis, _pin_count) =
            crate::emoji::filtered_ordered_emojis_with_pins(filter, selected_category, &self.pinned_emojis);
        let emoji = ordered_emojis.get(selected_index)?;

        let frontmost_app_name =
            crate::frontmost_app_tracker::get_last_real_app().map(|app| app.name);

        Some(crate::actions::EmojiActionInfo {
            value: emoji.emoji.to_string(),
            name: emoji.name.to_string(),
            pinned: self.pinned_emojis.contains(emoji.emoji),
            frontmost_app_name,
            category: Some(emoji.category),
        })
    }

    /// Handle emoji-specific actions. Returns a DispatchOutcome.
    fn handle_emoji_action(
        &mut self,
        action_id: &str,
        selected_emoji: Option<crate::actions::EmojiActionInfo>,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        match action_id {
            "emoji_paste" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(
                    action = "emoji_paste",
                    emoji = %emoji.value,
                    emoji_name = %emoji.name,
                    "emoji action"
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.value.clone()));
                self.finalize_paste_after_clipboard_ready(
                    "emoji",
                    &emoji.name,
                    PasteCloseBehavior::HideWindow,
                    cx,
                )
            }
            "emoji_copy" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(action = "emoji_copy", emoji = %emoji.value, "emoji action");
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.value.clone()));
                self.show_hud(format!("Copied {}", emoji.value), Some(HUD_SHORT_MS), cx);
                DispatchOutcome::success()
            }
            "emoji_paste_keep_open" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(
                    action = "emoji_paste_keep_open",
                    emoji = %emoji.value,
                    emoji_name = %emoji.name,
                    "emoji action"
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.value.clone()));
                self.finalize_paste_after_clipboard_ready(
                    "emoji",
                    &emoji.name,
                    PasteCloseBehavior::KeepWindowOpen,
                    cx,
                )
            }
            "emoji_pin" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(action = "emoji_pin", emoji = %emoji.value, "emoji action");
                self.pinned_emojis.insert(emoji.value.clone());
                if let Err(error) = crate::emoji_pins::save_pinned_emojis(&self.pinned_emojis) {
                    tracing::error!(error = %error, emoji = %emoji.value, "failed to pin emoji");
                    self.show_error_toast(format!("Failed to pin emoji: {}", error), cx);
                } else {
                    self.show_hud(format!("Pinned {}", emoji.value), Some(HUD_SHORT_MS), cx);
                    cx.notify();
                }
                DispatchOutcome::success()
            }
            "emoji_unpin" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                tracing::info!(action = "emoji_unpin", emoji = %emoji.value, "emoji action");
                self.pinned_emojis.remove(&emoji.value);
                if let Err(error) = crate::emoji_pins::save_pinned_emojis(&self.pinned_emojis) {
                    tracing::error!(error = %error, emoji = %emoji.value, "failed to unpin emoji");
                    self.show_error_toast(format!("Failed to unpin emoji: {}", error), cx);
                } else {
                    self.show_hud(format!("Unpinned {}", emoji.value), Some(HUD_SHORT_MS), cx);
                    cx.notify();
                }
                DispatchOutcome::success()
            }
            "emoji_copy_unicode" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                let unicode_str: String = {
                    use itertools::Itertools as _;
                    emoji
                        .value
                        .chars()
                        .map(|c| format!("U+{:04X}", c as u32))
                        .join(" ")
                };

                tracing::info!(
                    action = "emoji_copy_unicode",
                    emoji = %emoji.value,
                    unicode = %unicode_str,
                    "emoji action"
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(unicode_str.clone()));
                self.show_hud(
                    format!("Copied {}", unicode_str),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                DispatchOutcome::success()
            }
            "emoji_copy_section" => {
                let Some(emoji) = selected_emoji else {
                    self.show_error_toast("No emoji selected", cx);
                    return DispatchOutcome::success();
                };

                let Some(category) = emoji.category else {
                    self.show_error_toast("No category for this emoji", cx);
                    return DispatchOutcome::success();
                };

                let category_emojis = crate::emoji::emojis_by_category(category);
                let all_emojis: String = {
                    use itertools::Itertools as _;
                    category_emojis.iter().map(|e| e.emoji).join("")
                };

                tracing::info!(
                    action = "emoji_copy_section",
                    category = %category,
                    count = category_emojis.len(),
                    "emoji action"
                );
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(all_emojis));
                self.show_hud(
                    format!(
                        "Copied {} emojis from {}",
                        category_emojis.len(),
                        category
                    ),
                    Some(HUD_SHORT_MS),
                    cx,
                );
                DispatchOutcome::success()
            }
            _ => DispatchOutcome::not_handled(),
        }
    }
}
