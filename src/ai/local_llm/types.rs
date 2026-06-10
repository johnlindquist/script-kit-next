//! Shared types for the on-device (GGUF / llama.cpp) ghost-text engine.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// What the local model should complete. Each surface builds its prompt at
/// its own layer; the runtime actor only resolves the final prompt string.
#[derive(Clone, Debug)]
pub(crate) enum GhostPromptSpec {
    /// Launcher input continuation: prompt composed by
    /// `crate::scripts::search::ghost::build_local_ghost_prompt` from the
    /// caret prefix plus the cwd-derived context digest.
    Launcher {
        partial_query: String,
        context: crate::scripts::search::ghost::GhostContext,
    },
    /// Notes editor continuation: a prebuilt prompt from
    /// `crate::notes::ghost_llm::build_notes_ghost_prompt` (note excerpt +
    /// brain recall block + current line prefix).
    NotesContinuation { prompt: String },
}

/// A single ghost-completion request handed to the local runtime actor.
#[derive(Clone, Debug)]
pub(crate) struct LocalGhostRequest {
    /// The surface-specific prompt to continue.
    pub prompt: GhostPromptSpec,
    /// Cooperative cancel flag. Checked before load, after debounce, and on
    /// every decode step so a new keystroke aborts in-flight generation.
    pub cancel: Arc<AtomicBool>,
}

/// The raw (unsanitized) model output plus the identity of the model that
/// produced it. The caller runs `sanitize_llm_completion_suffix` over the raw
/// text before it ever reaches the UI.
#[derive(Clone, Debug)]
pub(crate) struct LocalGhostResponse {
    pub model_id: String,
    pub raw_completion: String,
}

/// A GGUF model resolved on disk, ready to load.
#[derive(Clone, Debug)]
pub(crate) struct ResolvedLocalModel {
    pub path: PathBuf,
    /// Stable cache identity: filename + length + mtime + sampling fingerprint.
    pub model_id: String,
    pub display_name: String,
}

/// Sampling + context parameters for ghost generation. Mirrors Cotabby's
/// defaults (batch 512, topK 40, topP 0.95) tuned for a one-line, sub-second
/// autocomplete (low temperature, tiny token budget). `ctx_tokens` is sized
/// for the largest ghost prompt: the notes continuation prompt, which can
/// carry a brain recall block (up to ~4000 chars) plus a note excerpt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GhostSamplingParams {
    pub max_prediction_tokens: usize,
    pub temperature: f32,
    pub top_k: i32,
    pub top_p: f32,
    pub min_p: f32,
    pub repeat_penalty: f32,
    pub ctx_tokens: u32,
    pub batch_size: u32,
    pub gpu_layers: u32,
}

impl Default for GhostSamplingParams {
    fn default() -> Self {
        Self {
            max_prediction_tokens: 12,
            temperature: 0.15,
            top_k: 40,
            top_p: 0.95,
            min_p: 0.05,
            repeat_penalty: 1.08,
            ctx_tokens: 4096,
            batch_size: 512,
            // llama-cpp-2 uses a u32 gpu-layer count; "all practical layers".
            gpu_layers: 99,
        }
    }
}

impl GhostSamplingParams {
    /// Compact fingerprint folded into the model id so a sampling change
    /// invalidates the ghost LLM cache.
    pub fn fingerprint(&self) -> String {
        format!(
            "tok{}:t{:.2}:k{}:p{:.2}",
            self.max_prediction_tokens, self.temperature, self.top_k, self.top_p
        )
    }
}
