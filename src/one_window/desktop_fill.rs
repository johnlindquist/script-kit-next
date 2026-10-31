//! In-place preview only; the separate affirmative row is the write boundary.
use super::ledger_view::LedgerRun;
use crate::{
    desktop_fill::{explanation, launcher_api, Status},
    AppView, ScriptListApp,
};
use gpui::Context;

impl ScriptListApp {
    pub(crate) fn inspect_desktop_filename(
        &mut self,
        app: String,
        pid: u32,
        window_id: u32,
        title: String,
        filename: String,
        cx: &mut Context<Self>,
    ) -> Result<(), &'static str> {
        if self.one_window_host.acting_on.is_some() || self.one_window.staged().is_some() {
            self.one_window_notice(
                crate::line_notice::INFO,
                "Release the staged subject to inspect the originating Save sheet",
                cx,
            );
            return Err("staged_subject_mismatch");
        }
        let Ok(api) = launcher_api(self.main_services.is_production()) else {
            self.one_window_notice(
                crate::line_notice::INFO,
                "Desktop access is disabled in this session",
                cx,
            );
            return Err("native_effects_forbidden");
        };
        let generation = self.one_window_host.fill_preview.wrapping_add(1);
        self.one_window_host.fill_preview = generation;
        cx.spawn(async move |this, cx| {
            let preview = cx
                .background_executor()
                .spawn(async move {
                    api.propose("launcher", &app, pid, window_id, Some(&title), &filename)
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                if generation != app.one_window_host.fill_preview
                    || !matches!(app.current_view, AppView::ScriptList)
                    || (!crate::runtime_policy::is_owned_evaluation()
                        && !script_kit_gpui::is_main_window_visible())
                {
                    return;
                }
                match preview {
                    Err(e) => app.one_window_notice(crate::line_notice::FAILED, explanation(e), cx),
                    Ok(p) => {
                        let id = p.proposal_id;
                        let apply: LedgerRun = Box::new(move |_app, cx| {
                            cx.spawn(async move |this, cx| {
                                let receipt = cx
                                    .background_executor()
                                    .spawn(async move { api.apply("launcher", id) })
                                    .await;
                                let _ = this.update(cx, |app, cx| {
                                    let (icon, msg) = match receipt {
                                        Ok(Status::Verified) => (
                                            crate::line_notice::INFO,
                                            "Filename filled; Save sheet left open",
                                        ),
                                        Ok(Status::Uncertain { .. }) => (
                                            crate::line_notice::FAILED,
                                            "Couldn’t verify the change; check the Save sheet",
                                        ),
                                        Err(code) | Ok(Status::Refused { code }) => {
                                            (crate::line_notice::FAILED, explanation(code))
                                        }
                                        _ => (
                                            crate::line_notice::INFO,
                                            "The filename change is in progress",
                                        ),
                                    };
                                    app.one_window_notice(icon, msg, cx);
                                });
                            })
                            .detach();
                        });
                        let unchanged = || -> LedgerRun {
                            Box::new(|app, cx| {
                                app.one_window_notice(
                                    crate::line_notice::INFO,
                                    "Filename left unchanged",
                                    cx,
                                )
                            })
                        };
                        app.one_window_ask(
                            crate::line_notice::INFO,
                            format!("Fill filename with “{}”?", p.filename),
                            format!(
                                "{} · {} · leave the sheet open",
                                p.window.app, p.window.title
                            ),
                            vec![
                                ("Fill filename".into(), apply),
                                ("Leave unchanged".into(), unchanged()),
                            ],
                            Some(unchanged()),
                            cx,
                        );
                    }
                }
            });
        })
        .detach();
        Ok(())
    }
}
