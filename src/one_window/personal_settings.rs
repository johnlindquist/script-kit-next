//! The ordinary input offers a local, revision-checked edit. No subprocess runs while typing;
//! only submitting the displayed row calls the installed settings service, off the UI thread.
use crate::mcp_config_tools::font;
use crate::ScriptListApp;
use gpui::Context;
use sk_one_window::{
    font_preference::{self, FontPreference, FontSurface},
    Row,
};

const PREFIX: &str = "personal/font/";

impl ScriptListApp {
    fn surface_pixels(&self, surface: FontSurface) -> f32 {
        match surface {
            FontSurface::Editor => self.config.get_editor_font_size(),
            FontSurface::Terminal => self.config.get_terminal_font_size(),
        }
    }

    fn adopt_pixels(&mut self, surface: FontSurface, value: Option<f32>) {
        match surface {
            FontSurface::Editor => self.config.editor_font_size = value,
            FontSurface::Terminal => self.config.terminal_font_size = value,
        }
    }

    pub(super) fn personal_font_row(&self, text: &str) -> Option<Row> {
        let choice = font_preference::read(sk_personal::speech::directed(text).unwrap_or(text))?;
        let before = self.surface_pixels(choice.surface);
        Some(
            Row::new(
                format!(
                    "{PREFIX}{}/{}/{:08x}",
                    choice.surface.key(),
                    choice.pixels,
                    before.to_bits()
                ),
                format!(
                    "Set {} text to {} px",
                    choice.surface.label().to_ascii_lowercase(),
                    choice.pixels
                ),
            )
            .with_icon("type")
            .with_head("Customize Kit")
            .with_sub(format!("Currently {before} px · Only this Kit workspace"))
            .with_enter_label("Set text size"),
        )
    }

    pub(super) fn run_personal_font(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        // Bind the selected request, target and old size. Never reinterpret a stale row as a new edit.
        let text = self.filter_text.clone();
        let choice = font_preference::read(sk_personal::speech::directed(&text).unwrap_or(&text))
            .ok_or_else(|| anyhow::anyhow!("The text-size request changed"))?;
        anyhow::ensure!(
            self.personal_font_row(&text)
                .is_some_and(|shown| shown.key == row.key),
            "The displayed text size changed"
        );
        if self.one_window_host.editing_font {
            return Ok(());
        }
        let before = self.surface_pixels(choice.surface);
        self.one_window_host.editing_font = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { font::apply(choice, before) })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.one_window_host.editing_font = false;
                match result {
                    Err(error) => {
                        app.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx)
                    }
                    Ok(undo) => {
                        if app.filter_text == text && app.one_window_owns_list() {
                            app.one_window_set_input(String::new(), cx);
                        }
                        if let Some(undo) = undo {
                            app.adopt_pixels(choice.surface, Some(f32::from(choice.pixels)));
                            app.one_window_done_undoable(
                                crate::line_notice::INFO,
                                format!(
                                    "{} text set to {} px",
                                    choice.surface.label(),
                                    choice.pixels
                                ),
                                Box::new(move |app, cx| app.undo_personal_font(choice, undo, cx)),
                                cx,
                            );
                        } else {
                            app.one_window_notice(
                                crate::line_notice::INFO,
                                format!(
                                    "{} text is already {} px",
                                    choice.surface.label(),
                                    choice.pixels
                                ),
                                cx,
                            );
                        }
                    }
                }
                cx.notify();
            });
        })
        .detach();
        Ok(())
    }

    fn undo_personal_font(
        &mut self,
        choice: FontPreference,
        undo: font::FontUndo,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { undo.restore() })
                .await;
            let _ = this.update(cx, |app, cx| {
                match result {
                    Ok(value) => {
                        app.adopt_pixels(choice.surface, value);
                        app.one_window_notice(
                            crate::line_notice::INFO,
                            format!(
                                "{} text restored to {} px",
                                choice.surface.label(),
                                app.surface_pixels(choice.surface)
                            ),
                            cx,
                        );
                    }
                    Err(error) => {
                        app.one_window_notice(crate::line_notice::FAILED, error.to_string(), cx)
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
pub(super) fn is_font_row(row: &Row) -> bool {
    row.key.starts_with(PREFIX)
}
