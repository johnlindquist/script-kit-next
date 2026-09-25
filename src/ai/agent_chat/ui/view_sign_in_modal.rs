// Provider sign-in modals for Agent Chat.
//
// Both use the shared parent action dialog (the confirm popup attached to the
// window hosting the chat), so they share its keyboard contract: ↵ runs the
// focused button (the primary one by default), Esc chooses Not Now, and Tab /
// ⇧Tab move between buttons.

/// How long a Failed prompt waits for the main window to come back from the
/// sign-in page (visible and active) before attaching to it.
const SIGN_IN_MODAL_HOST_WAIT: std::time::Duration = std::time::Duration::from_millis(1_500);

/// Frames for the main window to process regaining focus before a modal
/// attaches to it.
const SIGN_IN_MODAL_FOCUS_SETTLE: std::time::Duration = std::time::Duration::from_millis(250);

impl AgentChatView {
    fn present_sign_in_prompt(
        &mut self,
        prompt: AgentChatSignInPrompt,
        thread: gpui::WeakEntity<AgentChatThread>,
        cx: &mut Context<Self>,
    ) {
        let Some(window_handle) = self
            .host_window_handle
            .or_else(crate::get_main_window_handle)
        else {
            tracing::error!(
                target: "script_kit::agent_chat",
                event = "agent_chat_sign_in_modal_host_missing",
            );
            return;
        };
        let hosted_in_main = crate::get_main_window_handle() == Some(window_handle);
        let view = cx.entity().downgrade();
        let provider = prompt.provider;
        let options = match prompt.kind {
            AgentChatSignInPromptKind::Consent => crate::confirm::ParentActionDialogOptions {
                title: format!("Sign in to {provider}").into(),
                body: format!(
                    "Script Kit uses your {provider} account for AI. Sign-in opens in your browser, and your message sends as soon as you finish."
                )
                .into(),
                primary_text: "Sign In".into(),
                secondary_text: None,
                dismiss_text: "Not Now".into(),
                ..Default::default()
            },
            AgentChatSignInPromptKind::Failed => crate::confirm::ParentActionDialogOptions {
                title: format!("{provider} sign-in didn't finish").into(),
                body: "Your message is saved. Try again when you're ready to finish signing in in your browser."
                    .into(),
                primary_text: "Try Again".into(),
                secondary_text: None,
                dismiss_text: "Not Now".into(),
                ..Default::default()
            },
        };
        tracing::info!(
            target: "script_kit::agent_chat",
            event = "agent_chat_sign_in_modal_presenting",
            kind = ?prompt.kind,
            generation = prompt.generation,
        );
        let kind = prompt.kind;
        cx.spawn(async move |_this, cx| {
            // A failed sign-in returns from the browser. When the main window
            // regains focus it closes any open modal (it reads that as a click
            // back into the window), on its next frame. Attach only once the
            // window is back, focused, and that frame has passed.
            if hosted_in_main && kind == AgentChatSignInPromptKind::Failed {
                let deadline = std::time::Instant::now() + SIGN_IN_MODAL_HOST_WAIT;
                while !(crate::is_main_window_visible()
                    && crate::platform::is_main_window_focused())
                    && std::time::Instant::now() < deadline
                {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(50))
                        .await;
                }
                cx.background_executor()
                    .timer(SIGN_IN_MODAL_FOCUS_SETTLE)
                    .await;
            }
            let opened = cx.update_window(window_handle, move |_, window, cx| {
                let (on_primary, on_dismiss) = sign_in_modal_callbacks(kind, thread, view);
                crate::confirm::open_parent_action_dialog(
                    window,
                    cx,
                    options,
                    move |window, cx| on_primary(window, cx),
                    |_window, _cx| {},
                    move |window, cx| on_dismiss(window, cx),
                );
            });
            if let Err(error) = opened {
                tracing::error!(
                    target: "script_kit::agent_chat",
                    event = "agent_chat_sign_in_modal_open_failed",
                    error = ?error,
                );
            }
        })
        .detach();
    }
}

type SignInModalCallback = Rc<dyn Fn(&mut Window, &mut App)>;

fn sign_in_modal_callbacks(
    kind: AgentChatSignInPromptKind,
    thread: gpui::WeakEntity<AgentChatThread>,
    view: gpui::WeakEntity<AgentChatView>,
) -> (SignInModalCallback, SignInModalCallback) {
    match kind {
        AgentChatSignInPromptKind::Consent => {
            let accept = thread.clone();
            (
                Rc::new(move |_window, cx| {
                    let _ = accept.update(cx, |thread, cx| thread.answer_sign_in_consent(true, cx));
                }),
                Rc::new(move |_window, cx| {
                    let _ =
                        thread.update(cx, |thread, cx| thread.answer_sign_in_consent(false, cx));
                }),
            )
        }
        AgentChatSignInPromptKind::Failed => (
            // Try Again is the card's Sign In recovery action: it resumes the
            // saved turn, which goes straight to the sign-in page.
            Rc::new(move |_window, cx| {
                let _ = view.update(cx, |view, cx| {
                    view.dispatch_ai_recovery_action("ai-recovery-sign-in", cx)
                });
            }),
            Rc::new(move |_window, cx| {
                let _ = thread.update(cx, |thread, cx| thread.dismiss_sign_in_failure(cx));
            }),
        ),
    }
}
