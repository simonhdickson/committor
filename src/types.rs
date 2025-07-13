//! Common types and data structures used throughout the commitor application

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a conventional commit type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitType {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Test,
    Chore,
    Perf,
    Ci,
    Build,
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            CommitType::Feat => "feat",
            CommitType::Fix => "fix",
            CommitType::Docs => "docs",
            CommitType::Style => "style",
            CommitType::Refactor => "refactor",
            CommitType::Test => "test",
            CommitType::Chore => "chore",
            CommitType::Perf => "perf",
            CommitType::Ci => "ci",
            CommitType::Build => "build",
        };
        write!(f, "{}", type_str)
    }
}

impl CommitType {
    /// Get all available commit types
    pub fn all() -> Vec<CommitType> {
        vec![
            CommitType::Feat,
            CommitType::Fix,
            CommitType::Docs,
            CommitType::Style,
            CommitType::Refactor,
            CommitType::Test,
            CommitType::Chore,
            CommitType::Perf,
            CommitType::Ci,
            CommitType::Build,
        ]
    }

    /// Get the description of the commit type
    pub fn description(&self) -> &'static str {
        match self {
            CommitType::Feat => "A new feature",
            CommitType::Fix => "A bug fix",
            CommitType::Docs => "Documentation only changes",
            CommitType::Style => "Changes that do not affect the meaning of the code",
            CommitType::Refactor => "A code change that neither fixes a bug nor adds a feature",
            CommitType::Test => "Adding missing tests or correcting existing tests",
            CommitType::Chore => "Changes to the build process or auxiliary tools",
            CommitType::Perf => "A code change that improves performance",
            CommitType::Ci => "Changes to CI configuration files and scripts",
            CommitType::Build => "Changes that affect the build system or external dependencies",
        }
    }
}

/// Represents a conventional commit message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConventionalCommit {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub description: String,
    pub breaking: bool,
}

impl ConventionalCommit {
    /// Create a new conventional commit
    pub fn new(commit_type: CommitType, description: String) -> Self {
        Self {
            commit_type,
            scope: None,
            description,
            breaking: false,
        }
    }

    /// Set the scope of the commit
    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Mark the commit as breaking
    pub fn with_breaking(mut self) -> Self {
        self.breaking = true;
        self
    }
}

impl fmt::Display for ConventionalCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let breaking_indicator = if self.breaking { "!" } else { "" };

        match &self.scope {
            Some(scope) => write!(
                f,
                "{}({}){}: {}",
                self.commit_type, scope, breaking_indicator, self.description
            ),
            None => write!(
                f,
                "{}{}: {}",
                self.commit_type, breaking_indicator, self.description
            ),
        }
    }
}

/// Represents a git diff change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffChange {
    pub file_path: String,
    pub change_type: DiffChangeType,
    pub additions: usize,
    pub deletions: usize,
}

/// Type of change in a git diff
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

impl fmt::Display for DiffChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            DiffChangeType::Added => "added",
            DiffChangeType::Modified => "modified",
            DiffChangeType::Deleted => "deleted",
            DiffChangeType::Renamed => "renamed",
            DiffChangeType::Copied => "copied",
        };
        write!(f, "{}", type_str)
    }
}

/// Represents the result of generating commit messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub messages: Vec<String>,
    pub model_used: String,
    pub generation_time: std::time::Duration,
}

/// Error types specific to commitor
#[derive(Debug, thiserror::Error)]
pub enum CommitorError {
    #[error("Git repository not found")]
    GitRepoNotFound,

    #[error("No staged changes found")]
    NoStagedChanges,

    #[error("OpenAI API error: {0}")]
    OpenAIError(String),

    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("Invalid commit message format: {0}")]
    InvalidCommitFormat(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
