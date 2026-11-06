//! ⌘K in a view acts in place. What the view has focused — the selected choice or the prompt's
//! value, the file, the clipboard entry — goes in the line above the input through the reducer,
//! and the view's list becomes that thing's actions, drawn by the main window under the same
//! input from the same `ActionsDialog` the actions have always come from. Nothing opens.
//!
//! The view is left exactly as it was underneath: Esc, ⌘K again, or running an action lets go,
//! and the view draws again with its own list, selection and words.

use std::rc::Rc;

use gpui::{prelude::*, px, rgb, AnyElement, App, Context, Entity, Subscription, Window};
use sk_one_window::contract::words;
use sk_one_window::{Event, Row, Subject, SubjectKind};

use crate::actions::ActionsDialog;
use crate::components::main_view_chrome::{
    render_main_view_chrome_footer_flush, render_main_view_shell, render_prompt_search_input,
    MainViewChrome, MainViewDividerChrome, MainViewHeaderChrome, PromptSearchInputChrome,
};
use crate::components::staged_actions::{self, StagedActionRow};
use crate::{
    protocol, ActionsDialogHost, ActionsRoute, ElementCollectionOutcome, FocusedInput,
    ScriptListApp,
};

const STAGED_SEARCH_ID: &str = "staged-actions-search";

/// What a view staged: whose actions they are and the dialog they come from.
pub(crate) struct ViewStage {
    /// Captured once for the Apps catalog; never resolve its actions from the launcher index.
    pub(super) catalog_app: Option<crate::app_launcher::AppInfo>,
    host: ActionsDialogHost,
    dialog: Entity<ActionsDialog>,
    /// What ↵ runs: the highlighted action.
    enter: Option<String>,
    /// Typing filters the dialog; the reducer's rows and ↵ follow it.
    _follow: Subscription,
}

fn action_row(row: &StagedActionRow) -> Row {
    let key = format!("{}{}", super::staging::ACTION_KEY, row.id);
    let staged = Row::new(key, row.title.clone())
        .with_enter_label(row.title.clone())
        .with_icon(row.icon.clone());
    match &row.description {
        Some(sub) => staged.with_sub(sub.clone()),
        None => staged,
    }
}

fn highlighted_title(dialog: &Entity<ActionsDialog>, cx: &App) -> Option<String> {
    dialog
        .read(cx)
        .get_selected_action()
        .map(|action| action.title.clone())
}

fn staged_rows(dialog: &Entity<ActionsDialog>, cx: &App) -> Vec<Row> {
    staged_actions::staged_action_rows(dialog.read(cx))
        .iter()
        .map(action_row)
        .collect()
}

/// The icon the line shows beside what a view staged.
fn subject_icon(host: ActionsDialogHost) -> &'static str {
    match host {
        ActionsDialogHost::FileSearch | ActionsDialogHost::PathPrompt => "file",
        ActionsDialogHost::ClipboardHistory => "copy",
        ActionsDialogHost::ArgPrompt
        | ActionsDialogHost::DivPrompt
        | ActionsDialogHost::EditorPrompt
        | ActionsDialogHost::TemplatePrompt
        | ActionsDialogHost::TermPrompt
        | ActionsDialogHost::FormPrompt
        | ActionsDialogHost::ChatPrompt
        | ActionsDialogHost::WebcamPrompt => "script",
        _ => "builtin",
    }
}

impl ScriptListApp {
    /// ⌘K in a view, from the key or its capsule: `title` (or, without one, what the dialog is
    /// about) goes in the line above the input and the view's list becomes `dialog`'s actions.
    pub(crate) fn one_window_stage_in_view(
        &mut self,
        host: ActionsDialogHost,
        dialog: Entity<ActionsDialog>,
        title: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title = title
            .filter(|title| !title.trim().is_empty())
            .or_else(|| {
                dialog
                    .read(cx)
                    .current_search_placeholder()
                    .map(str::to_string)
            })
            .unwrap_or_else(|| words::FOCUSED.to_string());
        self.resync_filter_input_after_actions_if_needed(window, cx);
        self.mark_actions_popup_opening();
        self.hovered_index = None;
        self.push_focus_overlay(crate::focus_coordinator::FocusRequest::actions_dialog(), cx);
        self.focus_handle.focus(window, cx);
        self.gpui_input_focused = false;
        self.focused_input = FocusedInput::ActionsSearch;
        dialog.update(cx, |dialog, cx| dialog.stage_in_place(window, cx));
        let rows = staged_rows(&dialog, cx);
        self.actions_dialog = Some(dialog.clone());
        self.one_window_host.view_stage = Some(ViewStage {
            catalog_app: None,
            host,
            dialog: dialog.clone(),
            enter: highlighted_title(&dialog, cx),
            _follow: cx.observe(&dialog, |app, dialog, cx| {
                let enter = highlighted_title(&dialog, cx);
                if let Some(stage) = app.one_window_host.view_stage.as_mut() {
                    stage.enter = enter;
                }
                let rows = staged_rows(&dialog, cx);
                app.one_window_handle(Event::Actions(rows), cx);
            }),
        });
        self.one_window_register_in_place(dialog, cx);
        tracing::info!(
            target: "script_kit::one_window",
            event = "view_staged",
            host = ?host,
        );
        self.one_window_handle(
            Event::StageView {
                subject: Subject {
                    kind: SubjectKind::Thing,
                    label: words::ACTING_ON.into(),
                    title,
                    icon: subject_icon(host).into(),
                    ..Subject::default()
                },
                rows,
            },
            cx,
        );
    }

    /// Runs `f` in the main window once the current update is done, for a view whose ⌘K
    /// arrives as an entity event, where there is no window to stage in.
    pub(crate) fn one_window_in_window_later(
        &self,
        cx: &mut Context<Self>,
        f: impl FnOnce(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let Some(handle) = self.one_window_host.window else {
            return;
        };
        let app = cx.entity().downgrade();
        cx.defer(move |cx| {
            let Some(app) = app.upgrade() else {
                return;
            };
            let _ = cx.update_window(handle, |_, window, cx| {
                app.update(cx, |app, cx| f(app, window, cx));
            });
        });
    }

    /// Surfaces that don't own this app (Agent Chat's view, the command bars) reach what's
    /// staged through `crate::actions`: they can let go of it or hand it a key.
    fn one_window_register_in_place(&self, dialog: Entity<ActionsDialog>, cx: &Context<Self>) {
        let app = cx.entity().downgrade();
        let let_go: crate::actions::LetGo = Rc::new({
            let app = app.clone();
            move |cx: &mut App| {
                let app = app.clone();
                cx.defer(move |cx| {
                    let Some(entity) = app.upgrade() else {
                        return;
                    };
                    let Some(handle) = entity.read(cx).one_window_host.window else {
                        return;
                    };
                    let _ = cx.update_window(handle, |_, window, cx| {
                        entity.update(cx, |app, cx| {
                            app.close_actions_popup_for_current_view(window, cx)
                        });
                    });
                });
            }
        });
        let route_key: crate::actions::RouteKey = Rc::new(move |key, key_char, modifiers, cx| {
            let Some(entity) = app.upgrade() else {
                return false;
            };
            let Some(handle) = entity.read(cx).one_window_host.window else {
                return false;
            };
            cx.update_window(handle, |_, window, cx| {
                entity.update(cx, |app, cx| {
                    app.one_window_staged_key(key, key_char, modifiers, window, cx)
                })
            })
            .unwrap_or(false)
        });
        crate::actions::stage_in_place(&dialog, let_go, route_key);
    }

    /// The view is itself again: the reducer lets go and nothing else can reach the dialog.
    pub(crate) fn one_window_let_go_in_view(&mut self, cx: &mut Context<Self>) {
        let Some(stage) = self.one_window_host.view_stage.take() else {
            return;
        };
        crate::actions::clear_in_place(&stage.dialog);
        tracing::info!(
            target: "script_kit::one_window",
            event = "view_let_go",
            host = ?stage.host,
        );
        self.one_window_handle(Event::LetGo, cx);
    }

    /// What ↵ says while a view has something staged: the highlighted action.
    pub(crate) fn one_window_staged_enter_label(&self) -> Option<&str> {
        self.one_window_view_staged()?;
        self.one_window_host.view_stage.as_ref()?.enter.as_deref()
    }

    /// The dialog the main window lists in the view's place, if a view staged one.
    pub(crate) fn one_window_view_staged(
        &self,
    ) -> Option<(ActionsDialogHost, Entity<ActionsDialog>)> {
        let stage = self.one_window_host.view_stage.as_ref()?;
        let current = self.show_actions_popup
            && self
                .actions_dialog
                .as_ref()
                .is_some_and(|dialog| dialog.entity_id() == stage.dialog.entity_id());
        current.then(|| (stage.host, stage.dialog.clone()))
    }

    /// A key while the staged actions own the list: the same routing the dialog always had.
    /// Returns whether the key was used.
    pub(crate) fn one_window_staged_key(
        &mut self,
        key: &str,
        key_char: Option<&str>,
        modifiers: &gpui::Modifiers,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some((host, _)) = self.one_window_view_staged() else {
            return false;
        };
        match self.route_key_to_actions_dialog(key, key_char, modifiers, host, window, cx) {
            ActionsRoute::Execute {
                action_id,
                should_close,
            } => {
                self.execute_actions_route_action(host, action_id, should_close, window, cx);
                true
            }
            ActionsRoute::Handled => true,
            ActionsRoute::NotHandled => false,
        }
    }

    /// A click on a staged row runs it the way Enter does.
    fn one_window_run_staged_action(
        &mut self,
        action_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((host, dialog)) = self.one_window_view_staged() else {
            return;
        };
        let activation = dialog.update(cx, |dialog, cx| dialog.activate_action_id(action_id, cx));
        self.handle_actions_dialog_activation(host, activation, window, cx);
    }

    /// While a view has something staged, the main window draws its actions in the view's
    /// place; otherwise `view` as it is.
    pub(crate) fn one_window_staged_view(
        &mut self,
        view: AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.one_window_host.view_stage.is_none() {
            return view;
        }
        let Some((_, dialog)) = self.one_window_view_staged() else {
            self.one_window_let_go_in_view(cx);
            return view;
        };
        let theme = self.theme.clone();
        let menu_def = self.current_main_menu_theme.def();
        let search = {
            let dialog = dialog.read(cx);
            match dialog.search_input.clone() {
                Some(input) => PromptSearchInputChrome::entity_backed(
                    gpui_component::input::Input::new(&input),
                ),
                None => PromptSearchInputChrome::controller_owned(
                    STAGED_SEARCH_ID,
                    dialog.search_text.clone(),
                    dialog
                        .current_search_placeholder()
                        .unwrap_or_default()
                        .to_string(),
                ),
            }
        };
        let header = MainViewHeaderChrome::canonical(
            menu_def,
            self.render_main_view_slot_line(menu_def),
            render_prompt_search_input(&theme, menu_def, search),
        );
        let head = self
            .one_window
            .staged()
            .map_or_else(|| words::STAGED.to_string(), |staged| staged.head.clone());
        let app = cx.entity().downgrade();
        let list = staged_actions::render_staged_actions(
            &dialog,
            &head,
            &theme,
            move |action_id, window, cx| {
                let _ = app.update(cx, |app, cx| {
                    app.one_window_run_staged_action(action_id, window, cx)
                });
            },
            cx,
        );
        let footer = self.main_window_footer_slot(crate::components::render_simple_hint_strip(
            crate::components::universal_prompt_hints(),
            None,
        ));
        let handle_key = cx.listener(
            |this: &mut Self, event: &gpui::KeyDownEvent, window: &mut Window, cx| {
                let used = this.one_window_staged_key(
                    event.keystroke.key.as_str(),
                    event.keystroke.key_char.as_deref(),
                    &event.keystroke.modifiers,
                    window,
                    cx,
                );
                if used {
                    cx.stop_propagation();
                }
            },
        );
        render_main_view_chrome_footer_flush(
            render_main_view_shell()
                .text_color(rgb(theme.colors.text.primary))
                .key_context("staged_actions")
                .track_focus(&self.focus_handle)
                .capture_key_down(handle_key),
            &theme,
            menu_def,
            MainViewChrome {
                header,
                divider: MainViewDividerChrome {
                    margin_x: menu_def.shell.divider_margin_x,
                    height: menu_def.shell.divider_height,
                    visible: false,
                },
                main: gpui::div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .child(list)
                    .into_any_element(),
                footer,
                overlays: Vec::new(),
            },
        )
    }

    /// What `getElements` sees while a view has something staged: the line, the search, the
    /// header and the rows, read from the same dialog the list draws.
    pub(crate) fn one_window_view_stage_elements(
        &self,
        limit: usize,
        include_headers: bool,
        cx: &Context<Self>,
    ) -> Option<ElementCollectionOutcome> {
        let (_, dialog) = self.one_window_view_staged()?;
        let dialog = dialog.read(cx);
        let rows = staged_actions::staged_action_rows(dialog);
        let mut elements = self.main_view_slot_line_elements();
        elements.push(protocol::ElementInfo::input(
            STAGED_SEARCH_ID,
            Some(dialog.search_text.as_str()),
            true,
        ));
        elements.push(protocol::ElementInfo::list(
            staged_actions::STAGED_ACTIONS_ID,
            rows.len(),
        ));
        if include_headers {
            let head = self
                .one_window
                .staged()
                .map_or_else(|| words::STAGED.to_string(), |staged| staged.head.clone());
            let mut header =
                protocol::ElementInfo::panel(&protocol::generate_semantic_id("section", 0, &head));
            header.text = Some(head.clone());
            header.value = Some(head);
            header.role = Some("sectionHeader".to_string());
            header.kind = Some("sectionHeader".to_string());
            header.selectable = Some(false);
            elements.push(header);
        }
        for (index, row) in rows.iter().enumerate() {
            let mut element = protocol::ElementInfo::product_static_choice(
                index,
                &row.title,
                &row.id,
                row.selected,
            );
            element.semantic_id = staged_actions::staged_action_semantic_id(&row.id);
            element.role = Some("row".to_string());
            element.kind = Some("stagedAction".to_string());
            element.source = Some("ActionsDialog".to_string());
            element.source_name = row.description.clone();
            elements.push(element);
        }
        let total = elements.len();
        elements.truncate(limit);
        Some(ElementCollectionOutcome::complete(
            "stagedActions",
            elements,
            total,
        ))
    }
}
