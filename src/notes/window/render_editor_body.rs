use super::*;

fn notes_supports_spine_action(action: &crate::spine::SpineListAction) -> bool {
    matches!(
        action,
        crate::spine::SpineListAction::InsertSegmentText { .. }
            | crate::spine::SpineListAction::ResolveSegment { .. }
    )
}

impl NotesApp {
    pub(super) fn render_editor_body(
        &mut self,
        is_trash: bool,
        has_selection: bool,
        is_preview: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let no_notes = self.get_visible_notes().is_empty();

        if no_notes && !has_selection && is_trash {
            return div()
                .id("notes-empty-trash")
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .child(
                    div()
                        .text_base()
                        .text_color(cx.theme().muted_foreground)
                        .child("Trash is empty"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground.opacity(OPACITY_SUBTLE))
                        .child("Deleted notes will appear here"),
                )
                .child(
                    div()
                        .id("back-to-notes-link")
                        .text_xs()
                        .text_color(cx.theme().accent)
                        .cursor_pointer()
                        .hover(|s| s.text_color(cx.theme().foreground))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.set_view_mode(NotesViewMode::AllNotes, window, cx);
                        }))
                        .child("← Back to Notes"),
                )
                .into_any_element();
        }

        if no_notes && !has_selection {
            return div()
                .id("notes-empty-state")
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .child(
                    div()
                        .text_base()
                        .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                        .child("No notes yet"),
                )
                .child(
                    div()
                        .id("create-first-note")
                        .text_sm()
                        .text_color(cx.theme().accent)
                        .cursor_pointer()
                        .hover(|s| s.text_color(cx.theme().foreground))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.create_note(window, cx);
                        }))
                        .child("Create your first note"),
                )
                .child(
                    div().flex().flex_col().items_center().gap_1().pt_2().child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child("⌘N  new"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground.opacity(OPACITY_MUTED))
                                    .child("⌘⇧N  from clipboard"),
                            ),
                    ),
                )
                .into_any_element();
        }

        if is_preview {
            let content = self.editor_state.read(cx).value().to_string();
            let metrics = style::adopted_metrics();
            let entity = cx.entity().downgrade();
            let on_toggle_task: markdown::TaskToggleHandler =
                std::rc::Rc::new(move |marker_range, checked, window, cx| {
                    if let Some(app) = entity.upgrade() {
                        app.update(cx, |app, cx| {
                            app.toggle_task_marker_at(marker_range, checked, window, cx);
                        });
                    }
                });
            return div()
                .id("notes-markdown-preview")
                .flex_1()
                .min_h(px(0.))
                .track_scroll(&self.preview_scroll_handle)
                .overflow_y_scroll()
                .vertical_scrollbar(&self.preview_scroll_handle)
                .px(px(metrics.editor_padding_x))
                .py(px(metrics.editor_padding_y))
                .child(markdown::render_markdown_preview_interactive(
                    &content,
                    cx.theme(),
                    on_toggle_task,
                ))
                .into_any_element();
        }

        // Ghost text renders through the editor's native inline-completion
        // channel (`sync_notes_ghost_inline_completion`), shaped inside the
        // editor's own text layout, so it aligns with the caret exactly. Do
        // not reintroduce an absolutely positioned overlay here: hand-derived
        // padding/advance/line-height math drifts from the Input's real
        // metrics and renders the ghost offset from the text.
        let input = crate::components::notes_editor::NotesEditor::render_input_state(
            &self.editor_state,
            cx,
        );
        let spine_panel = self.render_notes_spine_panel(cx);
        div()
            .relative()
            .flex_1()
            .min_h(px(0.))
            .child(input)
            .when_some(spine_panel, |d, panel| d.child(panel))
            .into_any_element()
    }
}

impl NotesApp {
    pub(super) fn notes_spine_surface_allows_editor_list(&self) -> bool {
        self.selected_note_id.is_some()
            && self.view_mode != NotesViewMode::Trash
            && self.surface_mode == NotesSurfaceMode::Notes
            && !self.preview_enabled
            && !self.show_search
            && !self.command_bar.is_open()
            && !self.note_switcher.is_open()
    }

    pub(super) fn notes_spine_input(
        &self,
        cx: &gpui::App,
    ) -> Option<(
        String,
        std::ops::Range<usize>,
        crate::spine::SpineParse,
        crate::spine::SpineCursorProjection,
    )> {
        if !self.notes_spine_surface_allows_editor_list() {
            return None;
        }
        let content = self.notes_editor.read(cx).content(cx);
        let selection = self.notes_editor.read(cx).selection(cx);
        let cursor = crate::components::notes_editor::spine::clamp_to_char_boundary(
            &content,
            selection.end.min(content.len()),
        );
        let line_range =
            crate::components::notes_editor::spine::current_line_range(&content, cursor);
        let line = &content[line_range.clone()];
        let line_cursor = cursor.saturating_sub(line_range.start);
        let parse = crate::spine::parse_spine(line);
        let projection = crate::spine::project_cursor(&parse, line_cursor);
        if !crate::components::notes_editor::spine::spine_projection_owns_editor_list(
            &parse,
            &projection,
        ) {
            return None;
        }
        if matches!(
            projection.active_segment_kind,
            crate::spine::SpineSegmentKind::ContextMention { .. }
                | crate::spine::SpineSegmentKind::ProjectCwd { .. }
        ) {
            return None;
        }

        let key = format!(
            "{}\u{1f}cursor={}\u{1f}active={:?}",
            line, line_cursor, projection.active_segment_kind
        );
        Some((key, line_range, parse, projection))
    }

    pub(super) fn notes_spine_model(
        &mut self,
        cx: &gpui::App,
    ) -> Option<crate::components::notes_editor::spine::NotesEditorSpineModel> {
        let (key, line_range, parse, projection) = self.notes_spine_input(cx)?;
        if self.spine_runtime.dismissed_cache_key.as_deref() == Some(key.as_str()) {
            return None;
        }

        if self.spine_runtime.cache_key == key {
            self.spine_runtime.coerce_selection_for_cached_rows();
            return Some(
                crate::components::notes_editor::spine::NotesEditorSpineModel {
                    line_range,
                    parse,
                    projection,
                    grouped: self.spine_runtime.grouped_cache.clone(),
                    flat: self.spine_runtime.flat_cache.clone(),
                },
            );
        }

        let rows = self.build_notes_spine_rows(&parse, &projection)?;
        if rows.flat.is_empty() {
            return None;
        }
        let grouped = rows.grouped;
        let flat = rows.flat;
        self.spine_runtime
            .replace_cached_rows(key, grouped.clone(), flat.clone(), rows.aliases);

        Some(
            crate::components::notes_editor::spine::NotesEditorSpineModel {
                line_range,
                parse,
                projection,
                grouped,
                flat,
            },
        )
    }

    fn build_notes_spine_rows(
        &self,
        parse: &crate::spine::SpineParse,
        projection: &crate::spine::SpineCursorProjection,
    ) -> Option<crate::components::notes_editor::spine::NotesEditorSpineRows> {
        if matches!(
            projection.active_segment_kind,
            crate::spine::SpineSegmentKind::ContextMention { .. }
                | crate::spine::SpineSegmentKind::ProjectCwd { .. }
        ) {
            return None;
        }
        let sections = crate::spine::list::build_spine_list_sections_full_with_resolved_tokens(
            parse,
            projection,
            None,
            &|_token| false,
        );
        let rows = crate::components::notes_editor::spine::push_spine_sections_as_grouped(
            sections,
            notes_supports_spine_action,
        );
        (!rows.flat.is_empty()).then_some(rows)
    }

    fn render_notes_spine_panel(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let model = self.notes_spine_model(cx)?;
        let theme = crate::theme::get_cached_theme();
        let item_colors = crate::list_item::ListItemColors::from_theme(&theme);
        let main_menu_theme = crate::designs::current_main_menu_theme();
        let editor_surface =
            crate::components::notes_editor::NotesEditorSurfaceStyle::from_theme(&theme);
        let selected =
            crate::list_item::coerce_selection(&model.grouped, self.spine_runtime.selected_index);

        let mut rows = div().flex().flex_col().w_full();
        for (ix, grouped_item) in model.grouped.iter().enumerate() {
            match grouped_item {
                crate::list_item::GroupedListItem::SectionHeader(label, icon) => {
                    rows = rows.child(
                        div()
                            .h(px(
                                crate::list_item::effective_section_header_height_for_theme(
                                    main_menu_theme,
                                ),
                            ))
                            .child(crate::list_item::render_section_header(
                                label,
                                icon.as_deref(),
                                item_colors,
                                ix == 0,
                            )),
                    );
                }
                crate::list_item::GroupedListItem::Status(status) => {
                    rows = rows.child(
                        div()
                            .h(px(
                                crate::list_item::effective_source_status_row_height_for_theme(
                                    main_menu_theme,
                                ),
                            ))
                            .px_4()
                            .flex()
                            .items_center()
                            .text_sm()
                            .text_color(gpui::rgb(item_colors.text_secondary))
                            .child(status.label.clone()),
                    );
                }
                crate::list_item::GroupedListItem::Item(flat_idx) => {
                    let Some(row) = model.flat.get(*flat_idx) else {
                        continue;
                    };
                    let is_selected = selected == Some(ix);
                    let click_handler = cx.listener(
                        move |this: &mut NotesApp, _event: &gpui::MouseDownEvent, _window, cx| {
                            this.spine_runtime.selected_index = ix;
                            cx.notify();
                        },
                    );
                    rows = rows.child(
                        div()
                            .h(px(crate::list_item::effective_list_item_height_for_theme(
                                main_menu_theme,
                            )))
                            .on_mouse_down(gpui::MouseButton::Left, click_handler)
                            .child(
                                crate::list_item::ListItem::new(row.title.to_string(), item_colors)
                                    .index(ix)
                                    .selected(is_selected)
                                    .hovered(false)
                                    .main_menu_theme(main_menu_theme)
                                    .semantic_id(row.id.to_string())
                                    .description_opt(row.subtitle.as_ref().map(|s| s.to_string()))
                                    .icon_kind_opt(None)
                                    .type_accessory(crate::list_item::TypeAccessory {
                                        label: row.kind.type_accessory_info().0,
                                        icon_name: row.kind.type_accessory_info().1,
                                    })
                                    .source_hint_opt(row.meta.as_ref().map(|m| m.to_string())),
                            ),
                    );
                }
            }
        }

        Some(
            div()
                .id("notes-spine-list")
                .absolute()
                .inset_0()
                .bg(rgba(editor_surface.occlusion_rgba))
                .occlude()
                .overflow_y_scroll()
                .child(rows)
                .into_any_element(),
        )
    }

    pub(super) fn selected_notes_spine_row(
        &self,
        model: &crate::components::notes_editor::spine::NotesEditorSpineModel,
    ) -> Option<crate::spine::SpineListRow> {
        model.selected_row(self.spine_runtime.selected_index)
    }

    pub(super) fn move_notes_spine_selection(
        &mut self,
        direction: isize,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(model) = self.notes_spine_model(cx) else {
            return false;
        };
        let len = model.grouped.len();
        if len == 0 {
            return false;
        }
        let mut next = self.spine_runtime.selected_index.min(len - 1);
        loop {
            next = if direction < 0 {
                next.saturating_sub(1)
            } else {
                (next + 1).min(len - 1)
            };
            if matches!(
                model.grouped.get(next),
                Some(crate::list_item::GroupedListItem::Item(_))
            ) {
                self.spine_runtime.selected_index = next;
                cx.notify();
                return true;
            }
            if next == 0 || next == len - 1 {
                return true;
            }
        }
    }

    pub(super) fn reset_notes_spine_navigation(&mut self, cx: &mut Context<Self>) {
        let key = self.notes_spine_input(cx).map(|(key, _, _, _)| key);
        self.spine_runtime.dismiss_current_key(key);
        cx.notify();
    }

    fn replace_notes_spine_segment(
        &mut self,
        model: &crate::components::notes_editor::spine::NotesEditorSpineModel,
        segment_index: usize,
        segment_byte_range: std::ops::Range<usize>,
        replacement: &str,
        trailing_space: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let content = self.notes_editor.read(cx).content(cx);
        let Some(segment) = model.parse.segments.get(segment_index) else {
            return false;
        };
        if segment.byte_range != segment_byte_range {
            return false;
        }
        let Some((new_content, cursor)) =
            crate::components::notes_editor::spine::replace_segment_content(
                &content,
                model.line_range.clone(),
                segment_byte_range,
                replacement,
                trailing_space,
            )
        else {
            return false;
        };

        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value(new_content.clone(), window, cx);
            editor.set_selection(cursor, cursor, window, cx);
        });
        self.spine_runtime.selected_index = 0;
        self.spine_runtime.clear_alias_cache();
        self.on_editor_change(window, cx);
        cx.notify();
        true
    }

    fn apply_notes_spine_action(
        &mut self,
        action: crate::spine::SpineListAction,
        model: &crate::components::notes_editor::spine::NotesEditorSpineModel,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match action {
            crate::spine::SpineListAction::InsertSegmentText {
                segment_index,
                segment_byte_range,
                text,
                trailing_space,
            } => self.replace_notes_spine_segment(
                model,
                segment_index,
                segment_byte_range,
                text.as_ref(),
                trailing_space,
                window,
                cx,
            ),
            crate::spine::SpineListAction::ResolveSegment {
                segment_index,
                segment_byte_range,
                replacement,
                resolution_source,
                trailing_space,
                ..
            } => {
                if resolution_source.as_ref() == "cwd" {
                    return false;
                }
                self.replace_notes_spine_segment(
                    model,
                    segment_index,
                    segment_byte_range,
                    replacement.as_ref(),
                    trailing_space,
                    window,
                    cx,
                )
            }
            crate::spine::SpineListAction::OpenFileSearchPortal { .. }
            | crate::spine::SpineListAction::OpenModeExit { .. }
            | crate::spine::SpineListAction::Noop => false,
        }
    }

    pub(super) fn accept_notes_spine_selection(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(model) = self.notes_spine_model(cx) else {
            return false;
        };
        let Some(row) = self.selected_notes_spine_row(&model) else {
            return false;
        };
        let row_id = row.id.to_string();
        let handled = self.apply_notes_spine_action(row.action, &model, window, cx);
        if !handled {
            tracing::warn!(
                target: "script_kit::spine",
                event = "notes_spine_action_unhandled",
                row_id = %row_id,
            );
        }
        handled
    }
}
