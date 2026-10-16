// Emoji picker actions: paste, copy, copy name/unicode/section, pin, reorder
// pins, forget usage, and the default skin tone.

impl ScriptListApp {
    /// The emoji picker's current display order.
    fn emoji_display(&self) -> Option<crate::emoji::DisplayOrder> {
        let AppView::EmojiPickerView {
            filter,
            selected_category,
            ..
        } = &self.current_view
        else {
            return None;
        };
        Some(crate::emoji::display_ordered_emojis(
            filter,
            *selected_category,
            &self.emoji_head,
        ))
    }

    /// Reload what leads the picker: pins from disk and the current
    /// Frequently Used ranking.
    fn refresh_emoji_head_snapshot(&mut self) {
        self.emoji_head = crate::emoji::EmojiHead {
            pinned: crate::emoji_pins::load_pinned_emojis().unwrap_or_else(|error| {
                tracing::warn!(%error, "failed to load pinned emoji");
                Vec::new()
            }),
            frequent: crate::emoji_usage::load_frequent_snapshot(
                crate::emoji_usage::EMOJI_FREQUENT_LIMIT,
            ),
        };
    }

    /// The selected emoji as the actions see it.
    fn selected_emoji_action_info(&self) -> Option<crate::actions::EmojiActionInfo> {
        let AppView::EmojiPickerView { selected_index, .. } = &self.current_view else {
            return None;
        };
        let display = self.emoji_display()?;
        let emoji = display.emojis.get(*selected_index)?;
        let pin_position = self
            .emoji_head
            .pinned
            .iter()
            .position(|pin| {
                crate::emoji::emoji_for_variant(pin).is_some_and(|e| e.emoji == emoji.emoji)
            })
            .map(|position| (position, self.emoji_head.pinned.len()));
        Some(crate::actions::EmojiActionInfo {
            value: emoji.with_tone(self.emoji_skin_tone).to_string(),
            base: emoji.emoji.to_string(),
            name: emoji.name.to_string(),
            pinned: pin_position.is_some(),
            pin_position,
            frequent: display
                .section_of(*selected_index)
                .is_some_and(|section| section.head == Some(crate::emoji::EmojiHeadKind::Frequent)),
            frontmost_app_name: crate::frontmost_app_tracker::get_last_real_app()
                .map(|app| app.name),
            category: Some(emoji.category),
            tones: emoji.tones,
            skin_tone: self.emoji_skin_tone,
        })
    }

    /// Keep the selection on `base` after the sections change.
    fn select_emoji_in_picker(&mut self, base: &str) {
        let Some(display) = self.emoji_display() else {
            return;
        };
        let Some(index) = display.emojis.iter().position(|emoji| emoji.emoji == base) else {
            return;
        };
        if let AppView::EmojiPickerView { selected_index, .. } = &mut self.current_view {
            *selected_index = index;
        }
        let row = crate::emoji::compute_display_scroll_row(index, &display);
        self.emoji_scroll_handle
            .scroll_to_item(row, gpui::ScrollStrategy::Nearest);
    }

    /// Insert the emoji into the app behind Script Kit and record the use.
    fn paste_emoji_from_picker(
        &mut self,
        value: &str,
        base: &str,
        name: &str,
        keep_open: bool,
        cx: &mut Context<Self>,
    ) -> DispatchOutcome {
        if let Err(error) = crate::emoji_usage::record_emoji_use(base) {
            tracing::debug!(emoji = base, %error, "failed to record emoji usage");
        }
        let receipt = match crate::platform::copy_text(value) {
            Ok(receipt) => receipt,
            Err(error) => {
                return DispatchOutcome::error(
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    format!("Failed to copy emoji: {error}"),
                )
            }
        };
        if receipt.destination() == crate::runtime_policy::CopyDestination::OwnedProcessLocal {
            self.one_window_notice(
                crate::line_notice::INFO,
                receipt.feedback(format!("Copied {value}")),
                cx,
            );
            return DispatchOutcome::success();
        }
        self.finalize_paste_after_clipboard_ready(
            "emoji",
            name,
            if keep_open {
                PasteCloseBehavior::KeepWindowOpen
            } else {
                PasteCloseBehavior::HideWindow
            },
            cx,
        )
    }

    /// Handle `emoji:*` actions. Returns not-handled for other ids.
    fn handle_emoji_action(&mut self, action_id: &str, cx: &mut Context<Self>) -> DispatchOutcome {
        let Some(action) = crate::actions::EmojiPickerAction::from_action_id(action_id) else {
            return DispatchOutcome::not_handled();
        };
        let Some(emoji) = self.selected_emoji_action_info() else {
            self.notice_failed("No emoji selected", cx);
            return DispatchOutcome::success();
        };
        tracing::info!(target: "script_kit::emoji", action = %action_id, emoji = %emoji.value, "emoji action");

        let copy = |this: &mut Self, text: String, message: String, cx: &mut Context<Self>| {
            match crate::platform::copy_text(&text) {
                Ok(receipt) => {
                    this.one_window_notice(crate::line_notice::INFO, receipt.feedback(message), cx);
                    DispatchOutcome::success()
                }
                Err(error) => DispatchOutcome::error(
                    crate::action_helpers::ERROR_ACTION_FAILED,
                    format!("Failed to copy: {error}"),
                ),
            }
        };

        match action {
            crate::actions::EmojiPickerAction::Paste { keep_open } => {
                self.paste_emoji_from_picker(&emoji.value, &emoji.base, &emoji.name, keep_open, cx)
            }
            crate::actions::EmojiPickerAction::Copy => {
                if let Err(error) = crate::emoji_usage::record_emoji_use(&emoji.base) {
                    tracing::debug!(emoji = %emoji.base, %error, "failed to record emoji usage");
                }
                copy(
                    self,
                    emoji.value.clone(),
                    format!("Copied {}", emoji.value),
                    cx,
                )
            }
            crate::actions::EmojiPickerAction::CopyName => copy(
                self,
                emoji.name.clone(),
                format!("Copied \u{201c}{}\u{201d}", emoji.name),
                cx,
            ),
            crate::actions::EmojiPickerAction::CopyUnicode => {
                let text = crate::actions::emoji_unicode_text(&emoji.value);
                copy(self, text.clone(), format!("Copied {text}"), cx)
            }
            crate::actions::EmojiPickerAction::CopySection => {
                let Some(category) = emoji.category else {
                    self.notice_failed("This emoji has no category", cx);
                    return DispatchOutcome::success();
                };
                let all = crate::emoji::emojis_by_category(category);
                let text: String = all
                    .iter()
                    .map(|e| e.with_tone(self.emoji_skin_tone))
                    .collect();
                copy(
                    self,
                    text,
                    format!(
                        "Copied {} emoji from {}",
                        all.len(),
                        category.display_name()
                    ),
                    cx,
                )
            }
            crate::actions::EmojiPickerAction::Pin(edit) => {
                let mut pins = std::mem::take(&mut self.emoji_head.pinned);
                // Pins are stored without a skin tone.
                pins.retain(|pin| {
                    !(crate::emoji::emoji_for_variant(pin).is_some_and(|e| e.emoji == emoji.base)
                        && pin != &emoji.base)
                });
                let changed = crate::emoji_pins::apply_pin_edit(&mut pins, &emoji.base, edit);
                self.emoji_head.pinned = pins;
                if changed {
                    if let Err(error) =
                        crate::emoji_pins::save_pinned_emojis(&self.emoji_head.pinned)
                    {
                        self.notice_failed(format!("Couldn't save pinned emoji: {error}"), cx);
                        return DispatchOutcome::success();
                    }
                    self.select_emoji_in_picker(&emoji.base);
                    let message = match edit {
                        crate::emoji_pins::PinEdit::Pin => format!("Pinned {}", emoji.value),
                        crate::emoji_pins::PinEdit::Unpin => format!("Unpinned {}", emoji.value),
                        _ => format!("Moved {}", emoji.value),
                    };
                    self.one_window_notice(crate::line_notice::INFO, message, cx);
                    cx.notify();
                }
                DispatchOutcome::success()
            }
            crate::actions::EmojiPickerAction::Forget => {
                match crate::emoji_usage::forget_emoji_use(&emoji.base) {
                    Ok(_) => {
                        self.emoji_head
                            .frequent
                            .retain(|value| value != &emoji.base);
                        self.select_emoji_in_picker(&emoji.base);
                        self.one_window_notice(
                            crate::line_notice::INFO,
                            format!("Removed {} from Frequently Used", emoji.value),
                            cx,
                        );
                        cx.notify();
                    }
                    Err(error) => {
                        self.notice_failed(format!("Couldn't reset its ranking: {error}"), cx)
                    }
                }
                DispatchOutcome::success()
            }
            crate::actions::EmojiPickerAction::Tone(tone) => {
                self.emoji_skin_tone = tone;
                if let Err(error) = crate::emoji_pins::save_skin_tone(tone) {
                    self.notice_failed(format!("Couldn't save the skin tone: {error}"), cx);
                } else {
                    self.one_window_notice(
                        crate::line_notice::INFO,
                        format!("Skin tone: {}", tone.title()),
                        cx,
                    );
                }
                cx.notify();
                DispatchOutcome::success()
            }
        }
    }
}
