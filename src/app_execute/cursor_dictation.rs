// Direct dictation owns a frozen cursor, not a Jev Talk reading. Partial audio is never
// inserted, sent to the intent reader, or used to choose an action.
use crate::hotkeys::dictation_keys::{Control as CursorControl, ListeningKeys};
use sk_one_window::cursor_dictation::Route as CursorRoute;

#[derive(Default)]
pub(crate) struct CursorHost {
    active: Option<CursorCapture>,
    retired: Option<u64>,
    spelling_result: Option<crate::dictation::screen_spelling::Receipt>,
    stop_to_delivery_ms: Option<u64>,
    saved_draft: Option<String>,
    pub(crate) reviewing: Option<String>,
    paste_anchor: Option<(
        String,
        CursorField,
        crate::dictation::DictationTargetSelection,
    )>,
    saved_id: Option<String>,
    pub(crate) dictated_input: Option<String>,
}
struct CursorCapture {
    generation: u64,
    app: gpui::WeakEntity<ScriptListApp>,
    held: bool,
    direct_owner: bool,
    stopped: bool,
    revealed: bool,
    route: CursorRoute,
    label: String,
    keys: ListeningKeys,
    take: Option<crate::dictation::recovery::TakeId>,
    history_allowed: bool,
    journal_failed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    revoked: std::sync::Arc<std::sync::atomic::AtomicBool>,
    partial: String,
    spelling: Option<crate::dictation::screen_spelling::Session>,
    stop_started: Option<std::time::Instant>,
    anchor: CursorAnchor,
    selection: crate::dictation::DictationTargetSelection,
}
enum CursorAnchor {
    Input {
        text: String,
        range: std::ops::Range<usize>,
        surface: u64,
    },
    Day(crate::dictation::DayPageDictationDestinationSnapshot),
    External(Option<CursorField>),
}
#[derive(Debug, thiserror::Error)]
#[error("Delivery unknown; check the original field before pasting again")]
struct CursorDispatchUnknown;
struct CursorField {
    #[cfg(target_os = "macos")]
    pid: i32,
    #[cfg(target_os = "macos")]
    element: crate::platform::accessibility::ax::OwnedAxElement,
    #[cfg(target_os = "macos")]
    range: crate::platform::accessibility::TextRangeUtf16,
}
impl CursorField {
    #[cfg(target_os = "macos")]
    fn capture() -> Option<Self> {
        use crate::platform::accessibility::app_identity;
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)
            .ok()?;
        let pid = app_identity::current_frontmost_app_identity().process_id?;
        Self::capture_pid(pid)
    }
    #[cfg(target_os = "macos")]
    fn capture_pid(pid: i32) -> Option<Self> {
        use crate::platform::accessibility::ax;
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)
            .ok()?;
        if pid == std::process::id() as i32 {
            return None;
        }
        let element = ax::focused_ui_element_for_pid(pid).ok()?;
        let ptr = element.as_ptr();
        if ax::subrole(ptr).as_deref() == Some("AXSecureTextField") {
            return None;
        }
        let range = ax::selected_text_range(ptr)?;
        Some(Self {
            pid,
            element,
            range,
        })
    }
    #[cfg(not(target_os = "macos"))]
    fn capture() -> Option<Self> {
        None
    }
    #[cfg(target_os = "macos")]
    fn paste(
        self,
        words: &str,
        selection: &crate::dictation::DictationTargetSelection,
    ) -> anyhow::Result<()> {
        use crate::dictation::FrozenDictationDestination;
        anyhow::ensure!(
            matches!(&selection.destination,FrozenDictationDestination::ExternalApp{pid,..} if *pid==self.pid),
            "The original application changed"
        );
        anyhow::ensure!(
            crate::dictation::capture_frozen_external_destination()
                .ok()
                .as_ref()
                == Some(&selection.destination),
            "The original window changed"
        );
        let now = Self::capture()
            .ok_or_else(|| anyhow::anyhow!("The original field is no longer focused"))?;
        anyhow::ensure!(
            self.pid == now.pid
                && self.element.same_element(&now.element)
                && self.range == now.range
                && !crate::platform::is_main_window_focused(),
            "The original field or selection changed"
        );
        use crate::platform::promised_paste::{paste_text_with_options, PasteOptions};
        anyhow::ensure!(
            crate::permissions::can_post_keystrokes(),
            "No permission to paste"
        );
        let _report = paste_text_with_options(
            words,
            Some(crate::platform::synthetic_keys::KeyTarget::Process(
                self.pid,
            )),
            PasteOptions {
                keep_text_if_unread: false,
            },
        )
        .map_err(|_| CursorDispatchUnknown)?;
        let inserted = crate::platform::accessibility::TextRangeUtf16 {
            location: self.range.location,
            length: words.encode_utf16().count(),
        };
        let after = Self::capture();
        anyhow::ensure!(
            after.as_ref().is_some_and(|f| f.pid == self.pid
                && self.element.same_element(&f.element)
                && f.range.location == inserted.location + inserted.length
                && f.range.length == 0)
                && crate::platform::accessibility::ax::string_for_range(
                    self.element.as_ptr(),
                    inserted
                )
                .is_ok_and(|s| s == words),
            CursorDispatchUnknown
        );
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    fn paste(self, _: &str, _: &crate::dictation::DictationTargetSelection) -> anyhow::Result<()> {
        anyhow::bail!("External field identity unavailable")
    }
}

impl ScriptListApp {
    pub(crate) fn cursor_voice_state(&self) -> serde_json::Value {
        let host = &self.one_window_host.cursor_voice;
        serde_json::json!({"screenSpelling":{"mode":self.config.get_dictation_preferences().screen_spelling_mode(),"capture":host.active.as_ref().and_then(|s|s.spelling.as_ref()).map(|s|s.status()),"lastDelivery":host.spelling_result,"stopToVerifiedDeliveryMs":host.stop_to_delivery_ms},"active":host.active.is_some(),"generation":host.active.as_ref().map(|s|s.generation),"route":host.active.as_ref().map(|s|match s.route{CursorRoute::Original=>"original",CursorRoute::Kit=>"kit",CursorRoute::Markdown=>"markdown"}),"switchShortcut":host.active.as_ref().and_then(|s|s.keys.switch_label.as_deref()),"switchUnavailable":host.active.as_ref().is_some_and(|s|s.keys.switch_unavailable),"cancelAvailable":host.active.as_ref().is_some_and(|s|s.keys.cancel_available),"stopping":host.active.as_ref().is_some_and(|s|s.stopped),"draftAvailable":host.saved_draft.is_some()})
    }
    fn shared_cursor_prompt(&self) -> bool {
        matches!(
            self.current_view,
            AppView::ArgPrompt { .. } | AppView::MiniPrompt { .. } | AppView::MicroPrompt { .. }
        )
    }
    pub(crate) fn cancel_cursor(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(id) = self
            .one_window_host
            .cursor_voice
            .active
            .as_ref()
            .map(|s| s.generation)
        else {
            return false;
        };
        let s = self.one_window_host.cursor_voice.active.as_ref().unwrap();
        s.revoked.store(true, std::sync::atomic::Ordering::Release);
        let take = s.take.clone();
        let saved = take.as_ref().map(|t| t.cancel(&s.partial));
        crate::dictation::discard_private_partial(id);
        if crate::dictation::current_capture_generation() == Some(id) {
            self.request_active_dictation_stop(
                crate::dictation::DictationStopReason::Hotkey,
                false,
                cx,
            );
        }
        self.end_empty_cursor(id, cx);
        match (take, saved) {
            (Some(t), Some(Ok(()))) => self.announce_recovered_cancel(&t, cx),
            (_, Some(Err(e))) => {
                self.notice_failed(format!("Couldn't save cancelled recording: {e}"), cx)
            }
            _ => self.one_window_notice(
                crate::line_notice::INFO,
                "Cancelled · No recording kept",
                cx,
            ),
        }
        true
    }
    pub(crate) fn cursor_dictation_pending(&self) -> bool {
        self.one_window_host.cursor_voice.active.is_some()
    }
    pub(crate) fn prepare_shortcut_launcher(&mut self, cx: &mut Context<Self>) -> bool {
        if self.prompt_receiver.is_some() && !matches!(self.current_view, AppView::ScriptList) {
            return false;
        }
        if let AppView::DayPage { entity } = &self.current_view {
            if !entity
                .clone()
                .update(cx, |v, cx| !v.is_dirty() || v.save_before_unmount(cx))
            {
                self.notice_failed("Couldn't save the open note", cx);
                return false;
            }
        }
        self.one_window_host.note_open_revision =
            self.one_window_host.note_open_revision.wrapping_add(1);
        self.one_window_host.note_waiting = None;
        // An explicit main shortcut means launcher, not the last visible note/chat.
        if !matches!(self.current_view, AppView::ScriptList)
            || self.one_window.views().top().is_some()
        {
            self.reset_to_script_list(cx);
            self.one_window_handle(sk_one_window::Event::Dismissed, cx);
            self.one_window_summon(cx);
        }
        self.pending_focus = Some(FocusTarget::MainFilter);
        true
    }
    pub(crate) fn start_cursor_hold(
        &mut self,
        external: bool,
        today: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::dictation::DictationTarget as T;
        if self.cursor_dictation_pending()
            || self.one_window_talking()
            || crate::dictation::is_dictation_busy()
        {
            return;
        }
        let kit = !today && self.config.get_dictation_preferences().start_in_kit == Some(true);
        let target = if external {
            T::ExternalApp
        } else if matches!(self.current_view, AppView::DayPage { .. }) {
            T::DayPageToday
        } else if !today && self.shared_cursor_prompt() {
            T::MainWindowPrompt
        } else if kit || (!today && matches!(self.current_view, AppView::ScriptList)) {
            T::MainWindowFilter
        } else {
            self.notice_failed("This field doesn't accept shortcut dictation", cx);
            return;
        };
        let action = DictationBuiltinAction::CurrentSurface;
        if self.prepare_dictation_builtin_start(
            action,
            Some(if kit { T::MainWindowFilter } else { target }),
            cx,
        ) != DictationStartPreflight::Ready
        {
            // Download/setup is a separate action. It must not replay a released hold.
            *pending_dictation_model_action().lock() = None;
            return;
        }
        self.start_cursor_recording(target, true, kit, cx);
    }
    fn cursor_context_target(&self) -> crate::dictation::DictationTarget {
        use crate::dictation::DictationTarget as T;
        if !crate::platform::is_main_window_focused() {
            T::ExternalApp
        } else if matches!(self.current_view, AppView::DayPage { .. }) {
            T::DayPageToday
        } else if self.shared_cursor_prompt() {
            T::MainWindowPrompt
        } else {
            T::MainWindowFilter
        }
    }
    fn toggle_kit_recording(
        &mut self,
        direct_owner: bool,
        dctx: &crate::action_helpers::DispatchContext,
        cx: &mut Context<Self>,
    ) -> crate::action_helpers::DispatchOutcome {
        use crate::dictation::DictationTarget as T;
        if let Some(s) = self.one_window_host.cursor_voice.active.as_mut() {
            if direct_owner && !s.direct_owner && !s.stopped {
                s.route = CursorRoute::Kit;
                cx.notify();
                return Self::builtin_success(dctx, "Recording redirected into Kit");
            }
            self.stop_cursor_capture(cx);
            return Self::builtin_success(dctx, "Finishing recording");
        }
        if crate::dictation::is_dictation_busy() || self.one_window_talking() {
            return Self::builtin_error(
                dctx,
                "dictation_busy",
                "Another recording owns the microphone",
                "Finish that recording first",
            );
        }
        let preflight = self.prepare_dictation_builtin_start(
            DictationBuiltinAction::CurrentSurface,
            Some(T::MainWindowFilter),
            cx,
        );
        if preflight != DictationStartPreflight::Ready {
            *pending_dictation_model_action().lock() = None;
            return Self::builtin_success(dctx, preflight.success_detail());
        }
        let target = self.cursor_context_target();
        self.start_cursor_recording(target, false, true, cx);
        if let Some(s) = self.one_window_host.cursor_voice.active.as_mut() {
            s.direct_owner = direct_owner;
            Self::builtin_success(dctx, "Listening for Script Kit")
        } else {
            Self::builtin_error(
                dctx,
                "dictation_unavailable",
                "Could not start recording",
                "Check Dictation Setup",
            )
        }
    }
    fn start_cursor_recording(
        &mut self,
        target: crate::dictation::DictationTarget,
        held: bool,
        kit: bool,
        cx: &mut Context<Self>,
    ) {
        self.begin_cursor_recording(target, held, kit, None, cx);
    }
    #[cfg(feature = "owned-ui-evaluation")]
    pub(crate) fn cursor_fixture_begin(
        &mut self,
        hands_free: bool,
        kit: bool,
        sample: bool,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        use crate::dictation::DictationTarget as T;
        anyhow::ensure!(
            crate::runtime_policy::is_owned_evaluation() && !self.cursor_dictation_pending(),
            "capture_already_started"
        );
        let target = match self.current_view {
            AppView::ScriptList => T::MainWindowFilter,
            AppView::DayPage { .. } => T::DayPageToday,
            _ => anyhow::bail!("no_cursor"),
        };
        let generation = self
            .one_window_host
            .cursor_voice
            .retired
            .unwrap_or(0)
            .saturating_add(1);
        self.begin_cursor_recording(target, !hands_free, kit, Some(generation), cx);
        if sample {
            let s = self
                .one_window_host
                .cursor_voice
                .active
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("no capture"))?;
            let take = ProtectedTake::begin()?;
            take.append(0, &[0.1; 1600], 16000)?;
            s.partial = "keep these words".into();
            s.take = Some(take);
        }
        anyhow::ensure!(self.cursor_dictation_pending(), "capture_not_started");
        Ok(())
    }
    #[cfg(feature = "owned-ui-evaluation")]
    pub(crate) fn cursor_fixture_final(
        &mut self,
        words: String,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            crate::runtime_policy::is_owned_evaluation()
                && !words.is_empty()
                && words.len() <= 8192,
            "invalid_fixture_transcript"
        );
        let s = self
            .one_window_host
            .cursor_voice
            .active
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no_cursor_capture"))?;
        s.stopped = true;
        if let Some(t) = &s.take {
            t.ready(&words, &words)?;
        }
        let id = s.generation;
        let selection = s.selection.clone();
        self.handle_dictation_transcript(
            Ok(Some(words)),
            DictationDeliveryTiming {
                audio_duration: std::time::Duration::ZERO,
                target: selection.target,
                record_history: false,
            },
            Some(selection),
            id,
            None,
            cx,
        );
        Ok(())
    }
    fn begin_cursor_recording(
        &mut self,
        target: crate::dictation::DictationTarget,
        held: bool,
        kit: bool,
        fixture: Option<u64>,
        cx: &mut Context<Self>,
    ) {
        use crate::dictation::DictationTarget as T;
        if self.cursor_dictation_pending() || crate::dictation::is_dictation_busy() {
            return;
        }
        let anchor = match target {
            T::ExternalApp => CursorAnchor::External(CursorField::capture()),
            T::DayPageToday => {
                let AppView::DayPage { entity } = &self.current_view else {
                    return;
                };
                match entity.read(cx).capture_owned_dictation_destination(cx) {
                    Ok(s) => CursorAnchor::Day(s),
                    Err(e) => {
                        self.notice_failed(format!("Cannot dictate into this note: {e}"), cx);
                        return;
                    }
                }
            }
            T::MainWindowFilter | T::MainWindowPrompt => {
                if target == T::MainWindowPrompt && !self.shared_cursor_prompt() {
                    self.notice_failed("This prompt does not expose a dictation cursor", cx);
                    return;
                }
                let input = self.gpui_input_state.read(cx);
                CursorAnchor::Input {
                    text: input.value().to_string(),
                    range: input.selected_range(),
                    surface: self.owned_revision_facts().surface_generation,
                }
            }
            _ => return,
        };
        let selection = match self.capture_dictation_target_selection(target, cx) {
            Ok(s) => s,
            Err(_) if kit && target == T::ExternalApp => {
                crate::dictation::DictationTargetSelection {
                    target,
                    destination: crate::dictation::FrozenDictationDestination::Unavailable {
                        target_id: "original".into(),
                        generation: 0,
                    },
                    display_label: "original field".into(),
                    icon_identity: None,
                    selection_generation: 0,
                }
            }
            Err(e) => {
                self.notice_failed(
                    format!("Cannot identify the dictation destination: {e}"),
                    cx,
                );
                return;
            }
        };
        let label = match &anchor {
            CursorAnchor::External(_) => selection.display_label.clone(),
            CursorAnchor::Input { .. } if target == T::MainWindowPrompt => "prompt field".into(),
            CursorAnchor::Input { .. } => "launcher field".into(),
            CursorAnchor::Day(_) => {
                if let AppView::DayPage { entity } = &self.current_view {
                    entity.read(cx).document_label()
                } else {
                    "Today".into()
                }
            }
        };
        crate::dictation::clear_dictation_return_origin();
        let prefs = self.config.get_dictation_preferences();
        let history_allowed = prefs.save_history_enabled() && fixture.is_none();
        let mut journal_failed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let take = if history_allowed && crate::dictation::takes::keep_recordings_enabled(&prefs) {
            match crate::dictation::recovery::TakeId::begin() {
                Ok(t) => Some(t),
                Err(e) => {
                    self.notice_failed(format!("Couldn't protect this recording: {e}"), cx);
                    return;
                }
            }
        } else {
            None
        };
        let generation = if let Some(id) = fixture {
            if !crate::runtime_policy::is_owned_evaluation() {
                return;
            }
            id
        } else {
            let journal = take.clone().map(|t| {
                let writer = crate::dictation::recovery::AudioJournal::new(t);
                journal_failed = writer.failed.clone();
                writer
            });
            if let Err(e) = crate::dictation::start_protected_recording(target, journal) {
                if let Some(t) = &take {
                    let _ = t.forget();
                }
                self.notice_failed(format!("Couldn't start dictation: {e}"), cx);
                return;
            }
            let Some(generation) = crate::dictation::current_capture_generation() else {
                return;
            };
            crate::dictation::set_dictation_session_selection(selection.clone());
            generation
        };
        let mut spelling = None;
        #[cfg(target_os = "macos")]
        let mode = prefs.screen_spelling_mode();
        #[cfg(target_os = "macos")]
        if !kit && fixture.is_none() && mode != "off" {
            if let (CursorAnchor::External(Some(field)), Some(window), Some(bundle)) = (
                &anchor,
                selection.destination.external_window_id(),
                selection.icon_identity.as_deref(),
            ) {
                if matches!(&selection.destination, crate::dictation::FrozenDictationDestination::ExternalApp {pid,..} if *pid == field.pid)
                {
                    let owner = crate::dictation::screen_spelling::Owner {
                        generation,
                        pid: field.pid,
                        window,
                    };
                    spelling = crate::platform::dictation_spelling::start(
                        owner,
                        &field.element,
                        bundle,
                        &selection.display_label,
                        mode,
                        prefs.vocabulary_words(),
                    );
                }
            }
        }
        self.one_window_host.cursor_voice.spelling_result = None;
        self.one_window_host.cursor_voice.stop_to_delivery_ms = None;
        let keys = ListeningKeys::install(
            generation,
            held,
            prefs.switch_shortcut.as_deref().unwrap_or("Shift+Escape"),
            fixture.is_some(),
        );
        let revealed = target == T::ExternalApp && !script_kit_gpui::is_main_window_visible();
        self.one_window_host.cursor_voice.active = Some(CursorCapture {
            generation,
            app: cx.weak_entity(),
            held,
            direct_owner: false,
            stopped: false,
            revealed,
            route: if kit {
                CursorRoute::Kit
            } else {
                CursorRoute::Original
            },
            label,
            keys,
            take,
            history_allowed,
            journal_failed,
            revoked: Default::default(),
            partial: String::new(),
            spelling,
            stop_started: None,
            anchor,
            selection,
        });
        if fixture.is_some() {
            cx.notify();
            return;
        }
        if revealed {
            script_kit_gpui::set_main_window_visible(true);
            crate::platform::show_main_window_background();
        }
        cx.notify();
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(16))
                .await;
            let keep = this
                .update(cx, |app, cx| {
                    app.drain_cursor_controls(cx);
                    if !app
                        .one_window_host
                        .cursor_voice
                        .active
                        .as_ref()
                        .is_some_and(|s| s.generation == generation)
                    {
                        return false;
                    }
                    if app
                        .one_window_host
                        .cursor_voice
                        .active
                        .as_ref()
                        .is_some_and(|s| s.stopped)
                    {
                        return true;
                    }
                    if let Some(partial) =
                        crate::dictation::snapshot_live_state().map(|live| live.transcript)
                    {
                        if let Some(s) = app.one_window_host.cursor_voice.active.as_mut() {
                            if s.partial != partial {
                                s.partial = partial;
                                cx.notify();
                            }
                        }
                    }

                    if crate::dictation::dictation_auto_stop_due() {
                        app.stop_cursor_capture(cx);
                        return true;
                    }
                    true
                })
                .unwrap_or(false);
            if !keep {
                break;
            }
        })
        .detach();
    }
    fn drain_cursor_controls(&mut self, cx: &mut Context<Self>) {
        let Some(s) = self.one_window_host.cursor_voice.active.as_mut() else {
            return;
        };
        let events = s.keys.take_pending();
        if !s.stopped && s.keys.after_toggle_release() {
            cx.notify();
        }
        for control in events {
            match control {
                CursorControl::Switch => {
                    self.toggle_cursor_kit(cx);
                }
                CursorControl::Cancel => {
                    self.cancel_cursor(cx);
                    break;
                }
            }
        }
    }
    pub(crate) fn cursor_local_key(
        &mut self,
        event: &gpui::KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(s) = self.one_window_host.cursor_voice.active.as_ref() else {
            return false;
        };
        // Native Carbon owns these keys in a live session. Hidden evaluation uses the
        // same chord table without registering anything on the operator's desktop.
        match s
            .keys
            .fixture_control(&event.keystroke.key, event.keystroke.modifiers)
        {
            Some(_) if event.is_held => true,
            Some(CursorControl::Switch) => {
                self.toggle_cursor_kit(cx);
                true
            }
            Some(CursorControl::Cancel) => self.cancel_cursor(cx),
            None => false,
        }
    }
    pub(crate) fn toggle_cursor_kit(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(s) = self
            .one_window_host
            .cursor_voice
            .active
            .as_mut()
            .filter(|s| !s.stopped)
        else {
            return false;
        };
        s.spelling.take(); // Never reuse window evidence after a destination switch.
        s.route.toggle_kit();
        cx.notify();
        true
    }
    fn stop_cursor_capture(&mut self, cx: &mut Context<Self>) {
        self.drain_cursor_controls(cx);
        let Some(s) = self.one_window_host.cursor_voice.active.as_mut() else {
            return;
        };
        if s.stopped || crate::dictation::current_capture_generation() != Some(s.generation) {
            return;
        }
        s.stopped = true;
        s.keys.finishing();
        self.request_active_dictation_stop(crate::dictation::DictationStopReason::Hotkey, true, cx);
        cx.notify();
    }
    pub(crate) fn finish_cursor_hold(&mut self, cx: &mut Context<Self>) {
        if self
            .one_window_host
            .cursor_voice
            .active
            .as_ref()
            .is_some_and(|s| s.held)
        {
            self.stop_cursor_capture(cx);
        }
    }
    fn take_cursor(&mut self, id: u64) -> Option<CursorCapture> {
        let host = &mut self.one_window_host.cursor_voice;
        if host.active.as_ref()?.generation != id {
            return None;
        }
        host.retired = Some(id);
        host.active.take()
    }
    fn close_cursor_indicator(&self, capture: &CursorCapture, cx: &mut Context<Self>) {
        if capture.revealed && !crate::platform::is_main_window_focused() {
            script_kit_gpui::set_main_window_visible(false);
            crate::platform::defer_hide_main_window(cx);
        }
        cx.notify();
    }
    fn end_empty_cursor(&mut self, id: u64, cx: &mut Context<Self>) -> bool {
        if let Some(s) = self.take_cursor(id) {
            self.close_cursor_indicator(&s, cx);
            true
        } else {
            false
        }
    }
    fn deliver_cursor_once(
        &mut self,
        request: crate::dictation::DictationDeliveryRequest,
        timing: DictationDeliveryTiming,
        cx: &mut Context<Self>,
    ) {
        let app = cx.entity();
        cx.defer(move |cx| {
            let handle =
                crate::get_main_window_handle().or_else(|| match cx.windows().as_slice() {
                    [one] => Some(*one),
                    _ => None,
                });
            let result = handle.map(|h| {
                h.update(cx, |_, window, cx| {
                    app.update(cx, |app, cx| {
                        app.drain_cursor_controls(cx);
                        let Some(mut capture) = app.take_cursor(request.session_generation) else {
                            return;
                        };
                        let original = capture.route == CursorRoute::Original;
                        let take = capture.take.clone();
                        let save = capture.history_allowed && timing.record_history;
                        // The optional model has already returned. Only the final, local paste
                        // can see a screen spelling. The normal frozen-field checks still run
                        // immediately before paste, including on a late window/field switch.
                        let mut replacement = None;
                        if let (
                            true,
                            Some(session),
                            Some(window),
                            crate::dictation::FrozenDictationDestination::ExternalApp {
                                pid, ..
                            },
                        ) = (
                            original,
                            capture.spelling.take(),
                            request.selection.destination.external_window_id(),
                            &request.selection.destination,
                        ) {
                            let owner = crate::dictation::screen_spelling::Owner {
                                generation: request.session_generation,
                                pid: *pid,
                                window,
                            };
                            let (text, receipt) = session.finish(owner, request.transcript.text());
                            app.one_window_host.cursor_voice.spelling_result = Some(receipt);
                            replacement = text;
                        }
                        let words = replacement.as_deref().unwrap_or(request.transcript.text());
                        if let Some(t) = &take {
                            if t.dispatching_with_spelling(replacement.as_deref()).is_err() {
                                if let Some(r) =
                                    &mut app.one_window_host.cursor_voice.spelling_result
                                {
                                    r.status = "withheld";
                                    r.changes = 0;
                                }
                                app.one_window_host.cursor_voice.saved_draft =
                                    Some(request.transcript.text().into());
                                app.notice_failed(
                                    "Couldn't confirm the saved recording · nothing delivered",
                                    cx,
                                );
                                return;
                            }
                        }
                        let stop_started = capture.stop_started;
                        match app.finish_cursor_words(capture, &request, words, window, cx) {
                            Ok(target) => {
                                app.one_window_host.cursor_voice.stop_to_delivery_ms =
                                    stop_started.map(|t| t.elapsed().as_millis() as u64);
                                if save {
                                    if let Err(e) = app.keep_finished_cursor(
                                        take.as_ref(),
                                        words,
                                        target,
                                        timing.audio_duration,
                                    ) {
                                        app.notice_failed(
                                            format!("Couldn't save dictation history: {e}"),
                                            cx,
                                        );
                                    }
                                }
                                let selection = if original {
                                    Some(request.selection.clone())
                                } else {
                                    app.capture_dictation_target_selection(target, cx).ok()
                                };
                                if let Some(selection) = selection {
                                    let _ = crate::dictation::record_delivery_receipt(
                                        request.delivery_id,
                                        request.session_generation,
                                        &selection,
                                        words,
                                        timing.audio_duration,
                                        target,
                                        target.destination(),
                                        target != crate::dictation::DictationTarget::ExternalApp,
                                        &request.history_entry_id,
                                        None,
                                    );
                                }
                            }
                            Err(error) => {
                                let unknown = error.is::<CursorDispatchUnknown>();
                                if let Some(r) =
                                    &mut app.one_window_host.cursor_voice.spelling_result
                                {
                                    r.status = if unknown {
                                        "deliveryUnknown"
                                    } else {
                                        "withheld"
                                    };
                                    if !unknown {
                                        r.changes = 0;
                                    }
                                }
                                if !unknown {
                                    if let Some(t) = &take {
                                        let _ = t.withheld_with_original(
                                            replacement.as_ref().map(|_| request.transcript.text()),
                                        );
                                    }
                                }
                                app.one_window_host.cursor_voice.saved_id =
                                    take.as_ref().map(|t| t.id.clone());
                                app.one_window_host.cursor_voice.saved_draft = Some(
                                    if unknown {
                                        words
                                    } else {
                                        request.transcript.text()
                                    }
                                    .into(),
                                );
                                app.notice_failed(
                                    if unknown {
                                        "Delivery unknown · check the field before pasting again"
                                            .into()
                                    } else {
                                        format!("Dictation kept for Markdown review: {error}")
                                    },
                                    cx,
                                );
                            }
                        }
                        app.schedule_dictation_transcriber_cleanup(
                            cx,
                            std::time::Duration::from_secs(300),
                        );
                        cx.notify();
                    })
                })
            });
            if !matches!(result, Some(Ok(()))) {
                app.update(cx, |app, cx| {
                    let Some(capture) = app.take_cursor(request.session_generation) else {
                        return;
                    };
                    app.one_window_host.cursor_voice.saved_id = capture.take.map(|t| t.id);
                    app.one_window_host.cursor_voice.saved_draft =
                        Some(request.transcript.text().into());
                    cx.notify();
                });
            }
        });
    }
    fn review_cursor_draft(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(s) = self.one_window_host.cursor_voice.active.as_mut() {
            if s.stopped {
                return false;
            }
            s.spelling.take();
            s.route = CursorRoute::Markdown;
            cx.notify();
            return true;
        }
        if let Some(id) = self.one_window_host.cursor_voice.saved_id.clone() {
            if ProtectedTake::existing(&id).get().is_ok() {
                self.restore_saved_take(&id, cx);
                return true;
            }
            self.one_window_host.cursor_voice.saved_id = None;
            self.one_window_host.cursor_voice.saved_draft = None;
        }
        if self.one_window_host.cursor_voice.saved_draft.is_none() {
            return false;
        }
        let app = cx.entity();
        cx.defer(move |cx| {
            if let Some(handle) = crate::get_main_window_handle() {
                let _ = handle.update(cx, |_, win, cx| {
                    app.update(cx, |app, cx| {
                        if let Some(text) = app.one_window_host.cursor_voice.saved_draft.clone() {
                            if app.prepare_shortcut_launcher(cx)
                                && app.cursor_review(text, win, cx).is_ok()
                            {
                                app.one_window_host.cursor_voice.saved_draft = None;
                            }
                        }
                    })
                });
            }
        });
        true
    }
    fn cursor_review(
        &mut self,
        words: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        use crate::design_evaluation::prompt_fixtures::{
            EditorPromptSeed, PromptSeed, PromptSeedCommon,
        };
        let seed = PromptSeed::Editor(EditorPromptSeed {
            common: PromptSeedCommon::local("dictation-draft"),
            content: words,
            language: "markdown".into(),
            template: None,
        });
        self.construct_prompt_seed(seed, cx)?;
        self.one_window_host.cursor_voice.reviewing = None;
        if !self.main_services.host_policy().is_hidden() {
            self.dispatch_window_event(
                crate::window_orchestrator::WindowEvent::ShowMain { activate_app: true },
                cx,
            );
            window.activate_window();
        }
        Ok(())
    }
    fn finish_cursor_words(
        &mut self,
        s: CursorCapture,
        request: &crate::dictation::DictationDeliveryRequest,
        words: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<crate::dictation::DictationTarget> {
        use crate::dictation::DictationTarget as T;
        if s.route == CursorRoute::Markdown {
            anyhow::ensure!(
                self.prepare_shortcut_launcher(cx),
                "A prompt or unsaved note still owns the window"
            );
            self.cursor_review(words.into(), window, cx)?;
            return Ok(T::MainWindowPrompt);
        }
        if s.route == CursorRoute::Kit {
            anyhow::ensure!(
                self.prepare_shortcut_launcher(cx),
                "A prompt or unsaved note still owns the window"
            );
            let words = crate::components::text_input::normalize_single_line_text(words);
            self.one_window_host.cursor_voice.dictated_input = Some(words.clone());
            if !self.main_services.host_policy().is_hidden() {
                self.dispatch_window_event(
                    crate::window_orchestrator::WindowEvent::ShowMain { activate_app: true },
                    cx,
                );
            }
            self.set_filter_text_immediate(words, window, cx);
            self.focus_main_filter(window, cx);
            return Ok(T::MainWindowFilter);
        }
        self.close_cursor_indicator(&s, cx);
        match s.anchor {
            CursorAnchor::External(field) => {
                field
                    .ok_or_else(|| anyhow::anyhow!("Original field unavailable"))?
                    .paste(words, &request.selection)?;
                Ok(T::ExternalApp)
            }
            CursorAnchor::Input {
                text,
                range,
                surface,
            } => {
                let input = self.gpui_input_state.read(cx);
                anyhow::ensure!(
                    ((request.selection.target == T::MainWindowFilter
                        && matches!(self.current_view, AppView::ScriptList))
                        || (request.selection.target == T::MainWindowPrompt
                            && self.shared_cursor_prompt()))
                        && surface == self.owned_revision_facts().surface_generation
                        && input.value().as_ref() == text
                        && input.selected_range() == range
                        && (self.main_services.host_policy().is_hidden()
                            || crate::platform::is_main_window_focused()),
                    "Original Kit input changed"
                );
                let (next, cursor) =
                    sk_one_window::cursor_dictation::insert_at(&text, range, words)
                        .ok_or_else(|| anyhow::anyhow!("Original selection changed"))?;
                if request.selection.target == T::MainWindowPrompt {
                    anyhow::ensure!(
                        self.try_set_prompt_input(next, cx),
                        "Original prompt changed"
                    );
                    return Ok(T::MainWindowPrompt);
                }
                self.one_window_host.cursor_voice.dictated_input = Some(next.clone());
                self.set_filter_text_and_cursor_immediate(next, cursor, window, cx);
                Ok(T::MainWindowFilter)
            }
            CursorAnchor::Day(snapshot) => {
                anyhow::ensure!(
                    self.main_services.host_policy().is_hidden()
                        || crate::platform::is_main_window_focused(),
                    "The note no longer has keyboard focus"
                );
                let AppView::DayPage { entity } = &self.current_view else {
                    anyhow::bail!("Original note is no longer open");
                };
                entity
                    .clone()
                    .update(cx, |view, cx| {
                        view.inject_dictation_text_into_owned_snapshot(&snapshot, words, window, cx)
                    })
                    .map_err(anyhow::Error::msg)?;
                Ok(T::DayPageToday)
            }
        }
    }
}
