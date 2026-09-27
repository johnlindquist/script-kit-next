//! Views that open in place of the list: a form, an answer, steps, an agent, a note, the model's
//! chat. The input never moves; Esc goes back one step and restores that step's words.

use crate::enter::EnterTarget;

#[derive(Clone, Debug, PartialEq)]
pub struct ViewFrame {
    /// "form", "chat", "steps", "agent", "note", "brain", "history", ...
    pub name: String,
    /// The crumb: what the view is, in words ("Remind me", "Steps").
    pub label: String,
    /// The words that were in the input when it opened, restored on the way back.
    pub saved_text: String,
    /// What the view leaves in the input on the way back instead, like nothing after an agent.
    pub leave_text: Option<String>,
    pub enter: Option<EnterTarget>,
    /// Typing goes to the view; otherwise typing closes views and searches.
    pub takes_input: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ViewStack {
    frames: Vec<ViewFrame>,
}

impl ViewStack {
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    pub fn top(&self) -> Option<&ViewFrame> {
        self.frames.last()
    }

    pub fn top_mut(&mut self) -> Option<&mut ViewFrame> {
        self.frames.last_mut()
    }

    /// The nearest open view named `name` under the top one.
    pub fn below_mut(&mut self, name: &str) -> Option<&mut ViewFrame> {
        let n = self.frames.len().saturating_sub(1);
        self.frames[..n].iter_mut().rev().find(|f| f.name == name)
    }

    /// The sentence the open views came from.
    pub fn first_saved_text(&self) -> Option<&str> {
        self.frames.first().map(|f| f.saved_text.as_str())
    }

    pub fn open(&mut self, frame: ViewFrame) {
        self.frames.push(frame);
    }

    /// Replaces the top view; the words to restore stay the original ones.
    pub fn replace(&mut self, mut frame: ViewFrame) {
        if let Some(old) = self.frames.pop() {
            frame.saved_text = old.saved_text;
        }
        self.frames.push(frame);
    }

    /// Back one step. Returns the popped frame.
    pub fn back(&mut self) -> Option<ViewFrame> {
        self.frames.pop()
    }

    /// Back to the list.
    pub fn home(&mut self) -> Vec<ViewFrame> {
        std::mem::take(&mut self.frames)
    }

    /// The crumb trail: "Steps › Remind me".
    pub fn crumb(&self) -> String {
        self.frames
            .iter()
            .map(|f| f.label.as_str())
            .collect::<Vec<_>>()
            .join(" › ")
    }
}
