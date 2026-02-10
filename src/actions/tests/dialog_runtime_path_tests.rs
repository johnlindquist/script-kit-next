use super::dialog::{ActionsDialog, GroupedActionItem};
use super::types::{Action, ActionCallback, ActionCategory, ActionsDialogConfig, SectionStyle};
use crate::theme;
use gpui::{App, AppContext, Application, Entity};
use std::sync::{Arc, Mutex};

fn sample_action(id: &str, title: &str, section: Option<&str>) -> Action {
    let mut action = Action::new(id, title, None, ActionCategory::ScriptContext);
    if let Some(section_name) = section {
        action = action.with_section(section_name);
    }
    action
}

fn run_headless_dialog_test(test_fn: impl FnOnce(&mut App) + 'static) {
    let did_run = Arc::new(Mutex::new(false));
    let did_run_for_app = Arc::clone(&did_run);

    Application::headless().run(move |cx| {
        test_fn(cx);
        *did_run_for_app
            .lock()
            .expect("runtime dialog test run marker lock poisoned") = true;
        cx.quit();
    });

    assert!(
        *did_run
            .lock()
            .expect("runtime dialog test completion lock poisoned"),
        "headless dialog test closure did not execute"
    );
}

fn build_dialog_entity(
    cx: &mut App,
    actions: Vec<Action>,
    config: ActionsDialogConfig,
    selected_ids: Arc<Mutex<Vec<String>>>,
) -> Entity<ActionsDialog> {
    let on_select: ActionCallback = {
        let selected_ids = Arc::clone(&selected_ids);
        Arc::new(move |action_id| {
            selected_ids
                .lock()
                .expect("runtime dialog callback lock poisoned")
                .push(action_id);
        })
    };
    let theme = Arc::new(theme::Theme::default());

    cx.new(move |entity_cx| {
        ActionsDialog::with_config(entity_cx.focus_handle(), on_select, actions, theme, config)
    })
}

#[test]
fn test_submit_selected_does_emit_action_id_when_item_is_selected() {
    let selected_ids = Arc::new(Mutex::new(Vec::new()));
    let selected_ids_for_test = Arc::clone(&selected_ids);

    run_headless_dialog_test(move |cx| {
        let dialog = build_dialog_entity(
            cx,
            vec![
                sample_action("action_alpha", "Alpha", None),
                sample_action("action_beta", "Beta", None),
            ],
            ActionsDialogConfig::default(),
            Arc::clone(&selected_ids_for_test),
        );

        cx.update_entity(&dialog, |dialog, _| {
            dialog.selected_index = 1;
            dialog.submit_selected();
        });
    });

    assert_eq!(
        *selected_ids
            .lock()
            .expect("submit_selected assertion lock poisoned"),
        vec!["action_beta".to_string()]
    );
}

#[test]
fn test_submit_cancel_does_emit_cancel_sentinel_when_cancel_is_triggered() {
    let selected_ids = Arc::new(Mutex::new(Vec::new()));
    let selected_ids_for_test = Arc::clone(&selected_ids);

    run_headless_dialog_test(move |cx| {
        let dialog = build_dialog_entity(
            cx,
            vec![sample_action("action_alpha", "Alpha", None)],
            ActionsDialogConfig::default(),
            Arc::clone(&selected_ids_for_test),
        );

        cx.update_entity(&dialog, |dialog, _| {
            dialog.submit_cancel();
        });
    });

    assert_eq!(
        *selected_ids
            .lock()
            .expect("submit_cancel assertion lock poisoned"),
        vec!["__cancel__".to_string()]
    );
}

#[test]
fn test_move_navigation_does_skip_headers_when_moving_up_and_down() {
    run_headless_dialog_test(|cx| {
        let dialog = build_dialog_entity(
            cx,
            vec![
                sample_action("action_one", "One", Some("Scripts")),
                sample_action("action_two", "Two", Some("Scripts")),
                sample_action("action_three", "Three", Some("Global")),
            ],
            ActionsDialogConfig {
                section_style: SectionStyle::Headers,
                ..ActionsDialogConfig::default()
            },
            Arc::new(Mutex::new(Vec::new())),
        );

        cx.update_entity(&dialog, |dialog, entity_cx| {
            assert_eq!(dialog.grouped_items.len(), 5);
            assert!(matches!(
                dialog.grouped_items.first(),
                Some(GroupedActionItem::SectionHeader(section)) if section == "Scripts"
            ));
            assert!(matches!(
                dialog.grouped_items.get(3),
                Some(GroupedActionItem::SectionHeader(section)) if section == "Global"
            ));
            assert_eq!(dialog.selected_index, 1);

            dialog.selected_index = 2;
            dialog.move_down(entity_cx);
            assert_eq!(dialog.selected_index, 4);

            dialog.move_up(entity_cx);
            assert_eq!(dialog.selected_index, 2);

            dialog.selected_index = 1;
            dialog.move_up(entity_cx);
            assert_eq!(dialog.selected_index, 1);
        });
    });
}

#[test]
fn test_search_handlers_do_update_results_when_typing_and_backspacing() {
    run_headless_dialog_test(|cx| {
        let dialog = build_dialog_entity(
            cx,
            vec![
                sample_action("action_alpha", "Alpha", None),
                sample_action("action_beta", "Beta", None),
                sample_action("action_gamma", "Gamma", None),
            ],
            ActionsDialogConfig::default(),
            Arc::new(Mutex::new(Vec::new())),
        );

        cx.update_entity(&dialog, |dialog, entity_cx| {
            assert_eq!(dialog.search_text, "");
            assert_eq!(dialog.filtered_actions.len(), 3);

            dialog.handle_char('b', entity_cx);
            assert_eq!(dialog.search_text, "b");
            assert_eq!(dialog.filtered_actions.len(), 1);
            assert_eq!(
                dialog
                    .get_selected_action()
                    .expect("expected selected action after typing 'b'")
                    .id,
                "action_beta"
            );

            dialog.handle_char('e', entity_cx);
            assert_eq!(dialog.search_text, "be");
            assert_eq!(dialog.filtered_actions.len(), 1);

            dialog.handle_backspace(entity_cx);
            assert_eq!(dialog.search_text, "b");
            assert_eq!(dialog.filtered_actions.len(), 1);

            dialog.handle_backspace(entity_cx);
            assert_eq!(dialog.search_text, "");
            assert_eq!(dialog.filtered_actions.len(), 3);

            dialog.handle_backspace(entity_cx);
            assert_eq!(dialog.search_text, "");
            assert_eq!(dialog.filtered_actions.len(), 3);
        });
    });
}
