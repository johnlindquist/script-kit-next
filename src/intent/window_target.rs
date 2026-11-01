//! Local-only observations. Never infer a mutation recipient from list order or a display name.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowIdentity {
    pub pid: i32,
    pub bundle: String,
    pub app: String,
    pub title: String,
    pub native_id: u32,
    pub generation: u64,
    pub nonce: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowTarget {
    pub identity: WindowIdentity,
    /// A deictic reference must still be the focused window of the originating app.
    pub front: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedWindow {
    pub identity: WindowIdentity,
    pub focused: bool,
    pub minimized: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub rows: Vec<ObservedWindow>,
    /// Tracked origin (PID, bundle, title), not Script Kit's panel.
    pub origin: Option<(i32, String, String)>,
    /// Another confirmed foreground process supersedes even a stale tracker entry.
    pub foreign_front: Option<i32>,
    pub scope: WindowScope,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum WindowScope {
    #[default]
    Screen,
    /// The visible window tab constrains "this", but doesn't override an explicit app name.
    WindowTab(Option<WindowTarget>),
    /// A staged non-window or an unobserved window grants no window authority.
    Staged(Option<WindowTarget>),
}

impl WindowSnapshot {
    pub fn bind(&self, app: &str, front: bool) -> Option<WindowTarget> {
        let front = match &self.scope {
            WindowScope::Staged(pin) => {
                let pin = pin.as_ref()?;
                if !pin.identity.app.eq_ignore_ascii_case(app) {
                    return None;
                }
                pin.front
            }
            _ => front,
        };
        let mut matching = self.rows.iter().filter(|row| {
            let w = &row.identity;
            w.app.eq_ignore_ascii_case(app)
                && (!front
                    || row.focused
                        && self.foreign_front.is_none_or(|pid| pid == w.pid)
                        && self
                            .origin
                            .as_ref()
                            .is_some_and(|(pid, bundle, _)| *pid == w.pid && bundle == &w.bundle))
        });
        let row = matching.next()?;
        let w = &row.identity;
        if matching.next().is_some()
            || (front && self.origin.as_ref()?.2 != w.title)
            || row.minimized
            || w.pid <= 0
            || w.native_id == 0
            || w.bundle.is_empty()
            || w.title.is_empty()
            || w.nonce == 0
            || w.generation == 0
            || self
                .rows
                .iter()
                .filter(|r| r.identity.pid == w.pid && r.identity.native_id == w.native_id)
                .count()
                != 1
        {
            return None;
        }
        let target = WindowTarget {
            identity: w.clone(),
            front,
        };
        match &self.scope {
            WindowScope::Screen => Some(target),
            WindowScope::WindowTab(_) if !front => Some(target),
            WindowScope::WindowTab(pin) | WindowScope::Staged(pin) => {
                pin.as_ref().filter(|p| p.identity == *w).cloned()
            }
        }
    }

    pub fn confirms(&self, target: &WindowTarget) -> bool {
        self.bind(&target.identity.app, target.front).as_ref() == Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> WindowSnapshot {
        let row = |id, focused| ObservedWindow {
            identity: WindowIdentity {
                pid: 7,
                bundle: "test.editor".into(),
                app: "Editor".into(),
                title: "Untitled".into(),
                native_id: id,
                generation: 3,
                nonce: u64::from(id) + 1000,
            },
            focused,
            minimized: false,
        };
        WindowSnapshot {
            rows: vec![row(42, false), row(43, true)],
            origin: Some((7, "test.editor".into(), "Untitled".into())),
            ..Default::default()
        }
    }

    #[test]
    fn two_documents_with_identical_titles_reorder_without_retargeting() {
        let mut s = fixture();
        let pin = s.bind("Editor", true).unwrap();
        assert_eq!(pin.identity.native_id, 43);
        s.rows.reverse();
        assert!(s.confirms(&pin));
        s.rows[0].focused = false;
        s.rows[1].focused = true;
        assert!(!s.confirms(&pin));
    }

    #[test]
    fn changed_process_generation_or_replacement_cannot_inherit_the_request() {
        let s = fixture();
        let pin = s.bind("Editor", true).unwrap();
        for change in 0..10 {
            let mut s = s.clone();
            let row = &mut s.rows[1];
            match change {
                0 => row.identity.generation += 1,
                1 => row.identity.nonce += 1,
                2 => row.identity.native_id = 99,
                3 => row.identity.pid += 1,
                4 => row.identity.bundle.push('x'),
                5 => row.identity.title.push('!'),
                6 => s.foreign_front = Some(8),
                7 => {
                    s.rows.pop();
                }
                8 => row.minimized = true,
                _ => s.rows[0].identity.native_id = 43,
            }
            assert!(!s.confirms(&pin), "case {change}");
        }
    }

    #[test]
    fn named_targets_need_one_window_and_staging_never_falls_back() {
        let mut s = fixture();
        assert!(s.bind("Editor", false).is_none());
        let pinned = s.bind("Editor", true).unwrap();
        s.scope = WindowScope::Staged(Some(pinned.clone()));
        assert_eq!(s.bind("Editor", false), Some(pinned));
        s.scope = WindowScope::Screen;
        s.rows.remove(0);
        let named = s.bind("Editor", false).unwrap();
        s.origin = None;
        assert!(s.confirms(&named));
        s.scope = WindowScope::Staged(None);
        assert!(s.bind("Editor", false).is_none());
    }
}
