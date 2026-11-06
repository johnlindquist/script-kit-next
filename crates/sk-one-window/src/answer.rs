//! Answers shaped like the question: a number for "how much", a list for "what's on", a card for
//! everything else. The answer opens in place of the list under the same input, and Enter does
//! the obvious next thing with it.

use serde::{Deserialize, Serialize};

use crate::enter::EnterTarget;
use crate::form::Source;

/// One thing Jev found for the question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fact {
    pub label: String,
    #[serde(default)]
    pub detail: String,
    pub from: Source,
}

impl Fact {
    pub fn new(label: impl Into<String>, detail: impl Into<String>, from: Source) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
            from,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "shape")]
pub enum Shape {
    /// One value, drawn big: "84", "3 meetings".
    Number { value: String, from: Source },
    /// Several things of the same kind, one per row.
    List { items: Vec<Fact> },
    /// One thing and what's known about it.
    Card { title: String, facts: Vec<Fact> },
    /// Nothing matched the question.
    Nothing,
}

const COUNT_WORDS: &[&str] = &["how many", "how much", "how long", "how far", "how old"];
const LIST_WORDS: &[&str] = &[
    "what's on",
    "whats on",
    "what is on",
    "who's in",
    "whos in",
    "who is in",
    "list",
    "which",
    "all my",
    "show me",
    "what did i",
];

/// Whether the question is arithmetic ("12*7", "15% of 86"): digits joined by an operator.
pub fn is_arithmetic(question: &str) -> bool {
    let q = question.trim();
    let digits = q.chars().filter(char::is_ascii_digit).count();
    let ops = q
        .chars()
        .filter(|c| matches!(c, '+' | '-' | '*' | '/' | '×' | '÷' | '^' | '%'))
        .count();
    digits >= 2
        && (ops >= 1 || q.to_lowercase().contains(" of "))
        && q.chars()
            .all(|c| c.is_ascii_digit() || c.is_whitespace() || "+-*/×÷^%.,()ofOF".contains(c))
}

/// The shape the question asks for, given what was found for it.
pub fn shape_for(question: &str, found: Vec<Fact>) -> Shape {
    let q = question.trim().to_lowercase();
    let q = q.trim_start_matches(['=', ' ']);
    if found.is_empty() {
        return Shape::Nothing;
    }
    let counts = is_arithmetic(q) || COUNT_WORDS.iter().any(|w| q.starts_with(w));
    if counts {
        let value = match found.as_slice() {
            [one] => one.label.clone(),
            many => many.len().to_string(),
        };
        let from = found[0].from.clone();
        return Shape::Number { value, from };
    }
    if LIST_WORDS
        .iter()
        .any(|w| q.starts_with(w) || q.contains(&format!(" {w} ")))
        && found.len() > 1
    {
        return Shape::List { items: found };
    }
    let mut found = found.into_iter();
    let first = found.next().expect("found is not empty");
    let mut facts = Vec::new();
    if !first.detail.is_empty() {
        facts.push(Fact::new("", first.detail, first.from.clone()));
    }
    facts.extend(found);
    Shape::Card {
        title: first.label,
        facts,
    }
}

/// An answer open under the input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub question: String,
    /// `None` while it's being looked up.
    pub shape: Option<Shape>,
}

impl Answer {
    pub fn asking(question: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            shape: None,
        }
    }

    pub fn arrived(&mut self, found: Vec<Fact>) {
        self.shape = Some(shape_for(&self.question, found));
    }

    /// The text Enter copies.
    pub fn copy_text(&self) -> Option<String> {
        match self.shape.as_ref()? {
            Shape::Number { value, .. } => Some(value.clone()),
            Shape::List { items } => Some(
                items
                    .iter()
                    .map(|f| f.label.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            Shape::Card { title, .. } => Some(title.clone()),
            Shape::Nothing => None,
        }
    }

    /// What ↵ says. The answer's arrival changes it on its own, which the window's Enter guard
    /// covers when the host reports it.
    pub fn enter(&self) -> EnterTarget {
        match &self.shape {
            None => EnterTarget::new("answer:wait", "Reading…"),
            Some(Shape::Number { value, .. }) => {
                EnterTarget::new(format!("answer:number:{value}"), format!("Copy {value}"))
            }
            Some(Shape::List { items }) => EnterTarget::new(
                format!("answer:list:{}", items.len()),
                format!("Copy all {}", items.len()),
            ),
            Some(Shape::Card { title, .. }) if title.contains('\n') => {
                let n = title.lines().count();
                EnterTarget::new(format!("answer:lines:{n}"), format!("Copy {n} lines"))
            }
            Some(Shape::Card { title, .. }) => {
                EnterTarget::new(format!("answer:card:{title}"), format!("Copy “{title}”"))
            }
            Some(Shape::Nothing) => EnterTarget::new("answer:nothing", "Nothing to copy"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(label: &str) -> Fact {
        Fact::new(label, "", Source::Known("calendar".into()))
    }

    #[test]
    fn arithmetic_is_a_number() {
        assert!(is_arithmetic("12*7"));
        assert!(is_arithmetic("15% of 86"));
        assert!(!is_arithmetic("call mom at 5"));
        let shape = shape_for("12*7", vec![Fact::new("84", "", Source::YourWords)]);
        assert_eq!(
            shape,
            Shape::Number {
                value: "84".into(),
                from: Source::YourWords
            }
        );
    }

    #[test]
    fn how_many_counts_what_was_found() {
        let shape = shape_for(
            "how many meetings do i have today",
            vec![fact("Standup"), fact("Design review"), fact("1:1")],
        );
        assert!(matches!(shape, Shape::Number { value, .. } if value == "3"));
    }

    #[test]
    fn whats_on_is_a_list_and_a_single_thing_is_a_card() {
        let day = shape_for("what's on today", vec![fact("Standup"), fact("Lunch")]);
        assert!(matches!(day, Shape::List { items } if items.len() == 2));
        let one = shape_for(
            "when's my next meeting",
            vec![Fact::new(
                "Standup · 9:30 AM",
                "with Sarah, Mo, Priya",
                Source::Known("calendar".into()),
            )],
        );
        let Shape::Card { title, facts } = one else {
            panic!("a single meeting is a card");
        };
        assert_eq!(title, "Standup · 9:30 AM");
        assert_eq!(facts[0].detail, "with Sarah, Mo, Priya");
        assert_eq!(shape_for("anything", Vec::new()), Shape::Nothing);
    }

    #[test]
    fn enter_waits_then_copies_the_shape() {
        let mut answer = Answer::asking("12*7");
        assert_eq!(answer.enter().label, "Reading…");
        answer.arrived(vec![Fact::new("84", "", Source::YourWords)]);
        assert_eq!(answer.enter().label, "Copy 84");
        assert_eq!(answer.copy_text().as_deref(), Some("84"));
    }
}
