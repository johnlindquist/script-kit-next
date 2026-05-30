//! In-process GGUF inference backend via `llama-cpp-2` (llama.cpp + Metal).
//!
//! Compiled only for `--features local-llm` on macOS. Owns a leaked `LlamaModel`
//! (process lifetime) and a `LlamaContext<'static>` so the !Send/!Sync context
//! can live on the runtime actor thread. Generation is token-by-token with a
//! cooperative cancel flag so a new keystroke aborts mid-decode.

use super::types::{GhostSamplingParams, ResolvedLocalModel};
use anyhow::{Context as _, Result};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

/// Fixed sampler seed: ghost text should be stable for a given prefix, and we
/// avoid pulling an RNG dependency into the hot path.
const GHOST_SAMPLER_SEED: u32 = 0x5C71_7ABB;
/// Hard byte cap on raw output before the sanitizer runs (defense in depth).
const RAW_OUTPUT_BYTE_CAP: usize = 160;

/// Process-wide llama backend (init once; freed at exit).
fn backend() -> Result<&'static LlamaBackend> {
    static BACKEND: OnceLock<LlamaBackend> = OnceLock::new();
    if BACKEND.get().is_none() {
        let initialized = LlamaBackend::init().context("llama backend init failed")?;
        let _ = BACKEND.set(initialized);
    }
    BACKEND.get().context("llama backend not initialized")
}

pub(crate) struct LoadedLocalLlm {
    model_id: String,
    sampling: GhostSamplingParams,
    backend: &'static LlamaBackend,
    model: LlamaModel,
}

impl LoadedLocalLlm {
    pub(crate) fn load(model: &ResolvedLocalModel) -> Result<Self> {
        let backend = backend()?;
        let sampling = GhostSamplingParams::default();

        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(sampling.gpu_layers)
            .with_use_mmap(true);
        let loaded_model = LlamaModel::load_from_file(backend, &model.path, &model_params)
            .with_context(|| format!("load gguf {}", model.path.display()))?;

        Ok(Self {
            model_id: model.model_id.clone(),
            sampling,
            backend,
            model: loaded_model,
        })
    }

    pub(crate) fn model_id(&self) -> &str {
        &self.model_id
    }

    pub(crate) fn generate_one_line(
        &mut self,
        prompt: &str,
        cancel: &Arc<AtomicBool>,
    ) -> Result<String> {
        if cancel.load(Ordering::Relaxed) {
            anyhow::bail!("ghost_local_llm_cancelled");
        }

        let mut tokens = self
            .model
            .str_to_token(prompt, AddBos::Always)
            .context("tokenize prompt")?;
        if tokens.is_empty() {
            anyhow::bail!("ghost_local_llm_empty_prompt");
        }

        // Keep the most recent tokens within the context budget.
        let max_prompt = (self.sampling.ctx_tokens as usize)
            .saturating_sub(self.sampling.max_prediction_tokens + 4)
            .max(1);
        if tokens.len() > max_prompt {
            tokens = tokens.split_off(tokens.len() - max_prompt);
        }

        // A fresh context per request (no KV reuse yet — later polish). Creating
        // and dropping the context each call also frees its Metal residency sets
        // promptly, which avoids the ggml metal-device teardown assert that fires
        // when a context outlives the backend at process exit.
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(self.sampling.ctx_tokens))
            .with_n_batch(self.sampling.batch_size)
            .with_n_ubatch(self.sampling.batch_size);
        let mut ctx: LlamaContext<'_> = self
            .model
            .new_context(self.backend, ctx_params)
            .context("create llama context")?;

        let batch_cap = tokens.len().max(self.sampling.batch_size as usize);
        let mut batch = LlamaBatch::new(batch_cap, 1);
        let last_index = tokens.len() - 1;
        for (i, token) in tokens.iter().enumerate() {
            batch
                .add(*token, i as i32, &[0], i == last_index)
                .context("add prompt token")?;
        }
        ctx.decode(&mut batch).context("decode prompt")?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(64, self.sampling.repeat_penalty, 0.0, 0.0),
            LlamaSampler::top_k(self.sampling.top_k),
            LlamaSampler::top_p(self.sampling.top_p, 1),
            LlamaSampler::min_p(self.sampling.min_p, 1),
            LlamaSampler::temp(self.sampling.temperature),
            LlamaSampler::dist(GHOST_SAMPLER_SEED),
        ]);

        // Accumulate raw bytes across tokens (a single token may be a UTF-8
        // fragment) and decode once at the end to avoid mid-codepoint splits.
        let mut out_bytes: Vec<u8> = Vec::new();
        let mut n_cur = batch.n_tokens();
        let mut stop = false;

        for _ in 0..self.sampling.max_prediction_tokens {
            if cancel.load(Ordering::Relaxed) {
                anyhow::bail!("ghost_local_llm_cancelled");
            }

            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if self.model.is_eog_token(token) {
                break;
            }

            let piece = self
                .model
                .token_to_piece_bytes(token, 32, false, None)
                .unwrap_or_default();

            // One line only: stop at the first newline byte.
            if let Some(newline) = piece.iter().position(|b| *b == b'\n') {
                out_bytes.extend_from_slice(&piece[..newline]);
                break;
            }
            out_bytes.extend_from_slice(&piece);
            if out_bytes.len() > RAW_OUTPUT_BYTE_CAP {
                stop = true;
            }
            if stop {
                break;
            }

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .context("add sampled token")?;
            n_cur += 1;
            ctx.decode(&mut batch).context("decode token")?;
        }

        Ok(String::from_utf8_lossy(&out_bytes).to_string())
    }
}
