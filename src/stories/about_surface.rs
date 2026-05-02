use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use gpui::*;

use crate::{
    about::{render::AboutSurfaceActions, AboutState},
    storybook::{Story, StoryCatalogRole, StorySurface, StoryVariant},
    updates::UpdateState,
};

pub struct AboutSurfaceStory;

impl Story for AboutSurfaceStory {
    fn id(&self) -> &'static str {
        "about_surface/default"
    }

    fn name(&self) -> &'static str {
        "About Script Kit"
    }

    fn category(&self) -> &'static str {
        "Launcher Surfaces"
    }

    fn catalog_role(&self) -> StoryCatalogRole {
        StoryCatalogRole::CanonicalState
    }

    fn surface(&self) -> StorySurface {
        StorySurface::AboutSurface
    }

    fn render(&self) -> AnyElement {
        self.render_variant(&StoryVariant::default_named("idle", "Idle"))
    }

    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        let stable_id = variant.stable_id();
        let update_state = match stable_id.as_str() {
            "checking" => UpdateState::Checking,
            "up-to-date" => UpdateState::UpToDate,
            "available" => UpdateState::Available {
                version: "9.9.9".to_string(),
                url: "https://github.com/johnlindquist/script-kit-next/releases/latest".to_string(),
            },
            "error" => UpdateState::Error("network unavailable".to_string()),
            _ => UpdateState::Idle,
        };
        let state = AboutState {
            acks_open: stable_id == "acknowledgements-open",
        };
        crate::about::render::render_about_surface_preview(
            &state,
            Arc::new(RwLock::new(update_state)),
            noop_actions(),
        )
    }

    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant::default_named("idle", "Idle"),
            StoryVariant::default_named("checking", "Checking"),
            StoryVariant::default_named("up-to-date", "Up To Date"),
            StoryVariant::default_named("available", "Available"),
            StoryVariant::default_named("error", "Error"),
            StoryVariant::default_named("acknowledgements-open", "Acknowledgements Open"),
        ]
    }
}

fn noop_actions() -> AboutSurfaceActions {
    let click = Rc::new(|_: &ClickEvent, _: &mut Window, _: &mut App| {});
    let key = Rc::new(|_: &KeyDownEvent, _: &mut Window, _: &mut App| {});
    AboutSurfaceActions {
        dismiss: click.clone(),
        open_github: click.clone(),
        open_discord: click.clone(),
        follow_x: click.clone(),
        check_updates: click.clone(),
        open_release: click.clone(),
        toggle_acknowledgements: click,
        key_down: key,
    }
}
