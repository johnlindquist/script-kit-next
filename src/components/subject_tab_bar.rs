//! Browser-shaped subject tabs. The active tab and its list rows share one panel fill;
//! the bottom rule belongs to the inactive tabs, never between the active tab and its rows.
use crate::one_window::slot_line::{render_hint, render_slot_icon};
use crate::{designs::MainMenuThemeDef, protocol::ElementInfo, theme::Theme};
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, SharedString};
use std::rc::Rc;

// T2 geometry, in GPUI points. Color is relative to the active theme in both appearances.
pub(crate) const HEIGHT: f32 = 38.;
pub(crate) const PANEL_INSET: f32 = 8.;
const TAB_WIDTH: f32 = 172.;
const TOP_RADIUS: f32 = 10.;
pub(crate) fn panel_color(theme: &Theme) -> gpui::Rgba {
    rgba((theme.colors.text.primary << 8) | 0x0e)
}

#[derive(Clone)]
pub(crate) struct TabItem {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub detail: String,
    pub age: String,
    pub active: bool,
    pub fresh: bool,
    pub hint: Option<&'static str>,
}
impl TabItem {
    pub fn semantic_id(&self) -> String {
        format!("subject-tab:{}", self.id)
    }
    pub fn element(&self, index: usize) -> ElementInfo {
        let mut e = ElementInfo::button(index, &self.label);
        e.semantic_id = self.semantic_id();
        e.role = Some("tab".into());
        e.selected = Some(self.active);
        e.kind = Some(self.icon.clone());
        e.value = Some(self.age.clone());
        // Do not export unselected private clipboard/AX previews in general state receipts.
        e.selectable = Some(true);
        e
    }
}
type PickSubject = Rc<dyn Fn(&str, &mut gpui::Window, &mut gpui::App)>;
pub(crate) fn render(
    theme: &Theme,
    def: MainMenuThemeDef,
    items: Vec<TabItem>,
    cycle: bool,
    reveal: bool,
    pick: PickSubject,
) -> AnyElement {
    let rule = rgba((theme.colors.text.primary << 8) | def.shell.divider_alpha);
    let panel = panel_color(theme);
    let mut bar = div()
        .id("subject-tab-bar")
        .debug_selector(|| "subject-tab-bar".into())
        .role(gpui::Role::TabList)
        .aria_label("Subjects")
        .w_full()
        .h(px(HEIGHT))
        .pt(px(def.slot_line.chip.gap_px))
        .flex()
        .items_stretch()
        .overflow_hidden()
        .child(
            div()
                .flex_none()
                .w(px(PANEL_INSET))
                .border_b_1()
                .border_color(rule),
        );
    for item in items {
        let active = item.active;
        let text = theme.colors.text.primary;
        let ink = rgba((text << 8) | if active { 0xff } else { 0xb8 });
        let key = item.id.clone();
        let name = item.semantic_id();
        let handler = pick.clone();
        let tab = div()
            .id(SharedString::from(name.clone()))
            .debug_selector(move || name.clone())
            .role(gpui::Role::Tab)
            .aria_selected(active)
            .aria_label(format!("{}, {}, {}", item.label, item.detail, item.age))
            .cursor_pointer()
            .w(px(TAB_WIDTH))
            .min_w(px(0.))
            .flex_shrink_1()
            .flex()
            .items_center()
            .px(px(TOP_RADIUS))
            .gap(px(def.slot_line.chip.gap_px))
            .whitespace_nowrap()
            .overflow_hidden()
            .text_size(px(def.slot_line.title_font_size))
            .text_color(ink)
            .when(active, |d| {
                d.rounded_t(px(TOP_RADIUS))
                    .bg(panel)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
            })
            .when(!active, |d| {
                d.border_b_1().border_color(rule).hover(|s| s.bg(panel))
            })
            .on_click(move |_, window, cx| handler(&key, window, cx))
            .child(
                div()
                    .flex_none()
                    .opacity(if active { 1.0 } else { 0.72 })
                    .child(render_slot_icon(
                        &item.icon,
                        def.slot_line.chip.icon_size,
                        text,
                    )),
            )
            .child(div().min_w(px(0.)).flex_1().truncate().child(item.label))
            .when(item.fresh, |d| {
                d.child(
                    div()
                        .size(px(def.footer.metrics.content_gap))
                        .rounded_full()
                        .bg(rgb(theme.colors.accent.selected)),
                )
            })
            .when_some(item.hint.filter(|_| reveal), |d, k| {
                d.child(render_hint(theme, def, k, ""))
            })
            .child(
                div()
                    .flex_none()
                    .text_size(px(def.slot_line.hint_font_size))
                    .font_weight(gpui::FontWeight::NORMAL)
                    .text_color(rgba((text << 8) | 0xa8))
                    .child(item.age),
            );
        bar = bar.child(tab);
    }
    bar.child(
        div()
            .min_w(px(def.slot_line.chip.height_px))
            .flex_1()
            .h_full()
            .border_b_1()
            .border_color(rule)
            .flex()
            .items_center()
            .justify_end()
            .pr(px(PANEL_INSET))
            .when(cycle && !reveal, |d| {
                d.child(render_hint(theme, def, "⇥", ""))
            }),
    )
    .into_any_element()
}
