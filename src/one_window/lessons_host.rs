//! Keeps what the reducer learned and ran across sessions, and runs a learned row.
//!
//! A learned row runs the plan it ran before when that plan is still in memory; after a restart
//! the sentence goes back in the input and Enter waits for Jev's reading of it, so a row never
//! runs something other than what the list showed without asking. The last things that ran come
//! back too, so the line above the input still says what ran and how long ago.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use gpui::Context;
use sk_one_window::{Event, Key, Lessons, Row, ViewOp};

use crate::intent::Plan;
use crate::ScriptListApp;

fn lessons_path() -> PathBuf {
    crate::setup::get_kit_path().join("db/one-window-lessons.json")
}

fn runs_path() -> PathBuf {
    crate::setup::get_kit_path().join("db/one-window-runs.json")
}

/// How many of the last runs the next session remembers.
const RUNS_REMEMBERED: usize = 50;

/// The last runs as written to disk, with the wall-clock time they were written so the next
/// session can tell how long ago each ran.
#[derive(serde::Serialize, serde::Deserialize)]
struct RememberedRuns {
    saved_at_ms: u64,
    entries: Vec<sk_one_window::Entry>,
}

fn wall_clock_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}

/// What changes when the ledger gains, settles or renames an entry: its newest entry's id,
/// state, title and saved-script name.
fn runs_fingerprint(window: &sk_one_window::OneWindow) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for entry in window.ledger().entries().iter().rev().take(2) {
        entry.id.hash(&mut hasher);
        format!("{:?}", entry.state).hash(&mut hasher);
        entry.title.hash(&mut hasher);
        entry.script.hash(&mut hasher);
    }
    hasher.finish()
}

/// Plans of the sentences learned this session, by learned row key.
#[derive(Default)]
pub(crate) struct LearnedPlans(HashMap<String, Arc<Plan>>);

impl ScriptListApp {
    /// Loads what earlier sessions learned and ran, once.
    pub(crate) fn one_window_load_lessons(&mut self, cx: &mut Context<Self>) {
        let learns = &mut self.one_window_host.learns;
        if learns.lessons_loaded {
            return;
        }
        learns.lessons_loaded = true;
        if !self.main_services.is_production() {
            return;
        }
        learns.saved_file =
            std::fs::read_to_string(super::saved_scripts::saved_path()).unwrap_or_default();
        learns.stopped = super::automations::load_off();
        let lessons = std::fs::read_to_string(lessons_path())
            .ok()
            .and_then(|text| serde_json::from_str::<Lessons>(&text).ok());
        if let Some(lessons) = lessons {
            let window = std::mem::take(&mut self.one_window);
            self.one_window = window.with_lessons(lessons);
        }
        let runs = std::fs::read_to_string(runs_path())
            .ok()
            .and_then(|text| serde_json::from_str::<RememberedRuns>(&text).ok());
        if let Some(runs) = runs {
            let now = self.one_window_now(cx);
            let since_saved = wall_clock_ms().saturating_sub(runs.saved_at_ms);
            let history = runs
                .entries
                .into_iter()
                .map(|entry| sk_one_window::Entry {
                    at: now,
                    before: entry.before.saturating_add(since_saved),
                    ..entry
                })
                .collect();
            let window = std::mem::take(&mut self.one_window);
            self.one_window = window.with_history(history);
        }
        self.one_window_host.learns.runs_fingerprint = runs_fingerprint(&self.one_window);
    }

    /// Something ran or settled: the next session will remember it. Written off the UI thread,
    /// and only in production.
    pub(crate) fn one_window_remember_runs(
        &mut self,
        now: sk_one_window::Ms,
        cx: &mut Context<Self>,
    ) {
        if !self.main_services.is_production() || !self.one_window_host.learns.lessons_loaded {
            return;
        }
        let fingerprint = runs_fingerprint(&self.one_window);
        if fingerprint == self.one_window_host.learns.runs_fingerprint {
            return;
        }
        self.one_window_host.learns.runs_fingerprint = fingerprint;
        let runs = RememberedRuns {
            saved_at_ms: wall_clock_ms(),
            entries: self.one_window.ledger().remembered(now, RUNS_REMEMBERED),
        };
        cx.background_executor()
            .spawn(async move {
                let path = runs_path();
                let written = serde_json::to_string(&runs)
                    .map_err(std::io::Error::other)
                    .and_then(|json| {
                        if let Some(dir) = path.parent() {
                            std::fs::create_dir_all(dir)?;
                        }
                        std::fs::write(&path, json)
                    });
                if let Err(error) = written {
                    tracing::warn!(target: "script_kit::one_window", %error, "runs_not_saved");
                }
            })
            .detach();
    }

    /// Something new was learned: keep its plan for this session and the lesson for the next.
    pub(crate) fn one_window_learned(&mut self) {
        if let Some(lesson) = self.one_window.lessons().learned().first() {
            if let Some(plan) = self.intent_launcher.plan(&lesson.said, &lesson.row.key) {
                self.one_window_host
                    .learns
                    .plans
                    .0
                    .insert(lesson.key(), plan);
            }
        }
        if !self.main_services.is_production() {
            return;
        }
        let path = lessons_path();
        let written = serde_json::to_string(self.one_window.lessons())
            .map_err(std::io::Error::other)
            .and_then(|json| {
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir)?;
                }
                std::fs::write(&path, json)
            });
        if let Err(error) = written {
            tracing::warn!(target: "script_kit::one_window", %error, "lessons_not_saved");
        }
    }

    /// The owned evaluator starts with one sentence already learned and one run already saved,
    /// the state a user reaches after running them once; neither shows until its words are
    /// typed.
    pub(crate) fn one_window_seed_owned_fixture(&mut self) {
        let row = Row::new(
            "jev:window:Move Google Chrome to Next Display",
            "Move Google Chrome to Next Display",
        )
        .with_icon("window");
        let lessons = Lessons::new(vec![sk_one_window::Lesson {
            said: "put the pr on my second screen".to_string(),
            row,
            at: 0,
        }]);
        let window = std::mem::take(&mut self.one_window);
        self.one_window = window.with_lessons(lessons);
        let name = "Tell Sarah + Post in #launcher";
        let said = "tell sarah the beta slipped and post in #launcher that it ships in two weeks";
        let body = [
            "await sendMessage({ to: \"Sarah\", text: \"The beta slipped a week.\" });".to_string(),
            "await postToSlack({ channel: \"#launcher\", text: \"The beta ships in two weeks.\" });"
                .to_string(),
        ];
        let learns = &mut self.one_window_host.learns;
        learns.lessons_loaded = true;
        learns.saved_file = super::saved_scripts::with_entry(
            "",
            name,
            &super::saved_scripts::entry_source(name, said, &body),
            &[],
        );
        let microphone =
            |id: &str, name: &str, is_default: bool| crate::dictation::DictationDeviceInfo {
                id: crate::dictation::DictationDeviceId(id.to_string()),
                name: name.to_string(),
                is_default,
                transport: crate::dictation::DictationDeviceTransport::BuiltIn,
            };
        learns.fixture_microphones = Some(vec![
            microphone("fixture-built-in", "MacBook Pro Microphone", true),
            microphone("fixture-studio", "Studio Display Microphone", false),
        ]);
        let dictations = [
            "move the design review to thursday afternoon",
            "remind me to send the invoice before friday",
            "the beta ships in two weeks",
        ]
        .map(|words| {
            crate::dictation::build_history_entry(
                words,
                std::time::Duration::from_secs(3),
                crate::dictation::DictationTarget::MainWindowFilter,
            )
        });
        if let Err(error) = crate::dictation::seed_owned_dictation_history(&dictations) {
            tracing::warn!(target: "script_kit::one_window", %error, "fixture_history_not_seeded");
        }
    }

    pub(super) fn one_window_run_learned(
        &mut self,
        row: &Row,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        let lesson = self
            .one_window
            .lesson(&row.key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("one_window_lesson_missing"))?;
        if let Some(plan) = self.one_window_host.learns.plans.0.get(&row.key).cloned() {
            return self
                .execute_intent_plan(&plan, cx)
                .map_err(anyhow::Error::msg);
        }
        self.one_window_handle(Event::View(ViewOp::Home { text: lesson.said }), cx);
        self.one_window_key(Key::Enter, cx);
        Ok(())
    }
}
