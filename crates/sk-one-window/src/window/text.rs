//! What typing does to the words, the list and Jev's debounce.

use super::*;

impl OneWindow {
    pub(super) fn on_text(&mut self, now: Ms, text: String, typed: bool, fx: &mut Vec<Effect>) {
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
        self.host_names.clear();
        self.list.on_text(&self.text);
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

        let mut names: Vec<Row> = self
            .staged
            .as_ref()
            .map(|s| s.matches(&self.text))
            .unwrap_or_default();
        let (fallbacks, real): (Vec<Row>, Vec<Row>) =
            self.host_names.iter().cloned().partition(|r| r.fallback);
        for r in real {
            if !names.iter().any(|n| n.title.eq_ignore_ascii_case(&r.title)) {
                names.push(r);
            }
        }
        names.truncate(MAX_MATCHES);
        if names.is_empty() && !empty {
            let down = self.list.jev.is_down_for(&self.text);
            names = fallbacks
                .into_iter()
                .filter(|r| !(down && r.key.starts_with("fallback-ai:")))
                .collect();
        }
        self.list.names = names;
        self.list.move_sel(0);
    }
}
