//! Pixel-perfect main menu representation for storybook.
//!
//! This story intentionally renders a runtime-captured launcher snapshot instead
//! of reconstructing the UI from approximated components. For this surface,
//! exactness matters more than interactivity.

use gpui::*;
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use super::StoryVariant;

const MAIN_MENU_SNAPSHOT_FILE: &str = "test-screenshots/main-menu-reference.png";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MainMenuVariationId {
    CurrentMainMenu,
}

impl MainMenuVariationId {
    pub const ALL: [Self; 1] = [Self::CurrentMainMenu];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => "current-main-menu",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => "Current Main Menu",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::CurrentMainMenu => {
                "Pixel-perfect runtime snapshot of the real launcher main menu"
            }
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "current-main-menu" => Some(Self::CurrentMainMenu),
            _ => None,
        }
    }
}

#[derive(Clone)]
struct MainMenuSnapshot {
    image: Arc<RenderImage>,
    width: u32,
    height: u32,
}

fn main_menu_snapshot_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(MAIN_MENU_SNAPSHOT_FILE)
}

fn load_main_menu_snapshot() -> Option<MainMenuSnapshot> {
    static SNAPSHOT: OnceLock<Option<MainMenuSnapshot>> = OnceLock::new();

    SNAPSHOT
        .get_or_init(|| {
            let path = main_menu_snapshot_path();
            let bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    tracing::warn!(
                        event = "storybook_main_menu_snapshot_missing",
                        path = %path.display(),
                        error = %error,
                        "Main menu story snapshot is unavailable"
                    );
                    return None;
                }
            };

            let dimensions = match image::load_from_memory(&bytes) {
                Ok(image) => image.dimensions(),
                Err(error) => {
                    tracing::warn!(
                        event = "storybook_main_menu_snapshot_dimensions_failed",
                        path = %path.display(),
                        error = %error,
                        "Failed to read main menu snapshot dimensions"
                    );
                    return None;
                }
            };

            let render_image =
                match crate::list_item::decode_png_to_render_image_with_bgra_conversion(&bytes) {
                    Ok(image) => image,
                    Err(error) => {
                        tracing::warn!(
                            event = "storybook_main_menu_snapshot_decode_failed",
                            path = %path.display(),
                            error = %error,
                            "Failed to decode main menu snapshot for storybook"
                        );
                        return None;
                    }
                };

            Some(MainMenuSnapshot {
                image: render_image,
                width: dimensions.0,
                height: dimensions.1,
            })
        })
        .clone()
}

pub fn main_menu_story_variants() -> Vec<StoryVariant> {
    MainMenuVariationId::ALL
        .iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "main-menu")
                .with_prop("representation", "runtime-snapshot")
                .with_prop("snapshot", MAIN_MENU_SNAPSHOT_FILE)
                .with_prop("variantId", id.as_str())
        })
        .collect()
}

pub fn render_main_menu_story_preview(stable_id: &str) -> AnyElement {
    let _ = MainMenuVariationId::from_stable_id(stable_id)
        .unwrap_or(MainMenuVariationId::CurrentMainMenu);

    match load_main_menu_snapshot() {
        Some(snapshot) => render_snapshot(snapshot, false),
        None => render_snapshot_missing_state().into_any_element(),
    }
}

pub fn render_main_menu_compare_thumbnail(_stable_id: &str) -> AnyElement {
    match load_main_menu_snapshot() {
        Some(snapshot) => render_snapshot(snapshot, true),
        None => render_snapshot_missing_state().into_any_element(),
    }
}

fn render_snapshot(snapshot: MainMenuSnapshot, compact: bool) -> AnyElement {
    let image = snapshot.image.clone();
    let width = if compact {
        320.0
    } else {
        snapshot.width as f32
    };
    let height = if compact {
        212.0
    } else {
        snapshot.height as f32
    };
    let fit = if compact {
        ObjectFit::Contain
    } else {
        ObjectFit::Fill
    };

    div()
        .w_full()
        .h_full()
        .flex()
        .justify_center()
        .items_start()
        .overflow_hidden()
        .child(
            img(move |_window: &mut Window, _cx: &mut App| Some(Ok(image.clone())))
                .w(px(width))
                .h(px(height))
                .object_fit(fit),
        )
        .into_any_element()
}

fn render_snapshot_missing_state() -> Div {
    let snapshot_path = main_menu_snapshot_path();

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .justify_center()
        .items_center()
        .gap(px(8.))
        .text_center()
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Main menu snapshot missing"),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgba(0xFFFFFF99))
                .child(snapshot_path.display().to_string()),
        )
}
