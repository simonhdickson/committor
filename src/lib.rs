//! Commitor - Automatically generate conventional commit messages based on git diff using AI
//!
//! This library provides the core functionality for analyzing git diffs and generating
//! conventional commit messages using AI models.

pub mod commit;
pub mod diff;
pub mod prompt;
pub mod types;

use anyhow::Result;
use rig::providers::openai;
use std::env;

/// Main configuration for the commitor
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub count: u8,
    pub auto_commit: bool,
    pub show_diff: bool,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable not set"))?;

        Ok(Config {
            api_key,
            model: "gpt-4".to_string(),
            count: 3,
            auto_commit: false,
            show_diff: false,
        })
    }

    /// Create a new configuration with custom values
    pub fn with_options(
        api_key: String,
        model: String,
        count: u8,
        auto_commit: bool,
        show_diff: bool,
    ) -> Self {
        Config {
            api_key,
            model,
            count,
            auto_commit,
            show_diff,
        }
    }
}

/// Main commitor service
pub struct Commitor {
    config: Config,
    client: openai::Client,
}

impl Commitor {
    /// Create a new commitor instance
    pub fn new(config: Config) -> Self {
        let client = openai::Client::new(&config.api_key);
        Self { config, client }
    }

    /// Generate commit messages for the given diff
    pub async fn generate_commit_messages(&self, diff: &str) -> Result<Vec<String>> {
        commit::generate_commit_messages(diff, &self.client, &self.config.model, self.config.count)
            .await
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
            api_key: String::new(),
            model: "gpt-4".to_string(),
            count: 3,
            auto_commit: false,
            show_diff: false,
        })
    }
}
