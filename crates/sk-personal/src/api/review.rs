//! What a person would save, derived from the store's own typed request and the exact base
//! revision. Looking never creates a journal, reserves a write or grants access to a source.
use super::*;
use crate::views::*;

#[derive(Debug, PartialEq, Eq)]
pub struct SaveReview {
    pub title: &'static str,
    pub detail: String,
}

pub fn review_apply(root: &Path, args: Value, safe_mode: bool) -> Result<SaveReview> {
    if safe_mode {
        return Err(Error::SafeMode);
    }
    let Query::Apply { request } = Endpoint::Apply.query(args)? else {
        return Err(Error::InvalidRequest);
    };
    let store = Store::open(root)?;
    if request.workspace != store.workspace {
        return Err(Error::WrongWorkspace);
    }
    if request.request_id.get_version_num() != 4 || !valid_revision(&request.expected_revision) {
        return Err(Error::InvalidRequest);
    }
    let before = store.observed()?;
    if let Some(record) = store.record(request.request_id)? {
        if record.author != Author::ConfiguredAgent || record.request != request {
            return Err(Error::RequestConflict);
        }
        if store.contains(&before, request.request_id)? {
            let mut review = describe(&store, &request.change, &record.before.doc, &record.after)?;
            review.detail.push_str(
                " This request was saved earlier; an identical replay returns its receipt.",
            );
            return Ok(review);
        }
    }
    if request.expected_revision != before.revision {
        return Err(Error::StaleRevision);
    }
    let (after, _) = store.proposed(&before, &request.change, None)?;
    describe(&store, &request.change, &before.doc, &after)
}

fn describe(
    store: &Store,
    change: &Change,
    before: &Document,
    after: &Document,
) -> Result<SaveReview> {
    let mut facts = match change {
        Change::Undo { of } => {
            let record = store.record(*of)?.ok_or(Error::NothingToUndo)?;
            let restoring = match record.request.change {
                Change::Edit { edit } => Change::Edit {
                    edit: after.rules.edit_at(edit),
                },
                Change::PutView {
                    view: ViewRecipe { id, .. },
                }
                | Change::RemoveView { id } => match after.views.iter().find(|v| v.id == id) {
                    Some(v) => Change::PutView { view: v.clone() },
                    None => Change::RemoveView { id },
                },
                Change::Enable | Change::Disable => {
                    if after.enabled {
                        Change::Enable
                    } else {
                        Change::Disable
                    }
                }
                Change::Undo { .. } => return Err(Error::InvalidDocument),
            };
            SaveReview {
                title: "Undo a launcher change",
                detail: explain(&restoring),
            }
        }
        change => SaveReview {
            title: match change {
                Change::PutView { view } if before.views.iter().any(|v| v.id == view.id) => {
                    "Replace a personal list"
                }
                Change::PutView { .. } => "Save a personal list",
                Change::RemoveView { .. } => "Remove a personal list",
                _ => "Change your launcher",
            },
            detail: explain(change),
        },
    };
    if !after.enabled && !matches!(change, Change::Disable) {
        facts
            .detail
            .push_str(" Personalizations are off; Kit will keep using stock settings.");
    }
    Ok(facts)
}

fn explain(change: &Change) -> String {
    match change {
        Change::Edit {
            edit: Edit::PinWeight(weight),
        } => match weight {
            Weight::Regular => "Use regular weight for “Stays open” whenever Kit is pinned.",
            Weight::Semibold => {
                "Use semibold for “Stays open” whenever Kit is pinned, even when not dictating."
            }
            Weight::SemiboldWhileDictating => {
                "Use semibold for “Stays open” only while dictating; otherwise use regular weight."
            }
        }
        .into(),
        Change::Edit { .. } => format!("{}.", crate::speech::label(change)),
        Change::Disable => "Use the stock launcher. Keep your saved rules and lists.".into(),
        Change::Enable => "Use your saved launcher rules and lists again.".into(),
        Change::RemoveView { id } => {
            format!("Remove the list with ID {id}. No note or pane will be changed.")
        }
        Change::PutView { view } => {
            let (source, order, detail, action) = match view.list {
                ListSpec::Notes {
                    filter,
                    sort,
                    detail,
                    enter: NoteAction::OpenNote,
                } => (
                    match filter {
                        NoteFilter::Pinned => "Pinned note titles only",
                        NoteFilter::All => "All note titles",
                    },
                    match sort {
                        NoteSort::Title => "by title",
                        NoteSort::LastChanged => "last changed first",
                    },
                    match detail {
                        NoteDetail::None => "no row detail",
                        NoteDetail::LastChanged => "show last change",
                        NoteDetail::Pinned => "show pin state",
                    },
                    "open the note",
                ),
                ListSpec::Panes {
                    filter,
                    sort,
                    detail,
                    enter,
                } => (
                    match filter {
                        PaneFilter::NeedsAttention => "Herdr panes needing attention only",
                        PaneFilter::All => "All Herdr panes",
                    },
                    match sort {
                        PaneSort::Title => "by title",
                        PaneSort::AttentionFirst => "attention first",
                    },
                    match detail {
                        PaneDetail::None => "no row detail",
                        PaneDetail::Space => "show space",
                        PaneDetail::Status => "show status",
                    },
                    match enter {
                        PaneAction::None => "no action (read only)",
                        PaneAction::Focus => "focus the pane",
                        PaneAction::FocusAndZoom => "focus and zoom the pane",
                    },
                ),
            };
            format!(
                "“{}” (ID: {})\n{source}; {order}; up to {} rows; {detail}. Return: {action}. Saving grants no access.",
                view.title, view.id, view.limit
            )
        }
        // Undo needs the journal and is resolved before calling this helper.
        Change::Undo { .. } => "Undo the named launcher change.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_two_weight_proposals_differ_before_saving_and_undo_names_the_restored_weight() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let store = Store::open(root).unwrap();
        let state = store.inspect().unwrap();
        let first = Request {
            workspace: state.workspace,
            expected_revision: state.revision,
            request_id: Uuid::new_v4(),
            change: Change::Edit {
                edit: Edit::PinWeight(Weight::Semibold),
            },
        };
        let mut second = first.clone();
        second.change = Change::Edit {
            edit: Edit::PinWeight(Weight::SemiboldWhileDictating),
        };
        let inspect =
            |req: &Request| review_apply(root, json!({"op":"apply","request":req}), false);
        assert!(
            inspect(&first)
                .unwrap()
                .detail
                .contains("even when not dictating")
        );
        assert!(
            inspect(&second)
                .unwrap()
                .detail
                .contains("only while dictating")
        );
        assert!(!root.join("db").exists() && !root.join("plugins").exists());
        let saved = store.apply(first.clone(), Author::ConfiguredAgent).unwrap();
        assert!(
            inspect(&first)
                .unwrap()
                .detail
                .contains("even when not dictating")
        );
        assert_eq!(inspect(&second), Err(Error::RequestConflict));
        let mut undo = first.clone();
        undo.request_id = Uuid::new_v4();
        undo.expected_revision = saved.applied_revision;
        undo.change = Change::Undo {
            of: first.request_id,
        };
        assert!(inspect(&undo).unwrap().detail.contains("regular weight"));
        undo.expected_revision = "stock".into();
        assert_eq!(inspect(&undo), Err(Error::StaleRevision));
    }
}
