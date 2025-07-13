//! Commit operations for generating conventional commit messages and executing git commits

use crate::prompt::create_commit_prompt;
use crate::providers::AIProvider;
use crate::types::{CommittorError, ConventionalCommit};
use anyhow::{Context, Result};
use colored::*;
use std::io::{self, Write};
use std::process::Command;
use std::time::Instant;
use tracing::{info, warn};

/// Generate commit messages using AI
pub async fn generate_commit_messages(
    diff: &str,
    provider: &dyn AIProvider,
    count: u8,
) -> Result<Vec<String>> {
    info!(
        "Generating commit messages using provider: {}",
        provider.provider_name()
    );

    let start_time = Instant::now();
    let prompt = create_commit_prompt(diff);

    let mut messages = Vec::new();
    let mut attempts = 0;
    let max_attempts = count as usize * 2; // Allow more attempts than requested count

    while messages.len() < count as usize && attempts < max_attempts {
        attempts += 1;

        match provider.generate_message(&prompt).await {
            Ok(response) => {
                let message = response.trim().to_string();
                if !message.is_empty() && is_valid_commit_message(&message) {
                    // Avoid duplicates
                    if !messages.contains(&message) {
                        messages.push(message);
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Failed to generate commit message (attempt {}): {}",
                    attempts, e
                );
                if attempts == 1 {
                    // If first attempt fails, return the error
                    return Err(CommittorError::AIProviderError(e.to_string()).into());
                }
                // For subsequent attempts, just continue trying
            }
        }
    }

    let generation_time = start_time.elapsed();
    info!(
        "Generated {} messages in {:?}",
        messages.len(),
        generation_time
    );

    if messages.is_empty() {
        return Err(CommittorError::AIProviderError(
            "Failed to generate any valid commit messages".to_string(),
        )
        .into());
    }

    Ok(messages)
}

/// Validate if a commit message follows conventional commit format
pub fn is_valid_commit_message(message: &str) -> bool {
    // Basic validation for conventional commit format
    let regex = regex::Regex::new(
        r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build)(\(.+\))?: .+$",
    )
    .unwrap();
    regex.is_match(message) && message.len() <= 72
}

/// Parse a commit message into a ConventionalCommit struct
pub fn parse_commit_message(message: &str) -> Result<ConventionalCommit> {
    let regex = regex::Regex::new(
        r"^(feat|fix|docs|style|refactor|test|chore|perf|ci|build)(\(([^)]+)\))?(!)?: (.+)$",
    )
    .unwrap();

    if let Some(captures) = regex.captures(message) {
        let commit_type = match captures.get(1).unwrap().as_str() {
            "feat" => crate::types::CommitType::Feat,
            "fix" => crate::types::CommitType::Fix,
            "docs" => crate::types::CommitType::Docs,
            "style" => crate::types::CommitType::Style,
            "refactor" => crate::types::CommitType::Refactor,
            "test" => crate::types::CommitType::Test,
            "chore" => crate::types::CommitType::Chore,
            "perf" => crate::types::CommitType::Perf,
            "ci" => crate::types::CommitType::Ci,
            "build" => crate::types::CommitType::Build,
            _ => {
                return Err(
                    CommittorError::InvalidCommitFormat("Unknown commit type".to_string()).into(),
                )
            }
        };

        let scope = captures.get(3).map(|m| m.as_str().to_string());
        let breaking = captures.get(4).is_some();
        let description = captures.get(5).unwrap().as_str().to_string();

        let mut commit = ConventionalCommit::new(commit_type, description);
        if let Some(scope) = scope {
            commit = commit.with_scope(scope);
        }
        if breaking {
            commit = commit.with_breaking();
        }

        Ok(commit)
    } else {
        Err(
            CommittorError::InvalidCommitFormat("Invalid conventional commit format".to_string())
                .into(),
        )
    }
}

/// Display commit message options to the user
pub fn display_commit_options(messages: &[String]) {
    println!("{}", "Generated commit message options:".green().bold());
    println!();

    for (i, message) in messages.iter().enumerate() {
        println!("{} {}", format!("{}.", i + 1).cyan().bold(), message);
    }
    println!();
}

/// Prompt user to choose a commit message
pub fn prompt_user_choice(count: usize) -> Result<Option<usize>> {
    print!(
        "{}",
        format!("Choose an option (1-{count}, or 'q' to quit): ").yellow()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
        return Ok(None);
    }

    match input.parse::<usize>() {
        Ok(n) if n >= 1 && n <= count => Ok(Some(n - 1)),
        _ => {
            println!("{}", "Invalid choice. Please try again.".red());
            prompt_user_choice(count)
        }
    }
}

/// Execute a git commit with the given message
pub fn commit_with_message(message: &str) -> Result<()> {
    println!("{}", format!("Committing with message: {message}").green());

    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .context("Failed to execute git commit")?;

    if output.status.success() {
        println!("{}", "✓ Commit successful!".green().bold());

        // Show commit hash if available
        if let Ok(hash_output) = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
        {
            if hash_output.status.success() {
                let hash = String::from_utf8_lossy(&hash_output.stdout)
                    .trim()
                    .to_string();
                println!("{}", format!("Commit hash: {hash}").cyan());
            }
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(CommittorError::GitError(error.to_string()).into());
    }

    Ok(())
}

/// Check if git is available and we're in a git repository
pub fn validate_git_environment() -> Result<()> {
    // Check if git is available
    let git_version = Command::new("git")
        .args(["--version"])
        .output()
        .context("Git is not installed or not available in PATH")?;

    if !git_version.status.success() {
        return Err(anyhow::anyhow!("Git is not working properly"));
    }

    // Check if we're in a git repository
    let git_status = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .context("Not in a git repository")?;

    if !git_status.status.success() {
        return Err(CommittorError::GitRepoNotFound.into());
    }

    Ok(())
}

/// Get the current git branch name
pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(branch)
    } else {
        Ok("HEAD".to_string()) // Fallback for detached HEAD
    }
}

/// Get the last commit message
pub fn get_last_commit_message() -> Result<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%s"])
        .output()
        .context("Failed to get last commit message")?;

    if output.status.success() {
        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(message)
    } else {
        Err(anyhow::anyhow!("Failed to get last commit message"))
    }
}

/// Check if there are any uncommitted changes
pub fn has_uncommitted_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout);
        Ok(!status.trim().is_empty())
    } else {
        Err(anyhow::anyhow!("Failed to check git status"))
    }
}

/// Enhance commit message with additional context
pub fn enhance_commit_message(message: &str, branch: &str) -> String {
    let mut enhanced = message.to_string();

    // Add branch context for feature branches
    if branch.starts_with("feature/") || branch.starts_with("feat/") {
        if !enhanced.starts_with("feat") {
            enhanced = format!("feat: {}", enhanced.trim_start_matches("feat: "));
        }
    } else if (branch.starts_with("fix/") || branch.starts_with("bugfix/"))
        && !enhanced.starts_with("fix")
    {
        enhanced = format!("fix: {}", enhanced.trim_start_matches("fix: "));
    }

    enhanced
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_commit_message() {
        assert!(is_valid_commit_message("feat: add new feature"));
        assert!(is_valid_commit_message("fix(auth): resolve login issue"));
        assert!(is_valid_commit_message("docs: update README"));
        assert!(is_valid_commit_message("style: format code"));
        assert!(is_valid_commit_message(
            "refactor(utils): simplify helper functions"
        ));
        assert!(is_valid_commit_message("test: add unit tests"));
        assert!(is_valid_commit_message("chore: update dependencies"));
        assert!(is_valid_commit_message("perf: optimize database queries"));
        assert!(is_valid_commit_message("ci: update GitHub Actions"));
        assert!(is_valid_commit_message("build: configure webpack"));

        // Invalid messages
        assert!(!is_valid_commit_message("invalid message"));
        assert!(!is_valid_commit_message("feat"));
        assert!(!is_valid_commit_message("feat:"));
        assert!(!is_valid_commit_message("feature: add something")); // wrong type
        assert!(!is_valid_commit_message(&"feat: ".repeat(100))); // too long
    }

    #[test]
    fn test_parse_commit_message() {
        let commit = parse_commit_message("feat(auth): add JWT validation").unwrap();
        assert_eq!(commit.commit_type, crate::types::CommitType::Feat);
        assert_eq!(commit.scope, Some("auth".to_string()));
        assert_eq!(commit.description, "add JWT validation");
        assert!(!commit.breaking);

        let commit = parse_commit_message("fix!: resolve critical bug").unwrap();
        assert_eq!(commit.commit_type, crate::types::CommitType::Fix);
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "resolve critical bug");
        assert!(commit.breaking);

        let commit = parse_commit_message("docs: update README").unwrap();
        assert_eq!(commit.commit_type, crate::types::CommitType::Docs);
        assert_eq!(commit.scope, None);
        assert_eq!(commit.description, "update README");
        assert!(!commit.breaking);

        // Invalid message
        assert!(parse_commit_message("invalid message").is_err());
    }

    #[test]
    fn test_enhance_commit_message() {
        assert_eq!(
            enhance_commit_message("add new feature", "feature/user-auth"),
            "feat: add new feature"
        );

        assert_eq!(
            enhance_commit_message("resolve login issue", "fix/auth-bug"),
            "fix: resolve login issue"
        );

        assert_eq!(
            enhance_commit_message("feat: add new feature", "main"),
            "feat: add new feature"
        );
    }
}
