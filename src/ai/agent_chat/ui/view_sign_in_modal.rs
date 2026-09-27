// Provider sign-in questions for Agent Chat.
//
// Both go in the line above the main input: the answers are rows, ↵ runs the highlighted one,
// and Esc lets go of the question, which answers Not Now.

impl AgentChatView {
    fn present_sign_in_prompt(
        &mut self,
        prompt: AgentChatSignInPrompt,
        thread: gpui::WeakEntity<AgentChatThread>,
        cx: &mut Context<Self>,
    ) {
        let view = cx.entity().downgrade();
        let provider = prompt.provider;
        let (title, detail, primary) = match prompt.kind {
            AgentChatSignInPromptKind::Consent => (
                format!("Sign in to {provider}"),
                format!(
                    "Script Kit uses your {provider} account for AI. Sign-in opens in your browser, and your message sends as soon as you finish."
                ),
                "Sign In",
            ),
            AgentChatSignInPromptKind::Failed => (
                format!("{provider} sign-in didn't finish"),
                "Your message is saved. Try again when you're ready to finish signing in in your browser."
                    .to_string(),
                "Try Again",
            ),
        };
        tracing::info!(
            target: "script_kit::agent_chat",
            event = "agent_chat_sign_in_question_asked",
            kind = ?prompt.kind,
            generation = prompt.generation,
        );
        let (on_primary, on_dismiss) = sign_in_answers(prompt.kind, thread, view);
        crate::line_notice::ask(
            cx,
            "ai",
            title,
            detail,
            vec![
                (primary.to_string(), on_primary),
                ("Not Now".to_string(), later(&on_dismiss)),
            ],
            Some(later(&on_dismiss)),
        );
    }
}

type SignInAnswer = Rc<dyn Fn(&mut App)>;

/// Not Now is both a row and what letting go of the question means.
fn later(answer: &SignInAnswer) -> crate::line_notice::Later {
    let answer = answer.clone();
    Box::new(move |cx| answer(cx))
}

fn sign_in_answers(
    kind: AgentChatSignInPromptKind,
    thread: gpui::WeakEntity<AgentChatThread>,
    view: gpui::WeakEntity<AgentChatView>,
) -> (crate::line_notice::Later, SignInAnswer) {
    match kind {
        AgentChatSignInPromptKind::Consent => {
            let accept = thread.clone();
            (
                Box::new(move |cx| {
                    let _ = accept.update(cx, |thread, cx| thread.answer_sign_in_consent(true, cx));
                }),
                Rc::new(move |cx| {
                    let _ =
                        thread.update(cx, |thread, cx| thread.answer_sign_in_consent(false, cx));
                }),
            )
        }
        AgentChatSignInPromptKind::Failed => (
            // Try Again is the card's Sign In recovery action: it resumes the
            // saved turn, which goes straight to the sign-in page.
            Box::new(move |cx| {
                let _ = view.update(cx, |view, cx| {
                    view.dispatch_ai_recovery_action("ai-recovery-sign-in", cx)
                });
            }),
            Rc::new(move |cx| {
                let _ = thread.update(cx, |thread, cx| thread.dismiss_sign_in_failure(cx));
            }),
        ),
    }
}
