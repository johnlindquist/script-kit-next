//! The list under the input.
//!
//! Names match instantly on top; three rows are kept for Jev from the first key and fill in
//! place. Every list starts with a header, and typing changes the header's words, never whether
//! it is there. A row read from older words never owns Enter, and a weak fuzzy name never does.

use crate::Ms;
use crate::contract::{ASK_LEAD_P, JEV_ROWS, MIN_READING_P, ROW_ARM_MS, words};
use crate::enter::EnterTarget;
use crate::row::{Row, RowAct};
use crate::steady::Steady;

/// One of Jev's reserved rows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JevSlot {
    pub row: Option<Row>,
    /// The row this one replaced; Enter keeps running it for [`ROW_ARM_MS`].
    pub prev: Option<Row>,
    pub shown_at: Ms,
    /// Read from older words than the input has now.
    pub stale: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JevSlots {
    slots: Vec<JevSlot>,
    answered_for: Option<String>,
    down_for: Option<String>,
}

impl Default for JevSlots {
    fn default() -> Self {
        Self::new(JEV_ROWS)
    }
}

impl JevSlots {
    pub fn new(count: usize) -> Self {
        Self {
            slots: vec![JevSlot::default(); count],
            answered_for: None,
            down_for: None,
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn get(&self, i: usize) -> Option<&JevSlot> {
        self.slots.get(i)
    }

    pub fn reset(&mut self) {
        let n = self.slots.len();
        *self = Self::new(n);
    }

    /// The input changed: everything Jev said is about older words now.
    pub fn on_text(&mut self, text: &str) {
        if text.trim().is_empty() {
            self.reset();
            return;
        }
        for slot in &mut self.slots {
            slot.stale = slot.row.is_some();
        }
    }

    /// Jev answered for the current words. A row that is still in the answer keeps its place;
    /// new rows fill the empty places in order.
    pub fn assign(&mut self, now: Ms, text: &str, rows: Vec<Row>) {
        let mut incoming: Vec<Row> = rows;
        let mut next: Vec<Option<Row>> = vec![None; self.slots.len()];
        for (i, slot) in self.slots.iter().enumerate() {
            if let Some(row) = &slot.row {
                if let Some(ix) = incoming.iter().position(|r| r.key == row.key) {
                    next[i] = Some(incoming.remove(ix));
                }
            }
        }
        for place in next.iter_mut() {
            if place.is_none() && !incoming.is_empty() {
                *place = Some(incoming.remove(0));
            }
        }
        for (slot, row) in self.slots.iter_mut().zip(next) {
            let changed =
                row.as_ref().map(|r| r.key.as_str()) != slot.row.as_ref().map(|r| r.key.as_str());
            let prev = if slot.stale {
                None
            } else if changed {
                slot.row.take()
            } else {
                slot.prev.take()
            };
            *slot = JevSlot {
                shown_at: if changed { now } else { slot.shown_at },
                row,
                prev,
                stale: false,
            };
        }
        self.answered_for = Some(text.to_string());
        self.down_for = None;
    }

    /// Jev didn't answer: its rows settle in place into rows that don't need it.
    pub fn down(&mut self, now: Ms, text: &str) {
        let rows = down_rows(text);
        for (slot, row) in self
            .slots
            .iter_mut()
            .zip(rows.into_iter().map(Some).chain(std::iter::repeat(None)))
        {
            *slot = JevSlot {
                row,
                prev: None,
                shown_at: now,
                stale: false,
            };
        }
        self.answered_for = Some(text.to_string());
        self.down_for = Some(text.to_string());
    }

    pub fn is_down_for(&self, text: &str) -> bool {
        self.down_for.as_deref() == Some(text)
    }

    /// Jev has answered the current words; a row it left empty is settled, not reading.
    pub fn settled_for(&self, text: &str) -> bool {
        self.answered_for.as_deref() == Some(text)
    }

    /// The row Enter runs in place `i`: none while stale; the replaced row while arming.
    pub fn effective(&self, i: usize, now: Ms) -> Option<&Row> {
        let slot = self.slots.get(i)?;
        let row = slot.row.as_ref()?;
        if slot.stale {
            return None;
        }
        match &slot.prev {
            Some(prev) if now.saturating_sub(slot.shown_at) < ROW_ARM_MS => Some(prev),
            _ => Some(row),
        }
    }

    /// When a row's arming ends and Enter moves to it.
    pub fn next_wake(&self, now: Ms) -> Option<Ms> {
        self.slots
            .iter()
            .filter(|s| s.prev.is_some() && now < s.shown_at + ROW_ARM_MS)
            .map(|s| s.shown_at + ROW_ARM_MS)
            .min()
    }
}

/// Jev's readings worth a place: likely enough, not a sentence already learned, and not a name
/// that's already listed (unless the reading does something to it).
pub fn fresh_readings(rows: Vec<Row>, names: &[Row]) -> Vec<Row> {
    let strong: Vec<String> = names
        .iter()
        .filter(|r| r.is_strong())
        .map(|r| r.title.to_lowercase())
        .collect();
    let learned: Vec<&str> = names
        .iter()
        .filter(|r| r.learned)
        .map(|r| r.key.strip_prefix("learned:").unwrap_or(&r.key))
        .collect();
    rows.into_iter()
        .filter(|r| r.p.is_none_or(|p| p >= MIN_READING_P))
        .filter(|r| !learned.contains(&r.key.strip_prefix("jev:").unwrap_or(&r.key)))
        .filter(|r| {
            let target = r.target.as_deref().unwrap_or_default().to_lowercase();
            !(r.verb.is_none() && strong.contains(&target))
        })
        .filter(|r| {
            let fill = r.fill.as_deref().unwrap_or_default().to_lowercase();
            fill.is_empty() || !strong.contains(&fill)
        })
        .collect()
}

/// Whether an Enter that waited for Jev should ask once instead of running `held`: it isn't Jev's
/// top reading, or the next reading is too close behind.
pub fn unsure(held: &Row, rows: &[Row]) -> bool {
    let mut ps: Vec<(&str, f32)> = rows
        .iter()
        .map(|r| (r.key.as_str(), r.p.unwrap_or(0.0)))
        .collect();
    ps.sort_by(|a, b| b.1.total_cmp(&a.1));
    match ps.as_slice() {
        [] => false,
        [(top, _)] => *top != held.key,
        [(top, p), (_, next), ..] => *top != held.key || p - next < ASK_LEAD_P,
    }
}

/// What Jev's reserved rows become when it doesn't answer. None is a reading of the sentence and
/// none needs Jev; the first is where Enter already was.
pub fn down_rows(text: &str) -> Vec<Row> {
    let t = text.trim();
    let said: String = if t.chars().count() > 44 {
        let head: String = t.chars().take(43).collect();
        format!("{head}…")
    } else {
        t.to_string()
    };
    vec![
        Row::new("down:model", format!("Ask the model: “{said}”")).with_icon("ask"),
        Row::new("down:retry", "Ask Jev again")
            .with_icon("again")
            .with_act(RowAct::AskJevAgain),
        Row::new("down:note", "Add it to today's note").with_icon("append"),
    ]
}

/// What the list starts with when its first row brings no header of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lead {
    Nothing,
    Row,
    Fallback,
}

/// The words of the header that leads a list whose first row has none.
pub fn lead_header(has_text: bool, lead: Lead) -> &'static str {
    match (has_text, lead) {
        (false, Lead::Nothing) => words::SUGGESTED,
        (false, Lead::Row | Lead::Fallback) => words::PINNED,
        (true, Lead::Row) => words::MATCHES,
        (true, Lead::Nothing | Lead::Fallback) => words::NO_MATCHES,
    }
}

/// A row as the list lays it out.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A section header; every list starts with one.
    Header(String),
    /// A pinned row (suggested names, the staged subject's actions, finished work).
    Pinned(usize),
    /// A name match (or a fallback when nothing matched).
    Name(usize),
    /// One of Jev's reserved places.
    Slot(usize),
}

impl Item {
    pub fn is_header(&self) -> bool {
        matches!(self, Item::Header(_))
    }
}

/// What the list shows and where Enter points.
#[derive(Clone, Debug, Default)]
pub struct ListState {
    pub text: String,
    pub names: Vec<Row>,
    pub pinned: Vec<Row>,
    pub jev: JevSlots,
    pub sel: usize,
    /// Enter was pressed on a Jev place that had nothing to run yet; it runs when Jev answers.
    pub held: Option<usize>,
    /// Jev's reading was close to another; the next Enter confirms.
    pub asking: bool,
    /// Jev's places come first when no name is strong enough to own Enter.
    pub jev_first: bool,
    /// A stale Jev row never owns Enter.
    pub stale_holds: bool,
    /// Keeps ↵'s words steady while a sentence is typed forward.
    pub steady: Steady,
}

impl ListState {
    pub fn new(jev_first: bool, stale_holds: bool) -> Self {
        Self {
            jev_first,
            stale_holds,
            ..Self::default()
        }
    }

    fn has_text(&self) -> bool {
        !self.text.trim().is_empty()
    }

    fn has_strong_name(&self) -> bool {
        self.names
            .iter()
            .any(|r| !r.fallback && r.is_strong() && self.steady.may_own(&r.key))
    }

    /// The input changed.
    pub fn on_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.sel = 0;
        self.held = None;
        self.asking = false;
        self.jev.on_text(text);
    }

    /// Selectable rows in order, without headers.
    pub fn visible(&self) -> Vec<Item> {
        let mut out: Vec<Item> = (0..self.pinned.len()).map(Item::Pinned).collect();
        if !self.has_text() {
            return out;
        }
        let names = (0..self.names.len()).map(Item::Name);
        let slots = (0..self.jev.len()).map(Item::Slot);
        if self.jev_first && !self.has_strong_name() {
            out.extend(slots);
            out.extend(names);
        } else {
            out.extend(names);
            out.extend(slots);
        }
        out
    }

    /// The list as drawn: headers in place. The first item is always a header.
    pub fn layout(&self) -> Vec<Item> {
        let visible = self.visible();
        let mut out = Vec::with_capacity(visible.len() + 2);
        let mut jev_head = false;
        for (ix, item) in visible.iter().enumerate() {
            match item {
                Item::Pinned(i) | Item::Name(i) => {
                    let row = self.row(item).expect("row for index");
                    let _ = i;
                    if let Some(head) = &row.head {
                        out.push(Item::Header(head.clone()));
                    } else if ix == 0 {
                        let lead = if row.fallback {
                            Lead::Fallback
                        } else {
                            Lead::Row
                        };
                        out.push(Item::Header(lead_header(self.has_text(), lead).to_string()));
                    }
                }
                Item::Slot(_) => {
                    if !jev_head {
                        jev_head = true;
                        out.push(Item::Header(words::JEV.to_string()));
                    }
                }
                Item::Header(_) => {}
            }
            out.push(item.clone());
        }
        if out.is_empty() {
            out.push(Item::Header(
                lead_header(self.has_text(), Lead::Nothing).to_string(),
            ));
        }
        out
    }

    /// The first header's words.
    pub fn header(&self) -> String {
        match self.layout().into_iter().next() {
            Some(Item::Header(words)) => words,
            _ => String::new(),
        }
    }

    pub fn row(&self, item: &Item) -> Option<&Row> {
        match item {
            Item::Pinned(i) => self.pinned.get(*i),
            Item::Name(i) => self.names.get(*i),
            Item::Slot(i) => self.jev.get(*i).and_then(|s| s.row.as_ref()),
            Item::Header(_) => None,
        }
    }

    pub fn current(&self) -> Option<Item> {
        self.visible().get(self.sel).cloned()
    }

    /// The row Enter would run now, if any.
    pub fn current_row(&self, now: Ms) -> Option<Row> {
        match self.current()? {
            Item::Slot(i) => self.effective(i, now).cloned(),
            item => self.row(&item).cloned(),
        }
    }

    pub fn effective(&self, i: usize, now: Ms) -> Option<&Row> {
        let slot = self.jev.get(i)?;
        let row = slot.row.as_ref()?;
        if self.stale_holds && slot.stale {
            return None;
        }
        match &slot.prev {
            Some(prev) if now.saturating_sub(slot.shown_at) < ROW_ARM_MS => Some(prev),
            _ => Some(row),
        }
    }

    /// What the ↵ button says, with an id that changes only when the target does.
    pub fn target(&self, now: Ms) -> EnterTarget {
        let Some(item) = self.current() else {
            return EnterTarget::new("none", words::ENTER_NOTHING);
        };
        match item {
            Item::Slot(i) => {
                let slot = self.jev.get(i).expect("slot");
                let eff = self.effective(i, now);
                let id = if slot.prev.is_some() && (eff.is_some() || !self.stale_holds) {
                    format!("slot{i}:{}", eff.map(|r| r.key.as_str()).unwrap_or(""))
                } else {
                    format!("slot{i}")
                };
                match eff.filter(|_| self.asking || self.steady.settled(now)) {
                    None => {
                        let label = if self.held.is_some() {
                            words::ENTER_WAITING
                        } else if self.jev_first && !self.has_strong_name() {
                            words::ENTER_JEV_FIRST
                        } else {
                            words::ENTER_WAIT
                        };
                        EnterTarget::new(id, label)
                    }
                    Some(row) if self.asking => EnterTarget::new(id, words::ask_again(&row.title)),
                    Some(row) => EnterTarget::new(id, row.title.clone()),
                }
            }
            item => {
                let row = self.row(&item).expect("row");
                EnterTarget::new(row.key.clone(), row.enter_words())
            }
        }
    }

    pub fn move_sel(&mut self, delta: i32) {
        let len = self.visible().len();
        if len == 0 {
            self.sel = 0;
            return;
        }
        let next = self.sel as i64 + delta as i64;
        self.sel = next.clamp(0, len as i64 - 1) as usize;
    }

    /// Position of Jev place `i` among the visible rows.
    pub fn slot_position(&self, i: usize) -> Option<usize> {
        self.visible().iter().position(|v| *v == Item::Slot(i))
    }
}

impl ListState {
    /// A header as the list draws it: Jev's carries the keys that run its rows.
    pub fn drawn_header(&self, head: &str) -> String {
        if head == words::JEV {
            format!("{head} · {}", words::jev_keys(self.jev.len()))
        } else {
            head.to_string()
        }
    }

    /// The host listed the names for the current words.
    pub fn names_came(&mut self) {
        let strong = self
            .names
            .iter()
            .filter(|r| !r.fallback && r.is_strong())
            .map(|r| r.key.as_str());
        self.steady.names(strong);
    }

    /// When a fresh reading that waits for the keys to pause may name ↵.
    pub fn reading_wake(&self, now: Ms) -> Option<Ms> {
        let waiting = (0..self.jev.len())
            .filter_map(|i| self.jev.get(i))
            .any(|s| s.row.is_some() && !s.stale);
        waiting.then(|| self.steady.settles_at(now)).flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(rows: &[(&str, u8)]) -> Vec<Row> {
        rows.iter()
            .map(|(t, s)| Row::new(format!("name:{t}"), *t).with_score(*s))
            .collect()
    }

    fn jev(keys: &[&str]) -> Vec<Row> {
        keys.iter()
            .map(|k| Row::new(*k, format!("Jev {k}")))
            .collect()
    }

    #[test]
    fn every_list_starts_with_a_header() {
        let mut l = ListState::new(true, true);
        assert!(l.layout()[0].is_header());
        l.on_text("sl");
        assert!(l.layout()[0].is_header());
        l.names = names(&[("Slack", 90)]);
        assert_eq!(l.header(), words::MATCHES);
        l.names = names(&[("Launcher", 20)]);
        assert_eq!(l.header(), words::JEV, "weak names put Jev first");
        l.pinned = vec![Row::new("s", "Slack").with_head(words::SUGGESTED)];
        l.on_text("");
        assert_eq!(l.header(), words::SUGGESTED);
    }

    #[test]
    fn typing_changes_the_lead_header_words_never_its_presence() {
        assert_eq!(lead_header(false, Lead::Nothing), words::SUGGESTED);
        assert_eq!(lead_header(false, Lead::Row), words::PINNED);
        assert_eq!(lead_header(true, Lead::Row), words::MATCHES);
        assert_eq!(lead_header(true, Lead::Fallback), words::NO_MATCHES);
        assert_eq!(lead_header(true, Lead::Nothing), words::NO_MATCHES);
        let mut l = ListState::new(false, true);
        for text in ["", "s", "sl", "", "zzz"] {
            l.on_text(text);
            assert!(
                l.layout()[0].is_header(),
                "{text:?} must lead with a header"
            );
        }
    }

    #[test]
    fn three_places_are_kept_from_the_first_key() {
        let mut l = ListState::new(false, true);
        l.on_text("w");
        let slots = l
            .visible()
            .iter()
            .filter(|i| matches!(i, Item::Slot(_)))
            .count();
        assert_eq!(slots, JEV_ROWS);
    }

    #[test]
    fn stale_rows_never_own_enter() {
        let mut l = ListState::new(true, true);
        l.on_text("when does");
        l.jev.assign(0, "when does", jev(&["a"]));
        assert!(l.current_row(1_000).is_some());
        l.on_text("when does lebron");
        assert!(l.current_row(1_000).is_none());
        assert_eq!(l.target(1_000).label, words::ENTER_JEV_FIRST);
    }

    #[test]
    fn a_replaced_row_keeps_enter_while_arming() {
        let mut l = ListState::new(true, false);
        l.on_text("x");
        l.jev.assign(0, "x", jev(&["a"]));
        l.jev.assign(1_000, "x", jev(&["b"]));
        assert_eq!(l.current_row(1_100).unwrap().key, "a");
        assert_eq!(l.current_row(1_000 + ROW_ARM_MS).unwrap().key, "b");
    }

    #[test]
    fn filling_an_empty_place_keeps_the_target_id() {
        let mut l = ListState::new(true, true);
        l.on_text("x");
        let before = l.target(0).id;
        l.jev.assign(10, "x", jev(&["a"]));
        assert_eq!(l.target(10).id, before);
    }

    #[test]
    fn rows_that_stay_in_the_answer_keep_their_place() {
        let mut s = JevSlots::new(3);
        s.assign(0, "t", jev(&["a", "b", "c"]));
        s.assign(10, "t", jev(&["c", "d", "a"]));
        assert_eq!(s.get(0).unwrap().row.as_ref().unwrap().key, "a");
        assert_eq!(s.get(1).unwrap().row.as_ref().unwrap().key, "d");
        assert_eq!(s.get(2).unwrap().row.as_ref().unwrap().key, "c");
    }

    #[test]
    fn a_waiting_enter_asks_unless_its_row_leads_clearly() {
        let reading = |k: &str, p: f32| Row {
            p: Some(p),
            ..Row::new(k, k)
        };
        let rows = [reading("a", 0.6), reading("b", 0.5), reading("c", 0.1)];
        assert!(unsure(&rows[0], &rows), "10 points ahead is too close");
        assert!(unsure(&rows[1], &rows), "not the top reading");
        let rows = [reading("a", 0.8), reading("b", 0.5)];
        assert!(!unsure(&rows[0], &rows));
    }

    #[test]
    fn down_rows_settle_in_place() {
        let mut l = ListState::new(true, true);
        l.on_text("book a table");
        l.jev.down(0, "book a table");
        assert!(l.jev.is_down_for("book a table"));
        assert_eq!(l.current_row(0).unwrap().key, "down:model");
    }
}
