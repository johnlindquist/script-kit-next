mod component;
mod render;
#[cfg(test)]
mod tests;
mod types;

pub use component::IntegratedSurfaceShell;
pub use types::{
    IntegratedOverlayAnchor, IntegratedOverlayPlacement, IntegratedOverlayState,
    IntegratedSurfaceShellConfig,
};
