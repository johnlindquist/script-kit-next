mod llama_engine;

use anyhow::{anyhow, Context as _, Result};
use llama_engine::{GhostSamplingParams, LoadedEmbedder, LoadedLocalLlm};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WireRequest {
    Load {
        id: u64,
        model_path: String,
        model_id: String,
        sampling: GhostSamplingParams,
    },
    Generate {
        id: u64,
        model_path: String,
        model_id: String,
        prompt: String,
        sampling: GhostSamplingParams,
    },
    Embed {
        id: u64,
        model_path: String,
        model_id: String,
        texts: Vec<String>,
        #[serde(default)]
        gpu_layers: u32,
    },
    Cancel {
        id: u64,
    },
    Shutdown {
        id: u64,
    },
}

#[derive(Debug, Serialize)]
struct WireResponse {
    id: u64,
    ok: bool,
    model_id: Option<String>,
    raw_completion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeddings: Option<Vec<Vec<f32>>>,
    error: Option<String>,
}

enum WorkerRequest {
    Load {
        id: u64,
        model_path: String,
        model_id: String,
        sampling: GhostSamplingParams,
    },
    Generate {
        id: u64,
        model_path: String,
        model_id: String,
        prompt: String,
        sampling: GhostSamplingParams,
    },
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

#[derive(Default)]
struct CancelRegistry {
    active: HashMap<u64, Arc<AtomicBool>>,
    early: HashSet<u64>,
}

impl CancelRegistry {
    fn register(&mut self, id: u64) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(self.early.remove(&id)));
        self.active.insert(id, Arc::clone(&flag));
        flag
    }

    fn cancel(&mut self, id: u64) {
        if let Some(flag) = self.active.get(&id) {
            flag.store(true, Ordering::Relaxed);
        } else {
            self.early.insert(id);
        }
    }

    fn unregister(&mut self, id: u64) {
        self.active.remove(&id);
        self.early.remove(&id);
    }
}

#[derive(Default)]
struct HelperEngine {
    loaded: Option<LoadedLocalLlm>,
    embedder: Option<LoadedEmbedder>,
}

impl HelperEngine {
    fn load_if_needed(
        &mut self,
        model_path: &str,
        model_id: &str,
        sampling: GhostSamplingParams,
    ) -> Result<()> {
        if self
            .loaded
            .as_ref()
            .is_some_and(|loaded| loaded.model_id() == model_id)
        {
            return Ok(());
        }
        let path = PathBuf::from(model_path);
        self.loaded = Some(LoadedLocalLlm::load(&path, model_id, sampling)?);
        Ok(())
    }

    fn generate(
        &mut self,
        model_path: &str,
        model_id: &str,
        prompt: &str,
        sampling: GhostSamplingParams,
        cancel: &Arc<AtomicBool>,
        request_id: u64,
    ) -> Result<String> {
        self.load_if_needed(model_path, model_id, sampling)?;
        self.loaded
            .as_mut()
            .context("ghost llm helper model not loaded")?
            .generate_one_line(prompt, cancel, request_id)
    }

    fn embed(
        &mut self,
        model_path: &str,
        model_id: &str,
        texts: &[String],
        gpu_layers: u32,
    ) -> Result<Vec<Vec<f32>>> {
        if !self
            .embedder
            .as_ref()
            .is_some_and(|embedder| embedder.model_id() == model_id)
        {
            let path = PathBuf::from(model_path);
            self.embedder = Some(LoadedEmbedder::load(&path, model_id, gpu_layers)?);
        }
        self.embedder
            .as_ref()
            .context("embedding model not loaded")?
            .embed_texts(texts)
    }
}

impl WireResponse {
    fn ok_load(id: u64, model_id: String) -> Self {
        Self {
            id,
            ok: true,
            model_id: Some(model_id),
            raw_completion: None,
            embeddings: None,
            error: None,
        }
    }

    fn ok_generate(id: u64, model_id: String, raw_completion: String) -> Self {
        Self {
            id,
            ok: true,
            model_id: Some(model_id),
            raw_completion: Some(raw_completion),
            embeddings: None,
            error: None,
        }
    }

    fn ok_embed(id: u64, model_id: String, embeddings: Vec<Vec<f32>>) -> Self {
        Self {
            id,
            ok: true,
            model_id: Some(model_id),
            raw_completion: None,
            embeddings: Some(embeddings),
            error: None,
        }
    }

    fn ok_shutdown(id: u64) -> Self {
        Self {
            id,
            ok: true,
            model_id: None,
            raw_completion: None,
            embeddings: None,
            error: None,
        }
    }

    fn err(id: u64, err: anyhow::Error) -> Self {
        Self {
            id,
            ok: false,
            model_id: None,
            raw_completion: None,
            embeddings: None,
            error: Some(err.to_string()),
        }
    }
}

fn write_response(response: WireResponse) -> Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &response).context("write helper response")?;
    stdout.write_all(b"\n").context("write helper newline")?;
    stdout.flush().context("flush helper response")
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel::<WorkerRequest>();
    let cancels = Arc::new(Mutex::new(CancelRegistry::default()));
    let reader_cancels = Arc::clone(&cancels);

    std::thread::Builder::new()
        .name("script-kit-ghost-llm-helper-stdin".to_string())
        .spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let Ok(line) = line else {
                    break;
                };
                let Ok(request) = serde_json::from_str::<WireRequest>(&line) else {
                    let _ =
                        write_response(WireResponse::err(0, anyhow!("malformed helper request")));
                    continue;
                };
                match request {
                    WireRequest::Cancel { id } => {
                        if let Ok(mut cancels) = reader_cancels.lock() {
                            cancels.cancel(id);
                        }
                    }
                    WireRequest::Load {
                        id,
                        model_path,
                        model_id,
                        sampling,
                    } => {
                        let _ = tx.send(WorkerRequest::Load {
                            id,
                            model_path,
                            model_id,
                            sampling,
                        });
                    }
                    WireRequest::Generate {
                        id,
                        model_path,
                        model_id,
                        prompt,
                        sampling,
                    } => {
                        let _ = tx.send(WorkerRequest::Generate {
                            id,
                            model_path,
                            model_id,
                            prompt,
                            sampling,
                        });
                    }
                    WireRequest::Embed {
                        id,
                        model_path,
                        model_id,
                        texts,
                        gpu_layers,
                    } => {
                        let _ = tx.send(WorkerRequest::Embed {
                            id,
                            model_path,
                            model_id,
                            texts,
                            gpu_layers,
                        });
                    }
                    WireRequest::Shutdown { id } => {
                        let _ = tx.send(WorkerRequest::Shutdown { id });
                        break;
                    }
                }
            }
        })
        .context("spawn ghost llm helper stdin thread")?;

    let mut engine = HelperEngine::default();
    while let Ok(request) = rx.recv() {
        match request {
            WorkerRequest::Load {
                id,
                model_path,
                model_id,
                sampling,
            } => {
                let response = match engine.load_if_needed(&model_path, &model_id, sampling) {
                    Ok(()) => WireResponse::ok_load(id, model_id),
                    Err(err) => WireResponse::err(id, err),
                };
                write_response(response)?;
            }
            WorkerRequest::Generate {
                id,
                model_path,
                model_id,
                prompt,
                sampling,
            } => {
                let cancel = if let Ok(mut cancels) = cancels.lock() {
                    cancels.register(id)
                } else {
                    Arc::new(AtomicBool::new(false))
                };
                let response =
                    match engine.generate(&model_path, &model_id, &prompt, sampling, &cancel, id) {
                        Ok(raw_completion) => {
                            WireResponse::ok_generate(id, model_id, raw_completion)
                        }
                        Err(err) => WireResponse::err(id, err),
                    };
                if let Ok(mut cancels) = cancels.lock() {
                    cancels.unregister(id);
                }
                write_response(response)?;
            }
            WorkerRequest::Embed {
                id,
                model_path,
                model_id,
                texts,
                gpu_layers,
            } => {
                let response = match engine.embed(&model_path, &model_id, &texts, gpu_layers) {
                    Ok(embeddings) => WireResponse::ok_embed(id, model_id, embeddings),
                    Err(err) => WireResponse::err(id, err),
                };
                write_response(response)?;
            }
            WorkerRequest::Shutdown { id } => {
                write_response(WireResponse::ok_shutdown(id))?;
                break;
            }
        }
    }
    Ok(())
}
