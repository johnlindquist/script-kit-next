//! Signing in to a service from the connect view.
//!
//! Script Kit has no Google Calendar, Slack, Gmail, GitHub, Linear or Notion integration yet, so
//! signing in is this stub: it takes as long as a sign-in would and reports the service connected
//! for the rest of the session. It stores nothing and touches no account. When a real integration
//! lands, its OAuth flow replaces `sign_in`; the reducer's hand-back doesn't change.

use std::time::Duration;

use gpui::Context;
use sk_one_window::work::WorkEvent;
use sk_one_window::Event;

use crate::ScriptListApp;

const SIGN_IN_STUB: Duration = Duration::from_millis(600);

pub(crate) fn sign_in(integration: String, cx: &mut Context<ScriptListApp>) {
    tracing::info!(target: "script_kit::one_window", event = "sign_in_stub", %integration);
    cx.spawn(async move |this, cx| {
        cx.background_executor().timer(SIGN_IN_STUB).await;
        let _ = this.update(cx, |app, cx| {
            app.one_window_handle(Event::Work(WorkEvent::Connected { integration }), cx);
        });
    })
    .detach();
}
