//! What typing does to the words, the list and Jev's debounce.

use super::*;

impl OneWindow {
    pub(super) fn on_text(&mut self, now: Ms, text: String, typed: bool, fx: &mut Vec<Effect>) {
        if typed && self.talk.is_some() {
            return self.talk_heard(now, text, fx);
        }
        if typed {
            if let Some(top) = self.views.top() {
                if top.takes_input {
                    self.text = text;
                    self.work_text(now, fx);
                    return;
                }
                self.views.home();
                self.guard.forget_view();
                self.cause = Some((ContentCause::Home, now));
            }
        }
        if !typed && self.list.text == text {
            self.text = text;
            self.refresh_rows();
            return;
        }
        self.text = text;
        self.chose = false;
        self.chip_at = None;
        if self.text.trim().is_empty() {
            self.agent_owns = false;
        }
        self.host_names.clear();
        self.list.on_text(&self.text);
        self.list.steady.on_text(now, &self.text, typed);
        self.refresh_rows();
        self.jev_due =
            (!self.text.trim().is_empty()).then(|| (now + JEV_DEBOUNCE_MS, self.text.clone()));
        let _ = fx;
    }

    /// Sets the input's words the way the window does after a view closes: the list follows, but
    /// it isn't typing.
    pub(super) fn set_input(&mut self, now: Ms, text: String, fx: &mut Vec<Effect>) {
        fx.push(Effect::SetInput(text.clone()));
        self.on_text(now, text, false, fx);
    }

    pub(super) fn refresh_rows(&mut self) {
        let empty = self.text.trim().is_empty();
        self.list.owned = self.agent_owns && !empty;
        if self.list.owned {
            self.list.pinned = vec![crate::strip::owner_row(&self.text)];
            self.list.names.clear();
            self.list.move_sel(0);
            return;
        }
        let mut pins = self.host_pinned.clone();
        if let Some(staged) = &self.staged {
            if empty {
                pins.extend(staged.pinned());
            }
        }
        if empty && pins.is_empty() && self.staged.is_none() {
            pins = self
                .suggested
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let mut r = r.clone();
                    if i == 0 && r.head.is_none() {
                        r.head = Some(words::SUGGESTED.into());
                    }
                    r
                })
                .collect();
        }
        self.list.pinned = pins;

        // With something staged, what you said before comes first, then what can be done to it,
        // then anything else the words name. Jev's readings keep their own places.
        let (fallbacks, real): (Vec<Row>, Vec<Row>) =
            self.host_names.iter().cloned().partition(|r| r.fallback);
        let (all, mut real): (Vec<Row>, Vec<Row>) = real
            .into_iter()
            .partition(|r| r.key.starts_with(crate::contract::ALL_RESULTS_KEY));
        let mut names: Vec<Row> = Vec::new();
        if let Some(staged) = &self.staged {
            let (learned, rest): (Vec<Row>, Vec<Row>) = real.into_iter().partition(|r| r.learned);
            names = learned;
            real = rest;
            for r in staged.matches(&self.text) {
                if !names.iter().any(|n| n.title.eq_ignore_ascii_case(&r.title)) {
                    names.push(r);
                }
            }
        }
        for r in real {
            if !names.iter().any(|n| n.title.eq_ignore_ascii_case(&r.title)) {
                names.push(r);
            }
        }
        names.truncate(MAX_MATCHES);
        let down = self.list.jev.is_down_for(&self.text);
        let offered = fallbacks
            .into_iter()
            .filter(|r| !(down && r.key.starts_with("fallback-ai:")));
        if names.is_empty() {
            if !empty {
                names = offered.collect();
            }
        } else if self.staged.is_none() {
            // Every result stays one row away, and the words can still go elsewhere: the rest of
            // the search under the names (never owning ↵), then the fallbacks after Jev's rows.
            names.extend(all.into_iter().take(1).map(|r| Row { score: 0, ..r }));
            let mut offered: Vec<Row> = offered.take(MAX_FALLBACKS_WITH_MATCHES).collect();
            if let Some(first) = offered.first_mut() {
                first.head = Some(words::use_with(&self.text));
            }
            names.extend(offered);
        }
        self.list.names = names;
        self.list.move_sel(0);
    }
}
