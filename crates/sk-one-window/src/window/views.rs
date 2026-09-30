//! Views that open in the main window under the same input, and the way back.

use super::*;

impl OneWindow {
    pub(super) fn on_view(&mut self, now: Ms, op: ViewOp, fx: &mut Vec<Effect>) {
        let user = self.user_turn(now);
        match op {
            ViewOp::Open(spec) | ViewOp::Reopen(spec) => {
                let frame = self.frame(&spec);
                self.views.open(frame);
                self.opened(now, spec, user, fx);
            }
            ViewOp::Replace(spec) => {
                let frame = self.frame(&spec);
                self.views.replace(frame);
                self.opened(now, spec, user, fx);
            }
            ViewOp::Back => {
                let Some(popped) = self.views.back() else {
                    return;
                };
                if let Some(top) = self.views.top() {
                    let target = top
                        .enter
                        .clone()
                        .unwrap_or_else(|| EnterTarget::new(top.name.clone(), ""));
                    self.cause = Some((ContentCause::ViewOpened, now));
                    self.guard.observe_view(now, &target.id, user);
                    let restore = popped.leave_text.unwrap_or(popped.saved_text);
                    fx.push(Effect::SetInput(restore.clone()));
                    self.text = restore;
                } else {
                    self.guard.forget_view();
                    self.guard.arm_unless_user(now, user);
                    self.cause = Some((ContentCause::Home, now));
                    self.set_input(now, popped.leave_text.unwrap_or(popped.saved_text), fx);
                }
            }
            ViewOp::Home { text } => self.home(now, text, fx),
            ViewOp::Park => {
                self.views.home();
                self.guard.forget_view();
                self.guard.arm_unless_user(now, user);
                self.cause = Some((ContentCause::Home, now));
                self.set_input(now, String::new(), fx);
            }
            ViewOp::Label(label) => {
                if let Some(top) = self.views.top_mut() {
                    top.label = label;
                }
            }
        }
    }

    pub(super) fn frame(&self, spec: &ViewSpec) -> ViewFrame {
        ViewFrame {
            name: spec.name.clone(),
            label: spec.label.clone(),
            saved_text: self.text.clone(),
            leave_text: spec.leave_text.clone(),
            enter: spec.enter.clone(),
            takes_input: spec.takes_input,
        }
    }

    pub(super) fn opened(&mut self, now: Ms, spec: ViewSpec, user: bool, fx: &mut Vec<Effect>) {
        self.cause = Some((ContentCause::ViewOpened, now));
        self.list.held = None;
        let target = spec
            .enter
            .unwrap_or_else(|| EnterTarget::new(spec.name.clone(), ""));
        self.guard.forget_view();
        self.guard.observe_view(now, &target.id, user);
        if spec.name == crate::strip::AGENT_VIEW && std::mem::take(&mut self.agent_owns) {
            self.refresh_rows();
        }
        if let Some(text) = spec.text {
            if text != self.text {
                fx.push(Effect::SetInput(text.clone()));
            }
            self.text = text;
        }
    }

    pub(super) fn home(&mut self, now: Ms, text: String, fx: &mut Vec<Effect>) {
        self.views.home();
        self.drop_chips();
        self.guard.forget_view();
        let user = self.user_turn(now);
        self.guard.arm_unless_user(now, user);
        self.cause = Some((ContentCause::Home, now));
        self.set_input(now, text, fx);
    }
}
