//! AI provider abstraction for different AI services

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use rig::{
    client::CompletionClient,
    completion::Prompt,
    providers::{ollama, openai},
};
use serde::Deserialize;
use std::time::Duration;

/// Trait for AI providers that can generate commit messages
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_message(&self, prompt: &str) -> Result<String>;
    fn provider_name(&self) -> &'static str;
}

/// Configuration for different AI providers
#[derive(Debug, Clone)]
pub enum ProviderConfig {
    OpenAI {
        api_key: String,
        model: String,
    },
    Ollama {
        base_url: String,
        model: String,
        timeout: Duration,
    },
}

impl ProviderConfig {
    /// Create an OpenAI provider configuration
    pub fn openai(api_key: String, model: String) -> Self {
        Self::OpenAI { api_key, model }
    }

    /// Create an Ollama provider configuration
    pub fn ollama(base_url: String, model: String) -> Self {
        Self::Ollama {
            base_url,
            model,
            timeout: Duration::from_secs(30),
        }
    }

    /// Create an Ollama provider configuration with custom timeout
    pub fn ollama_with_timeout(base_url: String, model: String, timeout: Duration) -> Self {
        Self::Ollama {
            base_url,
            model,
            timeout,
        }
    }
}

/// OpenAI provider implementation
pub struct OpenAIProvider {
    client: openai::Client,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        let client = openai::Client::new(&api_key);
        Self { client, model }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate_message(&self, prompt: &str) -> Result<String> {
        let agent = self.client.agent(&self.model).build();
        let response = agent.prompt(prompt).await?;
        Ok(response.trim().to_string())
    }

    fn provider_name(&self) -> &'static str {
        "OpenAI"
    }
}

/// Ollama provider implementation
pub struct OllamaProvider {
    client: ollama::Client,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String, _timeout: Duration) -> Result<Self> {
        let client = if base_url == "http://localhost:11434" {
            ollama::Client::new()
        } else {
            ollama::Client::from_url(&base_url)
        };

        Ok(Self { client, model })
    }

    pub fn with_default_url(model: String) -> Result<Self> {
        Ok(Self {
            client: ollama::Client::new(),
            model,
        })
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn generate_message(&self, prompt: &str) -> Result<String> {
        let agent = self.client.agent(&self.model).build();
        let response = agent.prompt(prompt).await?;
        Ok(response.trim().to_string())
    }

    fn provider_name(&self) -> &'static str {
        "Ollama"
    }
}

/// Factory function to create AI providers
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn AIProvider>> {
    match config {
        ProviderConfig::OpenAI { api_key, model } => {
            Ok(Box::new(OpenAIProvider::new(api_key, model)))
        }
        ProviderConfig::Ollama {
            base_url,
            model,
            timeout,
        } => {
            let provider = OllamaProvider::new(base_url, model, timeout)?;
            Ok(Box::new(provider))
        }
    }
}

/// Check if Ollama is available at the given URL
pub async fn check_ollama_availability(base_url: &str) -> Result<bool> {
    let client = HttpClient::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));

    match client.get(&url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Get available models from Ollama using /api/tags endpoint
pub async fn get_ollama_models(base_url: &str) -> Result<Vec<String>> {
    let client = HttpClient::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to get models from Ollama: {}",
            response.status()
        ));
    }

    #[derive(Deserialize)]
    struct ModelInfo {
        name: String,
    }

    #[derive(Deserialize)]
    struct ModelsResponse {
        models: Vec<ModelInfo>,
    }

    let models_response: ModelsResponse = response.json().await?;
    let models = models_response.models.into_iter().map(|m| m.name).collect();

    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_creation() {
        let openai_config = ProviderConfig::openai("test-key".to_string(), "gpt-4".to_string());
        match openai_config {
            ProviderConfig::OpenAI { api_key, model } => {
                assert_eq!(api_key, "test-key");
                assert_eq!(model, "gpt-4");
            }
            _ => panic!("Expected OpenAI config"),
        }

        let ollama_config =
            ProviderConfig::ollama("http://localhost:11434".to_string(), "llama2".to_string());
        match ollama_config {
            ProviderConfig::Ollama {
                base_url, model, ..
            } => {
                assert_eq!(base_url, "http://localhost:11434");
                assert_eq!(model, "llama2");
            }
            _ => panic!("Expected Ollama config"),
        }
    }

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama2".to_string(),
            Duration::from_secs(30),
        );
        assert!(provider.is_ok());
    }

    #[test]
    fn test_ollama_provider_with_default_url() {
        let provider = OllamaProvider::with_default_url("llama2".to_string());
        assert!(provider.is_ok());
    }
}
