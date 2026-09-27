// Day Page Agent Chat handoff helpers.

use crate::components::notes_editor::spine::mention_atomic_delete_fixup;

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
        let metrics = crate::notes::style::adopted_metrics();
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
        let mut text_plane = protocol::ElementInfo::panel("day-page-editor-text-plane");
        text_plane.role = Some("day_page_editor_text_plane".to_string());
        text_plane.kind = Some("layout_receipt".to_string());
        text_plane.source = Some("DayPageEditorTextPlane".to_string());
        text_plane.source_name = Some("getLayoutInfo".to_string());
        text_plane.selectable = Some(false);

        let identity = crate::components::main_view_chrome::SemanticChipSpec::enabled_identity(
            self.document_semantic_id(),
            self.document_label(),
            crate::components::main_view_chrome::SemanticChipAction::OpenDetails,
            "⌘P",
        );
        let mut elements = vec![
            protocol::ElementInfo::panel("day-page"),
            crate::windows::automation_surface_collector::collect_semantic_chip_element(&identity),
            editor,
            text_plane,
        ];

        if !self.clipboard_shelf.is_empty() && self.kit_resource_preview.is_none() {
            let count = self.clipboard_shelf.len();
            let mut shelf = protocol::ElementInfo::panel("day-page-clipboard-shelf");
            shelf.text = Some(format!(
                "Clipboard · {count} kept {}",
                if count == 1 { "entry" } else { "entries" }
            ));
            shelf.value = Some(if self.clipboard_shelf_expanded {
                "expanded".to_string()
            } else {
                "collapsed".to_string()
            });
            shelf.selected = Some(self.clipboard_shelf_expanded);
            shelf.role = Some("day_page_clipboard_shelf".to_string());
            shelf.kind = Some("layout_receipt".to_string());
            shelf.source = Some("DayPageClipboardShelf".to_string());
            shelf.source_name = Some("getLayoutInfo".to_string());
            shelf.selectable = Some(false);
            elements.push(shelf);
            let mut toggle = protocol::ElementInfo::button(
                0,
                if self.clipboard_shelf_expanded {
                    "Collapse kept clipboard entries"
                } else {
                    "Expand kept clipboard entries"
                },
            );
            toggle.semantic_id = "day-page-clipboard-shelf-toggle".to_string();
            toggle.source = Some("day_page:clipboard_toggle".to_string());
            toggle.selected = Some(self.clipboard_shelf_expanded);
            toggle.selectable = Some(true);
            elements.push(toggle);
            if self.clipboard_shelf_expanded {
                for (index, entry) in self.clipboard_shelf.iter().enumerate() {
                    let mut control =
                        protocol::ElementInfo::button(index, "Open kept clipboard entry");
                    control.semantic_id =
                        format!("day-page-clipboard-shelf-item-{}", entry.item.entry_id);
                    control.source =
                        Some(format!("day_page:clipboard_open:{}", entry.item.entry_id));
                    control.value = Some(crate::clipboard_history::entry_resource_uri(
                        &entry.item.entry_id,
                    ));
                    control.selectable = Some(true);
                    elements.push(control);
                }
            }
        }

        let status = crate::components::document_status::DocumentSaveStatus::from_facts(
            self.session.is_dirty(),
            self.save_failure.as_ref(),
        );
        let mut save = protocol::ElementInfo::panel("day-page-save-status");
        save.text = Some(status.label().to_string());
        save.status_kind = Some(format!("{status:?}").to_lowercase());
        elements.push(save);
        if let Some(failure) = self.handoff_failure.as_ref() {
            let mut panel = protocol::ElementInfo::panel("day-page-handoff-failure");
            panel.text = Some(failure.primary_message().to_string());
            panel.value = Some(serde_json::json!({"failure": failure.failure}).to_string());
            panel.selectable = Some(false);
            elements.push(panel);
        }

        if self.read_mode && self.kit_resource_preview.is_none() {
            elements.push(protocol::ElementInfo {
                semantic_id: "day-page-read-preview".to_string(),
                element_type: protocol::ElementType::Panel,
                text: Some(format!("{} Preview", self.document_label())),
                value: Some(content.clone()),
                content: None,
                selected: None,
                focused: None,
                index: None,
                role: Some("day_page_read_preview".to_string()),
                kind: Some("markdown_preview".to_string()),
                source: Some(
                    crate::components::notes_editor::NOTES_EDITOR_PREVIEW_RENDER_PATH.to_string(),
                ),
                source_name: Some(
                    crate::components::notes_editor::NOTES_EDITOR_STYLE_OWNER.to_string(),
                ),
                selectable: Some(false),
                status_kind: None,
                action_disabled: None,
                style: Some(protocol::ElementStyleInfo {
                    owner: editor_surface.owner.to_string(),
                    input_render_path: Some(
                        crate::components::notes_editor::NOTES_EDITOR_PREVIEW_RENDER_PATH
                            .to_string(),
                    ),
                    editor_runtime: None,
                    surface_background_rgb: Some(editor_surface.background_rgb),
                    occlusion_rgba: Some(editor_surface.occlusion_rgba),
                    padding_x: Some(metrics.editor_padding_x),
                    padding_y: Some(metrics.editor_padding_y),
                    font_family_source: Some("theme.mono_font_family".to_string()),
                    text_size_source: Some("theme.mono_font_size".to_string()),
                }),
            });
        }

        if self.session.is_viewing_fragment() {
            elements.push(protocol::ElementInfo {
                semantic_id: script_kit_gpui::day_page::FRAGMENT_BACK_ID.to_string(),
                element_type: protocol::ElementType::Button,
                text: Some("Back to Today".to_string()),
                value: Some("day_page:back_to_today".to_string()),
                content: None,
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

        // NOTE: the live day/note switcher is `note_switcher`, a centered
        // CommandBar popup that surfaces its own rows through its popup
        // automation window — not through this main-window element collection.
        // The old inline `day_switcher` state was dead (always None), so its
        // element block was removed along with the machinery.

        let total_count = elements.len();
        (elements.into_iter().take(limit).collect(), total_count)
    }
}
