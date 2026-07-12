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
