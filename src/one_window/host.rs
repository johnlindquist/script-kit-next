//! Feeds the main window's words, name matches, Jev's answers and a monotonic clock to
//! `sk_one_window::OneWindow`, and applies every effect it returns.
//!
//! The reducer owns the rules: which header leads the list, where Jev's rows go, what Enter runs
//! and when it is ignored, and how the window's height moves. Nothing here decides any of that;
//! it translates between the reducer's rows and the launcher's search results, runs what the
//! reducer says to run, and moves the window where the reducer says.

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use gpui::Context;
use sk_one_window::contract::{words, MAX_MATCHES};
use sk_one_window::{Effect, Event, Item, Key, Row};

use crate::intent::{JevError, JevRow, Plan};
use crate::list_item::GroupedListItem;
use crate::scripts::{IntentMatch, SearchResult};
use crate::{AppView, MainMenuDispatchResult, MainMenuSelectionOrigin, ScriptListApp};

/// Jev counts as down for the words it was asked about once this passes without an answer.
const JEV_TIMEOUT: Duration = Duration::from_secs(5);

/// Milliseconds on the host's clock.
type Ms = sk_one_window::Ms;

#[derive(Default)]
pub(crate) struct OneWindowHost {
    epoch: Option<Instant>,
    now: Ms,
    /// The one pending `Tick`; a timer for any other time is stale.
    wake_at: Option<Ms>,
    published_subject_page: Option<String>,
    pub(crate) tab_hints: bool,
    pub(crate) tab_modifier_generation: u64,
    /// The fixed height the main list's window is at or was last asked to take; cleared while a
    /// prompt sizes the window its own way.
    pub(super) height: Option<f32>,
    pub(super) window: Option<gpui::AnyWindowHandle>,
    /// Words the reducer put in the input; their echo from the input isn't typing.
    echo: Option<String>,
    /// Invalidates a pending, read-only Save-sheet preview when the user moves on.
    pub(super) fill_preview: u64,
    pub(super) handoff_listening: bool,
    pub(super) handoff_draft: Option<super::handoff::OpenDraft>,
    jev_asked: u64,
    jev_pending: Option<(u64, String)>,
    /// A row that ran and opened something; its title is what finished when that ends.
    finishing: Option<Row>,
    last_run: Option<Row>,
    /// What was highlighted when ⌘K staged something.
    pub(super) staged_from: Option<super::staging::StagedFrom>,
    pub(crate) work: super::work::WorkHost,
    pub(crate) learns: super::learns::LearnsHost,
    pub(crate) cursor_voice: crate::CursorHost,
    pub(super) ledger: super::ledger_view::LedgerHost,
    /// The form, answer or note open in place of the list.
    pub(super) view: Option<super::views::HostedView>,
    /// The input's words the open view last took.
    pub(super) view_text: Option<String>,
    /// What was staged when this turn began: the subject a staged row that ran acts on.
    pub(super) acting_on: Option<sk_one_window::Subject>,
    /// Whether a view covered the list after the last turn. The input's ghost words belong to
    /// the list, so they go when a view opens and come back when it closes.
    pub(super) covered: bool,
    /// A note asked for while the window was hidden, opened once it's shown.
    pub(crate) note_open_revision: u64,
    pub(crate) note_waiting: Option<crate::notes::NoteTarget>,
    /// Scrolls a long view's rows so the highlighted one stays in sight.
    pub(super) view_scroll: gpui::ScrollHandle,
    /// The row it last scrolled to, so the wheel can scroll freely until the highlight moves.
    pub(super) view_scrolled_to: std::cell::Cell<Option<usize>>,
    /// What a view staged with ⌘K, while its actions are listed in its place.
    pub(super) view_stage: Option<super::view_staging::ViewStage>,
    /// What each name the list last drew stands for, so a name the reducer keeps under the
    /// highlight still draws and runs when a search for the same words leaves it out.
    names_drawn: HashMap<String, SearchResult>,
}

/// What the reducer did with one event.
#[derive(Default)]
pub(crate) struct Handled {
    pub(crate) consumed: bool,
    /// The key goes to the list's old default (Esc hides the window).
    pub(crate) passed: bool,
    pub(crate) ignored: bool,
    pub(crate) ran: bool,
    pub(crate) run_error: Option<anyhow::Error>,
}

impl Handled {
    pub(crate) fn taken(&self) -> bool {
        self.consumed && !self.passed
    }
}

/// How strongly a search result's match owns Enter, 0–100, by the tier its score encodes.
/// A compact fuzzy near-miss or a match only in a description, file name, path or body is
/// listed but lands under [`sk_one_window::contract::WEAK_SCORE`].
fn name_strength(result: &SearchResult) -> u8 {
    use crate::scripts::search::{
        TIER_ACRONYM_PRIMARY, TIER_ALIAS, TIER_COMPACT_FUZZY_PRIMARY, TIER_DESCRIPTION,
        TIER_EXACT_PRIMARY, TIER_FILENAME, TIER_KEYWORD, TIER_PATH, TIER_PREFIX_PRIMARY,
        TIER_SLIP_PRIMARY, TIER_SUBSTRING_PRIMARY, TIER_WORD_BOUNDARY_PRIMARY,
    };
    match result.match_tier() {
        TIER_EXACT_PRIMARY.. => 100,
        TIER_PREFIX_PRIMARY.. => 95,
        TIER_WORD_BOUNDARY_PRIMARY.. => 90,
        TIER_SUBSTRING_PRIMARY.. => 80,
        TIER_ACRONYM_PRIMARY.. => 75,
        TIER_COMPACT_FUZZY_PRIMARY.. => 40,
        TIER_SLIP_PRIMARY.. => 38,
        TIER_ALIAS.. => 70,
        TIER_KEYWORD.. => 60,
        TIER_DESCRIPTION.. => 35,
        TIER_FILENAME.. => 30,
        TIER_PATH.. => 20,
        1.. => 10,
        _ => 0,
    }
}

/// A fallback the window offers for the words: the key prefix the reducer knows it by and where
/// it ranks. What the words' shape asks for comes first (a URL, a sum, a path), then the user's own
/// fallback scripts, the web, the files and the model. The rest stay in the full results.
fn offered_fallback(result: &SearchResult) -> Option<(String, u8)> {
    let SearchResult::Fallback(fallback) = result else {
        return None;
    };
    let item = match &fallback.fallback {
        crate::fallbacks::FallbackItem::Builtin(item) => item,
        crate::fallbacks::FallbackItem::Script(config) => {
            return Some((
                format!("fallback-script/{}", config.script.source_command_id()),
                3,
            ));
        }
    };
    let (prefix, rank) = match item.id {
        "open-url" => ("fallback-open-url", 0),
        "calculate" => ("fallback-calculate", 1),
        "open-file" => ("fallback-open-file", 2),
        "search-google" => ("fallback-google", 4),
        crate::fallbacks::builtins::SEARCH_FILES_FALLBACK_ID => ("fallback-search-files", 5),
        "search-google-ai" => ("fallback-ai", 6),
        _ => return None,
    };
    Some((prefix.to_string(), rank))
}

/// Where a name comes from, for spreading the few name rows across sources: commands (scripts,
/// apps, built-ins) are one source, and each passive source (files, tabs, conversations, notes,
/// clipboard…) is its own.
fn name_source(result: &SearchResult) -> &'static str {
    match result {
        SearchResult::Script(_)
        | SearchResult::Scriptlet(_)
        | SearchResult::Flow(_)
        | SearchResult::Skill(_)
        | SearchResult::BuiltIn(_)
        | SearchResult::App(_)
        | SearchResult::Agent(_) => "command",
        other => other.type_label(),
    }
}

/// Which names, by their place in the search's order, fill `budget` rows: the best one, then the
/// best of each source not shown yet, then the next best. One busy source (a folder of files)
/// never hides the conversation or tab that also matched. Returned in the search's order.
pub(crate) fn pick_across_sources(sources: &[&str], budget: usize) -> Vec<usize> {
    let mut picked: Vec<usize> = Vec::with_capacity(budget);
    let mut seen: Vec<&str> = Vec::new();
    for (at, source) in sources.iter().enumerate() {
        if picked.len() >= budget {
            break;
        }
        if !seen.contains(source) {
            seen.push(source);
            picked.push(at);
        }
    }
    for at in 0..sources.len() {
        if picked.len() >= budget {
            break;
        }
        if !picked.contains(&at) {
            picked.push(at);
        }
    }
    picked.sort_unstable();
    picked
}

fn jev_rows(readings: &[JevRow]) -> Vec<Row> {
    readings
        .iter()
        .map(|reading| {
            let row = Row::new(reading.key.clone(), reading.plan.title.clone());
            if crate::intent::launcher::unavailable(&reading.plan).is_some() {
                row.with_enter_label("Unavailable")
            } else {
                row
            }
        })
        .collect()
}

impl ScriptListApp {
    pub(super) fn one_window_now(&mut self, cx: &Context<Self>) -> Ms {
        let now = cx.background_executor().now();
        let epoch = *self.one_window_host.epoch.get_or_insert(now);
        let ms =
            u64::try_from(now.saturating_duration_since(epoch).as_millis()).unwrap_or(u64::MAX);
        self.one_window_host.now = self.one_window_host.now.max(ms);
        self.one_window_host.now
    }

    /// The reducer's time as of the last event the host fed it; what rendering reads.
    pub(crate) fn one_window_clock(&self) -> Ms {
        self.one_window_host.now
    }

    /// The reducer's clock read now, without feeding it an event.
    fn one_window_peek_now(&self, cx: &Context<Self>) -> Ms {
        self.one_window_host
            .epoch
            .map_or(self.one_window_host.now, |epoch| {
                let since = cx
                    .background_executor()
                    .now()
                    .saturating_duration_since(epoch);
                u64::try_from(since.as_millis())
                    .unwrap_or(u64::MAX)
                    .max(self.one_window_host.now)
            })
    }

    /// Whether the reducer owns the list for these words. Launcher syntax, the calculator,
    /// attachment portals and pickers keep their own lists.
    pub(crate) fn one_window_owns(&self, text: &str) -> bool {
        if !matches!(self.current_view, AppView::ScriptList) {
            return false;
        }
        if self.one_window.in_view() {
            return true;
        }
        let picker_owns = self.menu_syntax_object_selector_state.owns_main_list()
            || self.menu_syntax_trigger_picker_state.owns_main_list();
        if self.menu_syntax_mode.capture_composer_owns_input_for(text) {
            return !picker_owns;
        }
        self.inline_calculator.is_none()
            && self.active_script_list_attachment_portal_kind().is_none()
            && !picker_owns
            && !self.spine_projection_owns_main_list()
            && !crate::menu_syntax::active_filter_head_owns_main_list(text)
            && self.menu_syntax_mode.command_for(text).is_none()
            && self.menu_syntax_mode.incomplete_hint_for(text).is_none()
            && !self.menu_syntax_mode.command_owns_input_for(text)
            && !self
                .menu_syntax_mode
                .advanced_query_for(text)
                .is_some_and(|query| query.has_source_filters() || query.has_predicates())
    }

    pub(crate) fn one_window_owns_list(&self) -> bool {
        self.one_window_owns(&self.filter_text)
    }

    /// Feeds one event and applies everything the reducer says, including events that
    /// effects answer at once (a recalled Jev answer).
    pub(crate) fn one_window_handle(&mut self, event: Event, cx: &mut Context<Self>) -> Handled {
        self.one_window_handle_turn(vec![event], cx)
    }

    /// Feeds events that happen in one turn: the reducer sees them at the same moment.
    pub(super) fn one_window_handle_turn(
        &mut self,
        events: Vec<Event>,
        cx: &mut Context<Self>,
    ) -> Handled {
        let mut queue = VecDeque::from(events);
        let mut handled = Handled::default();
        let mut runs = Vec::new();
        let action_page = self.one_window.page_for_action();
        let mut works = Vec::new();
        let mut first = true;
        self.one_window_load_lessons(cx);
        let now = self.one_window_now(cx);
        self.one_window_host.acting_on = self.one_window.staged().map(|s| s.subject.clone());
        while let Some(event) = queue.pop_front() {
            if matches!(
                event,
                Event::Text(_)
                    | Event::Dismissed
                    | Event::Summon
                    | Event::View(_)
                    | Event::Stage(_)
                    | Event::Key(_)
            ) {
                self.one_window_host.fill_preview =
                    self.one_window_host.fill_preview.wrapping_add(1);
            }
            let typed = matches!(event, Event::Text(_));
            let outcome = self.one_window.handle(now, event);
            if std::mem::take(&mut first) {
                handled.consumed = outcome.consumed;
            }
            for effect in outcome.effects {
                match effect {
                    Effect::Run { row, .. } => runs.push(row),
                    Effect::PassKey(_) => handled.passed = true,
                    Effect::EnterIgnored => {
                        handled.ignored = true;
                        tracing::info!(target: "script_kit::one_window", event = "enter_ignored");
                    }
                    Effect::AskJev { text } => self.one_window_ask_jev(text, &mut queue, cx),
                    Effect::WakeAt(at) => self.one_window_wake_at(at, cx),
                    Effect::SetInput(text) => self.one_window_set_input(text, cx),
                    Effect::Hide { .. } => self.close_and_reset_window(cx),
                    // Summon comes from the show path, which is already showing the window.
                    Effect::Show => {}
                    Effect::ActionsFor(subject) => {
                        queue.push_back(self.one_window_actions_for(&subject));
                    }
                    Effect::ViewKey(key) => match self.one_window.views().top() {
                        Some(top) if super::work::WORK_VIEWS.contains(&top.name.as_str()) => {
                            queue.push_back(Event::Work(sk_one_window::work::WorkEvent::ViewKey(
                                key,
                            )));
                        }
                        Some(top)
                            if [
                                super::views::FORM,
                                super::views::ANSWER,
                                super::notes::NOTES,
                                super::reading::READING,
                            ]
                            .contains(&top.name.as_str()) =>
                        {
                            queue.extend(self.one_window_hosted_view_key(key, cx));
                        }
                        _ => self.one_window_ledger_effect(Effect::ViewKey(key), cx),
                    },
                    Effect::Work(work) => works.push(work),
                    Effect::OpenScript { name } => self.one_window_open_script(name, cx),
                    Effect::SaveScript { entry, name, said } => {
                        if !self.one_window_save_script(name.clone(), said) {
                            queue.push_back(Event::ScriptNotSaved(entry));
                            queue.push_back(Event::Notice(sk_one_window::notice::Notice::new(
                                crate::line_notice::FAILED,
                                format!("Couldn't save “{name}” as a script."),
                            )));
                        }
                    }
                    Effect::TalkAsk { words, last } => self.one_window_talk_ask(words, last, cx),
                    Effect::Paste { text, app } => self.one_window_paste(text, app, cx),
                    Effect::TalkEnded => self.one_window_talk_ended(),
                    Effect::Learned => self.one_window_learned(),
                    Effect::AddToNote { text } => self.one_window_add_to_note(text, cx),
                    effect @ (Effect::Held { .. } | Effect::Undo { .. } | Effect::Again { .. }) => {
                        self.one_window_ledger_effect(effect, cx)
                    }
                }
            }
            if typed {
                queue.extend(self.one_window_view_typed());
                self.one_window_say_notes_trouble(cx);
            }
        }
        for row in runs {
            handled.ran = true;
            let said = self.one_window_host.ledger.said;
            let title = row.title.clone();
            if let Err(error) = if row.key.starts_with("subject/") {
                self.run_subject_action(&row, action_page.clone(), cx)
            } else {
                self.one_window_run(row, cx)
            } {
                tracing::warn!(
                    target: "script_kit::one_window",
                    event = "run_failed",
                    error = %error,
                );
                if self.one_window_host.ledger.said == said {
                    let words = run_failure_words(&title, &error);
                    self.one_window_notice(crate::line_notice::FAILED, words, cx);
                }
                handled.run_error = Some(error);
            }
        }
        for work in works {
            self.one_window_apply_work(work, cx);
        }
        self.one_window_let_go(cx);
        self.one_window_views_follow(cx);
        self.one_window_after_turn();
        self.one_window_remember_runs(now, cx);
        cx.notify();
        handled
    }

    /// A key the list got. The reducer sees the row the user is on first, and the highlight
    /// follows wherever the key moved it.
    pub(crate) fn one_window_key(&mut self, key: Key, cx: &mut Context<Self>) -> Handled {
        self.one_window_follow_host_selection(cx);
        let before = self.one_window_jev_token();
        let handled = self.one_window_handle(Event::Key(key), cx);
        if self.one_window_jev_token() != before {
            self.one_window_republish("one_window_key", cx);
        }
        self.one_window_show_selection(cx);
        handled
    }

    /// Enter on the list: what the reducer runs, or nothing when it waits or ignores the key.
    pub(crate) fn one_window_enter(
        &mut self,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<MainMenuDispatchResult> {
        let handled = self.one_window_key(Key::Enter, cx);
        if let Some(error) = handled.run_error {
            return Err(error);
        }
        if handled.ran {
            return Ok(MainMenuDispatchResult::Dispatched);
        }
        self.update_main_menu_dispatch_status(
            "pendingConfirmation",
            Some(if handled.ignored {
                "enter_guard"
            } else {
                "waiting_for_jev"
            }),
        );
        Ok(MainMenuDispatchResult::PendingConfirmation)
    }

    /// The list's place of the highlighted row, among the rows it can land on.
    fn one_window_host_position(&self) -> Option<usize> {
        self.main_menu_committed_rows()
            .iter()
            .find(|row| row.grouped_index == self.selected_index)
            .and_then(|row| row.selectable_ordinal)
    }

    /// The highlight moved by pointer, agent or a publication: the reducer's follows.
    pub(crate) fn one_window_follow_host_selection(&mut self, cx: &mut Context<Self>) {
        if self.one_window_shows_every_result() {
            self.one_window_name_every_results_enter(cx);
            return;
        }
        if !self.one_window_owns_list() || self.one_window_view_covers_list() {
            return;
        }
        let Some(position) = self.one_window_host_position() else {
            return;
        };
        if position != self.one_window.list().sel {
            self.one_window_handle(Event::Select(position), cx);
        }
    }

    /// In every result's view, ↵ names what the highlighted result does, as it does in the
    /// launcher.
    fn one_window_name_every_results_enter(&mut self, cx: &mut Context<Self>) {
        let Some(crate::ResolvedMainMenuSelection::SearchResult { result, .. }) =
            self.painted_main_menu_selected_subject()
        else {
            return;
        };
        let label = crate::ui_window::main_window_result_action_label(
            result,
            crate::ui_window::footer_frontmost_app_name().as_deref(),
        );
        let target =
            sk_one_window::EnterTarget::new(super::learns::OwnView::AllResults.name(), label);
        if self.one_window.enter_target(self.one_window_clock()) != target {
            self.one_window_handle(Event::ViewEnter(target), cx);
        }
    }

    pub(super) fn one_window_show_selection(&mut self, cx: &mut Context<Self>) {
        if self.one_window_view_covers_list() || self.one_window_shows_every_result() {
            return;
        }
        let position = self.one_window.list().sel;
        let Some(grouped_index) = self
            .main_menu_committed_rows()
            .iter()
            .find(|row| row.selectable_ordinal == Some(position))
            .map(|row| row.grouped_index)
        else {
            return;
        };
        if grouped_index != self.selected_index {
            self.set_selected_index(grouped_index, "one_window", cx);
        }
    }

    /// Every result for the words is listed as it came from the search, not the reducer's few
    /// rows, so neither highlight follows the other.
    pub(crate) fn one_window_shows_every_result(&self) -> bool {
        self.one_window_own_view() == Some(super::learns::OwnView::AllResults)
    }

    /// The committed words changed. With Jev off there is nothing to wait for: its rows settle
    /// in the same turn as the key.
    pub(crate) fn one_window_text(&mut self, text: &str, cx: &mut Context<Self>) {
        if self.one_window_host.echo.take().as_deref() == Some(text)
            || !self.one_window_owns(text)
            || self.one_window.text() == text
        {
            return;
        }
        self.settle_intent_oracle_slot();
        let mut turn = vec![Event::Text(text.to_string())];
        if self.intent_launcher.is_off()
            && !text.trim().is_empty()
            && self
                .intent_tab_browser()
                .and_then(|app| crate::intent::engine::named_tab_plan(text, app))
                .is_none()
        {
            turn.push(Event::JevDown {
                text: text.to_string(),
            });
        }
        self.one_window_handle_turn(turn, cx);
    }

    pub(super) fn one_window_set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if text == self.filter_text || !matches!(self.current_view, AppView::ScriptList) {
            return;
        }
        self.one_window_host.echo = Some(text.clone());
        self.filter_text = text.clone();
        self.pending_filter_sync = true;
        if matches!(
            self.one_window_hosted_view(),
            Some(super::views::HostedView::Form(_))
        ) {
            self.filter_coalescer.reset();
        } else {
            self.queue_filter_compute(text, cx);
        }
    }

    fn one_window_wake_at(&mut self, at: Ms, cx: &mut Context<Self>) {
        self.one_window_host.wake_at = Some(at);
        let delay = Duration::from_millis(at.saturating_sub(self.one_window_host.now));
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let _ = this.update(cx, |app, cx| {
                if app.one_window_host.wake_at != Some(at) {
                    return;
                }
                app.one_window_host.wake_at = None;
                let before = app.one_window_jev_token();
                app.one_window_handle(Event::Tick, cx);
                if app.one_window_jev_token() != before {
                    app.one_window_republish("one_window_tick", cx);
                }
            });
        })
        .detach();
    }

    pub(super) fn one_window_republish(&mut self, reason: &'static str, cx: &mut Context<Self>) {
        if self.one_window_owns(&self.computed_filter_text) {
            self.commit_main_menu_results_refresh(reason, None, cx, |_, _| true);
        }
    }

    /// Changes whenever Jev's rows would draw differently, so the list's cache rebuilds.
    pub(crate) fn one_window_jev_token(&self) -> u64 {
        let list = self.one_window.list();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        list.jev.settled_for(&list.text).hash(&mut hasher);
        for slot in (0..list.jev.len()).filter_map(|i| list.jev.get(i)) {
            slot.row.as_ref().map(|row| &row.key).hash(&mut hasher);
            slot.stale.hash(&mut hasher);
        }
        for row in self.one_window.staged().iter().flat_map(|s| &s.rows) {
            row.key.hash(&mut hasher);
        }
        self.one_window
            .subject_page()
            .map(|p| p.id)
            .hash(&mut hasher);
        self.one_window_own_token().hash(&mut hasher);
        self.one_window_ledger_token(&mut hasher);
        hasher.finish()
    }

    // ------------------------------------------------------------------ the list

    /// Every list the reducer owns is its layout, headers and all: for typed words the name
    /// matches the search found and Jev's three rows; for an empty input what the launcher
    /// suggests, or what's staged. Lists the reducer doesn't own come back unchanged.
    pub(crate) fn one_window_list(
        &mut self,
        grouped: Vec<GroupedListItem>,
        flat: Vec<SearchResult>,
        text: &str,
        cx: &mut Context<Self>,
    ) -> (Vec<GroupedListItem>, Vec<SearchResult>, bool) {
        if !self.one_window_owns(text) {
            return (grouped, flat, false);
        }
        if self.one_window_shows_every_result() {
            return (grouped, flat, false);
        }
        if let Some((grouped, flat)) = self
            .one_window_own_list()
            .or_else(|| self.one_window_ledger_list(text))
        {
            return (grouped, flat, true);
        }
        if self.one_window.text() != text && !self.one_window.in_view() {
            self.one_window_handle(Event::Text(text.to_string()), cx);
        }
        let results = if text.trim().is_empty() {
            let (rows, results) = self.one_window_suggestions(&grouped, &flat);
            self.one_window_handle(Event::Suggested(rows), cx);
            results
        } else {
            let (rows, results) = self.one_window_names(&grouped, &flat, text);
            let (rows, mut results) = self.one_window_more_names(text, rows, results, cx);
            self.one_window_handle(Event::Names(rows), cx);
            for row in &self.one_window.list().names {
                if results.contains_key(&row.key) {
                    continue;
                }
                if let Some(result) = self.one_window_host.names_drawn.get(&row.key) {
                    results.insert(row.key.clone(), result.clone());
                }
            }
            self.one_window_host.names_drawn = results.clone();
            results
        };
        let (mut laid_out, flat) = self.one_window_layout(&results, text);
        laid_out.extend(
            grouped
                .into_iter()
                .filter(|item| matches!(item, GroupedListItem::Status(_))),
        );
        (laid_out, flat, true)
    }

    /// What the launcher lists for an empty input, in order, each section's first row carrying
    /// its header.
    fn one_window_suggestions(
        &self,
        grouped: &[GroupedListItem],
        flat: &[SearchResult],
    ) -> (Vec<Row>, HashMap<String, SearchResult>) {
        let frontmost = crate::ui_window::footer_frontmost_app_name();
        let mut head = None;
        let mut rows = self.one_window_setup_rows();
        let mut results: HashMap<String, SearchResult> = rows
            .iter()
            .map(|row| (row.key.clone(), super::ledger_view::own_row_result(row)))
            .collect();
        for item in grouped {
            let index = match item {
                GroupedListItem::SectionHeader(label, _) => {
                    head = Some(label.clone());
                    continue;
                }
                GroupedListItem::Item(index) => *index,
                GroupedListItem::Status(_) => continue,
            };
            let Some(result) = flat.get(index) else {
                continue;
            };
            let key = result
                .stable_selection_key()
                .unwrap_or_else(|| format!("name/{}", result.name()));
            if results.contains_key(&key) {
                continue;
            }
            let mut row = Row::new(key.clone(), result.name()).with_enter_label(
                crate::ui_window::main_window_result_action_label(result, frontmost.as_deref()),
            );
            row.head = head.take();
            rows.push(row);
            results.insert(key, result.clone());
        }
        (rows, results)
    }

    /// The name matches spread across their sources, a row that opens every result when more
    /// matched than fit, then the fallbacks the window offers for the words, best first.
    fn one_window_names(
        &self,
        grouped: &[GroupedListItem],
        flat: &[SearchResult],
        text: &str,
    ) -> (Vec<Row>, HashMap<String, SearchResult>) {
        let frontmost = crate::ui_window::footer_frontmost_app_name();
        let learned = self.input_history.preferred_result_key(text);
        let mut rows = Vec::new();
        let mut names: Vec<(Row, &SearchResult)> = Vec::new();
        let mut fallbacks: Vec<(u8, Row, &SearchResult)> = Vec::new();
        let mut results = HashMap::new();
        if let Some((row, result)) = self.one_window_capture_row(text) {
            results.insert(row.key.clone(), result);
            rows.push(row);
        }
        let mut keys: std::collections::HashSet<String> = results.keys().cloned().collect();
        for item in grouped {
            let GroupedListItem::Item(index) = item else {
                continue;
            };
            let Some(result) = flat.get(*index) else {
                continue;
            };
            let offered = offered_fallback(result);
            if offered.is_none() && matches!(result, SearchResult::Fallback(_)) {
                continue;
            }
            let key = match &offered {
                Some((prefix, _)) => format!("{prefix}:{text}"),
                None => result
                    .stable_selection_key()
                    .unwrap_or_else(|| format!("name/{}", result.name())),
            };
            if !keys.insert(key.clone()) {
                continue;
            }
            let row = Row::new(key, result.name())
                .with_enter_label(crate::ui_window::main_window_result_action_label(
                    result,
                    frontmost.as_deref(),
                ))
                .with_score(name_strength(result));
            match offered {
                Some((_, rank)) => fallbacks.push((rank, row.fallback(), result)),
                None => names.push((
                    Row {
                        learned: learned.is_some()
                            && learned == result.history_result_key().as_deref(),
                        ..row
                    },
                    result,
                )),
            }
        }
        let budget = MAX_MATCHES.saturating_sub(rows.len());
        let sources: Vec<&str> = names
            .iter()
            .map(|(_, result)| name_source(result))
            .collect();
        // The row an alias names comes first (A1), then the row learned for these words, then
        // any name typed exactly, whichever source it comes from: a note called "Budget" goes
        // above a command that merely contains the word. Each of those is listed even when
        // spreading rows across sources would leave it to "See All".
        let typed = text.trim().to_lowercase();
        let alias = (!typed.is_empty())
            .then(|| self.find_alias_match(text.trim()))
            .flatten();
        let rank = |at: usize| {
            let (row, result) = &names[at];
            let aliased = alias.as_ref().is_some_and(|alias| alias.is_target(result));
            let exact = result.name().trim().to_lowercase() == typed;
            (!aliased, !row.learned, !exact)
        };
        let plain = (true, true, true);
        let mut picked = pick_across_sources(&sources, budget);
        let mut must: Vec<usize> = (0..names.len()).filter(|&at| rank(at) != plain).collect();
        must.sort_by_key(|&at| (rank(at), at));
        for at in must {
            if picked.contains(&at) {
                continue;
            }
            let Some(evict) = picked.iter().rposition(|&kept| rank(kept) == plain) else {
                break;
            };
            picked.remove(evict);
            picked.push(at);
        }
        picked.sort_by_key(|&at| (rank(at), at));
        // The reducer makes the final cut after learned rows arrive. Give it the
        // best open tab even when the source spread filled every slot without one.
        let extra_tab = (!picked.iter().any(|&at| names[at].0.is_open_tab()))
            .then(|| names.iter().position(|(row, _)| row.is_open_tab()))
            .flatten();
        for at in picked.iter().copied().chain(extra_tab) {
            let (row, result) = &names[at];
            results.insert(row.key.clone(), (*result).clone());
            rows.push(row.clone());
        }
        if names.len() > picked.len() {
            rows.push(
                Row::new(
                    format!("{}{text}", sk_one_window::contract::ALL_RESULTS_KEY),
                    sk_one_window::contract::words::see_all(names.len()),
                )
                .with_icon("list")
                .with_score(0)
                .with_enter_label("See All"),
            );
        }
        fallbacks.sort_by_key(|(rank, _, _)| *rank);
        for (_, row, result) in fallbacks {
            results.insert(row.key.clone(), result.clone());
            rows.push(row);
        }
        (rows, results)
    }

    fn one_window_layout(
        &self,
        results: &HashMap<String, SearchResult>,
        text: &str,
    ) -> (Vec<GroupedListItem>, Vec<SearchResult>) {
        let list = self.one_window.list();
        let mut grouped = Vec::new();
        let mut flat = Vec::new();
        for item in list.layout() {
            let result = match &item {
                Item::Header(head) => {
                    grouped.push(GroupedListItem::SectionHeader(
                        list.drawn_header(head),
                        None,
                    ));
                    continue;
                }
                Item::Slot(slot) => SearchResult::Intent(self.one_window_jev_match(*slot, text)),
                item => match list.row(item).map(|row| {
                    results
                        .get(&row.key)
                        .cloned()
                        .or_else(|| Self::one_window_action_result(row))
                        .unwrap_or_else(|| super::ledger_view::own_row_result(row))
                }) {
                    Some(result) => result,
                    None => continue,
                },
            };
            grouped.push(GroupedListItem::Item(flat.len()));
            flat.push(result);
        }
        (grouped, flat)
    }

    fn one_window_jev_match(&self, slot: usize, text: &str) -> IntentMatch {
        let list = self.one_window.list();
        let settled = list.jev.settled_for(text);
        let shown = list
            .jev
            .get(slot)
            .and_then(|s| s.row.clone().map(|row| (row, s.stale)));
        let blank = IntentMatch {
            slot,
            request: text.to_string(),
            title: String::new(),
            subtitle: String::new(),
            verb: "Run",
            reading: !settled,
            stale: false,
            plan: None,
            score: 0,
        };
        match shown {
            Some((row, stale)) => {
                let plan = self.intent_launcher.plan(text, &row.key);
                IntentMatch {
                    subtitle: super::work::needs_subtitle(&row, self.one_window.work())
                        .or_else(|| plan.as_deref().map(jev_row_subtitle))
                        .or_else(|| row.sub.clone())
                        .unwrap_or_default(),
                    title: row.title,
                    verb: plan.as_deref().map_or("Run", crate::intent::plan_verb),
                    reading: false,
                    stale,
                    plan,
                    ..blank
                }
            }
            None if slot == 0 && !settled => IntentMatch {
                title: words::UNDERSTANDING.to_string(),
                ..blank
            },
            None => blank,
        }
    }

    /// After every publication: the highlight takes the reducer's row, which it kept through
    /// whatever arrived. The list re-anchors by place (Jev's rows keep one key per place), so its
    /// own pick can name a different reading than the one the user was on.
    pub(crate) fn one_window_published(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.current_view, AppView::ScriptList) {
            return;
        }
        if self.one_window_owns_list() {
            self.one_window_show_selection(cx);
        }
        self.one_window_follow_own_selection(cx);
        // Re-anchoring the old first suggestion would scroll the entire new subject panel
        // out of sight, leaving Return bound to an invisible action on first show.
        let page = self
            .has_launcher_tabs()
            .then(|| self.one_window.subject_page())
            .flatten()
            .map(|p| p.id);
        if page != self.one_window_host.published_subject_page
            && page.is_some()
            && self.filter_text.is_empty()
            && self.one_window.list().sel == 0
        {
            self.main_list_state.scroll_to(gpui::ListOffset {
                item_ix: 0,
                offset_in_item: gpui::px(0.),
            });
        }
        self.one_window_host.published_subject_page = page;
    }

    // ------------------------------------------------------------------ Jev

    fn one_window_ask_jev(
        &mut self,
        text: String,
        queue: &mut VecDeque<Event>,
        cx: &mut Context<Self>,
    ) {
        if !self.one_window_owns(&text) || self.one_window.list().jev.is_down_for(&text) {
            return;
        }
        if let Some(plan) = self
            .intent_tab_browser()
            .and_then(|app| crate::intent::engine::named_tab_plan(&text, app))
        {
            let readings = crate::intent::readings(plan);
            self.one_window_host.jev_pending = None;
            self.intent_launcher
                .show(&text, self.intent_screen_key(), &readings);
            queue.push_back(Event::Jev {
                rows: jev_rows(&readings),
                text,
            });
            return;
        }
        if self.intent_launcher.is_off() {
            queue.push_back(Event::JevDown { text });
            return;
        }
        if self.main_services.is_production()
            && (crate::intent::sensitive_words(&text)
                || self
                    .one_window
                    .list()
                    .names
                    .iter()
                    .any(|r| !r.fallback && r.title.eq_ignore_ascii_case(text.trim())))
        {
            self.one_window_cancel_jev();
            queue.push_back(Event::Jev { rows: vec![], text });
            return;
        }
        if !crate::intent::wants_reading(&text) {
            queue.push_back(Event::Jev {
                rows: sk_one_window::list::stand_in_rows(&text),
                text,
            });
            return;
        }
        let screen = self.intent_screen_key();
        if let Some(readings) = self.intent_launcher.recalled(&text, screen) {
            self.one_window_host.jev_pending = None;
            self.intent_launcher.show(&text, screen, &readings);
            queue.push_back(Event::Jev {
                rows: jev_rows(&readings),
                text,
            });
            return;
        }
        self.one_window_send_jev(text, screen, cx);
    }

    pub(super) fn one_window_send_jev(
        &mut self,
        text: String,
        screen: u64,
        cx: &mut Context<Self>,
    ) {
        self.one_window_host.jev_asked += 1;
        let id = self.one_window_host.jev_asked;
        self.one_window_host.jev_pending = Some((id, text.clone()));
        self.ask_jev(id, text, screen, cx);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(JEV_TIMEOUT).await;
            let _ = this.update(cx, |app, cx| app.one_window_jev_timed_out(id, cx));
        })
        .detach();
    }

    pub(super) fn one_window_cancel_jev(&mut self) {
        self.one_window_host.jev_pending = None;
    }

    /// Jev answered the request `id` for `text`.
    pub(crate) fn one_window_jev_answered(
        &mut self,
        id: u64,
        text: String,
        screen: u64,
        result: Result<Plan, JevError>,
        cx: &mut Context<Self>,
    ) {
        let current = self
            .one_window_host
            .jev_pending
            .as_ref()
            .is_some_and(|(pending, words)| *pending == id && words == &text);
        // Identical words can belong to a later request or a different screen. Neither
        // a successful late answer nor a timeout may replace that newer interpretation.
        if !current {
            return;
        }
        self.one_window_host.jev_pending = None;
        if screen != self.intent_screen_key() {
            self.one_window_talk_unread(&text, cx);
            self.one_window_handle(Event::JevDown { text }, cx);
            self.one_window_republish("jev_context_changed", cx);
            return;
        }
        let event = match result {
            Ok(plan) => {
                let readings = crate::intent::readings(plan);
                self.intent_launcher.remember(&text, screen, &readings);
                if self.one_window.list().text == text {
                    self.intent_launcher.show(&text, screen, &readings);
                }
                self.one_window_talk_reading(&text, &readings, cx);
                tracing::info!(
                    target: "script_kit::intent",
                    event = "jev_answered",
                    rows = readings.len(),
                    provider_ms = readings.first().map(|r| r.plan.timing.ms),
                    input_tokens = readings.first().map(|r| r.plan.timing.input_tokens),
                );
                Event::Jev {
                    rows: jev_rows(&readings),
                    text,
                }
            }
            Err(error) => {
                self.one_window_talk_unread(&text, cx);
                tracing::info!(target: "script_kit::intent", event = "jev_failed", error = %error);
                self.intent_launcher.failed(&error);
                Event::JevDown { text }
            }
        };
        self.one_window_handle(event, cx);
        self.one_window_republish("jev_answer", cx);
    }

    fn one_window_jev_timed_out(&mut self, id: u64, cx: &mut Context<Self>) {
        let Some((_, text)) = self
            .one_window_host
            .jev_pending
            .take_if(|(pending, _)| *pending == id)
        else {
            return;
        };
        tracing::info!(target: "script_kit::intent", event = "jev_timed_out");
        self.one_window_talk_unread(&text, cx);
        self.one_window_handle(Event::JevDown { text }, cx);
        self.one_window_republish("jev_timeout", cx);
    }

    // ------------------------------------------------------------------ running

    /// Runs a row the reducer chose: one of Jev's rows by its plan, anything else as the
    /// launcher row the list shows for it.
    fn one_window_run(&mut self, row: Row, cx: &mut Context<Self>) -> anyhow::Result<()> {
        tracing::info!(target: "script_kit::one_window", event = "run", key = %row.key);
        if let Some(query) = row
            .key
            .strip_prefix(sk_one_window::contract::ALL_RESULTS_KEY)
        {
            return self.one_window_open_all_results(query, cx);
        }
        if self.one_window_route_work(&row, cx) {
            return Ok(());
        }
        self.one_window_host.last_run = Some(row.clone());
        self.one_window_host.finishing = Some(row.clone());
        if let Some(id) = row.key.strip_prefix(super::staging::ACTION_KEY) {
            self.one_window_run_action(id.to_string(), cx);
            return Ok(());
        }
        if let Some(ran) = self.one_window_run_capture(&row, cx) {
            return ran;
        }
        let text = self.one_window.text().to_string();
        if let Some(ran) = self.one_window_run_act(&row, cx) {
            return ran;
        }
        if let Some(ran) = self.one_window_run_ledger_row(&row, cx) {
            return ran;
        }
        if let Some(ran) = self.one_window_run_learns_row(&row, cx) {
            return ran;
        }
        if let Some(ran) = self.run_jev_row(&row.key, &text, cx) {
            return ran.map_err(anyhow::Error::msg);
        }
        if row.key.starts_with("jev:") {
            let screen = self.intent_screen_key();
            self.one_window_send_jev(text, screen, cx);
            self.one_window_notice(
                crate::line_notice::INFO,
                format!("Jev is reading “{}” again · ↵ in a moment", row.title),
                cx,
            );
            anyhow::bail!("intent_plan_missing");
        }
        let list = self.one_window.list();
        let position = list
            .visible()
            .iter()
            .position(|item| list.row(item).is_some_and(|shown| shown.key == row.key))
            .ok_or_else(|| anyhow::anyhow!("one_window_row_not_listed"))?;
        let grouped_index = self
            .main_menu_committed_rows()
            .iter()
            .find(|projected| projected.selectable_ordinal == Some(position))
            .map(|projected| projected.grouped_index)
            .ok_or_else(|| anyhow::anyhow!("one_window_row_not_published"))?;
        if grouped_index != self.selected_index {
            self.select_main_menu_row(grouped_index, MainMenuSelectionOrigin::Keyboard, cx);
        }
        self.one_window_remember_rerun(&row);
        let observation = self.selected_row_dispatch_observation();
        self.run_selected_row(observation, cx).map(|_| ())
    }

    /// An app launched from the list: the reducer hides the window and records it.
    pub(crate) fn one_window_app_launched(&mut self, app: &str, cx: &mut Context<Self>) {
        match self.one_window_host.finishing.take() {
            Some(row) if self.one_window_owns(&self.computed_filter_text) => {
                self.one_window_handle(
                    Event::Ran {
                        row,
                        app: Some(app.to_string()),
                    },
                    cx,
                );
            }
            _ => self.close_and_reset_window(cx),
        }
    }

    /// A row finished where it ran and opened nothing.
    pub(crate) fn one_window_ran_here(&mut self, cx: &mut Context<Self>) {
        if let Some(row) = self.one_window_host.finishing.take() {
            self.one_window_handle(Event::Ran { row, app: None }, cx);
        }
    }

    /// What a row opened has finished.
    pub(crate) fn one_window_done(&mut self, cx: &mut Context<Self>) {
        let Some(row) = self.one_window_host.finishing.take() else {
            return;
        };
        self.one_window_handle(
            Event::Done(sk_one_window::Done {
                icon: row.icon.unwrap_or_else(|| "check".to_string()),
                title: row.title,
                ..sk_one_window::Done::default()
            }),
            cx,
        );
    }

    /// The window comes back after it went away for a launch.
    pub(crate) fn one_window_summon(&mut self, cx: &mut Context<Self>) {
        if self.one_window.is_away() {
            self.one_window_handle(Event::Summon, cx);
        }
    }

    /// What the reducer last ran.
    pub(crate) fn one_window_last_run(&self) -> Option<&Row> {
        self.one_window_host.last_run.as_ref()
    }

    /// The reducer's state as `getState` reports it.
    pub(crate) fn one_window_automation_snapshot(&self, cx: &Context<Self>) -> serde_json::Value {
        let list = self.one_window.list();
        let now = self.one_window_peek_now(cx);
        let cursor_voice = self.cursor_voice_state();
        let marks = {
            let input = self.gpui_input_state.read(cx);
            let value = input.value();
            input
                .mark_ranges()
                .iter()
                .zip(input.mark_range_roles())
                .map(|((range, _), word)| {
                    serde_json::json!({
                        "word": word,
                        "range": [range.start, range.end],
                        "text": value.get(range.clone()),
                    })
                })
                .collect::<Vec<_>>()
        };
        serde_json::json!({
            "owned": self.one_window_owns_list(),
            "text": self.one_window.text(),
            "cursorDictation":cursor_voice,
            "marks": marks,
            "inputWords": self.gpui_input_state.read(cx).value().to_string(),
            "context": self.one_window.context(),
            "chipAt": self.one_window.chip_at(),
            "agentOwns": self.one_window.agent_owns(),
            "subjectTabs": self.subject_tab_elements(),
            "header": self.one_window_drawn_header(),
            "enter": self.one_window.enter_target(self.one_window_host.now),
            "enterArmed": self.one_window_enter_arming(),
            "selected": list.sel,
            "jevRows": (0..list.jev.len())
                .filter_map(|i| list.jev.get(i))
                .map(|slot| serde_json::json!({
                    "key": slot.row.as_ref().map(|row| row.key.as_str()),
                    "title": slot.row.as_ref().map(|row| row.title.as_str()),
                    "stale": slot.stale,
                }))
                .collect::<Vec<_>>(),
            "jevSettled": list.jev.settled_for(&list.text),
            "heightPx": self.one_window_fixed_height(),
            "nowMs": now,
            "view": self.one_window.views().top().map(|top| top.name.as_str()),
            "crumb": self.one_window.views().crumb(),
            "strip": super::work_view::strip_snapshot(self),
            "lastRun": self.one_window_last_run().map(|row| serde_json::json!({
                "key": row.key,
                "title": row.title,
            })),
            "placeholder": self.gpui_input_state.read(cx).presentation().placeholder().to_string(),
            "line": self.observable_prompt_header().line,
            "recent": self.one_window.recent_line(now),
            "staged": self.one_window.staged().map(|staged| staged.subject.title.as_str()),
        })
    }

    /// What the ↵ button says now.
    pub(crate) fn one_window_enter_label(&self) -> String {
        self.one_window.enter_target(self.one_window_host.now).label
    }

    /// Enter pressed now would be ignored; the ↵ button shows its arming fill.
    pub(crate) fn one_window_enter_arming(&self) -> bool {
        self.one_window.enter_armed_left(self.one_window_host.now) > 0
    }
}

/// What the line says when a row didn't run and nothing along the way said why.
fn run_failure_words(title: &str, error: &anyhow::Error) -> String {
    let why = match error.to_string().as_str() {
        "intent_app_missing" => "the app isn't installed",
        "intent_builtin_missing" | "intent_script_missing" | "intent_scriptlet_missing" => {
            "that command is gone"
        }
        "intent_step_not_combinable" => "Jev's plan mixes steps that can't run together",
        "intent_plan_has_no_launcher_line" => "Jev's plan has nothing to type",
        "intent_brain_write_failed" => "today's note couldn't be written",
        "intent_clipboard_empty" => "the clipboard is empty",
        "intent_scripts_dir_unavailable"
        | "intent_script_name_taken"
        | "intent_script_write_failed" => "the script couldn't be saved",
        _ => return format!("Couldn't run “{title}”"),
    };
    format!("Couldn't run “{title}”: {why}")
}

/// What one of Jev's rows says under its title.
fn jev_row_subtitle(plan: &Plan) -> String {
    if let Some(why) = crate::intent::launcher::unavailable(plan) {
        format!("Can't do this yet · needs {why}")
    } else if plan.ready {
        crate::intent::plan_subtitle(plan)
    } else {
        match plan.missing.as_deref() {
            Some(missing) => format!("Needs {missing}"),
            None => "Needs more to go on".to_string(),
        }
    }
}
