//! [`TransactionStateProvider`] implementations for non-main automation targets.
//!
//! Bridges the generic transaction executor (used by `batch`) with the live state of the
//! Actions dialog, enabling `setInput`, `waitFor`, `selectByValue`, and `selectBySemanticId`
//! against it, and applies selections to registered root layers.

use crate::protocol::transaction_executor::TransactionStateProvider;
use crate::protocol::{AutomationWindowInfo, UiStateSnapshot};
use anyhow::{anyhow, Result};
use gpui::{App, Entity};

/// Select a retained Root layer by its production collector ID. Validation-only
/// selection does not invent a persistent selection; submission dismisses the
/// exact layer and does not claim a dialog Confirm or Cancel callback ran.
pub fn apply_registered_root_layer_selection(
    resolved: &AutomationWindowInfo,
    semantic_id: &str,
    submit: bool,
    cx: &mut App,
) -> Result<Option<String>> {
    let mut parts = semantic_id.split(':');
    let dialog = match parts.next() {
        Some("root-dialog") => true,
        Some("root-notification") => false,
        _ => return Ok(None),
    };
    let missing = || crate::protocol::TransactionError::element_not_found(semantic_id);
    let entity_id = parts
        .next()
        .and_then(|part| part.parse::<u64>().ok())
        .ok_or_else(missing)?;
    let dialog_id = if dialog {
        let generation = parts
            .next()
            .and_then(|part| part.parse::<u64>().ok())
            .ok_or_else(missing)?;
        Some(gpui_component::RootDialogId {
            root_entity_id: entity_id,
            generation,
        })
    } else {
        None
    };
    if parts.next().is_some() {
        return Err(missing().into());
    }
    let canonical_id = if let Some(id) = dialog_id {
        format!("root-dialog:{}:{}", id.root_entity_id, id.generation)
    } else {
        format!("root-notification:{entity_id}")
    };
    if canonical_id != semantic_id {
        return Err(missing().into());
    }

    let target = crate::windows::automation_surface_collector::current_surface_metadata(resolved)
        .ok_or_else(|| anyhow!("root_layer_target_stale"))?;
    let generation = target
        .generation
        .ok_or_else(|| anyhow!("root_layer_generation_missing"))?;
    let policy = crate::windows::runtime_window_host_policy(&target.id, generation)?;
    policy.validate()?;
    anyhow::ensure!(
        !policy.is_hidden() || (!target.visible && !target.focused),
        "root_layer_hidden_metadata_mismatch"
    );
    let handle = crate::windows::get_runtime_window_handle_for_generation(&target.id, generation)
        .ok_or_else(|| anyhow!("root_layer_target_stale"))?;
    let root = handle.read(cx, |root: Entity<gpui_component::Root>, _| root)?;
    let layers = root.read(cx).layer_snapshot(cx);
    if let Some(id) = dialog_id {
        if layers.dialogs.last() != Some(&id) {
            return Err(missing().into());
        }
    } else if !layers
        .notifications
        .iter()
        .any(|notification| notification.entity_id == entity_id && !notification.closing)
    {
        return Err(missing().into());
    }
    if submit {
        let dismissed = handle.update(cx, |_, window, cx| {
            anyhow::ensure!(
                window.is_owned_hidden() == policy.is_hidden(),
                "root_layer_host_mismatch"
            );
            Ok(if let Some(id) = dialog_id {
                gpui_component::Root::close_dialog_if_current(id, window, cx)
            } else {
                gpui_component::Root::dismiss_notification_if_current(entity_id, window, cx)
            })
        })??;
        if !dismissed {
            return Err(missing().into());
        }
    }
    Ok(Some(canonical_id))
}

#[cfg(test)]
mod registered_root_layer_tests {
    use super::*;
    use gpui::{prelude::*, AnyWindowHandle, TestAppContext};
    use gpui_component::{notification::Notification, Root, WindowExt as _};

    struct Content;
    impl Render for Content {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            gpui::div()
        }
    }

    #[gpui::test]
    fn registered_layer_selection_validates_then_dismisses_only_current_owners(
        cx: &mut TestAppContext,
    ) {
        cx.update(crate::theme::gpui_kit_bridge::init_component_libraries);
        let root = cx.update(|cx| {
            cx.open_window(
                gpui::WindowOptions {
                    show: false,
                    focus: false,
                    ..Default::default()
                },
                |window, cx| {
                    let content = cx.new(|_| Content);
                    cx.new(|cx| Root::new(content, window, cx))
                },
            )
            .unwrap()
        });
        let handle: AnyWindowHandle = root.into();
        let target = cx.update(|cx| {
            crate::windows::register_runtime_window_instance(
                AutomationWindowInfo {
                    id: "test:registered-root-layer-selection".into(),
                    kind: crate::protocol::AutomationWindowKind::SnapOverlay,
                    title: None,
                    focused: false,
                    visible: false,
                    semantic_surface: Some("snapOverlay".into()),
                    bounds: None,
                    parent_window_id: None,
                    parent_window_generation: None,
                    parent_kind: None,
                    pid: Some(std::process::id()),
                    generation: None,
                },
                handle,
                cx,
            )
            .unwrap()
        });
        handle
            .update(cx, |_, window, cx| {
                window.open_dialog(cx, |dialog, _, _| dialog.title("Lower"));
                window.open_dialog(cx, |dialog, _, _| dialog.title("Current"));
            })
            .unwrap();
        let before = root
            .read_with(cx, |root, cx| root.layer_snapshot(cx))
            .unwrap();
        let lower = before.dialogs[0];
        let current = *before.dialogs.last().unwrap();
        let lower_id = format!("root-dialog:{}:{}", lower.root_entity_id, lower.generation);
        let current_id = format!(
            "root-dialog:{}:{}",
            current.root_entity_id, current.generation
        );
        assert_eq!(
            cx.update(|cx| apply_registered_root_layer_selection(
                &target,
                "choice:0:item",
                true,
                cx
            ))
            .unwrap(),
            None
        );
        assert!(cx
            .update(|cx| apply_registered_root_layer_selection(&target, &lower_id, true, cx))
            .is_err());
        assert_eq!(
            cx.update(|cx| apply_registered_root_layer_selection(&target, &current_id, false, cx))
                .unwrap(),
            Some(current_id.clone())
        );
        assert_eq!(
            before,
            root.read_with(cx, |root, cx| root.layer_snapshot(cx))
                .unwrap()
        );
        let alias = format!(
            "root-dialog:0{}:{}",
            current.root_entity_id, current.generation
        );
        assert!(cx
            .update(|cx| apply_registered_root_layer_selection(&target, &alias, true, cx))
            .is_err());
        let mut stale = target.clone();
        stale.generation = target.generation.map(|generation| generation + 1);
        assert!(cx
            .update(|cx| apply_registered_root_layer_selection(&stale, &current_id, true, cx))
            .is_err());
        assert_eq!(
            before,
            root.read_with(cx, |root, cx| root.layer_snapshot(cx))
                .unwrap()
        );
        assert_eq!(
            cx.update(|cx| apply_registered_root_layer_selection(&target, &current_id, true, cx))
                .unwrap(),
            Some(current_id.clone())
        );
        assert!(cx
            .update(|cx| apply_registered_root_layer_selection(&target, &current_id, true, cx))
            .is_err());

        handle
            .update(cx, |_, window, cx| {
                window.push_notification(Notification::success("Saved").autohide(false), cx)
            })
            .unwrap();
        let notifications = root
            .read_with(cx, |root, cx| root.layer_snapshot(cx))
            .unwrap();
        let notification_id = format!(
            "root-notification:{}",
            notifications.notifications[0].entity_id
        );
        assert_eq!(
            cx.update(|cx| apply_registered_root_layer_selection(
                &target,
                &notification_id,
                false,
                cx
            ))
            .unwrap(),
            Some(notification_id.clone())
        );
        assert_eq!(
            notifications,
            root.read_with(cx, |root, cx| root.layer_snapshot(cx))
                .unwrap()
        );
        assert_eq!(
            cx.update(|cx| apply_registered_root_layer_selection(
                &target,
                &notification_id,
                true,
                cx
            ))
            .unwrap(),
            Some(notification_id.clone())
        );
        assert!(cx
            .update(|cx| apply_registered_root_layer_selection(&target, &notification_id, true, cx))
            .is_err());
        crate::windows::remove_runtime_window_instance(&target.id, target.generation.unwrap());
        handle
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
    }
}
