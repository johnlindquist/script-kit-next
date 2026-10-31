// Direct dictation owns a frozen cursor, not a Jev Talk reading. Partial audio is never
// inserted, sent to the intent reader, or used to choose an action.
use sk_one_window::cursor_dictation::{Route as CursorRoute, ShiftTap};

#[derive(Default)]
pub(crate) struct CursorHost {
    active: Option<CursorCapture>,
    retired: Option<u64>,
    saved_draft: Option<String>,
    pub(crate) dictated_input: Option<String>,
}
struct CursorCapture {
    generation: u64,
    app: gpui::WeakEntity<ScriptListApp>,
    held: bool,
    stopped: bool,
    revealed: bool,
    route: CursorRoute,
    label: String,
    shift: ShiftTap,
    partial: String,
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
struct CursorField {
    #[cfg(target_os = "macos")]
    pid: i32,
    #[cfg(target_os = "macos")]
    element: crate::platform::accessibility::ax::OwnedAxElement,
    #[cfg(target_os = "macos")]
    range: Option<crate::platform::accessibility::TextRangeUtf16>,
}
impl CursorField {
    #[cfg(target_os = "macos")]
    fn capture() -> Option<Self> {
        use crate::platform::accessibility::{app_identity, ax};
        crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)
            .ok()?;
        let pid = app_identity::current_frontmost_app_identity().process_id?;
        if pid == std::process::id() as i32 {
            return None;
        }
        let element = ax::focused_ui_element_for_pid(pid).ok()?;
        let ptr = element.as_ptr();
        if ax::subrole(ptr).as_deref() == Some("AXSecureTextField") {
            return None;
        }
        let range = ax::selected_text_range(ptr);
        if range.is_none()
            && !matches!(
                ax::role(ptr).as_deref(),
                Some("AXTextField" | "AXTextArea" | "AXComboBox")
            )
        {
            return None;
        }
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
        let now = Self::capture()
            .ok_or_else(|| anyhow::anyhow!("The original field is no longer focused"))?;
        anyhow::ensure!(
            self.pid == now.pid
                && self.element.same_element(&now.element)
                && self.range == now.range
                && !crate::platform::is_main_window_focused(),
            "The original field or selection changed"
        );
        let result = crate::text_injector::TextInjector::new().paste_text_in(
            crate::platform::synthetic_keys::KeyTarget::Process(self.pid),
            words,
        )?;
        anyhow::ensure!(
            result.landed(),
            "Paste was not confirmed: {}",
            result.code()
        );
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    fn paste(self, _: &str, _: &crate::dictation::DictationTargetSelection) -> anyhow::Result<()> {
        anyhow::bail!("External field identity unavailable")
    }
}

// Sample modifier flags only during our recording. No global keyboard hook, key contents,
// or additional Input Monitoring permission. KeyDown event type = 10, combined session = 0.
fn cursor_shift_sample() -> (bool, u32) {
    #[cfg(target_os = "macos")]
    {
        if crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)
            .is_ok()
        {
            #[link(name = "CoreGraphics", kind = "framework")]
            extern "C" {
                fn CGEventSourceFlagsState(state: i32) -> u64;
                fn CGEventSourceCounterForEventType(state: i32, event: u32) -> u32;
            }
            return unsafe {
                (
                    CGEventSourceFlagsState(0) & (1 << 17) != 0,
                    CGEventSourceCounterForEventType(0, 10),
                )
            };
        }
    }
    (false, 0)
}
impl ScriptListApp {
    pub(crate) fn cursor_voice_state(&self) -> serde_json::Value {
        let host = &self.one_window_host.cursor_voice;
        serde_json::json!({"active":host.active.is_some(),"generation":host.active.as_ref().map(|s|s.generation),"route":host.active.as_ref().map(|s|match s.route{CursorRoute::Original=>"original",CursorRoute::Kit=>"kit",CursorRoute::Markdown=>"markdown"}),"stopping":host.active.as_ref().is_some_and(|s|s.stopped),"draftAvailable":host.saved_draft.is_some()})
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
        if crate::dictation::current_capture_generation() == Some(id) {
            self.request_active_dictation_stop(
                crate::dictation::DictationStopReason::Hotkey,
                false,
                cx,
            );
        }
        self.end_empty_cursor(id, cx)
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
        let target = if external {
            T::ExternalApp
        } else if matches!(self.current_view, AppView::DayPage { .. }) {
            T::DayPageToday
        } else if !today && self.shared_cursor_prompt() {
            T::MainWindowPrompt
        } else if !today && matches!(self.current_view, AppView::ScriptList) {
            T::MainWindowFilter
        } else {
            self.notice_failed("This field doesn't accept shortcut dictation", cx);
            return;
        };
        let action = DictationBuiltinAction::CurrentSurface;
        if self.prepare_dictation_builtin_start(action, Some(target), cx)
            != DictationStartPreflight::Ready
        {
            // Download/setup is a separate action. It must not replay a released hold.
            *pending_dictation_model_action().lock() = None;
            return;
        }
        self.start_cursor_recording(target, true, cx);
    }
    fn start_cursor_recording(
        &mut self,
        target: crate::dictation::DictationTarget,
        held: bool,
        cx: &mut Context<Self>,
    ) {
        self.begin_cursor_recording(target, held, None, cx);
    }
    #[cfg(feature = "owned-ui-evaluation")]
    pub(crate) fn cursor_fixture_begin(&mut self, cx: &mut Context<Self>) -> anyhow::Result<()> {
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
        self.begin_cursor_recording(target, true, Some(generation), cx);
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
            CursorAnchor::Input { .. } => "Script Kit".into(),
            CursorAnchor::Day(_) => {
                if let AppView::DayPage { entity } = &self.current_view {
                    entity.read(cx).document_label()
                } else {
                    "Today".into()
                }
            }
        };
        crate::dictation::clear_dictation_return_origin();
        let generation = if let Some(id) = fixture {
            if !crate::runtime_policy::is_owned_evaluation() {
                return;
            }
            id
        } else {
            match crate::dictation::toggle_dictation(target) {
                Ok(crate::dictation::DictationToggleOutcome::Started) => {}
                Ok(_) => return,
                Err(e) => {
                    self.notice_failed(format!("Couldn't start dictation: {e}"), cx);
                    return;
                }
            }
            let Some(generation) = crate::dictation::current_capture_generation() else {
                return;
            };
            crate::dictation::set_dictation_session_selection(selection.clone());
            generation
        };
        let mut shift = ShiftTap::default();
        let (d, k) = cursor_shift_sample();
        shift.sample(d, k);
        let revealed = target == T::ExternalApp && !script_kit_gpui::is_main_window_visible();
        self.one_window_host.cursor_voice.active = Some(CursorCapture {
            generation,
            app: cx.weak_entity(),
            held,
            stopped: false,
            revealed,
            route: CursorRoute::Original,
            label,
            shift,
            partial: String::new(),
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
                    if !app
                        .one_window_host
                        .cursor_voice
                        .active
                        .as_ref()
                        .is_some_and(|s| s.generation == generation && !s.stopped)
                        || crate::dictation::current_capture_generation() != Some(generation)
                    {
                        return false;
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
                    app.poll_cursor_shift(cx);
                    if crate::dictation::dictation_auto_stop_due() {
                        app.stop_cursor_capture(cx);
                        return false;
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
    fn poll_cursor_shift(&mut self, cx: &mut Context<Self>) {
        let (d, k) = cursor_shift_sample();
        if let Some(s) = self.one_window_host.cursor_voice.active.as_mut() {
            if s.shift.sample(d, k) {
                self.toggle_cursor_kit(cx);
            }
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
        s.route.toggle_kit();
        cx.notify();
        true
    }
    fn stop_cursor_capture(&mut self, cx: &mut Context<Self>) {
        self.poll_cursor_shift(cx);
        let Some(s) = self.one_window_host.cursor_voice.active.as_mut() else {
            return;
        };
        if s.stopped || crate::dictation::current_capture_generation() != Some(s.generation) {
            return;
        }
        s.stopped = true;
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
                        let Some(capture) = app.take_cursor(request.session_generation) else {
                            return;
                        };
                        let original = capture.route == CursorRoute::Original;
                        match app.finish_cursor_words(capture, &request, window, cx) {
                            Ok(target) => {
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
                                        request.transcript.text(),
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
                                app.one_window_host.cursor_voice.saved_draft =
                                    Some(request.transcript.text().into());
                                app.notice_failed(
                                    format!("Dictation kept for Markdown review: {error}"),
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
                    app.take_cursor(request.session_generation);
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
            s.route = CursorRoute::Markdown;
            cx.notify();
            return true;
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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<crate::dictation::DictationTarget> {
        use crate::dictation::DictationTarget as T;
        let words = request.transcript.text();
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
            self.one_window_host.cursor_voice.dictated_input = Some(words.into());
            if !self.main_services.host_policy().is_hidden() {
                self.dispatch_window_event(
                    crate::window_orchestrator::WindowEvent::ShowMain { activate_app: true },
                    cx,
                );
            }
            self.set_filter_text_immediate(words.into(), window, cx);
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
