//! Fixture data and controls for the actual secondary production factories.
use super::fixture_ids::SECONDARY_FIXTURE_IDS;
use crate::{
    runtime_policy::WindowHostPolicy,
    window_control::{
        snap_overlay::open_snap_overlay_window,
        snap_session::{SnapOverlayModel, SnapOverlayTarget},
    },
};
use anyhow::{ensure, Result};
use gpui::{point, px, size, AnyView, AnyWindowHandle, App, Entity};
use std::sync::{Arc, Mutex};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub(crate) struct SecondaryFixtureParent {
    pub id: String,
    pub generation: u64,
    pub handle: AnyWindowHandle,
}

pub(crate) enum SecondaryFixtureControls {
    Snap {
        view: Entity<crate::window_control::SnapOverlayView>,
    },
}

pub(crate) struct MountedSecondaryFixture {
    pub handle: AnyWindowHandle,
    pub root: AnyView,
    pub automation_id: String,
    pub generation: u64,
    pub controls: SecondaryFixtureControls,
}

impl SecondaryFixtureControls {
    pub(crate) fn observation(&self, cx: &App) -> Result<serde_json::Value> {
        Ok(match self {
            Self::Snap { view } => {
                let view = view.read(cx);
                match view.model() {
                    Some(model) => {
                        serde_json::json!({"mode": format!("{:?}", model.mode), "targets": model.targets.iter().map(|target| serde_json::json!({"tile": format!("{:?}", target.tile), "active": target.active, "bounds": {"x": target.bounds.x, "y": target.bounds.y, "width": target.bounds.width, "height": target.bounds.height}})).collect::<Vec<_>>()})
                    }
                    None => serde_json::json!({"mode": null, "targets": []}),
                }
            }
        })
    }
}

impl MountedSecondaryFixture {
    /// Invoke the real owner teardown, with an exact generation check before singleton access.
    pub(crate) fn close(&self, cx: &mut App) -> Result<()> {
        ensure!(
            crate::windows::get_runtime_window_handle_for_generation(
                &self.automation_id,
                self.generation
            ) == Some(self.handle),
            "secondary_fixture_stale"
        );
        let info = crate::windows::automation_window_by_id(&self.automation_id)
            .ok_or_else(|| anyhow::anyhow!("secondary_fixture_stale"))?;
        crate::windows::automation_surface_collector::close_owned_registered_surface(&info, cx)
    }
}

fn parent_geometry(
    parent: &SecondaryFixtureParent,
    cx: &mut App,
) -> Result<(gpui::Bounds<gpui::Pixels>, Option<gpui::DisplayId>)> {
    ensure!(
        crate::windows::get_runtime_window_handle_for_generation(&parent.id, parent.generation)
            == Some(parent.handle),
        "secondary_parent_stale"
    );
    ensure!(
        crate::windows::runtime_window_host_policy(&parent.id, parent.generation)?
            == WindowHostPolicy::OwnedHidden,
        "secondary_parent_not_owned"
    );
    parent.handle.update(cx, |_, window, cx| {
        (
            window.bounds(),
            window.display(cx).map(|display| display.id()),
        )
    })
}

pub(crate) fn mount_secondary_fixture(
    fixture_id: &str,
    parent: Option<SecondaryFixtureParent>,
    cx: &mut App,
) -> Result<MountedSecondaryFixture> {
    WindowHostPolicy::OwnedHidden.validate()?;
    ensure!(
        SECONDARY_FIXTURE_IDS.contains(&fixture_id),
        "unknown_secondary_fixture"
    );
    let (handle, root, controls) = match fixture_id {
        "secondary.snap" => {
            let bounds = gpui::Bounds::new(point(px(0.0), px(0.0)), size(px(800.0), px(500.0)));
            let window = open_snap_overlay_window(bounds, None, WindowHostPolicy::OwnedHidden, cx)?;
            let display = crate::window_control::Bounds {
                x: 0,
                y: 0,
                width: 800,
                height: 500,
            };
            window.update(cx, |view, _, cx| {
                view.set_model(
                    Some(SnapOverlayModel {
                        display_bounds: display,
                        mode: crate::window_control::SnapMode::Simple,
                        is_dominant: true,
                        targets: vec![SnapOverlayTarget {
                            tile: crate::window_control::TilePosition::LeftHalf,
                            bounds: crate::window_control::Bounds {
                                width: 400,
                                ..display
                            },
                            active: true,
                        }],
                    }),
                    cx,
                )
            })?;
            let view = window.entity(cx)?;
            (
                window.into(),
                view.clone().into(),
                SecondaryFixtureControls::Snap { view },
            )
        }
        _ => unreachable!("closed fixture catalogue validated above"),
    };
    let info = crate::windows::list_automation_windows()
        .into_iter()
        .find(|info| {
            info.generation.is_some_and(|generation| {
                crate::windows::get_runtime_window_handle_for_generation(&info.id, generation)
                    == Some(handle)
            })
        })
        .ok_or_else(|| anyhow::anyhow!("secondary_factory_did_not_register"))?;
    let generation = info
        .generation
        .ok_or_else(|| anyhow::anyhow!("secondary_generation_missing"))?;
    Ok(MountedSecondaryFixture {
        handle,
        root,
        automation_id: info.id,
        generation,
        controls,
    })
}
