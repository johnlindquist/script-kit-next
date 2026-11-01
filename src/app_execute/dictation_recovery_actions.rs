use crate::dictation::recovery::TakeId as ProtectedTake;

impl ScriptListApp {
    pub(crate) fn history_paste_label(&self, id: &str) -> Option<&str> {
        self.one_window_host
            .cursor_voice
            .paste_anchor
            .as_ref()
            .filter(|(owner, _, _)| owner == id)
            .map(|(_, _, s)| s.display_label.as_str())
    }
    pub(crate) fn clear_forgotten_draft(&mut self, id: &str, cx: &mut Context<Self>) {
        let close = self.one_window_host.cursor_voice.reviewing.as_deref() == Some(id)
            && matches!(&self.current_view,AppView::EditorPrompt{id,..} if id=="dictation-draft");
        let uri = format!("kit://dictation-history?id={id}");
        self.one_window_host
            .work
            .parts
            .retain(|_, p| !matches!(p,crate::ai::AiContextPart::ResourceUri{uri:u,..} if u==&uri));
        let h = &mut self.one_window_host.cursor_voice;
        if h.saved_id.as_deref() == Some(id) {
            h.saved_draft = None;
            h.saved_id = None;
        }
        if h.reviewing.as_deref() == Some(id) {
            h.reviewing = None;
        }
        if close {
            self.reset_to_script_list(cx);
        }
        if self.one_window_own_view() == Some(crate::one_window::learns::OwnView::TakeActions) {
            self.open_recovery_list(cx);
        }
    }
    fn choose_dictation_default(&mut self, cx: &mut Context<Self>) {
        self.one_window_ask(
            "mic",
            "Start dictation where?",
            "Explicit Today and Dictate into Kit shortcuts stay unchanged.",
            vec![
                (
                    "At the cursor".into(),
                    Box::new(|app, cx| app.save_cursor_default(false, cx)),
                ),
                (
                    "In Script Kit".into(),
                    Box::new(|app, cx| app.save_cursor_default(true, cx)),
                ),
            ],
            None,
            cx,
        );
    }
    fn save_cursor_default(&mut self, kit: bool, cx: &mut Context<Self>) {
        let mut prefs = crate::config::load_user_preferences();
        prefs.dictation.start_in_kit = Some(kit);
        match crate::config::save_user_preferences(&prefs) {
            Ok(()) => {
                self.config.dictation = Some(prefs.dictation);
                self.one_window_notice(
                    crate::line_notice::DONE,
                    if kit {
                        "Dictation starts in Kit"
                    } else {
                        "Dictation starts at the cursor"
                    },
                    cx,
                );
            }
            Err(e) => self.notice_failed(format!("Couldn't save dictation preference: {e}"), cx),
        }
    }
    pub(crate) fn open_recovery_list(&mut self, cx: &mut Context<Self>) {
        use crate::one_window::learns::OwnView;
        let items = match crate::dictation::recovery::recent() {
            Ok(items) => items,
            Err(e) => {
                self.notice_failed(format!("Couldn't read private recordings: {e}"), cx);
                return;
            }
        };
        let active = self
            .one_window_host
            .cursor_voice
            .active
            .as_ref()
            .and_then(|s| s.take.as_ref())
            .map(|t| &t.id);
        self.one_window_host.learns.view_rows = items
            .into_iter()
            .filter(|i| Some(&i.id) != active && (i.draft.is_some() || i.has_audio))
            .map(|i| {
                let time = chrono::DateTime::from_timestamp(i.started, 0)
                    .map(|d| {
                        d.with_timezone(&chrono::Local)
                            .format("%b %d %H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_default();
                sk_one_window::Row::new(
                    format!("recover:{}", i.id),
                    format!("{} · {time}", i.state),
                )
                .with_sub(if i.draft.is_some() {
                    if i.state == "cancelled" && i.raw.is_none() {
                        "Captured words · full audio may also be available"
                    } else {
                        "Text available"
                    }
                } else {
                    "Audio available"
                })
            })
            .collect();
        self.one_window_host.learns.view_head = "Recently cancelled or interrupted".into();
        self.one_window_open_view(
            OwnView::Recovery,
            "Recently Cancelled".into(),
            Some(sk_one_window::EnterTarget::new(
                "recover",
                "Review recording",
            )),
            true,
            cx,
        );
    }
    pub(crate) fn open_take_actions(&mut self, id: &str, cx: &mut Context<Self>) {
        use crate::one_window::learns::OwnView;
        let t = ProtectedTake::existing(id);
        let item = t.get().ok();
        let history = crate::dictation::get_history_entry(id);
        if item.is_none() && history.is_none() {
            self.notice_failed("Recording expired or forgotten", cx);
            return;
        }
        let row =
            |verb: &str, title: &str| sk_one_window::Row::new(format!("take:{verb}:{id}"), title);
        let mut rows = Vec::new();
        if history.is_some() || item.as_ref().is_some_and(|i| i.draft.is_some()) {
            rows.push(row("restore", "Open editable draft"));
        }
        if let Some(i) = item.as_ref() {
            if i.has_audio {
                rows.push(row(
                    "retry",
                    if i.draft.is_some() {
                        "Regenerate from audio"
                    } else {
                        "Transcribe saved audio"
                    },
                ));
            }
            for version in t.versions().unwrap_or_default() {
                rows.push(row(
                    &format!("version-{version}"),
                    &format!("Review version {version}"),
                ));
                rows.push(row(
                    &format!("keep-{version}"),
                    &format!("Keep version {version} in History"),
                ));
            }
            if !i.has_audio {
                rows.push(row("no-audio", "No saved audio to retry"));
            }
            if i.draft.is_some() {
                rows.push(row("keep", "Keep draft in History"));
            }
        }
        rows.push(row("forget", "Forget this recording"));
        self.one_window_host.learns.view_rows = rows;
        self.one_window_host.learns.view_head = "Recording actions".into();
        self.one_window_host.cursor_voice.reviewing = Some(id.to_owned());
        if self.one_window_own_view() == Some(OwnView::TakeActions) {
            self.one_window_republish_own(cx);
        } else {
            self.one_window_open_view(
                OwnView::TakeActions,
                "Recording".into(),
                Some(sk_one_window::EnterTarget::new(
                    "recording-action",
                    "Choose action",
                )),
                true,
                cx,
            );
        }
    }
    pub(crate) fn run_take_action(&mut self, key: &str, cx: &mut Context<Self>) {
        let Some((verb, id)) = key.strip_prefix("take:").and_then(|k| k.split_once(':')) else {
            return;
        };
        if self.one_window_host.cursor_voice.reviewing.as_deref() != Some(id) {
            return;
        }
        match verb {
            "restore" => self.review_take_text(id, None, cx),
            "no-audio" => self.one_window_notice(
                crate::line_notice::INFO,
                "No saved audio for this recording",
                cx,
            ),
            v if v.starts_with("version-") => {
                if let Ok(seq) = v[8..].parse() {
                    self.review_take_text(id, Some(seq), cx);
                }
            }
            v if v.starts_with("keep-") => {
                if let Ok(seq) = v[5..].parse() {
                    self.keep_reviewed_take(id, Some(seq), cx);
                }
            }
            "retry" => self.transcribe_saved_take(id, cx),
            "keep" => self.keep_reviewed_take(id, None, cx),
            "forget" => {
                let id = id.to_owned();
                self.one_window_hold(
                    "trash",
                    "Forget this recording",
                    Box::new(move |app, cx| app.delete_dictation_history_entry_confirmed(&id, cx)),
                    cx,
                );
            }
            _ => {}
        }
    }
    fn keep_finished_cursor(
        &self,
        take: Option<&ProtectedTake>,
        words: &str,
        target: crate::dictation::DictationTarget,
        duration: std::time::Duration,
    ) -> anyhow::Result<()> {
        let mut entry = crate::dictation::build_history_entry(words, duration, target);
        if let Some(t) = take {
            let item = t.get()?;
            entry.id = t.id.clone();
            entry.raw_transcript = item.raw.filter(|s| s != words);
        }
        crate::dictation::record_entry(entry)?;
        if let Some(t) = take {
            t.finish()?;
        }
        Ok(())
    }
    fn announce_recovered_cancel(&mut self, t: &ProtectedTake, cx: &mut Context<Self>) {
        match t.get() {
            Ok(i) if i.has_audio || i.draft.is_some() => {
                let id = t.id.clone();
                self.one_window_done_undoable(
                    "mic",
                    "Recording cancelled · Restore in Kit",
                    Box::new(move |app, cx| app.restore_saved_take(&id, cx)),
                    cx,
                );
            }
            Ok(_) => self.one_window_notice(
                crate::line_notice::INFO,
                "Cancelled · No saved words or audio yet",
                cx,
            ),
            Err(e) => self.notice_failed(format!("Couldn't verify cancelled recording: {e}"), cx),
        }
    }
    pub(crate) fn restore_saved_take(&mut self, id: &str, cx: &mut Context<Self>) {
        let t = ProtectedTake::existing(id);
        match t.get() {
            Ok(item) if item.draft.is_some() => self.review_take_text(id, None, cx),
            Ok(_) => self.open_take_actions(id, cx),
            Err(_) if crate::dictation::get_history_entry(id).is_some() => {
                self.review_take_text(id, None, cx)
            }
            Err(_) => self.notice_failed("Recording expired or no longer available", cx),
        }
    }
    pub(crate) fn review_take_text(
        &mut self,
        id: &str,
        candidate: Option<i64>,
        cx: &mut Context<Self>,
    ) {
        let id = id.to_owned();
        let app = cx.entity();
        cx.defer(move|cx|{if let Some(w)=crate::get_main_window_handle().or_else(|| match cx.windows().as_slice(){[one]=>Some(*one),_=>None}){let _=w.update(cx,|_,win,cx|app.update(cx,|app,cx|{
            let t=ProtectedTake::existing(&id);let item=t.get().ok();
            let words=if let Some(seq)=candidate{t.version(seq).ok()}else{item.as_ref().and_then(|i|i.draft.clone()).or_else(||crate::dictation::get_history_entry(&id).map(|e|e.transcript))};
            let Some(words)=words else{app.notice_failed("No saved transcript to restore",cx);return;};
            if !app.prepare_shortcut_launcher(cx){app.notice_failed("Finish the open prompt before restoring a recording",cx);return;}
            if let Err(e)=app.cursor_review(words.clone(),win,cx){app.notice_failed(format!("Couldn't open draft: {e}"),cx);return;}
            app.one_window_host.cursor_voice.reviewing=Some(id.clone());
            if let Some(item)=item {if let AppView::EditorPrompt{entity,..}=&app.current_view {
                let entity=entity.clone();let mut previous=words;let mut revision=item.rev;
                let observed=entity.entity_id();cx.observe(&entity,move|app,editor,cx|{
                    if !matches!(&app.current_view, AppView::EditorPrompt{entity,..} if entity.entity_id()==observed) {return;}
                    let words=editor.read(cx).content_from_app(cx);if words==previous{return;}
                    // A regenerated candidate never replaces the user's previous draft.
                    let result=if let Some(seq)=candidate{t.edit_version(seq,&previous,&words)}else{t.edit(revision,&words).map(|r|revision=r)};
                    if result.is_ok(){previous=words;}else{app.notice_failed("Couldn't save edits · recording changed or expired",cx);}
                }).detach();}
                if let Some(until)=item.expires {if let Some(at)=chrono::DateTime::from_timestamp(until,0){app.one_window_notice(crate::line_notice::INFO,format!("Private draft until {} · Keep in History to retain",at.with_timezone(&chrono::Local).format("%b %d %H:%M")),cx);}}
            }
        }));}});
    }
    pub(crate) fn transcribe_saved_take(&mut self, id: &str, cx: &mut Context<Self>) {
        let t = ProtectedTake::existing(id);
        let id = t.id.clone();
        if t.get().is_err() {
            self.notice_failed("No recording to retry", cx);
            return;
        }
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let chunks = t.chunks()?;
                    let words = crate::dictation::transcribe_captured_audio(&chunks)?
                        .ok_or_else(|| anyhow::anyhow!("No words recognized"))?;
                    t.add_candidate(&words)?;
                    anyhow::Ok(())
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                match result {
                    Ok(()) => {
                        // Don't steal focus from another editor or overwrite its words on async completion.
                        app.one_window_notice(
                            crate::line_notice::INFO,
                            "Transcription ready · Review new version",
                            cx,
                        );
                        if app.one_window_own_view()
                            == Some(crate::one_window::learns::OwnView::TakeActions)
                            && app.one_window_host.cursor_voice.reviewing.as_ref() == Some(&id)
                        {
                            app.open_take_actions(&id, cx);
                        }
                    }
                    Err(e) => app.notice_failed(format!("Couldn't transcribe recording: {e}"), cx),
                }
            });
        })
        .detach();
    }
    pub(crate) fn keep_reviewed_take(
        &mut self,
        id: &str,
        candidate: Option<i64>,
        cx: &mut Context<Self>,
    ) {
        let t = ProtectedTake::existing(id);
        let result = (|| -> anyhow::Result<()> {
            let item = t.get()?;
            let words = if let Some(seq) = candidate {
                t.version(seq)?
            } else {
                item.draft
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("Transcribe first"))?
            };
            let mut entry = crate::dictation::get_history_entry(id).unwrap_or_else(|| {
                crate::dictation::build_history_entry(
                    &words,
                    std::time::Duration::ZERO,
                    crate::dictation::DictationTarget::MainWindowFilter,
                )
            });
            entry.id = id.to_owned();
            entry.raw_transcript = item.raw.or(entry.raw_transcript);
            entry.transcript = words.clone();
            entry.preview = words
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(120)
                .collect();
            if candidate.is_some() {
                if let Some(previous) = &item.draft {
                    t.add_candidate(previous)?;
                }
                t.edit(item.rev, &words)?;
            }
            crate::dictation::keep_reviewed(&entry)?;
            t.finish()?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.one_window_notice(crate::line_notice::DONE, "Kept in Dictation History", cx)
            }
            Err(e) => self.notice_failed(format!("Couldn't keep the draft: {e}"), cx),
        }
    }
    pub(crate) fn bind_history_paste(
        &mut self,
        id: &str,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        self.one_window_host.cursor_voice.paste_anchor = None;
        #[cfg(target_os = "macos")]
        {
            let selection = self
                .capture_dictation_target_selection(
                    crate::dictation::DictationTarget::ExternalApp,
                    cx,
                )
                .ok()?;
            let crate::dictation::FrozenDictationDestination::ExternalApp { pid, .. } =
                selection.destination
            else {
                return None;
            };
            let field = CursorField::capture_pid(pid)?;
            let label = selection.display_label.clone();
            self.one_window_host.cursor_voice.paste_anchor = Some((id.into(), field, selection));
            Some(label)
        }
        #[cfg(not(target_os = "macos"))]
        None
    }
    pub(crate) fn paste_saved_take(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some((owner, field, selection)) = self.one_window_host.cursor_voice.paste_anchor.take()
        else {
            self.notice_failed(
                "Choose a paste destination from this recording's actions",
                cx,
            );
            return;
        };
        if owner != id {
            self.notice_failed("Paste destination changed; nothing pasted", cx);
            return;
        }
        let Some(entry) = crate::dictation::get_history_entry(id) else {
            self.notice_failed("Recording unavailable", cx);
            return;
        };
        self.hide_main_and_reset(cx);
        cx.spawn(async move |app, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(200))
                .await;
            let _ = app.update(cx, |app, cx| {
                if let Err(e) = field.paste(&entry.transcript, &selection) {
                    app.notice_failed(format!("Paste not confirmed · recording kept: {e}"), cx);
                }
            });
        })
        .detach();
    }
}
