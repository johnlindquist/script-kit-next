//! What an agent session's file tools changed, so the user can undo it.
//!
//! Every `write` and `edit` served by Script Kit records the file's content
//! before the session first touched it in the current turn, and what the tool
//! left behind. "Revert" puts the earlier content back for the latest turn that
//! changed files; "Undo Revert" puts back what was there just before the revert.
//! Shell commands run by the engine are not tracked: they can change anything.

// The lib target has no one-window host to offer Revert.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use parking_lot::Mutex;

/// One file a turn changed.
#[derive(Debug, Clone)]
struct FileChange {
    path: PathBuf,
    /// Content before the turn's first change; `None` = the file did not exist.
    before: Option<Vec<u8>>,
    /// Content the agent's last change in this turn left.
    after: Option<Vec<u8>>,
}

#[derive(Debug, Default)]
struct TurnChanges {
    turn: u64,
    files: Vec<FileChange>,
    /// Content just before the revert, per path, while the turn is reverted.
    reverted_over: Option<Vec<(PathBuf, Option<Vec<u8>>)>>,
}

#[derive(Debug, Default)]
struct Journal {
    turn: u64,
    turns: Vec<TurnChanges>,
}

static JOURNALS: LazyLock<Mutex<HashMap<String, Journal>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Turns of history kept per session.
const MAX_TURNS: usize = 20;

/// Files changed by the latest turn that changed any, as the user sees them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChangeSummary {
    pub paths: Vec<PathBuf>,
    pub reverted: bool,
}

/// What a revert or undo did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RevertReport {
    pub restored: Vec<PathBuf>,
    /// Files someone else changed after the agent did; their content was
    /// replaced too (and comes back with Undo Revert).
    pub changed_since: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

pub(super) fn begin_turn(session: &str) {
    let mut journals = JOURNALS.lock();
    let journal = journals.entry(session.to_string()).or_default();
    journal.turn += 1;
}

pub(super) fn forget(session: &str) {
    JOURNALS.lock().remove(session);
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

fn write_optional(path: &Path, content: Option<&[u8]>) -> Result<(), String> {
    match content {
        Some(bytes) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("{}: {error}", parent.display()))?;
            }
            std::fs::write(path, bytes).map_err(|error| format!("{}: {error}", path.display()))
        }
        None => match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("{}: {error}", path.display())),
        },
    }
}

/// Change `path` to `content` and record it for this session's current turn.
pub(super) fn apply(session: Option<&str>, path: &Path, content: &[u8]) -> Result<(), String> {
    let before = read_optional(path)?;
    write_optional(path, Some(content))?;
    record(session, path, before, content);
    Ok(())
}

/// Record a change another writer already made (`before` = prior content).
pub(super) fn record(session: Option<&str>, path: &Path, before: Option<Vec<u8>>, content: &[u8]) {
    let Some(session) = session else {
        return;
    };
    let mut journals = JOURNALS.lock();
    let journal = journals.entry(session.to_string()).or_default();
    let turn = journal.turn;
    if journal.turns.last().is_none_or(|last| last.turn != turn) {
        journal.turns.push(TurnChanges {
            turn,
            ..TurnChanges::default()
        });
        if journal.turns.len() > MAX_TURNS {
            journal.turns.remove(0);
        }
    }
    let Some(current) = journal.turns.last_mut() else {
        return;
    };
    // A new change after a revert starts the turn's record over.
    current.reverted_over = None;
    match current.files.iter_mut().find(|change| change.path == path) {
        Some(change) => change.after = Some(content.to_vec()),
        None => current.files.push(FileChange {
            path: path.to_path_buf(),
            before,
            after: Some(content.to_vec()),
        }),
    }
}

pub(crate) fn latest(session: &str) -> Option<ChangeSummary> {
    let journals = JOURNALS.lock();
    let turn = journals.get(session)?.turns.last()?;
    Some(ChangeSummary {
        paths: turn
            .files
            .iter()
            .map(|change| change.path.clone())
            .collect(),
        reverted: turn.reverted_over.is_some(),
    })
}

/// Put back what the latest changing turn replaced.
pub(crate) fn revert_latest(session: &str) -> Result<RevertReport, String> {
    let mut journals = JOURNALS.lock();
    let turn = journals
        .get_mut(session)
        .and_then(|journal| journal.turns.last_mut())
        .ok_or("The model hasn't changed any files in this chat.")?;
    if turn.reverted_over.is_some() {
        return Err("Those changes are already reverted.".into());
    }
    let mut report = RevertReport::default();
    let mut over = Vec::new();
    for change in &turn.files {
        let current = match read_optional(&change.path) {
            Ok(current) => current,
            Err(error) => {
                report.failed.push((change.path.clone(), error));
                continue;
            }
        };
        if current != change.after {
            report.changed_since.push(change.path.clone());
        }
        match write_optional(&change.path, change.before.as_deref()) {
            Ok(()) => {
                report.restored.push(change.path.clone());
                over.push((change.path.clone(), current));
            }
            Err(error) => report.failed.push((change.path.clone(), error)),
        }
    }
    turn.reverted_over = Some(over);
    Ok(report)
}

/// Bring back what the last revert replaced.
pub(crate) fn undo_revert(session: &str) -> Result<RevertReport, String> {
    let mut journals = JOURNALS.lock();
    let turn = journals
        .get_mut(session)
        .and_then(|journal| journal.turns.last_mut())
        .ok_or("Nothing to bring back.")?;
    let over = turn.reverted_over.take().ok_or("Nothing was reverted.")?;
    let mut report = RevertReport::default();
    for (path, content) in over {
        match write_optional(&path, content.as_deref()) {
            Ok(()) => report.restored.push(path),
            Err(error) => report.failed.push((path, error)),
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revert_and_undo_round_trip_including_created_files() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("a.txt");
        let created = dir.path().join("new/b.txt");
        std::fs::write(&existing, "one").unwrap();
        let session = uuid::Uuid::new_v4().to_string();
        begin_turn(&session);
        apply(Some(&session), &existing, b"two").unwrap();
        apply(Some(&session), &existing, b"three").unwrap();
        apply(Some(&session), &created, b"hello").unwrap();
        let summary = latest(&session).unwrap();
        assert_eq!(summary.paths, vec![existing.clone(), created.clone()]);
        assert!(!summary.reverted);

        let report = revert_latest(&session).unwrap();
        assert_eq!(report.restored.len(), 2);
        assert!(report.changed_since.is_empty());
        assert_eq!(std::fs::read_to_string(&existing).unwrap(), "one");
        assert!(!created.exists());
        assert!(latest(&session).unwrap().reverted);
        assert!(revert_latest(&session).is_err());

        undo_revert(&session).unwrap();
        assert_eq!(std::fs::read_to_string(&existing).unwrap(), "three");
        assert_eq!(std::fs::read_to_string(&created).unwrap(), "hello");
        forget(&session);
    }

    #[test]
    fn a_file_changed_after_the_agent_is_reported_and_comes_back_on_undo() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("c.txt");
        std::fs::write(&file, "base").unwrap();
        let session = uuid::Uuid::new_v4().to_string();
        begin_turn(&session);
        apply(Some(&session), &file, b"agent").unwrap();
        std::fs::write(&file, "user edit").unwrap();
        let report = revert_latest(&session).unwrap();
        assert_eq!(report.changed_since, vec![file.clone()]);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "base");
        undo_revert(&session).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "user edit");
        forget(&session);
    }

    #[test]
    fn only_the_latest_changing_turn_is_reverted() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("d.txt");
        let session = uuid::Uuid::new_v4().to_string();
        begin_turn(&session);
        apply(Some(&session), &file, b"first").unwrap();
        begin_turn(&session);
        apply(Some(&session), &file, b"second").unwrap();
        begin_turn(&session); // a turn with no changes
        revert_latest(&session).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "first");
        forget(&session);
    }
}
