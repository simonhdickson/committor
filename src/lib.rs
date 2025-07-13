//! Committor - Automatically generate conventional commit messages based on git diff using AI
//!
//! This library provides the core functionality for analyzing git diffs and generating
//! conventional commit messages using AI models.

pub mod commit;
pub mod diff;
pub mod prompt;
pub mod providers;
pub mod types;

use anyhow::Result;
use providers::{create_provider, AIProvider, ProviderConfig};
use std::env;
use std::time::Duration;

/// Main configuration for the committor
pub struct Config {
    pub provider_config: ProviderConfig,
    pub count: u8,
    pub auto_commit: bool,
    pub show_diff: bool,
}

impl Config {
    /// Create a new configuration with default OpenAI values
    pub fn new() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable not set"))?;

        Ok(Config {
            provider_config: ProviderConfig::openai(api_key, "gpt-4".to_string()),
            count: 3,
            auto_commit: false,
            show_diff: false,
        })
    }

    /// Create a new configuration with OpenAI provider
    pub fn with_openai(
        api_key: String,
        model: String,
        count: u8,
        auto_commit: bool,
        show_diff: bool,
    ) -> Self {
        Config {
            provider_config: ProviderConfig::openai(api_key, model),
            count,
            auto_commit,
            show_diff,
        }
    }

    /// Create a new configuration with Ollama provider
    pub fn with_ollama(
        base_url: String,
        model: String,
        count: u8,
        auto_commit: bool,
        show_diff: bool,
    ) -> Self {
        Config {
            provider_config: ProviderConfig::ollama(base_url, model),
            count,
            auto_commit,
            show_diff,
        }
    }

    /// Create a new configuration with Ollama provider and custom timeout
    pub fn with_ollama_timeout(
        base_url: String,
        model: String,
        timeout: Duration,
        count: u8,
        auto_commit: bool,
        show_diff: bool,
    ) -> Self {
        Config {
            provider_config: ProviderConfig::ollama_with_timeout(base_url, model, timeout),
            count,
            auto_commit,
            show_diff,
        }
    }
}

/// Main committor service
pub struct Committor {
    config: Config,
    provider: Box<dyn AIProvider>,
}

impl Committor {
    /// Create a new committor instance
    pub fn new(config: Config) -> Result<Self> {
        let provider = create_provider(config.provider_config.clone())?;
        Ok(Self { config, provider })
    }

    /// Generate commit messages for the given diff
    pub async fn generate_commit_messages(&self, diff: &str) -> Result<Vec<String>> {
        commit::generate_commit_messages(diff, &*self.provider, self.config.count).await
    }

    /// Get the staged diff from the repository
    pub fn get_staged_diff(&self) -> Result<String> {
        diff::get_staged_diff()
    }

    /// Commit with the given message
    pub fn commit_with_message(&self, message: &str) -> Result<()> {
        commit::commit_with_message(message)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Config {
            provider_config: ProviderConfig::openai(String::new(), "gpt-4".to_string()),
            count: 3,
            auto_commit: false,
            show_diff: false,
        })
    }
}
