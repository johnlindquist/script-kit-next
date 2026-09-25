// Provider browser sign-in as seen by an Agent Chat thread.
//
// When a turn needs a subscription provider's browser sign-in, the goose
// runtime asks first (`SignInRequired`). The thread turns that into a modal
// prompt its view presents with the shared parent action dialog: Enter signs
// in, Esc means not now. After consent the runtime starts the login
// (`SignInStarted`); the thread gets the main window out of the login page's
// way and brings the chat back when the sign-in lands (`SignInCompleted`) or
// the turn ends. A sign-in that does not finish comes back as a second modal
// prompt instead of an in-window error card.

/// A provider browser sign-in the current turn is waiting on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentChatSignInWait {
    provider: &'static str,
    completed: bool,
}

/// What an automation client sees about provider sign-in for this thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatSignInState {
    NotSignedIn,
    AwaitingConsent,
    Waiting,
    SignedIn,
    Failed,
}

impl AgentChatSignInState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NotSignedIn => "notSignedIn",
            Self::AwaitingConsent => "awaitingConsent",
            Self::Waiting => "waitingForBrowser",
            Self::SignedIn => "signedIn",
            Self::Failed => "failed",
        }
    }
}

/// Which sign-in modal the view should present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentChatSignInPromptKind {
    /// Ask before opening the provider's sign-in page.
    Consent,
    /// The sign-in did not finish; offer to try again.
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AgentChatSignInPrompt {
    pub(crate) kind: AgentChatSignInPromptKind,
    /// User-facing account name ("ChatGPT").
    pub(crate) provider: &'static str,
    /// Increases for every prompt, so a view presents each one exactly once.
    pub(crate) generation: u64,
}

impl AgentChatThread {
    /// Goose provider id of the thread's current model selection.
    fn selected_provider_id(&self) -> Option<&str> {
        self.selected_model_id
            .as_deref()
            .and_then(|id| id.split_once('/'))
            .map(|(provider, _)| provider)
    }

    pub(crate) fn sign_in_state(&self) -> Option<AgentChatSignInState> {
        if let Some(prompt) = self.sign_in_prompt {
            return Some(match prompt.kind {
                AgentChatSignInPromptKind::Consent => AgentChatSignInState::AwaitingConsent,
                AgentChatSignInPromptKind::Failed => AgentChatSignInState::Failed,
            });
        }
        match self.sign_in_wait {
            Some(wait) if wait.completed => Some(AgentChatSignInState::SignedIn),
            Some(_) => Some(AgentChatSignInState::Waiting),
            None if self.sign_in_failed => Some(AgentChatSignInState::Failed),
            None => self
                .selected_provider_id()
                .filter(|provider| crate::ai::agent_chat::goose::sign_in::needs_sign_in(provider))
                .map(|_| AgentChatSignInState::NotSignedIn),
        }
    }

    /// The sign-in modal the view has not presented yet, if any.
    pub(crate) fn pending_sign_in_prompt(&self) -> Option<AgentChatSignInPrompt> {
        self.sign_in_prompt
    }

    /// Status line for the transient lane above the composer, or `None` when
    /// sign-in needs no mention. Modals carry the decisions; this line only
    /// says where things stand.
    pub(crate) fn sign_in_note(&self) -> Option<String> {
        if self.sign_in_prompt.is_some() {
            return None;
        }
        if let Some(wait) = self.sign_in_wait {
            return Some(if wait.completed {
                format!("Signed in to {}", wait.provider)
            } else {
                format!("Finish signing in to {} in your browser", wait.provider)
            });
        }
        if !matches!(
            self.status,
            AgentChatThreadStatus::Idle | AgentChatThreadStatus::Error
        ) {
            return None;
        }
        let provider = self.selected_provider_id()?;
        crate::ai::agent_chat::goose::sign_in::needs_sign_in(provider).then(|| {
            format!(
                "Not signed in to {}",
                crate::ai::agent_chat::goose::sign_in::provider_display_name(provider)
            )
        })
    }

    fn show_sign_in_prompt(&mut self, kind: AgentChatSignInPromptKind, provider: &str) -> bool {
        self.sign_in_prompt_generation = self.sign_in_prompt_generation.wrapping_add(1);
        self.sign_in_prompt = Some(AgentChatSignInPrompt {
            kind,
            provider: crate::ai::agent_chat::goose::sign_in::provider_display_name(provider),
            generation: self.sign_in_prompt_generation,
        });
        true
    }

    /// The runtime asks before opening the sign-in page.
    fn request_sign_in_consent(
        &mut self,
        provider: &str,
        consent: async_channel::Sender<bool>,
    ) -> bool {
        // Try Again already said yes for this turn.
        if std::mem::take(&mut self.sign_in_preapproved) {
            let _ = consent.try_send(true);
            return false;
        }
        tracing::info!(
            target: "script_kit::agent_chat",
            event = "agent_chat_sign_in_consent_requested",
            provider = %provider,
        );
        self.sign_in_consent = Some(consent);
        self.show_sign_in_prompt(AgentChatSignInPromptKind::Consent, provider)
    }

    /// Answer the consent modal: Sign In (`true`) or Not Now (`false`).
    /// Not Now is a user cancel, exactly like Stop: nothing opens, nothing
    /// fails, and the chat is ready for the next message.
    pub(crate) fn answer_sign_in_consent(&mut self, approve: bool, cx: &mut Context<Self>) {
        if self
            .sign_in_prompt
            .is_some_and(|prompt| prompt.kind == AgentChatSignInPromptKind::Consent)
        {
            self.sign_in_prompt = None;
        }
        tracing::info!(
            target: "script_kit::agent_chat",
            event = "agent_chat_sign_in_consent_answered",
            approve,
        );
        if approve {
            if let Some(consent) = self.sign_in_consent.take() {
                let _ = consent.try_send(true);
            }
            self.notify_semantic_change(cx);
        } else {
            // Stopping drops the consent sender; the runtime reads that as no.
            self.stop_streaming(cx);
            self.notify_semantic_change(cx);
        }
    }

    /// Try Again on a failed sign-in: the next consent request is already
    /// answered, so the retried turn goes straight to the sign-in page.
    pub(crate) fn prepare_sign_in_retry(&mut self) {
        self.sign_in_preapproved = true;
        self.sign_in_failed = false;
        self.sign_in_prompt = None;
    }

    /// Not Now on a failed sign-in: close the recovery without retrying.
    pub(crate) fn dismiss_sign_in_failure(&mut self, cx: &mut Context<Self>) {
        self.sign_in_prompt = None;
        self.sign_in_failed = false;
        self.dismiss_recovery(cx);
    }

    /// Whether the sign-in modal, not the in-window recovery card, presents
    /// the current failure.
    pub(crate) fn sign_in_failure_owns_recovery(&self) -> bool {
        self.sign_in_failed
    }

    /// A failure that only a provider sign-in can fix.
    fn is_sign_in_failure(&self, failure: &crate::ai::reliability::AppFailureRecord) -> bool {
        failure.failure.code == sk_protocol::ai_reliability::AiFailureCode::AuthenticationMissing
            && self
                .selected_provider_id()
                .is_some_and(crate::ai::agent_chat::goose::sign_in::is_browser_sign_in_provider)
    }

    /// Route a sign-in failure to the modal. Returns whether it did.
    fn present_sign_in_failure(
        &mut self,
        failure: &crate::ai::reliability::AppFailureRecord,
    ) -> bool {
        if !self.is_sign_in_failure(failure) {
            return false;
        }
        let Some(provider) = self.selected_provider_id().map(str::to_string) else {
            return false;
        };
        self.sign_in_failed = true;
        self.show_sign_in_prompt(AgentChatSignInPromptKind::Failed, &provider)
    }

    fn hosted_in_main_window(&self) -> bool {
        self.host_window_state
            .is_some_and(|state| state.kind == AgentChatHostWindowKind::Main)
    }

    /// The sign-in page is opening; the turn continues once it lands.
    fn begin_sign_in_wait(&mut self, provider: &str, cx: &mut Context<Self>) -> bool {
        self.sign_in_failed = false;
        let wait = AgentChatSignInWait {
            provider: crate::ai::agent_chat::goose::sign_in::provider_display_name(provider),
            completed: false,
        };
        if self.sign_in_wait == Some(wait) {
            return false;
        }
        self.sign_in_wait = Some(wait);
        tracing::info!(
            target: "script_kit::agent_chat",
            event = "agent_chat_sign_in_waiting",
            provider = %provider,
        );
        // The main window floats above other apps and would cover the login
        // page. Hide it without dismissing so the chat comes back intact.
        if self.hosted_in_main_window()
            && crate::platform::defer_hide_main_window_preserving_state(cx, "provider_sign_in")
        {
            self.sign_in_hid_main_window = true;
        }
        true
    }

    /// The sign-in finished; the waiting turn is continuing.
    fn complete_sign_in_wait(&mut self, provider: &str) -> bool {
        tracing::info!(
            target: "script_kit::agent_chat",
            event = "agent_chat_sign_in_completed",
            provider = %provider,
        );
        self.sign_in_wait = Some(AgentChatSignInWait {
            provider: crate::ai::agent_chat::goose::sign_in::provider_display_name(provider),
            completed: true,
        });
        self.restore_main_window_after_sign_in();
        true
    }

    /// The turn ended (answered, failed, or stopped): drop any open question
    /// and the status line, and make sure the user can see the outcome.
    fn end_sign_in_wait(&mut self) -> bool {
        let mut changed = self.sign_in_wait.take().is_some();
        if self.sign_in_consent.take().is_some() {
            changed = true;
        }
        if self
            .sign_in_prompt
            .is_some_and(|prompt| prompt.kind == AgentChatSignInPromptKind::Consent)
        {
            self.sign_in_prompt = None;
            changed = true;
        }
        self.restore_main_window_after_sign_in();
        changed
    }

    fn restore_main_window_after_sign_in(&mut self) {
        if std::mem::take(&mut self.sign_in_hid_main_window) && !crate::is_main_window_visible() {
            crate::request_show_main_window();
        }
    }
}
