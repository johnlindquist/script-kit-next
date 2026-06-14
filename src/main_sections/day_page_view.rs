// Day Page surface entry, render host, and footer helpers.

use chrono::Utc;

use crate::components::notes_editor::{NotesEditorLayout, NotesEditorMarkdownConfig};
use crate::footer_popup::{FooterAction, FooterButtonConfig};
use script_kit_gpui::day_page::normalize_day_page_markdown_references;
use script_kit_gpui::brain::{substrate::BrainSubstrate, wake_indexer};
use script_kit_gpui::day_page::{
    parse_day_page_segments, resolve_fragment_path, DayPageBinding, DayPageSegment,
};

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
            |this, _, event: &InputEvent, window, cx| {
                if !matches!(event, InputEvent::Change) {
                    return;
                }
                this.on_editor_change(window, cx);
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
            last_editor_content_len: 0,
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
        self.refresh_fragment_open_targets(&content);
        // Loads are not typing: pre-set the length so the Change event this
        // emits cannot read as growth and auto-swap to the main menu.
        self.last_editor_content_len = content.len();
        self.notes_editor.update(cx, |editor, cx| {
            editor.load_value_with_cursor_at_end(content, window, cx);
        });
        self.sync_footer(window, cx);
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
        self.spine_handoff.prune_mention_aliases_for_content(&content);
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

    pub fn primary_action_label(&self) -> String {
        if self.session.is_dirty() {
            "Save".to_string()
        } else {
            "Saved".to_string()
        }
    }

    pub(crate) fn automation_input_value(&self, cx: &App) -> String {
        self.notes_editor.read(cx).content(cx)
    }

    pub fn focus_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.focus(window, cx);
        });
    }

    pub fn set_input(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value_with_cursor_at_end(text.clone(), window, cx);
        });
        self.session.apply_editor_content(&text);
        self.refresh_fragment_open_targets(&text);
        self.reset_day_page_spine_handoff_state(false, true);
        self.sync_footer(window, cx);
        cx.notify();
    }

    pub(crate) fn append_main_hotkey_carry(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = normalize_day_page_markdown_references(text.trim());
        if text.is_empty() {
            return;
        }

        let mut content = self.notes_editor.read(cx).content(cx);
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&text);

        self.last_editor_content_len = content.len();
        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value_with_cursor_at_end(content.clone(), window, cx);
        });
        self.session.apply_editor_content(&content);
        self.refresh_fragment_open_targets(&content);
        self.reset_day_page_spine_handoff_state(false, true);
        self.schedule_autosave_flush(cx);
        self.sync_footer(window, cx);
        self.focus_editor(window, cx);
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
        let viewing_fragment = self.session.is_viewing_fragment();
        let theme = app_state.theme.clone();
        let day_switcher_panel = self.render_day_page_day_switcher_panel(cx);

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
        } else {
            None
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
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h(px(0.))
                    .child(editor_input)
                    .when_some(day_switcher_panel, |parent, panel| parent.child(panel)),
            );

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
                    crate::components::prompt_layout_shell::render_native_main_window_footer_hover_blocker(),
                ),
                overlays: Vec::new(),
            },
        )
    }
}

impl DayPageView {
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

        if self.is_day_switcher_open() {
            if self.handle_day_switcher_key(key, cmd, shift, alt, control, window, cx) {
                return;
            }
        }

        if exact_plain && crate::ui_foundation::is_key_escape(&key) {
            if self.session.is_viewing_fragment() {
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
            self.toggle_day_switcher(window, cx);
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
}

pub(crate) fn day_page_footer_buttons(
    app: &ScriptListApp,
    cx: Option<&gpui::App>,
) -> Vec<FooterButtonConfig> {
    let footer_disabled = crate::confirm::is_confirm_window_open();
    let actions_open = app.show_actions_popup || crate::actions::is_actions_window_open();
    let enabled = !footer_disabled;

    let primary_label = match (&app.current_view, cx) {
        (AppView::DayPage { entity }, Some(cx)) => entity.read(cx).primary_action_label(),
        _ => "Save".to_string(),
    };

    let save_enabled = enabled
        && match (&app.current_view, cx) {
            (AppView::DayPage { entity }, Some(cx)) => entity.read(cx).is_dirty(),
            _ => false,
        };

    vec![
        FooterButtonConfig::new(FooterAction::Run, "⌘S", primary_label).enabled(save_enabled),
        FooterButtonConfig::new(FooterAction::Actions, "⌘K", "Actions")
            .selected(actions_open)
            .enabled(enabled),
    ]
}

impl ScriptListApp {
    pub(crate) fn show_day_page_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_day_page_view_with_substrate(None, window, cx);
    }

    /// Binary-side body for the Notes Cmd+P "open day page" handoff.
    /// Registered with
    /// `notes::day_page_rows::register_open_day_page_in_main_hook` at startup
    /// because the dual-compiled Notes code cannot name `ScriptListApp`.
    pub(crate) fn open_day_page_in_main_window_hook(
        date: chrono::NaiveDate,
        cx: &mut gpui::App,
    ) -> bool {
        let Some(handle) = crate::get_main_window_handle() else {
            return false;
        };
        handle
            .update(cx, |any_view, window, cx| {
                let Ok(root) = any_view.downcast::<gpui_component::Root>() else {
                    return false;
                };
                let inner = root.read(cx).view().clone();
                let Ok(app) = inner.downcast::<ScriptListApp>() else {
                    return false;
                };
                app.update(cx, |app, cx| {
                    app.dispatch_window_event(
                        crate::window_orchestrator::WindowEvent::ShowMain {
                            activate_app: false,
                        },
                        cx,
                    );
                    app.show_day_page_view(window, cx);
                    if let AppView::DayPage { entity } = &app.current_view {
                        let entity = entity.clone();
                        entity.update(cx, |view, cx| view.bind_day(date, window, cx));
                    }
                });
                true
            })
            .unwrap_or(false)
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

    pub(crate) fn dispatch_day_page_save_with_footer(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if let AppView::DayPage { entity } = &self.current_view {
            let entity = entity.clone();
            entity.update(cx, |view, cx| view.save_and_sync_footer(window, cx))
        } else {
            false
        }
    }
}
