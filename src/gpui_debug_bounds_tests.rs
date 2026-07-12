use gpui::{
    div, px, AnyView, AppContext as _, Context, InteractiveElement, IntoElement, ParentElement,
    Render, StyleRefinement, Styled, TestAppContext, Window,
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
