// Day Page surface entry, render host, and footer helpers.

use chrono::Utc;

use crate::components::notes_editor::{NotesEditorLayout, NotesEditorMarkdownConfig};
use crate::footer_popup::{FooterAction, FooterButtonConfig};
use crate::notes::deeplink_activation::{
    resolve_activation, run_deeplink_confirm_options, Activation, ActivationErrorReason,
    ActivationSurface,
};
use script_kit_gpui::brain::{substrate::BrainSubstrate, wake_indexer};
use script_kit_gpui::day_page::normalize_day_page_markdown_references;
use script_kit_gpui::day_page::{
    parse_day_page_segments, resolve_fragment_path, DayPageBinding, DayPageSegment,
};

const DAY_PAGE_KIT_PREVIEW_MUTED_OPACITY: f32 = 0.72;
const DAY_PAGE_KIT_PREVIEW_BORDER_OPACITY: f32 = 0.2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DayPageKitResourceSourceTarget {
    Note(crate::notes::NoteId),
}

#[derive(Debug, Clone)]
pub(crate) struct DayPageKitPreviewActionAvailability {
    pub(crate) can_add_to_agent_chat: bool,
    pub(crate) can_copy_uri: bool,
    pub(crate) open_source_target: Option<DayPageKitResourceSourceTarget>,
    pub(crate) open_source_unavailable_reason: Option<String>,
    pub(crate) can_close: bool,
}

fn day_page_kit_resource_source_target_for_uri(
    uri: &str,
) -> (Option<DayPageKitResourceSourceTarget>, Option<String>) {
    if uri == "kit://notes" || uri.starts_with("kit://notes?") {
        return (
            None,
            Some("Notes collection previews do not have a single source note.".to_string()),
        );
    }

    if let Some(rest) = uri.strip_prefix("kit://notes/") {
        let id = rest.split(['?', '#']).next().unwrap_or_default();
        let Some(note_id) = crate::notes::NoteId::parse(id) else {
            return (
                None,
                Some("This notes resource URI does not include a valid note id.".to_string()),
            );
        };
        if crate::notes::get_note(note_id).ok().flatten().is_some() {
            return (Some(DayPageKitResourceSourceTarget::Note(note_id)), None);
        }
        return (None, Some("The source note no longer exists.".to_string()));
    }

    if uri == "kit://scripts" {
        return (
            None,
            Some("Scripts collection previews do not have a single source file.".to_string()),
        );
    }

    if uri.starts_with("kit://clipboard-history") {
        return (
            None,
            Some(
                "Clipboard history previews do not have an editable source in this slice."
                    .to_string(),
            ),
        );
    }

    (
        None,
        Some("This resource has no source opener in this slice.".to_string()),
    )
}

impl DayPageView {
    pub fn new(
        app: Entity<ScriptListApp>,
        substrate: BrainSubstrate,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let metrics = crate::notes::window::style::adopted_metrics();
        let (editor_state, notes_editor) = NotesEditor::new_markdown_pair(
            window,
            cx,
            NotesEditorMarkdownConfig::new("")
                .placeholder("Today...")
                .layout(NotesEditorLayout::new(
                    metrics.editor_padding_x,
                    metrics.editor_padding_y,
                ))
                .rows(20),
        );

        // `subscribe_in` already runs the handler with this DayPageView leased
        // (`this` is `&mut Self`); re-leasing via `entity.update` here would
        // double-lease and panic the moment the editor emits a Change.
        let editor_subscription = cx.subscribe_in(
            &editor_state,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Change => this.on_editor_change(window, cx),
                InputEvent::SelectionChange => {
                    this.notes_editor
                        .update(cx, |editor, cx| editor.sync_markdown_link_highlights(cx));
                }
                _ => {}
            },
        );

        Self {
            app: app.downgrade(),
            session: DayPageDocumentSession::new(substrate),
            notes_editor,
            editor_state,
            editor_subscription,
            focus_handle: cx.focus_handle(),
            fragment_open_targets: Vec::new(),
            spine_handoff: Default::default(),
            last_autosave: None,
            autosave_flush_scheduled: false,
            day_switcher: None,
            note_switcher: crate::actions::CommandBar::new(
                Vec::new(),
                crate::actions::CommandBarConfig::notes_recent_style(),
                std::sync::Arc::new(crate::theme::get_cached_theme()),
            ),
            last_editor_content_len: 0,
            kit_resource_preview: None,
        }
    }

    pub fn rebind_substrate(
        &mut self,
        substrate: BrainSubstrate,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        self.session = DayPageDocumentSession::new(substrate);
        self.bind_today(window, cx);
        Ok(())
    }

    pub fn bind_today(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = Utc::now();
        if let Err(error) = self.session.bind_today(now) {
            tracing::error!(error = %error, "Failed to bind today's day page");
            return;
        }
        self.apply_loaded_content_to_editor(window, cx);
    }

    fn apply_loaded_content_to_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.session.disk_content().to_string();
        self.reset_day_page_spine_handoff_state(true, true);
        self.kit_resource_preview = None;
        self.refresh_fragment_open_targets(&content);
        // Loads are not typing: pre-set the length so the Change event this
        // emits cannot read as growth and auto-swap to the main menu.
        self.last_editor_content_len = content.len();
        self.notes_editor.update(cx, |editor, cx| {
            editor.load_value_with_cursor_at_end(content, window, cx);
        });
        self.sync_footer(window, cx);
        self.defer_editor_bottom_scroll(window, cx);
        self.schedule_editor_bottom_scroll_retries(cx);
    }

    fn refresh_fragment_open_targets(&mut self, content: &str) {
        let Some(day_path) = self.session.path().cloned() else {
            self.fragment_open_targets.clear();
            return;
        };
        if self.session.is_viewing_fragment() {
            self.fragment_open_targets.clear();
            return;
        }

        self.fragment_open_targets = parse_day_page_segments(content)
            .into_iter()
            .filter_map(|segment| match segment {
                DayPageSegment::FragmentRef { relative_link, .. } => {
                    resolve_fragment_path(&day_path, &relative_link)
                }
                _ => None,
            })
            .collect();
    }

    pub fn open_fragment_at(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(fragment_path) = self.fragment_open_targets.get(index).cloned() else {
            return;
        };
        let now = Utc::now();
        if let Err(error) = self.session.bind_fragment(fragment_path, now) {
            tracing::error!(error = %error, "Failed to open fragment from day page");
            return;
        }
        self.apply_loaded_content_to_editor(window, cx);
        self.focus_editor(window, cx);
        cx.notify();
    }

    pub fn return_to_day_page(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = Utc::now();
        if let Err(error) = self.session.return_to_day(now) {
            tracing::error!(error = %error, "Failed to return to day page from fragment");
            return;
        }
        self.apply_loaded_content_to_editor(window, cx);
        self.focus_editor(window, cx);
        cx.notify();
    }

    fn on_editor_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor
            .update(cx, |editor, cx| editor.sync_markdown_link_highlights(cx));
        let previous = self.session.disk_content().to_string();
        let mut content = self.notes_editor.read(cx).content(cx);
        let previous_len = self.last_editor_content_len;
        let selection = self.notes_editor.read(cx).selection(cx);
        self.last_editor_content_len = content.len();
        if should_normalize_day_page_references_after_edit(&content, previous_len, selection.end) {
            let normalized = normalize_day_page_markdown_references(&content);
            if normalized != content {
                let mut cursor = selection.end.min(content.len());
                while cursor > 0 && !content.is_char_boundary(cursor) {
                    cursor -= 1;
                }
                let normalized_cursor =
                    normalize_day_page_markdown_references(&content[..cursor]).len();
                self.last_editor_content_len = normalized.len();
                self.notes_editor.update(cx, |editor, cx| {
                    editor.set_value(normalized.clone(), window, cx);
                    editor.set_selection(normalized_cursor, normalized_cursor, window, cx);
                });
                content = normalized;
            }
        }
        if let Some((fixed, cursor)) =
            mention_atomic_delete_fixup(&previous, &content, &self.spine_handoff.mention_aliases)
        {
            self.notes_editor.update(cx, |editor, cx| {
                editor.set_value(fixed.clone(), window, cx);
                editor.set_selection(cursor, cursor, window, cx);
            });
            self.last_editor_content_len = fixed.len();
            self.session.apply_editor_content(&fixed);
            self.refresh_fragment_open_targets(&fixed);
            self.spine_handoff.prune_mention_aliases_for_content(&fixed);
            self.poll_external_disk_changes(window, cx);
            self.schedule_autosave_flush(cx);
            self.sync_footer(window, cx);
            cx.notify();
            return;
        }
        self.session.apply_editor_content(&content);
        self.refresh_fragment_open_targets(&content);
        self.spine_handoff
            .prune_mention_aliases_for_content(&content);
        self.poll_external_disk_changes(window, cx);
        self.schedule_autosave_flush(cx);
        self.sync_footer(window, cx);
        self.maybe_begin_day_page_context_round_trip_from_edit(previous_len, &content, window, cx);
        cx.notify();
    }

    /// Notes-parity autosave: same debounce interval as the Notes window
    /// (`NotesApp::SAVE_DEBOUNCE_MS`), driven from render side effects the
    /// same way `NotesApp::process_render_side_effects` drives
    /// `save_current_note`. A trailing flush timer (scheduled in
    /// `on_editor_change`) guarantees the final keystroke also lands on disk
    /// so the footer dirty state always converges to the real disk state.
    const SAVE_DEBOUNCE_MS: u64 = 300;

    fn maybe_autosave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.session.is_dirty() {
            return;
        }
        let due = self.last_autosave.map_or(true, |at| {
            at.elapsed() >= std::time::Duration::from_millis(Self::SAVE_DEBOUNCE_MS)
        });
        if !due {
            return;
        }
        self.last_autosave = Some(std::time::Instant::now());
        if self.save(cx) {
            tracing::debug!(
                target: "script_kit::day_page",
                event = "day_page_autosaved",
            );
        }
        self.sync_footer(window, cx);
    }

    fn schedule_autosave_flush(&mut self, cx: &mut Context<Self>) {
        if self.autosave_flush_scheduled {
            return;
        }
        self.autosave_flush_scheduled = true;
        let flush_delay = std::time::Duration::from_millis(Self::SAVE_DEBOUNCE_MS + 50);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(flush_delay).await;
            this.update(cx, |this, cx| {
                this.autosave_flush_scheduled = false;
                // Render side effects run the actual save; notify forces a
                // render even when no further input arrives.
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn poll_external_disk_changes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Ok(Some(content)) = self.session.maybe_refresh_from_disk() {
            self.reset_day_page_spine_handoff_state(true, true);
            self.refresh_fragment_open_targets(&content);
            // External refresh is not typing: keep the growth detector quiet.
            self.last_editor_content_len = content.len();
            self.notes_editor.update(cx, |editor, cx| {
                editor.set_value(content, window, cx);
            });
            cx.notify();
        }
    }

    pub fn save(&mut self, cx: &mut Context<Self>) -> bool {
        let content = self.notes_editor.read(cx).content(cx);
        self.session.apply_editor_content(&content);
        match self.session.save_content(&content, Utc::now()) {
            Ok(()) => {
                wake_indexer();
                true
            }
            Err(error) => {
                tracing::error!(error = %error, "Failed to save day page");
                false
            }
        }
    }

    pub fn save_and_sync_footer(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let saved = self.save(cx);
        self.sync_footer(window, cx);
        cx.notify();
        saved
    }

    pub fn is_dirty(&self) -> bool {
        self.session.is_dirty()
    }

    pub(crate) fn automation_input_value(&self, cx: &App) -> String {
        self.notes_editor.read(cx).content(cx)
    }

    pub(crate) fn automation_state(&self, cx: &App) -> serde_json::Value {
        let input = self.automation_input_value(cx);
        let kit_resource_preview = match self.kit_resource_preview.as_ref() {
            Some(preview) => {
                let availability = self
                    .kit_resource_preview_action_availability()
                    .expect("preview action availability exists when preview is open");
                serde_json::json!({
                    "schemaVersion": 1,
                    "active": true,
                    "redacted": true,
                    "title": preview.title,
                    "uri": preview.uri,
                    "mimeType": preview.mime_type,
                    "readOnly": true,
                    "truncated": preview.truncated,
                    "textLength": preview.text.chars().count(),
                    "actionAvailability": {
                        "addToAgentChat": availability.can_add_to_agent_chat,
                        "copyUri": availability.can_copy_uri,
                        "openSource": availability.open_source_target.is_some(),
                        "openSourceReason": availability.open_source_unavailable_reason,
                        "closePreview": availability.can_close,
                    },
                })
            }
            None => serde_json::json!({
                "schemaVersion": 1,
                "active": false,
                "redacted": true,
            }),
        };

        serde_json::json!({
            "schemaVersion": 1,
            "redacted": true,
            "inputLength": input.chars().count(),
            "kitResourcePreview": kit_resource_preview,
        })
    }

    pub fn focus_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_editor_at_end(window, cx);
        self.defer_editor_bottom_scroll(window, cx);
        self.schedule_editor_bottom_scroll_retries(cx);
        let day_page = cx.entity().downgrade();
        window.defer(cx, move |window, cx| {
            let Some(day_page) = day_page.upgrade() else {
                return;
            };
            day_page.update(cx, |view, cx| {
                view.focus_editor_at_end(window, cx);
            });
        });
    }

    fn focus_editor_at_end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.focus_with_cursor_at_end(window, cx);
        });
    }

    fn defer_editor_bottom_scroll(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let day_page = cx.entity().downgrade();
        window.defer(cx, move |window, cx| {
            let Some(day_page) = day_page.upgrade() else {
                return;
            };
            day_page.update(cx, |view, cx| {
                view.notes_editor.update(cx, |editor, cx| {
                    editor.scroll_to_bottom(cx);
                });
                let day_page = cx.entity().downgrade();
                window.defer(cx, move |_window, cx| {
                    let Some(day_page) = day_page.upgrade() else {
                        return;
                    };
                    day_page.update(cx, |view, cx| {
                        view.notes_editor.update(cx, |editor, cx| {
                            editor.scroll_to_bottom(cx);
                        });
                    });
                });
            });
        });
    }

    fn schedule_editor_bottom_scroll_retries(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            for delay_ms in [50_u64, 150, 350, 800] {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(delay_ms))
                    .await;
                this.update(cx, |view, cx| {
                    view.notes_editor.update(cx, |editor, cx| {
                        editor.scroll_to_bottom(cx);
                    });
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    pub fn set_input(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        self.kit_resource_preview = None;
        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value_with_cursor_at_end(text.clone(), window, cx);
        });
        self.session.apply_editor_content(&text);
        self.refresh_fragment_open_targets(&text);
        self.reset_day_page_spine_handoff_state(false, true);
        self.sync_footer(window, cx);
        cx.notify();
    }

    fn reset_day_page_spine_handoff_state(&mut self, clear_cwd_anchor: bool, clear_mentions: bool) {
        self.spine_handoff.reset(clear_cwd_anchor, clear_mentions);
    }

    fn sync_footer(&self, window: &mut Window, cx: &mut Context<Self>) {
        // Deferred on purpose: several callers run while the ScriptListApp
        // lease is already held (e.g. `show_day_page_view` → `bind_today`
        // inside `app_entity.update`, including the hotkey gesture path).
        // A synchronous `app.update` here double-leases ScriptListApp and
        // panics ("cannot update ... while it is already being updated").
        // `window.defer` (not `cx.defer_in`) — the deferred closure must hold
        // NO entity lease, because `sync_main_footer_popup` reads this
        // DayPageView back (`is_dirty` for the footer save button).
        let Some(app) = self.app.upgrade() else {
            return;
        };
        window.defer(cx, move |window, cx| {
            app.update(cx, |app, cx| {
                app.sync_main_footer_popup(window, cx);
            });
        });
    }
}

fn should_normalize_day_page_references_after_edit(
    content: &str,
    previous_len: usize,
    cursor: usize,
) -> bool {
    if content.len() <= previous_len {
        return false;
    }
    let growth = content.len().saturating_sub(previous_len);
    if growth > 1 {
        return true;
    }
    let mut cursor = cursor.min(content.len());
    while cursor > 0 && !content.is_char_boundary(cursor) {
        cursor -= 1;
    }
    content[..cursor]
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace)
}

impl Focusable for DayPageView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DayPageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_external_disk_changes(window, cx);
        self.maybe_autosave(window, cx);

        let app = self.app.upgrade().expect("DayPageView app entity dropped");

        let app_state = app.read(cx);
        let menu_def = app_state.current_main_menu_theme.def();
        let shell = menu_def.shell;
        let search = menu_def.search;
        let tokens = get_tokens(app_state.current_design);
        let design_visual = tokens.visual();
        let is_default_design = app_state.current_design.is_default();
        let design_spacing = tokens.spacing();
        let text_primary = app_state.theme.colors.text.primary;
        let font_family = app_state.theme_font_family();

        let header_padding_x = shell.header_padding_x;
        let header_padding_y = if is_default_design {
            shell.header_padding_y
        } else {
            design_spacing.padding_sm
        };
        let header_gap = if is_default_design {
            shell.header_gap
        } else {
            design_spacing.gap_md
        };

        let columns = crate::components::main_view_chrome::main_view_content_columns(menu_def);
        let editor_input = self.notes_editor.read(cx).render_input(cx);
        let editor_input = div()
            .relative()
            .flex_1()
            .min_h(px(0.))
            .h_full()
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, window, cx| {
                    this.activate_deeplink_from_mouse_up(event.clone(), window, cx);
                }),
            )
            .child(editor_input);
        let viewing_fragment = self.session.is_viewing_fragment();
        let theme = app_state.theme.clone();

        let local_today = Utc::now()
            .with_timezone(&self.session.substrate().timezone())
            .date_naive();
        let viewing_past_day = !viewing_fragment
            && self
                .session
                .bound_date()
                .is_some_and(|date| date != local_today);

        let back_bar = if viewing_fragment {
            let label = match self.session.binding() {
                DayPageBinding::Fragment {
                    return_day_date, ..
                } => {
                    format!("Today · {return_day_date}")
                }
                DayPageBinding::Day => "Today".to_string(),
                DayPageBinding::Note { title, .. } => title.clone(),
            };
            Some(
                div()
                    .id(script_kit_gpui::day_page::FRAGMENT_BACK_ID)
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .pb(px(6.))
                    .text_sm()
                    .cursor_pointer()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            this.return_to_day_page(window, cx);
                        }),
                    )
                    .child("←")
                    .child(label),
            )
        } else if viewing_past_day {
            let label = self
                .session
                .bound_date()
                .map(|date| format!("Back to Today · viewing {date}"))
                .unwrap_or_else(|| "Back to Today".to_string());
            Some(
                div()
                    .id("day-page-past-day-back")
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .pb(px(6.))
                    .text_sm()
                    .cursor_pointer()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            this.bind_today(window, cx);
                            this.focus_editor(window, cx);
                        }),
                    )
                    .child("←")
                    .child(label),
            )
        } else if self.session.is_viewing_note() {
            let label = self
                .session
                .viewing_note_title()
                .map(|title| format!("Back to Today · viewing {title}"))
                .unwrap_or_else(|| "Back to Today".to_string());
            Some(
                div()
                    .id("day-page-note-back")
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .pb(px(6.))
                    .text_sm()
                    .cursor_pointer()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            this.return_to_day_page(window, cx);
                            this.focus_editor(window, cx);
                        }),
                    )
                    .child("←")
                    .child(label),
            )
        } else {
            None
        };

        let editor_content = if self.kit_resource_preview.is_some() {
            self.render_kit_resource_preview(cx)
        } else {
            div()
                .relative()
                .flex_1()
                .min_h(px(0.))
                .child(editor_input)
                .into_any_element()
        };

        let editor_body = div()
            .id(DAY_PAGE_EDITOR_ID)
            .flex_1()
            .min_h(px(0.))
            .h_full()
            .pl(px(columns.input_text_inset_left))
            .pr(px(columns.content_right_inset_x))
            .flex()
            .flex_col()
            .when_some(back_bar, |parent, bar| parent.child(bar))
            .child(editor_content);

        let context_zone = app.update(cx, |app, _cx| {
            app.render_inert_main_view_context_zone(menu_def)
        });

        let main = div()
            .flex()
            .flex_col()
            .h_full()
            .min_h(px(0.))
            .w_full()
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .min_h(px(search.height))
                    .flex_1()
                    .min_h(px(0.))
                    .child(editor_body),
            )
            .into_any_element();

        let header = crate::components::main_view_chrome::MainViewHeaderChrome {
            context: Some(context_zone),
            input: div().into_any_element(),
            padding_x: header_padding_x,
            padding_y: header_padding_y,
            gap: header_gap,
        };

        let divider = crate::components::main_view_chrome::MainViewDividerChrome {
            margin_x: shell.divider_margin_x,
            height: if is_default_design {
                shell.divider_height
            } else {
                design_visual.border_thin
            },
            visible: false,
        };

        let root = crate::components::main_view_chrome::render_main_view_shell()
            .text_color(rgb(text_primary))
            .font_family(font_family)
            .key_context("day_page")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, window, cx| {
                this.handle_key_down(event, window, cx);
            }));

        crate::components::main_view_chrome::render_main_view_chrome(
            root,
            &theme,
            menu_def,
            crate::components::main_view_chrome::MainViewChrome {
                header,
                divider,
                main,
                footer: Some(
                    crate::components::prompt_layout_shell::render_native_main_window_footer_spacer(
                    ),
                ),
                overlays: Vec::new(),
            },
        )
    }
}

impl DayPageView {
    pub(crate) fn execute_day_page_action_from_preview(
        &mut self,
        action_id: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(app) = self.app.upgrade() else {
            return;
        };
        window.defer(cx, move |window, cx| {
            let _ = app.update(cx, |app, cx| {
                app.execute_day_page_action(action_id, window, cx);
            });
        });
    }

    fn render_kit_resource_preview(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(preview) = self.kit_resource_preview.as_ref() else {
            return div().into_any_element();
        };

        let title = preview.title.clone();
        let uri = preview.uri.clone();
        let mime_type = preview.mime_type.clone();
        let text = preview.text.clone();
        let truncated = preview.truncated;
        let availability = self
            .kit_resource_preview_action_availability()
            .expect("preview action availability exists when preview is open");

        div()
            .id("day-page-kit-resource-preview")
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_col()
            .gap_3()
            .py_2()
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .id("day-page-kit-resource-preview-title")
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .id("day-page-kit-resource-preview-uri")
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(uri),
                            )
                            .child(
                                div()
                                    .id("day-page-kit-resource-preview-meta")
                                    .text_xs()
                                    .text_color(
                                        cx.theme()
                                            .muted_foreground
                                            .opacity(DAY_PAGE_KIT_PREVIEW_MUTED_OPACITY),
                                    )
                                    .child(format!(
                                        "{mime_type} · read-only{}",
                                        if truncated { " · truncated" } else { "" }
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when(availability.can_add_to_agent_chat, |parent| {
                                parent.child(
                                    div()
                                        .id("day-page-kit-resource-preview-add-agent-chat")
                                        .text_xs()
                                        .text_color(cx.theme().accent)
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(cx.theme().foreground))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.execute_day_page_action_from_preview(
                                                crate::DAY_PAGE_PREVIEW_ADD_TO_AGENT_CHAT_ACTION_ID,
                                                window,
                                                cx,
                                            );
                                        }))
                                        .child("Add to Agent Chat"),
                                )
                            })
                            .child(
                                div()
                                    .id("day-page-kit-resource-preview-copy-uri")
                                    .text_xs()
                                    .text_color(cx.theme().accent)
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(cx.theme().foreground))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.execute_day_page_action_from_preview(
                                            crate::DAY_PAGE_PREVIEW_COPY_URI_ACTION_ID,
                                            window,
                                            cx,
                                        );
                                    }))
                                    .child("Copy URI"),
                            )
                            .when(availability.open_source_target.is_some(), |parent| {
                                parent.child(
                                    div()
                                        .id("day-page-kit-resource-preview-open-source")
                                        .text_xs()
                                        .text_color(cx.theme().accent)
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(cx.theme().foreground))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.execute_day_page_action_from_preview(
                                                crate::DAY_PAGE_PREVIEW_OPEN_SOURCE_ACTION_ID,
                                                window,
                                                cx,
                                            );
                                        }))
                                        .child("Open Source"),
                                )
                            })
                            .child(
                                div()
                                    .id("day-page-kit-resource-preview-close")
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .cursor_pointer()
                                    .hover(|s| s.text_color(cx.theme().foreground))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.execute_day_page_action_from_preview(
                                            crate::DAY_PAGE_PREVIEW_CLOSE_ACTION_ID,
                                            window,
                                            cx,
                                        );
                                    }))
                                    .child("Close Preview"),
                            ),
                    ),
            )
            .child(
                div()
                    .id("day-page-kit-resource-preview-body")
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_y_scroll()
                    .rounded(px(6.))
                    .border_1()
                    .border_color(
                        cx.theme()
                            .border
                            .opacity(DAY_PAGE_KIT_PREVIEW_BORDER_OPACITY),
                    )
                    .p_3()
                    .text_xs()
                    .font_family(FONT_MONO)
                    .text_color(cx.theme().foreground)
                    .child(text),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(
                        cx.theme()
                            .muted_foreground
                            .opacity(DAY_PAGE_KIT_PREVIEW_MUTED_OPACITY),
                    )
                    .child("Esc to return"),
            )
            .into_any_element()
    }

    pub(crate) fn kit_resource_preview_action_availability(
        &self,
    ) -> Option<DayPageKitPreviewActionAvailability> {
        let preview = self.kit_resource_preview.as_ref()?;
        let (open_source_target, open_source_unavailable_reason) =
            day_page_kit_resource_source_target_for_uri(&preview.uri);
        Some(DayPageKitPreviewActionAvailability {
            can_add_to_agent_chat: preview.allow_agent_chat_action,
            can_copy_uri: true,
            open_source_target,
            open_source_unavailable_reason,
            can_close: true,
        })
    }

    pub(crate) fn open_kit_resource_preview_source(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(availability) = self.kit_resource_preview_action_availability() else {
            return false;
        };
        match availability.open_source_target {
            Some(DayPageKitResourceSourceTarget::Note(note_id)) => {
                let Ok(Some(note)) = crate::notes::get_note(note_id) else {
                    return false;
                };
                let path = crate::notes::note_file_path(note.id).ok().flatten();
                if let Err(error) = self.session.bind_note_content(
                    note.id.as_str().to_string(),
                    note.title.clone(),
                    note.content.clone(),
                    path,
                    Utc::now(),
                ) {
                    tracing::error!(
                        target: "script_kit::day_page",
                        error = %error,
                        "day_page_kit_preview_open_source_failed"
                    );
                    return false;
                }
                self.apply_loaded_content_to_editor(window, cx);
                self.focus_editor(window, cx);
                cx.notify();
                true
            }
            None => false,
        }
    }

    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.to_lowercase();
        self.handle_key_parts(
            &key,
            event.keystroke.modifiers.platform,
            event.keystroke.modifiers.shift,
            event.keystroke.modifiers.alt,
            event.keystroke.modifiers.control,
            window,
            cx,
        );
    }

    pub(crate) fn handle_key_parts(
        &mut self,
        key: &str,
        cmd: bool,
        shift: bool,
        alt: bool,
        control: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let exact_plain = !cmd && !shift && !alt && !control;
        let exact_cmd = cmd && !shift && !alt && !control;

        if exact_plain
            && crate::ui_foundation::is_key_escape(&key)
            && crate::confirm::is_confirm_window_open()
        {
            crate::confirm::route_key_to_confirm_popup("escape", cx);
            return;
        }

        if exact_plain
            && crate::ui_foundation::is_key_escape(&key)
            && self.kit_resource_preview.is_some()
        {
            self.close_kit_resource_preview(window, cx);
            return;
        }

        if self.is_day_switcher_open() {
            if self.handle_day_switcher_key(key, cmd, shift, alt, control, window, cx) {
                return;
            }
        }

        if exact_plain && crate::ui_foundation::is_key_escape(&key) {
            if self.session.is_viewing_fragment() || self.session.is_viewing_note() {
                self.return_to_day_page(window, cx);
                return;
            }
            // Escape from a past day returns to today before closing the
            // window, keeping the dismissal ladder predictable.
            let today = Utc::now()
                .with_timezone(&self.session.substrate().timezone())
                .date_naive();
            if self.session.bound_date().is_some_and(|date| date != today) {
                self.bind_today(window, cx);
                self.focus_editor(window, cx);
                return;
            }
            if let Some(app) = self.app.upgrade() {
                window.defer(cx, move |_window, cx| {
                    app.update(cx, |app, cx| {
                        app.close_and_reset_window(cx);
                    });
                });
            }
            return;
        }

        if exact_cmd && key == "s" {
            self.save_and_sync_footer(window, cx);
            return;
        }

        if exact_cmd && key == "p" {
            self.open_note_switcher(window, cx);
            return;
        }

        if exact_cmd && key == "." {
            self.activate_deeplink_under_cursor(window, cx);
            return;
        }

        // Markdown formatting shortcuts — same bindings as the Notes window
        // (`src/notes/window/keyboard.rs`), routed through the shared
        // NotesEditor toolbar action executor.
        if exact_cmd && key == "b" {
            self.run_shared_markdown_toolbar_action("bold", window, cx);
            return;
        }
        if exact_cmd && key == "i" {
            self.run_shared_markdown_toolbar_action("italic", window, cx);
            return;
        }
        if exact_cmd && key == "e" {
            self.run_shared_markdown_toolbar_action("code", window, cx);
            return;
        }
        if cmd && shift && !alt && !control && key == "x" {
            self.run_shared_markdown_toolbar_action("strikethrough", window, cx);
        }
    }

    fn activate_deeplink_under_cursor(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let href = self.notes_editor.read(cx).activation_href_at_cursor(cx);
        let Some(href) = href else {
            return false;
        };
        let activation = resolve_activation(&href, ActivationSurface::DayPage);
        self.handle_deeplink_activation(activation, window, cx);
        true
    }

    fn activate_deeplink_from_mouse_up(
        &mut self,
        event: gpui::MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let day_page = cx.entity().downgrade();
        window.defer(cx, move |window, cx| {
            let Some(day_page) = day_page.upgrade() else {
                return;
            };
            day_page.update(cx, |this, cx| {
                let selection = this.notes_editor.read(cx).selection(cx);
                if !crate::components::notes_editor::should_activate_deeplink_from_mouse_up(
                    &event, selection,
                ) {
                    return;
                }

                this.activate_deeplink_under_cursor(window, cx);
            });
        });
    }

    fn handle_deeplink_activation(
        &mut self,
        activation: Activation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match activation {
            Activation::ConfirmBeforeRun {
                command_id,
                raw_href,
            } => self.open_run_deeplink_confirm(command_id, raw_href, window, cx),
            Activation::Error(error) => {
                self.open_deeplink_info_dialog(
                    "Can't open this link",
                    format!(
                        "{}\n\n{}",
                        error.raw_href,
                        day_page_activation_error_message(&error.reason)
                    ),
                    error.raw_href,
                    window,
                    cx,
                );
            }
            Activation::OpenExternalUrl { href } => {
                self.open_external_deeplink_url(href, window, cx);
            }
            Activation::OpenFile { path, raw_href } => {
                self.open_file_deeplink(path, raw_href, window, cx);
            }
            Activation::OpenNote { note_id } => {
                self.open_note_deeplink(note_id, window, cx);
            }
            Activation::ScopedSearch { source, query } => {
                self.open_scoped_search_deeplink(source, query, window, cx);
            }
            Activation::KitResourcePreview {
                uri,
                allow_agent_chat_action,
            } => {
                self.open_kit_resource_preview(uri, allow_agent_chat_action, window, cx);
            }
        }
    }

    fn open_kit_resource_preview(
        &mut self,
        uri: String,
        allow_agent_chat_action: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match crate::notes::deeplink_activation::read_cheap_kit_resource_preview(&uri) {
            Ok(preview) => {
                tracing::info!(
                    event = "day_page_deeplink_kit_resource_preview_opened",
                    uri = %preview.uri,
                    mime_type = %preview.mime_type,
                    truncated = preview.truncated,
                );
                self.kit_resource_preview = Some(DayPageKitResourcePreviewState::from_preview(
                    preview,
                    allow_agent_chat_action,
                ));
                self.focus_handle.focus(window, cx);
                self.sync_footer(window, cx);
                cx.notify();
            }
            Err(error) => self.open_deeplink_info_dialog(
                "Can't open this link",
                format!("{uri}\n\n{error}"),
                uri,
                window,
                cx,
            ),
        }
    }

    fn close_kit_resource_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.kit_resource_preview.take().is_some() {
            self.focus_editor(window, cx);
            self.sync_footer(window, cx);
            cx.notify();
        }
    }

    fn open_external_deeplink_url(
        &mut self,
        href: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match open::that(&href) {
            Ok(()) => {
                tracing::info!(event = "day_page_deeplink_url_opened", href = %href);
                self.sync_footer(window, cx);
            }
            Err(error) => self.open_deeplink_info_dialog(
                "Can't open this link",
                format!("{href}\n\nFailed to open URL: {error}"),
                href,
                window,
                cx,
            ),
        }
    }

    fn open_file_deeplink(
        &mut self,
        path: PathBuf,
        raw_href: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let path_display = path.to_string_lossy().to_string();
        if !path.exists() {
            self.open_missing_file_deeplink_dialog(path, raw_href, window, cx);
            return;
        }

        match crate::file_search::open_file(&path_display) {
            Ok(()) => {
                tracing::info!(
                    event = "day_page_deeplink_file_opened",
                    path = %path_display,
                    raw_href = %raw_href,
                );
                self.sync_footer(window, cx);
            }
            Err(error) => self.open_deeplink_info_dialog(
                "Can't open this link",
                format!("{raw_href}\n\nFailed to open file:\n{path_display}\n\n{error}"),
                raw_href,
                window,
                cx,
            ),
        }
    }

    fn open_missing_file_deeplink_dialog(
        &mut self,
        path: PathBuf,
        raw_href: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let path_display = path.to_string_lossy().to_string();
        self.open_deeplink_info_dialog(
            "Can't open this link",
            format!("{raw_href}\n\nFile does not exist:\n{path_display}"),
            raw_href,
            window,
            cx,
        );
    }

    fn open_note_deeplink(
        &mut self,
        note_id: crate::notes::NoteId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match crate::notes::open_note_in_notes_window(cx, note_id) {
            Ok(()) => {
                tracing::info!(event = "day_page_deeplink_note_opened", note_id = %note_id);
                self.sync_footer(window, cx);
            }
            Err(error) => self.open_deeplink_info_dialog(
                "Can't open this link",
                format!(
                    "scriptkit://notes/{}\n\nCould not open note: {}",
                    note_id.as_str(),
                    error
                ),
                format!("scriptkit://notes/{}", note_id.as_str()),
                window,
                cx,
            ),
        }
    }

    fn open_scoped_search_deeplink(
        &mut self,
        source: crate::spine::catalog_subsearch::ContextSubsearchSource,
        query: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let context_link = format!("@{}:{query}", source.prefix());
        if matches!(
            source,
            crate::spine::catalog_subsearch::ContextSubsearchSource::File
                | crate::spine::catalog_subsearch::ContextSubsearchSource::Project
        ) {
            let path = PathBuf::from(query.trim());
            if path.exists() {
                self.open_file_deeplink(path, context_link, window, cx);
                return;
            }
        }

        if source == crate::spine::catalog_subsearch::ContextSubsearchSource::BrowserHistory
            && (query.starts_with("http://") || query.starts_with("https://"))
        {
            self.open_external_deeplink_url(query, window, cx);
            return;
        }

        self.open_deeplink_info_dialog(
            "Open context search",
            format!(
                "{context_link}\n\nUse this scoped context token to search {} for matching context. Exact file/project paths and exact browser URLs open directly.",
                source.search_hint_noun()
            ),
            context_link,
            window,
            cx,
        );
    }

    fn open_run_deeplink_confirm(
        &mut self,
        command_id: String,
        raw_href: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.app.upgrade().is_none() {
            self.open_deeplink_info_dialog(
                "Can't open this link",
                format!(
                    "{}\n\nCould not attach the confirmation prompt to the Day Page.",
                    raw_href
                ),
                raw_href,
                window,
                cx,
            );
            return;
        }

        let command_id_for_confirm = command_id.clone();
        let command_id_for_cancel = command_id.clone();
        let app = self.app.clone();
        let (sender, receiver) = async_channel::bounded::<bool>(1);
        self.open_deferred_confirm_prompt(
            run_deeplink_confirm_options(&command_id, &raw_href),
            sender,
            cx,
        );
        cx.spawn(async move |_this, cx| {
            if receiver.recv().await.unwrap_or(false) {
                let executed = cx.update(|cx| {
                    app.upgrade()
                        .map(|app| {
                            app.update(cx, |app, cx| {
                                app.execute_by_command_id_or_path(&command_id_for_confirm, cx)
                            })
                        })
                        .unwrap_or(false)
                });
                tracing::info!(
                    event = "day_page_deeplink_run_confirmed",
                    command_id = %command_id_for_confirm,
                    executed,
                    "day_page_deeplink_run_confirmed",
                );
            } else {
                tracing::info!(
                    event = "day_page_deeplink_run_cancelled",
                    command_id = %command_id_for_cancel,
                    "day_page_deeplink_run_cancelled",
                );
            }
        })
        .detach();
    }

    fn open_deeplink_info_dialog(
        &mut self,
        title: &'static str,
        body: String,
        copy_link: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.app.upgrade().is_none() {
            tracing::warn!(event = "day_page_deeplink_modal_unavailable", title);
            self.sync_footer(window, cx);
            return;
        }

        let (sender, receiver) = async_channel::bounded::<bool>(1);
        self.open_deferred_confirm_prompt(
            crate::confirm::ParentConfirmOptions {
                title: title.into(),
                body: body.into(),
                confirm_text: "Copy link".into(),
                cancel_text: "Dismiss".into(),
                confirm_variant: gpui_component::button::ButtonVariant::Primary,
                width: gpui::px(crate::confirm::PARENT_CONFIRM_DIALOG_WIDTH_PX),
            },
            sender,
            cx,
        );
        cx.spawn(async move |_this, cx| {
            if receiver.recv().await.unwrap_or(false) {
                cx.update(|cx| {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(copy_link.clone()));
                });
            }
        })
        .detach();
    }

    fn open_deferred_confirm_prompt(
        &self,
        options: crate::confirm::ParentConfirmOptions,
        sender: async_channel::Sender<bool>,
        cx: &mut Context<Self>,
    ) {
        let app = self.app.clone();
        cx.spawn(async move |_this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1))
                .await;
            cx.update(|cx| {
                if let Some(app) = app.upgrade() {
                    app.update(cx, |app, cx| {
                        app.open_confirm_prompt(options, sender, cx);
                    });
                }
            });
        })
        .detach();
    }
}

fn day_page_activation_error_message(reason: &ActivationErrorReason) -> String {
    match reason {
        ActivationErrorReason::EmptyHref => "The link is empty.".to_string(),
        ActivationErrorReason::UnknownScheme { scheme } => {
            format!("`{scheme}` is not a supported link scheme.")
        }
        ActivationErrorReason::UnknownSpinePrefix { prefix, supported } => format!(
            "`{prefix}` is not a supported context type. Supported types: {}.",
            supported.join(", ")
        ),
        ActivationErrorReason::EmptySpineValue { prefix } => {
            format!("`{prefix}` context links need a value to search or open.")
        }
        ActivationErrorReason::MalformedUri { message } => message.clone(),
    }
}

pub(crate) fn day_page_footer_buttons(
    app: &ScriptListApp,
    _cx: Option<&gpui::App>,
) -> Vec<FooterButtonConfig> {
    let footer_disabled = crate::confirm::is_confirm_window_open();
    let actions_open = app.show_actions_popup || crate::actions::is_actions_window_open();
    let enabled = !footer_disabled;

    vec![
        FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
            .selected(actions_open)
            .enabled(enabled),
    ]
}

impl ScriptListApp {
    pub(crate) fn show_day_page_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_day_page_view_with_substrate(None, window, cx);
    }

    pub(crate) fn show_day_page_view_with_substrate(
        &mut self,
        substrate: Option<BrainSubstrate>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let substrate = substrate.unwrap_or_else(BrainSubstrate::default_kit);
        let app_entity = cx.entity();

        // A pending Today → main-menu context round trip holds the live Day
        // Page entity; re-entering Today resumes it (and abandons the search)
        // instead of binding a fresh view.
        if let Some(pending) = self.day_page_context_return.take() {
            self.restore_day_page_view_after_round_trip(pending.entity, window, cx);
            return;
        }

        let entity = if let AppView::DayPage { entity } = &self.current_view {
            let entity = entity.clone();
            entity.update(cx, |view, cx| {
                if let Err(error) = view.rebind_substrate(substrate, window, cx) {
                    tracing::error!(
                        target: "script_kit::day_page",
                        event = "day_page_rebind_failed",
                        error = %error,
                    );
                }
            });
            entity
        } else {
            let day_page = cx.new(|cx| DayPageView::new(app_entity, substrate, window, cx));
            day_page.update(cx, |view, cx| view.bind_today(window, cx));
            day_page
        };

        entity.update(cx, |view, cx| view.focus_editor(window, cx));
        self.current_view = AppView::DayPage { entity };
        self.focused_input = FocusedInput::None;
        self.rekey_main_automation_surface_from_current_view();
        self.sync_main_footer_popup(window, cx);
        cx.notify();
    }
}
