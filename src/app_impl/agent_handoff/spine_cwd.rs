//! The Spine's working directory: kept across launches, handed to the model's session, and
//! optionally warmed ahead of the first turn.

use super::*;

fn agent_chat_hot_prewarm_enabled_from(disabled: Option<&str>, enabled: Option<&str>) -> bool {
    let truthy = |value: Option<&str>| {
        value
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    };

    !truthy(disabled) && truthy(enabled)
}

impl ScriptListApp {
    /// Persist the current `spine_cwd` to user preferences (`ai.cwd`) so it is
    /// restored on the next app launch. Non-fatal and off the UI thread.
    pub(crate) fn persist_spine_cwd(&self) {
        let cwd = self
            .spine_cwd
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        // Every persisted cwd change is also a flow-discovery boundary:
        // kick the roster fetch for the new effective cwd immediately so
        // the main-menu Flows section reflects it without waiting for TTL.
        crate::flows::catalog::flow_catalog().refresh(&crate::flows::resolve_flow_cwd(cwd.clone()));
        std::thread::Builder::new()
            .name("persist-spine-cwd".into())
            .spawn(move || {
                let mut prefs = crate::config::load_user_preferences();
                if prefs.ai.cwd == cwd {
                    return;
                }
                prefs.ai.cwd = cwd.clone();
                if let Err(error) = crate::config::save_user_preferences(&prefs) {
                    tracing::warn!(
                        target: "script_kit::spine",
                        event = "persist_spine_cwd_failed",
                        error = %error,
                    );
                } else {
                    tracing::info!(
                        target: "script_kit::spine",
                        event = "persist_spine_cwd",
                        cwd = ?cwd,
                    );
                }
            })
            .ok();
    }

    /// The working directory to launch the agent in, derived from the Spine cwd
    /// chip. Returns `None` (use the profile/default cwd) unless the user has
    /// *explicitly* picked a cwd (revision > 0) and it is still a directory.
    ///
    /// The startup default (`~/.scriptkit`, revision 0) intentionally does not
    /// override the profile's launch cwd, so default launches keep hitting the
    /// startup-warmed session and the General profile's scratch directory.
    pub(crate) fn spine_cwd_for_agent_chat_launch(&self) -> Option<std::path::PathBuf> {
        if self.spine_cwd_revision == 0 {
            return None;
        }
        let cwd = self.spine_cwd.as_ref()?;
        if cwd.is_dir() {
            Some(cwd.clone())
        } else {
            tracing::warn!(
                target: "script_kit::spine",
                event = "spine_cwd_for_agent_chat_launch_not_a_dir",
                cwd = %cwd.display(),
                "Spine cwd is not a directory; falling back to profile cwd"
            );
            None
        }
    }

    pub(crate) fn agent_chat_hot_prewarm_enabled() -> bool {
        let disabled = std::env::var("SCRIPT_KIT_DISABLE_AGENT_CHAT_HOT_PREWARM").ok();
        let enabled = std::env::var("SCRIPT_KIT_ENABLE_AGENT_CHAT_HOT_PREWARM").ok();
        agent_chat_hot_prewarm_enabled_from(disabled.as_deref(), enabled.as_deref())
    }

    /// Start warming an Agent Chat session for the current Spine cwd so a
    /// later Cmd+Enter acquires a ready warm session with the correct working
    /// directory instead of missing (which would surface the "try again"
    /// toast). Invoked when the user picks a cwd and hot prewarm is explicitly
    /// enabled; idle goose workers are otherwise deferred until first use because
    /// they can consume multiple CPU cores and starve GPUI frame delivery.
    pub(crate) fn prewarm_agent_chat_for_spine_cwd(&self, cx: &mut Context<Self>) {
        let _ = cx;
        let ai_preferences = crate::config::load_user_preferences().ai;
        self.prewarm_selected_agent_chat_profile_for_current_cwd(
            &ai_preferences,
            "prewarm_agent_chat_for_spine_cwd",
        );
    }

    pub(crate) fn prewarm_selected_agent_chat_profile_for_current_cwd(
        &self,
        ai_preferences: &crate::config::AiPreferences,
        source: &'static str,
    ) {
        if !Self::agent_chat_hot_prewarm_enabled() {
            tracing::info!(
                target: "script_kit::spine",
                event = "prewarm_selected_agent_chat_profile_skipped",
                source,
                reason = "disabled_by_default",
            );
            return;
        }

        let profile_ctx = crate::ai::agent_chat::profiles::AgentChatProfileContext::from_setup();
        match crate::ai::agent_chat::launch::resolve_selected_launch_with_cwd_override(
            ai_preferences,
            &profile_ctx,
            self.spine_cwd_for_agent_chat_launch(),
        ) {
            Ok(agent_launch) => {
                let manager = crate::ai::agent_chat::launch::warm_session_manager();
                if let Err(error) = manager.prepare_warm_background(agent_launch.warm_spec()) {
                    tracing::warn!(
                        target: "script_kit::spine",
                        event = "prewarm_selected_agent_chat_profile_failed",
                        source,
                        warm_key = %agent_launch.warm_key,
                        profile_id = %agent_launch.profile.id,
                        error = %error,
                    );
                } else {
                    tracing::info!(
                        target: "script_kit::spine",
                        event = "prewarm_selected_agent_chat_profile",
                        source,
                        warm_key = %agent_launch.warm_key,
                        profile_id = %agent_launch.profile.id,
                        cwd = %agent_launch.cwd.display(),
                    );
                }
            }
            Err(error) => {
                tracing::debug!(
                    target: "script_kit::spine",
                    event = "prewarm_selected_agent_chat_profile_resolution_failed",
                    source,
                    error = %error,
                );
            }
        }
    }
}

#[cfg(test)]
mod hot_prewarm_tests {
    use super::agent_chat_hot_prewarm_enabled_from;

    #[test]
    fn hot_prewarm_is_opt_in_and_disable_wins() {
        assert!(!agent_chat_hot_prewarm_enabled_from(None, None));
        assert!(!agent_chat_hot_prewarm_enabled_from(None, Some("0")));
        assert!(agent_chat_hot_prewarm_enabled_from(None, Some("true")));
        assert!(agent_chat_hot_prewarm_enabled_from(None, Some("1")));
        assert!(!agent_chat_hot_prewarm_enabled_from(
            Some("true"),
            Some("true")
        ));
    }
}
