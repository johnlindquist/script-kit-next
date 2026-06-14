// Day Page Agent Chat handoff helpers.

use crate::components::notes_editor::spine::{
    clamp_to_char_boundary, current_line_range, mention_atomic_delete_fixup,
};

impl DayPageView {
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
            editor_runtime: Some(
                self.notes_editor
                    .read(cx)
                    .markdown_runtime_info_with_scroll(cx),
            ),
            surface_background_rgb: Some(editor_surface.background_rgb),
            occlusion_rgba: Some(editor_surface.occlusion_rgba),
            padding_x: Some(metrics.editor_padding_x),
            padding_y: Some(metrics.editor_padding_y),
            font_family_source: Some("theme.mono_font_family".to_string()),
            text_size_source: Some("theme.mono_font_size".to_string()),
        });
        let mut elements = vec![protocol::ElementInfo::panel("day-page"), editor];

        if self.session.is_viewing_fragment() {
            elements.push(protocol::ElementInfo {
                semantic_id: script_kit_gpui::day_page::FRAGMENT_BACK_ID.to_string(),
                element_type: protocol::ElementType::Button,
                text: Some("Back to Today".to_string()),
                value: Some("day_page:back_to_today".to_string()),
                selected: None,
                focused: None,
                index: None,
                role: Some("day_page_fragment_back".to_string()),
                kind: Some("FragmentBack".to_string()),
                source: None,
                source_name: None,
                selectable: Some(true),
                status_kind: None,
                action_disabled: None,
                style: None,
            });
        }

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

        let total_count = elements.len();
        (elements.into_iter().take(limit).collect(), total_count)
    }

}
