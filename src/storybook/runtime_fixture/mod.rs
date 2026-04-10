//! Shared runtime-fixture host for storybook surfaces.
//!
//! Surfaces that cannot be reconstructed inside the stateless `Story::render()`
//! contract use runtime-captured PNG fixtures instead. This module provides the
//! shared load/render/manifest infrastructure so each surface does not reinvent
//! its own snapshot pipeline.

mod types;

pub use types::RuntimeFixtureManifest;

use gpui::*;
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Base directory for all runtime fixture assets, relative to the crate root.
const FIXTURES_DIR: &str = "test-screenshots/storybook-fixtures";

/// Return the manifest JSON path for a given surface + variant.
pub fn runtime_fixture_manifest_path(surface: &str, variant_id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES_DIR)
        .join(surface)
        .join(format!("{variant_id}.json"))
}

/// Return the image path for a given surface + variant.
pub fn runtime_fixture_image_path(surface: &str, variant_id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES_DIR)
        .join(surface)
        .join(format!("{variant_id}.png"))
}

/// Cheaply probe whether the fixture image and manifest exist for a surface + variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeFixturePresence {
    pub image_present: bool,
    pub manifest_present: bool,
}

pub fn describe_runtime_fixture(surface: &str, variant_id: &str) -> RuntimeFixturePresence {
    RuntimeFixturePresence {
        image_present: runtime_fixture_image_path(surface, variant_id).is_file(),
        manifest_present: runtime_fixture_manifest_path(surface, variant_id).is_file(),
    }
}

/// Load and parse a fixture manifest from disk.
pub fn load_runtime_fixture_manifest(
    surface: &str,
    variant_id: &str,
) -> anyhow::Result<RuntimeFixtureManifest> {
    let path = runtime_fixture_manifest_path(surface, variant_id);
    let bytes = std::fs::read(&path).map_err(|e| {
        anyhow::anyhow!("Failed to read fixture manifest at {}: {e}", path.display())
    })?;
    let manifest: RuntimeFixtureManifest = serde_json::from_slice(&bytes).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse fixture manifest at {}: {e}",
            path.display()
        )
    })?;
    Ok(manifest)
}

/// Render a runtime fixture as a GPUI element suitable for storybook preview.
///
/// If `compact` is true, the image is rendered at a reduced thumbnail size
/// appropriate for compare-mode grids. Otherwise it is rendered at full size.
pub fn render_runtime_fixture(
    surface: &'static str,
    variant_id: &str,
    compact: bool,
) -> AnyElement {
    match load_fixture_image(surface, variant_id) {
        Some(snapshot) => {
            tracing::info!(
                event = "storybook_runtime_fixture_loaded",
                surface,
                variant_id,
                path = %snapshot.path.display(),
                width = snapshot.width,
                height = snapshot.height,
                "Loaded storybook runtime fixture"
            );
            render_fixture_snapshot(snapshot, compact)
        }
        None => {
            let image_path = runtime_fixture_image_path(surface, variant_id);
            let manifest_path = runtime_fixture_manifest_path(surface, variant_id);
            tracing::warn!(
                event = "storybook_runtime_fixture_missing",
                surface,
                variant_id,
                image_path = %image_path.display(),
                manifest_path = %manifest_path.display(),
                "Storybook runtime fixture missing"
            );
            render_fixture_missing_state(surface, variant_id).into_any_element()
        }
    }
}

struct FixtureSnapshot {
    image: Arc<RenderImage>,
    width: u32,
    height: u32,
    path: PathBuf,
}

fn load_fixture_image(surface: &str, variant_id: &str) -> Option<FixtureSnapshot> {
    let path = runtime_fixture_image_path(surface, variant_id);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(error) => {
            tracing::warn!(
                event = "storybook_runtime_fixture_image_missing",
                path = %path.display(),
                error = %error,
                "Runtime fixture image not found"
            );
            return None;
        }
    };

    let dimensions = match image::load_from_memory(&bytes) {
        Ok(img) => img.dimensions(),
        Err(error) => {
            tracing::warn!(
                event = "storybook_runtime_fixture_dimensions_failed",
                path = %path.display(),
                error = %error,
                "Failed to read fixture image dimensions"
            );
            return None;
        }
    };

    let render_image =
        match crate::list_item::decode_png_to_render_image_with_bgra_conversion(&bytes) {
            Ok(image) => image,
            Err(error) => {
                tracing::warn!(
                    event = "storybook_runtime_fixture_decode_failed",
                    path = %path.display(),
                    error = %error,
                    "Failed to decode fixture image"
                );
                return None;
            }
        };

    Some(FixtureSnapshot {
        image: render_image,
        width: dimensions.0,
        height: dimensions.1,
        path,
    })
}

fn render_fixture_snapshot(snapshot: FixtureSnapshot, compact: bool) -> AnyElement {
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

fn render_fixture_missing_state(surface: &str, variant_id: &str) -> Div {
    let image_path = runtime_fixture_image_path(surface, variant_id);

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
                .child(format!("Runtime fixture missing: {surface}/{variant_id}")),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgba(0xFFFFFF99))
                .child(image_path.display().to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgba(0xFFFFFF66))
                .child("Capture with agentic-testing to populate this fixture"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip_serde() {
        let manifest = RuntimeFixtureManifest {
            schema_version: 1,
            surface: "notes-window".to_string(),
            variant_id: "current".to_string(),
            image_path: "test-screenshots/storybook-fixtures/notes-window/current.png".to_string(),
            width: 640,
            height: 480,
            automation_kind: "notes".to_string(),
            semantic_surface: "notesWindow".to_string(),
        };

        let json = serde_json::to_string_pretty(&manifest).expect("serialize");
        let parsed: RuntimeFixtureManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(manifest, parsed);
        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.surface, "notes-window");
    }

    #[test]
    fn manifest_path_includes_surface_and_variant() {
        let path = runtime_fixture_manifest_path("notes-window", "current");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("storybook-fixtures"));
        assert!(path_str.contains("notes-window"));
        assert!(path_str.ends_with("current.json"));
    }

    #[test]
    fn load_missing_manifest_returns_error() {
        let result = load_runtime_fixture_manifest("nonexistent-surface", "nonexistent-variant");
        assert!(result.is_err());
    }

    #[test]
    fn describe_missing_fixture_reports_both_absent() {
        let presence = describe_runtime_fixture("nonexistent-surface", "nonexistent-variant");
        assert!(!presence.image_present);
        assert!(!presence.manifest_present);
    }

    #[test]
    fn describe_fixture_serializes_camel_case() {
        let presence = RuntimeFixturePresence {
            image_present: true,
            manifest_present: false,
        };
        let json = serde_json::to_string(&presence).expect("serialize");
        assert!(json.contains("\"imagePresent\""));
        assert!(json.contains("\"manifestPresent\""));
    }
}
