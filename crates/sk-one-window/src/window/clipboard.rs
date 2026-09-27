//! ⌘V into the empty input stages what was copied; into typed words it pastes as usual.

use super::*;

/// The key the host pastes with when the window leaves the paste to it.
pub const PASTE_KEY: &str = "cmd-v";

impl OneWindow {
    pub(super) fn on_paste(&mut self, now: Ms, text: String, fx: &mut Vec<Effect>) -> bool {
        let stages = self.text.is_empty() && !self.in_view() && !self.away;
        match crate::clipboard::subject(&text).filter(|_| stages) {
            Some(subject) => {
                let rows = crate::clipboard::actions(&subject);
                self.stage(now, subject, Some(rows), words::STAGED, fx);
                self.list.sel = 0;
                true
            }
            None => {
                fx.push(Effect::PassKey(Key::Other(PASTE_KEY.into())));
                false
            }
        }
    }
}
