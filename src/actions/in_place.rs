//! The actions staged in the main window, for surfaces that don't own the main window's app.
//!
//! ⌘K acts in place: the main window lists what's staged under its own input, and no other
//! window opens. Agent Chat's view, Notes and the command bars ask here whether actions are
//! staged, route a key to them, set their search, or let go of them. The main window registers
//! what it staged and how to let go; these names predate the change and are kept for the
//! surfaces other lanes still own.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{App, Entity, Modifiers, WeakEntity};

use super::dialog::ActionsDialog;

/// Lets go of what's staged, the way Esc does.
pub(crate) type LetGo = Rc<dyn Fn(&mut App)>;
/// Gives a key to the staged actions; `true` when they used it.
pub(crate) type RouteKey = Rc<dyn Fn(&str, Option<&str>, &Modifiers, &mut App) -> bool>;

struct Staged {
    dialog: WeakEntity<ActionsDialog>,
    let_go: LetGo,
    route_key: RouteKey,
}

thread_local! {
    static STAGED: RefCell<Option<Staged>> = const { RefCell::new(None) };
}

/// The main window staged `dialog`'s actions in place.
#[allow(dead_code, reason = "the binary's ScriptListApp stages")]
pub(crate) fn stage_in_place(dialog: &Entity<ActionsDialog>, let_go: LetGo, route_key: RouteKey) {
    STAGED.with(|staged| {
        *staged.borrow_mut() = Some(Staged {
            dialog: dialog.downgrade(),
            let_go,
            route_key,
        });
    });
}

/// The main window let go of `dialog`; a newer stage stays.
#[allow(dead_code, reason = "the binary's ScriptListApp lets go")]
pub(crate) fn clear_in_place(dialog: &Entity<ActionsDialog>) {
    STAGED.with(|staged| {
        let mut staged = staged.borrow_mut();
        if staged
            .as_ref()
            .is_some_and(|current| current.dialog.entity_id() == dialog.entity_id())
        {
            *staged = None;
        }
    });
}

/// Whether the main window has actions staged in place.
pub fn is_actions_window_open() -> bool {
    STAGED.with(|staged| {
        staged
            .borrow()
            .as_ref()
            .is_some_and(|current| current.dialog.upgrade().is_some())
    })
}

/// Lets go of the staged actions, as Esc would.
pub fn close_actions_window(cx: &mut App) {
    let let_go = STAGED.with(|staged| staged.borrow().as_ref().map(|s| s.let_go.clone()));
    if let Some(let_go) = let_go {
        let_go(cx);
    }
}

/// Gives a key that reached another surface to the staged actions. Returns whether they used it.
pub fn route_key_to_detached_actions_window(
    key: &str,
    key_char: Option<&str>,
    modifiers: &Modifiers,
    cx: &mut App,
) -> bool {
    let route_key = STAGED.with(|staged| staged.borrow().as_ref().map(|s| s.route_key.clone()));
    route_key.is_some_and(|route_key| route_key(key, key_char, modifiers, cx))
}

/// Replaces the staged actions' search through their input. Returns false when `dialog` isn't
/// what's staged or the main window is gone.
pub(crate) fn set_actions_dialog_search_text(
    dialog: &Entity<ActionsDialog>,
    text: String,
    cx: &mut App,
) -> bool {
    let staged = STAGED.with(|staged| {
        staged
            .borrow()
            .as_ref()
            .is_some_and(|current| current.dialog.entity_id() == dialog.entity_id())
    });
    let Some(main) = crate::get_main_window_handle().filter(|_| staged) else {
        return false;
    };
    main.update(cx, |_, window, cx| {
        dialog.update(cx, |dialog, cx| {
            dialog.set_search_text_in_window(text, window, cx)
        });
    })
    .is_ok()
}
