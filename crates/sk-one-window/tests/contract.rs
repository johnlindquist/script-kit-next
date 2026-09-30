//! The crate's numbers and words are the prototype's, as exported to `fixtures/contract.json`.

use serde_json::Value;
use sk_one_window::contract::{self, prototype, words};

fn load() -> Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/contract.json");
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{path}: {e} (copy it from the SSD fixtures)"));
    serde_json::from_str(&text).expect("contract.json parses")
}

fn num(v: &Value, path: &str) -> Option<f64> {
    path.split('.')
        .try_fold(v, |v, k| v.get(k))
        .and_then(Value::as_f64)
}

fn text<'a>(v: &'a Value, path: &str) -> Option<&'a str> {
    path.split('.')
        .try_fold(v, |v, k| v.get(k))
        .and_then(Value::as_str)
}

#[test]
fn timings_match_the_prototype() {
    let c = load();
    let expect = [
        ("timing.shrinkHoldMs", contract::SHRINK_HOLD_MS),
        ("timing.viewHoldMs", contract::VIEW_HOLD_MS),
        ("timing.enterGuardMs", contract::ENTER_GUARD_MS),
        ("timing.rowArmMs", contract::ROW_ARM_MS),
        ("timing.messageHoldMs", contract::MESSAGE_HOLD_MS),
        ("timing.jevDebounceMs", contract::JEV_DEBOUNCE_MS),
        ("timing.slotDimEaseMs", contract::SLOT_DIM_EASE_MS),
        ("timing.dismissFadeMs", contract::DISMISS_FADE_MS),
        ("timing.dismissSettleMs", contract::DISMISS_SETTLE_MS),
        ("timing.summonFadeMs", contract::SUMMON_FADE_MS),
        ("timing.holdTickMs", contract::HOLD_TICK_MS),
    ];
    let mut checked = 0;
    for (path, ours) in expect {
        if let Some(theirs) = num(&c, path) {
            assert_eq!(theirs as u64, ours, "{path}");
            checked += 1;
        }
    }
    assert!(
        checked >= 6,
        "contract.json has the timing block ({checked} checked)"
    );
}

#[test]
fn counts_and_geometry_match_the_prototype() {
    let c = load();
    if let Some(n) = num(&c, "counts.jevRows") {
        assert_eq!(n as usize, contract::JEV_ROWS);
    }
    if let Some(n) = num(&c, "counts.maxMatches") {
        assert_eq!(n as usize, contract::MAX_MATCHES);
    }
    let thresholds = [
        ("thresholds.askLeadP", contract::ASK_LEAD_P),
        ("thresholds.jevRowMinP", contract::MIN_READING_P),
        ("thresholds.weakMatchScore", f32::from(contract::WEAK_SCORE)),
    ];
    for (path, ours) in thresholds {
        let theirs = num(&c, path).unwrap_or_else(|| panic!("{path} in contract.json"));
        assert_eq!(theirs as f32, ours, "{path}");
    }
    let geometry = [
        ("geometry.prototype.inputRowPx", prototype::INPUT_ROW_PX),
        ("geometry.prototype.slotLinePx", prototype::SLOT_LINE_PX),
        ("geometry.prototype.listHeaderPx", prototype::LIST_HEADER_PX),
        ("geometry.prototype.rowPx", prototype::ROW_PX),
        ("geometry.prototype.tilePx", prototype::TILE_PX),
        ("geometry.prototype.tileRadiusPx", prototype::TILE_RADIUS_PX),
        ("geometry.prototype.capsulePx", prototype::CAPSULE_PX),
        ("geometry.prototype.capsuleGapPx", prototype::CAPSULE_GAP_PX),
        (
            "geometry.prototype.capsuleInsetPx",
            prototype::CAPSULE_INSET_PX,
        ),
        ("geometry.prototype.footerZonePx", prototype::FOOTER_ZONE_PX),
        (
            "geometry.prototype.capsuleRadiusPx",
            prototype::CAPSULE_RADIUS_PX,
        ),
        ("geometry.prototype.keycapPx", prototype::KEYCAP_PX),
        (
            "geometry.prototype.windowRadiusPx",
            prototype::WINDOW_RADIUS_PX,
        ),
        (
            "geometry.prototype.windowWidthPx",
            prototype::WINDOW_WIDTH_PX,
        ),
    ];
    let opacity = [
        ("slot.opacity.focused", contract::slot_opacity::FOCUSED),
        ("slot.opacity.recent", contract::slot_opacity::RECENT),
        (
            "slot.opacity.typingFocused",
            contract::slot_opacity::TYPING_FOCUSED,
        ),
        (
            "slot.opacity.typingRecent",
            contract::slot_opacity::TYPING_RECENT,
        ),
        (
            "slot.opacity.inViewNotHolding",
            contract::slot_opacity::IN_VIEW_NOT_HOLDING,
        ),
    ];
    for (path, ours) in opacity {
        let theirs = num(&c, path).unwrap_or_else(|| panic!("{path} in contract.json"));
        assert_eq!(theirs as f32, ours, "{path}");
    }
    for (path, ours) in geometry {
        if let Some(theirs) = num(&c, path) {
            assert_eq!(theirs as f32, ours, "{path}");
        }
    }
    if let Some(tile) = text(&c, "color.tile") {
        let rgb = u32::from_str_radix(tile.trim_start_matches('#'), 16).expect("hex color");
        assert_eq!(rgb, contract::TILE_RGB);
    }
    if let Some(alpha) = num(&c, "color.tileAlpha") {
        assert_eq!((alpha * 255.0).round() as u8, contract::TILE_ALPHA);
    }
}

#[test]
fn words_match_the_prototype() {
    let c = load();
    let expect = [
        ("headers.empty", words::SUGGESTED),
        ("headers.matches", words::MATCHES),
        ("headers.noMatches", words::NO_MATCHES),
        ("headers.pinned", words::PINNED),
        ("headers.jev", words::JEV),
        ("headers.staged", words::STAGED),
        ("headers.share", words::SHARE_HOW),
        ("enterLabels.waiting", words::ENTER_WAITING),
        ("enterLabels.jevFirst", words::ENTER_JEV_FIRST),
        ("enterLabels.wait", words::ENTER_WAIT),
        ("enterLabels.nothing", words::ENTER_NOTHING),
        ("slot.labels.focused", words::FOCUSED),
        ("slot.labels.recent", words::LAST),
        ("slot.labels.staged", words::ACTING_ON),
        ("slot.labels.paused", words::PAUSED),
    ];
    let mut checked = 0;
    for (path, ours) in expect {
        if let Some(theirs) = text(&c, path) {
            assert_eq!(theirs, ours, "{path}");
            checked += 1;
        }
    }
    assert!(
        checked >= 10,
        "contract.json has headers, Enter labels and slot labels ({checked} checked)"
    );
    if let Some(holding) = text(&c, "slot.labels.holding") {
        assert_eq!(holding.replace("{s}", "8"), words::sending_in(8));
    }
    let ask = text(&c, "enterLabels.askOnce").expect("enterLabels.askOnce in contract.json");
    assert_eq!(ask.replace("{title}", "Lunch"), words::ask_again("Lunch"));
}

#[test]
fn undoable_icons_match_the_prototype() {
    let c = load();
    let Some(icons) = c.pointer("/risk/undoableIcons").and_then(Value::as_array) else {
        panic!("contract.json lists risk.undoableIcons");
    };
    let mut theirs: Vec<&str> = icons.iter().filter_map(Value::as_str).collect();
    let mut ours = contract::UNDOABLE_ICONS.to_vec();
    theirs.sort_unstable();
    ours.sort_unstable();
    assert_eq!(theirs, ours);
}
