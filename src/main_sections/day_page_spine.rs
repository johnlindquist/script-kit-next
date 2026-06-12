// Day Page spine adapter: current-line parsing, row projection, and acceptance.

use std::{collections::HashMap, ops::Range};

struct DayPageSpineRows {
    grouped: Vec<GroupedListItem>,
    flat: Vec<scripts::SearchResult>,
    aliases: HashMap<String, (String, crate::ai::message_parts::AiContextPart)>,
}

impl DayPageSpineRows {
    fn new(grouped: Vec<GroupedListItem>, flat: Vec<scripts::SearchResult>) -> Self {
        Self {
            grouped,
            flat,
            aliases: HashMap::new(),
        }
    }
}

struct DayPageSpineModel {
    line_range: Range<usize>,
    parse: crate::spine::SpineParse,
    projection: crate::spine::SpineCursorProjection,
    grouped: Vec<GroupedListItem>,
    flat: Vec<scripts::SearchResult>,
    active_empty_subsearch: Option<crate::spine::catalog_subsearch::ContextSubsearchSource>,
}

impl DayPageSpineModel {
    fn selected_row(&self, selected_index: usize) -> Option<crate::spine::SpineListRow> {
        let selected_index = crate::list_item::coerce_selection(&self.grouped, selected_index)?;
        let GroupedListItem::Item(flat_idx) = self.grouped.get(selected_index)? else {
            return None;
        };
        let scripts::SearchResult::SpineProjection(row) = self.flat.get(*flat_idx)? else {
            return None;
        };
        row.is_selectable.then(|| row.clone())
    }
}

fn current_line_range(content: &str, cursor: usize) -> Range<usize> {
    let cursor = clamp_to_char_boundary(content, cursor.min(content.len()));
    let start = content[..cursor].rfind('\n').map_or(0, |idx| idx + 1);
    let end = content[cursor..]
        .find('\n')
        .map_or(content.len(), |idx| cursor + idx);
    start..end
}

fn clamp_to_char_boundary(text: &str, mut pos: usize) -> usize {
    pos = pos.min(text.len());
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

fn spine_projection_owns_day_page_list(
    parse: &crate::spine::SpineParse,
    projection: &crate::spine::SpineCursorProjection,
) -> bool {
    if matches!(
        projection.active_segment_kind,
        crate::spine::SpineSegmentKind::Capture { .. }
    ) {
        let raw = parse
            .segments
            .get(projection.active_segment_index)
            .map(|segment| segment.raw.as_str())
            .unwrap_or("");
        if !raw.starts_with(';') {
            return false;
        }
    }

    !matches!(
        projection.active_segment_kind,
        crate::spine::SpineSegmentKind::FreeText
    ) || (projection.is_tail
        && projection.has_prompt_segments
        && crate::spine::parse_has_prompt_builder_segments(parse))
}

fn active_empty_day_page_subsearch(
    projection: &crate::spine::SpineCursorProjection,
) -> Option<crate::spine::catalog_subsearch::ContextSubsearchSource> {
    let crate::spine::SpineSegmentKind::ContextMention {
        context_type,
        sub_query,
    } = &projection.active_segment_kind
    else {
        return None;
    };
    let (source, query) = crate::spine::catalog_subsearch::parse_context_subsearch(
        context_type,
        sub_query.as_deref(),
    )?;
    query.trim().is_empty().then_some(source)
}

fn replace_segment_content(
    content: &str,
    line_range: Range<usize>,
    segment_byte_range: Range<usize>,
    replacement: &str,
    trailing_space: bool,
) -> Option<(String, usize)> {
    let start = line_range.start.checked_add(segment_byte_range.start)?;
    let end = line_range.start.checked_add(segment_byte_range.end)?;
    if start > end
        || end > content.len()
        || !content.is_char_boundary(start)
        || !content.is_char_boundary(end)
    {
        return None;
    }

    let suffix = &content[end..];
    let add_space = trailing_space
        && !replacement.ends_with(char::is_whitespace)
        && !suffix.starts_with(char::is_whitespace);
    let space = if add_space { " " } else { "" };
    let new_content = format!("{}{}{}{}", &content[..start], replacement, space, suffix);
    let cursor = start + replacement.len() + space.len();
    Some((new_content, cursor))
}

fn push_spine_sections_as_grouped(
    sections: Vec<crate::spine::SpineListSection>,
) -> DayPageSpineRows {
    let mut grouped = Vec::new();
    let mut flat = Vec::new();
    for section in sections {
        grouped.push(GroupedListItem::SectionHeader(
            section.title.to_string(),
            section.icon.as_ref().map(|icon| icon.as_ref().to_string()),
        ));
        for row in section.rows {
            if row.is_selectable && !day_page_supports_spine_action(&row.action) {
                continue;
            }
            if !row.is_selectable {
                let mut label = row.title.to_string();
                if let Some(subtitle) = row.subtitle.as_ref() {
                    if !subtitle.is_empty() {
                        label.push_str(" \u{b7} ");
                        label.push_str(subtitle.as_ref());
                    }
                }
                grouped.push(GroupedListItem::SectionHeader(
                    label,
                    row.icon.as_ref().map(|icon| icon.as_ref().to_string()),
                ));
                continue;
            }
            let flat_idx = flat.len();
            flat.push(scripts::SearchResult::SpineProjection(row));
            grouped.push(GroupedListItem::Item(flat_idx));
        }
    }
    DayPageSpineRows::new(grouped, flat)
}

fn day_page_supports_spine_action(action: &crate::spine::SpineListAction) -> bool {
    !matches!(
        action,
        crate::spine::SpineListAction::OpenFileSearchPortal { .. }
            | crate::spine::SpineListAction::Noop
    )
}


fn cwd_rows_from_directory_results(
    query: &str,
    projection: &crate::spine::SpineCursorProjection,
    parse: &crate::spine::SpineParse,
    grouped: Vec<GroupedListItem>,
    directory_flat: Vec<scripts::SearchResult>,
) -> DayPageSpineRows {
    let Some(segment) = parse.segments.get(projection.active_segment_index) else {
        return DayPageSpineRows::new(grouped, Vec::new());
    };

    let mut remapped_grouped = Vec::new();
    let mut flat = Vec::new();

    for grouped_item in grouped {
        match grouped_item {
            GroupedListItem::SectionHeader(label, icon) => {
                remapped_grouped.push(GroupedListItem::SectionHeader(label, icon));
            }
            GroupedListItem::Status(status) => {
                remapped_grouped.push(GroupedListItem::Status(status));
            }
            GroupedListItem::Item(result_idx) => {
                let Some(scripts::SearchResult::File(file_match)) = directory_flat.get(result_idx)
                else {
                    continue;
                };
                if file_match.file.file_type != crate::file_search::FileType::Directory {
                    continue;
                }
                let label = crate::file_search::shorten_path(&file_match.file.path)
                    .trim_end_matches('/')
                    .to_string();
                let replacement = format!(
                    ">:{}",
                    crate::spine::catalog_subsearch::escape_ref_component(&label)
                );
                let row_id = format!(
                    "spine:>:dir:{}",
                    crate::spine::catalog_subsearch::escape_ref_component(&label)
                );
                let row = crate::spine::SpineListRow {
                    id: row_id.into(),
                    kind: crate::spine::SpineListRowKind::Hint,
                    title: file_match.file.name.clone().into(),
                    subtitle: Some(label.clone().into()),
                    meta: Some(">:".into()),
                    icon: Some("folder".into()),
                    badges: Vec::new(),
                    score: 0,
                    is_selectable: true,
                    action_label: None,
                    action: crate::spine::SpineListAction::ResolveSegment {
                        segment_index: projection.active_segment_index,
                        segment_byte_range: segment.byte_range.clone(),
                        replacement: replacement.into(),
                        resolution_id: file_match.file.path.clone().into(),
                        resolution_label: label.into(),
                        resolution_source: "cwd".into(),
                        trailing_space: false,
                    },
                };
                let new_idx = flat.len();
                flat.push(scripts::SearchResult::SpineProjection(row));
                remapped_grouped.push(GroupedListItem::Item(new_idx));
            }
        }
    }

    if query.trim().is_empty() {
        filtering_cache::append_choose_hint_to_first_section_header(&mut remapped_grouped);
    }

    DayPageSpineRows::new(remapped_grouped, flat)
}


fn day_page_spine_prompt_plan_can_submit(
    parse: &crate::spine::SpineParse,
    cwd_anchor: bool,
    mention_aliases: &std::collections::HashMap<String, crate::ai::message_parts::AiContextPart>,
) -> bool {
    let plan = crate::spine::prompt_plan::build_spine_prompt_plan_with_aliases(
        parse,
        mention_aliases,
    );
    plan.should_submit_to_chat()
        || (cwd_anchor
            && matches!(
                plan.blocked_reason,
                Some(crate::spine::prompt_plan::SpinePromptPlanBlockReason::NoPromptBuilderSegments)
            )
            && plan.unknown_warnings.is_empty()
            && !plan.normalized_prompt.trim().is_empty())
}

fn day_page_spine_single_char_deletion_index(previous: &str, next: &str) -> Option<usize> {
    let previous_chars: Vec<char> = previous.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();
    if previous_chars.len() != next_chars.len() + 1 {
        return None;
    }
    let mut index = 0;
    while index < next_chars.len() && previous_chars[index] == next_chars[index] {
        index += 1;
    }
    (previous_chars[index + 1..] == next_chars[index..]).then_some(index)
}

fn byte_index_for_char_index(text: &str, char_index: usize) -> usize {
    if char_index == text.chars().count() {
        return text.len();
    }
    text.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

fn day_page_spine_mention_atomic_delete_fixup(
    previous: &str,
    next: &str,
    mention_aliases: &std::collections::HashMap<String, crate::ai::message_parts::AiContextPart>,
) -> Option<(String, usize)> {
    if mention_aliases.is_empty() {
        return None;
    }
    let deleted_char_index = day_page_spine_single_char_deletion_index(previous, next)?;
    let deleted_registered_token = crate::ai::context_mentions::inline_token_spans(previous)
        .into_iter()
        .any(|span| {
            deleted_char_index >= span.range.start
                && deleted_char_index < span.range.end
                && mention_aliases.contains_key(&span.token)
        });
    if !deleted_registered_token {
        return None;
    }
    let (fixed, cursor_char) =
        crate::ai::context_mentions::remove_inline_mention_at_cursor_with_aliases(
            previous,
            deleted_char_index + 1,
            false,
            mention_aliases,
        )?;
    let cursor = byte_index_for_char_index(&fixed, cursor_char);
    Some((fixed, cursor))
}

fn prune_day_page_spine_mention_aliases(
    mention_aliases: &mut std::collections::HashMap<
        String,
        crate::ai::message_parts::AiContextPart,
    >,
    content: &str,
) {
    if mention_aliases.is_empty() {
        return;
    }
    let visible_tokens = crate::ai::context_mentions::inline_token_spans(content)
        .into_iter()
        .map(|span| span.token)
        .collect::<std::collections::HashSet<_>>();
    mention_aliases.retain(|token, _| visible_tokens.contains(token));
}


impl DayPageView {
    fn shared_day_page_spine_rows(
        &self,
        parse: &crate::spine::SpineParse,
        projection: &crate::spine::SpineCursorProjection,
    ) -> DayPageSpineRows {
        let sections = crate::spine::list::build_spine_list_sections_full_with_resolved_tokens(
            parse,
            projection,
            None,
            &|token| self.spine_mention_aliases.contains_key(token),
        );
        push_spine_sections_as_grouped(sections)
    }

    fn render_day_page_spine_panel(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let model = self.day_page_spine_model(cx)?;
        let app = self.app.upgrade()?;
        let app_state = app.read(cx);
        let theme = app_state.theme.clone();
        let item_colors = crate::list_item::ListItemColors::from_theme(&theme);
        let main_menu_theme = app_state.current_main_menu_theme;
        let editor_surface =
            crate::components::notes_editor::NotesEditorSurfaceStyle::from_theme(&theme);

        let selected = if self.day_page_spine_empty_selection_suppressed(&model) {
            None
        } else {
            crate::list_item::coerce_selection(&model.grouped, self.spine_selected_index)
        };

        let mut rows = div().flex().flex_col().w_full();
        for (ix, grouped_item) in model.grouped.iter().enumerate() {
            match grouped_item {
                GroupedListItem::SectionHeader(label, icon) => {
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
                GroupedListItem::Status(status) => {
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
                            .text_color(rgb(item_colors.text_secondary))
                            .child(status.label.clone()),
                    );
                }
                GroupedListItem::Item(flat_idx) => {
                    let Some(scripts::SearchResult::SpineProjection(row)) =
                        model.flat.get(*flat_idx)
                    else {
                        continue;
                    };
                    let is_selected = selected == Some(ix);
                    let is_hovered = false;
                    let click_handler = cx.listener(
                        move |this: &mut DayPageView,
                              _event: &gpui::MouseDownEvent,
                              _window,
                              cx| {
                            this.spine_selected_index = ix;
                            if let Some(model) = this.day_page_spine_model(cx) {
                                if let Some(source) = model.active_empty_subsearch {
                                    this.spine_empty_subsearch_armed_for = Some(source);
                                }
                            }
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
                                    .hovered(is_hovered)
                                    .main_menu_theme(main_menu_theme)
                                    .semantic_id(row.id.to_string())
                                    .description_opt(row.subtitle.as_ref().map(|s| s.to_string()))
                                    .icon_kind_opt(spine_projection_icon_kind(row))
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
                .id("day-page-spine-list")
                .absolute()
                .inset_0()
                .bg(rgba(editor_surface.occlusion_rgba))
                .occlude()
                .overflow_y_scroll()
                .child(rows)
                .into_any_element(),
        )
    }

    fn day_page_spine_input(
        &self,
        cx: &App,
    ) -> Option<(
        String,
        Range<usize>,
        crate::spine::SpineParse,
        crate::spine::SpineCursorProjection,
        Option<crate::spine::catalog_subsearch::ContextSubsearchSource>,
    )> {
        let content = self.notes_editor.read(cx).content(cx);
        let selection = self.notes_editor.read(cx).selection(cx);
        let cursor = clamp_to_char_boundary(&content, selection.end.min(content.len()));
        let line_range = current_line_range(&content, cursor);
        let line = &content[line_range.clone()];
        let line_cursor = cursor.saturating_sub(line_range.start);
        let parse = crate::spine::parse_spine(line);
        let projection = crate::spine::project_cursor(&parse, line_cursor);
        if !spine_projection_owns_day_page_list(&parse, &projection) {
            return None;
        }

        let active_empty_subsearch = active_empty_day_page_subsearch(&projection);
        let key = format!(
            "{}\u{1f}cursor={}\u{1f}active={:?}\u{1f}cwd_rev={}",
            line, line_cursor, projection.active_segment_kind, self.spine_cwd_revision
        );
        Some((key, line_range, parse, projection, active_empty_subsearch))
    }

    fn day_page_spine_model(&mut self, cx: &App) -> Option<DayPageSpineModel> {
        let (key, line_range, parse, projection, active_empty_subsearch) =
            self.day_page_spine_input(cx)?;
        if self.spine_dismissed_cache_key.as_deref() == Some(key.as_str()) {
            return None;
        }

        if self.spine_cache_key == key {
            self.spine_selected_index = crate::list_item::coerce_selection(
                &self.spine_grouped_cache,
                self.spine_selected_index,
            )
            .unwrap_or(0);
            return Some(DayPageSpineModel {
                line_range,
                parse,
                projection,
                grouped: self.spine_grouped_cache.clone(),
                flat: self.spine_flat_cache.clone(),
                active_empty_subsearch,
            });
        }

        let rows = self.build_day_page_spine_rows(&parse, &projection, None, cx)?;
        let grouped = rows.grouped;
        let flat = rows.flat;

        self.spine_cache_key = key;
        self.spine_grouped_cache = grouped.clone();
        self.spine_flat_cache = flat.clone();
        self.spine_alias_cache = rows.aliases;
        self.spine_selected_index =
            crate::list_item::coerce_selection(&grouped, self.spine_selected_index).unwrap_or(0);

        Some(DayPageSpineModel {
            line_range,
            parse,
            projection,
            grouped,
            flat,
            active_empty_subsearch,
        })
    }

    fn build_day_page_spine_rows(
        &self,
        parse: &crate::spine::SpineParse,
        projection: &crate::spine::SpineCursorProjection,
        app_state: Option<&ScriptListApp>,
        cx: &App,
    ) -> Option<DayPageSpineRows> {
        if matches!(
            projection.active_segment_kind,
            crate::spine::SpineSegmentKind::ContextMention { .. }
        ) {
            // `@` mentions never render an inline Day Page selector — typing
            // into one swaps to the real main menu instead (the round trip in
            // day_page_round_trip.rs, triggered from on_editor_change), so
            // the selection UX is exactly the launcher's own.
            None
        } else if let crate::spine::SpineSegmentKind::ProjectCwd { sub_query } =
            &projection.active_segment_kind
        {
            let query = sub_query.as_deref().unwrap_or("").trim();
            let recent_dirs = if let Some(app_state) = app_state {
                app_state.recent_directory_results_from_frecency(
                    crate::spine::catalog_subsearch::SUBSEARCH_RENDER_LIMIT,
                )
            } else {
                let app = self.app.upgrade()?;
                app.read(cx).recent_directory_results_from_frecency(
                    crate::spine::catalog_subsearch::SUBSEARCH_RENDER_LIMIT,
                )
            };
            if recent_dirs.is_empty() {
                return Some(self.shared_day_page_spine_rows(parse, projection));
            }
            let (grouped, flat) = if query.is_empty() {
                filtering_cache::build_rich_cwd_root_rows(&recent_dirs)
            } else {
                filtering_cache::build_rich_cwd_subsearch_rows(query, &recent_dirs)
            };
            Some(cwd_rows_from_directory_results(
                query, projection, parse, grouped, flat,
            ))
        } else {
            Some(self.shared_day_page_spine_rows(parse, projection))
        }
    }

    pub(crate) fn collect_day_page_elements(
        &self,
        limit: usize,
        app_state: &ScriptListApp,
        cx: &App,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let content = self.notes_editor.read(cx).content(cx);
        let selection = self.notes_editor.read(cx).selection(cx);
        let mut editor =
            protocol::ElementInfo::input("day-page-editor", Some(content.as_str()), true);
        let metrics = crate::notes::window::style::adopted_metrics();
        let editor_surface =
            crate::components::notes_editor::NotesEditorSurfaceStyle::from_theme(&app_state.theme);
        editor.role = Some("day_page_editor".to_string());
        editor.kind = Some("editor_selection".to_string());
        editor.source = Some(format!("{}:{}", selection.start, selection.end));
        editor.source_name = Some(content.len().to_string());
        editor.style = Some(protocol::ElementStyleInfo {
            owner: editor_surface.owner.to_string(),
            input_render_path: Some(editor_surface.input_render_path.to_string()),
            surface_background_rgb: Some(editor_surface.background_rgb),
            occlusion_rgba: Some(editor_surface.occlusion_rgba),
            padding_x: Some(metrics.editor_padding_x),
            padding_y: Some(metrics.editor_padding_y),
            font_family_source: Some("theme.mono_font_family".to_string()),
            text_size_source: Some("theme.mono_font_size".to_string()),
        });
        let mut elements = vec![protocol::ElementInfo::panel("day-page"), editor];

        if let Some(state) = self.day_switcher.as_ref() {
            let today = Utc::now()
                .with_timezone(&self.session.substrate().timezone())
                .date_naive();
            let filtered = filtered_day_switcher_indices(state, today);
            elements.push(protocol::ElementInfo::list(
                DAY_SWITCHER_LIST_ID,
                filtered.len(),
            ));
            let selected_row = if filtered.is_empty() {
                None
            } else {
                Some(state.selected.min(filtered.len() - 1))
            };
            for (row_ix, entry_index) in filtered.iter().enumerate() {
                let Some(entry) = state.entries.get(*entry_index) else {
                    continue;
                };
                elements.push(protocol::ElementInfo {
                    semantic_id: day_switcher_semantic_id(entry.date),
                    element_type: protocol::ElementType::Choice,
                    text: Some(day_switcher_entry_label(entry.date, today)),
                    value: Some(entry.date.to_string()),
                    selected: Some(selected_row == Some(row_ix)),
                    focused: None,
                    index: Some(row_ix),
                    role: Some("day_page_day_switcher_row".to_string()),
                    kind: None,
                    source: Some(state.query.clone()),
                    source_name: None,
                    selectable: Some(true),
                    status_kind: None,
                    action_disabled: None,
                    style: None,
                });
            }
            let total_count = elements.len();
            return (elements.into_iter().take(limit).collect(), total_count);
        }

        let Some((key, _, parse, projection, active_empty_subsearch)) =
            self.day_page_spine_input(cx)
        else {
            let total_count = elements.len();
            return (elements.into_iter().take(limit).collect(), total_count);
        };
        if self.spine_dismissed_cache_key.as_deref() == Some(key.as_str()) {
            let total_count = elements.len();
            return (elements.into_iter().take(limit).collect(), total_count);
        }
        let rows = if self.spine_cache_key == key {
            Some(DayPageSpineRows {
                grouped: self.spine_grouped_cache.clone(),
                flat: self.spine_flat_cache.clone(),
                aliases: self.spine_alias_cache.clone(),
            })
        } else {
            self.build_day_page_spine_rows(&parse, &projection, Some(app_state), cx)
        };
        let Some(rows) = rows else {
            let total_count = elements.len();
            return (elements.into_iter().take(limit).collect(), total_count);
        };
        let grouped = rows.grouped;
        let flat = rows.flat;

        let item_count = grouped
            .iter()
            .filter(|item| matches!(item, GroupedListItem::Item(_)))
            .count();
        elements.push(protocol::ElementInfo::list(
            "day-page-spine-list",
            item_count,
        ));

        let selection_suppressed = active_empty_subsearch
            .is_some_and(|source| self.spine_empty_subsearch_armed_for != Some(source));
        let selected = if selection_suppressed {
            None
        } else {
            crate::list_item::coerce_selection(&grouped, self.spine_selected_index)
        };
        for (ix, grouped_item) in grouped.iter().enumerate() {
            let GroupedListItem::Item(flat_idx) = grouped_item else {
                continue;
            };
            let Some(scripts::SearchResult::SpineProjection(row)) = flat.get(*flat_idx) else {
                continue;
            };
            elements.push(protocol::ElementInfo {
                semantic_id: row.id.to_string(),
                element_type: protocol::ElementType::Choice,
                text: Some(row.title.to_string()),
                value: Some(row.id.to_string()),
                selected: Some(selected == Some(ix)),
                focused: None,
                index: Some(ix),
                role: Some("day_page_spine_row".to_string()),
                kind: Some(row.kind.type_accessory_info().0.to_string()),
                source: row.meta.as_ref().map(|meta| meta.to_string()),
                source_name: None,
                selectable: Some(row.is_selectable),
                status_kind: None,
                action_disabled: None,
                style: None,
            });
        }

        let total_count = elements.len();
        (elements.into_iter().take(limit).collect(), total_count)
    }

    fn day_page_spine_empty_selection_suppressed(&self, model: &DayPageSpineModel) -> bool {
        match model.active_empty_subsearch {
            Some(source) => self.spine_empty_subsearch_armed_for != Some(source),
            None => false,
        }
    }

    fn selected_day_page_spine_row(
        &self,
        model: &DayPageSpineModel,
    ) -> Option<crate::spine::SpineListRow> {
        if self.day_page_spine_empty_selection_suppressed(model) {
            return None;
        }
        model.selected_row(self.spine_selected_index)
    }

    fn move_day_page_spine_selection(&mut self, direction: isize, cx: &mut Context<Self>) -> bool {
        let Some(model) = self.day_page_spine_model(cx) else {
            return false;
        };

        if direction > 0 {
            if let Some(source) = model.active_empty_subsearch {
                if self.spine_empty_subsearch_armed_for != Some(source) {
                    self.spine_empty_subsearch_armed_for = Some(source);
                    if let Some(index) = crate::list_item::coerce_selection(&model.grouped, 0) {
                        self.spine_selected_index = index;
                    }
                    cx.notify();
                    return true;
                }
            }
        }

        let len = model.grouped.len();
        if len == 0 {
            return false;
        }
        let mut next = self.spine_selected_index.min(len - 1);
        loop {
            next = if direction < 0 {
                next.saturating_sub(1)
            } else {
                (next + 1).min(len - 1)
            };
            if matches!(model.grouped.get(next), Some(GroupedListItem::Item(_))) {
                self.spine_selected_index = next;
                cx.notify();
                return true;
            }
            if next == 0 || next == len - 1 {
                return true;
            }
        }
    }

    fn reset_day_page_spine_navigation(&mut self, cx: &mut Context<Self>) {
        if let Some((key, _, _, _, _)) = self.day_page_spine_input(cx) {
            self.spine_dismissed_cache_key = Some(key);
        }
        self.spine_selected_index = 0;
        self.spine_hovered_index = None;
        self.spine_empty_subsearch_armed_for = None;
        cx.notify();
    }

    fn replace_day_page_spine_segment(
        &mut self,
        model: &DayPageSpineModel,
        segment_index: usize,
        segment_byte_range: Range<usize>,
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
        let Some((new_content, cursor)) = replace_segment_content(
            &content,
            model.line_range.clone(),
            segment_byte_range,
            replacement,
            trailing_space,
        ) else {
            return false;
        };

        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value(new_content.clone(), window, cx);
            editor.set_selection(cursor, cursor, window, cx);
        });
        self.session.apply_editor_content(&new_content);
        self.refresh_fragment_open_targets(&new_content);
        self.spine_selected_index = 0;
        self.spine_empty_subsearch_armed_for = None;
        self.spine_alias_cache.clear();
        self.sync_footer(window, cx);
        cx.notify();
        true
    }

    fn apply_day_page_spine_action(
        &mut self,
        action: crate::spine::SpineListAction,
        model: &DayPageSpineModel,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match action {
            crate::spine::SpineListAction::InsertSegmentText {
                segment_index,
                segment_byte_range,
                text,
                trailing_space,
            } => self.replace_day_page_spine_segment(
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
                resolution_id,
                resolution_label,
                resolution_source,
                trailing_space,
            } => {
                if resolution_source.as_ref() == "cwd" {
                    let applied = self.replace_day_page_spine_segment(
                        model,
                        segment_index,
                        segment_byte_range,
                        "",
                        false,
                        window,
                        cx,
                    );
                    if applied {
                        self.spine_cwd_revision = self.spine_cwd_revision.wrapping_add(1);
                        self.spine_cwd_submit_anchor = true;
                        if let Some(app) = self.app.upgrade() {
                            let app_resolution_id = resolution_id.to_string();
                            let app_resolution_label = resolution_label.to_string();
                            window.defer(cx, move |_window, cx| {
                                app.update(cx, |app, cx| {
                                    let path = std::path::PathBuf::from(&app_resolution_id);
                                    app.spine_cwd = Some(path);
                                    app.spine_cwd_label = Some(app_resolution_label);
                                    app.spine_cwd_revision = app.spine_cwd_revision.wrapping_add(1);
                                    app.persist_spine_cwd();
                                    app.prewarm_agent_chat_for_spine_cwd(cx);
                                    app.invalidate_grouped_cache();
                                    cx.notify();
                                });
                            });
                        }
                    }
                    return applied;
                }

                if let Some(app) = self.app.upgrade() {
                    let app_resolution_source = resolution_source.to_string();
                    let app_resolution_id = resolution_id.to_string();
                    let app_resolution_label = resolution_label.to_string();
                    let app_replacement = replacement.to_string();
                    window.defer(cx, move |_window, cx| {
                        app.update(cx, |app, _cx| {
                            if app_resolution_source.as_str() == "file" {
                                if let Some(path) = app_resolution_id.strip_prefix("file/") {
                                    app.register_spine_file_mention_alias(
                                        app_replacement.clone(),
                                        path.to_string(),
                                    );
                                }
                            } else if app_resolution_source.as_str() == "clipboard" {
                                if let Some(id) = app_resolution_id.strip_prefix("clipboard/") {
                                    app.register_spine_clipboard_mention_alias(
                                        app_replacement.clone(),
                                        id.to_string(),
                                        app_resolution_label.clone(),
                                    );
                                }
                            }
                        });
                    });
                }
                self.replace_day_page_spine_segment(
                    model,
                    segment_index,
                    segment_byte_range,
                    replacement.as_ref(),
                    trailing_space,
                    window,
                    cx,
                )
            }
            crate::spine::SpineListAction::OpenModeExit { sigil, rest } => {
                let Some(app) = self.app.upgrade() else {
                    return false;
                };
                app.update(cx, |app, cx| match sigil {
                    '~' => {
                        app.open_file_search_view(
                            rest.to_string(),
                            FileSearchPresentation::Mini,
                            cx,
                        );
                        true
                    }
                    '!' => {
                        app.open_quick_terminal(None, cx);
                        true
                    }
                    '?' => {
                        if app.has_actions() {
                            app.toggle_actions(cx, window);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
            }
            crate::spine::SpineListAction::OpenConversation { conversation_id } => {
                let Some(app) = self.app.upgrade() else {
                    return false;
                };
                app.update(cx, |app, cx| {
                    app.resume_agent_chat_conversation_from_history(
                        conversation_id.as_ref(),
                        "",
                        cx,
                    );
                    true
                })
            }
            crate::spine::SpineListAction::SubmitPromptPlan => {
                let Some(app) = self.app.upgrade() else {
                    return false;
                };
                let parse = model.parse.clone();
                let mention_aliases = self.spine_mention_aliases.clone();
                window.defer(cx, move |_window, cx| {
                    app.update(cx, |app, cx| {
                        app.submit_day_page_spine_prompt_plan_with_aliases(
                            parse,
                            mention_aliases,
                            cx,
                        );
                    });
                });
                true
            }
            crate::spine::SpineListAction::OpenFileSearchPortal { .. }
            | crate::spine::SpineListAction::Noop => false,
        }
    }

    fn submit_day_page_spine_prompt_from_current_line(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let content = self.notes_editor.read(cx).content(cx);
        let selection = self.notes_editor.read(cx).selection(cx);
        let cursor = clamp_to_char_boundary(&content, selection.end.min(content.len()));
        let line_range = current_line_range(&content, cursor);
        let line = &content[line_range];
        if line.trim().is_empty() {
            return false;
        }

        let parse = crate::spine::parse_spine(line);
        if !day_page_spine_prompt_plan_can_submit(
            &parse,
            self.spine_cwd_submit_anchor,
            &self.spine_mention_aliases,
        ) {
            return false;
        }

        let Some(app) = self.app.upgrade() else {
            return false;
        };
        let mention_aliases = self.spine_mention_aliases.clone();

        window.defer(cx, move |_window, cx| {
            app.update(cx, |app, cx| {
                app.submit_day_page_spine_prompt_plan_with_aliases(parse, mention_aliases, cx);
            });
        });
        true
    }

    fn accept_day_page_spine_selection(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(model) = self.day_page_spine_model(cx) else {
            return false;
        };
        let Some(row) = self.selected_day_page_spine_row(&model) else {
            return false;
        };
        if let Some((token, part)) = self.spine_alias_cache.get(row.id.as_ref()).cloned() {
            self.spine_mention_aliases.insert(token.clone(), part.clone());
            if let Some(app) = self.app.upgrade() {
                window.defer(cx, move |_window, cx| {
                    app.update(cx, |app, _cx| {
                        tracing::info!(
                            target: "script_kit::spine",
                            event = "day_page_spine_subsearch_alias_registered",
                            token = %token,
                        );
                        app.spine_mention_aliases.insert(token, part);
                    });
                });
            }
        }
        let row_id = row.id.to_string();
        let handled = self.apply_day_page_spine_action(row.action, &model, window, cx);
        if !handled {
            tracing::warn!(
                target: "script_kit::spine",
                event = "day_page_spine_action_unhandled",
                row_id = %row_id,
            );
        }
        handled
    }
}

#[cfg(test)]
mod day_page_spine_tests {
    use super::*;

    #[test]
    fn current_line_parser_ignores_prior_captured_mentions() {
        let content = "captured @file:old\nnew @file:read";
        let cursor = content.len();
        let range = current_line_range(content, cursor);
        assert_eq!(&content[range], "new @file:read");
    }

    #[test]
    fn current_line_parser_targets_non_final_active_line() {
        let content = "first @file:old\nmiddle @file:read\nlast @clipboard:tail";
        let cursor = content.find("read").expect("active query exists") + "re".len();
        let range = current_line_range(content, cursor);
        assert_eq!(&content[range], "middle @file:read");
    }

    #[test]
    fn current_line_parser_clamps_unicode_cursor_to_char_boundary() {
        let content = "emoji é @file:read\nnext";
        let cursor_inside_e_acute = "emoji ".len() + 1;
        let range = current_line_range(content, cursor_inside_e_acute);
        assert_eq!(&content[range], "emoji é @file:read");
    }

    #[test]
    fn current_line_parser_handles_blank_active_line() {
        let content = "above\n\nbelow @file:read";
        let cursor = "above\n".len();
        let range = current_line_range(content, cursor);
        assert_eq!(&content[range], "");
    }

    #[test]
    fn replace_segment_content_preserves_surrounding_lines() {
        let content = "captured @file:old\nnew @fi\nnext line";
        let line_start = content.find("new ").expect("line exists");
        let line_range = line_start.."captured @file:old\nnew @fi".len();
        let segment_start = "new ".len();
        let segment_end = segment_start + "@fi".len();
        let (new_content, cursor) = replace_segment_content(
            content,
            line_range,
            segment_start..segment_end,
            "@file:",
            false,
        )
        .expect("replacement should fit current line");

        assert_eq!(new_content, "captured @file:old\nnew @file:\nnext line");
        assert_eq!(cursor, "captured @file:old\nnew @file:".len());
    }

    #[test]
    fn replace_segment_content_adds_trailing_space_when_needed() {
        let content = "ask @fi";
        let line_range = 0..content.len();
        let segment_start = "ask ".len();
        let segment_end = segment_start + "@fi".len();
        let (new_content, cursor) = replace_segment_content(
            content,
            line_range,
            segment_start..segment_end,
            "@file:",
            true,
        )
        .expect("replacement should fit");

        assert_eq!(new_content, "ask @file: ");
        assert_eq!(cursor, "ask @file: ".len());
    }

    fn test_text_block_part(label: &str) -> crate::ai::message_parts::AiContextPart {
        crate::ai::message_parts::AiContextPart::TextBlock {
            label: label.to_string(),
            source: format!("test:{label}"),
            text: format!("{label} body"),
            mime_type: None,
        }
    }

    #[test]
    fn alias_backed_token_deletes_atomically_and_consumes_space() {
        let mut aliases = std::collections::HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );

        let fixed = day_page_spine_mention_atomic_delete_fixup(
            "ask @clipboard:Latest now",
            "ask @clipboard:Lates now",
            &aliases,
        )
        .expect("registered token should delete atomically");

        assert_eq!(fixed, ("ask now".to_string(), "ask ".len()));
    }

    #[test]
    fn unresolved_subsearch_token_keeps_normal_character_delete() {
        let mut aliases = std::collections::HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );

        assert_eq!(
            day_page_spine_mention_atomic_delete_fixup(
                "ask @file:readme now",
                "ask @file:readm now",
                &aliases,
            ),
            None
        );
    }

    #[test]
    fn prune_aliases_drops_tokens_no_longer_visible() {
        let mut aliases = std::collections::HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );
        aliases.insert("@file:demo.rs".to_string(), test_text_block_part("demo.rs"));

        prune_day_page_spine_mention_aliases(&mut aliases, "ask @file:demo.rs");

        assert!(!aliases.contains_key("@clipboard:Latest"));
        assert!(aliases.contains_key("@file:demo.rs"));
    }

    #[test]
    fn set_input_prune_boundary_uses_inline_token_spans_not_substrings() {
        let mut aliases = std::collections::HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            test_text_block_part("Latest"),
        );

        prune_day_page_spine_mention_aliases(&mut aliases, "literal @clipboard:Latest-ish");

        assert!(aliases.is_empty());
    }

    #[test]
    fn day_page_spine_owns_context_fragment_on_active_line() {
        let line = "@file:readme";
        let parse = crate::spine::parse_spine(line);
        let projection = crate::spine::project_cursor(&parse, line.len());
        assert!(spine_projection_owns_day_page_list(&parse, &projection));
    }

    #[test]
    fn day_page_projection_keeps_submit_and_filters_dead_end_actions() {
        let rows = push_spine_sections_as_grouped(vec![crate::spine::SpineListSection {
            id: "test-section".into(),
            title: "Test".into(),
            subtitle: None,
            icon: None,
            rows: vec![
                crate::spine::SpineListRow {
                    id: "spine:tail:ready".into(),
                    kind: crate::spine::SpineListRowKind::Hint,
                    title: "Ready to send".into(),
                    subtitle: None,
                    meta: None,
                    icon: None,
                    badges: vec![],
                    score: 0,
                    is_selectable: true,
                    action_label: None,
                    action: crate::spine::SpineListAction::SubmitPromptPlan,
                },
                crate::spine::SpineListRow {
                    id: "spine:@file:portal".into(),
                    kind: crate::spine::SpineListRowKind::Hint,
                    title: "Browse files".into(),
                    subtitle: None,
                    meta: None,
                    icon: None,
                    badges: vec![],
                    score: 0,
                    is_selectable: true,
                    action_label: None,
                    action: crate::spine::SpineListAction::OpenFileSearchPortal {
                        segment_index: 0,
                        segment_byte_range: 0..6,
                        query: "".into(),
                    },
                },
                crate::spine::SpineListRow {
                    id: "spine:@:subsearch:file".into(),
                    kind: crate::spine::SpineListRowKind::ContextSubSearch {
                        context_type: "file".into(),
                    },
                    title: "Files".into(),
                    subtitle: None,
                    meta: None,
                    icon: None,
                    badges: vec![],
                    score: 0,
                    is_selectable: true,
                    action_label: None,
                    action: crate::spine::SpineListAction::InsertSegmentText {
                        segment_index: 0,
                        segment_byte_range: 0..3,
                        text: "@file:".into(),
                        trailing_space: false,
                    },
                },
            ],
        }]);

        assert_eq!(rows.flat.len(), 2);
        let scripts::SearchResult::SpineProjection(row) = &rows.flat[0] else {
            panic!("expected projected spine row");
        };
        assert_eq!(row.id.as_ref(), "spine:tail:ready");
        let scripts::SearchResult::SpineProjection(row) = &rows.flat[1] else {
            panic!("expected projected spine row");
        };
        assert_eq!(row.id.as_ref(), "spine:@:subsearch:file");
    }

    #[test]
    fn selected_row_coerces_section_header_index_to_first_item() {
        let rows = push_spine_sections_as_grouped(vec![crate::spine::SpineListSection {
            id: "test-section".into(),
            title: "Test".into(),
            subtitle: None,
            icon: None,
            rows: vec![crate::spine::SpineListRow {
                id: "spine:@:subsearch:file".into(),
                kind: crate::spine::SpineListRowKind::ContextSubSearch {
                    context_type: "file".into(),
                },
                title: "Files".into(),
                subtitle: None,
                meta: None,
                icon: None,
                badges: vec![],
                score: 0,
                is_selectable: true,
                action_label: None,
                action: crate::spine::SpineListAction::InsertSegmentText {
                    segment_index: 0,
                    segment_byte_range: 0..3,
                    text: "@file:".into(),
                    trailing_space: false,
                },
            }],
        }]);
        let parse = crate::spine::parse_spine("@fi");
        let projection = crate::spine::project_cursor(&parse, "@fi".len());
        let model = DayPageSpineModel {
            line_range: 0..3,
            parse,
            projection,
            grouped: rows.grouped,
            flat: rows.flat,
            active_empty_subsearch: None,
        };

        let row = model
            .selected_row(0)
            .expect("section-header selection should coerce to first item");
        assert_eq!(row.id.as_ref(), "spine:@:subsearch:file");
    }

    #[test]
    fn cwd_directory_results_remap_to_spine_projection_rows() {
        let parse = crate::spine::parse_spine(">dev");
        let projection = crate::spine::project_cursor(&parse, ">dev".len());
        let directory = crate::file_search::FileResult {
            path: "/Users/test/dev".to_string(),
            name: "dev".to_string(),
            size: 0,
            modified: 0,
            file_type: crate::file_search::FileType::Directory,
        };
        let grouped = vec![
            GroupedListItem::SectionHeader("Directories".to_string(), Some("folder".to_string())),
            GroupedListItem::Item(0),
        ];
        let flat = vec![scripts::SearchResult::File(scripts::FileMatch {
            file: directory,
            score: 0,
        })];

        let rows = cwd_rows_from_directory_results("dev", &projection, &parse, grouped, flat);

        assert_eq!(rows.flat.len(), 1);
        let scripts::SearchResult::SpineProjection(row) = &rows.flat[0] else {
            panic!("cwd rows should be remapped to spine projections");
        };
        assert_eq!(row.id.as_ref(), "spine:>:dir:/Users/test/dev");
        let crate::spine::SpineListAction::ResolveSegment {
            resolution_id,
            resolution_label,
            resolution_source,
            ..
        } = &row.action
        else {
            panic!("cwd row should resolve the active segment");
        };
        assert_eq!(resolution_id.as_ref(), "/Users/test/dev");
        assert_eq!(resolution_label.as_ref(), "/Users/test/dev");
        assert_eq!(resolution_source.as_ref(), "cwd");
    }

    #[test]
    fn cmd_enter_preflight_rejects_plain_text_without_cwd_anchor() {
        let parse = crate::spine::parse_spine("summarize this folder");
        let aliases = std::collections::HashMap::new();

        assert!(!day_page_spine_prompt_plan_can_submit(
            &parse, false, &aliases
        ));
    }

    #[test]
    fn cmd_enter_preflight_allows_plain_text_with_cwd_anchor() {
        let parse = crate::spine::parse_spine("summarize this folder");
        let aliases = std::collections::HashMap::new();

        assert!(day_page_spine_prompt_plan_can_submit(&parse, true, &aliases));
    }

    #[test]
    fn cmd_enter_preflight_allows_prompt_builder_plan_without_cwd_anchor() {
        let parse = crate::spine::parse_spine("@selection summarize this");
        let aliases = std::collections::HashMap::new();

        assert!(day_page_spine_prompt_plan_can_submit(
            &parse, false, &aliases
        ));
    }

    #[test]
    fn cmd_enter_preflight_rejects_partial_context_even_with_cwd_anchor() {
        let parse = crate::spine::parse_spine("@clip");
        let aliases = std::collections::HashMap::new();

        assert!(!day_page_spine_prompt_plan_can_submit(
            &parse, true, &aliases
        ));
    }

    #[test]
    fn cmd_enter_preflight_allows_alias_backed_context() {
        let parse = crate::spine::parse_spine("@clipboard:Latest");
        let mut aliases = std::collections::HashMap::new();
        aliases.insert(
            "@clipboard:Latest".to_string(),
            crate::ai::message_parts::AiContextPart::TextBlock {
                label: "Latest".to_string(),
                source: "clipboard/test".to_string(),
                text: "Copied context".to_string(),
                mime_type: None,
            },
        );

        assert!(day_page_spine_prompt_plan_can_submit(
            &parse, false, &aliases
        ));
    }
}
