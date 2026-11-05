//! Pure selection rules for a bounded personal view. A late snapshot keeps the person's
//! chosen identity, not its old position. If that target disappeared, Return has no owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub detail: Option<String>,
    pub icon: &'static str,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonalList {
    query: String,
    items: Vec<Item>,
    visible: Vec<usize>,
    chosen: Option<String>,
    deliberate: bool,
}
impl PersonalList {
    pub fn rows(&self) -> impl ExactSizeIterator<Item = &Item> {
        self.visible.iter().map(|&i| &self.items[i])
    }
    pub fn selected(&self) -> Option<&Item> {
        self.rows().find(|r| Some(&r.id) == self.chosen.as_ref())
    }
    pub fn replace(&mut self, items: Vec<Item>) {
        self.items = items;
        self.refilter();
    }
    /// Return commits the row the person actually saw, not an old index or a note
    /// that still exists somewhere outside this view. A changed row must be chosen again.
    pub fn confirm_refreshed(&mut self, shown: &Item, refreshed: Vec<Item>) -> bool {
        let was_selected = self.selected() == Some(shown);
        self.replace(refreshed);
        if was_selected && self.selected() == Some(shown) {
            return true;
        }
        self.chosen = None;
        self.deliberate = true;
        false
    }
    pub fn type_query(&mut self, query: String) {
        self.query = query;
        self.deliberate = false;
        self.refilter();
    }
    fn refilter(&mut self) {
        let query = self.query.to_lowercase();
        self.visible = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                let title = r.title.to_lowercase();
                query.split_whitespace().all(|w| title.contains(w))
            })
            .map(|(i, _)| i)
            .collect();
        if self.selected().is_none() {
            self.chosen = if self.deliberate {
                None
            } else {
                self.visible.first().map(|&i| self.items[i].id.clone())
            };
        }
    }
    pub fn pick(&mut self, id: &str) -> bool {
        if !self.rows().any(|r| r.id == id) {
            return false;
        }
        self.deliberate = true;
        self.chosen = Some(id.to_owned());
        true
    }
    pub fn arrow(&mut self, down: bool) {
        if self.visible.is_empty() {
            return;
        }
        let ix = self
            .rows()
            .position(|r| Some(&r.id) == self.chosen.as_ref());
        let last = self.visible.len() - 1;
        let next = match (ix, down) {
            (Some(i), true) => (i + 1).min(last),
            (Some(i), false) => i.saturating_sub(1),
            (None, true) => 0,
            (None, false) => last,
        };
        self.chosen = Some(self.items[self.visible[next]].id.clone());
        self.deliberate = true;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn item(id: &str, title: &str) -> Item {
        Item {
            id: id.into(),
            title: title.into(),
            detail: None,
            icon: "file",
        }
    }
    #[test]
    fn return_allows_a_reordered_identity_but_never_a_changed_or_now_excluded_row() {
        let mut list = PersonalList::default();
        let shown = item("one", "A draft");
        let other = item("two", "B draft");
        list.replace(vec![shown.clone(), other.clone()]);
        assert!(list.confirm_refreshed(&shown, vec![other.clone(), shown.clone()]));
        let renamed = item("one", "Renamed after selection");
        assert!(!list.confirm_refreshed(&shown, vec![renamed.clone(), other.clone()]));
        assert!(list.selected().is_none());
        assert!(list.pick("one"));
        assert!(!list.confirm_refreshed(&renamed, vec![other.clone()]));
        assert!(list.selected().is_none());
        list.arrow(true);
        assert_eq!(list.selected(), Some(&other));
    }
    #[test]
    fn a_late_source_respects_typed_words_and_never_substitutes_for_a_lost_choice() {
        let mut v = PersonalList::default();
        v.type_query("deployment".into());
        assert!(v.selected().is_none());
        let a = item("a", "Deployment");
        let b = item("b", "Deployment");
        let c = item("c", "Unrelated");
        v.replace(vec![c.clone(), a.clone(), b.clone()]);
        assert_eq!(v.selected(), Some(&a));
        v.arrow(true);
        v.replace(vec![b.clone(), c.clone(), a.clone()]);
        assert_eq!(v.selected(), Some(&b));
        v.replace(vec![c, a.clone()]);
        assert!(v.selected().is_none());
        assert!(!v.pick("b"));
        v.arrow(true);
        assert_eq!(v.selected(), Some(&a));
    }
}
