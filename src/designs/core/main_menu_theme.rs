use std::sync::atomic::{AtomicU8, Ordering};

use gpui::FontWeight;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum MainMenuThemeVariant {
    #[default]
    TahoeClear = 1,
    TahoeGraphite = 2,
    TahoeBlueGlass = 3,
    TahoeSmoke = 4,
    TahoeWarmGold = 5,
    FrostedCommand = 6,
    LiquidPrism = 7,
    AuroraSlate = 8,
    MilkGlass = 9,
    ProConsole = 10,
    SpotlightLuxe = 11,
    OceanGlass = 12,
    CarbonNeon = 13,
    StudioPaperGlass = 14,
    OperatorMonoGlass = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuThemeTier {
    TahoeClose,
    ExpressiveTahoe,
    Exploratory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuRowKind {
    IconTile,
    GraphitePill,
    BlueGlass,
    Smoke,
    WarmGold,
    FrostedCommand,
    LiquidPrism,
    AuroraSlate,
    MilkGlass,
    ProConsole,
    SpotlightLuxe,
    OceanGlass,
    CarbonNeon,
    StudioPaperGlass,
    OperatorMonoGlass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterButtonTheme {
    pub rest: Option<u32>,
    pub hover: u32,
    pub active: u32,
    pub border_alpha: u32,
    pub uses_accent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenuShellTokens {
    pub content_inset_x: f32,
    pub content_inset_bottom: f32,
    pub header_padding_x: f32,
    pub header_padding_y: f32,
    pub header_gap: f32,
    pub divider_margin_x: f32,
    pub divider_height: f32,
    pub divider_alpha: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenuSearchTokens {
    pub height: f32,
    pub radius: f32,
    pub text_inset_x: f32,
    pub text_inset_y: f32,
    pub surface_alpha: u32,
    pub border_alpha: u32,
    pub font_size: f32,
    pub font_weight: FontWeight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenuListTokens {
    pub item_height: f32,
    pub section_header_height: f32,
    pub first_section_header_height: f32,
    pub average_scroll_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenuRowTokens {
    pub outer_padding_x: f32,
    pub outer_padding_y: f32,
    pub inner_padding_x: f32,
    pub inner_padding_y: f32,
    pub radius: f32,
    pub name_desc_gap: f32,
    pub icon_text_gap: f32,
    pub accessory_gap: f32,
    pub selected_border_width: f32,
    pub selected_border_alpha: u32,
    pub selected_fill_alpha: u32,
    pub hover_fill_alpha: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenuIconTokens {
    pub container_size: f32,
    pub svg_size: f32,
    pub tile_size: f32,
    pub tile_radius: f32,
    pub tile_fill_alpha: u32,
    pub tile_border_alpha: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenuTypographyTokens {
    pub name_font_size: f32,
    pub name_line_height: f32,
    pub name_weight: FontWeight,
    pub selected_name_weight: FontWeight,
    pub desc_font_size: f32,
    pub desc_line_height: f32,
    pub desc_weight: FontWeight,
    pub section_font_size: f32,
    pub section_line_height: f32,
    pub section_weight: FontWeight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenuMetadataTokens {
    pub metadata_alpha: u32,
    pub type_accessory_size: f32,
    pub source_font_size: f32,
    pub badge_font_size: f32,
    pub keycap_font_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FooterMetricsTokens {
    pub side_inset_px: f32,
    pub item_gap_px: f32,
    pub content_gap: f32,
    pub button_padding_x: f32,
    pub button_padding_y: f32,
    pub run_button_padding_x: f32,
    pub button_radius: f32,
    pub label_font_size: f32,
    pub keycap_padding_x: f32,
    pub keycap_padding_y: f32,
    pub keycap_radius: f32,
    pub keycap_font_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FooterTheme {
    pub text_accent: bool,
    pub keycap_accent: bool,
    pub divider_accent: bool,
    pub divider_alpha: u32,
    pub button: FooterButtonTheme,
    pub metrics: FooterMetricsTokens,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenuThemeDef {
    pub variant: MainMenuThemeVariant,
    pub name: &'static str,
    pub intent: &'static str,
    pub tier: MainMenuThemeTier,
    pub row_kind: MainMenuRowKind,
    pub shell: MainMenuShellTokens,
    pub search: MainMenuSearchTokens,
    pub list: MainMenuListTokens,
    pub row: MainMenuRowTokens,
    pub icon: MainMenuIconTokens,
    pub typography: MainMenuTypographyTokens,
    pub metadata: MainMenuMetadataTokens,
    pub footer: FooterTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MainMenuGeometrySignature {
    pub shell_inset_x: u32,
    pub header_padding_x: u32,
    pub search_height: u32,
    pub search_radius: u32,
    pub row_height: u32,
    pub row_padding_x: u32,
    pub row_radius: u32,
    pub icon_container_size: u32,
    pub icon_tile_size: u32,
    pub name_font_size: u32,
    pub desc_font_size: u32,
    pub metadata_alpha: u32,
    pub footer_gap: u32,
    pub footer_button_radius: u32,
}

impl MainMenuThemeVariant {
    pub const COUNT: usize = 15;

    pub fn all() -> &'static [MainMenuThemeVariant] {
        use MainMenuThemeVariant::*;
        &[
            TahoeClear,
            TahoeGraphite,
            TahoeBlueGlass,
            TahoeSmoke,
            TahoeWarmGold,
            FrostedCommand,
            LiquidPrism,
            AuroraSlate,
            MilkGlass,
            ProConsole,
            SpotlightLuxe,
            OceanGlass,
            CarbonNeon,
            StudioPaperGlass,
            OperatorMonoGlass,
        ]
    }

    pub fn from_u8(value: u8) -> MainMenuThemeVariant {
        Self::all()
            .iter()
            .copied()
            .find(|v| *v as u8 == value)
            .unwrap_or_default()
    }

    pub fn next(self) -> MainMenuThemeVariant {
        let all = Self::all();
        let idx = all.iter().position(|&v| v == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    pub fn prev(self) -> MainMenuThemeVariant {
        let all = Self::all();
        let idx = all.iter().position(|&v| v == self).unwrap_or(0);
        all[if idx == 0 { all.len() - 1 } else { idx - 1 }]
    }

    pub fn index(self) -> usize {
        Self::all().iter().position(|&v| v == self).unwrap_or(0)
    }

    pub fn name(self) -> &'static str {
        self.def().name
    }

    pub fn placeholder(self) -> String {
        format!(
            "Theme {}/{} · {}   ·   alt+\u{2190}/\u{2192} to cycle",
            self.index() + 1,
            Self::COUNT,
            self.name()
        )
    }

    pub fn def(self) -> MainMenuThemeDef {
        use MainMenuThemeTier::*;
        use MainMenuThemeVariant::*;

        let accent_button = |rest, hover, active, border_alpha| FooterButtonTheme {
            rest: Some(rest),
            hover,
            active,
            border_alpha,
            uses_accent: true,
        };
        let neutral_button = |rest, hover, active, border_alpha| FooterButtonTheme {
            rest,
            hover,
            active,
            border_alpha,
            uses_accent: false,
        };
        let footer = |button, divider_alpha, metrics| FooterTheme {
            text_accent: false,
            keycap_accent: false,
            divider_accent: false,
            divider_alpha,
            button,
            metrics,
        };
        let metrics = |side_inset_px,
                       item_gap_px,
                       content_gap,
                       button_padding_x,
                       button_padding_y,
                       run_button_padding_x,
                       button_radius,
                       label_font_size,
                       keycap_padding_x,
                       keycap_padding_y,
                       keycap_radius,
                       keycap_font_size| FooterMetricsTokens {
            side_inset_px,
            item_gap_px,
            content_gap,
            button_padding_x,
            button_padding_y,
            run_button_padding_x,
            button_radius,
            label_font_size,
            keycap_padding_x,
            keycap_padding_y,
            keycap_radius,
            keycap_font_size,
        };
        let def = |variant: MainMenuThemeVariant,
                   name: &'static str,
                   intent: &'static str,
                   tier: MainMenuThemeTier,
                   row_kind: MainMenuRowKind,
                   footer: FooterTheme,
                   shell_inset: f32,
                   header_y: f32,
                   header_gap: f32,
                   search_height: f32,
                   search_radius: f32,
                   row_height: f32,
                   section_height: f32,
                   first_section_height: f32,
                   row_outer_x: f32,
                   row_outer_y: f32,
                   row_inner_x: f32,
                   row_inner_y: f32,
                   row_radius: f32,
                   icon_container: f32,
                   icon_svg: f32,
                   icon_tile: f32,
                   icon_tile_radius: f32,
                   name_size: f32,
                   name_line: f32,
                   desc_size: f32,
                   desc_line: f32,
                   section_size: f32,
                   section_line: f32,
                   metadata_alpha: u32,
                   type_accessory_size: f32,
                   source_font_size: f32,
                   badge_font_size: f32,
                   selected_border_width: f32,
                   selected_border_alpha: u32,
                   selected_fill_alpha: u32,
                   hover_fill_alpha: u32|
         -> MainMenuThemeDef {
            MainMenuThemeDef {
                variant,
                name,
                intent,
                tier,
                row_kind,
                shell: MainMenuShellTokens {
                    content_inset_x: shell_inset,
                    content_inset_bottom: shell_inset * 0.35,
                    header_padding_x: shell_inset,
                    header_padding_y: header_y,
                    header_gap,
                    divider_margin_x: shell_inset,
                    divider_height: 1.0,
                    divider_alpha: footer.divider_alpha,
                },
                search: MainMenuSearchTokens {
                    height: search_height,
                    radius: search_radius,
                    text_inset_x: (shell_inset * 0.55).clamp(8.0, 16.0),
                    text_inset_y: 0.0,
                    surface_alpha: match tier {
                        TahoeClose => 0x00,
                        ExpressiveTahoe => 0x12,
                        Exploratory => 0x18,
                    },
                    border_alpha: selected_border_alpha.min(0x80),
                    font_size: (name_size + 1.0).clamp(14.0, 17.0),
                    font_weight: FontWeight(430.0),
                },
                list: MainMenuListTokens {
                    item_height: row_height,
                    section_header_height: section_height,
                    first_section_header_height: first_section_height,
                    average_scroll_height: ((row_height * 3.0) + section_height) / 4.0,
                },
                row: MainMenuRowTokens {
                    outer_padding_x: row_outer_x,
                    outer_padding_y: row_outer_y,
                    inner_padding_x: row_inner_x,
                    inner_padding_y: row_inner_y,
                    radius: row_radius,
                    name_desc_gap: (row_height / 18.0).clamp(1.5, 3.5),
                    icon_text_gap: (icon_container * 0.42).clamp(7.0, 12.0),
                    accessory_gap: (row_inner_x * 0.45).clamp(5.0, 9.0),
                    selected_border_width,
                    selected_border_alpha,
                    selected_fill_alpha,
                    hover_fill_alpha,
                },
                icon: MainMenuIconTokens {
                    container_size: icon_container,
                    svg_size: icon_svg,
                    tile_size: icon_tile,
                    tile_radius: icon_tile_radius,
                    tile_fill_alpha: selected_fill_alpha.max(0x80),
                    tile_border_alpha: selected_border_alpha,
                },
                typography: MainMenuTypographyTokens {
                    name_font_size: name_size,
                    name_line_height: name_line,
                    name_weight: FontWeight(450.0),
                    selected_name_weight: if matches!(
                        row_kind,
                        MainMenuRowKind::CarbonNeon
                            | MainMenuRowKind::ProConsole
                            | MainMenuRowKind::OperatorMonoGlass
                    ) {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    },
                    desc_font_size: desc_size,
                    desc_line_height: desc_line,
                    desc_weight: FontWeight::NORMAL,
                    section_font_size: section_size,
                    section_line_height: section_line,
                    section_weight: if matches!(
                        row_kind,
                        MainMenuRowKind::AuroraSlate
                            | MainMenuRowKind::StudioPaperGlass
                            | MainMenuRowKind::OperatorMonoGlass
                    ) {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::NORMAL
                    },
                },
                metadata: MainMenuMetadataTokens {
                    metadata_alpha,
                    type_accessory_size,
                    source_font_size,
                    badge_font_size,
                    keycap_font_size: footer.metrics.keycap_font_size,
                },
                footer,
            }
        };

        match self {
            TahoeClear => def(
                self,
                "Tahoe Clear",
                "Clean native command palette with nearly invisible footer chrome",
                TahoeClose,
                MainMenuRowKind::IconTile,
                footer(
                    neutral_button(None, 0x10, 0x24, 0x12),
                    0x0A,
                    metrics(
                        14.0, 6.0, 4.0, 4.0, 2.0, 6.0, 6.0, 12.0, 5.0, 2.0, 6.0, 12.0,
                    ),
                ),
                16.0,
                10.0,
                10.0,
                30.0,
                9.0,
                40.0,
                32.0,
                26.0,
                4.0,
                2.0,
                14.0,
                4.0,
                8.0,
                20.0,
                16.0,
                20.0,
                7.0,
                14.0,
                20.0,
                12.0,
                16.0,
                12.0,
                16.0,
                0x73,
                12.0,
                11.0,
                10.0,
                0.0,
                0x00,
                0x20,
                0x12,
            ),
            TahoeGraphite => def(
                self,
                "Tahoe Graphite",
                "Restrained graphite structure with grouped footer controls",
                TahoeClose,
                MainMenuRowKind::GraphitePill,
                footer(
                    neutral_button(Some(0x06), 0x14, 0x28, 0x14),
                    0x00,
                    metrics(
                        15.0, 7.0, 4.5, 4.5, 2.0, 6.5, 7.0, 12.0, 5.0, 2.0, 6.0, 12.0,
                    ),
                ),
                18.0,
                10.0,
                11.0,
                32.0,
                11.0,
                42.0,
                32.0,
                27.0,
                4.0,
                2.0,
                15.0,
                4.0,
                9.0,
                22.0,
                17.0,
                22.0,
                7.0,
                14.0,
                20.0,
                12.0,
                16.0,
                12.0,
                16.0,
                0x80,
                12.0,
                11.0,
                10.0,
                1.0,
                0x20,
                0x26,
                0x14,
            ),
            TahoeBlueGlass => def(
                self,
                "Tahoe Blue Glass",
                "System-blue Liquid Glass accents without over-coloring metadata",
                TahoeClose,
                MainMenuRowKind::BlueGlass,
                footer(
                    accent_button(0x00, 0x12, 0x30, 0x12),
                    0x14,
                    metrics(
                        15.0, 7.0, 5.0, 5.0, 2.0, 7.0, 8.0, 12.0, 5.0, 2.0, 6.0, 12.0,
                    ),
                ),
                18.0,
                11.0,
                12.0,
                34.0,
                12.0,
                44.0,
                34.0,
                28.0,
                5.0,
                2.0,
                15.0,
                5.0,
                10.0,
                24.0,
                18.0,
                24.0,
                8.0,
                14.2,
                20.0,
                12.0,
                16.0,
                12.0,
                16.0,
                0x88,
                12.0,
                11.0,
                10.0,
                1.0,
                0x44,
                0x24,
                0x12,
            ),
            TahoeSmoke => def(
                self,
                "Tahoe Smoke",
                "Smoky low-contrast native palette with whisper footer states",
                TahoeClose,
                MainMenuRowKind::Smoke,
                footer(
                    neutral_button(None, 0x0D, 0x1F, 0x00),
                    0x00,
                    metrics(
                        12.0, 5.0, 3.5, 3.5, 1.5, 5.0, 5.0, 11.5, 4.0, 1.5, 5.0, 11.5,
                    ),
                ),
                14.0,
                8.0,
                8.0,
                30.0,
                8.0,
                38.0,
                28.0,
                22.0,
                3.0,
                1.0,
                12.0,
                3.0,
                6.0,
                18.0,
                15.0,
                18.0,
                5.0,
                13.5,
                18.0,
                11.0,
                15.0,
                11.0,
                15.0,
                0x62,
                11.0,
                10.0,
                9.5,
                0.0,
                0x00,
                0x16,
                0x0D,
            ),
            TahoeWarmGold => def(
                self,
                "Tahoe Warm Gold",
                "Script Kit gold signature translated into native active states",
                TahoeClose,
                MainMenuRowKind::WarmGold,
                footer(
                    accent_button(0x08, 0x20, 0x3A, 0x12),
                    0x18,
                    metrics(
                        14.0, 6.0, 4.5, 4.5, 2.0, 6.5, 7.0, 12.0, 5.0, 2.0, 6.0, 12.0,
                    ),
                ),
                16.0,
                10.0,
                10.0,
                32.0,
                10.0,
                42.0,
                33.0,
                27.0,
                4.0,
                2.0,
                14.0,
                4.0,
                9.0,
                22.0,
                17.0,
                22.0,
                7.0,
                14.0,
                20.0,
                12.0,
                16.0,
                12.0,
                16.0,
                0x96,
                12.0,
                11.5,
                10.0,
                1.0,
                0x2A,
                0x24,
                0x10,
            ),
            FrostedCommand => def(
                self,
                "Frosted Command",
                "Stronger command-center hierarchy with segmented footer buttons",
                ExpressiveTahoe,
                MainMenuRowKind::FrostedCommand,
                footer(
                    accent_button(0x08, 0x22, 0x45, 0x22),
                    0x10,
                    metrics(
                        16.0, 8.0, 5.0, 6.0, 2.5, 7.0, 8.0, 12.5, 5.5, 2.0, 6.5, 12.0,
                    ),
                ),
                20.0,
                12.0,
                12.0,
                36.0,
                12.0,
                46.0,
                34.0,
                28.0,
                5.0,
                3.0,
                16.0,
                5.0,
                11.0,
                24.0,
                18.0,
                24.0,
                8.0,
                14.5,
                20.5,
                12.0,
                16.0,
                12.0,
                16.0,
                0xA0,
                13.0,
                11.5,
                10.5,
                1.0,
                0x48,
                0x28,
                0x14,
            ),
            LiquidPrism => def(
                self,
                "Liquid Prism",
                "Subtle prismatic glass edge with coherent button cells",
                ExpressiveTahoe,
                MainMenuRowKind::LiquidPrism,
                footer(
                    accent_button(0x07, 0x22, 0x3C, 0x14),
                    0x12,
                    metrics(
                        17.0, 9.0, 5.5, 6.0, 2.5, 7.5, 10.0, 12.5, 5.5, 2.0, 7.0, 12.0,
                    ),
                ),
                22.0,
                13.0,
                13.0,
                38.0,
                14.0,
                48.0,
                36.0,
                30.0,
                6.0,
                3.0,
                17.0,
                5.0,
                12.0,
                26.0,
                20.0,
                26.0,
                10.0,
                14.8,
                21.0,
                12.2,
                16.0,
                12.0,
                16.0,
                0xA6,
                13.0,
                11.5,
                10.5,
                1.0,
                0x50,
                0x28,
                0x12,
            ),
            AuroraSlate => def(
                self,
                "Aurora Slate",
                "Cool slate surface with aurora accent active states",
                ExpressiveTahoe,
                MainMenuRowKind::AuroraSlate,
                footer(
                    accent_button(0x0A, 0x28, 0x55, 0x20),
                    0x00,
                    metrics(
                        16.0, 8.0, 5.0, 5.5, 2.0, 7.0, 8.0, 12.0, 5.0, 2.0, 6.0, 12.0,
                    ),
                ),
                20.0,
                11.0,
                12.0,
                34.0,
                11.0,
                44.0,
                34.0,
                28.0,
                5.0,
                2.0,
                16.0,
                5.0,
                9.0,
                24.0,
                18.0,
                24.0,
                8.0,
                14.2,
                20.0,
                12.0,
                16.0,
                12.5,
                17.0,
                0xA0,
                13.0,
                11.5,
                10.5,
                1.0,
                0x44,
                0x24,
                0x12,
            ),
            MilkGlass => def(
                self,
                "Milk Glass",
                "Milky translucency that remains usable in light and dark mode",
                ExpressiveTahoe,
                MainMenuRowKind::MilkGlass,
                footer(
                    accent_button(0x08, 0x16, 0x30, 0x18),
                    0x08,
                    metrics(
                        18.0, 9.0, 5.5, 6.0, 2.5, 8.0, 11.0, 12.5, 6.0, 2.5, 7.0, 12.5,
                    ),
                ),
                24.0,
                14.0,
                14.0,
                40.0,
                16.0,
                46.0,
                36.0,
                30.0,
                6.0,
                3.0,
                17.0,
                6.0,
                14.0,
                24.0,
                18.0,
                24.0,
                11.0,
                14.8,
                21.0,
                12.2,
                16.5,
                12.0,
                16.0,
                0x94,
                13.0,
                11.5,
                10.5,
                1.0,
                0x38,
                0x24,
                0x10,
            ),
            ProConsole => def(
                self,
                "Pro Console",
                "Sharper developer command strip without retro-terminal cosplay",
                ExpressiveTahoe,
                MainMenuRowKind::ProConsole,
                footer(
                    accent_button(0x0A, 0x20, 0x4D, 0x20),
                    0x18,
                    metrics(
                        11.0, 5.0, 3.5, 3.5, 1.5, 5.0, 5.0, 11.5, 4.0, 1.5, 5.0, 11.5,
                    ),
                ),
                12.0,
                8.0,
                8.0,
                30.0,
                6.0,
                36.0,
                28.0,
                22.0,
                3.0,
                1.0,
                12.0,
                3.0,
                6.0,
                18.0,
                15.0,
                18.0,
                5.0,
                13.2,
                18.0,
                11.0,
                15.0,
                11.0,
                15.0,
                0x80,
                11.0,
                10.5,
                9.5,
                1.0,
                0x50,
                0x22,
                0x14,
            ),
            SpotlightLuxe => def(
                self,
                "Spotlight Luxe",
                "Premium Spotlight-like hero search and floating pill footer",
                ExpressiveTahoe,
                MainMenuRowKind::SpotlightLuxe,
                footer(
                    accent_button(0x06, 0x14, 0x38, 0x14),
                    0x00,
                    metrics(
                        20.0, 10.0, 6.0, 7.0, 3.0, 8.5, 13.0, 13.0, 6.0, 2.5, 8.0, 12.5,
                    ),
                ),
                28.0,
                16.0,
                16.0,
                44.0,
                18.0,
                52.0,
                38.0,
                32.0,
                7.0,
                3.0,
                18.0,
                6.0,
                16.0,
                28.0,
                22.0,
                28.0,
                12.0,
                16.0,
                23.0,
                13.0,
                18.0,
                12.5,
                17.0,
                0xA0,
                14.0,
                12.0,
                11.0,
                1.0,
                0x42,
                0x26,
                0x10,
            ),
            OceanGlass => def(
                self,
                "Ocean Glass",
                "Immersive oceanic stress-test for how far color can go",
                Exploratory,
                MainMenuRowKind::OceanGlass,
                footer(
                    accent_button(0x09, 0x24, 0x50, 0x24),
                    0x18,
                    metrics(
                        17.0, 9.0, 5.5, 6.0, 2.5, 7.5, 10.0, 12.5, 5.5, 2.0, 7.0, 12.0,
                    ),
                ),
                22.0,
                13.0,
                13.0,
                38.0,
                14.0,
                48.0,
                36.0,
                30.0,
                6.0,
                3.0,
                17.0,
                5.0,
                12.0,
                26.0,
                20.0,
                26.0,
                10.0,
                15.0,
                21.0,
                12.2,
                16.0,
                12.0,
                16.0,
                0xA8,
                13.0,
                11.8,
                10.5,
                1.0,
                0x58,
                0x30,
                0x14,
            ),
            CarbonNeon => def(
                self,
                "Carbon Neon",
                "High-contrast neon edge as a deliberate contrast limit",
                Exploratory,
                MainMenuRowKind::CarbonNeon,
                footer(
                    accent_button(0x0C, 0x35, 0x70, 0x40),
                    0x22,
                    metrics(
                        15.0, 8.0, 5.0, 5.5, 2.0, 7.0, 6.0, 12.0, 5.0, 2.0, 5.0, 12.0,
                    ),
                ),
                18.0,
                10.0,
                11.0,
                34.0,
                8.0,
                42.0,
                32.0,
                26.0,
                4.0,
                2.0,
                14.0,
                4.0,
                6.0,
                22.0,
                17.0,
                22.0,
                6.0,
                14.5,
                20.0,
                12.0,
                16.0,
                12.0,
                16.0,
                0xA0,
                12.0,
                11.5,
                10.5,
                2.0,
                0x80,
                0x28,
                0x0C,
            ),
            StudioPaperGlass => def(
                self,
                "Studio Paper Glass",
                "Warm editorial paper influence blended into glass",
                Exploratory,
                MainMenuRowKind::StudioPaperGlass,
                footer(
                    accent_button(0x08, 0x1C, 0x3A, 0x20),
                    0x12,
                    metrics(
                        19.0, 9.0, 5.5, 6.5, 2.5, 8.0, 12.0, 12.5, 6.0, 2.0, 7.0, 12.5,
                    ),
                ),
                26.0,
                15.0,
                15.0,
                40.0,
                15.0,
                50.0,
                40.0,
                34.0,
                7.0,
                3.0,
                18.0,
                6.0,
                13.0,
                24.0,
                18.0,
                24.0,
                10.0,
                15.5,
                22.0,
                12.5,
                17.0,
                13.0,
                18.0,
                0x98,
                13.0,
                12.0,
                10.5,
                1.0,
                0x48,
                0x26,
                0x12,
            ),
            OperatorMonoGlass => def(
                self,
                "Operator Mono Glass",
                "Operator command mood with equalized footer states",
                Exploratory,
                MainMenuRowKind::OperatorMonoGlass,
                footer(
                    accent_button(0x08, 0x16, 0x48, 0x20),
                    0x14,
                    metrics(
                        15.0, 7.0, 5.0, 5.0, 2.0, 7.0, 8.0, 12.0, 5.0, 2.0, 6.0, 12.0,
                    ),
                ),
                18.0,
                11.0,
                11.0,
                34.0,
                10.0,
                44.0,
                34.0,
                28.0,
                4.0,
                2.0,
                15.0,
                4.0,
                8.0,
                20.0,
                16.0,
                20.0,
                6.0,
                14.0,
                20.0,
                12.0,
                16.0,
                12.0,
                16.0,
                0x90,
                12.0,
                11.5,
                10.0,
                1.0,
                0x44,
                0x24,
                0x10,
            ),
        }
    }

    pub fn geometry_signature(self) -> MainMenuGeometrySignature {
        let def = self.def();
        let q = |value: f32| (value * 10.0).round() as u32;
        MainMenuGeometrySignature {
            shell_inset_x: q(def.shell.content_inset_x),
            header_padding_x: q(def.shell.header_padding_x),
            search_height: q(def.search.height),
            search_radius: q(def.search.radius),
            row_height: q(def.list.item_height),
            row_padding_x: q(def.row.inner_padding_x),
            row_radius: q(def.row.radius),
            icon_container_size: q(def.icon.container_size),
            icon_tile_size: q(def.icon.tile_size),
            name_font_size: q(def.typography.name_font_size),
            desc_font_size: q(def.typography.desc_font_size),
            metadata_alpha: def.metadata.metadata_alpha,
            footer_gap: q(def.footer.metrics.item_gap_px),
            footer_button_radius: q(def.footer.metrics.button_radius),
        }
    }
}

static CURRENT_MAIN_MENU_THEME: AtomicU8 = AtomicU8::new(MainMenuThemeVariant::TahoeClear as u8);

pub fn set_current_main_menu_theme(theme: MainMenuThemeVariant) {
    CURRENT_MAIN_MENU_THEME.store(theme as u8, Ordering::Relaxed);
}

pub fn current_main_menu_theme() -> MainMenuThemeVariant {
    MainMenuThemeVariant::from_u8(CURRENT_MAIN_MENU_THEME.load(Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::{current_main_menu_theme, set_current_main_menu_theme, MainMenuThemeVariant};
    use std::collections::HashSet;

    #[test]
    fn main_menu_theme_has_exactly_fifteen_variants() {
        assert_eq!(
            MainMenuThemeVariant::all().len(),
            MainMenuThemeVariant::COUNT
        );
        assert_eq!(MainMenuThemeVariant::COUNT, 15);
        for v in MainMenuThemeVariant::all() {
            assert_eq!(MainMenuThemeVariant::from_u8(*v as u8), *v);
        }
    }

    #[test]
    fn main_menu_theme_cycles_forward_and_backward() {
        assert_eq!(
            MainMenuThemeVariant::OperatorMonoGlass.next(),
            MainMenuThemeVariant::TahoeClear
        );
        assert_eq!(
            MainMenuThemeVariant::TahoeClear.prev(),
            MainMenuThemeVariant::OperatorMonoGlass
        );
        let mut v = MainMenuThemeVariant::default();
        for _ in 0..MainMenuThemeVariant::COUNT {
            v = v.next();
        }
        assert_eq!(v, MainMenuThemeVariant::default());
    }

    #[test]
    fn theme_names_and_placeholders_are_identifiable() {
        for theme in MainMenuThemeVariant::all() {
            assert!(!theme.name().trim().is_empty());
            assert!(theme.placeholder().contains(theme.name()));
            assert!(theme.placeholder().contains("/15"));
        }
    }

    #[test]
    fn footer_button_state_ladder_never_gets_weaker() {
        for theme in MainMenuThemeVariant::all() {
            let button = theme.def().footer.button;
            let rest = button.rest.unwrap_or(0);
            assert!(
                rest <= button.hover && button.hover <= button.active,
                "{theme:?} must satisfy rest <= hover <= active"
            );
        }
    }

    #[test]
    fn default_is_tahoe_clear_and_first() {
        assert_eq!(
            MainMenuThemeVariant::default(),
            MainMenuThemeVariant::TahoeClear
        );
        assert_eq!(MainMenuThemeVariant::default().index(), 0);
    }

    #[test]
    fn global_round_trips_through_u8() {
        for v in MainMenuThemeVariant::all() {
            set_current_main_menu_theme(*v);
            assert_eq!(current_main_menu_theme(), *v);
        }
        set_current_main_menu_theme(MainMenuThemeVariant::default());
    }

    #[test]
    fn every_theme_has_a_unique_geometry_signature() {
        let mut signatures = HashSet::new();
        for theme in MainMenuThemeVariant::all() {
            assert!(
                signatures.insert(theme.geometry_signature()),
                "{theme:?} reused a main-menu geometry signature"
            );
        }
    }

    #[test]
    fn tahoe_themes_are_close_but_not_identical() {
        let tahoe = &MainMenuThemeVariant::all()[0..5];
        let row_heights = tahoe
            .iter()
            .map(|theme| theme.def().list.item_height as u32)
            .collect::<HashSet<_>>();
        let search_heights = tahoe
            .iter()
            .map(|theme| theme.def().search.height as u32)
            .collect::<HashSet<_>>();
        assert!(row_heights.len() >= 4);
        assert!(search_heights.len() >= 3);
        assert!(tahoe
            .iter()
            .all(|theme| matches!(theme.def().tier, super::MainMenuThemeTier::TahoeClose)));
    }

    #[test]
    fn exploratory_themes_have_larger_structural_differences() {
        let compact = MainMenuThemeVariant::ProConsole.def();
        let luxe = MainMenuThemeVariant::SpotlightLuxe.def();
        assert!(luxe.list.item_height - compact.list.item_height >= 16.0);
        assert!(luxe.search.height - compact.search.height >= 14.0);
        assert!(luxe.shell.content_inset_x - compact.shell.content_inset_x >= 16.0);
        assert!(luxe.icon.container_size - compact.icon.container_size >= 10.0);
    }
}
