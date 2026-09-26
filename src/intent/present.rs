//! How a plan reads in the launcher: the row subtitle and the Enter button's verb.

use super::catalog::{CommandKind, WindowAction};
use super::engine::{Pathway, Plan, Step};

/// The Enter button's label for a plan.
pub fn plan_verb(plan: &Plan) -> &'static str {
    if plan.clauses.len() > 1 {
        return "Do All";
    }
    match plan.pathway {
        Pathway::Run => {
            let all_apps = !plan.steps.is_empty()
                && plan.steps.iter().all(
                    |step| matches!(step, Step::Command { command } if command.kind == CommandKind::App),
                );
            if all_apps {
                "Open"
            } else {
                "Run"
            }
        }
        Pathway::AppCommand => "Click",
        Pathway::Window => match window_action(plan) {
            Some(WindowAction::Quit) => "Quit",
            Some(WindowAction::Close) => "Close",
            Some(WindowAction::Minimize) => "Minimize",
            _ => "Arrange",
        },
        Pathway::Capture => "Add",
        Pathway::Remember => "Remember",
        Pathway::Recall => "Search",
        Pathway::Transform => "Rewrite",
        Pathway::Ask => "Ask AI",
        Pathway::Delegate => "Hand Off",
        Pathway::Automate => "Save Script",
        Pathway::None => "Run",
    }
}

fn window_action(plan: &Plan) -> Option<WindowAction> {
    plan.steps.iter().find_map(|step| match step {
        Step::Window { op, .. } => op.action(),
        _ => None,
    })
}

/// The row's second line: what kind of thing happens, and the launcher line that
/// does the same when there is one, so the syntax is learned by using it.
pub fn plan_subtitle(plan: &Plan) -> String {
    let what = if plan.clauses.len() > 1 {
        format!("{} steps", plan.clauses.len())
    } else {
        match plan.pathway {
            Pathway::Run => "Runs a command".to_string(),
            Pathway::AppCommand => match plan.steps.first() {
                Some(Step::Menu { app, .. }) => format!("{app} menu"),
                _ => "App menu".to_string(),
            },
            Pathway::Window => match window_action(plan) {
                Some(WindowAction::Quit) => "Quits an app".to_string(),
                Some(WindowAction::Close) => "Closes a window".to_string(),
                Some(WindowAction::Minimize) => "Hides a window".to_string(),
                _ => "Moves a window".to_string(),
            },
            Pathway::Capture => "Writes it down".to_string(),
            Pathway::Remember => "Saves to your brain".to_string(),
            Pathway::Recall => "Searches your history".to_string(),
            Pathway::Transform => "Rewrites text".to_string(),
            Pathway::Ask => "Quick AI".to_string(),
            Pathway::Delegate => "Hands off to an agent".to_string(),
            Pathway::Automate => "Saves a script".to_string(),
            Pathway::None => String::new(),
        }
    };
    match (&plan.typed, plan.typed_is_proposed) {
        (Some(typed), false) if !typed.trim().is_empty() => format!("{what} · same as {typed}"),
        _ => what,
    }
}
