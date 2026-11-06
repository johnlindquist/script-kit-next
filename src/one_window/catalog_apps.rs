//! Contextual actions for the Apps catalog, bound to its app, never a background launcher row.
use crate::{
    actions::{Action, ActionCategory, ActionsDialog, ActionsDialogConfig},
    ActionsDialogHost, AppView, ScriptListApp,
};
use gpui::{AppContext as _, Context, Window};
use std::sync::Arc;
const COPY_NAME: &str = "catalog-app:copy-name";
const COPY_ID: &str = "catalog-app:copy-bundle-id";
impl ScriptListApp {
    pub(crate) fn toggle_catalog_app_actions(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.show_actions_popup {
            self.close_actions_popup(ActionsDialogHost::AppLauncher, window, cx);
            return true;
        }
        if self.was_actions_recently_closed() {
            return true;
        }
        let AppView::AppLauncherView {
            filter,
            selected_index,
        } = &self.current_view
        else {
            return false;
        };
        let Some((_, app)) = self.app_launcher_selected_visible_entry(filter, *selected_index)
        else {
            return false;
        };
        let app = app.clone();
        let mut rows = vec![Action::new(
            COPY_NAME,
            "Copy app name",
            Some(format!("Copy {} without launching it", app.name)),
            ActionCategory::ScriptContext,
        )];
        if let Some(id) = app.bundle_id.as_deref().filter(|s| !s.is_empty()) {
            rows.push(Action::new(
                COPY_ID,
                "Copy bundle identifier",
                Some(id.to_string()),
                ActionCategory::ScriptContext,
            ));
        }
        let theme = self.theme.clone();
        let title = app.name.clone();
        let dialog = cx.new(move |cx| {
            let mut d = ActionsDialog::with_config(
                cx.focus_handle(),
                Arc::new(|_| {}),
                rows,
                theme,
                ActionsDialogConfig::default(),
            );
            d.set_skip_track_focus(true);
            d.set_match_main_window_background(true);
            d
        });
        self.one_window_stage_in_view(
            ActionsDialogHost::AppLauncher,
            dialog,
            Some(title),
            window,
            cx,
        );
        if let Some(stage) = self.one_window_host.view_stage.as_mut() {
            stage.catalog_app = Some(app);
        }
        true
    }
    pub(crate) fn handle_bound_catalog_action(
        &mut self,
        host: ActionsDialogHost,
        id: &str,
        close: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if host != ActionsDialogHost::AppLauncher || !matches!(id, COPY_NAME | COPY_ID) {
            return false;
        }
        let target = self
            .one_window_view_staged()
            .filter(|(h, _)| *h == host)
            .and_then(|_| {
                self.one_window_host
                    .view_stage
                    .as_ref()?
                    .catalog_app
                    .clone()
            });
        if close {
            self.close_actions_popup(host, window, cx);
        }
        let Some(app) = target else {
            self.one_window_notice(crate::line_notice::INFO, "Select the app again", cx);
            return true;
        };
        let (value, notice) = if id == COPY_NAME {
            (Some(app.name), "Copied app name")
        } else {
            (app.bundle_id, "Copied bundle identifier")
        };
        if let Some(value) = value {
            self.copy_to_clipboard_with_feedback(&value, notice.to_string(), false, cx);
        }
        true
    }
}
