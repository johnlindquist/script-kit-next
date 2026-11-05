//! The model asks before it does something (runs a command, edits a file). Its question is rows
//! under the reply, never a sheet: the one-time yes first, so ↵ allows it once; ⌘2 says no.

use crate::row::Row;

/// One answer the model offers.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub id: String,
    pub label: String,
    pub allows: bool,
    /// The yes holds for later requests too ("Always allow").
    pub lasting: bool,
}

impl Choice {
    pub fn new(id: impl Into<String>, label: impl Into<String>, allows: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            allows,
            lasting: false,
        }
    }

    pub fn lasting(mut self) -> Self {
        self.lasting = true;
        self
    }

    fn order(&self) -> u8 {
        match (self.allows, self.lasting) {
            (true, false) => 0,
            (true, true) => 1,
            (false, _) => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Asks {
    /// The model's request, as the host answers it.
    pub id: u64,
    /// What it wants to do: "Run rm -rf build", "Edit src/main.rs".
    pub title: String,
    pub detail: Option<String>,
    /// Full Kit-validated proposal, displayed above both choices (never a clipped row subtitle).
    pub launcher_review: Option<String>,
    pub choices: Vec<Choice>,
}

const KEY_PREFIX: &str = "ask:";

impl Asks {
    fn choice_key(&self, id: &str) -> String {
        if self.launcher_review.is_some() {
            format!("{KEY_PREFIX}{}:{id}", self.id)
        } else {
            format!("{KEY_PREFIX}{id}")
        }
    }

    /// The choices as rows: a one-time yes, a lasting yes, then no.
    pub fn rows(&self) -> Vec<Row> {
        let mut choices: Vec<&Choice> = self.choices.iter().collect();
        choices.sort_by_key(|c| c.order());
        choices
            .into_iter()
            .map(|c| {
                let row = Row::new(self.choice_key(&c.id), c.label.clone())
                    .with_icon(if c.allows { "check" } else { "close" })
                    .with_enter_label(format!(
                        "{}: {}",
                        if c.allows { "Allow" } else { "Don't allow" },
                        self.title
                    ));
                match (&self.detail, c.allows) {
                    (_, false) => row.with_sub("⌘2"),
                    (Some(detail), true) => row.with_sub(detail.clone()),
                    (None, true) => row,
                }
            })
            .collect()
    }

    pub fn choice_of(&self, key: &str) -> Option<String> {
        self.choices
            .iter()
            .find(|choice| self.choice_key(&choice.id) == key)
            .map(|choice| choice.id.clone())
    }

    /// The first way to say no; `None` cancels when the model offers none.
    pub fn deny(&self) -> Option<String> {
        self.choices
            .iter()
            .find(|c| !c.allows)
            .map(|c| c.id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_one_time_yes_comes_first_and_no_is_cmd_2() {
        let asks = Asks {
            id: 1,
            title: "Edit README.md".into(),
            detail: Some("+3 −1".into()),
            launcher_review: None,
            choices: vec![
                Choice::new("reject", "Reject", false),
                Choice::new("always", "Always allow", true).lasting(),
                Choice::new("once", "Allow once", true),
            ],
        };
        let rows = asks.rows();
        let titles: Vec<&str> = rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, ["Allow once", "Always allow", "Reject"]);
        assert_eq!(rows[0].enter_words(), "Allow: Edit README.md");
        assert_eq!(rows[0].sub.as_deref(), Some("+3 −1"));
        assert_eq!(rows[2].sub.as_deref(), Some("⌘2"));
        assert_eq!(rows[2].enter_words(), "Don't allow: Edit README.md");
        assert_eq!(asks.deny().as_deref(), Some("reject"));
        assert_eq!(asks.choice_of(&rows[1].key).as_deref(), Some("always"));
    }

    #[test]
    fn an_old_save_row_cannot_answer_a_replacement_proposal() {
        let mut asks = Asks {
            id: 8,
            title: "Change your launcher".into(),
            detail: None,
            launcher_review: Some("Only while dictating".into()),
            choices: vec![
                Choice::new("once", "Save once", true),
                Choice::new("no", "Don’t save", false),
            ],
        };
        let old = asks.rows()[0].key.clone();
        asks.id = 9;
        assert!(asks.choice_of(&old).is_none());
        assert_eq!(asks.choice_of(&asks.rows()[0].key).as_deref(), Some("once"));
        assert_eq!(asks.deny().as_deref(), Some("no"));
    }
}
