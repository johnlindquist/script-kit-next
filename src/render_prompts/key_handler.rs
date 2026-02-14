#[derive(Clone, Copy)]
struct PromptKeyPreambleCfg {
    is_dismissable: bool,
    stop_propagation_on_global_shortcut: bool,
    stop_propagation_when_handled: bool,
    host: ActionsDialogHost,
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn handle_prompt_key_preamble<
    PreHandle,
    TogglePredicate,
    OnToggleActions,
    OnActionsDialogExecute,
    AllowSdkShortcuts,
    OnSdkShortcut,
>(
    app: &mut ScriptListApp,
    event: &gpui::KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<ScriptListApp>,
    cfg: PromptKeyPreambleCfg,
    mut pre_handle: PreHandle,
    mut toggle_predicate: TogglePredicate,
    mut on_toggle_actions: OnToggleActions,
    mut on_actions_dialog_execute: OnActionsDialogExecute,
    mut allow_sdk_shortcuts: AllowSdkShortcuts,
    mut on_sdk_shortcut: OnSdkShortcut,
) -> bool
where
    PreHandle: FnMut(
        &mut ScriptListApp,
        &gpui::KeyDownEvent,
        &mut Window,
        &mut Context<ScriptListApp>,
    ) -> bool,
    TogglePredicate: FnMut(&str, Option<&str>, &gpui::Modifiers) -> bool,
    OnToggleActions: FnMut(&mut ScriptListApp, &mut Window, &mut Context<ScriptListApp>),
    OnActionsDialogExecute: FnMut(&mut ScriptListApp, &str, &mut Context<ScriptListApp>),
    AllowSdkShortcuts: FnMut(&str, Option<&str>, &gpui::Modifiers) -> bool,
    OnSdkShortcut: FnMut(&mut ScriptListApp, &SdkActionShortcutMatch, &mut Context<ScriptListApp>),
{
    let stop_if_configured = |cx: &mut Context<ScriptListApp>| {
        if cfg.stop_propagation_when_handled {
            cx.stop_propagation();
        }
    };

    app.hide_mouse_cursor(cx);

    if key_preamble(
        app,
        event,
        cfg.is_dismissable,
        cfg.stop_propagation_on_global_shortcut,
        cx,
    ) {
        stop_if_configured(cx);
        return true;
    }

    if pre_handle(app, event, window, cx) {
        stop_if_configured(cx);
        return true;
    }

    let key = event.keystroke.key.as_str();
    let key_char = event.keystroke.key_char.as_deref();
    let modifiers = &event.keystroke.modifiers;

    if toggle_predicate(key, key_char, modifiers) {
        on_toggle_actions(app, window, cx);
        stop_if_configured(cx);
        return true;
    }

    match app.route_key_to_actions_dialog(key, key_char, modifiers, cfg.host, window, cx) {
        ActionsRoute::Execute { action_id } => {
            on_actions_dialog_execute(app, &action_id, cx);
            stop_if_configured(cx);
            return true;
        }
        ActionsRoute::Handled => {
            stop_if_configured(cx);
            return true;
        }
        ActionsRoute::NotHandled => {}
    }

    if allow_sdk_shortcuts(key, key_char, modifiers) {
        let key_lower = key.to_lowercase();
        if let Some(matched_shortcut) =
            check_sdk_action_shortcut(&app.action_shortcuts, &key_lower, modifiers)
        {
            on_sdk_shortcut(app, &matched_shortcut, cx);
            stop_if_configured(cx);
            return true;
        }
    }

    false
}

#[inline]
pub(crate) fn handle_prompt_key_preamble_default(
    app: &mut ScriptListApp,
    event: &gpui::KeyDownEvent,
    window: &mut Window,
    cx: &mut Context<ScriptListApp>,
    cfg: PromptKeyPreambleCfg,
    has_actions: bool,
    prompt_label: &'static str,
) -> bool {
    handle_prompt_key_preamble(
        app,
        event,
        window,
        cx,
        cfg,
        |_, _, _, _| false,
        |key, _, modifiers| modifiers.platform && ui_foundation::is_key_k(key) && has_actions,
        |app, window, cx| {
            logging::log(
                "KEY",
                &format!("Cmd+K in {prompt_label} - calling toggle_arg_actions"),
            );
            app.toggle_arg_actions(cx, window);
        },
        |app, action_id, cx| {
            app.trigger_action_by_name(action_id, cx);
        },
        |_, _, _| true,
        |app, matched_shortcut, cx| {
            logging::log(
                "KEY",
                &format!(
                    "SDK action shortcut matched in {prompt_label}: {}",
                    matched_shortcut.action_name
                ),
            );
            app.trigger_action_by_name(&matched_shortcut.action_name, cx);
        },
    )
}
