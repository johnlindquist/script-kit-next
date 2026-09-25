//! Stop, completion, and turn replacement, simulated under GPUI's seeded
//! scheduler.
//!
//! Each iteration drives a different random history through the real thread
//! and the provider-free fixture connection: submits, text sent to the live
//! turn and to old turns, Stops, completions, and scheduler ticks. The
//! iteration seed also reorders the thread's async stream tasks. A failure
//! prints the seed that replays it (`SEED=<n>`).

use super::*;
use crate::ai::agent_chat::ui::mock_fixture::create_mock_fixture_thread;

#[derive(Default)]
struct Turn {
    /// Every token the fixture accepted for this turn, and whether the turn
    /// was still live when it was sent.
    sent: Vec<(String, bool)>,
    live: bool,
    /// The turn's tokens in the transcript right after the user pressed Stop.
    frozen: Option<Vec<String>>,
}

fn token_turn(token: &str) -> usize {
    token
        .trim_start_matches("[t")
        .split('.')
        .next()
        .and_then(|turn| turn.parse().ok())
        .expect("well-formed token")
}

/// Tokens in one message body, in order.
fn tokens_in(body: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("[t") {
        let Some(length) = rest[start..].find(']') else {
            break;
        };
        tokens.push(rest[start..start + length + 1].to_string());
        rest = &rest[start + length + 1..];
    }
    tokens
}

fn assistant_bodies(thread: &AgentChatThread) -> Vec<String> {
    thread
        .messages
        .iter()
        .filter(|message| message.role == AgentChatThreadMessageRole::Assistant)
        .map(|message| message.body.to_string())
        .collect()
}

fn tokens_of_turn(thread: &AgentChatThread, turn: usize) -> Vec<String> {
    assistant_bodies(thread)
        .iter()
        .flat_map(|body| tokens_in(body))
        .filter(|token| token_turn(token) == turn)
        .collect()
}

/// Run one seeded history and check every rule against the final transcript.
fn simulate(cx: &mut gpui::TestAppContext) -> Result<(), String> {
    let (thread, control) = cx.update(|cx| {
        create_mock_fixture_thread("stop-simulation", AgentChatSessionPolicy::Full, cx)
    });
    let rng = cx.executor().rng();
    let mut turns: Vec<Turn> = Vec::new();
    let mut stopped_since_submit = false;
    let mut next_token = 0usize;
    let mut history: Vec<String> = Vec::new();

    for step in 0..40 {
        let roll: u32 = rng.random_range(0..100);
        match roll {
            0..=17 => {
                history.push("submit".into());
                stopped_since_submit = false;
                thread.update(cx, |thread, cx| {
                    thread.set_input(format!("ask {step}"), cx);
                    let _ = thread.submit_input(cx);
                });
            }
            18..=54 if !turns.is_empty() => {
                let index = if rng.random_bool(0.7) {
                    turns.len() - 1
                } else {
                    rng.random_range(0..turns.len())
                };
                let token = format!("[t{index}.{next_token}]");
                next_token += 1;
                history.push(format!("send {token}"));
                let live = turns[index].live;
                if control
                    .emit(
                        index as u64 + 1,
                        AgentChatEvent::AgentMessageDelta(format!("{token} ")),
                    )
                    .is_ok()
                {
                    turns[index].sent.push((token, live));
                }
            }
            55..=64 => {
                let streaming = thread.read_with(cx, |thread, _| {
                    matches!(thread.status, AgentChatThreadStatus::Streaming)
                });
                history.push(format!("stop (streaming={streaming})"));
                thread.update(cx, |thread, cx| thread.stop_streaming(cx));
                if streaming {
                    stopped_since_submit = true;
                    if let Some(index) = turns.iter().rposition(|turn| turn.live) {
                        turns[index].live = false;
                        let frozen =
                            thread.read_with(cx, |thread, _| tokens_of_turn(thread, index));
                        turns[index].frozen = Some(frozen);
                    }
                }
            }
            65..=72 => {
                if let Some(index) = turns.iter().rposition(|turn| turn.live) {
                    history.push(format!("complete t{index}"));
                    if control
                        .emit(index as u64 + 1, AgentChatEvent::completed("fixture"))
                        .is_ok()
                    {
                        turns[index].live = false;
                    }
                }
            }
            73..=86 => {
                history.push("park".into());
                cx.run_until_parked();
            }
            _ => {
                history.push("tick".into());
                cx.executor()
                    .advance_clock(std::time::Duration::from_millis(16));
                cx.run_until_parked();
            }
        }

        let started = control.receipts().unwrap().len();
        while turns.len() < started {
            if stopped_since_submit {
                return Err(format!(
                    "a turn started after Stop without a new submit\nhistory: {history:?}"
                ));
            }
            for turn in &mut turns {
                turn.live = false;
            }
            turns.push(Turn {
                live: true,
                ..Turn::default()
            });
            history.push(format!("turn t{} started", turns.len() - 1));
        }
    }

    for _ in 0..40 {
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(16));
        cx.run_until_parked();
    }

    thread.read_with(cx, |thread, _| {
        for body in assistant_bodies(thread) {
            let mut owners: Vec<usize> = tokens_in(&body).iter().map(|t| token_turn(t)).collect();
            owners.dedup();
            if owners.len() > 1 {
                return Err(format!(
                    "one reply mixes text from turns {owners:?}: {body:?}\nhistory: {history:?}"
                ));
            }
        }
        for (index, turn) in turns.iter().enumerate() {
            let present = tokens_of_turn(thread, index);
            for (token, live) in &turn.sent {
                if !live && present.contains(token) {
                    return Err(format!(
                        "{token} was sent after turn t{index} ended but still appeared\nhistory: {history:?}"
                    ));
                }
            }
            if let Some(frozen) = &turn.frozen {
                if &present != frozen {
                    return Err(format!(
                        "turn t{index} changed after Stop: {frozen:?} became {present:?}\nhistory: {history:?}"
                    ));
                }
            }
        }
        Ok(())
    })
}

#[gpui::test(iterations = 64)]
fn stopped_and_replaced_turns_never_gain_text(cx: &mut gpui::TestAppContext) {
    if let Err(broken) = simulate(cx) {
        panic!("{broken}");
    }
}
