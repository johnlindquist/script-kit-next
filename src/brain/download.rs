//! First-run download of the brain's embedding model.
//!
//! Semantic search should be zero-setup: when the brain has content but no
//! embedding model, the indexer asks this module to fetch embeddinggemma
//! (~318 MiB, one file, resumable) into `~/.scriptkit/models/brain/`.
//! Lexical search keeps working throughout; the moment the file lands, the
//! next index cycle lights up vectors.
//!
//! Politeness rules:
//! - never more than one attempt per 24h (marker in `brain_meta`)
//! - resumable via a `.partial` file + HTTP Range (same pattern as the
//!   Parakeet dictation model in `dictation::download`)
//! - opt out by creating `~/.scriptkit/models/brain/.no-download`.

use super::store;
use anyhow::{Context as _, Result};
use std::io::{Read as _, Write as _};
use std::path::PathBuf;

const MODEL_URL: &str = "https://huggingface.co/ggml-org/embeddinggemma-300M-GGUF/resolve/main/embeddinggemma-300M-Q8_0.gguf";
const MODEL_FILE: &str = "embeddinggemma-300M-Q8_0.gguf";
const MODEL_SIZE: u64 = 333_590_944;
const ATTEMPT_MARKER: &str = "embed_model_download_attempt";
const ATTEMPT_COOLDOWN_SECS: i64 = 24 * 60 * 60;

fn brain_models_dir() -> Option<PathBuf> {
    Some(
        dirs::home_dir()?
            .join(".scriptkit")
            .join("models")
            .join("brain"),
    )
}

/// Ensure the embedding model exists, downloading it when appropriate.
/// Returns `true` when a model is present after the call.
pub fn ensure_embed_model(have_docs: bool) -> bool {
    if super::embedder::resolve_embed_model().is_some() {
        return true;
    }
    // Only spend bandwidth once the brain actually has something to embed.
    if !have_docs {
        return false;
    }
    let Some(dir) = brain_models_dir() else {
        return false;
    };
    if dir.join(".no-download").exists() {
        return false;
    }
    let now = chrono::Utc::now().timestamp();
    if let Ok(Some(last)) = store::meta_get(ATTEMPT_MARKER) {
        if let Ok(last) = last.parse::<i64>() {
            if now - last < ATTEMPT_COOLDOWN_SECS {
                return false;
            }
        }
    }
    let _ = store::meta_set(ATTEMPT_MARKER, &now.to_string());
    match download_model(&dir) {
        Ok(()) => {
            tracing::info!(
                target: "script_kit::brain",
                "embedding model downloaded; semantic search enabled"
            );
            true
        }
        Err(error) => {
            tracing::warn!(
                target: "script_kit::brain",
                error = %error,
                "embedding model download failed; brain stays lexical-only (retry in 24h)"
            );
            false
        }
    }
}

fn download_model(dir: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dir).context("create brain models dir")?;
    let final_path = dir.join(MODEL_FILE);
    let partial_path = dir.join(format!("{MODEL_FILE}.partial"));

    let resume_from = std::fs::metadata(&partial_path)
        .map(|meta| meta.len())
        .unwrap_or(0);
    if resume_from < MODEL_SIZE {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .new_agent();
        let mut request = agent.get(MODEL_URL);
        if resume_from > 0 {
            request = request.header("Range", &format!("bytes={resume_from}-"));
        }
        let response = request.call().context("embed model HTTP request")?;
        let status = response.status().as_u16();
        let actual_resume = match status {
            206 => resume_from,
            200 => 0,
            other => anyhow::bail!("unexpected HTTP status {other} downloading embed model"),
        };
        let mut file = if actual_resume > 0 {
            std::fs::OpenOptions::new()
                .append(true)
                .open(&partial_path)
                .context("open partial embed model")?
        } else {
            std::fs::File::create(&partial_path).context("create partial embed model")?
        };
        let mut reader = response.into_body().into_reader();
        let mut buf = vec![0u8; 256 * 1024];
        loop {
            let read = reader.read(&mut buf).context("read embed model body")?;
            if read == 0 {
                break;
            }
            file.write_all(&buf[..read]).context("write embed model")?;
        }
        file.flush().context("flush embed model")?;
    }

    let downloaded = std::fs::metadata(&partial_path)
        .map(|meta| meta.len())
        .unwrap_or(0);
    if downloaded != MODEL_SIZE {
        anyhow::bail!("embed model incomplete: {downloaded} of {MODEL_SIZE} bytes");
    }
    std::fs::rename(&partial_path, &final_path).context("finalize embed model")?;
    Ok(())
}
