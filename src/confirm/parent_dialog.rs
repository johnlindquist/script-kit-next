use std::rc::Rc;

use gpui::{
    px, App, ClickEvent, ParentElement as _, Pixels, SharedString, Styled as _, WeakEntity, Window,
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

        dialog
            .rounded_lg()
            .w(width)
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
