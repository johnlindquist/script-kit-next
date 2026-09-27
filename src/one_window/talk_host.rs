//! Holding the voice key in any app: the main window fades in over the front app without taking
//! its cursor, the heard words fill the input, and Jev reads whether they are writing or a
//! request. The reducer decides what letting go does; this listens, asks Jev, and pastes.

use std::sync::Arc;
use std::time::Duration;

use gpui::Context;
use sk_one_window::{Event, Row, TalkEvent};

use crate::intent::jev::{Answer, Criteria, Question, Questions};
use crate::intent::{JevClient, JevConfig, JevRow, Oracle, OracleSlot};
use crate::ScriptListApp;

/// How often the heard words are read from the transcriber while talking.
const HEARD_POLL: Duration = Duration::from_millis(16);
/// The front app takes its cursor back before the words are pasted.
const PASTE_SETTLE: Duration = Duration::from_millis(200);
/// Bumped whenever the words or the questions change, so a remembered answer is for them.
const TALK_TAG: &str = "|talk:v1";

#[derive(Default)]
pub(crate) struct TalkHost {
    /// Bumped per session, so a pump from an older one stops.
    generation: u64,
    /// The microphone is on for this session (off in the owned evaluator).
    listening: bool,
}

fn talk_questions() -> Questions {
    let mut criteria = Criteria::default();
    criteria.set(
        "type",
        Some(
            "Words written to a reader: part of the reply or message in the front app, meant to \
             appear as text at the cursor."
                .to_string(),
        ),
    );
    criteria.set(
        "ask",
        Some(
            "A request to the computer to do something for the user (remind, send, open, find, \
             schedule, move, rewrite…)."
                .to_string(),
        ),
    );
    let mut questions = Questions::default();
    questions.insert(
        "mode",
        Question::Choice {
            instructions: "The user is holding the voice key and talking while their text cursor \
                           is in the front app. Decide who the words are addressed to: a reader \
                           of what they are writing, or the computer. Judge the words heard so \
                           far; the sentence may continue."
                .to_string(),
            criteria,
        },
    );
    questions
}

fn talk_state(words: &str, app: &str, still_talking: bool) -> serde_json::Value {
    serde_json::json!({
        "request": format!("{words}{TALK_TAG}"),
        "screen": { "frontmost_app": app },
        "heard_so_far": words,
        "words_heard": sk_one_window::talk::word_count(words),
        "still_talking": still_talking,
    })
}

impl ScriptListApp {
    /// A voice session starts in the main window over `app`.
    pub(crate) fn one_window_talk_began(&mut self, listening: bool, cx: &mut Context<Self>) {
        let app = crate::ui_window::footer_frontmost_app_name()
            .unwrap_or_else(|| "the front app".to_string());
        if !self.main_services.host_policy().is_hidden() {
            crate::platform::show_main_window_without_activation();
        }
        let talk = &mut self.one_window_host.learns.talk;
        talk.generation += 1;
        talk.listening = listening;
        let generation = talk.generation;
        self.one_window_handle(
            Event::Talk(TalkEvent::Start {
                app,
                field: String::new(),
                to: None,
            }),
            cx,
        );
        self.one_window_republish_own(cx);
        if listening {
            self.one_window_talk_pump(generation, cx);
        }
    }

    /// Reads what the transcriber has heard while the microphone is on.
    fn one_window_talk_pump(&mut self, generation: u64, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let mut heard = String::new();
            loop {
                cx.background_executor().timer(HEARD_POLL).await;
                let Some(state) = crate::dictation::snapshot_overlay_state() else {
                    break;
                };
                let now = state.transcript.to_string();
                let current = this
                    .update(cx, |app, cx| {
                        let current = app.one_window_host.learns.talk.generation == generation
                            && app.one_window.talk().is_some();
                        if current && now != heard {
                            app.one_window_handle(Event::Talk(TalkEvent::Heard(now.clone())), cx);
                            app.one_window_republish_own(cx);
                        }
                        current
                    })
                    .unwrap_or(false);
                if !current {
                    break;
                }
                heard = now;
            }
        })
        .detach();
    }

    /// The final words came back from the transcriber after the voice key came up.
    pub(crate) fn one_window_talk_final(&mut self, transcript: String, cx: &mut Context<Self>) {
        self.one_window_host.learns.talk.listening = false;
        self.one_window_handle(Event::Talk(TalkEvent::Heard(transcript)), cx);
        self.one_window_handle(Event::Talk(TalkEvent::Release), cx);
        self.one_window_republish_own(cx);
    }

    /// Whether the main window's voice session owns the dictation that just finished.
    pub(crate) fn one_window_talking(&self) -> bool {
        self.one_window.talk().is_some()
    }

    /// Asks Jev whether `words` are writing or a request; for the final words, also what the
    /// request would do.
    pub(crate) fn one_window_talk_ask(
        &mut self,
        words: String,
        last: bool,
        cx: &mut Context<Self>,
    ) {
        if last && self.one_window_host.learns.talk.listening {
            self.one_window_talk_stop_listening(cx);
            return;
        }
        self.settle_intent_oracle_slot();
        let slot = self.intent_launcher.oracle_slot().clone();
        if matches!(slot, OracleSlot::Off(_)) {
            if last {
                self.one_window_handle(Event::Talk(TalkEvent::Unread { words }), cx);
                self.one_window_republish_own(cx);
            }
            return;
        }
        let app = self
            .one_window
            .talk()
            .map(|talk| talk.app.clone())
            .unwrap_or_default();
        let state = talk_state(&words, &app, !last);
        cx.spawn(async move |this, cx| {
            let answer = cx
                .background_executor()
                .spawn(async move {
                    let oracle: Arc<dyn Oracle> = match slot {
                        OracleSlot::Ready(oracle) => oracle,
                        OracleSlot::Unloaded => Arc::new(JevClient::new(JevConfig::load()?)),
                        OracleSlot::Off(reason) => {
                            return Err(crate::intent::JevError::Refused(reason));
                        }
                    };
                    oracle.ask(&state, &talk_questions())
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                let event = match answer
                    .ok()
                    .and_then(|result| result.answers.get("mode").cloned())
                {
                    Some(Answer::Choice { probabilities, .. }) => TalkEvent::Read {
                        type_p: probabilities.get("type").unwrap_or(0.0) as f32,
                        ask_p: probabilities.get("ask").unwrap_or(0.0) as f32,
                        words: words.clone(),
                        ask: None,
                    },
                    _ if last => TalkEvent::Unread {
                        words: words.clone(),
                    },
                    _ => return,
                };
                app.one_window_handle(Event::Talk(event), cx);
                app.one_window_republish_own(cx);
                if last {
                    app.one_window_talk_read_request(words, cx);
                }
            });
        })
        .detach();
    }

    /// Jev's ordinary reading of the final words is what the request would run.
    fn one_window_talk_read_request(&mut self, words: String, cx: &mut Context<Self>) {
        self.one_window_host.learns.talk_request = Some(words.clone());
        self.ask_jev(0, words, self.intent_screen_key(), cx);
    }

    /// Jev read `text`; while the voice view waits on those words, its top reading becomes the
    /// request row.
    pub(crate) fn one_window_talk_reading(
        &mut self,
        text: &str,
        readings: &[JevRow],
        cx: &mut Context<Self>,
    ) {
        if self.one_window_host.learns.talk_request.as_deref() != Some(text) {
            return;
        }
        self.one_window_host.learns.talk_request = None;
        let Some(top) = readings.first() else {
            return;
        };
        let row = Row::new(top.key.clone(), top.plan.title.clone())
            .with_sub(crate::intent::plan_subtitle(&top.plan));
        let answer = self.one_window.talk().and_then(|talk| talk.answer.clone());
        if let Some(answer) = answer.filter(|a| a.words == text) {
            self.one_window_handle(
                Event::Talk(TalkEvent::Read {
                    words: answer.words,
                    type_p: answer.type_p,
                    ask_p: answer.ask_p,
                    ask: Some(row),
                }),
                cx,
            );
            self.one_window_republish_own(cx);
        }
    }

    /// Letting go with Enter while the microphone is on: stop and transcribe; the final words
    /// come back through [`Self::one_window_talk_final`].
    fn one_window_talk_stop_listening(&mut self, cx: &mut Context<Self>) {
        let target = crate::dictation::get_dictation_target()
            .unwrap_or(crate::dictation::DictationTarget::ExternalApp);
        self.request_active_dictation_stop(
            crate::dictation::DictationStopReason::Hotkey,
            target,
            true,
            cx,
        );
    }

    /// The voice session is over: whatever is still listening stops, and nothing it heard is
    /// delivered anywhere else.
    pub(crate) fn one_window_talk_ended(&mut self) {
        let talk = &mut self.one_window_host.learns.talk;
        talk.generation += 1;
        if std::mem::take(&mut talk.listening) {
            if let Err(error) = crate::dictation::abort_dictation() {
                tracing::warn!(target: "script_kit::one_window", %error, "talk_abort_failed");
            }
        }
    }

    /// Pastes the words at the front app's cursor once the window is out of its way.
    pub(crate) fn one_window_paste(&mut self, text: String, app: String, cx: &mut Context<Self>) {
        tracing::info!(target: "script_kit::one_window", event = "paste", app = %app, chars = text.chars().count());
        cx.background_executor()
            .spawn(async move {
                std::thread::sleep(PASTE_SETTLE);
                if let Err(error) = crate::text_injector::TextInjector::new().paste_text(&text) {
                    tracing::warn!(target: "script_kit::one_window", %error, "paste_failed");
                }
            })
            .detach();
    }

    /// Starts a voice session from the main window: the same session the voice key starts.
    pub(crate) fn one_window_talk_from_list(&mut self, cx: &mut Context<Self>) {
        if crate::runtime_policy::is_owned_evaluation() {
            self.one_window_talk_began(false, cx);
        } else {
            self.start_dictation_session(cx);
        }
    }
}
