use std::rc::Rc;

use gpui::{
    px, App, AppContext as _, ClickEvent, ParentElement as _, Pixels, SharedString, Styled as _,
    WeakEntity, Window,
};
use gpui_component::{
    button::ButtonVariant, dialog::DialogButtonProps, ActiveTheme as _, WindowExt as _,
};

type ConfirmCallback = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
pub(crate) struct ParentConfirmOptions {
    pub title: SharedString,
    pub body: SharedString,
    pub confirm_text: SharedString,
    pub cancel_text: SharedString,
    pub confirm_variant: ButtonVariant,
    pub width: Pixels,
}

impl Default for ParentConfirmOptions {
    fn default() -> Self {
        Self {
            title: "Confirm".into(),
            body: "".into(),
            confirm_text: "OK".into(),
            cancel_text: "Cancel".into(),
            confirm_variant: ButtonVariant::Primary,
            width: px(448.),
        }
    }
}

impl ParentConfirmOptions {
    #[allow(dead_code)]
    pub(crate) fn destructive(
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
        confirm_text: impl Into<SharedString>,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            confirm_text: confirm_text.into(),
            cancel_text: "Cancel".into(),
            confirm_variant: ButtonVariant::Danger,
            width: px(448.),
        }
    }
}

// Used by include!() code in app_actions/handle_action/scripts.rs — clippy
// cannot trace through include!() and reports a false-positive dead_code lint.
#[allow(dead_code)]
pub(crate) fn open_parent_confirm_dialog(
    window: &mut Window,
    cx: &mut App,
    options: ParentConfirmOptions,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&mut Window, &mut App) + 'static,
) {
    open_parent_confirm_dialog_with_lifecycle(window, cx, options, || true, on_confirm, on_cancel);
}

#[allow(dead_code)]
pub(crate) fn open_parent_confirm_dialog_for_entity<T: 'static>(
    window: &mut Window,
    cx: &mut App,
    owner: WeakEntity<T>,
    options: ParentConfirmOptions,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&mut Window, &mut App) + 'static,
) {
    let owner_for_lifecycle = owner.clone();
    open_parent_confirm_dialog_with_lifecycle(
        window,
        cx,
        options,
        move || owner_for_lifecycle.upgrade().is_some(),
        on_confirm,
        on_cancel,
    );
}

pub(crate) fn open_parent_confirm_dialog_with_lifecycle(
    window: &mut Window,
    cx: &mut App,
    options: ParentConfirmOptions,
    keep_open_while: impl Fn() -> bool + 'static,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&mut Window, &mut App) + 'static,
) {
    let has_lifecycle_predicate = true; // keep_open_while is always provided
    tracing::info!(
        event = "parent_confirm_dialog_opened",
        title = %options.title,
        has_lifecycle_predicate,
        "parent_confirm_dialog_opened"
    );

    window.activate_window();

    let keep_open_while: Rc<dyn Fn() -> bool> = Rc::new(keep_open_while);
    let on_confirm: ConfirmCallback = Rc::new(on_confirm);
    let on_cancel: ConfirmCallback = Rc::new(on_cancel);

    window.open_dialog(cx, move |dialog, _window, cx| {
        let on_confirm = on_confirm.clone();
        let on_cancel = on_cancel.clone();
        let keep_open_while = keep_open_while.clone();

        let ParentConfirmOptions {
            title,
            body,
            confirm_text,
            cancel_text,
            confirm_variant,
            width,
        } = options.clone();

        let width_value: f32 = width.into();
        tracing::info!(
            event = "parent_confirm_dialog_building",
            title = %title,
            width = width_value,
            "parent_confirm_dialog_building"
        );

        let vibrancy_bg = crate::ui_foundation::get_vibrancy_surface_background(0.65);

        dialog
            .rounded_lg()
            .w(width)
            .bg(vibrancy_bg)
            .confirm()
            .title(title)
            .button_props(
                DialogButtonProps::default()
                    .cancel_text(cancel_text)
                    .cancel_variant(ButtonVariant::Secondary)
                    .ok_text(confirm_text)
                    .ok_variant(confirm_variant),
            )
            .child(
                gpui::div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(body),
            )
            .keep_open_while(move || (keep_open_while)())
            .on_ok(move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                on_confirm(window, cx);
                true
            })
            .on_cancel(move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                on_cancel(window, cx);
                true
            })
    });
}

/// Open a confirmation modal as an in-window parent dialog and return whether
/// the user confirmed.  Uses the global main window handle to open the dialog
/// from async contexts that only have `&mut AsyncApp`.
///
/// Returns `Ok(true)` if the user clicks confirm, `Ok(false)` if they cancel
/// or close the dialog, and `Err` if the dialog could not be opened.
#[allow(dead_code)]
pub(crate) async fn confirm_with_parent_dialog(
    cx: &mut gpui::AsyncApp,
    options: ParentConfirmOptions,
    trace_id: &str,
) -> anyhow::Result<bool> {
    tracing::info!(
        category = "UI",
        trace_id = %trace_id,
        event = "confirm_modal_open",
        title = %options.title,
        "Opening confirmation modal"
    );

    let (confirm_tx, confirm_rx) = async_channel::bounded::<bool>(1);

    let window_handle = crate::get_main_window_handle()
        .ok_or_else(|| anyhow::anyhow!("Main window handle not available"))?;

    let sender_ok = confirm_tx.clone();
    let sender_cancel = confirm_tx.clone();

    cx.update_window(window_handle, move |_, window, cx| {
        open_parent_confirm_dialog(
            window,
            cx,
            options,
            move |_window, _cx| {
                let _ = sender_ok.try_send(true);
            },
            move |_window, _cx| {
                let _ = sender_cancel.try_send(false);
            },
        );
    })?;

    let confirmed = confirm_rx.recv().await.unwrap_or(false);
    tracing::info!(
        category = "UI",
        trace_id = %trace_id,
        event = "confirm_modal_result",
        confirmed,
        "Confirmation modal resolved"
    );
    Ok(confirmed)
}

#[cfg(test)]
mod tests {
    use std::fs;

    fn normalize_ws(source: &str) -> String {
        source.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn parent_confirm_dialog_activates_window_before_opening() {
        let source = fs::read_to_string("src/confirm/parent_dialog.rs")
            .expect("Failed to read src/confirm/parent_dialog.rs");
        let normalized = normalize_ws(&source);

        let activate_idx = normalized
            .find("window.activate_window();")
            .expect("parent confirm dialog should activate the parent window");
        let open_idx = normalized
            .find("window.open_dialog(cx, move |dialog, _window, cx|")
            .expect("parent confirm dialog should still open an in-window dialog");

        assert!(
            activate_idx < open_idx,
            "parent confirm dialog should activate the window before opening the dialog"
        );
    }

    #[test]
    fn async_confirmation_callers_use_shared_parent_dialog_helper() {
        let clipboard_source = fs::read_to_string("src/app_actions/handle_action/clipboard.rs")
            .expect("Failed to read clipboard.rs");
        let chat_source = fs::read_to_string("src/app_impl/chat_actions.rs")
            .expect("Failed to read chat_actions.rs");
        let execution_source = fs::read_to_string("src/app_impl/execution_paths.rs")
            .expect("Failed to read execution_paths.rs");
        let builtin_source = fs::read_to_string("src/app_execute/builtin_execution.rs")
            .expect("Failed to read builtin_execution.rs");

        let clipboard = normalize_ws(&clipboard_source);
        let chat = normalize_ws(&chat_source);
        let execution = normalize_ws(&execution_source);
        let builtin = normalize_ws(&builtin_source);

        assert!(
            clipboard.contains("crate::confirm::confirm_with_parent_dialog(")
                && !clipboard.contains("confirm_with_modal("),
            "clipboard.rs should call confirm_with_parent_dialog directly, not confirm_with_modal"
        );

        assert!(
            chat.contains("crate::confirm::confirm_with_parent_dialog(")
                && chat.contains("ParentConfirmOptions::destructive(")
                && chat.contains("\"Clear Conversation\"")
                && chat.contains("\"Clear\"")
                && !chat.contains("async_channel::bounded::<bool>(1)"),
            "chat_actions should use the shared destructive async confirm helper"
        );

        assert!(
            execution.contains("crate::confirm::confirm_with_parent_dialog(")
                && execution.contains("ParentConfirmOptions::destructive(")
                && execution.contains("\"Move to Trash\"")
                && !execution.contains("async_channel::bounded::<bool>(1)"),
            "execution_paths should use the shared destructive async confirm helper"
        );

        assert!(
            builtin.contains("crate::confirm::confirm_with_parent_dialog(")
                && !builtin.contains("confirm_with_modal("),
            "builtin_execution.rs should call confirm_with_parent_dialog directly, not confirm_with_modal"
        );
    }

    #[test]
    fn parent_confirm_dialog_logs_build_width_before_render() {
        let source = fs::read_to_string("src/confirm/parent_dialog.rs")
            .expect("Failed to read src/confirm/parent_dialog.rs");
        let normalized = normalize_ws(&source);

        assert!(
            normalized.contains("event = \"parent_confirm_dialog_building\"")
                && normalized.contains("width = width_value,"),
            "parent confirm dialog should log the concrete width used to build the dialog"
        );
    }

    fn extract_section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
        source
            .split(start)
            .nth(1)
            .and_then(|section| section.split(end).next())
            .expect("expected section to exist")
    }

    #[test]
    fn script_list_confirm_actions_bind_dialog_lifecycle_to_script_list_entity() {
        let source = fs::read_to_string("src/app_actions/handle_action/scripts.rs")
            .expect("Failed to read scripts.rs");

        let remove_branch = normalize_ws(extract_section(
            &source,
            "\"remove_script\" | \"delete_script\" => {",
            "\"reload_scripts\" => {",
        ));
        assert!(
            remove_branch.contains("let weak_entity = cx.entity().downgrade();")
                && remove_branch.contains(
                    "crate::confirm::open_parent_confirm_dialog_for_entity("
                )
                && remove_branch.contains("this.refresh_scripts(cx);")
                && !remove_branch.contains("crate::confirm::open_parent_confirm_dialog("),
            "script removal should use the entity-owned parent confirm helper and keep refreshing the list after confirmation"
        );

        let quit_branch = normalize_ws(extract_section(
            &source,
            "\"quit\" => {",
            "\"copy_content\" => {",
        ));
        assert!(
            quit_branch.contains("let owner = cx.entity().downgrade();")
                && quit_branch.contains(
                    "crate::confirm::open_parent_confirm_dialog_for_entity("
                )
                && quit_branch.contains("Self::quit_script_kit_confirm_options()")
                && quit_branch.contains("Self::prepare_script_kit_shutdown();")
                && !quit_branch.contains("crate::confirm::open_parent_confirm_dialog("),
            "quit should use the entity-owned parent confirm helper while preserving the shared copy and shutdown cleanup"
        );
    }

    #[test]
    fn prompt_handler_confirm_uses_shared_async_confirm_helper() {
        let source = fs::read_to_string("src/prompt_handler/mod.rs")
            .expect("Failed to read src/prompt_handler/mod.rs");
        let normalized = normalize_ws(&source);

        assert!(
            normalized.contains("crate::confirm::confirm_with_parent_dialog(")
                && !normalized.contains("crate::confirm::open_parent_confirm_dialog("),
            "prompt_handler confirm should delegate to the shared async confirm helper"
        );
    }

    #[test]
    fn quit_action_and_builtin_quit_share_shutdown_cleanup() {
        let scripts_source = fs::read_to_string("src/app_actions/handle_action/scripts.rs")
            .expect("Failed to read scripts.rs");
        let builtin_source = fs::read_to_string("src/app_execute/builtin_execution.rs")
            .expect("Failed to read builtin_execution.rs");

        let scripts = normalize_ws(&scripts_source);
        let builtins = normalize_ws(&builtin_source);

        assert!(
            scripts.contains("Self::quit_script_kit_confirm_options()"),
            "direct quit action should keep using the shared quit confirm copy"
        );

        let helper_idx = builtins
            .find("fn prepare_script_kit_shutdown()")
            .expect("Expected shared quit shutdown helper");
        let kill_idx = builtins
            .find("PROCESS_MANAGER.kill_all_processes();")
            .expect("Expected helper to stop running processes");
        let remove_pid_idx = builtins
            .find("PROCESS_MANAGER.remove_main_pid();")
            .expect("Expected helper to clear the main pid");

        assert!(
            helper_idx < kill_idx && kill_idx < remove_pid_idx,
            "shared quit shutdown helper should stop processes before clearing the main pid"
        );

        let action_helper_idx = scripts
            .find("Self::prepare_script_kit_shutdown();")
            .expect("Expected direct quit action to use the shared shutdown helper");
        let action_quit_idx = scripts[action_helper_idx..]
            .find("cx.quit();")
            .map(|idx| action_helper_idx + idx)
            .expect("Expected direct quit action to quit after shutdown cleanup");

        assert!(
            action_helper_idx < action_quit_idx,
            "direct quit action should run shutdown cleanup before quitting"
        );

        let builtin_branch_idx = builtins
            .find("SystemActionType::QuitScriptKit => {")
            .expect("Expected builtin quit system action branch");
        let builtin_helper_idx = builtins[builtin_branch_idx..]
            .find("Self::prepare_script_kit_shutdown();")
            .map(|idx| builtin_branch_idx + idx)
            .expect("Expected builtin quit to use the shared shutdown helper");
        let builtin_quit_idx = builtins[builtin_helper_idx..]
            .find("cx.quit();")
            .map(|idx| builtin_helper_idx + idx)
            .expect("Expected builtin quit to call cx.quit() after cleanup");

        assert!(
            builtin_helper_idx < builtin_quit_idx,
            "builtin quit should run shutdown cleanup before quitting"
        );
    }

    #[test]
    fn no_legacy_confirmation_surfaces_exist_outside_shared_helper() {
        use std::path::{Path, PathBuf};

        fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
            for entry in fs::read_dir(dir).expect("read_dir failed") {
                let entry = entry.expect("dir entry failed");
                let path = entry.path();

                if path.is_dir() {
                    let name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default();
                    if matches!(name, "target" | "vendor" | ".git") {
                        continue;
                    }
                    collect_rs_files(&path, out);
                    continue;
                }

                if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                    out.push(path);
                }
            }
        }

        let mut files = Vec::new();
        collect_rs_files(Path::new("src"), &mut files);

        let allow_inline_confirm = [Path::new("src/confirm/parent_dialog.rs")];

        let offenders: Vec<String> = files
            .into_iter()
            .filter(|path| !allow_inline_confirm.iter().any(|allowed| path == allowed))
            .filter_map(|path| {
                let source = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
                // Only scan production code — strip everything after #[cfg(test)]
                let production_source = source.split("#[cfg(test)]").next().unwrap_or(&source);
                let normalized = normalize_ws(production_source);

                let uses_legacy_helper = normalized.contains("confirm_with_modal(")
                    || normalized.contains("open_confirm_window(");

                let inlines_confirm_dialog = normalized
                    .contains("window.open_dialog(cx, move |dialog")
                    && normalized.contains(".confirm()");

                (uses_legacy_helper || inlines_confirm_dialog).then(|| path.display().to_string())
            })
            .collect();

        assert!(
            offenders.is_empty(),
            "remaining legacy confirmation callers: {:?}",
            offenders
        );
    }
}
