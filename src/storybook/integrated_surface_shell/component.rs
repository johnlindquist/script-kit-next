use gpui::{AnyElement, IntoElement};

use super::types::{IntegratedOverlayPlacement, IntegratedSurfaceShellConfig};

/// A reusable scene host that renders a body surface, an optional footer,
/// and an optional overlay popup in a single storybook preview.
#[derive(IntoElement)]
pub struct IntegratedSurfaceShell {
    pub(crate) config: IntegratedSurfaceShellConfig,
    pub(crate) body: AnyElement,
    pub(crate) footer: Option<AnyElement>,
    pub(crate) overlay: Option<(IntegratedOverlayPlacement, AnyElement)>,
}

impl IntegratedSurfaceShell {
    pub fn new(config: IntegratedSurfaceShellConfig, body: impl IntoElement) -> Self {
        Self {
            config,
            body: body.into_any_element(),
            footer: None,
            overlay: None,
        }
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn overlay(
        mut self,
        placement: IntegratedOverlayPlacement,
        overlay: impl IntoElement,
    ) -> Self {
        self.overlay = Some((placement, overlay.into_any_element()));
        self
    }
}
