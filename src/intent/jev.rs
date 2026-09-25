//! Jev's wire format and the blocking client that asks it questions.
//!
//! Every question goes out in one request. Jev answers each one with a choice and a
//! probability for every option, a yes/no probability, or a rubric score.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::runtime_policy::{self, ExternalEffect};

pub const JEV_ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
/// TypeSafe rejects choice questions with more options than this.
pub const JEV_MAX_CHOICES: usize = 255;
const DEFAULT_MODEL: &str = "jev-latest";
/// Past this the launcher has moved on; a late plan would only be thrown away.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(6);
/// The key stays where the Jev demos keep it; it is read, never copied.
const DEMOS_ENV_FILE: &str = "dev/jev-demos/.env";

/// Options in the order they are offered. A repeated label replaces the earlier
/// description in place, the way a JSON object literal would.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Criteria(Vec<(String, Option<String>)>);

impl Criteria {
    pub fn set(&mut self, label: impl Into<String>, description: Option<String>) {
        let label = label.into();
        match self.0.iter_mut().find(|(existing, _)| *existing == label) {
            Some(slot) => slot.1 = description,
            None => self.0.push((label, description)),
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, label: &str) -> bool {
        self.0.iter().any(|(existing, _)| existing == label)
    }

    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(label, _)| label.as_str())
    }

    pub fn description(&self, label: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(existing, _)| existing == label)
            .and_then(|(_, description)| description.as_deref())
    }
}

impl Serialize for Criteria {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (label, description) in &self.0 {
            map.serialize_entry(label, description)?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    Choice {
        instructions: String,
        criteria: Criteria,
    },
    Noul {
        instructions: String,
    },
    Score {
        instructions: String,
        criteria: Vec<String>,
    },
}

/// Questions by name, in the order they are asked.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Questions(Vec<(String, Question)>);

impl Questions {
    pub fn insert(&mut self, name: impl Into<String>, question: Question) {
        let name = name.into();
        match self.0.iter_mut().find(|(existing, _)| *existing == name) {
            Some(slot) => slot.1 = question,
            None => self.0.push((name, question)),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Question> {
        self.0
            .iter()
            .find(|(existing, _)| existing == name)
            .map(|(_, question)| question)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Question)> {
        self.0
            .iter()
            .map(|(name, question)| (name.as_str(), question))
    }

    /// Jev refuses the whole request when one choice question is too long.
    pub fn check_choice_limits(&self) -> Result<(), JevError> {
        for (name, question) in self.iter() {
            if let Question::Choice { criteria, .. } = question {
                if criteria.len() > JEV_MAX_CHOICES {
                    return Err(JevError::TooManyChoices {
                        question: name.to_string(),
                        count: criteria.len(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl Serialize for Questions {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (name, question) in &self.0 {
            map.serialize_entry(name, question)?;
        }
        map.end()
    }
}

/// Label → probability, in the order Jev sent them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Probabilities(Vec<(String, f64)>);

impl Probabilities {
    pub fn new(entries: Vec<(String, f64)>) -> Self {
        Self(entries)
    }

    pub fn get(&self, label: &str) -> Option<f64> {
        self.0
            .iter()
            .find(|(existing, _)| existing == label)
            .map(|(_, p)| *p)
    }

    /// Most likely first; equal probabilities keep Jev's order.
    pub fn ranked(&self) -> Vec<(&str, f64)> {
        let mut out: Vec<(&str, f64)> = self.0.iter().map(|(l, p)| (l.as_str(), *p)).collect();
        out.sort_by(|a, b| b.1.total_cmp(&a.1));
        out
    }
}

impl Serialize for Probabilities {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (label, p) in &self.0 {
            map.serialize_entry(label, p)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Probabilities {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OrderedMap;
        impl<'de> Visitor<'de> for OrderedMap {
            type Value = Probabilities;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map of option labels to probabilities")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::with_capacity(access.size_hint().unwrap_or(0));
                while let Some((label, p)) = access.next_entry::<String, f64>()? {
                    entries.push((label, p));
                }
                Ok(Probabilities(entries))
            }
        }
        deserializer.deserialize_map(OrderedMap)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    Choice {
        choice: String,
        #[serde(default)]
        confidence: f64,
        #[serde(default)]
        probabilities: Probabilities,
    },
    Noul {
        noul: f64,
    },
    Score {
        score: f64,
        #[serde(default)]
        confidence: f64,
        #[serde(default)]
        probabilities: Probabilities,
    },
}

pub type Answers = HashMap<String, Answer>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct JevResult {
    pub answers: Answers,
    pub model: String,
    pub ms: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum JevError {
    #[error("no TypeSafe API key is set")]
    MissingKey,
    #[error("question {question} has {count} options; Jev allows {JEV_MAX_CHOICES}")]
    TooManyChoices { question: String, count: usize },
    #[error("refused by runtime policy: {0}")]
    Refused(&'static str),
    #[error("Jev request failed (HTTP {status}): {detail}")]
    Http { status: u16, detail: String },
    #[error("Jev request failed: {0}")]
    Transport(String),
    #[error("Jev answered with something unreadable: {0}")]
    Decode(String),
}

/// Anything that can answer Jev questions: the live client, or recorded answers in tests.
pub trait Oracle: Send + Sync {
    fn ask(&self, state: &serde_json::Value, questions: &Questions) -> Result<JevResult, JevError>;
}

#[derive(Clone)]
pub struct JevConfig {
    api_key: String,
    pub model: String,
}

impl fmt::Debug for JevConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JevConfig")
            .field("api_key", &"<redacted>")
            .field("model", &self.model)
            .finish()
    }
}

impl JevConfig {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    /// Environment first, then Script Kit's secret store, then the Jev demos' `.env`.
    pub fn load() -> Result<Self, JevError> {
        runtime_policy::check(ExternalEffect::Credentials)
            .map_err(|error| JevError::Refused(error.code))?;
        let demos = demos_env();
        let api_key = env_nonempty("SCRIPT_KIT_TYPESAFE_API_KEY")
            .or_else(|| env_nonempty("TYPESAFE_API_KEY"))
            .or_else(|| {
                crate::secrets::get_secret("TYPESAFE_API_KEY").filter(|v| !v.trim().is_empty())
            })
            .or_else(|| demos.get("TYPESAFE_API_KEY").cloned())
            .ok_or(JevError::MissingKey)?;
        let model = env_nonempty("TYPESAFE_MODEL")
            .or_else(|| demos.get("TYPESAFE_MODEL").cloned())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        Ok(Self::new(api_key, model))
    }
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn demos_env() -> HashMap<String, String> {
    let Some(path) = dirs::home_dir().map(|home| home.join(DEMOS_ENV_FILE)) else {
        return HashMap::new();
    };
    std::fs::read_to_string(path)
        .map(|text| parse_env(&text))
        .unwrap_or_default()
}

pub(crate) fn parse_env(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let (key, value) = line.trim().split_once('=')?;
            let key = key.trim();
            if key.is_empty() || !key.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                return None;
            }
            let value = value.trim().trim_matches('"').to_string();
            (!value.is_empty()).then(|| (key.to_string(), value))
        })
        .collect()
}

pub struct JevClient {
    config: JevConfig,
    agent: ureq::Agent,
}

impl JevClient {
    pub fn new(config: JevConfig) -> Self {
        let agent = ureq::Agent::config_builder()
            .https_only(true)
            .timeout_global(Some(REQUEST_TIMEOUT))
            .http_status_as_error(false)
            .build()
            .new_agent();
        Self { config, agent }
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }
}

#[derive(Deserialize, Default)]
struct Usage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}

#[derive(Deserialize)]
struct ResponseBody {
    answers: Answers,
    #[serde(default)]
    model: String,
    #[serde(default)]
    usage: Option<Usage>,
}

pub(crate) fn decode_response(status: u16, text: &str, ms: u64) -> Result<JevResult, JevError> {
    if !(200..300).contains(&status) {
        return Err(JevError::Http {
            status,
            detail: text.chars().take(400).collect(),
        });
    }
    let parsed: ResponseBody =
        serde_json::from_str(text).map_err(|error| JevError::Decode(error.to_string()))?;
    let usage = parsed.usage.unwrap_or_default();
    Ok(JevResult {
        answers: parsed.answers,
        model: parsed.model,
        ms,
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
    })
}

impl Oracle for JevClient {
    fn ask(&self, state: &serde_json::Value, questions: &Questions) -> Result<JevResult, JevError> {
        questions.check_choice_limits()?;
        runtime_policy::check(ExternalEffect::Provider)
            .map_err(|error| JevError::Refused(error.code))?;
        let body = serde_json::json!({
            "model": self.config.model,
            "state": state,
            "questions": questions,
        });
        let started = Instant::now();
        let mut response = self
            .agent
            .post(JEV_ENDPOINT)
            .header("Authorization", &format!("Bearer {}", self.config.api_key))
            .header("Accept", "application/json")
            .send_json(&body)
            .map_err(|error| JevError::Transport(error.to_string()))?;
        let status = response.status().as_u16();
        let text = response
            .body_mut()
            .read_to_string()
            .map_err(|error| JevError::Transport(error.to_string()))?;
        decode_response(status, &text, started.elapsed().as_millis() as u64)
    }
}
