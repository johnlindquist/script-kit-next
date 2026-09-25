//! Rule for queued main-window animation replays, checked against generated
//! histories of shows, hides, replay commands, preset changes, and late
//! deliveries.
//!
//! The binary owns the event wiring, so this history mirrors it: every real
//! visibility transition advances the visibility generation, ordinary hides
//! and replay commands advance the replay generation, and a replay command
//! hides the window and queues one request. The rule is stated without the
//! counters: a queued replay may reopen the window exactly when the window is
//! still hidden, nothing the user did since superseded it, and the preferred
//! preset is still the one it would replay.

use gpui::proptest::prelude::*;
use gpui::proptest::test_runner::TestCaseResult;

use super::ShowWindowRequest;
use crate::test_utils::property::assert_rule;

const PRESETS: usize = crate::theme::animation::MainWindowAnimationPreset::ALL.len();

#[derive(Clone, Debug)]
enum Event {
    Show,
    Hide,
    /// "Replay Current Animation" or applying a preset from the launcher.
    Replay {
        preset: usize,
    },
    /// A config edit changes the preferred preset without replaying.
    PresetEdited {
        preset: usize,
    },
    /// A queued request arrives, possibly long after it was sent.
    Deliver(prop::sample::Index),
}

fn event() -> impl Strategy<Value = Event> {
    prop_oneof![
        2 => Just(Event::Show),
        2 => Just(Event::Hide),
        3 => (1..=PRESETS).prop_map(|preset| Event::Replay { preset }),
        1 => (1..=PRESETS).prop_map(|preset| Event::PresetEdited { preset }),
        4 => any::<prop::sample::Index>().prop_map(Event::Deliver),
    ]
}

struct Queued {
    request: ShowWindowRequest,
    preset: usize,
    superseded: bool,
}

struct World {
    visible: bool,
    visibility_generation: u64,
    replay_generation: u64,
    preset: usize,
    queue: Vec<Queued>,
}

impl World {
    fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.visibility_generation += 1;
            for queued in &mut self.queue {
                queued.superseded = true;
            }
        }
    }
}

/// Replay one history, checking every delivery against the rule.
fn check_history(events: Vec<Event>) -> TestCaseResult {
    let mut world = World {
        visible: true,
        visibility_generation: 1,
        replay_generation: 0,
        preset: 1,
        queue: Vec::new(),
    };
    for event in &events {
        match event {
            Event::Show => world.set_visible(true),
            Event::Hide => {
                if world.visible {
                    world.replay_generation += 1;
                    world.set_visible(false);
                }
            }
            Event::Replay { preset } => {
                if world.visible {
                    world.preset = *preset;
                    world.replay_generation += 1;
                    world.set_visible(false);
                    world.queue.push(Queued {
                        request: ShowWindowRequest::AnimationReplay {
                            visibility_generation: world.visibility_generation,
                            replay_generation: world.replay_generation,
                            preset_number: *preset,
                        },
                        preset: *preset,
                        superseded: false,
                    });
                }
            }
            Event::PresetEdited { preset } => world.preset = *preset,
            Event::Deliver(pick) => {
                if world.queue.is_empty() {
                    continue;
                }
                let queued = world.queue.remove(pick.index(world.queue.len()));
                let should_reopen =
                    !world.visible && !queued.superseded && queued.preset == world.preset;
                let reopens = queued.request.is_current(
                    world.visibility_generation,
                    world.visible,
                    world.replay_generation,
                    world.preset,
                );
                prop_assert_eq!(
                    reopens,
                    should_reopen,
                    "after {:?}: visible={} preset={} request preset={} superseded={}",
                    events,
                    world.visible,
                    world.preset,
                    queued.preset,
                    queued.superseded
                );
                if reopens {
                    world.set_visible(true);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn a_queued_replay_reopens_the_window_exactly_when_nothing_superseded_it() {
    assert_rule(
        "A queued replay reopens the window exactly when it is still current",
        1024,
        prop::collection::vec(event(), 1..40),
        check_history,
    );
}
