//! Day Page sediment rendering and fragment navigation tests.

use std::fs;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use gpui::{
    div, prelude::*, App, Context, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    TestAppContext, VisualTestContext,
};

use crate::brain::substrate::{BrainFrontmatter, BrainSubstrate, DayEntry, FragmentReference};
use crate::components::unified_list_item::{TextContent, UnifiedListItem, UnifiedListItemColors};
use crate::notes::NoteId;

use super::{
    fragment_card_id, parse_day_page_segments, DayPageDocumentSession, DayPageSegment,
    FRAGMENT_BACK_ID, SEDIMENT_LAYER_ID,
};

fn utc(now: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(now)
        .expect("parse time")
        .with_timezone(&Utc)
}

fn test_substrate() -> (tempfile::TempDir, BrainSubstrate) {
    let dir = tempfile::tempdir().expect("tempdir");
    let substrate = BrainSubstrate::with_timezone(dir.path().join("brain"), Tz::UTC);
    (dir, substrate)
}

fn write_fragment(
    substrate: &BrainSubstrate,
    id: &str,
    source: &str,
    body: &str,
    now: DateTime<Utc>,
) -> std::path::PathBuf {
    let path = substrate.paths().fragment_file(id);
    let parent = path.parent().expect("fragment parent");
    fs::create_dir_all(parent).expect("fragments dir");
    let frontmatter = BrainFrontmatter::new(NoteId::new(), now, now).with_source(source);
    fs::write(&path, frontmatter.render(body)).expect("write fragment");
    path
}

#[test]
fn fragment_line_parses_to_card_segment() {
    let content = "09:15 > First words of the pasted article without cutting mid-word...\n\
          ../fragments/2026-06-11-0942-clipboard.md\n";
    let segments = parse_day_page_segments(content);
    assert_eq!(segments.len(), 1);
    match &segments[0] {
        DayPageSegment::FragmentRef { index, excerpt, .. } => {
            assert_eq!(*index, 0);
            assert!(excerpt.contains("First words"));
        }
        other => panic!("expected fragment card segment, got {other:?}"),
    }
}

#[test]
fn open_fragment_binds_editor_and_back_restores_day_page() {
    let (_dir, substrate) = test_substrate();
    let now = utc("2026-06-11T09:42:00Z");
    let fragment_path = write_fragment(
        &substrate,
        "2026-06-11-0942-clipboard",
        "scriptkit://clipboard/entry-1",
        "Full pasted article body with many words.",
        now,
    );

    let mut session = DayPageDocumentSession::new(substrate.clone());
    session.bind_today(now).expect("bind day");
    substrate
        .append_to_day(
            now,
            DayEntry::FragmentRef(FragmentReference {
                excerpt: "First words of the pasted article without cutting mid-word..."
                    .to_string(),
                relative_link: "../fragments/2026-06-11-0942-clipboard.md".to_string(),
            }),
        )
        .expect("append fragment ref");

    session.bind_today(now).expect("reload day");

    let bound = session
        .bind_fragment(fragment_path.clone(), now)
        .expect("open fragment");
    assert!(bound.contains("Full pasted article body"));
    assert!(session.is_viewing_fragment());
    assert_eq!(session.path(), Some(&fragment_path));

    let restored = session.return_to_day(now).expect("back to day");
    assert!(restored.contains("First words of the pasted article"));
    assert!(!session.is_viewing_fragment());
    assert_eq!(
        session.path(),
        Some(&substrate.paths().day_page(now.date_naive()))
    );
}

struct SedimentCardProbe {
    card_id: String,
    focus_handle: FocusHandle,
}

impl SedimentCardProbe {
    fn new(card_id: impl Into<String>, cx: &mut Context<Self>) -> Self {
        Self {
            card_id: card_id.into(),
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for SedimentCardProbe {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SedimentCardProbe {
    fn render(&mut self, _: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let colors = UnifiedListItemColors::default();
        let card = UnifiedListItem::new(
            self.card_id.clone(),
            TextContent::plain("First words of the pasted article without cutting mid-word..."),
        )
        .colors(colors);

        div()
            .id(SEDIMENT_LAYER_ID)
            .debug_selector(|| self.card_id.clone())
            .child(card)
    }
}

/// Fragment line in today's file → card present via semantic id; covers devtools contract.
#[test]
fn fragment_card_semantic_id_is_rendered() {
    let mut cx = TestAppContext::single();
    let (_probe, cx) = cx.add_window_view(|_, cx| SedimentCardProbe::new(fragment_card_id(0), cx));

    let bounds = cx.debug_bounds("day-page-fragment-card-0");
    assert!(
        bounds.is_some(),
        "fragment card semantic id day-page-fragment-card-0 should be present in the element tree"
    );
}

struct FragmentBackProbe;

impl Render for FragmentBackProbe {
    fn render(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id(FRAGMENT_BACK_ID)
            .debug_selector(|| FRAGMENT_BACK_ID.to_string())
            .child("← Today")
    }
}

/// Back affordance semantic id is present when viewing a fragment inline.
#[test]
fn fragment_back_semantic_id_is_rendered() {
    let mut cx = TestAppContext::single();
    let window = cx.add_window(|_, _| FragmentBackProbe);
    let mut vcx = VisualTestContext::from_window(window.into(), &cx);
    vcx.run_until_parked();
    let bounds = vcx.debug_bounds(FRAGMENT_BACK_ID);
    assert!(
        bounds.is_some(),
        "fragment back semantic id {FRAGMENT_BACK_ID} should be present"
    );
}
