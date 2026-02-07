use super::*;

impl NotesApp {
    /// Render the keyboard shortcuts help overlay
    pub(super) fn render_shortcuts_help(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let accent = cx.theme().accent;
        let border_color = cx.theme().border;
        let bg = cx.theme().background.opacity(OPACITY_OVERLAY_BG);

        let shortcut = |keys: &str, desc: &str| -> AnyElement {
            div()
                .flex()
                .justify_between()
                .w_full()
                .py_1() // 4px — on the spacing grid
                .child(div().text_xs().text_color(muted).child(desc.to_string()))
                .child(div().text_xs().text_color(accent).child(keys.to_string()))
                .into_any_element()
        };

        let section = |title: &str| -> AnyElement {
            div()
                .pt_3()
                .pb_1() // 4px — on the spacing grid
                .mb_1() // 4px — on the spacing grid
                .border_b_1()
                .border_color(border_color.opacity(OPACITY_SECTION_BORDER))
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(muted.opacity(OPACITY_MUTED))
                .child(title.to_string())
                .into_any_element()
        };

        div()
            .id("shortcuts-help-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.show_shortcuts_help = false;
                    cx.notify();
                }),
            )
            .child(
                div()
                    .w(px(SHORTCUTS_PANEL_WIDTH))
                    .max_h(px(SHORTCUTS_PANEL_MAX_HEIGHT))
                    .overflow_y_scrollbar()
                    .rounded(px(SHORTCUTS_PANEL_RADIUS))
                    .border_1()
                    .border_color(border_color.opacity(OPACITY_SECTION_BORDER))
                    .p_4()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .pb_2()
                            .child("Keyboard Shortcuts"),
                    )
                    .child(section("Notes"))
                    .child(shortcut("⌘N", "New note"))
                    .child(shortcut("⌘⇧N", "New from clipboard"))
                    .child(shortcut("⌘D", "Duplicate note"))
                    .child(shortcut("⌘⌫", "Delete note"))
                    .child(shortcut("⌘⇧I", "Toggle pin"))
                    .child(section("Navigation"))
                    .child(shortcut("⌘↑ / ⌘↓", "Previous / next note"))
                    .child(shortcut("⌘⇧↑ / ⌘⇧↓", "First / last note"))
                    .child(shortcut("⌘[ / ⌘]", "Back / forward"))
                    .child(shortcut("⌘1–9", "Jump to pinned note"))
                    .child(shortcut("⌘P", "Note switcher"))
                    .child(shortcut("⌘K", "Actions"))
                    .child(section("Formatting"))
                    .child(shortcut("⌘B", "Bold"))
                    .child(shortcut("⌘I", "Italic"))
                    .child(shortcut("⌘E", "Inline code"))
                    .child(shortcut("⌘⇧X", "Strikethrough"))
                    .child(shortcut("⌘⇧H", "Cycle heading"))
                    .child(shortcut("⌘⇧L", "Toggle checklist"))
                    .child(shortcut("⌘⇧.", "Toggle blockquote"))
                    .child(shortcut("⌘⇧-", "Horizontal rule"))
                    .child(shortcut("⌘⇧8", "Bullet list"))
                    .child(shortcut("⌘⇧7", "Numbered list"))
                    .child(section("Text"))
                    .child(shortcut("⌘⇧D", "Insert date/time"))
                    .child(shortcut("⌘⇧C", "Copy as markdown"))
                    .child(shortcut("⌘L", "Select line"))
                    .child(shortcut("⌘J", "Join lines"))
                    .child(shortcut("⌘⇧U", "Cycle case"))
                    .child(shortcut("⌥↑ / ⌥↓", "Move line"))
                    .child(shortcut("⌥⇧↑ / ⌥⇧↓", "Duplicate line"))
                    .child(shortcut("⌃⇧K", "Delete line"))
                    .child(shortcut("⌘V", "Smart paste"))
                    .child(shortcut("Tab", "Indent (2 spaces)"))
                    .child(shortcut("⇧Tab", "Outdent"))
                    .child(section("View"))
                    .child(shortcut("⌘.  / Esc", "Focus mode"))
                    .child(shortcut("⌘⇧P", "Markdown preview"))
                    .child(shortcut("⌘F", "Find in note"))
                    .child(shortcut("⌘⇧F", "Search all notes"))
                    .child(shortcut("⌘⇧S", "Cycle sort"))
                    .child(shortcut("⌘⇧T", "Toggle trash"))
                    .child(section("Window"))
                    .child(shortcut("⌘W", "Close"))
                    .child(shortcut("Esc", "Close panel"))
                    .child(shortcut("⌘/", "This help"))
                    .child(
                        div()
                            .pt_3()
                            .text_xs()
                            .text_color(muted.opacity(OPACITY_SUBTLE))
                            .text_center()
                            .child("Click anywhere or press ⌘/ to dismiss"),
                    ),
            )
    }
}
