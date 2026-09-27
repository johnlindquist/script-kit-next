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

use gpui::{AppContext as _, Context};
use sk_one_window::contract::{words, MAX_MATCHES};
use sk_one_window::{Effect, Event, HeightDecision, HeightMode, Item, Key, Row};

use crate::intent::{JevError, JevRow, Plan};
use crate::list_item::GroupedListItem;
use crate::scripts::{IntentMatch, SearchResult};
use crate::{
    AppView, MainMenuDispatchResult, MainMenuSelectionOrigin, MainWindowMode, ScriptListApp,
};

/// Jev counts as down for the words it was asked about once this passes without an answer.
const JEV_TIMEOUT: Duration = Duration::from_secs(5);
/// How often a shrinking window is redrawn.
const SHRINK_FRAME: Duration = Duration::from_millis(8);

/// Milliseconds on the host's clock.
type Ms = sk_one_window::Ms;

#[derive(Default)]
pub(crate) struct OneWindowHost {
    epoch: Option<Instant>,
    now: Ms,
    /// The one pending `Tick`; a timer for any other time is stale.
    wake_at: Option<Ms>,
    /// Bumped by every height decision so an older shrink stops drawing.
    height_generation: u64,
    /// The main window's height as the reducer last placed it.
    height: Option<f32>,
    window: Option<gpui::AnyWindowHandle>,
    /// Words the reducer put in the input; their echo from the input isn't typing.
    echo: Option<String>,
    jev_asked: u64,
    jev_pending: Option<(u64, String)>,
    /// A row that ran and opened something; its title is what finished when that ends.
    finishing: Option<Row>,
    last_run: Option<Row>,
    pub(crate) work: super::work::WorkHost,
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
        TIER_SUBSTRING_PRIMARY, TIER_WORD_BOUNDARY_PRIMARY,
    };
    match result.match_tier() {
        TIER_EXACT_PRIMARY.. => 100,
        TIER_PREFIX_PRIMARY.. => 95,
        TIER_WORD_BOUNDARY_PRIMARY.. => 90,
        TIER_SUBSTRING_PRIMARY.. => 80,
        TIER_ACRONYM_PRIMARY.. => 75,
        TIER_COMPACT_FUZZY_PRIMARY.. => 40,
        TIER_ALIAS.. => 70,
        TIER_KEYWORD.. => 60,
        TIER_DESCRIPTION.. => 35,
        TIER_FILENAME.. => 30,
        TIER_PATH.. => 20,
        1.. => 10,
        _ => 0,
    }
}

/// The fallbacks the window offers when nothing matched, by the key prefix the reducer knows
/// them by: search the web, or ask the model.
fn offered_fallback(result: &SearchResult) -> Option<&'static str> {
    let SearchResult::Fallback(fallback) = result else {
        return None;
    };
    match &fallback.fallback {
        crate::fallbacks::FallbackItem::Builtin(item) if item.id == "search-google" => {
            Some("fallback-google")
        }
        crate::fallbacks::FallbackItem::Builtin(item) if item.id == "search-google-ai" => {
            Some("fallback-ai")
        }
        _ => None,
    }
}

fn jev_rows(readings: &[JevRow]) -> Vec<Row> {
    readings
        .iter()
        .map(|reading| Row {
            needs: super::work::needs_of(&reading.plan),
            ..Row::new(reading.key.clone(), reading.plan.title.clone())
        })
        .collect()
}

impl ScriptListApp {
    fn one_window_now(&mut self, cx: &Context<Self>) -> Ms {
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

    /// Whether the reducer owns the list for these words. Launcher syntax, the calculator,
    /// attachment portals and pickers keep their own lists.
    pub(crate) fn one_window_owns(&self, text: &str) -> bool {
        matches!(self.current_view, AppView::ScriptList)
            && self.inline_calculator.is_none()
            && self.active_script_list_attachment_portal_kind().is_none()
            && !self.menu_syntax_object_selector_state.owns_main_list()
            && !self.menu_syntax_trigger_picker_state.owns_main_list()
            && !self.spine_projection_owns_main_list()
            && !crate::menu_syntax::active_filter_head_owns_main_list(text)
            && self.menu_syntax_mode.capture_for(text).is_none()
            && self.menu_syntax_mode.command_for(text).is_none()
            && self.menu_syntax_mode.incomplete_hint_for(text).is_none()
            && !self.menu_syntax_mode.capture_composer_owns_input_for(text)
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
    fn one_window_handle_turn(&mut self, events: Vec<Event>, cx: &mut Context<Self>) -> Handled {
        let mut queue = VecDeque::from(events);
        let mut handled = Handled::default();
        let mut runs = Vec::new();
        let mut works = Vec::new();
        let mut first = true;
        let now = self.one_window_now(cx);
        while let Some(event) = queue.pop_front() {
            let outcome = self.one_window.handle(now, event);
            if std::mem::take(&mut first) {
                handled.consumed = outcome.consumed;
            }
            for effect in outcome.effects {
                match effect {
                    Effect::Run { row, .. } => runs.push(row),
                    Effect::PassKey(sk_one_window::Key::Tab)
                        if self.one_window_tab_hands_to_agent() =>
                    {
                        handled.consumed = true;
                        queue.push_back(Event::Work(sk_one_window::work::WorkEvent::Agent {
                            task: self.one_window.text().trim().to_string(),
                        }));
                    }
                    Effect::PassKey(_) => handled.passed = true,
                    Effect::EnterIgnored => {
                        handled.ignored = true;
                        tracing::info!(target: "script_kit::one_window", event = "enter_ignored");
                    }
                    Effect::AskJev { text } => self.one_window_ask_jev(text, &mut queue, cx),
                    Effect::Height(decision) => self.one_window_apply_height(decision, cx),
                    Effect::WakeAt(at) => self.one_window_wake_at(at, cx),
                    Effect::SetInput(text) => self.one_window_set_input(text, cx),
                    Effect::Hide { .. } => self.close_and_reset_window(cx),
                    // Summon comes from the show path, which is already showing the window.
                    Effect::Show => {}
                    Effect::ViewKey(key) => match self.one_window.views().top() {
                        Some(top) if super::work::WORK_VIEWS.contains(&top.name.as_str()) => {
                            queue.push_back(Event::Work(sk_one_window::work::WorkEvent::ViewKey(
                                key,
                            )));
                        }
                        _ => tracing::info!(
                            target: "script_kit::one_window",
                            event = "effect_not_hosted",
                            effect = ?Effect::ViewKey(key),
                        ),
                    },
                    Effect::Work(work) => works.push(work),
                    effect @ (Effect::Held { .. }
                    | Effect::Undo { .. }
                    | Effect::Again { .. }
                    | Effect::OpenScript { .. }
                    | Effect::SaveScript { .. }
                    | Effect::ActionsFor(_)) => tracing::info!(
                        target: "script_kit::one_window",
                        event = "effect_not_hosted",
                        effect = ?effect,
                    ),
                }
            }
        }
        for row in runs {
            handled.ran = true;
            if let Err(error) = self.one_window_run(row, cx) {
                handled.run_error = Some(error);
            }
        }
        for work in works {
            self.one_window_apply_work(work, cx);
        }
        cx.notify();
        handled
    }

    /// A key the list got. The reducer sees the row the user is on first, and the highlight
    /// follows wherever the key moved it.
    pub(crate) fn one_window_key(&mut self, key: Key, cx: &mut Context<Self>) -> Handled {
        self.one_window_follow_host_selection(cx);
        let before = self.one_window_jev_token();
        let handled = self.one_window_handle(Event::Key(key), cx);
        self.one_window_show_selection(cx);
        if self.one_window_jev_token() != before {
            self.one_window_republish("one_window_key", cx);
        }
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
        if !self.one_window_owns_list() {
            return;
        }
        let Some(position) = self.one_window_host_position() else {
            return;
        };
        if position != self.one_window.list().sel {
            self.one_window_handle(Event::Select(position), cx);
        }
    }

    fn one_window_show_selection(&mut self, cx: &mut Context<Self>) {
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
        if self.intent_launcher.is_off() && !text.trim().is_empty() {
            turn.push(Event::JevDown {
                text: text.to_string(),
            });
        }
        self.one_window_handle_turn(turn, cx);
    }

    fn one_window_set_input(&mut self, text: String, cx: &mut Context<Self>) {
        if text == self.filter_text || !matches!(self.current_view, AppView::ScriptList) {
            return;
        }
        self.one_window_host.echo = Some(text.clone());
        self.filter_text = text.clone();
        self.pending_filter_sync = true;
        self.queue_filter_compute(text, cx);
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

    fn one_window_republish(&mut self, reason: &'static str, cx: &mut Context<Self>) {
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
        hasher.finish()
    }

    // ------------------------------------------------------------------ the list

    /// The list for typed words is the reducer's layout: the name matches the search found,
    /// Jev's three rows, and their headers. An empty input keeps the launcher's own list, led by
    /// a header. Lists the reducer doesn't own come back unchanged.
    pub(crate) fn one_window_list(
        &mut self,
        mut grouped: Vec<GroupedListItem>,
        flat: Vec<SearchResult>,
        text: &str,
        cx: &mut Context<Self>,
    ) -> (Vec<GroupedListItem>, Vec<SearchResult>, bool) {
        if !self.one_window_owns(text) {
            return (grouped, flat, false);
        }
        if text.trim().is_empty() {
            if grouped
                .iter()
                .any(|item| matches!(item, GroupedListItem::Item(_)))
                && !matches!(grouped.first(), Some(GroupedListItem::SectionHeader(..)))
            {
                grouped.insert(
                    0,
                    GroupedListItem::SectionHeader(words::SUGGESTED.to_string(), None),
                );
            }
            return (grouped, flat, false);
        }
        if self.one_window.text() != text {
            self.one_window_handle(Event::Text(text.to_string()), cx);
        }
        let (rows, results) = self.one_window_names(&grouped, &flat, text);
        self.one_window_handle(Event::Names(rows), cx);
        let (grouped, flat) = self.one_window_layout(&results, text);
        (grouped, flat, true)
    }

    /// The top name matches in the search's order, then the fallbacks the window offers when
    /// nothing matched.
    fn one_window_names(
        &self,
        grouped: &[GroupedListItem],
        flat: &[SearchResult],
        text: &str,
    ) -> (Vec<Row>, HashMap<String, SearchResult>) {
        let frontmost = crate::ui_window::footer_frontmost_app_name();
        let learned = self.input_history.preferred_result_key(text);
        let mut rows = Vec::new();
        let mut fallbacks = Vec::new();
        let mut results = HashMap::new();
        super::work::steps_row(text, &mut rows, &mut results);
        for item in grouped {
            let GroupedListItem::Item(index) = item else {
                continue;
            };
            let Some(result) = flat.get(*index) else {
                continue;
            };
            let fallback = offered_fallback(result);
            let key = match fallback {
                Some(prefix) => format!("{prefix}:{text}"),
                None => result
                    .stable_selection_key()
                    .unwrap_or_else(|| format!("name/{}", result.name())),
            };
            if results.contains_key(&key) {
                continue;
            }
            let row = Row::new(key.clone(), result.name())
                .with_enter_label(crate::ui_window::main_window_result_action_label(
                    result,
                    frontmost.as_deref(),
                ))
                .with_score(name_strength(result));
            if fallback.is_some() {
                fallbacks.push(row.fallback());
            } else if matches!(result, SearchResult::Fallback(_)) || rows.len() >= MAX_MATCHES {
                continue;
            } else {
                rows.push(Row {
                    learned: learned.is_some() && learned == result.history_result_key().as_deref(),
                    ..row
                });
            }
            results.insert(key, result.clone());
        }
        rows.extend(fallbacks);
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
                Item::Header(head) if head == words::JEV => {
                    grouped.push(GroupedListItem::SectionHeader(
                        format!("{head} · ⌘1–3"),
                        None,
                    ));
                    continue;
                }
                Item::Header(head) => {
                    grouped.push(GroupedListItem::SectionHeader(head.clone(), None));
                    continue;
                }
                Item::Slot(slot) => SearchResult::Intent(self.one_window_jev_match(*slot, text)),
                item => match list.row(item).and_then(|row| results.get(&row.key)) {
                    Some(result) => result.clone(),
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
                title: "Understanding…".to_string(),
                ..blank
            },
            None => blank,
        }
    }

    /// The reducer's rows for an empty input: every row the launcher lists, in order, each
    /// section's first row carrying its header.
    pub(crate) fn one_window_suggested(&mut self, cx: &mut Context<Self>) {
        if !self.computed_filter_text.trim().is_empty()
            || !self.one_window_owns(&self.computed_filter_text)
        {
            return;
        }
        let (grouped, flat) = self.get_grouped_results_cached();
        let frontmost = crate::ui_window::footer_frontmost_app_name();
        let mut head = None;
        let mut rows = Vec::new();
        for row in self.main_menu_committed_rows() {
            let Some(item) = grouped.get(row.grouped_index) else {
                continue;
            };
            if row.grouped_index > 0 {
                if let Some(GroupedListItem::SectionHeader(label, _)) =
                    grouped.get(row.grouped_index - 1)
                {
                    head = Some(label.clone());
                }
            }
            if row.selectable_ordinal.is_none() {
                continue;
            }
            let GroupedListItem::Item(index) = item else {
                continue;
            };
            let Some(result) = flat.get(*index) else {
                continue;
            };
            let mut suggested = Row::new(row.stable_key.clone(), result.name()).with_enter_label(
                crate::ui_window::main_window_result_action_label(result, frontmost.as_deref()),
            );
            suggested.head = head.take();
            rows.push(suggested);
        }
        self.one_window_handle(Event::Suggested(rows), cx);
    }

    /// After every publication: the reducer follows the highlight and hears how tall the list
    /// wants the window.
    pub(crate) fn one_window_published(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.current_view, AppView::ScriptList) {
            return;
        }
        self.one_window_follow_host_selection(cx);
        let px = self.one_window_natural_height();
        self.one_window_handle(Event::Content { px }, cx);
    }

    /// How tall the launcher may get in its current mode.
    fn one_window_height_ceiling(&self) -> f32 {
        match self.main_window_mode {
            MainWindowMode::Mini => crate::window_resize::main_window_full_height(),
            MainWindowMode::Full => crate::window_resize::current_standard_height(),
        }
    }

    /// The line above the input, the input, the list as drawn, and the floating footer's zone.
    fn one_window_natural_height(&self) -> f32 {
        if self
            .one_window
            .views()
            .top()
            .is_some_and(|top| super::work::WORK_VIEWS.contains(&top.name.as_str()))
        {
            return self.one_window_height_ceiling();
        }
        let (grouped, _) = self.get_grouped_results_cached();
        let heights = crate::scrolling::list_geometry::GroupedListRowHeights::for_theme(
            self.current_main_menu_theme,
        );
        let list = crate::scrolling::list_geometry::content_height(grouped.len(), |ix| {
            heights.row_height(&grouped[ix], ix)
        });
        let chrome = sk_one_window::contract::app::SLOT_LINE_PX
            + f32::from(crate::main_list_header_overlay_height(
                self.current_main_menu_theme.def(),
            ))
            + crate::footer_popup::main_window_footer_reservation_height();
        (chrome + list).min(self.one_window_height_ceiling())
    }

    // ------------------------------------------------------------------ height

    fn one_window_apply_height(&mut self, decision: HeightDecision, cx: &mut Context<Self>) {
        self.one_window_host.height_generation += 1;
        let generation = self.one_window_host.height_generation;
        if decision.mode != HeightMode::Shrink {
            self.one_window_set_height(decision.px, cx);
            return;
        }
        let (from, to) = (decision.from, decision.px);
        let span = Duration::from_millis(decision.anim_ms.max(1));
        self.one_window_set_height(from, cx);
        cx.spawn(async move |this, cx| {
            let start = cx.background_executor().now();
            loop {
                cx.background_executor().timer(SHRINK_FRAME).await;
                let t = (cx
                    .background_executor()
                    .now()
                    .saturating_duration_since(start)
                    .as_secs_f32()
                    / span.as_secs_f32())
                .min(1.0);
                let height = from + (to - from) * sk_one_window::contract::shrink_ease(t);
                let drawing = this
                    .update(cx, |app, cx| {
                        let current = app.one_window_host.height_generation == generation;
                        if current {
                            app.one_window_set_height(height, cx);
                        }
                        current
                    })
                    .unwrap_or(false);
                if !drawing || t >= 1.0 {
                    break;
                }
            }
        })
        .detach();
    }

    fn one_window_set_height(&mut self, height: f32, cx: &mut Context<Self>) {
        self.one_window_host.height = Some(height);
        crate::window_resize::set_main_window_height(height);
        let Some(handle) = self.one_window_host.window else {
            return;
        };
        if !matches!(self.current_view, AppView::ScriptList) {
            return;
        }
        let height = height.min(self.one_window_height_ceiling());
        cx.defer(move |cx| {
            let _ = cx.update_window(handle, |_, window, cx| {
                if crate::runtime_policy::is_owned_evaluation() {
                    // The owned window is hidden and has no screen edge to keep.
                    let width = window.viewport_size().width;
                    window.resize(gpui::size(width, gpui::px(height)));
                } else {
                    // The native path keeps the top edge, so the input never moves.
                    crate::window_ops::queue_resize_with_width(
                        height,
                        Some(crate::window_resize::MAIN_WINDOW_WIDTH),
                        window,
                        cx,
                    );
                }
            });
        });
    }

    /// The window the main list draws in, where its height is applied.
    pub(crate) fn one_window_rendered_in(&mut self, window: &gpui::Window) {
        self.one_window_host.window = Some(window.window_handle());
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
        if self.intent_launcher.is_off() {
            queue.push_back(Event::JevDown { text });
            return;
        }
        if !crate::intent::wants_reading(&text) {
            queue.push_back(Event::Jev {
                text,
                rows: Vec::new(),
            });
            return;
        }
        let screen = self.intent_screen_key();
        if let Some(readings) = self.intent_launcher.recalled(&text, screen) {
            self.intent_launcher.show(&text, &readings);
            queue.push_back(Event::Jev {
                rows: jev_rows(&readings),
                text,
            });
            return;
        }
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
            .is_some_and(|(pending, _)| *pending == id);
        if current {
            self.one_window_host.jev_pending = None;
        }
        let event = match result {
            Ok(plan) => {
                let readings = crate::intent::readings(plan);
                self.intent_launcher.remember(&text, screen, &readings);
                self.intent_launcher.show(&text, &readings);
                tracing::info!(
                    target: "script_kit::intent",
                    event = "jev_answered",
                    rows = readings.len(),
                );
                Event::Jev {
                    rows: jev_rows(&readings),
                    text,
                }
            }
            Err(error) => {
                tracing::info!(target: "script_kit::intent", event = "jev_failed", error = %error);
                self.intent_launcher.failed(&error);
                if !current {
                    return;
                }
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
        self.one_window_handle(Event::JevDown { text }, cx);
        self.one_window_republish("jev_timeout", cx);
    }

    // ------------------------------------------------------------------ running

    /// Runs a row the reducer chose: one of Jev's rows by its plan, anything else as the
    /// launcher row the list shows for it.
    fn one_window_run(&mut self, row: Row, cx: &mut Context<Self>) -> anyhow::Result<()> {
        tracing::info!(target: "script_kit::one_window", event = "run", key = %row.key);
        if self.one_window_route_work(&row, cx) {
            return Ok(());
        }
        self.one_window_host.last_run = Some(row.clone());
        self.one_window_host.finishing = Some(row.clone());
        let text = self.one_window.text().to_string();
        if let Some(ran) = self.run_jev_row(&row.key, &text, cx) {
            return ran.map_err(anyhow::Error::msg);
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
    pub(crate) fn one_window_automation_snapshot(&self) -> serde_json::Value {
        let list = self.one_window.list();
        serde_json::json!({
            "owned": self.one_window_owns_list(),
            "text": self.one_window.text(),
            "header": list.header(),
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
            "heightPx": self.one_window_host.height,
            "view": self.one_window.views().top().map(|top| top.name.as_str()),
            "crumb": self.one_window.views().crumb(),
            "strip": super::work_view::strip_snapshot(self),
            "lastRun": self.one_window_last_run().map(|row| serde_json::json!({
                "key": row.key,
                "title": row.title,
            })),
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

/// What one of Jev's rows says under its title.
fn jev_row_subtitle(plan: &Plan) -> String {
    if plan.ready {
        crate::intent::plan_subtitle(plan)
    } else {
        match plan.missing.as_deref() {
            Some(missing) => format!("Needs {missing}"),
            None => "Needs more to go on".to_string(),
        }
    }
}
