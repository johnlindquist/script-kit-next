//! One pending Undo for the last reversible action.
//!
//! Reversible actions (moving a script, file, or note to the Trash) run
//! immediately instead of asking first; a toast offers Undo. ⌘Z in the window
//! that shows the toast, or the toast's Undo button, takes the offer. Typing
//! or deleting text in that window withdraws it, so ⌘Z goes back to undoing
//! text. A newer offer replaces the older one.

use std::cell::RefCell;
use std::time::{Duration, Instant};

use gpui::{AnyWindowHandle, App, Keystroke, SharedString, Window};
use gpui_component::WindowExt as _;

use crate::components::{Toast, ToastAction, ToastId};

/// How long the offer (and its toast) stays live.
pub(crate) const UNDO_OFFER_DURATION_MS: u64 = 6_000;

/// The window surface that owns an offer, for automation projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UndoSurface {
    Main,
    Notes,
}

type UndoCallback = Box<dyn FnOnce(&mut Window, &mut App)>;

/// Notes shows its toasts in an attached popup window; the main window keeps
/// in-window notifications.
enum UndoToast {
    InWindow(ToastId),
    Popup,
}

const UNDO_TOAST_POPUP_KEY: &str = "undo-offer";

struct PendingUndo {
    surface: UndoSurface,
    window: AnyWindowHandle,
    toast: UndoToast,
    message: SharedString,
    expires_at: Instant,
    undo: UndoCallback,
}

thread_local! {
    static PENDING: RefCell<Option<PendingUndo>> = const { RefCell::new(None) };
}

/// Show `message` with an Undo toast in `window` and arm ⌘Z there.
pub(crate) fn offer_undo(
    surface: UndoSurface,
    message: impl Into<SharedString>,
    undo: impl FnOnce(&mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) {
    withdraw(window, cx);

    let message = message.into();
    let toast = match surface {
        UndoSurface::Main => UndoToast::InWindow(push_in_window_toast(&message, window, cx)),
        UndoSurface::Notes => {
            crate::toast_popup::show_toast_popup(
                window,
                crate::toast_popup::ToastPopupRequest {
                    key: UNDO_TOAST_POPUP_KEY.into(),
                    message: message.clone(),
                    kind: crate::toast_popup::ToastPopupKind::Success,
                    duration_ms: Some(UNDO_OFFER_DURATION_MS),
                    action: Some(crate::toast_popup::ToastPopupAction {
                        label: "Undo".into(),
                        shortcut: Some("⌘Z".into()),
                        on_click: std::rc::Rc::new(|window, cx| {
                            take_offer(window, cx);
                        }),
                    }),
                },
                cx,
            );
            UndoToast::Popup
        }
    };

    PENDING.set(Some(PendingUndo {
        surface,
        window: window.window_handle(),
        toast,
        message,
        expires_at: Instant::now() + Duration::from_millis(UNDO_OFFER_DURATION_MS),
        undo: Box::new(undo),
    }));
}

fn push_in_window_toast(message: &SharedString, window: &mut Window, cx: &mut App) -> ToastId {
    let toast_id = ToastId::unique();
    let theme = crate::theme::get_theme_snapshot().theme.clone();
    let toast = Toast::success(message.clone(), &theme)
        .with_id(toast_id.as_str().to_string())
        .duration_ms(Some(UNDO_OFFER_DURATION_MS))
        .action(
            ToastAction::new(
                "undo",
                "Undo",
                Box::new(|_, window, cx| {
                    take_offer(window, cx);
                }),
            )
            .with_shortcut("⌘Z"),
        );
    window.push_notification(
        crate::toast_manager::notification::pending_toast_to_notification(
            crate::toast_manager::PendingToast {
                message: message.to_string(),
                variant: toast.get_variant(),
                duration_ms: Some(UNDO_OFFER_DURATION_MS),
                persistent: false,
                toast,
            },
        ),
        cx,
    );
    toast_id
}

/// [`offer_undo`] for main-window code that has no `Window` at hand.
pub(crate) fn offer_main_window_undo(
    message: impl Into<SharedString>,
    undo: impl FnOnce(&mut Window, &mut App) + 'static,
    cx: &mut App,
) {
    let Some(handle) = crate::get_main_window_handle() else {
        return;
    };
    let message = message.into();
    cx.defer(move |cx| {
        let _ = handle.update(cx, |_, window, cx| {
            offer_undo(UndoSurface::Main, message, undo, window, cx);
        });
    });
}

/// Route a keystroke received by `window`. Returns true when ⌘Z took the
/// offer, so the caller stops the key from reaching text undo.
pub(crate) fn route_window_key(window: &mut Window, keystroke: &Keystroke, cx: &mut App) -> bool {
    let Some((owner, expires_at)) =
        PENDING.with_borrow(|pending| pending.as_ref().map(|p| (p.window, p.expires_at)))
    else {
        return false;
    };
    if Instant::now() >= expires_at {
        PENDING.set(None);
        return false;
    }
    if owner != window.window_handle() {
        return false;
    }
    if is_undo_shortcut(keystroke) {
        return take_offer(window, cx);
    }
    if edits_text(keystroke) {
        withdraw(window, cx);
    }
    false
}

/// The live offer for `surface`, as an automation button that ⌘Z activates.
pub(crate) fn semantic_element(surface: UndoSurface) -> Option<crate::protocol::ElementInfo> {
    PENDING.with_borrow(|pending| {
        let pending = pending
            .as_ref()
            .filter(|p| p.surface == surface && Instant::now() < p.expires_at)?;
        let mut element = crate::protocol::ElementInfo::button(0, "Undo");
        element.semantic_id = "button:undo".to_string();
        element.value = Some(pending.message.to_string());
        element.kind = Some("undoOffer".to_string());
        element.role = Some("undo".to_string());
        element.source = Some("shortcut:cmd-z".to_string());
        element.selectable = Some(true);
        Some(element)
    })
}

fn take_offer(window: &mut Window, cx: &mut App) -> bool {
    let Some(pending) = PENDING.take() else {
        return false;
    };
    if pending.window != window.window_handle() || Instant::now() >= pending.expires_at {
        return false;
    }
    dismiss_toast(pending.toast, window, cx);
    (pending.undo)(window, cx);
    true
}

fn withdraw(window: &mut Window, cx: &mut App) {
    let Some(pending) = PENDING.take() else {
        return;
    };
    if pending.window == window.window_handle() {
        dismiss_toast(pending.toast, window, cx);
    } else {
        let toast = pending.toast;
        let handle = pending.window;
        cx.defer(move |cx| {
            let _ = handle.update(cx, |_, window, cx| dismiss_toast(toast, window, cx));
        });
    }
}

fn dismiss_toast(toast: UndoToast, window: &mut Window, cx: &mut App) {
    match toast {
        UndoToast::InWindow(toast_id) => {
            crate::toast_manager::notification::dismiss_toast_notification(&toast_id, window, cx);
        }
        UndoToast::Popup => crate::toast_popup::dismiss_toast_popup(UNDO_TOAST_POPUP_KEY, cx),
    }
}

fn is_undo_shortcut(keystroke: &Keystroke) -> bool {
    let m = &keystroke.modifiers;
    keystroke.key.eq_ignore_ascii_case("z") && m.platform && !m.shift && !m.control && !m.alt
}

fn edits_text(keystroke: &Keystroke) -> bool {
    let m = &keystroke.modifiers;
    if m.platform || m.control {
        return false;
    }
    matches!(keystroke.key.as_str(), "backspace" | "delete")
        || keystroke
            .key_char
            .as_deref()
            .is_some_and(|text| !text.is_empty() && !text.chars().any(char::is_control))
}

#[cfg(test)]
mod tests {
    use super::{edits_text, is_undo_shortcut};

    fn key(spec: &str) -> gpui::Keystroke {
        gpui::Keystroke::parse(spec).expect("keystroke")
    }

    #[test]
    fn only_plain_cmd_z_takes_the_offer() {
        assert!(is_undo_shortcut(&key("cmd-z")));
        assert!(!is_undo_shortcut(&key("cmd-shift-z")));
        assert!(!is_undo_shortcut(&key("z")));
        assert!(!is_undo_shortcut(&key("ctrl-z")));
    }

    #[test]
    fn typing_and_deleting_withdraw_but_navigation_does_not() {
        let mut typed = key("a");
        typed.key_char = Some("a".into());
        assert!(edits_text(&typed));
        assert!(edits_text(&key("backspace")));
        assert!(!edits_text(&key("down")));
        assert!(!edits_text(&key("escape")));
        assert!(!edits_text(&key("cmd-k")));
    }
}
