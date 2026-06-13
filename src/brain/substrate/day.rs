//! Day-page append API.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use super::io::atomic_write;
use super::paths::BrainPaths;
use super::trash::trash_file;
use super::FragmentReference;

/// A single append-only entry for today's day page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DayEntry {
    /// Freeform capture text (dictation, `;note` body, short clipboard paste).
    Capture { text: String },
    /// Task line from `;todo` capture.
    Task {
        body: String,
        tags: Vec<String>,
        due: Option<String>,
    },
    /// Auto-kept URL from clipboard sediment.
    KeptUrl { url: String },
    /// Reference to a long-capture fragment (excerpt + relative link).
    FragmentRef(FragmentReference),
    /// Agent Chat thread trace (one line per thread per day).
    Trace {
        summary: String,
        provenance_link: String,
    },
}

impl DayEntry {
    pub fn format_line(&self, timestamp: &str) -> String {
        match self {
            Self::Capture { text } => format!("{timestamp} {text}"),
            Self::Task { body, tags, due } => {
                let mut line = format!("{timestamp} - [ ] {body}");
                for tag in tags {
                    let normalized = tag.trim().trim_start_matches('#');
                    if !normalized.is_empty() {
                        line.push(' ');
                        line.push('#');
                        line.push_str(normalized);
                    }
                }
                if let Some(due) = due {
                    let due = due.trim();
                    if !due.is_empty() {
                        line.push_str(" due:");
                        line.push_str(due);
                    }
                }
                line
            }
            Self::KeptUrl { url } => format!("{timestamp} {url}"),
            Self::FragmentRef(reference) => {
                format!(
                    "{timestamp} [{}]({})",
                    reference.excerpt, reference.relative_link
                )
            }
            Self::Trace {
                summary,
                provenance_link,
            } => format!("{timestamp} — {summary} ({provenance_link})"),
        }
    }
}

pub fn local_day_and_time(now: DateTime<Utc>, tz: Tz) -> (chrono::NaiveDate, String) {
    let local = now.with_timezone(&tz);
    let date = local.date_naive();
    let time = local.format("%H:%M").to_string();
    (date, time)
}

/// Remove day-page lines (and linked fragments) written by clipboard sediment.
pub fn undo_clipboard_sediment_lines(
    paths: &BrainPaths,
    tz: Tz,
    now: DateTime<Utc>,
    entry_id: &str,
    text: &str,
    kept_url_day: Option<&str>,
    brain_kept: bool,
) -> Result<()> {
    if !brain_kept && kept_url_day.is_none() {
        return Ok(());
    }

    let source_uri = format!("scriptkit://clipboard/{entry_id}");
    let trimmed = text.trim();

    if let Some(day) = kept_url_day {
        if let Ok(date) = NaiveDate::parse_from_str(day, "%Y-%m-%d") {
            remove_kept_url_line(paths, date, trimmed)?;
        }
    }

    if brain_kept {
        let (today, _) = local_day_and_time(now, tz);
        remove_capture_line(paths, today, trimmed)?;
        remove_fragment_reference_for_source(paths, today, &source_uri)?;
        trash_fragment_for_source(paths, &source_uri)?;
    }

    Ok(())
}

fn remove_kept_url_line(paths: &BrainPaths, date: NaiveDate, url: &str) -> Result<()> {
    let path = paths.day_page(date);
    filter_day_page_lines(&path, |line| !line_contains_token(line, url))
}

fn remove_capture_line(paths: &BrainPaths, date: NaiveDate, text: &str) -> Result<()> {
    let path = paths.day_page(date);
    filter_day_page_lines(&path, |line| !line_contains_token(line, text))
}

fn remove_fragment_reference_for_source(
    paths: &BrainPaths,
    date: NaiveDate,
    source_uri: &str,
) -> Result<()> {
    let path = paths.day_page(date);
    if !path.exists() {
        return Ok(());
    }

    let original =
        fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut lines: Vec<&str> = original.lines().collect();
    let mut changed = false;
    let mut index = 0;
    while index < lines.len() {
        if lines[index].contains('>') && index + 1 < lines.len() {
            let link_line = lines[index + 1].trim();
            if let Some(fragment_path) = fragment_path_from_day_link(paths, link_line) {
                if fragment_has_source(&fragment_path, source_uri) {
                    lines.remove(index + 1);
                    lines.remove(index);
                    changed = true;
                    continue;
                }
            }
        }
        index += 1;
    }

    if changed {
        write_filtered_lines(&path, lines)?;
    }
    Ok(())
}

fn trash_fragment_for_source(paths: &BrainPaths, source_uri: &str) -> Result<()> {
    let fragments_dir = paths.fragments_dir();
    if !fragments_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&fragments_dir)
        .with_context(|| format!("reading fragments dir {}", fragments_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        if fragment_has_source(&path, source_uri) {
            let _ = trash_file(paths, &path);
        }
    }
    Ok(())
}

fn fragment_path_from_day_link(paths: &BrainPaths, link_line: &str) -> Option<PathBuf> {
    let relative = link_line.trim().trim_start_matches("../fragments/");
    if relative.is_empty() {
        return None;
    }
    let fragment_id = relative.trim_end_matches(".md");
    Some(paths.fragment_file(fragment_id))
}

fn fragment_has_source(fragment_path: &PathBuf, source_uri: &str) -> bool {
    fs::read_to_string(fragment_path)
        .ok()
        .is_some_and(|content| content.contains(&format!("source: {source_uri}")))
}

fn filter_day_page_lines(path: &PathBuf, keep: impl Fn(&str) -> bool) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let original =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let kept: Vec<&str> = original.lines().filter(|line| keep(line)).collect();
    if kept.len() == original.lines().count() {
        return Ok(());
    }
    write_filtered_lines(path, kept)
}

fn write_filtered_lines(path: &Path, lines: Vec<&str>) -> Result<()> {
    let mut contents = lines.join("\n");
    if !contents.is_empty() {
        contents.push('\n');
    }
    atomic_write(path, &contents)
}

fn line_contains_token(line: &str, token: &str) -> bool {
    let body = line.split_once(' ').map(|(_, rest)| rest).unwrap_or(line);
    body.contains(token)
}
