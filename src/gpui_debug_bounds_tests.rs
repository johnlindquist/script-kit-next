use std::{cell::Cell, rc::Rc};

use gpui::{
    div, point, px, size, AnyView, AppContext as _, Context, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, StyleRefinement, Styled, TestAppContext, Window,
};

struct CachedSelectorChild;

impl Render for CachedSelectorChild {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size(px(20.0))
            .flex_none()
            .debug_selector(|| "cached-selector-child".to_string())
    }
}

struct FidelitySelectorView {
    muted: bool,
}

#[derive(Clone)]
struct FidelityScrollbarHandle {
    offset: Rc<Cell<gpui::Point<gpui::Pixels>>>,
}

impl FidelityScrollbarHandle {
    fn new() -> Self {
        Self {
            offset: Rc::new(Cell::new(point(px(0.), px(0.)))),
        }
    }
}

impl gpui_component::scroll::ScrollbarHandle for FidelityScrollbarHandle {
    fn offset(&self) -> gpui::Point<gpui::Pixels> {
        self.offset.get()
    }

    fn set_offset(&self, offset: gpui::Point<gpui::Pixels>) {
        self.offset.set(offset);
    }

    fn content_size(&self) -> gpui::Size<gpui::Pixels> {
        size(px(100.), px(1_000.))
    }
}

struct FidelityScrollbarView {
    scrollbar_handle: FidelityScrollbarHandle,
}

struct FidelityTextView {
    state: Entity<gpui_component::text::TextViewState>,
}

impl Render for FidelityTextView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(240.)).h(px(120.)).child(
            gpui_component::text::TextView::new(&self.state)
                .selectable(true)
                .fidelity_scope("agent-chat.test.text"),
        )
    }
}

impl Render for FidelityScrollbarView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().relative().size_full().child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w(px(100.))
                .h(px(120.))
                .child(
                    gpui_component::scroll::Scrollbar::vertical(&self.scrollbar_handle)
                        .scrollbar_show(gpui_component::scroll::ScrollbarShow::Hover)
                        .fidelity_scope("agent-chat.test.scrollbar"),
                ),
        )
    }
}

impl Render for FidelitySelectorView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "agent-chat-test-parent".to_string())
            .size_full()
            .opacity(if self.muted { 0.5 } else { 1.0 })
            .bg(gpui::rgb(0x101010))
            .child(
                div()
                    .debug_selector(|| "agent-chat-test-child".to_string())
                    .w(px(40.0))
                    .h(px(20.0))
                    .bg(gpui::rgb(0xf0f0f0)),
            )
    }
}

struct CachedSelectorParent {
    child: gpui::Entity<CachedSelectorChild>,
    show_child: bool,
    revision: usize,
}

impl Render for CachedSelectorParent {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let parent = div().child(self.revision.to_string()).child(
            div()
                .hidden()
                .debug_selector(|| "display-none-selector-child".to_string()),
        );
        if self.show_child {
            parent.child(
                div()
                    .size(px(10.0))
                    .overflow_hidden()
                    .child(AnyView::from(self.child.clone()).cached(StyleRefinement::default())),
            )
        } else {
            parent
        }
    }
}

#[gpui::test]
fn selector_bounds_replay_cached_paint_and_drop_hidden_or_removed_nodes(cx: &mut TestAppContext) {
    let window = cx.update(|cx| {
        let child = cx.new(|_| CachedSelectorChild);
        cx.open_window(Default::default(), |_, cx| {
            cx.new(|_| CachedSelectorParent {
                child,
                show_child: true,
                revision: 0,
            })
        })
        .unwrap()
    });

    let initial_generation = window
        .update(cx, |_, window, _| {
            assert!(window.debug_bounds().contains_key("cached-selector-child"));
            assert!(!window
                .debug_bounds()
                .contains_key("display-none-selector-child"));
            assert_eq!(
                window
                    .debug_bounds_entries()
                    .iter()
                    .filter(|entry| entry.selector == "cached-selector-child")
                    .count(),
                1,
            );
            let entry = window
                .debug_bounds_entries()
                .iter()
                .find(|entry| entry.selector == "cached-selector-child")
                .unwrap();
            assert_eq!(entry.bounds.size, gpui::size(px(20.0), px(20.0)));
            assert_eq!(entry.visible_bounds.size, gpui::size(px(10.0), px(10.0)));
            assert_eq!(entry.clip_bounds.size, gpui::size(px(10.0), px(10.0)));
            window.rendered_frame_generation()
        })
        .unwrap();

    window
        .update(cx, |parent, _, cx| {
            parent.revision += 1;
            cx.notify();
        })
        .unwrap();
    cx.run_until_parked();
    window
        .update(cx, |_, window, _| {
            assert!(window.debug_bounds().contains_key("cached-selector-child"));
            assert_eq!(
                window
                    .debug_bounds_entries()
                    .iter()
                    .filter(|entry| entry.selector == "cached-selector-child")
                    .count(),
                1,
            );
            assert!(window.rendered_frame_generation() > initial_generation);
        })
        .unwrap();

    window
        .update(cx, |parent, _, cx| {
            parent.show_child = false;
            cx.notify();
        })
        .unwrap();
    cx.run_until_parked();
    window
        .update(cx, |_, window, _| {
            assert!(!window.debug_bounds().contains_key("cached-selector-child"));
        })
        .unwrap();
}

#[gpui::test]
fn fidelity_paint_telemetry_is_opt_in_nested_and_payload_sensitive(cx: &mut TestAppContext) {
    let window = cx.update(|cx| {
        cx.open_window(Default::default(), |_, cx| {
            cx.new(|_| FidelitySelectorView { muted: false })
        })
        .unwrap()
    });

    window
        .update(cx, |_, window, cx| {
            assert!(!window.fidelity_capture_active());
            assert!(window.fidelity_scope_summaries().is_empty());
            assert!(window.fidelity_paint_atoms().is_empty());
            window.set_fidelity_capture_target_for_test(Some("agent-chat"));
            cx.notify();
        })
        .unwrap();
    cx.run_until_parked();

    let (parent_digest, child_digest) = window
        .update(cx, |_, window, _| {
            let summaries = window.fidelity_scope_summaries();
            let parent = summaries
                .iter()
                .find(|scope| scope.id == "agent-chat-test-parent")
                .expect("parent fidelity scope");
            let child = summaries
                .iter()
                .find(|scope| scope.id == "agent-chat-test-child")
                .expect("child fidelity scope");
            let parent_atom_count = window
                .fidelity_paint_atoms()
                .iter()
                .filter(|atom| atom.scope_id == parent.id)
                .count();
            let child_atom_count = window
                .fidelity_paint_atoms()
                .iter()
                .filter(|atom| atom.scope_id == child.id)
                .count();

            assert_eq!(parent.parent_id, None);
            assert_eq!(child.parent_id.as_deref(), Some(parent.id.as_str()));
            assert!(parent_atom_count > 0);
            assert!(child_atom_count > 0);
            assert_eq!(parent.primitive_count, parent_atom_count);
            assert_eq!(child.primitive_count, child_atom_count);
            assert!(parent.first_paint_order.is_some());
            assert!(parent.last_paint_order.is_some());
            assert!(child.first_paint_order.is_some());
            assert!(child.last_paint_order.is_some());
            assert!(parent.first_paint_order <= parent.last_paint_order);
            assert!(child.first_paint_order <= child.last_paint_order);
            assert!(window
                .fidelity_paint_atoms()
                .iter()
                .all(|atom| atom.scope_id != "__unscoped__"));

            (
                parent.primitive_digest.clone(),
                child.primitive_digest.clone(),
            )
        })
        .unwrap();

    window
        .update(cx, |view, _, cx| {
            view.muted = true;
            cx.notify();
        })
        .unwrap();
    cx.run_until_parked();
    window
        .update(cx, |_, window, _| {
            let summaries = window.fidelity_scope_summaries();
            let parent = summaries
                .iter()
                .find(|scope| scope.id == "agent-chat-test-parent")
                .expect("muted parent fidelity scope");
            let child = summaries
                .iter()
                .find(|scope| scope.id == "agent-chat-test-child")
                .expect("muted child fidelity scope");
            assert_ne!(parent.primitive_digest, parent_digest);
            assert_ne!(child.primitive_digest, child_digest);
        })
        .unwrap();

    window
        .update(cx, |_, window, cx| {
            window.set_fidelity_capture_target_for_test(None);
            cx.notify();
        })
        .unwrap();
    cx.run_until_parked();
    window
        .update(cx, |_, window, _| {
            assert!(window.fidelity_scope_summaries().is_empty());
            assert!(window.fidelity_paint_atoms().is_empty());
        })
        .unwrap();
}

#[gpui::test]
fn fidelity_scrollbar_capture_uses_real_frozen_prepaint_state(cx: &mut TestAppContext) {
    let window = cx.update(|cx| {
        gpui_component::init(cx);
        cx.open_window(Default::default(), |_, cx| {
            cx.new(|_| FidelityScrollbarView {
                scrollbar_handle: FidelityScrollbarHandle::new(),
            })
        })
        .unwrap()
    });

    window
        .update(cx, |_, window, cx| {
            window.set_fidelity_capture_target_for_test(Some("agent-chat"));
            cx.notify();
        })
        .unwrap();
    cx.run_until_parked();

    window
        .update(cx, |_, window, _| {
            let summaries = window.fidelity_scope_summaries();
            let axis = summaries
                .iter()
                .find(|scope| scope.id == "agent-chat.test.scrollbar.vertical")
                .expect("vertical scrollbar fidelity scope");
            let track = summaries
                .iter()
                .find(|scope| scope.id == "agent-chat.test.scrollbar.vertical.track")
                .expect("scrollbar track fidelity scope");
            let thumb = summaries
                .iter()
                .find(|scope| scope.id == "agent-chat.test.scrollbar.vertical.thumb")
                .expect("scrollbar thumb fidelity scope");
            assert_eq!(axis.kind, gpui::FidelityNodeKind::Scrollbar);
            assert_eq!(track.kind, gpui::FidelityNodeKind::Scrollbar);
            assert_eq!(thumb.kind, gpui::FidelityNodeKind::Scrollbar);
            assert_eq!(axis.parent_id, None);
            assert_eq!(track.parent_id.as_deref(), Some(axis.id.as_str()));
            assert_eq!(thumb.parent_id.as_deref(), Some(axis.id.as_str()));
            assert!(axis.primitive_count > 0);
            assert!(track.primitive_count > 0);
            assert!(thumb.primitive_count > 0);

            for (scope, part) in [(axis, "axis"), (track, "track"), (thumb, "thumb")] {
                let metadata = scope.metadata.as_ref().expect("scrollbar metadata");
                assert_eq!(metadata["part"], part);
                assert_eq!(metadata["axis"], "vertical");
                assert_eq!(metadata["showMode"], "always");
                assert_eq!(metadata["configuredShowMode"], "hover");
                assert_eq!(metadata["captureFrozen"], true);
                assert_eq!(metadata["hovered"], false);
                assert_eq!(metadata["hoveredOnThumb"], false);
                assert_eq!(metadata["dragged"], false);
                assert_eq!(metadata["thumbPainted"], true);
                assert!(metadata["barBounds"]["height"].as_f64().unwrap() > 0.0);
                assert!(metadata["thumbFillBounds"]["height"].as_f64().unwrap() > 0.0);
            }
        })
        .unwrap();
}

#[gpui::test]
fn fidelity_textview_capture_hashes_exact_source_without_retaining_it(cx: &mut TestAppContext) {
    const SOURCE: &str = "abc";
    const SOURCE_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    let window = cx.update(|cx| {
        gpui_component::init(cx);
        cx.open_window(Default::default(), |_, cx| {
            let state = cx.new(|cx| {
                gpui_component::text::TextViewState::markdown_for_fidelity_test(SOURCE, cx)
            });
            cx.new(|_| FidelityTextView { state })
        })
        .unwrap()
    });

    window
        .update(cx, |_, window, cx| {
            window.set_fidelity_capture_target_for_test(Some("agent-chat"));
            cx.notify();
        })
        .unwrap();
    cx.run_until_parked();

    window
        .update(cx, |_, window, _| {
            let summaries = window.fidelity_scope_summaries();
            let document = summaries
                .iter()
                .find(|scope| scope.id == "agent-chat.test.text/document")
                .expect("TextView document fidelity scope");
            assert_eq!(document.kind, gpui::FidelityNodeKind::TextDocument);
            assert_eq!(document.text_hash.as_deref(), Some(SOURCE_SHA256));
            assert!(document.text_layout_hash.is_none());
            assert!(document.primitive_count > 0);

            let metadata = document.metadata.as_ref().expect("text metadata");
            assert_eq!(metadata["sourceByteLength"], SOURCE.len());
            assert_eq!(metadata["selectable"], true);
            assert_eq!(metadata["scrollable"], false);
            assert_eq!(metadata["textHashMode"], "exact-source-bytes");
            assert!(!metadata.to_string().contains(SOURCE));

            assert!(window
                .fidelity_paint_atoms()
                .iter()
                .all(|atom| !atom.payload_hash.contains(SOURCE)));
        })
        .unwrap();
}
