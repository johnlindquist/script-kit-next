//! What the main window lists in place of the launcher's rows while the reducer owns them: the
//! voice view's two readings, a staged subject's rows (share it how?), and the rows of the views
//! this module opens (automations, microphones, dictation history). Also the name rows that open
//! those views, the learned rows, and running any of them.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use gpui::Context;
use sk_one_window::{Event, Key, Row, ViewOp, ViewSpec};

use crate::list_item::GroupedListItem;
use crate::scripts::{OneWindowMatch, SearchResult};
use crate::ScriptListApp;

/// A view this module opens under the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OwnView {
    Automations,
    Microphones,
    DictationHistory,
    Script,
    Shortcut,
}

impl OwnView {
    pub(crate) fn name(self) -> &'static str {
        match self {
            OwnView::Automations => "automations",
            OwnView::Microphones => "microphones",
            OwnView::DictationHistory => "dictation-history",
            OwnView::Script => "script",
            OwnView::Shortcut => "shortcut",
        }
    }

    fn of(name: &str) -> Option<Self> {
        [
            OwnView::Automations,
            OwnView::Microphones,
            OwnView::DictationHistory,
            OwnView::Script,
            OwnView::Shortcut,
        ]
        .into_iter()
        .find(|view| view.name() == name)
    }

    /// Views whose rows the view picks from with the highlight and Enter.
    fn picks(self) -> bool {
        matches!(
            self,
            OwnView::Automations | OwnView::Microphones | OwnView::DictationHistory
        )
    }
}

/// What this module keeps between events.
#[derive(Default)]
pub(crate) struct LearnsHost {
    pub(crate) lessons_loaded: bool,
    /// The rows of the list view on top, with the header they lead with.
    pub(crate) view_rows: Vec<Row>,
    pub(crate) view_head: String,
    pub(crate) talk: super::talk_host::TalkHost,
    pub(crate) shortcut: Option<super::shortcut_view::Recording>,
    pub(crate) script: Option<super::saved_scripts::Shown>,
    pub(crate) plans: super::lessons_host::LearnedPlans,
    /// The final words of a voice session whose request Jev is reading.
    pub(crate) talk_request: Option<String>,
    /// saved.ts as this session knows it; written back only in production.
    pub(crate) saved_file: String,
    /// Automations stopped this session, by script path; kept on disk only in production.
    pub(crate) stopped: std::collections::BTreeSet<String>,
}

/// The icon names the list's resolver knows, for the spec's icon words.
pub(crate) fn list_icon(name: Option<&str>) -> &'static str {
    match name.unwrap_or_default() {
        "type" | "edit" => "pencil",
        "ask" | "message" | "reply" | "mic" => "message-circle",
        "schedule" | "again" | "automation" => "refresh",
        "copy" | "clipboard" => "copy",
        "open" | "link" | "share" => "share",
        "learned" | "star" => "star-filled",
        "script" | "code" => "code",
        "window" => "sidebar",
        "shortcut" | "keyboard" => "command",
        "off" | "hide" => "close",
        "check" => "check",
        _ => "bolt-filled",
    }
}

fn own_result(row: Row) -> SearchResult {
    let icon = list_icon(row.icon.as_deref());
    SearchResult::OneWindow(OneWindowMatch { row, icon })
}

/// Jev's likelihood for a reading, as the row's sub line says it.
fn with_p(mut row: Row) -> Row {
    let p = match row.p {
        Some(p) => format!("Jev {}%", (p * 100.0).round() as u32),
        None => "Jev —".to_string(),
    };
    row.sub = Some(match row.sub.take() {
        Some(sub) if !sub.is_empty() => format!("{sub} · {p}"),
        _ => p,
    });
    row
}

impl ScriptListApp {
    /// The view of this module on top of the reducer's views.
    pub(crate) fn one_window_own_view(&self) -> Option<OwnView> {
        self.one_window
            .views()
            .top()
            .and_then(|top| OwnView::of(&top.name))
    }

    /// Opens one of this module's views under the input.
    pub(crate) fn one_window_open_view(
        &mut self,
        view: OwnView,
        label: String,
        enter: Option<sk_one_window::EnterTarget>,
        takes_input: bool,
        cx: &mut Context<Self>,
    ) {
        self.one_window_handle(
            Event::View(ViewOp::Open(ViewSpec {
                name: view.name().to_string(),
                label,
                enter,
                takes_input,
                text: Some(String::new()),
                leave_text: Some(String::new()),
            })),
            cx,
        );
        self.one_window_republish_own(cx);
    }

    /// The rows the list shows now when the reducer, not the launcher, owns them.
    fn one_window_own_rows(&self) -> Option<(String, Vec<Row>)> {
        if let Some(talk) = self.one_window.talk() {
            return Some((talk.header(), talk.rows().into_iter().map(with_p).collect()));
        }
        if let Some(view) = self.one_window_own_view() {
            return match view {
                OwnView::Script => self.one_window_script_rows(),
                OwnView::Shortcut => self.one_window_shortcut_rows(),
                _ => Some((
                    self.one_window_host.learns.view_head.clone(),
                    self.one_window_own_view_rows(),
                )),
            };
        }
        if self.one_window.in_view() || !self.one_window.text().trim().is_empty() {
            return None;
        }
        let staged = self.one_window.staged()?;
        Some((staged.head.clone(), staged.pinned()))
    }

    /// The list view's rows, with a typed request's target first.
    fn one_window_own_view_rows(&self) -> Vec<Row> {
        let rows = self.one_window_host.learns.view_rows.clone();
        match self.one_window_own_view() {
            Some(OwnView::Automations) => {
                super::automations::said_first(rows, self.one_window.text())
            }
            _ => super::learns::filter_rows(rows, self.one_window.text()),
        }
    }

    /// Lays out the reducer's own rows under one header, for the launcher's list.
    pub(crate) fn one_window_own_list(&self) -> Option<(Vec<GroupedListItem>, Vec<SearchResult>)> {
        let (head, rows) = self.one_window_own_rows()?;
        let mut grouped = vec![GroupedListItem::SectionHeader(head, None)];
        let mut flat = Vec::with_capacity(rows.len());
        for mut row in rows {
            row.head = None;
            grouped.push(GroupedListItem::Item(flat.len()));
            flat.push(own_result(row));
        }
        Some((grouped, flat))
    }

    /// Changes whenever the reducer's own rows would draw differently.
    pub(crate) fn one_window_own_token(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        if let Some((head, rows)) = self.one_window_own_rows() {
            head.hash(&mut hasher);
            for row in rows {
                row.key.hash(&mut hasher);
                row.title.hash(&mut hasher);
                row.sub.hash(&mut hasher);
            }
        }
        self.one_window_own_view()
            .map(OwnView::name)
            .hash(&mut hasher);
        hasher.finish()
    }

    /// Republishes the list when the reducer's own rows changed without a text change.
    pub(crate) fn one_window_republish_own(&mut self, cx: &mut Context<Self>) {
        self.commit_main_menu_results_refresh("one_window_own_rows", None, cx, |_, _| true);
        self.one_window_follow_own_selection(cx);
        cx.notify();
    }

    /// After a publication, the highlight sits where the voice view's reading is, and the ↵
    /// button of a list view names the highlighted row. While talking the list stays at its top:
    /// the readings fill in place and never move.
    pub(crate) fn one_window_follow_own_selection(&mut self, cx: &mut Context<Self>) {
        let wanted = match (self.one_window.talk(), self.one_window_own_view()) {
            (Some(talk), _) => talk.selected(),
            _ => None,
        };
        if let Some(ordinal) = wanted {
            let grouped = self
                .main_menu_committed_rows()
                .iter()
                .find(|row| row.selectable_ordinal == Some(ordinal))
                .map(|row| row.grouped_index);
            if let Some(grouped) = grouped.filter(|ix| *ix != self.selected_index) {
                self.set_selected_index(grouped, "one_window_own", cx);
            }
        }
        if self.one_window.talk().is_some() {
            self.main_list_state.scroll_to(gpui::ListOffset {
                item_ix: 0,
                offset_in_item: gpui::px(0.),
            });
        }
        if self.one_window_own_view().is_some_and(OwnView::picks) {
            if let Some(row) = self.one_window_selected_own_row() {
                let target = sk_one_window::EnterTarget::new(
                    row.key.clone(),
                    row.enter_label.clone().unwrap_or_else(|| row.title.clone()),
                );
                if self.one_window.enter_target(self.one_window_clock()) != target {
                    self.one_window_handle(Event::ViewEnter(target), cx);
                }
            }
        }
    }

    /// The reducer's own row under the highlight.
    pub(crate) fn one_window_selected_own_row(&self) -> Option<Row> {
        match self.painted_main_menu_selected_subject()? {
            crate::ResolvedMainMenuSelection::SearchResult {
                result: SearchResult::OneWindow(m),
                ..
            } => Some(m.row.clone()),
            _ => None,
        }
    }

    /// Name rows this module adds for the typed words: learned sentences, and the rows that open
    /// its views or share what ran.
    pub(crate) fn one_window_more_names(
        &mut self,
        text: &str,
        rows: Vec<Row>,
        mut results: HashMap<String, SearchResult>,
        cx: &mut Context<Self>,
    ) -> (Vec<Row>, HashMap<String, SearchResult>) {
        self.one_window_load_lessons(cx);
        let mut more = self.one_window.learned_rows(text);
        let saved = self.one_window_saved_script_names();
        more.extend(saved_rows(text, &saved));
        let mut names = saved;
        names.extend(self.scripts.iter().map(|script| script.name.clone()));
        more.extend(super::learns::offered_rows(
            text,
            &names,
            self.one_window.ledger().last().is_some(),
        ));
        for row in &more {
            results.insert(row.key.clone(), own_result(row.clone()));
        }
        (sk_one_window::lessons::merge(rows, more), results)
    }

    /// Runs a row this module owns. `None` when the row is someone else's.
    pub(crate) fn one_window_run_learns_row(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> Option<anyhow::Result<()>> {
        let key = row.key.as_str();
        if key.starts_with("learned:") {
            return Some(self.one_window_run_learned(row, cx));
        }
        if key.starts_with("act:") {
            return self.one_window_run_share(row, cx);
        }
        let (kind, rest) = key.split_once(':')?;
        match kind {
            "open" => {
                match rest {
                    "automations" => self.one_window_open_automations(cx),
                    "microphones" => self.one_window_open_microphones(cx),
                    "dictation-history" => self.one_window_open_dictation_history(cx),
                    _ => return None,
                }
                Some(Ok(()))
            }
            "shortcut-for" => {
                self.one_window_open_shortcut_view(rest.to_string(), cx);
                Some(Ok(()))
            }
            "saved" => {
                self.one_window_open_script(rest.to_string(), cx);
                Some(Ok(()))
            }
            "talk" if rest == "start" => {
                self.one_window_talk_from_list(cx);
                Some(Ok(()))
            }
            _ => None,
        }
    }

    /// The list with what a view shows under its rows: the saved script's source.
    pub(crate) fn one_window_view_under_list(
        &self,
        list: gpui::AnyElement,
        colors: crate::list_item::ListItemColors,
    ) -> gpui::AnyElement {
        use gpui::{div, prelude::*, px};
        let below = match self.one_window_own_view() {
            Some(OwnView::Script) => self.one_window_script_element(colors),
            _ => None,
        };
        let Some(below) = below else {
            return list;
        };
        let def = self.current_main_menu_theme.def();
        let list_h = f32::from(crate::main_list_header_overlay_height(def))
            + crate::list_item::effective_first_section_header_height_for_theme(
                self.current_main_menu_theme,
            )
            + def.list.item_height;
        div()
            .relative()
            .w_full()
            .h_full()
            .child(list)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(list_h))
                    .bottom(px(
                        crate::footer_popup::main_window_footer_reservation_height(),
                    ))
                    .flex()
                    .flex_col()
                    .child(below),
            )
            .into_any_element()
    }

    /// How much taller than its rows a view wants the window: the saved script's source.
    pub(crate) fn one_window_view_extra_height(&self) -> f32 {
        match self.one_window_own_view() {
            Some(OwnView::Script) => self.one_window_script_height(),
            _ => 0.,
        }
    }

    /// Keys the reducer gave to the open view.
    pub(crate) fn one_window_view_key(&mut self, key: Key, cx: &mut Context<Self>) {
        match self.one_window_own_view() {
            Some(OwnView::Shortcut) => self.one_window_shortcut_key(key, cx),
            Some(view) if view.picks() && key == Key::Enter => {
                if let Some(row) = self.one_window_selected_own_row() {
                    match view {
                        OwnView::Automations => self.one_window_automation_enter(&row, cx),
                        OwnView::Microphones => self.one_window_microphone_enter(&row, cx),
                        OwnView::DictationHistory => self.one_window_history_enter(&row, cx),
                        OwnView::Script | OwnView::Shortcut => {}
                    }
                }
            }
            Some(OwnView::Script) if key == Key::Enter => self.one_window_script_enter(cx),
            _ => {}
        }
    }
}

/// Rows of a list view that match the typed words; all of them when nothing is typed.
pub(crate) fn filter_rows(rows: Vec<Row>, text: &str) -> Vec<Row> {
    let words: Vec<String> = text.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return rows;
    }
    rows.into_iter()
        .filter(|row| {
            let hay = format!(
                "{} {}",
                row.title.to_lowercase(),
                row.sub.as_deref().unwrap_or_default().to_lowercase()
            );
            words.iter().all(|w| hay.contains(w.as_str()))
        })
        .collect()
}

/// Saved runs whose names start with the typed words, word by word, as instant matches.
pub(crate) fn saved_rows(text: &str, saved: &[String]) -> Vec<Row> {
    let typed: Vec<String> = text.split_whitespace().map(str::to_lowercase).collect();
    if typed.is_empty() || text.trim().chars().count() < 3 {
        return Vec::new();
    }
    saved
        .iter()
        .filter(|name| {
            let words: Vec<String> = name
                .split(|c: char| !c.is_alphanumeric() && c != '#')
                .filter(|w| !w.is_empty())
                .map(str::to_lowercase)
                .collect();
            typed.len() <= words.len()
                && typed
                    .iter()
                    .zip(&words)
                    .all(|(typed, word)| word.starts_with(typed.as_str()))
        })
        .map(|name| {
            Row::new(format!("saved:{name}"), name.clone())
                .with_sub(format!(
                    "A run you saved · {}",
                    sk_one_window::contract::SAVED_SCRIPT_FILE_NAME
                ))
                .with_icon("script")
                .with_enter_label(format!(
                    "Open in {}",
                    sk_one_window::contract::SAVED_SCRIPT_FILE_NAME
                ))
                .with_score(95)
        })
        .collect()
}

/// Name rows for words that ask for one of this module's views, to share what ran, or to give a
/// saved script a shortcut.
pub(crate) fn offered_rows(text: &str, saved: &[String], has_last: bool) -> Vec<Row> {
    let t = text.trim().to_lowercase();
    if t.chars().count() < 3 {
        return Vec::new();
    }
    let mut rows = Vec::new();
    let offers = [
        (
            "talk:start",
            "Talk",
            "Say it: typed at your cursor, or done as a request",
            "mic",
            &["talk", "dictate", "dictation", "voice"][..],
        ),
        (
            "open:automations",
            "Automations",
            "What runs on its own · change or stop it by saying so",
            "automation",
            &[
                "automations",
                "list my automations",
                "scheduled",
                "schedules",
            ][..],
        ),
        (
            "open:microphones",
            "Microphone",
            "Pick the microphone dictation listens to",
            "mic",
            &["microphone", "mic", "input device"][..],
        ),
        (
            "open:dictation-history",
            "Dictation History",
            "What you said, newest first",
            "mic",
            &["dictation history", "what i said"][..],
        ),
    ];
    for (key, title, sub, icon, words) in offers {
        if words
            .iter()
            .any(|w| w.starts_with(&t) || t.starts_with(w) || t.contains(w))
        {
            let enter = if key == "talk:start" {
                "Start talking".to_string()
            } else {
                format!("Open {title}")
            };
            rows.push(
                Row::new(key, title)
                    .with_sub(sub)
                    .with_icon(icon)
                    .with_enter_label(enter)
                    .with_score(92),
            );
        }
    }
    if has_last && "share my last action".starts_with(&t) || t.starts_with("share my last") {
        rows.push(
            Row::new("act:share-last", "Share my last action")
                .with_sub("Copy it, copy a link, or send it")
                .with_icon("share")
                .with_act(sk_one_window::RowAct::Share)
                .with_score(92),
        );
    }
    if let Some(rest) = t.strip_prefix("give ") {
        let name = shortcut_subject(rest);
        if !name.is_empty() {
            if let Some(script) = saved
                .iter()
                .find(|s| s.to_lowercase().starts_with(name) || s.to_lowercase().contains(name))
            {
                rows.push(
                    Row::new(
                        format!("shortcut-for:{script}"),
                        format!("Give “{script}” a shortcut"),
                    )
                    .with_sub("Press the keys you want")
                    .with_icon("shortcut")
                    .with_enter_label("Record a shortcut")
                    .with_score(94),
                );
            }
        }
    }
    rows
}

/// The name in "give <name> a shortcut", also while " a shortcut" is half typed: the row must not
/// drop out at "give launch alpha a" and come back, or it can't own Enter again.
fn shortcut_subject(rest: &str) -> &str {
    const TAILS: [&str; 2] = [" a shortcut", " shortcut"];
    let cut = (1..=rest.len())
        .filter(|&n| rest.is_char_boundary(rest.len() - n))
        .filter(|&n| {
            TAILS
                .iter()
                .any(|tail| tail.starts_with(&rest[rest.len() - n..]))
        })
        .max()
        .unwrap_or(0);
    rest[..rest.len() - cut].trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_open_their_views() {
        let keys = |t: &str| {
            offered_rows(t, &[], true)
                .into_iter()
                .map(|r| r.key)
                .collect::<Vec<_>>()
        };
        assert_eq!(keys("list my automations"), ["open:automations"]);
        assert_eq!(keys("dictate"), ["talk:start"]);
        assert_eq!(keys("autom"), ["open:automations"]);
        assert_eq!(keys("share my last action"), ["act:share-last"]);
        assert!(keys("sl").is_empty());
    }

    #[test]
    fn a_saved_run_matches_its_name_word_by_word() {
        let saved = ["Tell Sarah + Post in #launcher".to_string()];
        assert_eq!(saved_rows("tell sa", &saved).len(), 1);
        assert_eq!(saved_rows("tell sarah post", &saved).len(), 1);
        assert!(saved_rows("sarah", &saved).is_empty());
        assert!(saved_rows("te", &saved).is_empty());
    }

    #[test]
    fn giving_a_saved_script_a_shortcut_names_it() {
        let rows = offered_rows(
            "give tell sarah a shortcut",
            &["Tell Sarah + Post in #launcher".to_string()],
            false,
        );
        assert_eq!(rows[0].key, "shortcut-for:Tell Sarah + Post in #launcher");
    }

    #[test]
    fn the_shortcut_row_stays_offered_while_the_sentence_is_typed() {
        let saved = ["Launch Alpha".to_string(), "Launch Beta".to_string()];
        let sentence = "give launch alpha a shortcut";
        for end in "give l".len()..=sentence.len() {
            let typed = &sentence[..end];
            let keys: Vec<String> = offered_rows(typed, &saved, false)
                .into_iter()
                .map(|r| r.key)
                .collect();
            assert_eq!(keys, ["shortcut-for:Launch Alpha"], "typed {typed:?}");
        }
        assert_eq!(
            offered_rows("give launch beta a", &saved, false)[0].key,
            "shortcut-for:Launch Beta"
        );
    }

    #[test]
    fn readings_show_jevs_percentage() {
        let mut row = Row::new("talk:type", "Type it into Mail").with_sub("at your cursor");
        row.p = Some(0.853);
        assert_eq!(with_p(row).sub.as_deref(), Some("at your cursor · Jev 85%"));
        assert_eq!(
            with_p(Row::new("talk:ask", "Do it as a request"))
                .sub
                .as_deref(),
            Some("Jev —")
        );
    }
}
