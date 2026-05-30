//! Fallback backend used when the `local-llm` feature is off or the platform
//! is not macOS. Loading always fails, so the ghost pipeline silently keeps the
//! deterministic starter (no network, no error surfaced to the user).

use super::types::ResolvedLocalModel;
use anyhow::Result;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub(crate) struct LoadedLocalLlm {
    model_id: String,
}

impl LoadedLocalLlm {
    pub(crate) fn load(_model: &ResolvedLocalModel) -> Result<Self> {
        anyhow::bail!(
            "ghost_local_llm_backend_unavailable: build with --features local-llm on macOS"
        )
    }

    pub(crate) fn model_id(&self) -> &str {
        &self.model_id
    }

    pub(crate) fn generate_one_line(
        &mut self,
        _prompt: &str,
        _cancel: &Arc<AtomicBool>,
    ) -> Result<String> {
        anyhow::bail!("ghost_local_llm_backend_unavailable")
    }
}
