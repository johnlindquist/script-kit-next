//! Jev may offer to inspect a sheet. Only the literal between the user's quotes
//! can become a filename. No provider response here can save a file or set a value.
use super::{
    engine::{Pathway, Plan, Reading, Step, Timing},
    jev::{Answer, Criteria, JevError, Oracle, Question, Questions},
    ScreenContext,
};
use regex::Regex;
use std::sync::LazyLock;

fn candidate(request: &str) -> Option<(&str, String)> {
    static LITERAL: LazyLock<Option<Regex>> = LazyLock::new(|| {
        Regex::new(r#"“([^“”\r\n]+)”|‘([^‘’\r\n]+)’|"([^"\r\n]+)"|(?:^|[\s:=])'([^'\r\n]+)'"#).ok()
    });
    let mut parts = LITERAL.as_ref()?.captures_iter(request);
    let c = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let value = c.iter().skip(1).flatten().next()?.as_str();
    if !crate::desktop_fill::valid_filename(value) {
        return None;
    }
    let mut words = request.to_owned();
    words.replace_range(c.get(0)?.range(), " [supplied name] ");
    let lower = words.to_lowercase();
    if ![
        "filename",
        "file name",
        "this export",
        "the export",
        "export name",
        "save as field",
        "save dialog",
        "save sheet",
        "name the file",
        "name this file",
    ]
    .iter()
    .any(|x| lower.contains(x))
    {
        return None;
    }
    Some((value, words))
}
pub(super) fn read(
    request: &str,
    ctx: &ScreenContext,
    oracle: &dyn Oracle,
) -> Option<Result<Plan, JevError>> {
    let parsed = candidate(request);
    // While the user is still typing the literal, never fall through to an app
    // menu such as Save As. This also keeps an unsafe path out of the reader.
    if parsed.is_none() {
        let lower = request.trim().to_ascii_lowercase();
        if ![
            "name this export",
            "name the export",
            "set the filename",
            "fill the filename",
            "set the file name",
        ]
        .iter()
        .any(|prefix| {
            lower == *prefix
                || lower
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with(' '))
        }) {
            return None;
        }
    }
    let (filename, words) = parsed.unwrap_or(("", String::new()));
    let mut criteria = Criteria::default();
    for (key, description) in [
        ("fill_only", "Replace the filename in the open Save or Export dialog. Leave the dialog open."),
        ("commit", "Submit, save or export, including both renaming and saving."),
        ("other", "Advice, reading, negation of the edit, renaming a file on disk, or any other or unclear request."),
    ] {
        criteria.set(key, Some(description.into()));
    }
    let mut q = Questions::default();
    q.insert("operation",Question::Choice{instructions:"Which complete effect did the speaker request? A mention of saving is not a request to save.".into(),criteria});
    Some((|| {
        let r = if filename.is_empty() {
            Default::default()
        } else {
            let r = oracle.ask(&serde_json::json!({"request":words}), &q)?;
            q.validate_answers(&r.answers)?;
            r
        };
        let probability = match r.answers.get("operation") {
            Some(Answer::Choice { probabilities, .. }) => {
                probabilities.get("fill_only").unwrap_or(0.0)
            }
            _ => 0.0,
        };
        let matched = matches!(r.answers.get("operation"), Some(Answer::Choice { choice, .. }) if choice == "fill_only")
            && probability >= 0.7;
        let step = ctx
            .filename_window
            .filter(|_| matched)
            .map(|(pid, window_id)| Step::InspectFilename {
                app: ctx.front_app.name.clone(),
                pid,
                window_id,
                title: ctx.window_title.clone(),
                filename: filename.into(),
            });
        let ready = step.is_some();
        let title = if ready {
            format!("Inspect filename in {}", ctx.front_app.name)
        } else {
            "Fill a Save dialog filename".into()
        };
        Ok(Plan {
            reading: Reading {
                pathway: Pathway::AppCommand,
                probability,
                title,
                steps: step.into_iter().collect(),
                typed: None,
                typed_is_proposed: false,
                missing: (!ready).then(|| {
                    if filename.is_empty() {
                        "one valid filename in quotation marks"
                    } else if matched {
                        "an identifiable originating window"
                    } else {
                        "a request to fill only the filename"
                    }
                    .into()
                }),
                tip: Some("Preview first; never presses Save".into()),
            },
            request: request.into(),
            complete: if ready { 1. } else { 0. },
            reuse: 0.,
            ready,
            alternatives: vec![],
            slots: Default::default(),
            words: vec![],
            clauses: vec![],
            timing: Timing {
                ms: r.ms,
                input_tokens: r.input_tokens,
                questions: usize::from(!filename.is_empty()),
                rounds: u8::from(!filename.is_empty()),
            },
        })
    })())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Controlled(&'static str);
    impl Oracle for Controlled {
        fn ask(
            &self,
            state: &serde_json::Value,
            questions: &Questions,
        ) -> Result<super::super::jev::JevResult, JevError> {
            assert!(questions.get("operation").is_some());
            assert_eq!(state.as_object().unwrap().len(), 1);
            assert!(!state.to_string().contains("Workshop outline"));
            let answers=serde_json::from_value(serde_json::json!({"operation":{"type":"choice","choice":self.0,"confidence":0.99,"probabilities":{(self.0):0.99}}})).unwrap();
            Ok(super::super::jev::JevResult {
                answers,
                ..Default::default()
            })
        }
    }
    #[test]
    fn only_a_fill_reading_and_a_pinned_window_offer_inspection() {
        let mut ctx = ScreenContext {
            filename_window: Some((42, 100)),
            window_title: "Draft".into(),
            ..Default::default()
        };
        ctx.front_app.name = "Editor".into();
        for (sentence, answer, ready) in [
            (
                "Name this export ‘Workshop outline’; don’t save.",
                "fill_only",
                true,
            ),
            ("Should the filename be ‘Workshop outline’?", "other", false),
            (
                "Name this export ‘Workshop outline’ and save it.",
                "commit",
                false,
            ),
            (
                "Don't change the filename to 'Workshop outline'.",
                "other",
                false,
            ),
        ] {
            let plan = read(sentence, &ctx, &Controlled(answer)).unwrap().unwrap();
            assert_eq!(plan.ready, ready);
            if ready {
                assert!(
                    matches!(&plan.steps[..],[Step::InspectFilename{pid:42,window_id:100,filename,..}] if filename=="Workshop outline")
                );
            } else {
                assert!(plan.steps.is_empty());
            }
        }
        ctx.filename_window = None;
        assert!(
            !read(
                "Name this export ‘Workshop outline’",
                &ctx,
                &Controlled("fill_only")
            )
            .unwrap()
            .unwrap()
            .ready
        );
    }
    #[test]
    fn incomplete_or_invalid_filename_never_reaches_a_menu_or_provider() {
        let ctx = ScreenContext::default();
        for request in [
            "Name this export ‘Half typed",
            "Set the filename to ‘../outside’",
            "Name this export",
        ] {
            let p = read(request, &ctx, &Controlled("fill_only"))
                .unwrap()
                .unwrap();
            assert!(!p.ready && p.steps.is_empty());
            assert_eq!(p.timing.rounds, 0);
        }
    }
    #[test]
    fn literal_is_verbatim_and_not_sent_to_the_classifier() {
        let (name, request) =
            candidate("Name this export ‘Workshop outline’; don’t save.").unwrap();
        assert_eq!(name, "Workshop outline");
        assert!(!request.contains(name));
        assert!(candidate("Name this export ‘../../bad’").is_none());
        assert!(candidate("Use ‘A’ or ‘B’ as the filename").is_none());
        assert!(candidate("Look up ‘save as’").is_none());
    }
}
