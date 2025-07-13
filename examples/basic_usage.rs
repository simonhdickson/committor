//! Basic usage examples for the commitor library
//!
//! This file demonstrates how to use the commitor library programmatically
//! to generate conventional commit messages.

use anyhow::Result;
use commitor::{commit, diff, prompt, Commitor, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Commitor Basic Usage Examples\n");

    // Example 1: Basic usage with environment API key
    example_basic_usage().await?;

    // Example 2: Custom configuration
    example_custom_config().await?;

    // Example 3: Analyzing git diff manually
    example_manual_diff_analysis().await?;

    // Example 4: Validating commit messages
    example_commit_validation()?;

    // Example 5: Getting repository context
    example_repository_context()?;

    Ok(())
}

/// Example 1: Basic usage with default configuration
async fn example_basic_usage() -> Result<()> {
    println!("📋 Example 1: Basic Usage");
    println!("{}", "─".repeat(40));

    // Check if we have staged changes
    if !diff::has_staged_changes()? {
        println!("⚠️  No staged changes found. Please stage some changes first with 'git add'");
        return Ok(());
    }

    // Create commitor with default config (requires OPENAI_API_KEY env var)
    match Config::new() {
        Ok(config) => {
            let commitor = Commitor::new(config);

            // Get the staged diff
            let diff = commitor.get_staged_diff()?;
            println!("📄 Current staged diff preview:");
            println!("{}", &diff[..diff.len().min(200)]);
            if diff.len() > 200 {
                println!("... (truncated)");
            }

            // Generate commit messages
            println!("\n🤖 Generating commit messages...");
            match commitor.generate_commit_messages(&diff).await {
                Ok(messages) => {
                    println!("✅ Generated {} commit messages:", messages.len());
                    for (i, message) in messages.iter().enumerate() {
                        println!("  {}. {}", i + 1, message);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to generate commit messages: {}", e);
                }
            }
        }
        Err(_) => {
            println!("⚠️  OPENAI_API_KEY environment variable not set. Skipping this example.");
        }
    }

    println!();
    Ok(())
}

/// Example 2: Custom configuration
async fn example_custom_config() -> Result<()> {
    println!("⚙️  Example 2: Custom Configuration");
    println!("{}", "─".repeat(40));

    // Check for API key
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️  OPENAI_API_KEY environment variable not set. Skipping this example.");
        println!();
        return Ok(());
    }

    let api_key = std::env::var("OPENAI_API_KEY")?;

    // Create custom configuration
    let config = Config::with_options(
        api_key,
        "gpt-3.5-turbo".to_string(), // Use a different model
        5,                           // Generate 5 options
        false,                       // Don't auto-commit
        true,                        // Show diff
    );

    let commitor = Commitor::new(config);

    // Check if we have staged changes
    if !diff::has_staged_changes()? {
        println!("⚠️  No staged changes found for custom config example");
        println!();
        return Ok(());
    }

    let diff = commitor.get_staged_diff()?;
    println!("📊 Using custom config: gpt-3.5-turbo, 5 options");

    match commitor.generate_commit_messages(&diff).await {
        Ok(messages) => {
            println!("✅ Generated {} commit message options:", messages.len());
            for (i, message) in messages.iter().enumerate() {
                println!("  {}. {}", i + 1, message);
            }
        }
        Err(e) => {
            println!("❌ Failed to generate commit messages: {}", e);
        }
    }

    println!();
    Ok(())
}

/// Example 3: Manual diff analysis
async fn example_manual_diff_analysis() -> Result<()> {
    println!("🔍 Example 3: Manual Diff Analysis");
    println!("{}", "─".repeat(40));

    // Create a sample diff for demonstration
    let sample_diff = r#"
diff --git a/src/auth.rs b/src/auth.rs
index 1234567..abcdefg 100644
--- a/src/auth.rs
+++ b/src/auth.rs
@@ -1,5 +1,8 @@
 use jwt::TokenData;

+/// Validates JWT tokens
+pub fn validate_jwt_token(token: &str) -> Result<TokenData, Error> {
+    // Implementation here
+}
+
 pub struct AuthService {
     secret: String,
 }
"#;

    // Get structured information about changes
    let changes = diff::get_staged_changes().unwrap_or_default();
    if !changes.is_empty() {
        println!("📁 Current staged changes:");
        for change in &changes {
            println!(
                "  {} {} (+{}, -{})",
                change.change_type, change.file_path, change.additions, change.deletions
            );
        }
    } else {
        println!("📁 No real staged changes, using sample diff for analysis");
    }

    // Analyze the diff type and suggest commit types
    let suggested_types = prompt::suggest_commit_type(&changes);
    if !suggested_types.is_empty() {
        println!("💡 Suggested commit types based on file changes:");
        for commit_type in suggested_types {
            println!("  - {}: {}", commit_type, commit_type.description());
        }
    }

    // Get repository context
    let mut context = prompt::RepositoryContext::new();
    context.language = prompt::RepositoryContext::detect_language(&changes);
    context.project_type = prompt::RepositoryContext::detect_project_type(&changes);

    println!("🏗️  Repository context:");
    println!("  Language: {}", context.language);
    println!("  Project Type: {}", context.project_type);

    println!();
    Ok(())
}

/// Example 4: Commit message validation
fn example_commit_validation() -> Result<()> {
    println!("✅ Example 4: Commit Message Validation");
    println!("{}", "─".repeat(40));

    let test_messages = vec![
        "feat(auth): add JWT token validation",
        "fix: resolve login issue",
        "docs: update README",
        "invalid commit message",
        "feat: this is a very long commit message that exceeds the recommended length limit",
        "refactor(utils): simplify helper functions",
        "test: add unit tests for auth module",
    ];

    for message in test_messages {
        let is_valid = commit::is_valid_commit_message(message);
        let status = if is_valid { "✅" } else { "❌" };
        println!("{} {}", status, message);

        // Try to parse valid messages
        if is_valid {
            match commit::parse_commit_message(message) {
                Ok(parsed) => {
                    println!(
                        "  📋 Type: {}, Scope: {:?}, Breaking: {}",
                        parsed.commit_type, parsed.scope, parsed.breaking
                    );
                }
                Err(_) => {
                    println!("  ⚠️  Failed to parse (unexpected)");
                }
            }
        }
    }

    println!();
    Ok(())
}

/// Example 5: Repository context detection
fn example_repository_context() -> Result<()> {
    println!("📋 Example 5: Repository Context");
    println!("{}", "─".repeat(40));

    // Check git environment
    match commit::validate_git_environment() {
        Ok(_) => println!("✅ Git environment is valid"),
        Err(e) => println!("❌ Git environment issue: {}", e),
    }

    // Get current branch
    match commit::get_current_branch() {
        Ok(branch) => println!("🌿 Current branch: {}", branch),
        Err(e) => println!("⚠️  Could not get branch: {}", e),
    }

    // Check for uncommitted changes
    match commit::has_uncommitted_changes() {
        Ok(has_changes) => {
            if has_changes {
                println!("📝 There are uncommitted changes");
            } else {
                println!("✨ Working directory is clean");
            }
        }
        Err(e) => println!("⚠️  Could not check status: {}", e),
    }

    // Get last commit message
    match commit::get_last_commit_message() {
        Ok(message) => println!("💬 Last commit: {}", message),
        Err(e) => println!("⚠️  Could not get last commit: {}", e),
    }

    // Get diff summary
    match diff::get_diff_summary() {
        Ok(summary) => println!("📊 Diff summary:\n{}", summary),
        Err(e) => println!("⚠️  Could not get diff summary: {}", e),
    }

    println!();
    Ok(())
}

/// Example helper function to demonstrate library usage in other contexts
pub async fn generate_commit_for_diff(diff_content: &str, api_key: &str) -> Result<String> {
    let config = Config::with_options(
        api_key.to_string(),
        "gpt-4".to_string(),
        1, // Just one message
        false,
        false,
    );

    let commitor = Commitor::new(config);
    let messages = commitor.generate_commit_messages(diff_content).await?;

    messages
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No commit message generated"))
}

/// Example of batch processing multiple diffs
pub async fn batch_process_commits(diffs: Vec<String>, api_key: &str) -> Result<Vec<String>> {
    let config = Config::with_options(api_key.to_string(), "gpt-4".to_string(), 1, false, false);

    let commitor = Commitor::new(config);
    let mut results = Vec::new();

    for diff in diffs {
        match commitor.generate_commit_messages(&diff).await {
            Ok(messages) => {
                if let Some(message) = messages.into_iter().next() {
                    results.push(message);
                }
            }
            Err(e) => {
                eprintln!("Failed to process diff: {}", e);
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_commit_for_diff() {
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let diff = "diff --git a/test.txt b/test.txt\n+Hello, world!";
            let result = generate_commit_for_diff(diff, &api_key).await;
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_commit_validation_examples() {
        assert!(commit::is_valid_commit_message("feat: add new feature"));
        assert!(commit::is_valid_commit_message(
            "fix(auth): resolve login bug"
        ));
        assert!(!commit::is_valid_commit_message("invalid message"));
        assert!(!commit::is_valid_commit_message("feat: ".repeat(100)));
    }
}
