// Day Page surface entry, render host, and footer helpers.

use std::time::Duration;

use chrono::Utc;

use crate::components::notes_editor::{NotesEditorConfig, NotesEditorLayout};
use crate::components::unified_list_item::{
    Density, ItemState, TextContent, TrailingContent, UnifiedListItem, UnifiedListItemColors,
};
use crate::footer_popup::{FooterAction, FooterButtonConfig};
use crate::ui_foundation::HexColorExt;
use script_kit_gpui::brain::substrate::BrainSubstrate;
use script_kit_gpui::day_page::{
    parse_day_page_segments, resolve_fragment_path, DayPageBinding, DayPageSegment,
    SEDIMENT_LAYER_ID, SEDIMENT_LINE_HEIGHT,
};

impl DayPageView {
    pub fn new(
        app: Entity<ScriptListApp>,
        substrate: BrainSubstrate,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let metrics = crate::notes::window::style::adopted_metrics();
        let editor_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("markdown")
                .code_editor_dynamic_bottom_margin(false)
                .line_number(false)
                .searchable(true)
                .auto_grow(20, 500)
                .placeholder("Today...")
                .default_value("")
        });
        let notes_editor = cx.new(|_| {
            NotesEditor::new(
                editor_state.clone(),
                NotesEditorConfig::new("")
                    .placeholder("Today...")
                    .layout(NotesEditorLayout::new(
                        metrics.editor_padding_x,
                        metrics.editor_padding_y,
                    )),
            )
        });

        let editor_subscription = cx.subscribe_in(&editor_state, window, {
            let view = cx.entity().downgrade();
            move |_, _, event: &InputEvent, window, cx| {
                if !matches!(event, InputEvent::Change) {
                    return;
                }
                if let Some(view) = view.upgrade() {
                    view.update(cx, |this, cx| {
                        this.on_editor_change(window, cx);
                    });
                }
            }
        });

        Self {
            app: app.downgrade(),
            session: DayPageDocumentSession::new(substrate),
            notes_editor,
            editor_state,
            editor_subscription,
            focus_handle: cx.focus_handle(),
            fragment_open_targets: Vec::new(),
            dictation_chrome: DayPageDictationChrome::Hidden,
            pending_dictation_commit: None,
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
        self.refresh_fragment_open_targets(&content);
        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value(content, window, cx);
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

    pub fn open_fragment_at(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
        let content = self.notes_editor.read(cx).content(cx);
        self.session.apply_editor_content(&content);
        self.refresh_fragment_open_targets(&content);
        self.poll_external_disk_changes(window, cx);
        self.sync_footer(window, cx);
        cx.notify();
    }

    pub fn poll_external_disk_changes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Ok(Some(content)) = self.session.maybe_refresh_from_disk() {
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
            Ok(()) => true,
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

    pub fn focus_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.notes_editor.update(cx, |editor, cx| {
            editor.focus(window, cx);
        });
    }

    pub(crate) fn set_dictation_chrome(&mut self, chrome: DayPageDictationChrome, cx: &mut Context<Self>) {
        self.dictation_chrome = chrome;
        cx.notify();
    }

    pub(crate) fn clear_dictation_chrome(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.dictation_chrome, DayPageDictationChrome::Hidden) {
            self.dictation_chrome = DayPageDictationChrome::Hidden;
            cx.notify();
        }
    }

    pub(crate) fn stage_dictation_commit(
        &mut self,
        updated_content: String,
        caret_offset: usize,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = self
            .session
            .adopt_disk_content_after_external_write(updated_content.clone())
        {
            tracing::error!(error = %error, "Failed to sync day page session after dictation");
        }
        self.refresh_fragment_open_targets(&updated_content);
        self.pending_dictation_commit = Some((updated_content, caret_offset));
        self.dictation_chrome = DayPageDictationChrome::Hidden;
        cx.notify();
    }

    fn apply_pending_dictation_commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((content, caret)) = self.pending_dictation_commit.take() else {
            return;
        };
        let caret = caret.min(content.len());
        self.notes_editor.update(cx, |editor, cx| {
            editor.set_value(content, window, cx);
            editor.set_selection(caret, caret, window, cx);
        });
        self.focus_editor(window, cx);
        self.sync_footer(window, cx);
        cx.notify();
    }

    fn refresh_dictation_listening_chrome(&mut self, cx: &mut Context<Self>) {
        let DayPageDictationChrome::Listening { display_bars } = self.dictation_chrome.clone() else {
            return;
        };
        let Some(state) = crate::dictation::snapshot_overlay_state() else {
            self.dictation_chrome = DayPageDictationChrome::Hidden;
            cx.notify();
            return;
        };
        let next_bars = crate::dictation::animate_bars(
            display_bars,
            state.bars,
            Duration::from_millis(16),
        );
        if next_bars != display_bars {
            self.dictation_chrome = DayPageDictationChrome::Listening {
                display_bars: next_bars,
            };
            cx.notify();
        }
    }

    fn sync_footer(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(app) = self.app.upgrade() {
            app.update(cx, |app, cx| {
                app.sync_main_footer_popup(window, cx);
            });
        }
    }
}

impl Focusable for DayPageView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DayPageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.refresh_dictation_listening_chrome(cx);
        self.apply_pending_dictation_commit(window, cx);
        self.poll_external_disk_changes(window, cx);

        let app = self
            .app
            .upgrade()
            .expect("DayPageView app entity dropped");

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
        let content = self.notes_editor.read(cx).content(cx);
        let segments = parse_day_page_segments(&content);
        let has_sediment = segments.iter().any(|segment| {
            matches!(
                segment,
                DayPageSegment::FragmentRef { .. } | DayPageSegment::KeptUrl { .. }
            )
        });
        let viewing_fragment = self.session.is_viewing_fragment();
        let editor_metrics = crate::notes::window::style::adopted_metrics();
        let list_colors = UnifiedListItemColors::from_theme(&app_state.theme);
        let editor_bg = app_state.theme.colors.background.main;
        let accent_color = app_state.theme.colors.accent.selected;
        let theme = app_state.theme.clone();
        let editor_padding_y = editor_metrics.editor_padding_y;
        drop(app_state);

        let back_bar = if viewing_fragment {
            let label = match self.session.binding() {
                DayPageBinding::Fragment { return_day_date, .. } => {
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
        } else {
            None
        };

        let sediment_layer = if has_sediment && !viewing_fragment {
            let day_path = self.session.path().cloned();
            let tz = self.session.substrate().timezone();
            let mut layer = div()
                .id(SEDIMENT_LAYER_ID)
                .absolute()
                .inset_0()
                .overflow_hidden();

            for segment in segments {
                match segment {
                    DayPageSegment::FragmentRef {
                        excerpt,
                        relative_link,
                        start_line,
                        line_count,
                        index,
                        ..
                    } => {
                        let subtitle = day_path
                            .as_ref()
                            .and_then(|day| resolve_fragment_path(day, &relative_link))
                            .and_then(|path| {
                                script_kit_gpui::day_page::load_fragment_provenance(&path)
                            })
                            .map(|meta| {
                                script_kit_gpui::day_page::format_provenance_hint(&meta, tz)
                            })
                            .unwrap_or_else(|| "Fragment".to_string());

                        let card = UnifiedListItem::new(
                            script_kit_gpui::day_page::fragment_card_id(index),
                            TextContent::plain(excerpt),
                        )
                        .subtitle(TextContent::plain(subtitle))
                        .trailing(TrailingContent::Chevron)
                        .density(Density::Comfortable)
                        .colors(list_colors)
                        .state(ItemState {
                            is_hovered: false,
                            is_selected: false,
                            is_disabled: false,
                        })
                        .with_direct_hover(true);

                        let top = editor_padding_y + (start_line as f32) * SEDIMENT_LINE_HEIGHT;
                        let height = (line_count as f32) * SEDIMENT_LINE_HEIGHT;

                        layer = layer.child(
                            div()
                                .absolute()
                                .left(px(0.))
                                .right(px(0.))
                                .top(px(top))
                                .h(px(height))
                                .bg(rgba((editor_bg << 8) | 0xFF))
                                .occlude()
                                .cursor_pointer()
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, window, cx| {
                                        this.open_fragment_at(index, window, cx);
                                    }),
                                )
                                .child(card),
                        );
                    }
                    DayPageSegment::KeptUrl {
                        url,
                        start_line,
                        index,
                        ..
                    } => {
                        let accent = accent_color;
                        let top = editor_padding_y + (start_line as f32) * SEDIMENT_LINE_HEIGHT;
                        layer = layer.child(
                            div()
                                .id(script_kit_gpui::day_page::kept_url_id(index))
                                .absolute()
                                .left(px(0.))
                                .right(px(0.))
                                .top(px(top))
                                .h(px(SEDIMENT_LINE_HEIGHT))
                                .bg(rgba((editor_bg << 8) | 0xFF))
                                .occlude()
                                .flex()
                                .items_center()
                                .px(px(12.))
                                .text_sm()
                                .text_color(rgb(accent))
                                .child(url),
                        );
                    }
                    DayPageSegment::Plain { .. } => {}
                }
            }

            Some(layer.into_any_element())
        } else {
            None
        };

        let dictation_chrome = self.render_dictation_chrome(&theme, cx);

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
            .when_some(dictation_chrome, |parent, chrome| parent.child(chrome))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h(px(0.))
                    .when_some(sediment_layer, |parent, layer| parent.child(layer))
                    .child(editor_input),
            );

        let context_zone = app.update(cx, |app, cx| {
            app.render_clickable_main_view_context_zone(menu_def, cx)
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
    fn render_dictation_chrome(
        &self,
        theme: &crate::theme::Theme,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        match &self.dictation_chrome {
            DayPageDictationChrome::Hidden => None,
            DayPageDictationChrome::Listening { display_bars } => {
                let success = theme.colors.ui.success;
                let hint = theme.colors.text.secondary;
                let opacity = theme.get_opacity().text_hint;
                let active = display_bars.iter().any(|bar| *bar > 0.05);
                let bar_color = if active {
                    success.with_opacity(1.0)
                } else {
                    theme.colors.text.primary.with_opacity(opacity)
                };

                let mut bars = div().flex().items_center().gap(px(2.));
                for &level in display_bars {
                    let height = px(4. + level * 10.);
                    bars = bars.child(
                        div()
                            .w(px(2.))
                            .h(height)
                            .min_h(px(3.))
                            .bg(bar_color)
                            .rounded(px(1.)),
                    );
                }

                Some(
                    div()
                        .id(DAY_PAGE_DICTATION_LISTENING_ID)
                        .w_full()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .pb(px(6.))
                        .text_sm()
                        .text_color(rgb(hint))
                        .child("Listening…")
                        .child(bars)
                        .into_any_element(),
                )
            }
            DayPageDictationChrome::Transcribing => {
                let hint = theme.colors.text.secondary;
                Some(
                    div()
                        .id(DAY_PAGE_DICTATION_LISTENING_ID)
                        .w_full()
                        .pb(px(6.))
                        .text_sm()
                        .text_color(rgb(hint))
                        .child("Transcribing…")
                        .into_any_element(),
                )
            }
            DayPageDictationChrome::Unavailable { message } => {
                let hint = theme.colors.text.secondary;
                let accent = theme.colors.accent.selected;
                Some(
                    div()
                        .id(DAY_PAGE_DICTATION_UNAVAILABLE_ID)
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .pb(px(6.))
                        .text_sm()
                        .text_color(rgb(hint))
                        .child(message.clone())
                        .child(
                            div()
                                .text_color(rgb(accent))
                                .cursor_pointer()
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(|this, _, _window, cx| {
                                        if let Some(app) = this.app.upgrade() {
                                            app.update(cx, |app, cx| {
                                                app.open_dictation_model_prompt(cx);
                                            });
                                        }
                                    }),
                                )
                                .child("Open Dictation Setup"),
                        )
                        .into_any_element(),
                )
            }
        }
    }

    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.to_lowercase();
        let cmd = event.keystroke.modifiers.platform;

        if crate::ui_foundation::is_key_escape(&key) {
            if self.session.is_viewing_fragment() {
                self.return_to_day_page(window, cx);
                return;
            }
            if let Some(app) = self.app.upgrade() {
                app.update(cx, |app, cx| {
                    app.close_and_reset_window(cx);
                });
            }
            return;
        }

        if cmd && key == "s" {
            self.save_and_sync_footer(window, cx);
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
        FooterButtonConfig::new(FooterAction::Ai, "⌘↵", "Agent").enabled(enabled),
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
