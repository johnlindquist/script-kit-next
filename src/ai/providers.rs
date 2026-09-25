//! AI provider layer for single-request callers.
//!
//! The SDK chat prompt, AI script generation, and conversation titles send a
//! message list and read the reply. Every request runs through the goose
//! engine ([`GooseProvider`]), so these callers get the same providers,
//! models, sign-ins, and local-model support as Agent Chat.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::config::ModelInfo;
use crate::ai::agent_chat::content::ImageContent;
use crate::ai::agent_chat::goose::oneshot::{self, OneshotRequest};

/// Image data for multimodal API calls
#[derive(Debug, Clone)]
pub struct ProviderImage {
    /// Base64 encoded image data
    pub data: String,
    /// MIME type of the image (e.g., "image/png", "image/jpeg")
    pub media_type: String,
}

impl ProviderImage {
    /// Create a new image from base64 data
    pub fn new(data: String, media_type: String) -> Self {
        Self { data, media_type }
    }

    /// Create a PNG image
    pub fn png(data: String) -> Self {
        Self::new(data, "image/png".to_string())
    }

    /// Create a JPEG image
    pub fn jpeg(data: String) -> Self {
        Self::new(data, "image/jpeg".to_string())
    }
}

/// Message for AI provider API calls.
#[derive(Debug, Clone)]
pub struct ProviderMessage {
    /// Role of the message sender: "user", "assistant", or "system"
    pub role: String,
    /// Text content of the message
    pub content: String,
    /// Image attachments for multimodal messages
    pub images: Vec<ProviderImage>,
}

impl ProviderMessage {
    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// Create a new user message with images.
    pub fn user_with_images(content: impl Into<String>, images: Vec<ProviderImage>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            images,
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            images: Vec::new(),
        }
    }

    /// Check if this message has images attached
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }
}

/// Callback type for streaming responses.
pub type StreamCallback = Box<dyn Fn(String) -> bool + Send + Sync>;

/// Interface the SDK chat prompt, script generation, and conversation titles
/// use to reach a model. Methods are synchronous; callers run them off the UI
/// thread.
pub trait AiProvider: Send + Sync {
    /// Unique identifier for this provider.
    fn provider_id(&self) -> &str;

    /// Human-readable display name.
    fn display_name(&self) -> &str;

    /// Get the list of available models for this provider.
    fn available_models(&self) -> Vec<ModelInfo>;

    /// Send a message and get a response (non-streaming).
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation history
    /// * `model_id` - The model to use for generation
    ///
    /// # Returns
    ///
    /// The generated response text, or an error.
    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String>;

    /// Send a message with streaming response.
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation history
    /// * `model_id` - The model to use for generation
    /// * `on_chunk` - Callback invoked for each chunk of the response
    /// * `session_id` - Optional conversation id (unused by stateless providers)
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error.
    fn stream_message(
        &self,
        messages: &[ProviderMessage],
        model_id: &str,
        on_chunk: StreamCallback,
        session_id: Option<&str>,
    ) -> Result<()>;
}

/// Provider id of the goose engine in the registry.
pub const GOOSE_PROVIDER_ID: &str = "goose";

/// Every model request runs through a short-lived goose session.
#[derive(Debug, Clone, Default)]
pub struct GooseProvider;

impl GooseProvider {
    /// Build the goose request for a message list. goose sessions here are
    /// stateless, so earlier turns travel as a transcript in the prompt.
    fn request(messages: &[ProviderMessage], model_id: &str) -> OneshotRequest {
        let system: Vec<&str> = messages
            .iter()
            .filter(|message| message.role == "system")
            .map(|message| message.content.as_str())
            .filter(|content| !content.trim().is_empty())
            .collect();
        let turns: Vec<&ProviderMessage> = messages
            .iter()
            .filter(|message| message.role != "system")
            .collect();
        let (last, history) = match turns.split_last() {
            Some((last, history)) => (Some(*last), history),
            None => (None, &[][..]),
        };
        let mut prompt = String::new();
        if !history.is_empty() {
            prompt.push_str("Conversation so far:\n");
            for message in history {
                let speaker = if message.role == "assistant" {
                    "Assistant"
                } else {
                    "User"
                };
                prompt.push_str(&format!("\n{speaker}: {}\n", message.content.trim()));
            }
            prompt.push_str("\nReply to the latest message:\n\n");
        }
        if let Some(last) = last {
            prompt.push_str(&last.content);
        }
        let images = last
            .map(|message| {
                message
                    .images
                    .iter()
                    .map(|image| ImageContent::new(image.data.clone(), image.media_type.clone()))
                    .collect()
            })
            .unwrap_or_default();
        OneshotRequest {
            system: (!system.is_empty()).then(|| system.join("\n\n")),
            prompt,
            images,
            model: (!model_id.trim().is_empty()).then(|| model_id.trim().to_string()),
            timeout: None,
            // The SDK chat prompt, script generation, and the built-in AI
            // prompt all answer a person waiting on screen.
            interactive_sign_in: true,
        }
    }
}

impl AiProvider for GooseProvider {
    fn provider_id(&self) -> &str {
        GOOSE_PROVIDER_ID
    }

    fn display_name(&self) -> &str {
        "goose"
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        goose_model_catalog()
    }

    fn send_message(&self, messages: &[ProviderMessage], model_id: &str) -> Result<String> {
        oneshot::complete(Self::request(messages, model_id))
    }

    fn stream_message(
        &self,
        messages: &[ProviderMessage],
        model_id: &str,
        on_chunk: StreamCallback,
        _session_id: Option<&str>,
    ) -> Result<()> {
        oneshot::stream(Self::request(messages, model_id), |delta| {
            on_chunk(delta.to_string())
        })
    }
}

/// Models offered to single-request callers: the user's selected Agent Chat
/// model first, then the curated catalog. Ids are `provider/model`.
pub fn goose_model_catalog() -> Vec<ModelInfo> {
    use crate::ai::agent_chat::profiles::{
        advanced_provider_model_catalog, provider_model_catalog, DEFAULT_MODEL, DEFAULT_PROVIDER,
    };
    let (preferred_provider, preferred_model) = oneshot::resolve_model(None);
    let mut models = vec![ModelInfo::new(
        format!("{preferred_provider}/{preferred_model}"),
        preferred_model.clone(),
        GOOSE_PROVIDER_ID,
        true,
        0,
    )];
    for entry in provider_model_catalog()
        .into_iter()
        .chain(advanced_provider_model_catalog())
    {
        for (model, name) in entry.models {
            let id = format!("{}/{model}", entry.id);
            if models.iter().any(|existing| existing.id == id) {
                continue;
            }
            models.push(ModelInfo::new(
                id,
                format!("{name} · {}", entry.display_name),
                GOOSE_PROVIDER_ID,
                true,
                0,
            ));
        }
    }
    if models.is_empty() {
        models.push(ModelInfo::new(
            format!("{DEFAULT_PROVIDER}/{DEFAULT_MODEL}"),
            DEFAULT_MODEL,
            GOOSE_PROVIDER_ID,
            true,
            0,
        ));
    }
    models
}

/// Registry of available AI providers.
///
/// Holds the goose engine provider when it is installed; tests register
/// fixture providers directly.
#[derive(Clone)]
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Create a registry with the goose engine, when it is installed.
    pub fn from_environment() -> Self {
        Self::from_environment_with_config(None)
    }

    /// Create a registry with the goose engine, when it is installed.
    ///
    /// Provider credentials, sign-ins, and model choice all belong to goose;
    /// `_config` is accepted for callers that pass the loaded config.
    pub fn from_environment_with_config(_config: Option<&crate::config::Config>) -> Self {
        let mut registry = Self::new();
        if crate::ai::agent_chat::goose::binary::default_goose_binary().is_some() {
            registry.register(Arc::new(GooseProvider));
        } else {
            tracing::debug!("goose engine not installed; no AI provider available");
        }
        registry
    }

    /// Register a provider with the registry.
    pub fn register(&mut self, provider: Arc<dyn AiProvider>) {
        self.providers
            .insert(provider.provider_id().to_string(), provider);
    }

    /// Check if any providers are available.
    pub fn has_any_provider(&self) -> bool {
        !self.providers.is_empty()
    }

    /// Get a provider by ID.
    pub fn get_provider(&self, id: &str) -> Option<&Arc<dyn AiProvider>> {
        self.providers.get(id)
    }

    /// Get all registered provider IDs.
    pub fn provider_ids(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Get all available models from all providers.
    pub fn get_all_models(&self) -> Vec<ModelInfo> {
        let mut models = Vec::new();
        for provider in self.providers.values() {
            models.extend(provider.available_models());
        }
        models
    }

    /// Get models for a specific provider.
    pub fn get_models_for_provider(&self, provider_id: &str) -> Vec<ModelInfo> {
        self.providers
            .get(provider_id)
            .map(|p| p.available_models())
            .unwrap_or_default()
    }

    /// Find the provider that owns a specific model.
    pub fn find_provider_for_model(&self, model_id: &str) -> Option<&Arc<dyn AiProvider>> {
        self.providers
            .values()
            .find(|provider| provider.available_models().iter().any(|m| m.id == model_id))
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
include!("providers_tests.rs");
