//! Brain embedder: a thin JSON-lines client for the `script-kit-ghost-llm-helper`
//! subprocess's `embed` request.
//!
//! Deliberately independent from `ai::local_llm::subprocess_backend`:
//! - it is NOT gated behind the `local-llm` feature (the helper is a separate
//!   process, so the app never links ggml for this path);
//! - it speaks only `embed`/`shutdown`, with blocking request semantics and
//!   no cancellation (embedding batches are short).
//!
//! Model resolution: `SCRIPT_KIT_BRAIN_EMBED_MODEL_PATH` env override, then
//! any `*.gguf` under `~/.scriptkit/models/brain/`. When no model is present
//! the brain degrades gracefully to FTS-only search.

use anyhow::{anyhow, Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

const HELPER_ENV: &str = "SCRIPT_KIT_GHOST_LLM_HELPER";
const HELPER_NAME: &str = "script-kit-ghost-llm-helper";
const MODEL_ENV: &str = "SCRIPT_KIT_BRAIN_EMBED_MODEL_PATH";

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EmbedRequest {
    Embed {
        id: u64,
        model_path: String,
        model_id: String,
        texts: Vec<String>,
        gpu_layers: u32,
    },
    Shutdown {
        id: u64,
    },
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    id: u64,
    ok: bool,
    #[serde(default)]
    embeddings: Option<Vec<Vec<f32>>>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedEmbedModel {
    pub path: PathBuf,
    pub model_id: String,
}

/// Locate the brain embedding model on disk. `None` => FTS-only mode.
pub fn resolve_embed_model() -> Option<ResolvedEmbedModel> {
    let candidate = if let Some(path) = std::env::var_os(MODEL_ENV) {
        let path = PathBuf::from(path);
        path.is_file().then_some(path)
    } else {
        let dir = dirs::home_dir()?
            .join(".scriptkit")
            .join("models")
            .join("brain");
        std::fs::read_dir(&dir).ok().and_then(|entries| {
            let mut ggufs: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "gguf"))
                .collect();
            ggufs.sort();
            ggufs.into_iter().next()
        })
    }?;
    let meta = std::fs::metadata(&candidate).ok()?;
    let name = candidate.file_name()?.to_string_lossy().into_owned();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some(ResolvedEmbedModel {
        model_id: format!("brain-embed:{name}:{}:{mtime}", meta.len()),
        path: candidate,
    })
}

/// A live helper subprocess dedicated to embeddings.
pub struct BrainEmbedder {
    model: ResolvedEmbedModel,
    child: Mutex<Child>,
    stdin: Mutex<Option<ChildStdin>>,
    pending: Arc<Mutex<HashMap<u64, mpsc::Sender<EmbedResponse>>>>,
    next_id: AtomicU64,
}

impl BrainEmbedder {
    pub fn spawn(model: ResolvedEmbedModel) -> Result<Self> {
        let helper_path = resolve_helper_path()?;
        let mut child = Command::new(&helper_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("spawn brain embed helper {}", helper_path.display()))?;
        let stdin = child.stdin.take().context("brain embed helper stdin")?;
        let stdout = child.stdout.take().context("brain embed helper stdout")?;
        let pending: Arc<Mutex<HashMap<u64, mpsc::Sender<EmbedResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let reader_pending = Arc::clone(&pending);
        std::thread::Builder::new()
            .name("script-kit-brain-embed-reader".to_string())
            .spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    let Ok(line) = line else { break };
                    let Ok(response) = serde_json::from_str::<EmbedResponse>(&line) else {
                        continue;
                    };
                    let sender = reader_pending
                        .lock()
                        .ok()
                        .and_then(|mut pending| pending.remove(&response.id));
                    if let Some(sender) = sender {
                        let _ = sender.send(response);
                    }
                }
            })
            .context("spawn brain embed reader thread")?;
        Ok(Self {
            model,
            child: Mutex::new(child),
            stdin: Mutex::new(Some(stdin)),
            pending,
            next_id: AtomicU64::new(1),
        })
    }

    pub fn model_id(&self) -> &str {
        &self.model.model_id
    }

    /// Embed a batch of texts. Blocking; returns unit-normalized vectors in
    /// input order (empty vec for empty inputs).
    pub fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel();
        self.pending
            .lock()
            .map_err(|_| anyhow!("brain embed pending lock poisoned"))?
            .insert(id, tx);
        let request = EmbedRequest::Embed {
            id,
            model_path: self.model.path.display().to_string(),
            model_id: self.model.model_id.clone(),
            texts,
            gpu_layers: 99,
        };
        if let Err(err) = self.write_request(&request) {
            let _ = self.pending.lock().map(|mut pending| pending.remove(&id));
            return Err(err);
        }
        let response = rx
            .recv_timeout(std::time::Duration::from_secs(300))
            .context("brain embed helper timed out")?;
        if !response.ok {
            return Err(anyhow!(response
                .error
                .unwrap_or_else(|| "brain embed failed".to_string())));
        }
        response.embeddings.context("missing brain embeddings")
    }

    fn write_request(&self, request: &EmbedRequest) -> Result<()> {
        let mut guard = self
            .stdin
            .lock()
            .map_err(|_| anyhow!("brain embed stdin lock poisoned"))?;
        let stdin = guard.as_mut().context("brain embed stdin closed")?;
        serde_json::to_writer(&mut *stdin, request).context("write brain embed request")?;
        stdin
            .write_all(b"\n")
            .context("write brain embed newline")?;
        stdin.flush().context("flush brain embed request")
    }
}

impl Drop for BrainEmbedder {
    fn drop(&mut self) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let _ = self.write_request(&EmbedRequest::Shutdown { id });
        if let Ok(mut stdin) = self.stdin.lock() {
            let _ = stdin.take();
        }
        if let Ok(mut child) = self.child.lock() {
            let _ = child.wait();
        }
    }
}

/// Whether the embed helper binary is locatable — health surface only;
/// spawning still does its own resolution.
pub fn helper_available() -> bool {
    resolve_helper_path().is_ok()
}

/// Same resolution order as the ghost-text helper client (env override, then
/// the stable install dir, then a sibling of the current executable).
fn resolve_helper_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os(HELPER_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }
    if let Some(home) = dirs::home_dir() {
        let stable = home.join(".scriptkit").join("bin").join(HELPER_NAME);
        if stable.is_file() {
            return Ok(stable);
        }
    }
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from));
    while let Some(current) = dir {
        let candidate = current.join(HELPER_NAME);
        if candidate.is_file() {
            return Ok(candidate);
        }
        dir = current.parent().map(PathBuf::from);
    }
    Err(anyhow!("brain embed helper binary not found"))
}
