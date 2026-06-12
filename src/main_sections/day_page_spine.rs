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

fn attach_rows_from_rich_results(
    source: crate::spine::catalog_subsearch::ContextSubsearchSource,
    query: &str,
    projection: &crate::spine::SpineCursorProjection,
    parse: &crate::spine::SpineParse,
    grouped: Vec<GroupedListItem>,
    rich_flat: Vec<scripts::SearchResult>,
) -> DayPageSpineRows {
    let Some(segment) = parse.segments.get(projection.active_segment_index) else {
        return DayPageSpineRows::new(grouped, Vec::new());
    };

    let mut remapped_grouped = Vec::new();
    let mut flat = Vec::new();
    let mut aliases = HashMap::new();

    for grouped_item in grouped {
        match grouped_item {
            GroupedListItem::SectionHeader(label, icon) => {
                remapped_grouped.push(GroupedListItem::SectionHeader(label, icon));
            }
            GroupedListItem::Status(status) => {
                remapped_grouped.push(GroupedListItem::Status(status));
            }
            GroupedListItem::Item(result_idx) => {
                let Some(result) = rich_flat.get(result_idx) else {
                    continue;
                };
                let Some(outcome) = crate::spine::attach::attach_outcome_for_result(
                    source,
                    &result,
                    projection.active_segment_index,
                    segment.byte_range.clone(),
                ) else {
                    continue;
                };
                let row_id = format!("day-page-spine:{}:{}", source.prefix(), flat.len());
                let row = crate::spine::SpineListRow {
                    id: row_id.clone().into(),
                    kind: crate::spine::SpineListRowKind::ContextResult {
                        context_type: source.prefix().into(),
                        result_id: flat.len().to_string().into(),
                    },
                    title: result.name().to_string().into(),
                    subtitle: result
                        .description()
                        .map(|description| description.to_string().into()),
                    meta: Some(format!("@{}:", source.prefix()).into()),
                    icon: Some(source_icon(source).into()),
                    badges: Vec::new(),
                    score: 0,
                    is_selectable: true,
                    action_label: None,
                    action: outcome.action,
                };
                if let Some(alias) = outcome.alias {
                    aliases.insert(row_id, alias);
                }
                let new_idx = flat.len();
                flat.push(scripts::SearchResult::SpineProjection(row));
                remapped_grouped.push(GroupedListItem::Item(new_idx));
            }
        }
    }

    if query.trim().is_empty() {
        filtering_cache::append_choose_hint_to_first_section_header(&mut remapped_grouped);
    }

    DayPageSpineRows {
        grouped: remapped_grouped,
        flat,
        aliases,
    }
}

fn source_icon(source: crate::spine::catalog_subsearch::ContextSubsearchSource) -> &'static str {
    match source {
        crate::spine::catalog_subsearch::ContextSubsearchSource::File => "file",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Project => "folder",
        crate::spine::catalog_subsearch::ContextSubsearchSource::BrowserHistory => "globe",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Clipboard => "clipboard",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Dictation => "mic",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Scripts => "file-code",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Scriptlets => "scroll-text",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Skills => "workflow",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Notes => "notebook-text",
        crate::spine::catalog_subsearch::ContextSubsearchSource::History => "message-circle",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Calendar => "calendar",
        crate::spine::catalog_subsearch::ContextSubsearchSource::Notifications => "bell",
    }
}

impl ScriptListApp {
    fn day_page_rich_spine_rows(
        &self,
        source: crate::spine::catalog_subsearch::ContextSubsearchSource,
        query: &str,
        parse: &crate::spine::SpineParse,
        projection: &crate::spine::SpineCursorProjection,
    ) -> DayPageSpineRows {
        let limit = crate::spine::catalog_subsearch::SUBSEARCH_RENDER_LIMIT;
        let query = query.trim();
        let (grouped, flat) = match source {
            crate::spine::catalog_subsearch::ContextSubsearchSource::File => {
                let provider_query = if query.is_empty() {
                    crate::file_search::RECENTLY_USED_FILES_MDQUERY.to_string()
                } else {
                    crate::file_search::root_file_provider_query_for_user_query(query)
                };
                let files = crate::file_search::search_files(&provider_query, None, limit);
                filtering_cache::build_rich_file_subsearch_rows(
                    filtering_cache::FileSubsearchFlavor::Global,
                    query,
                    false,
                    &files,
                    &[],
                )
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Project => {
                let onlyin = self
                    .spine_cwd
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string())
                    .or_else(|| dirs::home_dir().map(|path| path.to_string_lossy().to_string()));
                let provider_query = if query.is_empty() {
                    crate::file_search::RECENTLY_USED_FILES_MDQUERY.to_string()
                } else {
                    query.to_string()
                };
                let files =
                    crate::file_search::search_files(&provider_query, onlyin.as_deref(), limit);
                filtering_cache::build_rich_file_subsearch_rows(
                    filtering_cache::FileSubsearchFlavor::Project,
                    query,
                    false,
                    &files,
                    &[],
                )
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Clipboard => {
                let hits = crate::clipboard_history::search_root_clipboard_history_meta_direct(
                    query,
                    crate::clipboard_history::RootClipboardHistorySectionOptions {
                        enabled: true,
                        max_results: limit,
                        min_query_chars: 0,
                        ..Default::default()
                    },
                );
                filtering_cache::build_rich_clipboard_subsearch_rows(query, &hits)
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::BrowserHistory => {
                let hits = crate::browser_history::search_root_browser_history_meta_direct(
                    query,
                    crate::browser_history::RootBrowserHistorySectionOptions {
                        enabled: true,
                        max_results: limit,
                        min_query_chars: 0,
                        ..Default::default()
                    },
                );
                filtering_cache::build_rich_browser_history_rows(query, &hits)
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Notes => {
                let hits = crate::notes::search_root_notes_meta_direct(
                    query,
                    crate::notes::RootNotesSectionOptions {
                        enabled: true,
                        max_results: limit,
                        min_query_chars: 0,
                        ..Default::default()
                    },
                );
                filtering_cache::build_rich_notes_rows(query, &hits)
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Dictation => {
                let hits = crate::dictation::search_root_dictation_history_direct(
                    query,
                    crate::dictation::RootDictationHistorySectionOptions {
                        enabled: true,
                        max_results: limit,
                        min_query_chars: 0,
                        ..Default::default()
                    },
                );
                filtering_cache::build_rich_dictation_rows(query, &hits)
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::History => {
                let hits = crate::ai::agent_chat::ui::history::search_history_direct(query, limit);
                filtering_cache::build_rich_agent_chat_history_rows(query, &hits)
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Scripts => {
                filtering_cache::build_rich_script_rows(query, &self.scripts)
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Scriptlets => {
                filtering_cache::build_rich_scriptlet_rows(query, &self.scriptlets)
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Skills => {
                filtering_cache::build_rich_skill_rows(query, &self.skills)
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Calendar => {
                filtering_cache::build_rich_provider_json_rows(
                    query,
                    crate::mcp_resources::ProviderJsonResourceKind::Calendar,
                    "Calendar Events",
                    "calendar",
                )
            }
            crate::spine::catalog_subsearch::ContextSubsearchSource::Notifications => {
                filtering_cache::build_rich_provider_json_rows(
                    query,
                    crate::mcp_resources::ProviderJsonResourceKind::Notifications,
                    "Notifications",
                    "bell",
                )
            }
        };

        attach_rows_from_rich_results(source, query, projection, parse, grouped, flat)
    }
}

impl DayPageView {
    fn render_day_page_spine_panel(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let model = self.day_page_spine_model(cx)?;
        let app = self.app.upgrade()?;
        let app_state = app.read(cx);
        let theme = app_state.theme.clone();
        let item_colors = crate::list_item::ListItemColors::from_theme(&theme);
        let main_menu_theme = app_state.current_main_menu_theme;
        let editor_bg = theme.colors.background.main;

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
                .bg(rgba((editor_bg << 8) | 0xFF))
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
            "{}\u{1f}cursor={}\u{1f}active={:?}",
            line, line_cursor, projection.active_segment_kind
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
        if let Some((source, query)) = filtering_cache::active_rich_spine_subsearch(projection) {
            if let Some(app_state) = app_state {
                Some(app_state.day_page_rich_spine_rows(source, &query, parse, projection))
            } else {
                let app = self.app.upgrade()?;
                Some(
                    app.read(cx)
                        .day_page_rich_spine_rows(source, &query, parse, projection),
                )
            }
        } else if let crate::spine::SpineSegmentKind::ProjectCwd { sub_query } =
            &projection.active_segment_kind
        {
            let query = sub_query.as_deref().unwrap_or("").trim();
            let files = crate::file_search::search_files(
                if query.is_empty() {
                    crate::file_search::RECENTLY_USED_FILES_MDQUERY
                } else {
                    query
                },
                dirs::home_dir()
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string())
                    .as_deref(),
                crate::spine::catalog_subsearch::SUBSEARCH_RENDER_LIMIT,
            );
            let (grouped, flat) = if query.is_empty() {
                filtering_cache::build_rich_cwd_root_rows(&files)
            } else {
                filtering_cache::build_rich_cwd_subsearch_rows(query, &files)
            };
            Some(DayPageSpineRows::new(grouped, flat))
        } else {
            let sections = crate::spine::list::build_spine_list_sections(parse, projection);
            Some(push_spine_sections_as_grouped(sections))
        }
    }

    pub(crate) fn collect_day_page_elements(
        &self,
        limit: usize,
        app_state: &ScriptListApp,
        cx: &App,
    ) -> (Vec<protocol::ElementInfo>, usize) {
        let content = self.notes_editor.read(cx).content(cx);
        let mut elements = vec![
            protocol::ElementInfo::panel("day-page"),
            protocol::ElementInfo::input("day-page-editor", Some(content.as_str()), true),
        ];

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
                if let Some(app) = self.app.upgrade() {
                    app.update(cx, |app, cx| {
                        if resolution_source.as_ref() == "file" {
                            if let Some(path) = resolution_id.as_ref().strip_prefix("file/") {
                                app.register_spine_file_mention_alias(
                                    replacement.as_ref().to_string(),
                                    path.to_string(),
                                );
                            }
                        } else if resolution_source.as_ref() == "clipboard" {
                            if let Some(id) = resolution_id.as_ref().strip_prefix("clipboard/") {
                                app.register_spine_clipboard_mention_alias(
                                    replacement.as_ref().to_string(),
                                    id.to_string(),
                                    resolution_label.as_ref().to_string(),
                                );
                            }
                        } else if resolution_source.as_ref() == "cwd" {
                            let path = std::path::PathBuf::from(resolution_id.as_ref());
                            let label = resolution_label.as_ref().to_string();
                            app.spine_cwd = Some(path);
                            app.spine_cwd_label = Some(label);
                            app.spine_cwd_revision = app.spine_cwd_revision.wrapping_add(1);
                            app.persist_spine_cwd();
                            app.prewarm_agent_chat_for_spine_cwd(cx);
                            app.invalidate_grouped_cache();
                        }
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
                window.defer(cx, move |_window, cx| {
                    app.update(cx, |app, cx| {
                        app.submit_day_page_spine_prompt_plan(parse, cx);
                    });
                });
                true
            }
            crate::spine::SpineListAction::OpenFileSearchPortal { .. }
            | crate::spine::SpineListAction::Noop => false,
        }
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
            if let Some(app) = self.app.upgrade() {
                app.update(cx, |app, _cx| {
                    tracing::info!(
                        target: "script_kit::spine",
                        event = "day_page_spine_subsearch_alias_registered",
                        token = %token,
                    );
                    app.spine_mention_aliases.insert(token, part);
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
}
