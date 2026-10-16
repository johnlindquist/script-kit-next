//! How much the model may do before it asks.
//!
//! - **Ask before changes** (default): tools that only look run at once; the
//!   engine asks before anything else (shell, file changes, app actions that
//!   change something, the user's MCP tools), deciding unannotated tools with
//!   a quick model check and asking when unsure.
//! - **Accept edits**: as above, but Script Kit's own file edits run without
//!   asking (they can be reverted); shell and app actions still ask.
//! - **Full access**: nothing asks.
//!
//! The mode is a user preference (`ai.agentApprovalMode` in `config.ts`),
//! applied to new sessions and switched live in open ones.

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum AgentApprovalMode {
    #[default]
    AskBeforeChanges,
    AcceptEdits,
    FullAccess,
}

/// Tools whose changes Script Kit records and can revert.
const REVERTIBLE_EDIT_TOOLS: [&str; 3] = ["write", "edit", "create_file"];

impl AgentApprovalMode {
    pub const ALL: [Self; 3] = [Self::AskBeforeChanges, Self::AcceptEdits, Self::FullAccess];

    /// goose's session mode for this approval mode.
    pub fn goose_mode(self) -> &'static str {
        match self {
            Self::AskBeforeChanges | Self::AcceptEdits => "smart_approve",
            Self::FullAccess => "auto",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::AskBeforeChanges => "askBeforeChanges",
            Self::AcceptEdits => "acceptEdits",
            Self::FullAccess => "fullAccess",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.id() == id.trim())
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::AskBeforeChanges => "Ask before changes",
            Self::AcceptEdits => "Accept file edits",
            Self::FullAccess => "Full access",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::AskBeforeChanges => {
                "The model asks before running commands or changing files, notes, or settings"
            }
            Self::AcceptEdits => {
                "File edits happen without asking (you can revert them); commands still ask"
            }
            Self::FullAccess => "The model never asks; it can run commands and change anything",
        }
    }

    /// Whether Script Kit answers the engine's question for this tool itself.
    pub fn allows_without_asking(self, profile_tool_id: &str) -> bool {
        match self {
            Self::FullAccess => true,
            Self::AcceptEdits => REVERTIBLE_EDIT_TOOLS.contains(&profile_tool_id),
            Self::AskBeforeChanges => false,
        }
    }

    /// The saved preference, or the default.
    pub fn from_preferences(ai: &crate::config::AiPreferences) -> Self {
        ai.agent_approval_mode
            .as_deref()
            .and_then(Self::from_id)
            .unwrap_or_default()
    }

    /// The mode new sessions start in (read from `config.ts` once, then kept).
    pub fn current() -> Self {
        match CURRENT.load(Ordering::Acquire) {
            UNKNOWN => {
                let mode = Self::from_preferences(&crate::config::load_user_preferences().ai);
                CURRENT.store(mode as u8, Ordering::Release);
                mode
            }
            value => Self::from_u8(value),
        }
    }

    /// Make this the mode for new sessions and save it to `config.ts`.
    pub fn remember(self) -> anyhow::Result<()> {
        CURRENT.store(self as u8, Ordering::Release);
        let mut prefs = crate::config::load_user_preferences();
        prefs.ai.agent_approval_mode = Some(self.id().to_string());
        crate::config::save_user_preferences(&prefs)
    }

    fn from_u8(value: u8) -> Self {
        Self::ALL
            .into_iter()
            .find(|mode| *mode as u8 == value)
            .unwrap_or_default()
    }
}

const UNKNOWN: u8 = u8::MAX;
static CURRENT: AtomicU8 = AtomicU8::new(UNKNOWN);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_and_unknown_ids_are_rejected() {
        for mode in AgentApprovalMode::ALL {
            assert_eq!(AgentApprovalMode::from_id(mode.id()), Some(mode));
        }
        assert_eq!(AgentApprovalMode::from_id("yolo"), None);
    }

    #[test]
    fn accept_edits_only_skips_questions_for_revertible_edits() {
        let mode = AgentApprovalMode::AcceptEdits;
        assert!(mode.allows_without_asking("edit"));
        assert!(mode.allows_without_asking("write"));
        assert!(!mode.allows_without_asking("bash"));
        assert!(!mode.allows_without_asking("kit_notes_delete"));
        assert!(!AgentApprovalMode::AskBeforeChanges.allows_without_asking("edit"));
        assert!(AgentApprovalMode::FullAccess.allows_without_asking("bash"));
    }

    #[test]
    fn default_mode_asks() {
        let ai = crate::config::AiPreferences::default();
        assert_eq!(
            AgentApprovalMode::from_preferences(&ai),
            AgentApprovalMode::AskBeforeChanges
        );
        assert_eq!(
            AgentApprovalMode::AskBeforeChanges.goose_mode(),
            "smart_approve"
        );
    }
}
