use super::load_selected_story_variant;

/// Evidence quality for a Storybook variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum StorybookRepresentation {
    LiveSurface,
    PresenterFixture,
    DesignExperiment,
}

impl StorybookRepresentation {
    pub fn prop_value(self) -> &'static str {
        match self {
            Self::LiveSurface => "liveSurface",
            Self::PresenterFixture => "presenterFixture",
            Self::DesignExperiment => "designExperiment",
        }
    }
}

/// Data source backing a Storybook variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum StorybookDataSource {
    ProductionState,
    DeterministicProductionFixture,
    MockDesignOnly,
}

impl StorybookDataSource {
    pub fn prop_value(self) -> &'static str {
        match self {
            Self::ProductionState => "productionState",
            Self::DeterministicProductionFixture => "deterministicProductionFixture",
            Self::MockDesignOnly => "mockDesignOnly",
        }
    }
}

/// Source of footer hints used by a Storybook variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum FooterHintSource {
    ActiveFooterState,
    MainWindowFooterConfig,
    MockDesignOnly,
    None,
}

impl FooterHintSource {
    pub fn prop_value(self) -> &'static str {
        match self {
            Self::ActiveFooterState => "activeFooter",
            Self::MainWindowFooterConfig => "mainWindowFooterConfig",
            Self::MockDesignOnly => "mockDesignOnly",
            Self::None => "none",
        }
    }
}

/// Machine-readable adoption contract for registered main-menu stories.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct MainMenuStoryContract {
    pub variation_id: &'static str,
    pub representation: StorybookRepresentation,
    pub data_source: StorybookDataSource,
    pub footer_hint_source: FooterHintSource,
    pub uses_central_theme_tokens: bool,
}

impl MainMenuStoryContract {
    pub fn assert_primary_catalog_safe(&self) {
        assert!(
            !matches!(
                self.representation,
                StorybookRepresentation::DesignExperiment
            ),
            "design experiment registered as primary main-menu story: {}",
            self.variation_id
        );
        assert!(
            !matches!(self.data_source, StorybookDataSource::MockDesignOnly),
            "mock data registered as primary main-menu story: {}",
            self.variation_id
        );
        assert!(
            !matches!(self.footer_hint_source, FooterHintSource::MockDesignOnly),
            "mock footer registered as primary main-menu story: {}",
            self.variation_id
        );
        assert!(
            self.uses_central_theme_tokens,
            "main-menu story must use central theme tokens: {}",
            self.variation_id
        );
    }
}

/// Footer snapshot exported by Storybook variants for parity audits.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct StorybookFooterSnapshot {
    pub owner: &'static str,
    pub source: FooterHintSource,
    pub buttons: Vec<&'static str>,
    pub disabled_reasons: Vec<&'static str>,
    pub dispatch_target: Option<&'static str>,
}

impl StorybookFooterSnapshot {
    pub fn assert_launcher_contract(&self) {
        assert!(
            self.buttons.len() <= 3,
            "launcher footer exceeded three-affordance budget: {:?}",
            self.buttons
        );
    }

    pub fn assert_acp_ready_contract(&self) {
        self.assert_launcher_contract();
        assert!(
            self.disabled_reasons.is_empty(),
            "ACP-ready footer must not carry disabled reasons"
        );
        assert_eq!(
            self.dispatch_target,
            Some("execute_script_by_path"),
            "ACP-ready footer must dispatch through execute_script_by_path"
        );
    }

    pub fn assert_acp_not_ready_contract(&self) {
        self.assert_launcher_contract();
        assert!(
            !self.buttons.iter().any(|button| *button == "Run"),
            "ACP-not-ready footer must hide Run until SCRIPT_READY validated=true"
        );
    }
}

/// Contract for compare panels that might otherwise imply production parity.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ComparePanelContract {
    pub left_id: &'static str,
    pub right_id: &'static str,
    pub left_data_source: StorybookDataSource,
    pub right_data_source: StorybookDataSource,
    pub registered_primary_catalog: bool,
}

impl ComparePanelContract {
    pub fn assert_not_false_production_comparison(&self) {
        if !self.registered_primary_catalog {
            return;
        }

        assert_ne!(
            self.left_data_source,
            StorybookDataSource::MockDesignOnly,
            "primary compare panel left side uses mock design data: {}",
            self.left_id
        );
        assert_ne!(
            self.right_data_source,
            StorybookDataSource::MockDesignOnly,
            "primary compare panel right side uses mock design data: {}",
            self.right_id
        );
    }
}

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
