//! The Curator: a scheduled, headless distillation pass.
//!
//! Once a day (and on demand via `kit://brain/focus?refresh=1`), the curator
//! asks the user's existing pi/codex setup to distill the brain's recent
//! evidence — attention signals and activity journals — into a short
//! **Focus review**: what the user actually worked on, what's heating up,
//! what stalled. The review is stored as a brain doc, so it's recallable
//! ("what did I work on this week?") and feeds future context staging.
//!
//! Design constraints:
//! - Runs `pi -p --no-tools` as a one-shot subprocess (no RPC plumbing, no
//!   tool surface, nothing to go wrong beyond a text completion).
//! - Uses the same binary resolution and codex auth as Agent Chat — if the
//!   user never set up Agent Chat, the curator silently skips.
//! - Never blocks anything: invoked from the indexer thread.

use super::store::{self, DocSource};
use anyhow::{Context as _, Result};
use std::process::Command;
use std::time::Duration;

const LAST_RUN_MARKER: &str = "curator_last_run";
const RUN_INTERVAL_SECS: i64 = 24 * 60 * 60;
const PI_TIMEOUT: Duration = Duration::from_secs(120);
const FOCUS_SOURCE_ID: &str = "focus-review";

/// Run the curator if it's due. Called from the indexer cycle.
pub fn run_if_due() {
    let now = chrono::Utc::now().timestamp();
    let last = store::meta_get(LAST_RUN_MARKER)
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    if now - last < RUN_INTERVAL_SECS {
        return;
    }
    // Mark first so a crashing model call can't hot-loop.
    let _ = store::meta_set(LAST_RUN_MARKER, &now.to_string());
    match run_focus_review() {
        Ok(true) => {
            tracing::info!(target: "script_kit::brain", "curator wrote focus review")
        }
        Ok(false) => {}
        Err(error) => {
            tracing::debug!(target: "script_kit::brain", error = %error, "curator skipped");
        }
    }
}

/// Force a focus review now (e.g. kit://brain/focus?refresh=1).
pub fn run_focus_review() -> Result<bool> {
    let signals = store::recent_signals(200)?;
    let journals = recent_activity_journals(3)?;
    if signals.is_empty() && journals.is_empty() {
        return Ok(false); // Nothing to distill yet.
    }
    let Some(pi_binary) = crate::ai::agent_chat::pi::binary::default_pi_binary() else {
        return Ok(false); // No agent setup — curator waits.
    };

    let topics = super::search::aggregate_signals(&signals);
    let topics_block = topics
        .iter()
        .take(12)
        .map(|(topic, weight)| format!("- {topic} (weight {weight})"))
        .collect::<Vec<_>>()
        .join("\n");
    let journal_block = journals.join("\n\n");
    let today = chrono::Local::now().format("%Y-%m-%d");

    let prompt = format!(
        "You are the curator of a personal knowledge base. Distill the \
         evidence below into a focus review for {today}. Write 6-12 terse \
         lines of markdown: '## Current focus' (ranked topics with one-line \
         why), '## Recent activity' (what the user actually did, grouped), \
         '## Drifting' (topics with attention but no recent activity, if \
         any). Facts only — no advice, no filler, no preamble.\n\n\
         ATTENTION SIGNALS (topic, accumulated weight):\n{topics_block}\n\n\
         ACTIVITY JOURNALS (newest first):\n{journal_block}"
    );

    let output = run_pi_print(&pi_binary, &prompt)?;
    let review = output.trim();
    if review.is_empty() {
        return Ok(false);
    }
    store::upsert_doc(
        DocSource::Activity,
        FOCUS_SOURCE_ID,
        &format!("Focus review {today}"),
        review,
        chrono::Utc::now().timestamp(),
    )?;
    Ok(true)
}

/// The most recent N daily activity journals' contents.
fn recent_activity_journals(n: usize) -> Result<Vec<String>> {
    let mut journals = Vec::new();
    for back in 0..n {
        let day = (chrono::Local::now() - chrono::Duration::days(back as i64))
            .format("%Y-%m-%d")
            .to_string();
        if let Some(doc) = store::get_doc(DocSource::Activity, &format!("activity:{day}"))? {
            journals.push(format!("### {}\n{}", doc.title, doc.content));
        }
    }
    Ok(journals)
}

fn run_pi_print(pi_binary: &std::path::Path, prompt: &str) -> Result<String> {
    let mut child = Command::new(pi_binary)
        .args([
            "-p",
            "--no-tools",
            "--provider",
            crate::ai::agent_chat::profiles::DEFAULT_PI_PROVIDER,
            "--model",
            crate::ai::agent_chat::profiles::DEFAULT_PI_MODEL,
            prompt,
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("spawn curator pi")?;
    let deadline = std::time::Instant::now() + PI_TIMEOUT;
    loop {
        match child.try_wait().context("curator pi wait")? {
            Some(status) => {
                let mut output = String::new();
                if let Some(mut stdout) = child.stdout.take() {
                    use std::io::Read as _;
                    let _ = stdout.read_to_string(&mut output);
                }
                if !status.success() {
                    anyhow::bail!("curator pi exited with {status}");
                }
                return Ok(output);
            }
            None if std::time::Instant::now() > deadline => {
                let _ = child.kill();
                anyhow::bail!("curator pi timed out");
            }
            None => std::thread::sleep(Duration::from_millis(250)),
        }
    }
}
