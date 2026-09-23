use crate::{
    ActiveTheme, ElementExt, Placement, StyledExt,
    dialog::{ANIMATION_DURATION, Dialog},
    input::{AnyInputState, Copy},
    native_menu::FallbackMenuOverlay,
    notification::{Notification, NotificationList},
    sheet::Sheet,
    tooltip::render_tooltip,
    touch_selection::WindowTouchSelectionOverlay,
    window_border,
};
use gpui::{
    AnyView, App, AppContext, ClipboardItem, Context, DefiniteLength, ElementId, Entity,
    FocusHandle, InteractiveElement, IntoElement, KeyBinding, ParentElement as _, Pixels, Render,
    StyleRefinement, Styled, WeakFocusHandle, Window, actions, div, prelude::FluentBuilder as _,
};
use gpui_base::{TextSelection, TextSelectionLayer, TextSelectionScopeId};
use std::{any::TypeId, rc::Rc};

actions!(root, [Tab, TabPrev]);

const CONTEXT: &str = "Root";
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", Tab, Some(CONTEXT)),
        KeyBinding::new("shift-tab", TabPrev, Some(CONTEXT)),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-c", Copy, Some(CONTEXT)),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-c", Copy, Some(CONTEXT)),
    ]);
}

/// Root is a view for the App window for as the top level view (Must be the first view in the window).
///
/// It is used to manage the Sheet, Dialog, and Notification.
/// Script Kit: identity of one actual Root-owned dialog lifetime, not a stack
/// position — a replacement dialog in the same slot gets a new identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootDialogId {
    pub root_entity_id: u64,
    pub generation: u64,
}

/// Script Kit: read-only view of Root's retained layer owners.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootLayerSnapshot {
    pub revision: u64,
    /// Live stack order; the last identity owns modal interaction.
    pub dialogs: Vec<RootDialogId>,
    pub notifications: Vec<crate::notification::NotificationLayerSnapshot>,
    pub notifications_expanded: bool,
}

pub struct Root {
    /// Script Kit: monotonic dialog-layer epoch (see [`RootLayerSnapshot`]).
    dialog_revision: u64,
    /// Script Kit: whether the window-sized wrapper paints the background.
    paint_background: bool,
    style: StyleRefinement,
    view: AnyView,
    pub(crate) active_sheet: Option<ActiveSheet>,
    pub(crate) active_dialogs: Vec<ActiveDialog>,
    pub(super) focused_input: Option<AnyInputState>,
    pub notification: Entity<NotificationList>,
    pub(crate) tooltip_overlay: Entity<gpui_base::TooltipOverlay>,
    pub(crate) native_menu_overlay: Entity<FallbackMenuOverlay>,
    touch_selection_overlay: Entity<WindowTouchSelectionOverlay>,
    sheet_size: Option<DefiniteLength>,
    window_shadow_size: Pixels,
    /// Render the Linux CSD `window_border` wrapper.
    bordered: bool,
    /// The focus handle that will be restored after a dialog is closed with animation.
    /// Used to handle rapid dialog opening/closing to maintain correct focus chain.
    pending_focus_restore: Option<WeakFocusHandle>,
    window_id: gpui::WindowId,
}

#[derive(Clone)]
pub(crate) struct ActiveSheet {
    focus_handle: FocusHandle,
    /// The previous focused handle before opening the Sheet.
    previous_focused_handle: Option<WeakFocusHandle>,
    placement: Placement,
    selection_scope: TextSelectionScopeId,
    builder: Rc<dyn Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static>,
}

#[derive(Clone)]
pub(crate) struct ActiveDialog {
    /// Script Kit: stable identity of this dialog lifetime.
    pub(crate) id: RootDialogId,
    focus_handle: FocusHandle,
    /// The previous focused handle before opening the Dialog.
    previous_focused_handle: Option<WeakFocusHandle>,
    selection_scope: TextSelectionScopeId,
    builder: Rc<dyn Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static>,
}

impl ActiveDialog {
    pub(crate) fn new(
        id: RootDialogId,
        focus_handle: FocusHandle,
        previous_focused_handle: Option<WeakFocusHandle>,
        selection_scope: TextSelectionScopeId,
        builder: impl Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static,
    ) -> Self {
        Self {
            id,
            focus_handle,
            previous_focused_handle,
            selection_scope,
            builder: Rc::new(builder),
        }
    }
}

impl Root {
    /// Clears window-owned text selection synchronously.
    #[deprecated(note = "use gpui_base::TextSelection::clear instead")]
    pub fn clear_text_selection(&mut self, cx: &mut Context<Self>) {
        gpui_base::TextSelection::clear_for_window(self.window_id, cx);
    }

    /// Create a new Root view.
    pub fn new(view: impl Into<AnyView>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        #[cfg(all(target_os = "macos", not(test)))]
        gpui_base::install_window_hit_test_forwarder(window);

        Self {
            dialog_revision: 0,
            paint_background: true,
            style: StyleRefinement::default(),
            view: view.into(),
            active_sheet: None,
            active_dialogs: Vec::new(),
            focused_input: None,
            notification: cx.new(|cx| NotificationList::new(window, cx)),
            tooltip_overlay: cx
                .new(|_| gpui_base::TooltipOverlay::new().render_with(render_tooltip)),
            native_menu_overlay: cx.new(|_| FallbackMenuOverlay::new()),
            touch_selection_overlay: cx.new(|cx| WindowTouchSelectionOverlay::new(window, cx)),
            sheet_size: None,
            window_shadow_size: window_border::SHADOW_SIZE,
            bordered: true,
            pending_focus_restore: None,
            window_id: window.window_handle().window_id(),
        }
    }

    fn allocate_text_selection_scope(&mut self) -> TextSelectionScopeId {
        TextSelectionScopeId::new()
    }

    pub(crate) fn active_text_selection_scope(&self) -> TextSelectionScopeId {
        self.active_dialogs
            .last()
            .map(|dialog| dialog.selection_scope)
            .or_else(|| {
                self.active_sheet
                    .as_ref()
                    .map(|sheet| sheet.selection_scope)
            })
            .unwrap_or_default()
    }

    /// Enable or disable the Linux client-side window border wrapper.
    ///
    /// Defaults to `true`. Use `bordered(false)` for layer-shell fullscreen windows
    /// or other surfaces that should not render GPUI Component's window border.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Set the window border shadow size for Linux client-side decorations.
    ///
    /// Default: [`window_border::SHADOW_SIZE`]
    pub fn window_shadow_size(mut self, size: impl Into<Pixels>) -> Self {
        self.window_shadow_size = size.into();
        self
    }

    /// Script Kit: a Root whose window-sized wrapper stays transparent, for
    /// views that paint a smaller, bounded stage inside a transparent window.
    pub fn new_transparent(
        view: impl Into<AnyView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut root = Self::new(view, window, cx);
        root.paint_background = false;
        root
    }

    /// Script Kit: monotonic epoch covering dialogs and notifications.
    pub fn layer_revision_with(&self, cx: &App) -> u64 {
        self.dialog_revision
            .wrapping_add(self.notification.read(cx).layer_revision)
    }

    /// Script Kit: snapshot retained layer owners; never renders or mutates.
    pub fn layer_snapshot(&self, cx: &App) -> RootLayerSnapshot {
        let notifications = self.notification.read(cx);
        RootLayerSnapshot {
            revision: self.layer_revision_with(cx),
            dialogs: self.active_dialogs.iter().map(|dialog| dialog.id).collect(),
            notifications: notifications.layer_snapshot(cx),
            notifications_expanded: notifications.expanded(),
        }
    }

    /// Script Kit: whether `id` is the topmost live dialog.
    pub fn is_current_dialog(&self, id: RootDialogId) -> bool {
        self.active_dialogs
            .last()
            .is_some_and(|dialog| dialog.id == id)
    }

    /// Script Kit: close only if `id` is still the current dialog, rejecting
    /// stale handlers even when a replacement occupies the same stack slot.
    pub fn close_dialog_if_current(id: RootDialogId, window: &mut Window, cx: &mut App) -> bool {
        let Some(root) = window.root::<Root>().flatten() else {
            return false;
        };
        root.update(cx, |root, cx| {
            if !root.is_current_dialog(id) {
                return false;
            }
            root.close_dialog(window, cx);
            true
        })
    }

    /// Script Kit: dismiss only a notification entity still live in this Root.
    pub fn dismiss_notification_if_current(
        entity_id: u64,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        let Some(root) = window.root::<Root>().flatten() else {
            return false;
        };
        let list = root.read(cx).notification.clone();
        if list.read(cx).notification_closing(entity_id, cx) {
            return false;
        }
        let Some(notification) = list.read(cx).notification_entity(entity_id) else {
            return false;
        };
        notification.update(cx, |note, cx| note.dismiss(window, cx));
        true
    }

    pub fn update<F, R>(window: &mut Window, cx: &mut App, f: F) -> R
    where
        F: FnOnce(&mut Self, &mut Window, &mut Context<Self>) -> R,
    {
        let root = window
            .root::<Root>()
            .flatten()
            .expect("BUG: window first layer should be a gpui_component::Root.");

        root.update(cx, |root, cx| f(root, window, cx))
    }

    pub(crate) fn try_update<F, R>(window: &mut Window, cx: &mut App, f: F) -> Option<R>
    where
        F: FnOnce(&mut Self, &mut Window, &mut Context<Self>) -> R,
    {
        let root = window.root::<Root>().flatten()?;
        Some(root.update(cx, |root, cx| f(root, window, cx)))
    }

    pub fn read<'a>(window: &'a Window, cx: &'a App) -> &'a Self {
        &window
            .root::<Root>()
            .expect("The window root view should be of type `ui::Root`.")
            .unwrap()
            .read(cx)
    }

    // Render Notification layer.
    pub fn render_notification_layer(
        window: &mut Window,
        cx: &mut App,
    ) -> Option<impl IntoElement + use<>> {
        let root = window.root::<Root>()??;

        let active_sheet_placement = root.read(cx).active_sheet.clone().map(|d| d.placement);

        let sheet_size = root.read(cx).sheet_size;
        let (mt, mr, mb, ml) = match active_sheet_placement {
            Some(Placement::Top) => (sheet_size, None, None, None),
            Some(Placement::Right) => (None, sheet_size, None, None),
            Some(Placement::Bottom) => (None, None, sheet_size, None),
            Some(Placement::Left) => (None, None, None, sheet_size),
            _ => (None, None, None, None),
        };

        Some(
            div()
                .absolute()
                .inset_0()
                .when_some(mt, |this, offset| this.mt(offset))
                .when_some(mr, |this, offset| this.mr(offset))
                .when_some(mb, |this, offset| this.mb(offset))
                .when_some(ml, |this, offset| this.ml(offset))
                .child(root.read(cx).notification.clone()),
        )
    }

    /// Render the Sheet layer.
    pub fn render_sheet_layer(
        window: &mut Window,
        cx: &mut App,
    ) -> Option<impl IntoElement + use<>> {
        let root = window.root::<Root>()??;

        if let Some(active_sheet) = root.read(cx).active_sheet.clone() {
            let mut sheet = Sheet::new(window, cx);
            sheet = (active_sheet.builder)(sheet, window, cx);
            sheet.focus_handle = active_sheet.focus_handle.clone();
            sheet.placement = active_sheet.placement;
            sheet.selection_scope = active_sheet.selection_scope;

            let size = sheet.size;

            return Some(
                div()
                    .relative()
                    .child(sheet)
                    .on_prepaint(move |_, _, cx| root.update(cx, |r, _| r.sheet_size = Some(size))),
            );
        }

        None
    }

    /// Render the Dialog layer.
    pub fn render_dialog_layer(
        window: &mut Window,
        cx: &mut App,
    ) -> Option<impl IntoElement + use<>> {
        let root = window.root::<Root>()??;

        let active_dialogs = root.read(cx).active_dialogs.clone();

        if active_dialogs.is_empty() {
            return None;
        }

        let mut show_overlay_ix = None;

        let mut dialogs = active_dialogs
            .iter()
            .enumerate()
            .map(|(i, active_dialog)| {
                let mut dialog = Dialog::new(cx);

                dialog = (active_dialog.builder)(dialog, window, cx);

                // Give the dialog the focus handle, because `dialog` is a temporary value, is not possible to
                // keep the focus handle in the dialog.
                //
                // So we keep the focus handle in the `active_dialog`, this is owned by the `Root`.
                dialog.focus_handle = active_dialog.focus_handle.clone();
                dialog.selection_scope = active_dialog.selection_scope;

                dialog.layer_ix = i;
                // Find the dialog which one needs to show overlay.
                if dialog.has_overlay() {
                    show_overlay_ix = Some(i);
                }

                dialog
            })
            .collect::<Vec<_>>();

        if let Some(ix) = show_overlay_ix {
            if let Some(dialog) = dialogs.get_mut(ix) {
                dialog.props.overlay_visible = true;
            }
        }

        // Named so a test can assert the layer actually reached the screen. A
        // dialog that opens into a root which never renders this layer looks
        // exactly like one that does not open.
        Some(
            div()
                .debug_selector(|| "dialog-layer".to_string())
                .children(dialogs),
        )
    }

    pub fn open_dialog<F>(&mut self, build: F, window: &mut Window, cx: &mut Context<'_, Root>)
    where
        F: Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static,
    {
        let mut previous_focused_handle = window.focused(cx).map(|h| h.downgrade());

        // Use pending focus restore if available to maintain correct focus chain
        // when a new dialog is opened immediately after closing another dialog.
        if let Some(pending_handle) = self.pending_focus_restore.take() {
            previous_focused_handle = Some(pending_handle);
        }

        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        let selection_scope = self.allocate_text_selection_scope();
        self.dialog_revision = self.dialog_revision.wrapping_add(1);
        let id = RootDialogId {
            root_entity_id: cx.entity_id().as_u64(),
            generation: self.dialog_revision,
        };
        self.active_dialogs.push(ActiveDialog::new(
            id,
            focus_handle,
            previous_focused_handle,
            selection_scope,
            build,
        ));
        // Opening a modal confines selection to it; drop any background
        // selection so it cannot linger (or be copied) under the modal.
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
    }

    fn close_dialog_internal(&mut self) -> Option<FocusHandle> {
        if !self.active_dialogs.is_empty() {
            self.dialog_revision = self.dialog_revision.wrapping_add(1);
        }
        self.focused_input = None;
        self.active_dialogs
            .pop()
            .and_then(|d| d.previous_focused_handle)
            .and_then(|h| h.upgrade())
    }

    pub fn close_dialog(&mut self, window: &mut Window, cx: &mut Context<'_, Root>) {
        if let Some(handle) = self.close_dialog_internal() {
            window.focus(&handle, cx);
        }
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
    }

    pub(crate) fn defer_close_dialog(&mut self, window: &mut Window, cx: &mut Context<'_, Root>) {
        if let Some(handle) = self.close_dialog_internal() {
            let dialogs_count = self.active_dialogs.len();

            // Save for new dialogs opened during animation to maintain focus chain
            self.pending_focus_restore = Some(handle.downgrade());

            cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(*ANIMATION_DURATION).await;
                let _ = this.update_in(cx, |this, window, cx| {
                    let current_dialogs_count = this.active_dialogs.len();
                    // Only restore focus if no new dialogs were opened during animation
                    if current_dialogs_count == dialogs_count {
                        window.focus(&handle, cx);
                    }
                    this.pending_focus_restore = None;
                });
            })
            .detach();
        }
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
    }

    pub fn close_all_dialogs(&mut self, window: &mut Window, cx: &mut Context<'_, Root>) {
        if !self.active_dialogs.is_empty() {
            self.dialog_revision = self.dialog_revision.wrapping_add(1);
        }
        self.focused_input = None;
        let previous_focused_handle = self
            .active_dialogs
            .first()
            .and_then(|d| d.previous_focused_handle.clone());
        self.active_dialogs.clear();
        if let Some(handle) = previous_focused_handle.and_then(|h| h.upgrade()) {
            window.focus(&handle, cx);
        }
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
    }

    pub fn open_sheet_at<F>(
        &mut self,
        placement: Placement,
        build: F,
        window: &mut Window,
        cx: &mut Context<'_, Root>,
    ) where
        F: Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static,
    {
        let previous_focused_handle = self
            .active_sheet
            .take()
            .and_then(|s| s.previous_focused_handle)
            .or_else(|| window.focused(cx).map(|h| h.downgrade()));

        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        let selection_scope = self.allocate_text_selection_scope();
        self.active_sheet = Some(ActiveSheet {
            focus_handle,
            previous_focused_handle,
            placement,
            selection_scope,
            builder: Rc::new(build),
        });
        // Opening a modal confines selection to it; drop any background
        // selection so it cannot linger (or be copied) under the modal.
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
    }

    pub fn close_sheet(&mut self, window: &mut Window, cx: &mut Context<'_, Root>) {
        self.focused_input = None;
        if let Some(previous_handle) = self
            .active_sheet
            .as_ref()
            .and_then(|s| s.previous_focused_handle.as_ref())
            .and_then(|h| h.upgrade())
        {
            window.focus(&previous_handle, cx);
        }
        self.active_sheet = None;
        gpui_base::TextSelection::clear(window, cx);
        cx.notify();
    }

    pub fn push_notification(
        &mut self,
        note: impl Into<Notification>,
        window: &mut Window,
        cx: &mut Context<'_, Root>,
    ) {
        self.notification
            .update(cx, |view, cx| view.push(note, window, cx));
        cx.notify();
    }

    /// Removes all notifications whose id matches `T`, including ones registered with
    /// either [`Notification::id`] or [`Notification::id1`] (any key).
    pub fn remove_notification<T: Sized + 'static>(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, Root>,
    ) {
        self.notification.update(cx, |view, cx| {
            view.close_by_type(TypeId::of::<T>(), window, cx);
        });
        cx.notify();
    }

    /// Removes the notification matching the given type and element id (paired with [`Notification::id1`]).
    pub fn remove_notification1<T: Sized + 'static>(
        &mut self,
        key: impl Into<ElementId>,
        window: &mut Window,
        cx: &mut Context<'_, Root>,
    ) {
        let key = key.into();
        self.notification.update(cx, |view, cx| {
            view.close((TypeId::of::<T>(), key), window, cx);
        });
        cx.notify();
    }

    pub fn clear_notifications(&mut self, window: &mut Window, cx: &mut Context<'_, Root>) {
        self.notification
            .update(cx, |view, cx| view.clear(window, cx));
        cx.notify();
    }

    /// Get the tooltip overlay entity for this window.
    pub(crate) fn tooltip_overlay(
        window: &Window,
        cx: &App,
    ) -> Option<Entity<gpui_base::TooltipOverlay>> {
        let root = window.root::<Root>()??;
        Some(root.read(cx).tooltip_overlay.clone())
    }

    /// Get the fallback native-menu overlay entity for this window.
    pub(crate) fn native_menu_overlay(
        window: &Window,
        cx: &App,
    ) -> Option<Entity<FallbackMenuOverlay>> {
        let root = window.root::<Root>()??;
        Some(root.read(cx).native_menu_overlay.clone())
    }

    /// Return the root view of the Root.
    pub fn view(&self) -> &AnyView {
        &self.view
    }

    fn on_action_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        // Check if we're inside a focus trap
        if let Some(container_focus_handle) = gpui_base::active_focus_trap(window, cx) {
            // We're in a focus trap - try to focus next, then check if we're still inside
            let before_focus = window.focused(cx);

            // Try normal focus navigation
            window.focus_next(cx);

            // Check if we're still in the trap
            if !container_focus_handle.contains_focused(window, cx) {
                // We jumped out of the trap - need to cycle back to the beginning
                // Find the first focusable element in the trap by continuing to focus_next
                let mut attempts = 0;
                const MAX_ATTEMPTS: usize = 100; // Prevent infinite loop

                while !container_focus_handle.contains_focused(window, cx)
                    && attempts < MAX_ATTEMPTS
                {
                    window.focus_next(cx);
                    attempts += 1;

                    // If we cycled back to where we started, restore original focus
                    if window.focused(cx) == before_focus {
                        break;
                    }
                }
            }
            return;
        }

        // Normal tab navigation
        window.focus_next(cx);
    }

    fn on_action_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        // Check if we're inside a focus trap
        if let Some(container_focus_handle) = gpui_base::active_focus_trap(window, cx) {
            // We're in a focus trap - try to focus previous, then check if we're still inside
            let before_focus = window.focused(cx);

            // Try normal focus navigation
            window.focus_prev(cx);

            // Check if we're still in the trap
            if !container_focus_handle.contains_focused(window, cx) {
                // We jumped out of the trap - need to cycle back to the end
                // Find the last focusable element in the trap by continuing to focus_prev
                let mut attempts = 0;
                const MAX_ATTEMPTS: usize = 100; // Prevent infinite loop

                while !container_focus_handle.contains_focused(window, cx)
                    && attempts < MAX_ATTEMPTS
                {
                    window.focus_prev(cx);
                    attempts += 1;

                    // If we cycled back to where we started, restore original focus
                    if window.focused(cx) == before_focus {
                        break;
                    }
                }
            }
            return;
        }

        // Normal tab navigation
        window.focus_prev(cx);
    }

    fn on_action_copy(&mut self, _: &Copy, window: &mut Window, cx: &mut Context<Self>) {
        let text = gpui_base::TextSelection::selected_text(window, cx)
            .trim()
            .to_string();
        if text.is_empty() {
            cx.propagate();
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }
}

impl Styled for Root {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(cx.theme().font_size);
        let active_scope = self.active_text_selection_scope();
        TextSelection::activate_scope(active_scope, window, cx);

        let inner = div()
            .id("root")
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::on_action_tab))
            .on_action(cx.listener(Self::on_action_tab_prev))
            .on_action(cx.listener(Self::on_action_copy))
            .relative()
            .size_full()
            .font_family(cx.theme().font_family.clone())
            // Script Kit: transparent Roots leave the window-sized wrapper unpainted.
            .when(self.paint_background, |this| this.bg(cx.theme().tokens.background))
            .text_color(cx.theme().foreground)
            .refine_style(&self.style)
            .child(TextSelectionLayer)
            .child(self.view.clone())
            .child(self.touch_selection_overlay.clone())
            .child(self.tooltip_overlay.clone())
            .child(self.native_menu_overlay.clone());

        if self.bordered {
            window_border()
                .shadow_size(self.window_shadow_size)
                .child(inner)
                .into_any_element()
        } else {
            inner.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    struct TestView;

    impl Render for TestView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
        }
    }

    #[gpui::test]
    fn bordered_builder_toggles_window_border(cx: &mut TestAppContext) {
        cx.update(crate::init);

        let (default_root, _) = cx.add_window_view(|window, cx| {
            let view = cx.new(|_| TestView);
            Root::new(view, window, cx)
        });
        assert!(default_root.read_with(cx, |root, _| root.bordered));

        let (root, _) = cx.add_window_view(|window, cx| {
            let view = cx.new(|_| TestView);
            Root::new(view, window, cx).bordered(false)
        });
        assert!(!root.read_with(cx, |root, _| root.bordered));

        let (root, _) = cx.add_window_view(|window, cx| {
            let view = cx.new(|_| TestView);
            Root::new(view, window, cx).bordered(false).bordered(true)
        });
        assert!(root.read_with(cx, |root, _| root.bordered));
    }
}
