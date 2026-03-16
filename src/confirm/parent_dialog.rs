use std::rc::Rc;

use gpui::{px, App, ClickEvent, ParentElement as _, Pixels, SharedString, Styled as _, Window};
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

pub(crate) fn open_parent_confirm_dialog(
    window: &mut Window,
    cx: &mut App,
    options: ParentConfirmOptions,
    on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    on_cancel: impl Fn(&mut Window, &mut App) + 'static,
) {
    tracing::info!(
        event = "parent_confirm_dialog_opened",
        title = %options.title,
        "parent_confirm_dialog_opened"
    );

    let on_confirm: ConfirmCallback = Rc::new(on_confirm);
    let on_cancel: ConfirmCallback = Rc::new(on_cancel);

    window.open_dialog(cx, move |dialog, _window, cx| {
        let on_confirm = on_confirm.clone();
        let on_cancel = on_cancel.clone();

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
