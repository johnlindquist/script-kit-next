impl ScriptListApp {
    pub(crate) fn resolved_main_menu_selected_subject(
        &self,
    ) -> Option<ResolvedMainMenuSelection<'_>> {
        if !self.root_search.query_is_current() {
            return None;
        }
        self.painted_main_menu_selected_subject()
    }

    /// Keep the committed row and its highlight together while a new query coalesces.
    /// This is presentation only: dispatch must use the current-query resolver above.
    pub(crate) fn painted_main_menu_selected_subject(
        &self,
    ) -> Option<ResolvedMainMenuSelection<'_>> {
        if !matches!(self.current_view, AppView::ScriptList)
            || self.main_menu_committed_query_stamp() != self.root_search.computed_query_stamp()
            || self.spine_empty_subsearch_selection_suppressed()
            || self.menu_syntax_capture_form_owns_input()
        {
            return None;
        }
        let row = self
            .main_menu_committed_rows()
            .iter()
            .find(|row| row.grouped_index == self.selected_index && row.eligibility.selectable)?;
        match row.subject {
            MainMenuRowSubject::SearchResult { flat_index } => {
                Some(ResolvedMainMenuSelection::SearchResult {
                    row,
                    result: self.main_menu_committed_results().get(flat_index)?,
                })
            }
            MainMenuRowSubject::Calculator => Some(ResolvedMainMenuSelection::Calculator {
                row,
                result: self.main_menu_committed_calculator()?,
            }),
        }
    }

    pub(crate) fn resolve_main_menu_semantic_row(
        &self,
        semantic_id: &str,
    ) -> Option<&MainMenuRowProjection> {
        if !self.root_search.query_is_current()
            || self.main_menu_committed_query_stamp() != self.root_search.computed_query_stamp()
        {
            return None;
        }
        self.main_menu_committed_rows()
            .iter()
            .find(|row| row.semantic_id == semantic_id && row.eligibility.selectable)
    }
}

fn prepend_inline_calculator_group(
    grouped_items: Vec<GroupedListItem>,
    flat_results: Vec<scripts::SearchResult>,
    calculator: Option<&crate::calculator::CalculatorInlineResult>,
) -> (Vec<GroupedListItem>, Vec<scripts::SearchResult>) {
    let Some(_calculator) = calculator else {
        return (grouped_items, flat_results);
    };

    let mut merged_grouped_items = Vec::with_capacity(grouped_items.len() + 2);
    merged_grouped_items.push(GroupedListItem::SectionHeader(
        INLINE_CALCULATOR_SECTION_LABEL.to_string(),
        None,
    ));
    merged_grouped_items.push(GroupedListItem::Item(INLINE_CALCULATOR_RESULT_INDEX));
    merged_grouped_items.extend(grouped_items);

    (merged_grouped_items, flat_results)
}

#[path = "menu_syntax_main_list_rows.rs"]
mod menu_syntax_main_list_rows;
use menu_syntax_main_list_rows::{
    build_menu_syntax_object_selector_main_list_results,
    build_menu_syntax_trigger_picker_main_list_results,
};
