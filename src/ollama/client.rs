use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};
use url::Url;

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub timeout: Duration,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "phi4:latest".to_string(),
            temperature: 0.7,
            max_tokens: Some(2048),
            timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>, // max_tokens in Ollama
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    #[serde(default)]
    pub context: Vec<u32>,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
    base_url: Url,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to create HTTP client")?;

        let base_url = Url::parse(&config.base_url)
            .context("Invalid Ollama base URL")?;

        info!("Initialized Ollama client for model: {}", config.model);
        
        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(self.config.temperature),
                num_predict: self.config.max_tokens,
                top_p: Some(0.9),
                top_k: Some(40),
            }),
        };

        debug!("Sending request to Ollama: {}", prompt);

        let url = self.base_url.join("/api/generate")
            .context("Failed to construct Ollama API URL")?;

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Ollama API error {}: {}", status, text);
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        debug!(
            "Ollama response: {} tokens, duration: {:?}ms",
            ollama_response.eval_count.unwrap_or(0),
            ollama_response.total_duration.map(|d| d / 1_000_000) // Convert to ms
        );

        Ok(ollama_response.response)
    }

    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String> {
        // Convert chat messages to a single prompt for Ollama
        let prompt = self.format_chat_prompt(messages);
        self.generate(&prompt).await
    }

    fn format_chat_prompt(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();
        
        for message in messages {
            match message.role.as_str() {
                "system" => {
                    prompt.push_str(&format!("System: {}\n", message.content));
                }
                "user" => {
                    prompt.push_str(&format!("User: {}\n", message.content));
                }
                "assistant" => {
                    prompt.push_str(&format!("Assistant: {}\n", message.content));
                }
                _ => {
                    prompt.push_str(&format!("{}: {}\n", message.role, message.content));
                }
            }
        }
        
        prompt.push_str("Assistant: ");
        prompt
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = self.base_url.join("/api/tags")
            .context("Failed to construct Ollama health check URL")?;

        match self.client.get(url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("Ollama health check failed: {}", e);
                Ok(false)
            }
        }
    }

    pub fn get_model(&self) -> &str {
        &self.config.model
    }

    pub fn set_model(&mut self, model: String) {
        info!("Switching Ollama model from {} to {}", self.config.model, model);
        self.config.model = model;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: content.to_string(),
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
        }
    }
}
