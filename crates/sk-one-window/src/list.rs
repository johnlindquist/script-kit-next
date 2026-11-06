//! The list under the input.
//!
//! Names match instantly on top; Jev's first place shows from the first key, and its readings
//! fill in place, up to three. A later place with nothing in it takes no room, and when Jev has
//! nothing worth a row its place offers what doesn't need it. Every list starts with a header,
//! and typing changes the header's words, never whether it is there. A row read from older words never owns Enter, and a weak fuzzy name never does.
//!
//! The highlighted row is the user's. What arrives on its own (another answer from Jev, Jev going
//! quiet, names searched again) lands around it: the row stays, the highlight stays on it, and
//! Enter runs it until the user types, moves or runs something.

use crate::Ms;
use crate::contract::{ASK_LEAD_P, JEV_ROWS, MIN_READING_P, ROW_ARM_MS, words};
use crate::enter::EnterTarget;
use crate::row::{Row, RowAct};
use crate::steady::Steady;

/// One of Jev's reserved rows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JevSlot {
    pub row: Option<Row>,
    /// The row this one replaced; the place's ⌘ digit keeps running it for [`ROW_ARM_MS`].
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

    /// Jev answered for the current words. A row that is still in the answer keeps its place, and
    /// the highlighted row (`keep`) stays as it was even when the answer leaves it out; new rows
    /// fill the empty places in order, and the rows close up so no empty place sits between them.
    pub fn assign(&mut self, now: Ms, text: &str, rows: Vec<Row>, keep: Option<&str>) {
        let mut incoming: Vec<Row> = rows;
        let mut next: Vec<Option<Row>> = vec![None; self.slots.len()];
        for (i, slot) in self.slots.iter().enumerate() {
            let Some(row) = &slot.row else {
                continue;
            };
            let again = incoming.iter().position(|r| r.key == row.key);
            if keep == Some(row.key.as_str()) {
                if let Some(ix) = again {
                    incoming.remove(ix);
                }
                next[i] = Some(row.clone());
            } else if let Some(ix) = again {
                next[i] = Some(incoming.remove(ix));
            }
        }
        for place in next.iter_mut() {
            if place.is_none() && !incoming.is_empty() {
                *place = Some(incoming.remove(0));
            }
        }
        next.sort_by_key(Option::is_none);
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

    /// Jev didn't answer: its rows settle in place into rows that don't need it. The highlighted
    /// row (`keep`) stays where it is.
    pub fn down(&mut self, now: Ms, text: &str, keep: Option<&str>) {
        let kept = self
            .slots
            .iter()
            .position(|s| s.row.as_ref().is_some_and(|r| keep == Some(r.key.as_str())));
        // Only a failed/offline reading admits the local splitter. Use these same
        // reserved rows so a second name match can never race the Jev result.
        let mut fallback = down_rows(text);
        if let Some(local) = crate::steps::offline_row(text) {
            fallback.insert(0, local);
        }
        let mut rows = fallback
            .into_iter()
            .filter(|row| keep != Some(row.key.as_str()));
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if kept == Some(i) {
                slot.prev = None;
                continue;
            }
            *slot = JevSlot {
                row: rows.next(),
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

    /// Jev read the current words itself: an answer, not stand-ins for one that never came.
    pub fn read(&self, text: &str) -> bool {
        self.settled_for(text) && !self.is_down_for(text)
    }

    /// The row place `i`'s ⌘ digit runs: none while stale; the replaced row while arming.
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

    /// When a row's arming ends and its ⌘ digit moves to it.
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

/// What Jev's places offer when it answered but had nothing worth a row: the rows that don't
/// need it, without asking it again.
pub fn stand_in_rows(text: &str) -> Vec<Row> {
    down_rows(text)
        .into_iter()
        .filter(|row| row.act != RowAct::AskJevAgain)
        .collect()
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
    /// The Owner is decided: only the pinned row shows.
    pub owned: bool,
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
        if !self.has_text() || self.owned {
            return out;
        }
        // When something matched, the fallbacks go last, after Jev's rows; when nothing did, they
        // stand where the names would.
        let len = self.names.len();
        let split = match self.names.iter().position(|r| r.fallback) {
            Some(0) | None => len,
            Some(at) => at,
        };
        let names = (0..split).map(Item::Name);
        let slots = (0..self.jev.len())
            .filter(|&i| self.slot_drawn(i))
            .map(Item::Slot);
        if self.jev_first && !self.has_strong_name() {
            out.extend(slots);
            out.extend(names);
        } else {
            out.extend(names);
            out.extend(slots);
        }
        out.extend((split..len).map(Item::Name));
        out
    }

    /// Whether Jev's place `i` takes a row in the list: the first always does, so what owns ↵
    /// never comes and goes while a sentence is typed; the others only while they hold a row.
    fn slot_drawn(&self, i: usize) -> bool {
        i == 0 || self.jev.get(i).is_some_and(|slot| slot.row.is_some())
    }

    /// How many of Jev's places the list draws.
    pub fn drawn_slots(&self) -> usize {
        (0..self.jev.len()).filter(|&i| self.slot_drawn(i)).count()
    }

    /// Jev is still reading the current words: its places are in the list and it hasn't answered.
    pub fn reading(&self) -> bool {
        self.has_text() && !self.owned && !self.jev.settled_for(&self.text)
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

    /// The row Enter would run now, if any: the one the highlight is on.
    pub fn current_row(&self) -> Option<Row> {
        match self.current()? {
            Item::Slot(i) => self.slot_row(i).cloned(),
            item => self.row(&item).cloned(),
        }
    }

    /// The row the highlight is on, when it's one the user can act on: a name, a pinned row, or a
    /// reading of the current words. Nothing that arrives on its own takes it away.
    pub fn focused(&self) -> Option<&Row> {
        match self.current()? {
            Item::Slot(i) => self
                .jev
                .get(i)
                .filter(|slot| !slot.stale)
                .and_then(|slot| slot.row.as_ref()),
            item => self.row(&item),
        }
    }

    /// What Enter runs with the highlight on Jev's place `i`: the row drawn there, unless it was
    /// read from older words.
    pub fn slot_row(&self, i: usize) -> Option<&Row> {
        let slot = self.jev.get(i)?;
        if self.stale_holds && slot.stale {
            return None;
        }
        slot.row.as_ref()
    }

    /// The row place `i`'s ⌘ digit runs: the one it replaced a moment ago, while arming.
    pub fn effective(&self, i: usize, now: Ms) -> Option<&Row> {
        let slot = self.jev.get(i)?;
        let row = self.slot_row(i)?;
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
                let id = format!("slot{i}");
                match self
                    .slot_row(i)
                    // Only a reading waits for the typing pause. Once Jev is down for these
                    // exact words, the selected local action is already what Return runs.
                    .filter(|_| {
                        self.asking || self.jev.is_down_for(&self.text) || self.steady.settled(now)
                    }) {
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
                    Some(row) => EnterTarget::new(id, row.enter_words()),
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

    /// Position of the row with `key` among the visible rows.
    fn position_of(&self, key: &str) -> Option<usize> {
        self.visible()
            .iter()
            .position(|v| self.row(v).is_some_and(|row| row.key == key))
    }

    /// Puts the highlight back after rows opened or closed around it: on the row with `key`
    /// wherever it went, or else on `item`. A place that closed hands it to the last of Jev's
    /// rows still drawn.
    pub fn reselect(&mut self, item: Option<Item>, key: Option<&str>) {
        if let Some(at) = key.and_then(|key| self.position_of(key)) {
            self.sel = at;
            return;
        }
        let visible = self.visible();
        let at = item.and_then(|item| {
            visible
                .iter()
                .position(|v| *v == item)
                .or_else(|| match item {
                    Item::Slot(_) => visible.iter().rposition(|v| matches!(v, Item::Slot(_))),
                    _ => None,
                })
        });
        match at {
            Some(at) => self.sel = at,
            None => self.move_sel(0),
        }
    }

    /// Rows arrived for the same words while the highlight was on `row` at `item`: a name the new
    /// ones left out goes back where it was, and the highlight goes back on it. Returns whether
    /// the highlight is on `row` again.
    pub fn keep(&mut self, item: Item, row: &Row) -> bool {
        if self.position_of(&row.key).is_none() {
            if let Item::Name(i) = item {
                // A kept name goes back among the names, never among the fallbacks after them.
                let names_end = match self.names.iter().position(|r| r.fallback) {
                    Some(at) if !row.fallback => at,
                    _ => self.names.len(),
                };
                let at = i.min(names_end);
                self.names.insert(at, row.clone());
            }
        }
        self.reselect(Some(item), Some(&row.key));
        self.focused().is_some_and(|now| now.key == row.key)
    }
}

impl ListState {
    /// A header as the list draws it: Jev's carries the keys that run its rows.
    pub fn drawn_header(&self, head: &str) -> String {
        if head == words::JEV {
            format!("{head} · {}", words::jev_keys(self.drawn_slots()))
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
            .map(|r| {
                // These are explicit targets, not former fuzzy winners. A text action
                // can disappear while the next word is incomplete ("translate t").
                let instruction = r.key.starts_with("subject/") && r.key.ends_with("/ask-words");
                (
                    r.key.as_str(),
                    instruction || r.title.trim().eq_ignore_ascii_case(self.text.trim()),
                )
            });
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

    fn keys(s: &JevSlots) -> Vec<Option<&str>> {
        (0..s.len())
            .map(|i| {
                s.get(i)
                    .and_then(|slot| slot.row.as_ref())
                    .map(|r| r.key.as_str())
            })
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

    fn slots(l: &ListState) -> Vec<Item> {
        l.visible()
            .into_iter()
            .filter(|i| matches!(i, Item::Slot(_)))
            .collect()
    }

    #[test]
    fn one_place_reads_from_the_first_key() {
        let mut l = ListState::new(false, true);
        l.on_text("w");
        assert_eq!(slots(&l), [Item::Slot(0)]);
    }

    #[test]
    fn places_jev_left_empty_take_no_room() {
        let mut l = ListState::new(true, true);
        l.on_text("open a new tab");
        l.jev.assign(0, "open a new tab", jev(&["a"]), None);
        assert_eq!(slots(&l), [Item::Slot(0)]);
        assert_eq!(l.drawn_header(words::JEV), format!("{} · ⌘1", words::JEV));
        l.jev
            .assign(10, "open a new tab", jev(&["a", "b", "c"]), None);
        assert_eq!(slots(&l).len(), JEV_ROWS);
        l.jev.assign(20, "open a new tab", jev(&["b"]), None);
        assert_eq!(slots(&l), [Item::Slot(0)]);
    }

    #[test]
    fn readings_of_older_words_keep_their_places_while_typing() {
        let mut l = ListState::new(true, true);
        l.on_text("open a new");
        l.jev.assign(0, "open a new", jev(&["a", "b"]), None);
        l.on_text("open a new tab");
        assert_eq!(slots(&l), [Item::Slot(0), Item::Slot(1)]);
    }

    #[test]
    fn rows_close_up_when_an_earlier_one_leaves() {
        let mut s = JevSlots::new(3);
        s.assign(0, "t", jev(&["a", "b", "c"]), None);
        s.assign(10, "t", jev(&["c"]), None);
        assert_eq!(keys(&s), [Some("c"), None, None]);
    }

    #[test]
    fn the_highlight_stays_on_its_row_when_places_close_above_it() {
        let mut l = ListState::new(true, true);
        l.on_text("zz");
        l.names = names(&[("Zed", 10)]);
        l.jev.assign(0, "zz", jev(&["a", "b", "c"]), None);
        l.sel = 3;
        assert_eq!(l.current(), Some(Item::Name(0)));
        let before = l.current();
        l.jev.assign(10, "zz", jev(&["a"]), None);
        l.reselect(before, Some("name:Zed"));
        assert_eq!(l.current(), Some(Item::Name(0)));
    }

    #[test]
    fn stale_rows_never_own_enter() {
        let mut l = ListState::new(true, true);
        l.on_text("when does");
        l.jev.assign(0, "when does", jev(&["a"]), None);
        assert!(l.current_row().is_some());
        l.on_text("when does lebron");
        assert!(l.current_row().is_none());
        assert!(
            l.focused().is_none(),
            "a stale row isn't the user's to keep"
        );
        assert_eq!(l.target(1_000).label, words::ENTER_JEV_FIRST);
    }

    #[test]
    fn another_answer_never_replaces_the_highlighted_reading() {
        let mut l = ListState::new(true, true);
        l.on_text("quit gmgui");
        l.jev.assign(0, "quit gmgui", jev(&["a"]), None);
        assert_eq!(l.focused().unwrap().key, "a");
        let keep = l.focused().map(|r| r.key.clone());
        l.jev
            .assign(1_000, "quit gmgui", jev(&["b", "c"]), keep.as_deref());
        l.reselect(Some(Item::Slot(0)), keep.as_deref());
        assert_eq!(keys(&l.jev), [Some("a"), Some("b"), Some("c")]);
        assert_eq!(l.current_row().unwrap().key, "a");
        assert_eq!(l.target(5_000).label, "Jev a");
    }

    #[test]
    fn the_highlighted_reading_keeps_its_words_when_jev_says_it_again() {
        let mut s = JevSlots::new(3);
        s.assign(0, "t", jev(&["a", "b"]), None);
        s.assign(
            10,
            "t",
            vec![Row::new("b", "Reworded b"), Row::new("a", "Reworded a")],
            Some("a"),
        );
        assert_eq!(s.get(0).unwrap().row.as_ref().unwrap().title, "Jev a");
        assert_eq!(s.get(1).unwrap().row.as_ref().unwrap().title, "Reworded b");
    }

    #[test]
    fn a_highlighted_row_further_down_stays_when_the_rows_above_it_go() {
        let mut l = ListState::new(true, true);
        l.on_text("t");
        l.jev.assign(0, "t", jev(&["a", "b", "c"]), None);
        l.sel = 2;
        let keep = l.focused().map(|r| r.key.clone());
        assert_eq!(keep.as_deref(), Some("c"));
        l.jev.assign(1_000, "t", jev(&["d"]), keep.as_deref());
        l.reselect(Some(Item::Slot(2)), keep.as_deref());
        assert_eq!(keys(&l.jev), [Some("d"), Some("c"), None]);
        assert_eq!(l.current_row().unwrap().key, "c");
        assert_eq!(
            l.effective(1, 1_010).unwrap().key,
            "b",
            "⌘2 still runs what was just there"
        );
    }

    #[test]
    fn a_late_answer_leaves_the_highlighted_stand_in_alone() {
        let mut l = ListState::new(true, true);
        l.on_text("quit gmgui");
        l.jev.down(0, "quit gmgui", None);
        let keep = l.focused().map(|r| r.key.clone());
        assert_eq!(keep.as_deref(), Some("down:model"));
        l.jev
            .assign(2_000, "quit gmgui", jev(&["jev:quit"]), keep.as_deref());
        l.reselect(Some(Item::Slot(0)), keep.as_deref());
        assert_eq!(keys(&l.jev), [Some("down:model"), Some("jev:quit"), None]);
        assert_eq!(l.current_row().unwrap().key, "down:model");
    }

    #[test]
    fn going_quiet_leaves_the_highlighted_reading_alone() {
        let mut s = JevSlots::new(3);
        s.assign(0, "t", jev(&["a", "b"]), None);
        s.down(10, "t", Some("b"));
        assert_eq!(
            keys(&s),
            [Some("down:model"), Some("b"), Some("down:retry")]
        );
    }

    #[test]
    fn a_name_searched_away_stays_while_highlighted() {
        let mut l = ListState::new(false, true);
        l.on_text("sl");
        l.names = names(&[("Slack", 90), ("Slides", 80)]);
        l.sel = 1;
        let row = l.focused().cloned().unwrap();
        l.names = names(&[("Slack", 90), ("Sleep", 85)]);
        assert!(l.keep(Item::Name(1), &row));
        assert_eq!(l.current_row().unwrap().key, "name:Slides");
    }

    #[test]
    fn enter_runs_the_row_drawn_under_the_highlight() {
        let mut l = ListState::new(true, false);
        l.on_text("x");
        l.jev.assign(0, "x", jev(&["a", "b"]), None);
        l.jev.assign(1_000, "x", jev(&["a", "c"]), None);
        l.sel = 1;
        assert_eq!(l.current_row().unwrap().key, "c");
        assert_eq!(l.effective(1, 1_100).unwrap().key, "b");
        assert_eq!(l.effective(1, 1_000 + ROW_ARM_MS).unwrap().key, "c");
    }

    #[test]
    fn filling_an_empty_place_keeps_the_target_id() {
        let mut l = ListState::new(true, true);
        l.on_text("x");
        let before = l.target(0).id;
        l.jev.assign(10, "x", jev(&["a"]), None);
        assert_eq!(l.target(10).id, before);
    }

    #[test]
    fn rows_that_stay_in_the_answer_keep_their_place() {
        let mut s = JevSlots::new(3);
        s.assign(0, "t", jev(&["a", "b", "c"]), None);
        s.assign(10, "t", jev(&["c", "d", "a"]), None);
        assert_eq!(keys(&s), [Some("a"), Some("d"), Some("c")]);
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
        l.jev.down(0, "book a table", None);
        assert!(l.jev.is_down_for("book a table"));
        assert!(!l.jev.read("book a table"));
        assert_eq!(l.current_row().unwrap().key, "down:model");
    }
}
