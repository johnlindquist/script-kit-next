use anyhow::{Context as _, Result};
use llama_cpp_2::context::params::{LlamaContextParams, LlamaPoolingType};
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

const GHOST_SAMPLER_SEED: u32 = 0x5C71_7ABB;
const RAW_OUTPUT_BYTE_CAP: usize = 160;
const EMBED_CTX_TOKENS: u32 = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

/// A GGUF embedding model (e.g. embeddinggemma) loaded for mean-pooled,
/// L2-normalized sentence embeddings. Kept separate from [`LoadedLocalLlm`]
/// so the brain's embedder and the ghost-text generator can coexist without
/// evicting each other.
pub(crate) struct LoadedEmbedder {
    model_id: String,
    backend: &'static LlamaBackend,
    model: LlamaModel,
}

impl LoadedEmbedder {
    pub(crate) fn load(model_path: &Path, model_id: &str, gpu_layers: u32) -> Result<Self> {
        let backend = backend()?;
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(gpu_layers)
            .with_use_mmap(true);
        let model = LlamaModel::load_from_file(backend, model_path, &model_params)
            .with_context(|| format!("load embedding gguf {}", model_path.display()))?;
        Ok(Self {
            model_id: model_id.to_string(),
            backend,
            model,
        })
    }

    pub(crate) fn model_id(&self) -> &str {
        &self.model_id
    }

    pub(crate) fn embed_texts(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(EMBED_CTX_TOKENS))
            .with_n_batch(EMBED_CTX_TOKENS)
            .with_n_ubatch(EMBED_CTX_TOKENS)
            .with_embeddings(true)
            .with_pooling_type(LlamaPoolingType::Mean);
        let mut ctx: LlamaContext<'_> = self
            .model
            .new_context(self.backend, ctx_params)
            .context("create embedding context")?;
        let max_tokens = (EMBED_CTX_TOKENS as usize).saturating_sub(8).max(1);
        let mut out = Vec::with_capacity(texts.len());
        for text in texts {
            let mut tokens = self
                .model
                .str_to_token(text, AddBos::Always)
                .context("tokenize embedding input")?;
            if tokens.is_empty() {
                out.push(Vec::new());
                continue;
            }
            if tokens.len() > max_tokens {
                tokens.truncate(max_tokens);
            }
            let mut batch = LlamaBatch::new(tokens.len(), 1);
            batch
                .add_sequence(&tokens, 0, false)
                .context("add embedding sequence")?;
            ctx.clear_kv_cache();
            ctx.decode(&mut batch).context("decode embedding batch")?;
            let embedding = ctx
                .embeddings_seq_ith(0)
                .context("read pooled embedding")?
                .to_vec();
            out.push(l2_normalize(embedding));
        }
        Ok(out)
    }
}

fn l2_normalize(mut v: Vec<f32>) -> Vec<f32> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > f32::EPSILON {
        for x in &mut v {
            *x /= norm;
        }
    }
    v
}

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
    pub(crate) fn load(
        model_path: &Path,
        model_id: &str,
        sampling: GhostSamplingParams,
    ) -> Result<Self> {
        let backend = backend()?;
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(sampling.gpu_layers)
            .with_use_mmap(true);
        let loaded_model = LlamaModel::load_from_file(backend, model_path, &model_params)
            .with_context(|| format!("load gguf {}", model_path.display()))?;
        Ok(Self {
            model_id: model_id.to_string(),
            sampling,
            backend,
            model: loaded_model,
        })
    }

    pub(crate) fn model_id(&self) -> &str {
        &self.model_id
    }

    #[allow(clippy::manual_unwrap_or_default, clippy::needless_range_loop)]
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
        let max_prompt = (self.sampling.ctx_tokens as usize)
            .saturating_sub(self.sampling.max_prediction_tokens + 4)
            .max(1);
        if tokens.len() > max_prompt {
            tokens = tokens.split_off(tokens.len() - max_prompt);
        }
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(self.sampling.ctx_tokens))
            .with_n_batch(self.sampling.batch_size)
            .with_n_ubatch(self.sampling.batch_size);
        let mut ctx: LlamaContext<'_> = self
            .model
            .new_context(self.backend, ctx_params)
            .context("create llama context")?;
        let n_batch = (self.sampling.batch_size as usize).max(1);
        let mut batch = LlamaBatch::new(n_batch, 1);
        let last_index = tokens.len() - 1;
        let mut chunk_start = 0usize;
        while chunk_start < tokens.len() {
            let chunk_end = (chunk_start + n_batch).min(tokens.len());
            batch.clear();
            for index in chunk_start..chunk_end {
                batch
                    .add(tokens[index], index as i32, &[0], index == last_index)
                    .context("add prompt token")?;
            }
            ctx.decode(&mut batch).context("decode prompt")?;
            chunk_start = chunk_end;
        }
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(64, self.sampling.repeat_penalty, 0.0, 0.0),
            LlamaSampler::top_k(self.sampling.top_k),
            LlamaSampler::top_p(self.sampling.top_p, 1),
            LlamaSampler::min_p(self.sampling.min_p, 1),
            LlamaSampler::temp(self.sampling.temperature),
            LlamaSampler::dist(GHOST_SAMPLER_SEED),
        ]);
        let mut out_bytes: Vec<u8> = Vec::new();
        let mut n_cur = tokens.len() as i32;
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
            let piece = match self.model.token_to_piece_bytes(token, 32, false, None) {
                Ok(piece) => piece,
                Err(_) => Vec::new(),
            };
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
