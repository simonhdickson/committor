//! AI provider abstraction for different AI services

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rig::{
    client::CompletionClient,
    completion::Prompt,
    providers::{ollama, openai},
};

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
pub async fn check_ollama_availability(base_url: &str, model: &str) -> Result<bool> {
    let client = if base_url == "http://localhost:11434" {
        ollama::Client::new()
    } else {
        ollama::Client::from_url(base_url)
    };

    // Try a minimal test with a simple model to check availability
    // We use a very short prompt to minimize resource usage
    match client.agent(model).build().prompt("test").await {
        Ok(_) => Ok(true),
        Err(_) => Err(anyhow!(
            "Ollama is not available or {model} model not installed"
        )),
    }
}

/// Get available models from Ollama
pub async fn get_ollama_models(_base_url: &str) -> Result<Vec<String>> {
    // Note: rig.rs Ollama client doesn't expose model listing directly
    // This is a limitation of the current rig.rs implementation
    // For now, return a list of common models
    Ok(vec![
        "llama2".to_string(),
        "llama3.2".to_string(),
        "codellama".to_string(),
        "mistral".to_string(),
        "deepseek-coder".to_string(),
        "neural-chat".to_string(),
        "tinyllama".to_string(),
    ])
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
