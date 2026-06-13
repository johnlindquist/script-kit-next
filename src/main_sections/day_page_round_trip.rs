// Today → main-menu `@context` search round trip.
//
// Requirement (.notes/today-requirements.md "Spine And Agent Chat"):
// triggering an `@context` search row from Today swaps to the main menu so
// the user gets the normal main-menu search experience; accepting a context
// row returns to Today with the accepted token spliced into the originating
// line; Escape (or anything that would close the launcher) cancels back to
// Today unchanged.
//
// The held `Entity<DayPageView>` keeps the editor session alive across the
// trip — the day buffer is saved to disk before leaving, so the splice
// ranges recorded here stay valid (only this round trip may mutate the
// content in between).

use std::ops::Range;

use crate::components::notes_editor::spine::replace_segment_content;

pub(crate) struct DayPageContextReturn {
    pub entity: Entity<DayPageView>,
    /// Byte range of the active line within the day content at hand-off.
    pub line_range: Range<usize>,
    /// Active `@context` segment byte range relative to the line text.
    pub segment_byte_range: Range<usize>,
}

impl DayPageView {
    /// Auto-trigger: typing into ANY `@` mention on the active line swaps to
    /// the real main menu (the exact launcher selection UX) — the Day Page
    /// renders no `@` selector of its own. Called from `on_editor_change`;
    /// only insertions trigger so deletions can still edit a mention inline.
    pub(crate) fn maybe_begin_day_page_context_round_trip_from_edit(
        &mut self,
        previous_len: usize,
        content: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if content.len() <= previous_len {
            return false;
        }
        let selection = self.notes_editor.read(cx).selection(cx);
        let cursor = clamp_to_char_boundary(content, selection.end.min(content.len()));
        let line_range = current_line_range(content, cursor);
        let Some(line) = content.get(line_range.clone()) else {
            return false;
        };
        if line.trim().is_empty() {
            return false;
        }
        let parse = crate::spine::parse_spine(line);
        let cursor_in_line = cursor.saturating_sub(line_range.start);
        let projection = crate::spine::project_cursor(&parse, cursor_in_line);
        let Some(segment) = parse.segments.get(projection.active_segment_index) else {
            return false;
        };
        if !matches!(
            segment.kind,
            crate::spine::SpineSegmentKind::ContextMention { .. }
        ) {
            return false;
        }
        // `project_cursor` defaults to the last segment when the cursor sits
        // past the trailing space; only a cursor INSIDE the mention swaps, so
        // a completed `@selection ` doesn't bounce back to the launcher.
        if cursor_in_line < segment.byte_range.start || cursor_in_line > segment.byte_range.end {
            return false;
        }
        let segment_byte_range = segment.byte_range.clone();
        let Some(segment_text) = line.get(segment_byte_range.clone()).map(str::to_string) else {
            return false;
        };

        // Disk must be authoritative before leaving the surface.
        self.save(cx);

        let Some(app) = self.app.upgrade() else {
            return false;
        };
        let entity = cx.entity();
        tracing::info!(
            target: "script_kit::day_page",
            event = "day_page_context_round_trip_started",
            segment_text = %segment_text,
        );
        window.defer(cx, move |window, cx| {
            app.update(cx, |app, cx| {
                app.begin_day_page_context_round_trip(
                    entity,
                    line_range,
                    segment_byte_range,
                    segment_text,
                    window,
                    cx,
                );
            });
        });
        true
    }

    /// Splice the accepted token into the originating line and re-arm state.
    pub(crate) fn complete_context_round_trip(
        &mut self,
        line_range: Range<usize>,
        segment_byte_range: Range<usize>,
        token: &str,
        alias: Option<crate::ai::message_parts::AiContextPart>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let content = self.notes_editor.read(cx).content(cx);
        let visible_reference = markdown_reference_for_day_page_context_part(token, alias.as_ref())
            .unwrap_or_else(|| token.to_string());
        let spliced = replace_segment_content(
            &content,
            line_range,
            segment_byte_range,
            &visible_reference,
            true,
        );
        let (new_content, cursor) = match spliced {
            Some(done) => done,
            None => {
                // Ranges no longer fit (external edit while away) — append to
                // the end rather than dropping the accepted context.
                let trimmed = content.trim_end();
                let new_content = if trimmed.is_empty() {
                    format!("{visible_reference} ")
                } else {
                    format!("{trimmed} {visible_reference} ")
                };
                let cursor = new_content.len();
                (new_content, cursor)
            }
        };
        // The splice is not typing: pre-set the length so its Change event
        // cannot read as growth and immediately re-open the main-menu search.
        self.last_editor_content_len = new_content.len();
        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value(new_content.clone(), window, cx);
            editor.set_selection(cursor, cursor, window, cx);
        });
        self.session.apply_editor_content(&new_content);
        self.refresh_fragment_open_targets(&new_content);
        if let Some(part) = alias {
            self.spine_runtime
                .register_mention_alias(visible_reference, part);
        }
        self.spine_runtime.clear_alias_cache();
        self.spine_runtime.dismissed_cache_key = None;
        self.spine_runtime.selected_index = 0;
        self.schedule_autosave_flush(cx);
        self.sync_footer(window, cx);
        cx.notify();
    }
}

impl ScriptListApp {
    pub(crate) fn begin_day_page_context_round_trip(
        &mut self,
        entity: Entity<DayPageView>,
        line_range: Range<usize>,
        segment_byte_range: Range<usize>,
        segment_text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.day_page_context_return = Some(DayPageContextReturn {
            entity,
            line_range,
            segment_byte_range,
        });
        self.reset_to_script_list(cx);
        self.set_filter_text_immediate(segment_text, window, cx);
        self.request_script_list_main_filter_focus(cx);
        self.rekey_main_automation_surface_from_current_view();
        self.sync_main_footer_popup(window, cx);
        cx.notify();
    }

    /// Accept hook: a main-menu context row resolved into `token` while a
    /// Day Page round trip was pending. Returns true when the trip completed.
    pub(crate) fn try_complete_day_page_context_round_trip(
        &mut self,
        token: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.try_complete_day_page_context_round_trip_with_alias(token, None, window, cx)
    }

    pub(crate) fn try_complete_day_page_context_round_trip_with_alias(
        &mut self,
        token: &str,
        alias: Option<crate::ai::message_parts::AiContextPart>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(pending) = self.day_page_context_return.take() else {
            return false;
        };
        let token = token.trim();
        if token.is_empty() {
            self.day_page_context_return = Some(pending);
            return false;
        }
        let alias = alias.or_else(|| self.spine_mention_aliases.get(token).cloned());
        let has_alias = alias.is_some();
        let entity = pending.entity.clone();
        entity.update(cx, |view, cx| {
            view.complete_context_round_trip(
                pending.line_range,
                pending.segment_byte_range,
                token,
                alias,
                window,
                cx,
            );
        });
        self.restore_day_page_view_after_round_trip(entity, window, cx);
        tracing::info!(
            target: "script_kit::day_page",
            event = "day_page_context_round_trip_completed",
            token = %token,
            has_alias,
        );
        true
    }

    pub(crate) fn has_day_page_context_round_trip_pending(&self) -> bool {
        self.day_page_context_return.is_some()
    }

    /// Cancel hook: Escape/close from the main menu while a Day Page round
    /// trip is pending returns to Today unchanged instead of closing.
    pub(crate) fn try_cancel_day_page_context_round_trip(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(pending) = self.day_page_context_return.take() else {
            return false;
        };
        self.restore_day_page_view_after_round_trip(pending.entity, window, cx);
        tracing::info!(
            target: "script_kit::day_page",
            event = "day_page_context_round_trip_cancelled",
        );
        true
    }

    /// Deferred cancel for window-handle-less callers (`close_and_reset_window`).
    pub(crate) fn cancel_day_page_context_round_trip_deferred(&mut self, cx: &mut Context<Self>) {
        let app_entity = cx.entity();
        cx.defer(move |cx| {
            let Some(handle) = crate::get_main_window_handle() else {
                return;
            };
            let _ = handle.update(cx, |_, window, cx| {
                app_entity.update(cx, |app, cx| {
                    app.try_cancel_day_page_context_round_trip(window, cx);
                });
            });
        });
    }

    fn restore_day_page_view_after_round_trip(
        &mut self,
        entity: Entity<DayPageView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_filter_text_immediate(String::new(), window, cx);
        self.current_view = AppView::DayPage {
            entity: entity.clone(),
        };
        self.focused_input = FocusedInput::None;
        self.rekey_main_automation_surface_from_current_view();
        entity.update(cx, |view, cx| view.focus_editor(window, cx));
        self.sync_main_footer_popup(window, cx);
        cx.notify();
    }
}
