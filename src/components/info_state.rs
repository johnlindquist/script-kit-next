#![allow(dead_code)]

use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Div, FontWeight, Rgba, SharedString};

use crate::theme::{self, AppChromeColors};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfoTextMetric {
    pub size: f32,
    pub line: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfoTypeScale {
    pub micro: InfoTextMetric,
    pub caption: InfoTextMetric,
    pub body: InfoTextMetric,
    pub subhead: InfoTextMetric,
    pub title: InfoTextMetric,
    pub hero: InfoTextMetric,
    pub brand: InfoTextMetric,
}

pub(crate) const INFO_TYPE_SCALE: InfoTypeScale = InfoTypeScale {
    micro: InfoTextMetric {
        size: 11.0,
        line: 14.0,
    },
    caption: InfoTextMetric {
        size: 12.0,
        line: 16.0,
    },
    body: InfoTextMetric {
        size: 13.0,
        line: 18.0,
    },
    subhead: InfoTextMetric {
        size: 14.0,
        line: 20.0,
    },
    title: InfoTextMetric {
        size: 16.0,
        line: 22.0,
    },
    hero: InfoTextMetric {
        size: 20.0,
        line: 26.0,
    },
    brand: InfoTextMetric {
        size: 22.0,
        line: 28.0,
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfoSpacing {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

pub(crate) const INFO_SPACING: InfoSpacing = InfoSpacing {
    xxs: 4.0,
    xs: 8.0,
    sm: 12.0,
    md: 16.0,
    lg: 20.0,
    xl: 24.0,
    xxl: 32.0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InfoStateDensity {
    Compact,
    Comfortable,
    Hero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InfoStateLayout {
    Centered,
    AnchoredTop,
    MainViewColumns,
    ComposerEmpty,
    InlinePanel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InfoStateTone {
    Neutral,
    Help,
    Setup,
    Permission,
    Recovery,
    About,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InfoMetrics {
    pub max_width: f32,
    pub icon_size: f32,
    pub row_min_h: f32,
    pub radius: f32,
    pub block_gap: f32,
    pub item_gap: f32,
    pub pad_x: f32,
    pub pad_y: f32,
}

pub(crate) fn info_metrics(density: InfoStateDensity) -> InfoMetrics {
    match density {
        InfoStateDensity::Compact => InfoMetrics {
            max_width: 380.0,
            icon_size: 28.0,
            row_min_h: 28.0,
            radius: 8.0,
            block_gap: 12.0,
            item_gap: 6.0,
            pad_x: 12.0,
            pad_y: 10.0,
        },
        InfoStateDensity::Comfortable => InfoMetrics {
            max_width: 500.0,
            icon_size: 36.0,
            row_min_h: 34.0,
            radius: 9.0,
            block_gap: 16.0,
            item_gap: 8.0,
            pad_x: 16.0,
            pad_y: 14.0,
        },
        InfoStateDensity::Hero => InfoMetrics {
            max_width: 560.0,
            icon_size: 44.0,
            row_min_h: 36.0,
            radius: 10.0,
            block_gap: 20.0,
            item_gap: 10.0,
            pad_x: 20.0,
            pad_y: 16.0,
        },
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InfoPalette {
    pub title: Rgba,
    pub body: Rgba,
    pub hint: Rgba,
    pub strong: Rgba,
    pub placeholder: Rgba,
    pub icon: Rgba,
    pub accent: Rgba,
    pub hover: Rgba,
    pub selected: Rgba,
    pub border: Rgba,
    pub whisper: Rgba,
    pub panel: Rgba,
}

pub(crate) fn info_palette(theme: &theme::Theme) -> InfoPalette {
    let chrome = AppChromeColors::from_theme(theme);
    InfoPalette {
        title: rgb(chrome.text_primary_hex),
        body: rgba(chrome.text_muted_rgba),
        hint: rgba(chrome.text_hint_rgba),
        strong: rgba(chrome.text_strong_rgba),
        placeholder: rgba(chrome.placeholder_text_rgba),
        icon: rgba(chrome.text_icon_rgba),
        accent: rgb(chrome.accent_hex),
        hover: rgba(chrome.hover_rgba),
        selected: rgba(chrome.selection_rgba),
        border: rgba(chrome.whisper_border_rgba),
        whisper: rgba(chrome.whisper_surface_rgba),
        panel: rgba(chrome.panel_surface_rgba),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InfoGuidanceItem {
    pub shortcut: Option<&'static str>,
    pub label: SharedString,
    pub detail: Option<SharedString>,
}

impl InfoGuidanceItem {
    pub(crate) fn new(shortcut: Option<&'static str>, label: impl Into<SharedString>) -> Self {
        Self {
            shortcut,
            label: label.into(),
            detail: None,
        }
    }

    pub(crate) fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InfoSection {
    pub title: Option<SharedString>,
    pub items: Vec<InfoGuidanceItem>,
}

impl InfoSection {
    pub(crate) fn new(items: Vec<InfoGuidanceItem>) -> Self {
        Self { title: None, items }
    }

    pub(crate) fn titled(title: impl Into<SharedString>, items: Vec<InfoGuidanceItem>) -> Self {
        Self {
            title: Some(title.into()),
            items,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InfoShortcutNote {
    pub shortcut: &'static str,
    pub text: SharedString,
}

impl InfoShortcutNote {
    pub(crate) fn new(shortcut: &'static str, text: impl Into<SharedString>) -> Self {
        Self {
            shortcut,
            text: text.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InfoStateSpec {
    pub id: &'static str,
    pub layout: InfoStateLayout,
    pub density: InfoStateDensity,
    pub tone: InfoStateTone,
    pub eyebrow: Option<SharedString>,
    pub title: Option<SharedString>,
    pub body: Option<SharedString>,
    pub sections: Vec<InfoSection>,
    pub footer_note: Option<SharedString>,
    pub footer_shortcut_note: Option<InfoShortcutNote>,
}

impl InfoStateSpec {
    pub(crate) fn new(id: &'static str) -> Self {
        Self {
            id,
            layout: InfoStateLayout::Centered,
            density: InfoStateDensity::Compact,
            tone: InfoStateTone::Neutral,
            eyebrow: None,
            title: None,
            body: None,
            sections: Vec::new(),
            footer_note: None,
            footer_shortcut_note: None,
        }
    }

    pub(crate) fn layout(mut self, layout: InfoStateLayout) -> Self {
        self.layout = layout;
        self
    }

    pub(crate) fn density(mut self, density: InfoStateDensity) -> Self {
        self.density = density;
        self
    }

    pub(crate) fn tone(mut self, tone: InfoStateTone) -> Self {
        self.tone = tone;
        self
    }

    pub(crate) fn eyebrow(mut self, eyebrow: impl Into<SharedString>) -> Self {
        self.eyebrow = Some(eyebrow.into());
        self
    }

    pub(crate) fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub(crate) fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub(crate) fn section(mut self, section: InfoSection) -> Self {
        self.sections.push(section);
        self
    }

    pub(crate) fn footer_note(mut self, note: impl Into<SharedString>) -> Self {
        self.footer_note = Some(note.into());
        self.footer_shortcut_note = None;
        self
    }

    pub(crate) fn footer_shortcut_note(
        mut self,
        shortcut: &'static str,
        text: impl Into<SharedString>,
    ) -> Self {
        self.footer_shortcut_note = Some(InfoShortcutNote::new(shortcut, text));
        self.footer_note = None;
        self
    }
}

pub(crate) fn acp_empty_guidance_spec() -> InfoStateSpec {
    InfoStateSpec::new("acp-empty-composer-guidance")
        .layout(InfoStateLayout::ComposerEmpty)
        .density(InfoStateDensity::Comfortable)
        .tone(InfoStateTone::Help)
        .title("Ask with context")
        .body("Describe the result you want. Use / for skills or @ to attach context before you send.")
        .section(InfoSection::new(vec![
            InfoGuidanceItem::new(Some("/"), "Use a skill or agent command"),
            InfoGuidanceItem::new(Some("@"), "Attach files, scripts, clipboard, or history"),
            InfoGuidanceItem::new(Some("⇧↵"), "Add a newline"),
            InfoGuidanceItem::new(Some("⌘P"), "Open previous chats"),
        ]))
        .footer_shortcut_note("⌘K", "shows every chat action.")
}

pub(crate) fn render_acp_empty_guidance(theme: &theme::Theme) -> AnyElement {
    render_info_state(acp_empty_guidance_spec(), theme)
}

pub(crate) fn launcher_empty_or_no_results_spec(
    filter_text_for_render: &str,
    has_active_filter: bool,
) -> InfoStateSpec {
    if filter_text_for_render.is_empty() {
        return launcher_no_scripts_spec();
    }
    if has_active_filter {
        return launcher_active_filter_no_results_spec(filter_text_for_render);
    }
    if launcher_plain_hash_search(filter_text_for_render) {
        return launcher_plain_hash_no_results_spec(filter_text_for_render);
    }
    launcher_generic_no_results_spec(filter_text_for_render)
}

pub(crate) fn render_launcher_empty_or_no_results(
    filter_text_for_render: &str,
    has_active_filter: bool,
    theme: &theme::Theme,
) -> AnyElement {
    render_info_state(
        launcher_empty_or_no_results_spec(filter_text_for_render, has_active_filter),
        theme,
    )
}

fn launcher_no_scripts_spec() -> InfoStateSpec {
    InfoStateSpec::new("launcher-empty-no-scripts")
        .layout(InfoStateLayout::Centered)
        .density(InfoStateDensity::Compact)
        .tone(InfoStateTone::Help)
        .title("No scripts yet")
        .body("This launcher opens your Script Kit scripts and snippets. Create one now, ask Agent Chat to draft the workflow, or open Actions for setup and install options.")
        .section(InfoSection::new(vec![
            InfoGuidanceItem::new(Some("⌘N"), "Create a script")
                .detail("Start a new automation in your scripts folder."),
            InfoGuidanceItem::new(Some("⇥"), "Ask Agent Chat")
                .detail("Describe the workflow you want and let AI draft it."),
            InfoGuidanceItem::new(Some("⌘K"), "Open Actions")
                .detail("Find reload, install, and setup commands."),
        ]))
        .footer_note("After scripts exist, type here to search and run them.")
}

fn launcher_active_filter_no_results_spec(filter_text: &str) -> InfoStateSpec {
    let filter_display = launcher_filter_display(filter_text);
    InfoStateSpec::new("launcher-empty-active-filter")
        .layout(InfoStateLayout::Centered)
        .density(InfoStateDensity::Compact)
        .tone(InfoStateTone::Recovery)
        .title(format!("No matches for \"{filter_display}\""))
        .body("The search is working, but an active filter is narrowing the launcher to zero results. Remove a filter chip or loosen the query to widen the set.")
        .section(InfoSection::new(vec![
            InfoGuidanceItem::new(Some("Esc"), "Clear the search"),
            InfoGuidanceItem::new(Some("Filter"), "Remove a filter chip")
                .detail("Source and type filters apply before fuzzy matching."),
            InfoGuidanceItem::new(Some("⌘K"), "Open Actions")
                .detail("Use actions if you meant to manage scripts or filters."),
        ]))
        .footer_note("Filters narrow the library before the launcher ranks results.")
}

fn launcher_plain_hash_no_results_spec(filter_text: &str) -> InfoStateSpec {
    let filter_display = launcher_filter_display(filter_text);
    InfoStateSpec::new("launcher-empty-plain-hash")
        .layout(InfoStateLayout::Centered)
        .density(InfoStateDensity::Compact)
        .tone(InfoStateTone::Help)
        .title("Tags need a syntax prefix")
        .body(format!("Plain {filter_display} is treated as launcher text search. Use :#tag to filter existing tags, or add #tag after a capture like ;todo when you are creating one."))
        .section(InfoSection::titled(
            "Examples",
            vec![
                InfoGuidanceItem::new(Some(":#"), "Filter tagged items").detail("Example: :#work"),
                InfoGuidanceItem::new(Some(":tag:"), "Filter by tag name")
                    .detail("Example: :tag:work"),
                InfoGuidanceItem::new(Some(";todo"), "Create a tagged capture")
                    .detail("Example: ;todo Buy milk #errands"),
            ],
        ))
        .footer_note("Keep #tag plain only when you want text search, not tag filtering.")
}

fn launcher_generic_no_results_spec(filter_text: &str) -> InfoStateSpec {
    let filter_display = launcher_filter_display(filter_text);
    InfoStateSpec::new("launcher-empty-generic-no-results")
        .layout(InfoStateLayout::Centered)
        .density(InfoStateDensity::Compact)
        .tone(InfoStateTone::Recovery)
        .title(format!("No results for \"{filter_display}\""))
        .body("The launcher searches scripts, scriptlets, snippets, and built-in commands by name and metadata. Try fewer words, use a structured filter, capture the thought, or ask Agent Chat to turn it into a script.")
        .section(InfoSection::new(vec![
            InfoGuidanceItem::new(Some("Esc"), "Clear the search"),
            InfoGuidanceItem::new(Some("type:"), "Search by metadata")
                .detail("Examples: type:script · type:scriptlet · shortcut:cmd+k"),
            InfoGuidanceItem::new(Some(";todo"), "Capture instead")
                .detail("Examples: ;todo · ;note"),
            InfoGuidanceItem::new(Some("⌘↵"), "Ask Agent Chat")
                .detail("Turn this search into a script request."),
        ]))
        .footer_note("Structured filters work best for metadata; plain words work best for names.")
}

fn launcher_plain_hash_search(filter_text: &str) -> bool {
    filter_text.starts_with('#') && filter_text.chars().skip(1).all(|ch| !ch.is_whitespace())
}

fn launcher_filter_display(filter_text: &str) -> String {
    if filter_text.chars().count() > 30 {
        format!("{}...", crate::utils::truncate_str_chars(filter_text, 27))
    } else {
        filter_text.to_string()
    }
}

pub(crate) fn render_info_state(spec: InfoStateSpec, theme: &theme::Theme) -> AnyElement {
    let def = crate::designs::current_main_menu_theme().def();
    render_info_state_with_main_view_def(spec, theme, def)
}

pub(crate) fn render_info_state_with_main_view_def(
    spec: InfoStateSpec,
    theme: &theme::Theme,
    def: crate::designs::MainMenuThemeDef,
) -> AnyElement {
    let palette = info_palette(theme);
    let metrics = info_metrics(spec.density);
    let uses_main_view_columns = matches!(
        spec.layout,
        InfoStateLayout::ComposerEmpty | InfoStateLayout::MainViewColumns
    );
    let content = render_info_content(&spec, theme, palette, metrics, !uses_main_view_columns);

    match spec.layout {
        InfoStateLayout::Centered => div()
            .id(spec.id)
            .w_full()
            .h_full()
            .min_h(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .px(px(INFO_SPACING.xl))
            .child(content)
            .into_any_element(),
        InfoStateLayout::ComposerEmpty | InfoStateLayout::MainViewColumns => {
            let cols = crate::components::main_view_chrome::main_view_content_columns(def);
            div()
                .id(spec.id)
                .w_full()
                .h_full()
                .min_h(px(0.0))
                .flex()
                .items_start()
                .justify_start()
                .pl(px(cols.text_column_x))
                .pr(px(cols.content_right_inset_x))
                .pt(px(cols.top_inset_y))
                .pb(px(def.shell.content_inset_bottom))
                .child(content)
                .into_any_element()
        }
        InfoStateLayout::AnchoredTop => div()
            .id(spec.id)
            .w_full()
            .h_full()
            .min_h(px(0.0))
            .flex()
            .items_start()
            .justify_center()
            .px(px(INFO_SPACING.xl))
            .py(px(INFO_SPACING.xl))
            .child(content)
            .into_any_element(),
        InfoStateLayout::InlinePanel => div()
            .id(spec.id)
            .w_full()
            .child(
                content
                    .rounded(px(metrics.radius))
                    .border_1()
                    .border_color(palette.border)
                    .bg(palette.whisper)
                    .px(px(metrics.pad_x))
                    .py(px(metrics.pad_y)),
            )
            .into_any_element(),
    }
}

fn render_info_content(
    spec: &InfoStateSpec,
    theme: &theme::Theme,
    palette: InfoPalette,
    metrics: InfoMetrics,
    cap_width: bool,
) -> Div {
    let title_metric = match spec.density {
        InfoStateDensity::Compact => INFO_TYPE_SCALE.subhead,
        InfoStateDensity::Comfortable => INFO_TYPE_SCALE.title,
        InfoStateDensity::Hero => INFO_TYPE_SCALE.hero,
    };
    let body_metric = INFO_TYPE_SCALE.body;

    let mut stack = div().w_full().flex().flex_col().gap(px(metrics.block_gap));
    if cap_width {
        stack = stack.max_w(px(metrics.max_width));
    }

    if let Some(eyebrow) = spec.eyebrow.clone() {
        stack = stack.child(
            div()
                .text_size(px(INFO_TYPE_SCALE.micro.size))
                .line_height(px(INFO_TYPE_SCALE.micro.line))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.strong)
                .child(eyebrow),
        );
    }

    if spec.title.is_some() || spec.body.is_some() {
        let mut intro = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(INFO_SPACING.xs * 0.5));
        if let Some(title) = spec.title.clone() {
            intro = intro.child(
                div()
                    .text_size(px(title_metric.size))
                    .line_height(px(title_metric.line))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.title)
                    .child(title),
            );
        }
        if let Some(body) = spec.body.clone() {
            intro = intro.child(
                div()
                    .text_size(px(body_metric.size))
                    .line_height(px(body_metric.line))
                    .text_color(palette.body)
                    .child(body),
            );
        }
        stack = stack.child(intro);
    }

    for (index, section) in spec.sections.iter().enumerate() {
        stack = stack.child(render_info_section(
            section,
            format!("{}-section-{index}", spec.id),
            spec.density,
            theme,
            palette,
            metrics,
        ));
    }

    if let Some(note) = spec.footer_shortcut_note.clone() {
        // Align the footer shortcut keycap into the same fixed-width slot the
        // guidance rows use, so its trailing text lines up with the guidance
        // labels above it instead of starting at the keycap's natural width.
        let guidance_items: Vec<InfoGuidanceItem> = spec
            .sections
            .iter()
            .flat_map(|section| section.items.iter().cloned())
            .collect();
        let shortcut_slot_width_px = info_guidance_shortcut_slot_width_px(&guidance_items);
        stack = stack.child(render_info_shortcut_note(
            note,
            metrics,
            theme,
            palette,
            shortcut_slot_width_px,
        ));
    } else if let Some(note) = spec.footer_note.clone() {
        stack = stack.child(render_info_plain_footer_note(note, palette));
    }

    stack
}

fn render_info_plain_footer_note(note: SharedString, palette: InfoPalette) -> AnyElement {
    div()
        .text_size(px(INFO_TYPE_SCALE.caption.size))
        .line_height(px(INFO_TYPE_SCALE.caption.line))
        .text_color(palette.hint)
        .child(note)
        .into_any_element()
}

fn render_info_shortcut_note(
    note: InfoShortcutNote,
    metrics: InfoMetrics,
    theme: &theme::Theme,
    palette: InfoPalette,
    shortcut_slot_width_px: f32,
) -> AnyElement {
    let keycaps = crate::components::footer_chrome::render_footer_shortcut_keycaps(
        note.shortcut.to_string(),
        theme,
    );
    // Use the same fixed-width keycap slot + row gap as `render_guidance_row` so
    // the note text aligns horizontally with the guidance labels above it.
    let keycap_slot = if shortcut_slot_width_px > 0.0 {
        div()
            .w(px(shortcut_slot_width_px))
            .flex_none()
            .flex()
            .items_center()
            .child(keycaps)
    } else {
        div().flex().items_center().child(keycaps)
    };
    div()
        .w_full()
        .min_h(px(metrics.row_min_h))
        .flex()
        .items_center()
        .gap(px(INFO_SPACING.sm))
        .child(keycap_slot)
        .child(render_info_guidance_text(note.text, None, palette))
        .into_any_element()
}

fn render_info_section(
    section: &InfoSection,
    id: String,
    density: InfoStateDensity,
    theme: &theme::Theme,
    palette: InfoPalette,
    metrics: InfoMetrics,
) -> AnyElement {
    let mut stack = div()
        .id(id)
        .w_full()
        .flex()
        .flex_col()
        .gap(px(metrics.item_gap));

    if let Some(title) = section.title.clone() {
        stack = stack.child(
            div()
                .text_size(px(INFO_TYPE_SCALE.micro.size))
                .line_height(px(INFO_TYPE_SCALE.micro.line))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.strong)
                .child(title),
        );
    }

    stack
        .child(render_info_guidance_items(
            "info-guidance-items",
            &section.items,
            density,
            theme,
            palette,
        ))
        .into_any_element()
}

pub(crate) fn render_info_guidance_items(
    id: &'static str,
    items: &[InfoGuidanceItem],
    density: InfoStateDensity,
    theme: &theme::Theme,
    palette: InfoPalette,
) -> AnyElement {
    let metrics = info_metrics(density);
    let shortcut_slot_width_px = info_guidance_shortcut_slot_width_px(items);
    div()
        .id(id)
        .w_full()
        .flex()
        .flex_col()
        .gap(px(INFO_SPACING.xs * 0.5))
        .children(items.iter().map(|item| {
            render_guidance_row(item, metrics, theme, palette, shortcut_slot_width_px)
                .into_any_element()
        }))
        .into_any_element()
}

fn info_guidance_shortcut_slot_width_px(items: &[InfoGuidanceItem]) -> f32 {
    let min_shortcut_width = crate::components::footer_chrome::FOOTER_KEYCAP_HEIGHT_PX * 2.0
        + crate::components::footer_chrome::FOOTER_ACTION_CONTENT_GAP_PX;
    let max_width = items
        .iter()
        .filter_map(|item| item.shortcut)
        .map(crate::components::footer_chrome::footer_shortcut_keycaps_width_px)
        .fold(0.0, f32::max);

    if max_width > 0.0 {
        max_width.max(min_shortcut_width)
    } else {
        0.0
    }
}

fn render_guidance_row(
    item: &InfoGuidanceItem,
    metrics: InfoMetrics,
    theme: &theme::Theme,
    palette: InfoPalette,
    shortcut_slot_width_px: f32,
) -> Div {
    let mut row = div()
        .w_full()
        .min_h(px(metrics.row_min_h))
        .flex()
        .items_center()
        .gap(px(INFO_SPACING.sm));

    if let Some(shortcut) = item.shortcut {
        row = row.child(
            div()
                .w(px(shortcut_slot_width_px))
                .flex_none()
                .flex()
                .items_center()
                .child(
                    crate::components::footer_chrome::render_footer_shortcut_keycaps(
                        shortcut.to_string(),
                        theme,
                    ),
                ),
        );
    }

    row.child(render_info_guidance_text(
        item.label.clone(),
        item.detail.clone(),
        palette,
    ))
}

fn render_info_guidance_text(
    label: SharedString,
    detail: Option<SharedString>,
    palette: InfoPalette,
) -> Div {
    let mut text = div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_size(px(INFO_TYPE_SCALE.caption.size))
                .line_height(px(INFO_TYPE_SCALE.caption.line))
                .text_color(palette.body)
                .child(label),
        );

    if let Some(detail) = detail {
        text = text.child(
            div()
                .text_size(px(INFO_TYPE_SCALE.micro.size))
                .line_height(px(INFO_TYPE_SCALE.micro.line))
                .text_color(palette.hint)
                .child(detail),
        );
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_type_scale_matches_compact_palette_math() {
        assert_eq!(
            INFO_TYPE_SCALE.micro,
            InfoTextMetric {
                size: 11.0,
                line: 14.0
            }
        );
        assert_eq!(
            INFO_TYPE_SCALE.caption,
            InfoTextMetric {
                size: 12.0,
                line: 16.0
            }
        );
        assert_eq!(
            INFO_TYPE_SCALE.body,
            InfoTextMetric {
                size: 13.0,
                line: 18.0
            }
        );
        assert_eq!(
            INFO_TYPE_SCALE.title,
            InfoTextMetric {
                size: 16.0,
                line: 22.0
            }
        );
        assert!(INFO_TYPE_SCALE.brand.size <= 22.0);
    }

    #[test]
    fn info_metrics_use_four_pixel_rhythm_where_visible() {
        for metrics in [
            info_metrics(InfoStateDensity::Compact),
            info_metrics(InfoStateDensity::Comfortable),
            info_metrics(InfoStateDensity::Hero),
        ] {
            for value in [
                metrics.icon_size,
                metrics.row_min_h,
                metrics.block_gap,
                metrics.pad_x,
                metrics.pad_y,
            ] {
                assert_eq!(value.rem_euclid(2.0), 0.0);
            }
        }
    }

    #[test]
    fn acp_empty_guidance_teaches_starting_context_not_window_management() {
        let spec = acp_empty_guidance_spec();
        let copy = format!("{spec:?}");
        assert!(copy.contains("Ask with context"));
        assert!(copy.contains("Use a skill or agent command"));
        assert!(copy.contains("Attach files, scripts, clipboard, or history"));
        assert!(copy.contains("Add a newline"));
        assert!(copy.contains("Open previous chats"));
        assert!(!copy.contains("Type / for skills"));
        assert!(!copy.contains(&format!("{} new", "⌘N")));
        assert!(!copy.contains(&format!("{} close", "⌘W")));
    }

    #[test]
    fn guidance_shortcut_slot_width_tracks_footer_keycap_widths() {
        let items = vec![
            InfoGuidanceItem::new(Some("⌘P"), "Open previous chats"),
            InfoGuidanceItem::new(Some(":tag:"), "Filter by tag name"),
            InfoGuidanceItem::new(Some(";todo"), "Capture instead"),
        ];
        let width = info_guidance_shortcut_slot_width_px(&items);

        assert_eq!(
            width,
            crate::components::footer_chrome::footer_shortcut_keycaps_width_px(":tag:")
        );
        assert!(width > crate::components::footer_chrome::footer_shortcut_keycaps_width_px("⌘P"));
    }

    #[test]
    fn footer_shortcut_note_shares_guidance_keycap_slot_width() {
        // Regression: the acp empty-state footer note (⌘K) must align its
        // trailing text with the guidance labels above it. That requires the
        // note to render its keycap into the same fixed-width slot the guidance
        // rows compute from the section items.
        let spec = acp_empty_guidance_spec();
        assert!(
            spec.footer_shortcut_note.is_some(),
            "acp guidance should carry a footer shortcut note"
        );
        let guidance_items: Vec<InfoGuidanceItem> = spec
            .sections
            .iter()
            .flat_map(|section| section.items.iter().cloned())
            .collect();
        let slot = info_guidance_shortcut_slot_width_px(&guidance_items);
        assert!(
            slot > 0.0,
            "footer note must align into a real (positive) keycap slot, got {slot}"
        );
    }

    #[test]
    fn launcher_empty_guidance_teaches_library_and_next_actions() {
        let spec = launcher_empty_or_no_results_spec("", false);
        let copy = format!("{spec:?}");
        assert!(copy.contains("No scripts yet"));
        assert!(copy.contains("This launcher opens your Script Kit scripts and snippets"));
        assert!(copy.contains("Create a script"));
        assert!(copy.contains("Ask Agent Chat"));
        assert!(copy.contains("Open Actions"));
        assert!(!copy.contains("No scripts or snippets found"));
        assert!(!copy.contains("Press ⌘N to create a new script"));
    }

    #[test]
    fn launcher_no_results_preserves_active_filter_plain_hash_and_generic_cases() {
        let active = format!(
            "{:?}",
            launcher_empty_or_no_results_spec("type:script nope", true)
        );
        assert!(active.contains("No matches for"));
        assert!(active.contains("active filter is narrowing"));
        assert!(active.contains("Remove a filter chip"));
        assert!(active.contains("Source and type filters apply before fuzzy matching"));

        let tag = format!("{:?}", launcher_empty_or_no_results_spec("#work", false));
        assert!(tag.contains("Tags need a syntax prefix"));
        assert!(tag.contains("Plain #work is treated as launcher text search"));
        assert!(tag.contains("Example: :#work"));
        assert!(tag.contains("Example: :tag:work"));
        assert!(tag.contains("Example: ;todo Buy milk #errands"));

        let generic = format!("{:?}", launcher_empty_or_no_results_spec("zzz", false));
        assert!(generic.contains("No results for"));
        assert!(generic.contains("zzz"));
        assert!(generic.contains("scripts, scriptlets, snippets, and built-in commands"));
        assert!(generic.contains("type:script"));
        assert!(generic.contains("shortcut:cmd+k"));
        assert!(generic.contains("Ask Agent Chat"));
    }

    #[test]
    fn launcher_no_results_truncates_long_utf8_filter_display() {
        let input = "é".repeat(45);
        let spec = launcher_empty_or_no_results_spec(&input, false);
        let copy = format!("{spec:?}");
        assert!(copy.contains("..."));
        assert!(!copy.contains(&"é".repeat(45)));
    }

    #[test]
    fn launcher_empty_state_routes_through_info_state() {
        let source = std::fs::read_to_string("src/render_script_list/mod.rs")
            .expect("failed to read src/render_script_list/mod.rs");
        let old_empty_title = concat!("No scripts or ", "snippets found");
        let old_empty_hint = concat!("Press ", "⌘N", " to create a new script");
        let old_generic_fallback =
            concat!("Try a different search term or press ", "⌘↵", " to ask AI");

        assert!(
            source.contains("render_launcher_empty_or_no_results"),
            "launcher empty/no-results must render through shared InfoState"
        );
        assert!(
            !source.contains(old_empty_title),
            "old launcher empty title must not return"
        );
        assert!(
            !source.contains(old_empty_hint),
            "old launcher empty hint must not return"
        );
        assert!(
            !source.contains(old_generic_fallback),
            "old generic no-results fallback must not return"
        );
    }
}
