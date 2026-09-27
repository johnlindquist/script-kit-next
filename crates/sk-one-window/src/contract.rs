//! Every number, word and key the one-window spec fixes.
//!
//! `fixtures/contract.json` is generated from the prototype (`.notes/jev-intent`), and
//! `tests/contract.rs` holds these constants to it. The prototype was drawn at video scale
//! (52 px rows); the app draws at native scale (44 px rows), so geometry that the app owns is
//! the prototype value times [`GEOMETRY_SCALE`], rounded, except where the app's existing Tahoe
//! token is the reference the prototype copied (the list header and the floating buttons).

/// Typing pauses this long before the window shrinks.
pub const SHRINK_HOLD_MS: u64 = 300;
/// A view that just opened keeps the window's height this long before it may shrink.
pub const VIEW_HOLD_MS: u64 = 300;
/// A shrink animates over this long.
pub const SHRINK_ANIM_MS: u64 = 260;
/// The shrink's cubic-bezier control points.
pub const SHRINK_CURVE: [f32; 4] = [0.4, 0.0, 0.2, 1.0];
/// Enter is ignored this long after its target changed without a keypress.
pub const ENTER_GUARD_MS: u64 = 400;
/// After a Jev row replaces another under the highlight, Enter still runs the old row this long.
pub const ROW_ARM_MS: u64 = 300;
/// What can't be taken back waits this long in the line above the input.
pub const MESSAGE_HOLD_MS: u64 = 8_000;
/// Jev is asked once the keys pause this long.
pub const JEV_DEBOUNCE_MS: u64 = 70;
/// Only the line's dimming eases, over this long.
pub const SLOT_DIM_EASE_MS: u64 = 180;
/// Launching an app fades the window out over this long.
pub const DISMISS_FADE_MS: u64 = 160;
/// Nothing in a window that is going away changes for this long; then the launch is recorded and
/// the window goes home.
pub const DISMISS_SETTLE_MS: u64 = 180;
/// Summoning fades the window in over this long.
pub const SUMMON_FADE_MS: u64 = 160;
/// How often the prototype redrew a waiting message's seconds (the app redraws on each second).
pub const HOLD_TICK_MS: u64 = 200;
/// Rows kept for Jev from the first key of a sentence.
pub const JEV_ROWS: usize = 3;
/// Name rows shown above Jev's.
pub const MAX_MATCHES: usize = 3;
/// A name scoring below this is listed but never owns Enter.
pub const WEAK_SCORE: u8 = 50;
/// Jev's readings less likely than this don't get a row.
pub const MIN_READING_P: f32 = 0.03;
/// An Enter that waited for Jev runs its row only if that row is Jev's top reading by at least
/// this much; otherwise it asks once.
pub const ASK_LEAD_P: f32 = 0.15;

/// App geometry per prototype pixel.
pub const GEOMETRY_SCALE: f32 = 44.0 / 52.0;

/// Geometry as the prototype draws it (video scale).
pub mod prototype {
    pub const INPUT_ROW_PX: f32 = 68.0;
    pub const SLOT_LINE_PX: f32 = 40.0;
    pub const LIST_HEADER_PX: f32 = 30.0;
    pub const ROW_PX: f32 = 52.0;
    pub const TILE_PX: f32 = 28.0;
    pub const TILE_RADIUS_PX: f32 = 9.0;
    pub const CAPSULE_PX: f32 = 32.0;
    pub const CAPSULE_GAP_PX: f32 = 6.0;
    pub const CAPSULE_INSET_PX: f32 = 8.0;
    pub const FOOTER_ZONE_PX: f32 = 48.0;
    pub const CAPSULE_RADIUS_PX: f32 = 8.0;
    pub const KEYCAP_PX: f32 = 22.0;
    pub const WINDOW_RADIUS_PX: f32 = 18.0;
    pub const WINDOW_WIDTH_PX: f32 = 860.0;
}

/// Geometry as the app draws it (native scale).
pub mod app {
    /// The line above the input: always present, one height.
    pub const SLOT_LINE_PX: f32 = 34.0;
    /// Every list's first header; the existing Tahoe section header height.
    pub const LIST_HEADER_PX: f32 = 28.0;
    pub const ROW_PX: f32 = 44.0;
    /// The focused row's icon tile.
    pub const TILE_PX: f32 = 24.0;
    pub const TILE_RADIUS_PX: f32 = 8.0;
}

/// The focused row's tile color (0xRRGGBB) and alpha byte (95%).
pub const TILE_RGB: u32 = 0xFB_BF_24;
pub const TILE_ALPHA: u8 = 0xF2;
/// Selected rows have no fill.
pub const ROW_FILL_ALPHA: u8 = 0;

/// How strongly the line above the input is drawn.
pub mod slot_opacity {
    /// What's focused and the last thing that ran are context, drawn dimmed.
    pub const FOCUSED: f32 = 0.8;
    pub const RECENT: f32 = 0.8;
    /// Dimmer still while words are typed: the list is what matters then.
    pub const TYPING_FOCUSED: f32 = 0.5;
    pub const TYPING_RECENT: f32 = 0.5;
    /// Inside a view, unless a message is waiting.
    pub const IN_VIEW_NOT_HOLDING: f32 = 0.45;
}

/// Words the window shows. Pages and the app use these and no others.
pub mod words {
    pub const FOCUSED: &str = "Focused";
    pub const LAST: &str = "Last";
    pub const ACTING_ON: &str = "Acting on";
    pub const SENDING: &str = "Sending";
    pub const SHARING: &str = "Sharing";
    pub const PAUSED: &str = "Paused while you type";
    pub const JUST_NOW: &str = "just now";
    pub const NOT_SENT: &str = "not sent";
    pub const SENT: &str = "sent";

    pub const SUGGESTED: &str = "Suggested";
    pub const MATCHES: &str = "Matches";
    pub const NO_MATCHES: &str = "No matches";
    pub const PINNED: &str = "Pinned";
    pub const JEV: &str = "Jev";
    pub const STAGED: &str = "Say what to do with it, or pick one";
    pub const SHARE_HOW: &str = "Share it how?";

    /// ⇥ made the agent the Owner: the header over the one row that hands it the sentence.
    pub const AGENT: &str = "Agent";
    pub const HAND_TO_AGENT: &str = "Hand it to an agent";
    pub const START_AGENT: &str = "Start the agent";
    /// The Owner ⇥ sets, as the input's pill and the reading name it.
    pub const OWNER_AGENT: &str = "agent";

    pub const ENTER_WAITING: &str = "Runs when Jev is sure…";
    pub const ENTER_JEV_FIRST: &str = "Runs what Jev understands";
    pub const ENTER_WAIT: &str = "Wait for Jev, then run it";
    pub const ENTER_NOTHING: &str = "Nothing yet";

    /// "Sending in 8s".
    pub fn sending_in(seconds: u64) -> String {
        format!("Sending in {seconds}s")
    }

    /// What ↵ says while it asks once: "Lunch with Sarah is at 1:00 PM — ↵ again".
    pub fn ask_again(title: &str) -> String {
        format!("{title} — ↵ again")
    }

    /// The note beside Jev's header: the keys that run its rows, "⌘1–3".
    pub fn jev_keys(rows: usize) -> String {
        format!("⌘1–{rows}")
    }

    /// Jev's first reserved row while it reads the words.
    pub const UNDERSTANDING: &str = "Understanding…";

    pub const CLIPBOARD: &str = "Clipboard";
}

/// Icons whose runs can be undone (the rest either wait or just happen).
pub const UNDOABLE_ICONS: &[&str] = &[
    "move", "schedule", "append", "again", "delegate", "steps", "triage", "learned", "edit",
    "change",
];

/// The capture targets the launcher knows by name (`todo;`, `;note …`). Scripts can register more,
/// which only the launcher's grammar knows.
pub const CAPTURE_TARGETS: &[&str] = &[
    "todo", "note", "link", "snippet", "cal", "social", "reminder", "snooze", "defer", "notes",
];

/// The one TypeScript file every saved run lands in.
pub const SAVED_SCRIPT_FILE: &str = "~/.scriptkit/plugins/main/scripts/saved.ts";
pub const SAVED_SCRIPT_FILE_NAME: &str = "saved.ts";

/// Jev's reading names ↵ once the keys have paused this long; keys closer together are one
/// sentence still being typed, and a reading of part of it never shows on the button.
pub const READING_SETTLE_MS: u64 = 200;

/// The colors of the words under the sentence (the spec's vocabulary), 0xRRGGBB.
pub mod marks {
    pub const SUBJECT: u32 = 0xFF_B8_6B;
    pub const ACTION: u32 = 0xF5_C5_42;
    pub const TARGET: u32 = 0x6F_DC_9B;
    pub const DETAILS: u32 = 0x8F_B8_FF;
    pub const OWNER: u32 = 0xC9_A7_FF;
    /// The reading's chips for the two words nobody types.
    pub const CONTEXT: u32 = 0x9A_A0_AD;
    pub const FOCUSED: u32 = 0xFF_9E_CB;
}

/// Rounds a prototype length to the app's scale.
pub fn to_app_px(prototype_px: f32) -> f32 {
    (prototype_px * GEOMETRY_SCALE).round()
}

/// The cubic-bezier easing of the shrink at progress `t` (0..=1), solved for x like CSS does.
pub fn shrink_ease(t: f32) -> f32 {
    let [x1, y1, x2, y2] = SHRINK_CURVE;
    let t = t.clamp(0.0, 1.0);
    let bez = |a: f32, b: f32, s: f32| {
        let u = 1.0 - s;
        3.0 * u * u * s * a + 3.0 * u * s * s * b + s * s * s
    };
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    let mut s = t;
    for _ in 0..40 {
        let x = bez(x1, x2, s);
        if (x - t).abs() < 1e-5 {
            break;
        }
        if x < t {
            lo = s;
        } else {
            hi = s;
        }
        s = (lo + hi) / 2.0;
    }
    bez(y1, y2, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_geometry_is_the_prototype_scaled_where_the_app_owns_it() {
        assert_eq!(to_app_px(prototype::ROW_PX), app::ROW_PX);
        assert_eq!(to_app_px(prototype::SLOT_LINE_PX), app::SLOT_LINE_PX);
        assert_eq!(to_app_px(prototype::TILE_PX), app::TILE_PX);
        assert_eq!(to_app_px(prototype::TILE_RADIUS_PX), app::TILE_RADIUS_PX);
    }

    #[test]
    fn tile_alpha_is_95_percent() {
        assert_eq!(TILE_ALPHA, (0.95f32 * 255.0).round() as u8);
    }

    #[test]
    fn shrink_ease_starts_ends_and_rises() {
        assert!(shrink_ease(0.0).abs() < 1e-3);
        assert!((shrink_ease(1.0) - 1.0).abs() < 1e-3);
        let mut prev = 0.0;
        for i in 1..=20 {
            let v = shrink_ease(i as f32 / 20.0);
            assert!(v >= prev - 1e-4, "ease must not go backwards");
            prev = v;
        }
    }
}

/// Jev first reads what's heard at this many words, then again at each clause end.
pub const TALK_FIRST_READ_WORDS: usize = 4;
/// Letting go of the voice key acts without asking only when Jev is at least this sure.
pub const TALK_SURE_P: f32 = 0.65;
/// A learned sentence matches once this many letters of it are typed.
pub const LEARNED_MIN_CHARS: usize = 2;
/// A learned row's strength with one letter typed; it rises to 100 as the sentence fills in.
pub const LEARNED_SCORE_FLOOR: u8 = 70;
/// Sentences kept as lessons.
pub const MAX_LESSONS: usize = 200;
/// A learned row's strength once two words of its sentence are typed: above a name that only
/// starts with the words or matches them at word boundaries.
pub const LEARNED_SENTENCE_SCORE: u8 = 96;

/// Words the voice view shows.
pub mod talk_words {
    pub const LISTENING: &str = "Listening…";
    pub const SOUNDS_LIKE_REQUEST: &str = "Sounds like a request";
    pub const PICKED_TYPING: &str = "You picked typing · Jev won't switch it";
    pub const PICKED_REQUEST: &str = "You picked a request · Jev won't switch it";
    pub const TOO_CLOSE: &str = "Too close to call · pick one";
    pub const PICK_ONE: &str = "Pick one · ⌘1 or ⌘2";
    pub const WAITS_FOR_JEV: &str = "Waits for Jev";
    pub const DO_IT_AS_REQUEST: &str = "Do it as a request";
    pub const LIKE_ONE_YOU_TYPE: &str = "a sentence, like one you type";
    pub const AT_YOUR_CURSOR: &str = "at your cursor";
    pub const TYPING_INTO: &str = "Typing into";
    pub const VIEW_LABEL: &str = "Listening";
    pub const SEND_TO_MODEL: &str = "Send it to the model";
    pub const ADD_TO_NOTE: &str = "Add it to today's note";
    pub const WHEN_YOU_LET_GO: &str = "what you say, when you let go of the key";

    /// "Talking to the model" or "Talking to today's note".
    pub fn talking_to(model: bool) -> &'static str {
        if model {
            "Talking to the model"
        } else {
            "Talking to today's note"
        }
    }

    /// "Sounds like you're writing to Sarah".
    pub fn writing_to(to: Option<&str>) -> String {
        match to {
            Some(to) => format!("Sounds like you're writing to {to}"),
            None => "Sounds like you're writing".to_string(),
        }
    }

    /// "Type it into Mail".
    pub fn type_into(app: &str) -> String {
        format!("Type it into {app}")
    }

    /// "Typed 20 words into Mail".
    pub fn typed_into(words: usize, app: &str) -> String {
        format!("Typed {words} words into {app}")
    }
}

/// Words for what waits, what can be taken back, what ran today, and what still needs setting up.
pub mod ledger_words {
    /// "Runs in 8s": something that isn't a message waits before it runs.
    pub fn runs_in(seconds: u64) -> String {
        format!("Runs in {seconds}s")
    }
    pub const RAN: &str = "ran";
    pub const NOT_RUN: &str = "stopped";
    pub const UNDO: &str = "undo";
    pub const TODAY: &str = "Today";
    pub const ACT_ON_IT: &str = "Act on it";
    pub const HISTORY_HEAD: &str = "What you ran today · newest first";
    pub const NEEDS_SETUP: &str = "Needs setup";
    pub const BUN_MISSING: &str = "bun is not installed";
    pub const BUN_INSTALL: &str = "Install from bun.sh";
    pub const BUN_URL: &str = "https://bun.sh";

    /// "Didn't send: To Sarah" / "Didn't run: Empty Trash".
    pub fn stopped(label: &str, run: bool) -> String {
        if run {
            format!("Didn't run: {label}")
        } else {
            format!("Didn't send: {label}")
        }
    }
}
