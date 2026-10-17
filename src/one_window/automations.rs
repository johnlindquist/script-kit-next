//! Automations you can list: every script that runs on its own (on a schedule, or when a file
//! changes) is a row in a view under the input, and saying "stop the slack one" or "turn the
//! friday one back on" puts that row first with ↵ naming the change. A stopped automation stays
//! listed and its schedule skips it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use gpui::Context;
use sk_one_window::{Done, EnterTarget, Event, Row};

use super::learns::OwnView;
use crate::ScriptListApp;

const HEAD: &str = "Runs on its own";

fn off_path() -> PathBuf {
    crate::setup::get_kit_path().join("db/automations-off.json")
}

pub(super) fn load_off() -> BTreeSet<String> {
    std::fs::read_to_string(off_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Whether the user stopped the automation at `path`; its schedule skips it.
pub(crate) fn is_off(path: &Path) -> bool {
    load_off().contains(&path.to_string_lossy().to_string())
}

fn save_off(set: &BTreeSet<String>) -> std::io::Result<()> {
    let file = off_path();
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        file,
        serde_json::to_string(set).map_err(std::io::Error::other)?,
    )
}

/// When a script runs on its own, in words, and the next time if it's scheduled.
fn runs_when(script: &crate::scripts::Script) -> Option<String> {
    let text = std::fs::read_to_string(&script.path).unwrap_or_default();
    let (mut cron, mut schedule) = (None, None);
    for line in text.lines().take(30) {
        let Some((key, value)) = line
            .trim_start()
            .strip_prefix("//")
            .and_then(|rest| rest.split_once(':'))
        else {
            continue;
        };
        let value = value.trim().to_string();
        match key.trim().to_lowercase().as_str() {
            "cron" if !value.is_empty() => cron = Some(value),
            "schedule" if !value.is_empty() => schedule = Some(value),
            _ => {}
        }
    }
    let expr = match (&cron, &schedule) {
        (Some(expr), _) => Some(expr.clone()),
        (None, Some(natural)) => crate::scheduler::natural_to_cron(natural).ok(),
        (None, None) => None,
    };
    if let Some(expr) = expr {
        let said = schedule.unwrap_or_else(|| format!("Cron {expr}"));
        let next = crate::scheduler::parse_cron(&expr)
            .ok()
            .and_then(|parsed| {
                parsed
                    .find_next_occurrence(&chrono::Local::now(), false)
                    .ok()
            })
            .map(|at| format!(" · next {}", at.format("%a %-I:%M %p")))
            .unwrap_or_default();
        return Some(format!("{said}{next}"));
    }
    let meta = script.typed_metadata.as_ref()?;
    if let Some(first) = meta.watch.first() {
        return Some(format!("When {first} changes"));
    }
    if meta.system {
        return Some("When the system event it listens for happens".to_string());
    }
    meta.background
        .then(|| "Keeps running in the background".to_string())
}

fn automation_row(script: &crate::scripts::Script, off: &BTreeSet<String>) -> Option<Row> {
    let when = runs_when(script)?;
    let path = script.path.to_string_lossy().to_string();
    let stopped = off.contains(&path);
    // How the latest unattended run went, so a failing automation is visible
    // without opening logs.
    let when = match crate::scheduler::runs::state(&script.path)
        .and_then(|state| state.summary(chrono::Utc::now()))
    {
        Some(last) => format!("{when} · {last}"),
        None => when,
    };
    let row = Row::new(format!("auto:{path}"), script.name.clone())
        .with_icon(if stopped { "off" } else { "automation" })
        .with_enter_label(if stopped {
            "Turn it back on"
        } else {
            "Turn it off"
        });
    Some(if stopped {
        row.with_sub(format!("Off · {when}"))
    } else {
        row.with_sub(when)
    })
}

/// What a sentence asks of the automations: stop or start, and which one.
fn said(text: &str, rows: &[Row]) -> Option<(usize, bool)> {
    let t = text.to_lowercase();
    let stop = ["stop", "turn off", "pause", "disable"]
        .iter()
        .any(|v| t.contains(v));
    let start = [
        "start",
        "turn on",
        "turn it on",
        "back on",
        "resume",
        "enable",
    ]
    .iter()
    .any(|v| t.contains(v));
    if stop == start {
        return None;
    }
    let words: Vec<&str> = t
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .filter(|w| {
            !matches!(
                *w,
                "stop"
                    | "turn"
                    | "off"
                    | "the"
                    | "one"
                    | "pause"
                    | "start"
                    | "back"
                    | "resume"
                    | "enable"
                    | "disable"
                    | "that"
            )
        })
        .collect();
    let (ix, hits) = rows
        .iter()
        .enumerate()
        .map(|(ix, row)| {
            let hay = format!(
                "{} {}",
                row.title.to_lowercase(),
                row.sub.as_deref().unwrap_or_default().to_lowercase()
            );
            (ix, words.iter().filter(|w| hay.contains(**w)).count())
        })
        .max_by_key(|(_, hits)| *hits)?;
    (hits > 0).then_some((ix, stop))
}

/// The automations with the one a sentence names first, ↵ saying what will change.
pub(crate) fn said_first(mut rows: Vec<Row>, text: &str) -> Vec<Row> {
    let Some((ix, stop)) = said(text, &rows) else {
        return rows;
    };
    let mut row = rows.remove(ix);
    let name = row.title.clone();
    row.enter_label = Some(if stop {
        format!("Turn off “{name}”")
    } else {
        format!("Turn on “{name}”")
    });
    rows.insert(0, row);
    rows
}

impl ScriptListApp {
    fn one_window_automation_rows(&self) -> Vec<Row> {
        let off = &self.one_window_host.learns.stopped;
        self.scripts
            .iter()
            .filter_map(|script| automation_row(script, off))
            .collect()
    }

    pub(super) fn one_window_open_automations(&mut self, cx: &mut Context<Self>) {
        let rows = self.one_window_automation_rows();
        let enter = rows.first().map(|row| {
            EnterTarget::new(row.key.clone(), row.enter_label.clone().unwrap_or_default())
        });
        let learns = &mut self.one_window_host.learns;
        learns.view_rows = rows;
        learns.view_head = HEAD.to_string();
        self.one_window_open_view(
            OwnView::Automations,
            "Automations".to_string(),
            enter,
            true,
            cx,
        );
    }

    /// Stops or restarts the highlighted automation; the line above the input says which.
    pub(super) fn one_window_automation_enter(&mut self, row: &Row, cx: &mut Context<Self>) {
        let Some(path) = row.key.strip_prefix("auto:") else {
            return;
        };
        let stopped = &mut self.one_window_host.learns.stopped;
        let stop = stopped.insert(path.to_string());
        if !stop {
            stopped.remove(path);
        }
        if self.main_services.is_production() {
            if let Err(error) = save_off(&self.one_window_host.learns.stopped) {
                tracing::warn!(target: "script_kit::one_window", %error, "automation_not_saved");
                self.one_window_notice(
                    crate::line_notice::FAILED,
                    "Couldn't remember that change; it lasts until Script Kit quits.",
                    cx,
                );
            }
        }
        let title = if stop {
            format!("Turned off · {}", row.title)
        } else {
            format!("Turned back on · {}", row.title)
        };
        self.one_window_handle(
            Event::Done(Done {
                icon: "again".into(),
                title,
                stay: true,
                ..Done::default()
            }),
            cx,
        );
        self.one_window_host.learns.view_rows = self.one_window_automation_rows();
        self.one_window_republish_own(cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<Row> {
        vec![
            Row::new("auto:/a", "Mute Slack at 9").with_sub("every weekday at 9"),
            Row::new("auto:/b", "Post the weekly summary").with_sub("every Friday 4 PM"),
            Row::new("auto:/c", "Offer review").with_sub("When ~/Downloads changes"),
        ]
    }

    #[test]
    fn saying_so_puts_the_automation_first() {
        let out = said_first(rows(), "stop the slack one");
        assert_eq!(out[0].key, "auto:/a");
        assert_eq!(
            out[0].enter_label.as_deref(),
            Some("Turn off “Mute Slack at 9”")
        );
        let out = said_first(rows(), "turn the friday one back on");
        assert_eq!(out[0].key, "auto:/b");
        assert_eq!(
            out[0].enter_label.as_deref(),
            Some("Turn on “Post the weekly summary”")
        );
    }

    #[test]
    fn words_without_a_change_leave_the_list_alone() {
        let out = said_first(rows(), "slack");
        assert_eq!(out[0].key, "auto:/a");
        assert!(out[0].enter_label.is_none());
        assert_eq!(said("stop everything", &rows()), None, "names none of them");
    }
}
