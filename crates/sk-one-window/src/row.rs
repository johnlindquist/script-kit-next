//! A row the window can list: a name, one of Jev's readings, an action on the staged subject.

use serde::{Deserialize, Serialize};

use crate::contract::WEAK_SCORE;

/// What a row runs when it wins Enter. The host owns the meaning; the core only routes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum RowAct {
    /// The host runs it (a script, an app, a built-in, one of Jev's plans).
    #[default]
    Host,
    /// Stop the held message (`⌘Z`, "wait, don't send").
    Unsend,
    /// Send the held message now.
    SendNow,
    /// Undo the last run.
    Undo,
    /// Run the last run again.
    Again,
    /// Open the saved script the last run became.
    OpenScript,
    /// Offer copy, link, send for the last run.
    Share,
    /// Ask Jev again after it didn't answer.
    AskJevAgain,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    /// Stable identity: the same thing keeps the same key across renders.
    pub key: String,
    pub title: String,
    /// The muted line under the title ("or say “undo that”", "App · 12 min ago").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enter_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Match strength, 0..=100. Names below [`WEAK_SCORE`] never own Enter.
    #[serde(default = "full_score")]
    pub score: u8,
    /// A sentence learned from an earlier run: always strong.
    #[serde(default)]
    pub learned: bool,
    /// "Ask the model" style rows shown when nothing matched.
    #[serde(default)]
    pub fallback: bool,
    /// A header drawn above this row (for pinned lists like "Suggested").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    #[serde(default)]
    pub act: RowAct,
    /// Jev's confidence in this reading, 0..=1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p: Option<f32>,
    /// What Jev's reading acts on ("Sarah", "Linear").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// The reading's verb, when it does something to the target rather than open it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verb: Option<String>,
    /// Where the reading's target came from: `"last"` is something that just ran or is waiting
    /// to go, named by `target` (`"all"` for every waiting message).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The sentence Jev read the words as.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    /// The service (`connect::INTEGRATIONS` key) the reading needs to run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
}

fn full_score() -> u8 {
    100
}

impl Row {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            score: 100,
            ..Self::default()
        }
    }

    pub fn with_enter_label(mut self, label: impl Into<String>) -> Self {
        self.enter_label = Some(label.into());
        self
    }

    pub fn with_sub(mut self, sub: impl Into<String>) -> Self {
        self.sub = Some(sub.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_score(mut self, score: u8) -> Self {
        self.score = score;
        self
    }

    pub fn with_head(mut self, head: impl Into<String>) -> Self {
        self.head = Some(head.into());
        self
    }

    pub fn with_act(mut self, act: RowAct) -> Self {
        self.act = act;
        self
    }

    pub fn fallback(mut self) -> Self {
        self.fallback = true;
        self
    }

    /// What the ↵ button says for this row.
    pub fn enter_words(&self) -> &str {
        self.enter_label.as_deref().unwrap_or(&self.title)
    }

    /// A reading that takes back what just ran or stops what's waiting ("don't send the launcher
    /// one", "undo that").
    pub fn takes_back_last(&self) -> bool {
        self.verb.as_deref() == Some("undo") && self.source.as_deref() == Some("last")
    }

    pub fn is_open_tab(&self) -> bool {
        self.key.starts_with(crate::contract::OPEN_TAB_KEY_PREFIX)
    }

    /// A name strong enough to own Enter.
    pub fn is_strong(&self) -> bool {
        self.learned || self.score >= WEAK_SCORE
    }
}
