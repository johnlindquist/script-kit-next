use super::load_selected_story_variant;

/// Stable variant identity contract for adoptable storybook surfaces.
pub trait VariationId: Copy + Eq {
    fn as_str(self) -> &'static str;
    fn name(self) -> &'static str;
    fn description(self) -> &'static str;
    fn from_stable_id(value: &str) -> Option<Self>;
}

/// Shared adoption contract for a live-rendered surface backed by storybook variants.
pub trait AdoptableSurface {
    type Id: VariationId;
    type Spec: Copy;
    type Live: Copy;

    const STORY_ID: &'static str;
    const DEFAULT_ID: Self::Id;

    fn specs() -> &'static [Self::Spec];
    fn spec_id(spec: &Self::Spec) -> Self::Id;
    fn live_from_spec(spec: &Self::Spec) -> Self::Live;
}

/// Structured result of resolving a persisted selection for an adoptable surface.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceSelectionResolution {
    pub story_id: String,
    pub requested_variant_id: Option<String>,
    pub resolved_variant_id: String,
    pub fallback_used: bool,
}

/// Resolve a saved stable ID into the live typed representation for a surface.
pub fn resolve_surface_live<S>(selected: Option<&str>) -> (S::Live, SurfaceSelectionResolution)
where
    S: AdoptableSurface,
    S::Spec: 'static,
{
    let requested_variant_id = selected.map(str::to_owned);
    let resolved_id = selected
        .and_then(S::Id::from_stable_id)
        .unwrap_or(S::DEFAULT_ID);

    let spec = S::specs()
        .iter()
        .find(|spec| S::spec_id(spec) == resolved_id)
        .unwrap_or(&S::specs()[0]);
    let spec_id = S::spec_id(spec);

    (
        S::live_from_spec(spec),
        SurfaceSelectionResolution {
            story_id: S::STORY_ID.to_string(),
            requested_variant_id,
            resolved_variant_id: spec_id.as_str().to_string(),
            fallback_used: selected.is_some() && selected != Some(spec_id.as_str()),
        },
    )
}

/// Resolve the current on-disk storybook selection into a live typed surface value.
pub fn adopted_surface_live<S>() -> S::Live
where
    S: AdoptableSurface,
    S::Spec: 'static,
{
    let selected = load_selected_story_variant(S::STORY_ID);
    resolve_surface_live::<S>(selected.as_deref()).0
}
