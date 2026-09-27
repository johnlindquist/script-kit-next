//! "Share my last action": copy it as TypeScript, copy a link to it, or send it to someone. The
//! choices are rows in the list; what each did lands in the line above the input.

use gpui::{ClipboardItem, Context};
use sk_one_window::{Done, Event, Row, ViewOp};

use super::saved_scripts::entry_source;
use crate::ScriptListApp;

impl ScriptListApp {
    /// The last run as TypeScript: its saved entry, or a new one from what was said.
    fn one_window_last_as_script(&self) -> Option<(String, String)> {
        let entry = self.one_window.ledger().last()?.clone();
        let name = entry.script.clone().unwrap_or_else(|| entry.title.clone());
        let source =
            entry
                .script
                .as_ref()
                .and_then(|script| self.one_window_saved_entry(script))
                .unwrap_or_else(|| {
                    entry_source(
                    &name,
                    &entry.said,
                    &["// Jev's steps for this run weren't kept; the sentence above is what ran."
                        .to_string()],
                )
                });
        Some((name, source))
    }

    fn one_window_shared(&mut self, icon: &str, title: String, cx: &mut Context<Self>) {
        self.one_window_handle(
            Event::Done(Done {
                icon: icon.to_string(),
                title,
                ..Done::default()
            }),
            cx,
        );
    }

    pub(super) fn one_window_run_share(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> Option<anyhow::Result<()>> {
        let (name, source) = self.one_window_last_as_script()?;
        let title = row.title.as_str();
        if title.starts_with("Copy it to the clipboard") {
            cx.write_to_clipboard(ClipboardItem::new_string(source));
            self.one_window_shared("copy", format!("Copied “{name}” as TypeScript"), cx);
        } else if title.starts_with("Copy a link") {
            self.one_window_share_link(name, source, cx);
        } else if title.starts_with("Send it to") {
            self.one_window_handle(
                Event::View(ViewOp::Home {
                    text: format!("send “{name}” to "),
                }),
                cx,
            );
        } else {
            return None;
        }
        Some(Ok(()))
    }

    /// A secret gist anyone with the link can open, made with the signed-in `gh`.
    fn one_window_share_link(&mut self, name: String, source: String, cx: &mut Context<Self>) {
        let made = cx.background_executor().spawn(async move {
            crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)?;
            let dir = tempfile::tempdir()?;
            let file = dir
                .path()
                .join(format!("{}.ts", super::saved_scripts::function_name(&name)));
            std::fs::write(&file, source)?;
            let out = std::process::Command::new("gh")
                .args(["gist", "create", "--desc"])
                .arg(format!("Script Kit: {name}"))
                .arg(&file)
                .output()?;
            anyhow::ensure!(out.status.success(), "gh gist create failed");
            let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
            anyhow::ensure!(url.starts_with("https://"), "gh printed no link");
            anyhow::Ok((name, url))
        });
        cx.spawn(async move |this, cx| {
            let made = made.await;
            let _ = this.update(cx, |app, cx| match made {
                Ok((name, url)) => {
                    cx.write_to_clipboard(ClipboardItem::new_string(url));
                    app.one_window_shared("copy", format!("Copied a link to “{name}”"), cx);
                }
                Err(error) => {
                    tracing::warn!(target: "script_kit::one_window", %error, "share_link_failed");
                    app.one_window_shared(
                        "copy",
                        "Couldn't make a link · sign in with gh auth login".to_string(),
                        cx,
                    );
                }
            });
        })
        .detach();
    }
}
