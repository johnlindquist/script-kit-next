//! Holding the voice key: the voice view, Jev's two readings, and what letting go does.

use super::*;
use crate::contract::talk_words;
use crate::talk::{Answer, Decision, Phase, Reading, Talk, TalkEvent};

impl OneWindow {
    /// The voice session, while the voice key is held or the window asks which reading.
    pub fn talk(&self) -> Option<&Talk> {
        self.talk.as_ref()
    }

    pub(super) fn on_talk(&mut self, now: Ms, ev: TalkEvent, fx: &mut Vec<Effect>) -> bool {
        match ev {
            TalkEvent::Start { app, field, to } => {
                if self.talk.is_some() {
                    return false;
                }
                self.away = false;
                self.staged = None;
                self.talks += 1;
                let talk = Talk::new(self.talks, app, field, to);
                let enter = talk.enter();
                self.talk = Some(talk);
                self.on_view(
                    now,
                    ViewOp::Open(ViewSpec {
                        name: "voice".into(),
                        label: talk_words::VIEW_LABEL.into(),
                        enter: Some(enter),
                        takes_input: true,
                        text: Some(String::new()),
                        leave_text: Some(String::new()),
                    }),
                    fx,
                );
            }
            TalkEvent::Heard(text) => {
                if self.talk.is_none() {
                    return false;
                }
                if text != self.text {
                    fx.push(Effect::SetInput(text.clone()));
                }
                self.talk_heard(now, text, fx);
            }
            TalkEvent::Read {
                words,
                type_p,
                ask_p,
                ask,
            } => {
                let Some(talk) = self.talk.as_mut() else {
                    return false;
                };
                if !talk.answered(
                    Answer {
                        words,
                        type_p,
                        ask_p,
                    },
                    ask,
                ) {
                    return false;
                }
            }
            TalkEvent::Unread { words } => {
                let Some(talk) = self.talk.as_mut() else {
                    return false;
                };
                talk.unread(&words);
            }
            TalkEvent::Release => return self.talk_release(now, fx),
        }
        self.talk_sync(now);
        self.talk_decide(now, fx);
        true
    }

    /// Words heard, whether the host sent them or they were typed into the voice view.
    pub(super) fn talk_heard(&mut self, now: Ms, text: String, fx: &mut Vec<Effect>) {
        let Some(talk) = self.talk.as_mut().filter(|t| t.phase != Phase::Asking) else {
            return;
        };
        self.text = text.clone();
        if let Some((words, last)) = talk.hear(&text) {
            fx.push(Effect::TalkAsk { words, last });
        }
        self.talk_sync(now);
        self.talk_decide(now, fx);
    }

    /// Keys while the voice view is open: Enter lets go, ⌘1/⌘2 pick, Esc drops everything.
    pub(super) fn talk_key(&mut self, now: Ms, key: Key, fx: &mut Vec<Effect>) -> bool {
        match key {
            Key::Enter => {
                self.talk_release(now, fx);
            }
            Key::CmdDigit(d @ (1 | 2)) => {
                let reading = if d == 1 { Reading::Type } else { Reading::Ask };
                let acts = self.talk.as_mut().and_then(|t| t.pick(reading));
                match acts {
                    Some(reading) => self.talk_act(now, reading, fx),
                    None => {
                        self.talk_sync(now);
                        self.talk_decide(now, fx);
                    }
                }
            }
            Key::Escape => self.talk_drop(now, fx),
            _ => self.talk_decide(now, fx),
        }
        true
    }

    fn talk_release(&mut self, now: Ms, fx: &mut Vec<Effect>) -> bool {
        let Some(talk) = self.talk.as_mut() else {
            return false;
        };
        if let Some(words) = talk.release() {
            fx.push(Effect::TalkAsk { words, last: true });
        }
        self.talk_sync(now);
        self.talk_decide(now, fx);
        true
    }

    /// The ↵ button and the line above the input follow the reading. A reading that changed
    /// without a key arms the Enter guard, and letting go waits it out.
    fn talk_sync(&mut self, now: Ms) {
        let Some(talk) = self.talk.as_ref() else {
            return;
        };
        let enter = talk.enter();
        let line = talk.line();
        let user = self.user_turn(now);
        if let Some(top) = self.views.top_mut() {
            top.enter = Some(enter.clone());
        }
        self.guard.observe_view(now, &enter.id, user);
        self.line_from_view = line.is_some();
        self.line = line;
    }

    /// Acts once Jev has read the final words and nothing changed under the key just now.
    pub(super) fn talk_decide(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        if self.guard.armed(now) {
            return;
        }
        let Some(decision) = self.talk.as_ref().and_then(Talk::decision) else {
            return;
        };
        match decision {
            Decision::Act(reading) => self.talk_act(now, reading, fx),
            Decision::Drop => self.talk_drop(now, fx),
            Decision::AskOnce => {
                if let Some(talk) = self.talk.as_mut() {
                    talk.ask_once();
                }
                self.talk_sync(now);
            }
        }
    }

    fn talk_end(&mut self) -> Option<Talk> {
        let talk = self.talk.take()?;
        self.line = None;
        self.line_from_view = false;
        Some(talk)
    }

    /// Typing pastes at the front app's cursor and the window fades out; ↑ takes it back. A
    /// request becomes the sentence in the input and runs Jev's reading of it.
    fn talk_act(&mut self, now: Ms, reading: Reading, fx: &mut Vec<Effect>) {
        let Some(talk) = self.talk_end() else {
            return;
        };
        fx.push(Effect::TalkEnded);
        match reading {
            Reading::Type => {
                self.ledger.record(
                    now,
                    NewEntry {
                        icon: "type".into(),
                        title: talk.typed_title(),
                        said: talk.heard.clone(),
                        risk: Risk::Undo,
                        app: Some(talk.app.clone()),
                        ..NewEntry::default()
                    },
                );
                fx.push(Effect::Paste {
                    text: talk.heard,
                    app: talk.app.clone(),
                });
                self.away = true;
                fx.push(Effect::Hide {
                    app: Some(talk.app),
                });
                self.home(now, String::new(), fx);
            }
            Reading::Ask => {
                self.home(now, talk.heard, fx);
                if let Some(row) = talk.ask_row {
                    self.run(now, row, RunHow::Slot, fx);
                }
            }
        }
    }

    /// Nothing is pasted and nothing runs; the window fades out as it is.
    fn talk_drop(&mut self, now: Ms, fx: &mut Vec<Effect>) {
        let Some(talk) = self.talk_end() else {
            return;
        };
        fx.push(Effect::TalkEnded);
        self.away = true;
        fx.push(Effect::Hide {
            app: Some(talk.app),
        });
        self.home(now, String::new(), fx);
    }
}
