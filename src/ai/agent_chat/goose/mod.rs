//! goose engine: Script Kit's AI sessions run in a pinned goose sidecar
//! (https://github.com/aaif-goose/goose) spoken to over the Agent Client
//! Protocol on stdio.

pub(crate) mod auth_recovery;
pub mod binary;
pub(crate) mod events;
pub(crate) mod launch_spec;
pub(crate) mod oneshot;
pub(crate) mod protocol;
pub(crate) mod recipe;
pub(crate) mod runtime;
pub(crate) mod sign_in;

pub(crate) use launch_spec::GooseProfileLaunch;
pub(crate) use protocol::GooseLaunchSpec;
pub(crate) use runtime::GooseRuntime;
