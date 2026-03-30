use crate::{
    Placement, Root, dialog::Dialog, input::InputState, notification::Notification, sheet::Sheet,
};
use gpui::{App, Entity, Window};
use std::rc::Rc;

/// Extension trait for [`Window`] to add dialog, sheet .. functionality.
pub trait WindowExt: Sized {
    /// Opens a Sheet at right placement.
    fn open_sheet<F>(&mut self, cx: &mut App, build: F)
    where
        F: Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static;

    /// Opens a Sheet at the given placement.
    fn open_sheet_at<F>(&mut self, placement: Placement, cx: &mut App, build: F)
    where
        F: Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static;

    /// Return true, if there is an active Sheet.
    fn has_active_sheet(&mut self, cx: &mut App) -> bool;

    /// Closes the active Sheet.
    fn close_sheet(&mut self, cx: &mut App);

    /// Opens a Dialog.
    fn open_dialog<F>(&mut self, cx: &mut App, build: F)
    where
        F: Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static;

    /// Return true, if there is an active Dialog.
    fn has_active_dialog(&mut self, cx: &mut App) -> bool;

    /// Closes the last active Dialog.
    fn close_dialog(&mut self, cx: &mut App);

    /// Closes all active Dialogs.
    fn close_all_dialogs(&mut self, cx: &mut App);

    /// Pushes a notification to the notification list.
    fn push_notification(&mut self, note: impl Into<Notification>, cx: &mut App);

    /// Removes the notification with the given id.
    fn remove_notification<T: Sized + 'static>(&mut self, cx: &mut App);

    /// Clears all notifications.
    fn clear_notifications(&mut self, cx: &mut App);

    /// Returns number of notifications.
    fn notifications(&mut self, cx: &mut App) -> Rc<Vec<Entity<Notification>>>;

    /// Return current focused Input entity.
    fn focused_input(&mut self, cx: &mut App) -> Option<Entity<InputState>>;
    /// Returns true if there is a focused Input entity.
    fn has_focused_input(&mut self, cx: &mut App) -> bool;

    /// Move focus to the next tab stop within the active dialog.
    /// If focus escapes the dialog, wraps back to the first dialog button.
    fn focus_next_in_dialog(&mut self, cx: &mut App);

    /// Move focus to the previous tab stop within the active dialog.
    /// If focus escapes the dialog, wraps back to the last dialog button.
    fn focus_prev_in_dialog(&mut self, cx: &mut App);
}

impl WindowExt for Window {
    #[inline]
    fn open_sheet<F>(&mut self, cx: &mut App, build: F)
    where
        F: Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static,
    {
        self.open_sheet_at(Placement::Right, cx, build)
    }

    #[inline]
    fn open_sheet_at<F>(&mut self, placement: Placement, cx: &mut App, build: F)
    where
        F: Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static,
    {
        Root::update(self, cx, move |root, window, cx| {
            root.open_sheet_at(placement, build, window, cx);
        })
    }

    #[inline]
    fn has_active_sheet(&mut self, cx: &mut App) -> bool {
        Root::read(self, cx).active_sheet.is_some()
    }

    #[inline]
    fn close_sheet(&mut self, cx: &mut App) {
        Root::update(self, cx, |root, window, cx| {
            root.close_sheet(window, cx);
        })
    }

    #[inline]
    fn open_dialog<F>(&mut self, cx: &mut App, build: F)
    where
        F: Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static,
    {
        Root::update(self, cx, move |root, window, cx| {
            root.open_dialog(build, window, cx);
        })
    }

    #[inline]
    fn has_active_dialog(&mut self, cx: &mut App) -> bool {
        Root::read(self, cx).active_dialogs.len() > 0
    }

    #[inline]
    fn close_dialog(&mut self, cx: &mut App) {
        Root::update(self, cx, |root, window, cx| {
            root.close_dialog(window, cx);
        })
    }

    #[inline]
    fn close_all_dialogs(&mut self, cx: &mut App) {
        Root::update(self, cx, |root, window, cx| {
            root.close_all_dialogs(window, cx);
        })
    }

    #[inline]
    fn push_notification(&mut self, note: impl Into<Notification>, cx: &mut App) {
        let note = note.into();
        Root::update(self, cx, |root, window, cx| {
            root.push_notification(note, window, cx);
        })
    }

    #[inline]
    fn remove_notification<T: Sized + 'static>(&mut self, cx: &mut App) {
        Root::update(self, cx, |root, window, cx| {
            root.remove_notification::<T>(window, cx);
        })
    }

    #[inline]
    fn clear_notifications(&mut self, cx: &mut App) {
        Root::update(self, cx, |root, window, cx| {
            root.clear_notifications(window, cx);
        })
    }

    #[inline]
    fn notifications(&mut self, cx: &mut App) -> Rc<Vec<Entity<Notification>>> {
        Rc::new(Root::read(self, cx).notification.read(cx).notifications())
    }

    #[inline]
    fn has_focused_input(&mut self, cx: &mut App) -> bool {
        Root::read(self, cx).focused_input.is_some()
    }

    #[inline]
    fn focused_input(&mut self, cx: &mut App) -> Option<Entity<InputState>> {
        Root::read(self, cx).focused_input.clone()
    }

    fn focus_next_in_dialog(&mut self, cx: &mut App) {
        let dialog_handle = Root::read(self, cx)
            .active_dialogs
            .last()
            .map(|d| d.focus_handle.clone());
        self.focus_next(cx);
        if let Some(handle) = dialog_handle {
            if !handle.contains_focused(self, cx) {
                self.focus(&handle, cx);
                self.focus_next(cx);
            }
        }
    }

    fn focus_prev_in_dialog(&mut self, cx: &mut App) {
        let dialog_handle = Root::read(self, cx)
            .active_dialogs
            .last()
            .map(|d| d.focus_handle.clone());
        self.focus_prev(cx);
        if let Some(handle) = dialog_handle {
            if !handle.contains_focused(self, cx) {
                // Escaped backward — wrap to last dialog button.
                // Walk forward from the dialog container, tracking the
                // last position that was still inside the dialog.
                self.focus(&handle, cx);
                self.focus_next(cx);
                let mut last_inside = self.focused(cx);
                const MAX_DIALOG_TAB_STOPS: usize = 64;
                for _ in 0..MAX_DIALOG_TAB_STOPS {
                    self.focus_next(cx);
                    if !handle.contains_focused(self, cx) {
                        break;
                    }
                    last_inside = self.focused(cx);
                }
                // Restore the last known in-dialog position, or fall
                // back to the dialog container → first button.
                if let Some(target) = last_inside {
                    self.focus(&target, cx);
                } else {
                    self.focus(&handle, cx);
                    self.focus_next(cx);
                }
            }
        }
    }
}
