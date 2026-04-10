//! Typed manifest for runtime-captured storybook fixtures.

/// Machine-readable manifest for a runtime-captured surface fixture.
///
/// Each fixture lives under `test-screenshots/storybook-fixtures/<surface>/<variant>.png`
/// with a companion `<variant>.json` manifest describing capture metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeFixtureManifest {
    pub schema_version: u8,
    pub surface: String,
    pub variant_id: String,
    pub image_path: String,
    pub width: u32,
    pub height: u32,
    pub automation_kind: String,
    pub semantic_surface: String,
}
