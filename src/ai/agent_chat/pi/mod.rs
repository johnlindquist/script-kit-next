//! Pi-backed Agent Chat launch contracts.

pub mod binary;
mod events;
pub mod launch_spec;
mod protocol;
mod runtime;

pub(crate) use protocol::PiRpcLaunchSpec;
pub(crate) use runtime::PiRpcRuntime;

#[cfg(test)]
pub(crate) use protocol::{
    build_abort_command, build_get_available_models_command, build_prompt_command,
    build_set_model_command, parse_rpc_line, PiRpcModelSelection, PiRpcPromptPayload,
};
