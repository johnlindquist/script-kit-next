//! Frosted Surface Variations
//!
//! Focused on fake-frost strategies that map from common CSS hacks:
//! duplicated backdrop content, local contrast suppression, edge layers,
//! grain, refracted ghost text, and optical displacement.

use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::storybook::{
    story_container, story_item, story_section, Story, StorySurface, StoryVariant,
};
use crate::theme::get_cached_theme;

const PANEL_W: f32 = 312.0;
const PANEL_H: f32 = 184.0;
const SCENE_W: f32 = 560.0;
const SCENE_H: f32 = 340.0;
const PANEL_X: f32 = 118.0;
const PANEL_Y: f32 = 74.0;

const GOLD: u32 = 0xfbbf24;
const ICE: u32 = 0x96d8ff;
const MINT: u32 = 0x99f6e4;
const PEARL: u32 = 0xf8fafc;
const INK: u32 = 0x0f172a;
const STEEL: u32 = 0x1e293b;

const VARIANTS: [VariantSpec; 15] = [
    VariantSpec::new(
        "contrast-mask",
        "Contrast Mask",
        "Backdrop suppression before tinting.",
    )
    .backdrop(BackdropMode::DarkMask)
    .panel(PanelMode::Mask),
    VariantSpec::new(
        "refracted-copy",
        "Refracted Copy",
        "Single echoed backdrop copy.",
    )
    .backdrop(BackdropMode::Ghosted)
    .panel(PanelMode::GhostSingle),
    VariantSpec::new(
        "dual-refraction",
        "Dual Refraction",
        "Two echoed copies for light splitting.",
    )
    .backdrop(BackdropMode::DoubleGhost)
    .panel(PanelMode::GhostDual),
    VariantSpec::new(
        "condensation-band",
        "Condensation Band",
        "Top bloom plus interior streaks.",
    )
    .backdrop(BackdropMode::Condensed)
    .panel(PanelMode::Condensation),
    VariantSpec::new(
        "laminated-sheet",
        "Laminated Sheet",
        "Bright laminated sheet with hard edge.",
    )
    .backdrop(BackdropMode::LightMask)
    .panel(PanelMode::Laminated),
    VariantSpec::new(
        "edge-strip",
        "Edge Strip",
        "Separate lower edge to fake thickness.",
    )
    .backdrop(BackdropMode::DarkMask)
    .panel(PanelMode::EdgeStrip),
    VariantSpec::new("halo-lift", "Halo Lift", "Let halo do more work than fill.")
        .backdrop(BackdropMode::Halo)
        .panel(PanelMode::Halo),
    VariantSpec::new(
        "noise-grain",
        "Noise Grain",
        "Surface grain to break the alpha-card feel.",
    )
    .backdrop(BackdropMode::Ghosted)
    .panel(PanelMode::Noise),
    VariantSpec::new(
        "washed-backdrop",
        "Washed Backdrop",
        "Bleach backdrop instead of deepening the card.",
    )
    .backdrop(BackdropMode::Bleached)
    .panel(PanelMode::Bleached),
    VariantSpec::new(
        "stacked-ghost",
        "Stacked Ghost",
        "Three staggered backdrop echoes.",
    )
    .backdrop(BackdropMode::TripleGhost)
    .panel(PanelMode::GhostStack),
    VariantSpec::new(
        "cool-saturation",
        "Cool Saturation",
        "Blue-biased glass like CSS saturate hacks.",
    )
    .backdrop(BackdropMode::Cool)
    .panel(PanelMode::Cool),
    VariantSpec::new(
        "warm-satin",
        "Warm Satin",
        "Amber satin acrylic instead of neutral frost.",
    )
    .backdrop(BackdropMode::Warm)
    .panel(PanelMode::Warm),
    VariantSpec::new(
        "mask-window",
        "Mask Window",
        "Stronger clipped backdrop window under a quieter card.",
    )
    .backdrop(BackdropMode::Windowed)
    .panel(PanelMode::Quiet),
    VariantSpec::new(
        "offset-proxy",
        "Offset Proxy",
        "Backdrop shifted to feel optically displaced.",
    )
    .backdrop(BackdropMode::OffsetGhost)
    .panel(PanelMode::Offset),
    VariantSpec::new(
        "best-attempt",
        "Best Attempt",
        "Combined suppression, ghosting, edge, and grain.",
    )
    .backdrop(BackdropMode::Best)
    .panel(PanelMode::Best),
];

#[derive(Clone, Copy)]
struct VariantSpec {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    backdrop: BackdropMode,
    panel: PanelMode,
}

impl VariantSpec {
    const fn new(id: &'static str, name: &'static str, description: &'static str) -> Self {
        Self {
            id,
            name,
            description,
            backdrop: BackdropMode::DarkMask,
            panel: PanelMode::Mask,
        }
    }

    const fn backdrop(mut self, backdrop: BackdropMode) -> Self {
        self.backdrop = backdrop;
        self
    }

    const fn panel(mut self, panel: PanelMode) -> Self {
        self.panel = panel;
        self
    }
}

#[derive(Clone, Copy)]
enum BackdropMode {
    DarkMask,
    Ghosted,
    DoubleGhost,
    TripleGhost,
    Condensed,
    LightMask,
    Halo,
    Bleached,
    Cool,
    Warm,
    Windowed,
    OffsetGhost,
    Best,
}

#[derive(Clone, Copy)]
enum PanelMode {
    Mask,
    GhostSingle,
    GhostDual,
    GhostStack,
    Condensation,
    Laminated,
    EdgeStrip,
    Halo,
    Noise,
    Bleached,
    Cool,
    Warm,
    Quiet,
    Offset,
    Best,
}

pub struct FrostedSurfaceVariationsStory;

impl Story for FrostedSurfaceVariationsStory {
    fn id(&self) -> &'static str {
        "frosted-surface-variations"
    }

    fn name(&self) -> &'static str {
        "Frosted Surface Variations (15)"
    }

    fn category(&self) -> &'static str {
        "Components"
    }

    fn surface(&self) -> StorySurface {
        StorySurface::Component
    }

    fn render(&self) -> AnyElement {
        let variants = self.variants();
        let mut container = story_container();

        container = container.child(story_section("Frosted Illusion Experiments").children(
            variants.iter().enumerate().map(|(index, variant)| {
                story_item(
                    &format!("{}. {}", index + 1, variant.name),
                    self.render_variant(variant),
                )
            }),
        ));

        container.into_any_element()
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let spec = variant_spec(variant.stable_id().as_str());
        render_scene(spec).into_any_element()
    }

    fn variants(&self) -> Vec<StoryVariant> {
        VARIANTS
            .iter()
            .map(|spec| {
                StoryVariant::default_named(spec.id, spec.name).description(spec.description)
            })
            .collect()
    }
}

fn variant_spec(id: &str) -> &'static VariantSpec {
    VARIANTS
        .iter()
        .find(|spec| spec.id == id)
        .unwrap_or(&VARIANTS[0])
}

fn render_scene(spec: &VariantSpec) -> Div {
    let theme = get_cached_theme();
    let border_soft = rgba_hex(theme.colors.ui.border, 0x36);

    div()
        .relative()
        .w(px(SCENE_W))
        .h(px(SCENE_H))
        .rounded(px(22.))
        .bg(linear_gradient(
            180.,
            linear_color_stop(rgba_hex(0x272b31, 0xff), 0.0),
            linear_color_stop(rgba_hex(0x171a20, 0xff), 1.0),
        ))
        .border_1()
        .border_color(border_soft)
        .overflow_hidden()
        .child(background_grid())
        .child(background_copy())
        .child(backdrop_proxy(spec.backdrop))
        .child(
            div()
                .absolute()
                .left(px(PANEL_X))
                .top(px(PANEL_Y))
                .child(frosted_panel(spec.panel)),
        )
}

fn background_grid() -> Div {
    div()
        .absolute()
        .top(px(0.))
        .left(px(0.))
        .right(px(0.))
        .bottom(px(0.))
        .children((0..12).map(|index| {
            div()
                .absolute()
                .left(px(0.))
                .right(px(0.))
                .top(px(20.0 + index as f32 * 28.0))
                .h(px(1.))
                .bg(rgba_hex(0xffffff, if index % 2 == 0 { 0x08 } else { 0x04 }))
        }))
}

fn background_copy() -> Div {
    div()
        .absolute()
        .top(px(28.))
        .left(px(28.))
        .right(px(28.))
        .bottom(px(28.))
        .flex()
        .flex_col()
        .gap(px(18.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgba_hex(0xffffff, 0x5e))
                        .child("BACKGROUND CONTENT".to_string()),
                )
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgba_hex(0xffffff, 0xf5))
                        .child("Regular text behind the frosted panel".to_string()),
                ),
        )
        .child(
            div()
                .max_w(px(470.))
                .text_lg()
                .line_height(relative(1.4))
                .text_color(rgba_hex(0xffffff, 0xb8))
                .child(
                    "This layer stays ordinary on purpose. A convincing frosted component should alter how this text is perceived, not just place a colored rectangle over it."
                        .to_string(),
                ),
        )
        .child(
            div()
                .w(px(330.))
                .rounded(px(16.))
                .border_1()
                .border_color(rgba_hex(0xffffff, 0x10))
                .bg(rgba_hex(0xffffff, 0x05))
                .p_3()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgba_hex(0xffffff, 0xee))
                        .child("Behind-panel detail".to_string()),
                )
                .child(
                    div()
                        .mt_2()
                        .text_sm()
                        .line_height(relative(1.5))
                        .text_color(rgba_hex(0xffffff, 0xae))
                        .child(
                            "Hard edges and legible words make fake blur easy to judge. If the treatment works, this card should feel softened and pushed back where the panel crosses it."
                                .to_string(),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .gap(px(10.))
                .children([
                    footer_chip("Surface", "regular"),
                    footer_chip("Goal", "frost"),
                    footer_chip("Context", "in-window"),
                ]),
        )
}

fn backdrop_proxy(mode: BackdropMode) -> Div {
    let base = div()
        .absolute()
        .left(px(PANEL_X - 8.0))
        .top(px(PANEL_Y - 6.0))
        .w(px(PANEL_W + 16.0))
        .h(px(PANEL_H + 12.0))
        .rounded(px(30.))
        .overflow_hidden();

    match mode {
        BackdropMode::DarkMask => base
            .bg(linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0xffffff, 0x0a), 0.0),
                linear_color_stop(rgba_hex(0x0b0d11, 0x68), 1.0),
            ))
            .child(proxy_streaks(rgba_hex(0xffffff, 0x0e), 8)),
        BackdropMode::Ghosted => base.bg(rgba_hex(ICE, 0x10)).child(proxy_text(
            rgba_hex(0xffffff, 0x18),
            px(12.),
            px(24.),
        )),
        BackdropMode::DoubleGhost => base
            .bg(linear_gradient(
                180.,
                linear_color_stop(rgba_hex(ICE, 0x12), 0.0),
                linear_color_stop(rgba_hex(0x0b0d11, 0x5c), 1.0),
            ))
            .child(proxy_text(rgba_hex(0xffffff, 0x14), px(10.), px(24.)))
            .child(proxy_text(rgba_hex(ICE, 0x14), px(18.), px(30.))),
        BackdropMode::TripleGhost => base
            .bg(rgba_hex(ICE, 0x0c))
            .child(proxy_text(rgba_hex(0xffffff, 0x12), px(10.), px(24.)))
            .child(proxy_text(rgba_hex(PEARL, 0x0f), px(16.), px(28.)))
            .child(proxy_text(rgba_hex(ICE, 0x0d), px(22.), px(34.))),
        BackdropMode::Condensed => base
            .bg(linear_gradient(
                180.,
                linear_color_stop(rgba_hex(PEARL, 0x18), 0.0),
                linear_color_stop(rgba_hex(0x0b0d11, 0x54), 1.0),
            ))
            .child(proxy_streaks(rgba_hex(PEARL, 0x12), 14)),
        BackdropMode::LightMask => base.bg(rgba_hex(0xeff4fa, 0x58)).child(proxy_text(
            rgba_hex(INK, 0x14),
            px(12.),
            px(26.),
        )),
        BackdropMode::Halo => base.bg(rgba_hex(MINT, 0x10)).shadow(vec![BoxShadow {
            color: hsla_hex(ICE, 0x1a),
            offset: point(px(0.), px(12.)),
            blur_radius: px(32.),
            spread_radius: px(0.),
        }]),
        BackdropMode::Bleached => base
            .bg(linear_gradient(
                180.,
                linear_color_stop(rgba_hex(PEARL, 0x28), 0.0),
                linear_color_stop(rgba_hex(PEARL, 0x10), 1.0),
            ))
            .child(proxy_text(rgba_hex(INK, 0x10), px(12.), px(26.))),
        BackdropMode::Cool => base
            .bg(linear_gradient(
                180.,
                linear_color_stop(rgba_hex(ICE, 0x18), 0.0),
                linear_color_stop(rgba_hex(0x102032, 0x60), 1.0),
            ))
            .child(proxy_streaks(rgba_hex(ICE, 0x10), 9)),
        BackdropMode::Warm => base
            .bg(linear_gradient(
                180.,
                linear_color_stop(rgba_hex(GOLD, 0x10), 0.0),
                linear_color_stop(rgba_hex(0x241b12, 0x68), 1.0),
            ))
            .child(proxy_text(rgba_hex(0xfffbeb, 0x12), px(12.), px(26.))),
        BackdropMode::Windowed => base
            .bg(rgba_hex(0x0b0d11, 0x74))
            .border_1()
            .border_color(rgba_hex(PEARL, 0x10))
            .child(proxy_streaks(rgba_hex(PEARL, 0x0e), 11)),
        BackdropMode::OffsetGhost => {
            base.bg(rgba_hex(ICE, 0x0a))
                .child(proxy_text(rgba_hex(PEARL, 0x14), px(24.), px(18.)))
        }
        BackdropMode::Best => base
            .bg(linear_gradient(
                180.,
                linear_color_stop(rgba_hex(PEARL, 0x10), 0.0),
                linear_color_stop(rgba_hex(0x0b0d11, 0x72), 1.0),
            ))
            .child(proxy_text(rgba_hex(0xffffff, 0x18), px(14.), px(28.)))
            .child(proxy_streaks(rgba_hex(PEARL, 0x10), 12))
            .child(surface_noise(rgba_hex(PEARL, 0x08))),
    }
}

fn proxy_text(color: Rgba, left: Pixels, top: Pixels) -> Div {
    let lines = [
        "Regular text behind the frosted panel",
        "A convincing frosted component should",
        "change the way this text is perceived,",
        "not just place a colored rectangle.",
    ];

    div()
        .absolute()
        .left(left)
        .top(top)
        .flex()
        .flex_col()
        .gap(px(10.))
        .children(lines.into_iter().map(|line| {
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color)
                .child(line.to_string())
        }))
}

fn proxy_streaks(color: Rgba, count: usize) -> Div {
    div()
        .absolute()
        .top(px(0.))
        .left(px(0.))
        .right(px(0.))
        .bottom(px(0.))
        .children((0..count).map(|index| {
            let top = 10.0 + index as f32 * 11.0;
            let width = if index % 3 == 0 {
                PANEL_W - 24.0
            } else {
                PANEL_W - 54.0
            };

            div()
                .absolute()
                .left(px(12.0 + (index % 2) as f32 * 10.0))
                .top(px(top))
                .w(px(width))
                .h(px(1.))
                .bg(color)
        }))
}

fn surface_noise(color: Rgba) -> Div {
    div()
        .absolute()
        .top(px(0.))
        .left(px(0.))
        .right(px(0.))
        .bottom(px(0.))
        .children((0..28).map(|index| {
            let x = 12.0 + ((index * 17) % 260) as f32;
            let y = 10.0 + ((index * 23) % 150) as f32;
            let w = if index % 3 == 0 { 2.0 } else { 1.0 };
            let h = if index % 5 == 0 { 2.0 } else { 1.0 };

            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(w))
                .h(px(h))
                .bg(color)
        }))
}

fn frosted_panel(mode: PanelMode) -> Div {
    let style = panel_style(mode);

    div()
        .relative()
        .w(px(PANEL_W))
        .h(px(PANEL_H))
        .rounded(px(style.radius))
        .bg(style.fill)
        .border_1()
        .border_color(style.border)
        .shadow(style.shadow)
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .left(px(0.))
                .top(px(0.))
                .right(px(0.))
                .h(px(1.))
                .bg(style.edge_light),
        )
        .child(
            div()
                .absolute()
                .left(px(0.))
                .top(px(0.))
                .right(px(0.))
                .h(px(54.))
                .bg(style.top_glow),
        )
        .when_some(style.inner_ghost, |panel, ghost| panel.child(ghost))
        .when_some(style.bottom_edge, |panel, edge| panel.child(edge))
        .when_some(style.surface_noise, |panel, noise| panel.child(noise))
        .when(style.show_streaks, |panel| {
            panel.child(proxy_streaks(style.streak_color, 10))
        })
        .child(
            div()
                .absolute()
                .left(px(0.))
                .top(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .p_4()
                .flex()
                .flex_col()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .justify_between()
                                .items_center()
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(style.title)
                                        .child("Frosted component".to_string()),
                                )
                                .child(
                                    div()
                                        .px(px(10.))
                                        .py(px(4.))
                                        .rounded(px(999.))
                                        .bg(style.badge_bg)
                                        .border_1()
                                        .border_color(style.badge_border)
                                        .text_xs()
                                        .text_color(style.title)
                                        .child("foreground".to_string()),
                                ),
                        )
                        .child(
                            div()
                                .text_sm()
                                .line_height(relative(1.45))
                                .text_color(style.body)
                                .child(style.copy.to_string()),
                        ),
                )
                .child(div().flex().flex_row().gap(px(8.)).children([
                    footer_chip("Method", style.label),
                    footer_chip("Aim", "optical layer"),
                ])),
        )
}

struct PanelStyle {
    label: &'static str,
    copy: &'static str,
    radius: f32,
    fill: Background,
    border: Rgba,
    edge_light: Rgba,
    top_glow: Background,
    title: Rgba,
    body: Rgba,
    badge_bg: Rgba,
    badge_border: Rgba,
    streak_color: Rgba,
    show_streaks: bool,
    inner_ghost: Option<Div>,
    bottom_edge: Option<Div>,
    surface_noise: Option<Div>,
    shadow: Vec<BoxShadow>,
}

fn panel_style(mode: PanelMode) -> PanelStyle {
    match mode {
        PanelMode::Mask => dark_panel(
            "masked backdrop",
            "The card calms the backdrop first and only then adds a restrained neutral tint.",
        ),
        PanelMode::GhostSingle => ghost_panel(
            "refracted echo",
            "A single echoed copy of the backdrop gives the eye a diffusion cue that a flat alpha card does not.",
            Some(proxy_text(rgba_hex(PEARL, 0x1a), px(18.), px(54.))),
        ),
        PanelMode::GhostDual => ghost_panel(
            "split refraction",
            "Two internal echoes suggest the panel is bending the backdrop instead of merely sitting on top of it.",
            Some(
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .right(px(0.))
                    .bottom(px(0.))
                    .child(proxy_text(rgba_hex(PEARL, 0x14), px(18.), px(52.)))
                    .child(proxy_text(rgba_hex(ICE, 0x12), px(24.), px(58.))),
            ),
        ),
        PanelMode::GhostStack => ghost_panel(
            "stacked ghosts",
            "Three staggered copies produce more visual diffusion than one neat echo.",
            Some(
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .right(px(0.))
                    .bottom(px(0.))
                    .child(proxy_text(rgba_hex(PEARL, 0x12), px(18.), px(56.)))
                    .child(proxy_text(rgba_hex(PEARL, 0x0e), px(24.), px(62.)))
                    .child(proxy_text(rgba_hex(ICE, 0x0c), px(30.), px(68.))),
            ),
        ),
        PanelMode::Condensation => {
            let mut style = dark_panel(
                "condensation band",
                "A stronger top bloom and interior streaks try to make the material feel condensed, not merely translucent.",
            );
            style.top_glow = linear_gradient(
                180.,
                linear_color_stop(rgba_hex(PEARL, 0x28), 0.0),
                linear_color_stop(rgba_hex(PEARL, 0x00), 1.0),
            );
            style.edge_light = rgba_hex(PEARL, 0x46);
            style.show_streaks = true;
            style.streak_color = rgba_hex(PEARL, 0x10);
            style.surface_noise = Some(surface_noise(rgba_hex(PEARL, 0x08)));
            style
        }
        PanelMode::Laminated => PanelStyle {
            label: "laminated sheet",
            copy: "This leans bright and laminated, using a denser white sheet and hard edge rather than a smoky tint.",
            radius: 22.0,
            fill: linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0xf6f8fb, 0xe6), 0.0),
                linear_color_stop(rgba_hex(0xe9edf4, 0xdc), 1.0),
            ),
            border: rgba_hex(0xffffff, 0x70),
            edge_light: rgba_hex(0xffffff, 0xaa),
            top_glow: linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0xffffff, 0x34), 0.0),
                linear_color_stop(rgba_hex(0xffffff, 0x00), 1.0),
            ),
            title: rgba_hex(INK, 0xf4),
            body: rgba_hex(STEEL, 0xd0),
            badge_bg: rgba_hex(0xffffff, 0x2c),
            badge_border: rgba_hex(0xffffff, 0x46),
            streak_color: rgba_hex(0xffffff, 0x00),
            show_streaks: false,
            inner_ghost: Some(proxy_text(rgba_hex(INK, 0x12), px(22.), px(60.))),
            bottom_edge: None,
            surface_noise: None,
            shadow: panel_shadow(0x14, PEARL, 0x10, 20.0),
        },
        PanelMode::EdgeStrip => {
            let mut style = dark_panel(
                "thickened edge",
                "A separate lower edge helps the card read like a slab with thickness instead of a single transparent rectangle.",
            );
            style.radius = 20.0;
            style.bottom_edge = Some(
                div()
                    .absolute()
                    .left(px(0.))
                    .right(px(0.))
                    .bottom(px(0.))
                    .h(px(10.))
                    .bg(rgba_hex(PEARL, 0x10))
                    .child(
                        div()
                            .absolute()
                            .left(px(0.))
                            .right(px(0.))
                            .top(px(0.))
                            .h(px(1.))
                            .bg(rgba_hex(PEARL, 0x20)),
                    ),
            );
            style
        }
        PanelMode::Halo => {
            let mut style = dark_panel(
                "halo separation",
                "The fill stays quieter here. Halo, rim light, and suppressed backdrop do more of the separation work.",
            );
            style.radius = 28.0;
            style.border = rgba_hex(MINT, 0x1a);
            style.edge_light = rgba_hex(ICE, 0x34);
            style.top_glow = linear_gradient(
                180.,
                linear_color_stop(rgba_hex(ICE, 0x18), 0.0),
                linear_color_stop(rgba_hex(ICE, 0x00), 1.0),
            );
            style.badge_bg = rgba_hex(MINT, 0x10);
            style.badge_border = rgba_hex(ICE, 0x22);
            style.shadow = panel_shadow(0x18, ICE, 0x18, 34.0);
            style
        }
        PanelMode::Noise => {
            let mut style = dark_panel(
                "surface grain",
                "Fine grain breaks the perfect alpha-card look and makes the panel feel more like physical material.",
            );
            style.inner_ghost = Some(proxy_text(rgba_hex(PEARL, 0x12), px(20.), px(58.)));
            style.surface_noise = Some(surface_noise(rgba_hex(PEARL, 0x10)));
            style
        }
        PanelMode::Bleached => PanelStyle {
            label: "washed backdrop",
            copy: "This imitates CSS brightness tricks by bleaching the backdrop instead of deepening the card.",
            radius: 22.0,
            fill: linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0xeef3f9, 0xbc), 0.0),
                linear_color_stop(rgba_hex(0xe4e9f1, 0xc8), 1.0),
            ),
            border: rgba_hex(0xffffff, 0x5c),
            edge_light: rgba_hex(0xffffff, 0x8a),
            top_glow: linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0xffffff, 0x26), 0.0),
                linear_color_stop(rgba_hex(0xffffff, 0x00), 1.0),
            ),
            title: rgba_hex(INK, 0xf4),
            body: rgba_hex(STEEL, 0xd4),
            badge_bg: rgba_hex(0xffffff, 0x26),
            badge_border: rgba_hex(0xffffff, 0x42),
            streak_color: rgba_hex(0xffffff, 0x00),
            show_streaks: false,
            inner_ghost: Some(proxy_text(rgba_hex(INK, 0x10), px(20.), px(58.))),
            bottom_edge: None,
            surface_noise: None,
            shadow: panel_shadow(0x14, PEARL, 0x10, 18.0),
        },
        PanelMode::Cool => {
            let mut style = ghost_panel(
                "cool saturation",
                "This pushes the blue channel and rim light harder, like CSS blur plus saturation tweaks to avoid muddy glass.",
                Some(proxy_text(rgba_hex(PEARL, 0x10), px(20.), px(58.))),
            );
            style.fill = linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0x183044, 0xc2), 0.0),
                linear_color_stop(rgba_hex(0x0d1822, 0xd6), 1.0),
            );
            style.border = rgba_hex(ICE, 0x2a);
            style.edge_light = rgba_hex(ICE, 0x58);
            style.top_glow = linear_gradient(
                180.,
                linear_color_stop(rgba_hex(ICE, 0x20), 0.0),
                linear_color_stop(rgba_hex(ICE, 0x00), 1.0),
            );
            style.badge_bg = rgba_hex(ICE, 0x12);
            style.badge_border = rgba_hex(ICE, 0x2c);
            style.body = rgba_hex(0xd6ecff, 0xd2);
            style.show_streaks = true;
            style.streak_color = rgba_hex(ICE, 0x10);
            style.shadow = panel_shadow(0x22, ICE, 0x18, 26.0);
            style
        }
        PanelMode::Warm => PanelStyle {
            label: "warm satin",
            copy: "A warmer satin finish makes the material feel more like amber acrylic than neutral frost.",
            radius: 22.0,
            fill: linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0x2d2217, 0xc4), 0.0),
                linear_color_stop(rgba_hex(0x17120d, 0xd8), 1.0),
            ),
            border: rgba_hex(GOLD, 0x22),
            edge_light: rgba_hex(GOLD, 0x3a),
            top_glow: linear_gradient(
                180.,
                linear_color_stop(rgba_hex(GOLD, 0x18), 0.0),
                linear_color_stop(rgba_hex(GOLD, 0x00), 1.0),
            ),
            title: rgba_hex(0xfffbeb, 0xf8),
            body: rgba_hex(0xf8e7c2, 0xcc),
            badge_bg: rgba_hex(GOLD, 0x12),
            badge_border: rgba_hex(GOLD, 0x26),
            streak_color: rgba_hex(0xfffbeb, 0x08),
            show_streaks: true,
            inner_ghost: None,
            bottom_edge: None,
            surface_noise: None,
            shadow: panel_shadow(0x24, GOLD, 0x14, 24.0),
        },
        PanelMode::Quiet => {
            let mut style = dark_panel(
                "windowed proxy",
                "A stronger clipped backdrop window sits underneath while the card itself stays quieter.",
            );
            style.radius = 26.0;
            style.fill = linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0x232831, 0x9e), 0.0),
                linear_color_stop(rgba_hex(0x151920, 0xb4), 1.0),
            );
            style.border = rgba_hex(PEARL, 0x18);
            style.edge_light = rgba_hex(PEARL, 0x38);
            style.inner_ghost = Some(proxy_text(rgba_hex(PEARL, 0x10), px(20.), px(60.)));
            style.shadow = panel_shadow(0x20, PEARL, 0x0f, 22.0);
            style
        }
        PanelMode::Offset => {
            let mut style = ghost_panel(
                "offset backdrop",
                "The echoed backdrop is shifted more aggressively so the panel feels optically displaced.",
                Some(proxy_text(rgba_hex(PEARL, 0x12), px(26.), px(48.))),
            );
            style.shadow = panel_shadow(0x20, ICE, 0x12, 24.0);
            style
        }
        PanelMode::Best => {
            let mut style = ghost_panel(
                "combined target",
                "Suppression, ghosting, edge light, grain, and a lower edge work together instead of relying on alpha alone.",
                Some(proxy_text(rgba_hex(PEARL, 0x18), px(20.), px(56.))),
            );
            style.fill = linear_gradient(
                180.,
                linear_color_stop(rgba_hex(0x262c35, 0xc8), 0.0),
                linear_color_stop(rgba_hex(0x101317, 0xe0), 1.0),
            );
            style.border = rgba_hex(PEARL, 0x22);
            style.edge_light = rgba_hex(PEARL, 0x52);
            style.badge_bg = rgba_hex(GOLD, 0x12);
            style.badge_border = rgba_hex(GOLD, 0x26);
            style.show_streaks = true;
            style.streak_color = rgba_hex(PEARL, 0x0d);
            style.bottom_edge = Some(
                div()
                    .absolute()
                    .left(px(0.))
                    .right(px(0.))
                    .bottom(px(0.))
                    .h(px(8.))
                    .bg(rgba_hex(PEARL, 0x12)),
            );
            style.surface_noise = Some(surface_noise(rgba_hex(PEARL, 0x0a)));
            style.shadow = panel_shadow(0x22, GOLD, 0x14, 28.0);
            style
        }
    }
}

fn dark_panel(label: &'static str, copy: &'static str) -> PanelStyle {
    PanelStyle {
        label,
        copy,
        radius: 22.0,
        fill: linear_gradient(
            180.,
            linear_color_stop(rgba_hex(0x242a33, 0xd0), 0.0),
            linear_color_stop(rgba_hex(0x11151b, 0xcc), 1.0),
        ),
        border: rgba_hex(PEARL, 0x20),
        edge_light: rgba_hex(PEARL, 0x32),
        top_glow: linear_gradient(
            180.,
            linear_color_stop(rgba_hex(PEARL, 0x12), 0.0),
            linear_color_stop(rgba_hex(PEARL, 0x00), 1.0),
        ),
        title: rgba_hex(PEARL, 0xf8),
        body: rgba_hex(PEARL, 0xce),
        badge_bg: rgba_hex(PEARL, 0x10),
        badge_border: rgba_hex(PEARL, 0x20),
        streak_color: rgba_hex(PEARL, 0x00),
        show_streaks: false,
        inner_ghost: None,
        bottom_edge: None,
        surface_noise: None,
        shadow: panel_shadow(0x24, PEARL, 0x0e, 24.0),
    }
}

fn ghost_panel(label: &'static str, copy: &'static str, inner_ghost: Option<Div>) -> PanelStyle {
    PanelStyle {
        label,
        copy,
        radius: 24.0,
        fill: linear_gradient(
            180.,
            linear_color_stop(rgba_hex(0x1e2632, 0xc4), 0.0),
            linear_color_stop(rgba_hex(0x11161e, 0xcc), 1.0),
        ),
        border: rgba_hex(ICE, 0x24),
        edge_light: rgba_hex(ICE, 0x36),
        top_glow: linear_gradient(
            180.,
            linear_color_stop(rgba_hex(ICE, 0x16), 0.0),
            linear_color_stop(rgba_hex(ICE, 0x00), 1.0),
        ),
        title: rgba_hex(PEARL, 0xfa),
        body: rgba_hex(0xd8ebff, 0xcc),
        badge_bg: rgba_hex(ICE, 0x12),
        badge_border: rgba_hex(ICE, 0x28),
        streak_color: rgba_hex(PEARL, 0x00),
        show_streaks: false,
        inner_ghost,
        bottom_edge: None,
        surface_noise: None,
        shadow: panel_shadow(0x22, ICE, 0x14, 26.0),
    }
}

fn panel_shadow(base_alpha: u8, accent: u32, accent_alpha: u8, blur: f32) -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: hsla_hex(0x000000, base_alpha),
            offset: point(px(0.), px(14.)),
            blur_radius: px(blur),
            spread_radius: px(0.),
        },
        BoxShadow {
            color: hsla_hex(accent, accent_alpha),
            offset: point(px(0.), px(2.)),
            blur_radius: px(18.),
            spread_radius: px(-6.),
        },
    ]
}

fn footer_chip(label: &str, value: &str) -> Div {
    div()
        .px(px(10.))
        .py(px(5.))
        .rounded(px(999.))
        .bg(rgba_hex(0xffffff, 0x07))
        .border_1()
        .border_color(rgba_hex(0xffffff, 0x14))
        .flex()
        .flex_row()
        .gap(px(6.))
        .child(
            div()
                .text_xs()
                .text_color(rgba_hex(0xffffff, 0x88))
                .child(label.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgba_hex(0xffffff, 0xe8))
                .child(value.to_string()),
        )
}

fn rgba_hex(hex: u32, alpha: u8) -> Rgba {
    rgba((hex << 8) | alpha as u32)
}

fn hsla_hex(hex: u32, alpha: u8) -> Hsla {
    Hsla::from(rgba_hex(hex, alpha))
}

#[cfg(test)]
mod tests {
    use super::{FrostedSurfaceVariationsStory, VARIANTS};
    use crate::storybook::Story;

    #[test]
    fn frosted_surface_story_exposes_fifteen_variants() {
        let story = FrostedSurfaceVariationsStory;
        assert_eq!(story.variants().len(), 15);
        assert_eq!(story.variants().len(), VARIANTS.len());
    }

    #[test]
    fn frosted_surface_variant_ids_are_unique() {
        let mut ids: Vec<&str> = VARIANTS.iter().map(|variant| variant.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), VARIANTS.len());
    }
}
