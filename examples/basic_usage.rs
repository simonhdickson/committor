//! Basic usage examples for the Committor library
//!
//! This example demonstrates how to use the Committor library with both OpenAI and Ollama providers.

use anyhow::Result;
use committor::{diff, Committor, Config};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Committor Basic Usage Examples");
    println!("=================================");

    // Example 1: Basic OpenAI usage
    println!("\n1. Basic OpenAI Usage");
    println!("---------------------");
    basic_openai_example().await?;

    // Example 2: Basic Ollama usage
    println!("\n2. Basic Ollama Usage");
    println!("---------------------");
    basic_ollama_example().await?;

    // Example 3: Custom configuration with OpenAI
    println!("\n3. Custom OpenAI Configuration");
    println!("------------------------------");
    custom_openai_config_example().await?;

    // Example 4: Custom configuration with Ollama
    println!("\n4. Custom Ollama Configuration");
    println!("------------------------------");
    custom_ollama_config_example().await?;

    // Example 5: Error handling
    println!("\n5. Error Handling Example");
    println!("-------------------------");
    error_handling_example().await?;

    println!("\n✅ All examples completed!");
    Ok(())
}

/// Example 1: Basic usage with OpenAI provider
async fn basic_openai_example() -> Result<()> {
    // Check if API key is available
    if env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️  OPENAI_API_KEY not set, skipping OpenAI examples");
        println!("   Set your API key: export OPENAI_API_KEY=\"your-key-here\"");
        return Ok(());
    }

    // Check if we have staged changes
    if !diff::has_staged_changes()? {
        println!("⚠️  No staged changes found for OpenAI example");
        println!("   Stage some changes first: git add <files>");
        return Ok(());
    }

    // Create default configuration (uses OpenAI by default)
    let config = Config::new()?;
    let committor = Committor::new(config)?;

    // Get the diff
    let diff = committor.get_staged_diff()?;
    println!("📝 Staged diff found ({} characters)", diff.len());

    // Generate commit messages
    println!("🤖 Generating commit messages with OpenAI...");
    match committor.generate_commit_messages(&diff).await {
        Ok(messages) => {
            println!("✅ Generated {} commit messages:", messages.len());
            for (i, message) in messages.iter().enumerate() {
                println!("   {}. {}", i + 1, message);
            }
        }
        Err(e) => {
            println!("❌ Error generating messages: {}", e);
        }
    }

    Ok(())
}

/// Example 2: Basic usage with Ollama provider
async fn basic_ollama_example() -> Result<()> {
    // Check if we have staged changes
    if !diff::has_staged_changes()? {
        println!("⚠️  No staged changes found for Ollama example");
        println!("   Stage some changes first: git add <files>");
        return Ok(());
    }

    // Create Ollama configuration
    let config = Config::with_ollama(
        "http://localhost:11434".to_string(),
        "llama2".to_string(),
        3,     // Generate 3 options
        false, // Don't auto-commit
        false, // Don't show diff
    );

    match Committor::new(config) {
        Ok(committor) => {
            // Get the diff
            let diff = committor.get_staged_diff()?;
            println!("📝 Staged diff found ({} characters)", diff.len());

            // Generate commit messages
            println!("🦙 Generating commit messages with Ollama...");
            match committor.generate_commit_messages(&diff).await {
                Ok(messages) => {
                    println!("✅ Generated {} commit messages:", messages.len());
                    for (i, message) in messages.iter().enumerate() {
                        println!("   {}. {}", i + 1, message);
                    }
                }
                Err(e) => {
                    println!("❌ Error generating messages: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Ollama not available: {}", e);
            println!("   Make sure Ollama is running: ollama serve");
            println!("   And you have models installed: ollama pull llama2");
        }
    }

    Ok(())
}

/// Example 3: Custom configuration with OpenAI
async fn custom_openai_config_example() -> Result<()> {
    // Check if API key is available
    if env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️  OPENAI_API_KEY not set, skipping custom OpenAI example");
        return Ok(());
    }

    // Check if we have staged changes
    if !diff::has_staged_changes()? {
        println!("⚠️  No staged changes found for custom OpenAI example");
        return Ok(());
    }

    let api_key = env::var("OPENAI_API_KEY")?;

    // Create custom configuration
    let config = Config::with_openai(
        api_key,
        "gpt-3.5-turbo".to_string(), // Use a different model
        5,                           // Generate 5 options
        false,                       // Don't auto-commit
        true,                        // Show diff
    );

    let committor = Committor::new(config)?;

    // Get the diff
    let diff = committor.get_staged_diff()?;
    println!("📝 Staged diff found ({} characters)", diff.len());
    println!("📄 Diff content preview:");
    println!("{}", diff.chars().take(200).collect::<String>());
    if diff.len() > 200 {
        println!("... (truncated)");
    }

    // Generate commit messages
    println!("🤖 Generating 5 commit messages with GPT-3.5-turbo...");
    match committor.generate_commit_messages(&diff).await {
        Ok(messages) => {
            println!("✅ Generated {} commit messages:", messages.len());
            for (i, message) in messages.iter().enumerate() {
                println!("   {}. {}", i + 1, message);
            }
        }
        Err(e) => {
            println!("❌ Error generating messages: {}", e);
        }
    }

    Ok(())
}

/// Example 4: Custom configuration with Ollama
async fn custom_ollama_config_example() -> Result<()> {
    // Check if we have staged changes
    if !diff::has_staged_changes()? {
        println!("⚠️  No staged changes found for custom Ollama example");
        return Ok(());
    }

    // Create custom Ollama configuration with timeout
    let config = Config::with_ollama_timeout(
        "http://localhost:11434".to_string(),
        "codellama".to_string(),
        std::time::Duration::from_secs(60), // 60 second timeout
        3,                                  // Generate 3 options
        false,                              // Don't auto-commit
        true,                               // Show diff
    );

    match Committor::new(config) {
        Ok(committor) => {
            // Get the diff
            let diff = committor.get_staged_diff()?;
            println!("📝 Staged diff found ({} characters)", diff.len());
            println!("📄 Diff content preview:");
            println!("{}", diff.chars().take(200).collect::<String>());
            if diff.len() > 200 {
                println!("... (truncated)");
            }

            // Generate commit messages
            println!("🦙 Generating commit messages with CodeLlama (60s timeout)...");
            match committor.generate_commit_messages(&diff).await {
                Ok(messages) => {
                    println!("✅ Generated {} commit messages:", messages.len());
                    for (i, message) in messages.iter().enumerate() {
                        println!("   {}. {}", i + 1, message);
                    }
                }
                Err(e) => {
                    println!("❌ Error generating messages: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Ollama not available: {}", e);
            println!("   Make sure Ollama is running: ollama serve");
            println!("   And you have CodeLlama installed: ollama pull codellama");
        }
    }

    Ok(())
}

/// Example 5: Error handling patterns
async fn error_handling_example() -> Result<()> {
    println!("📚 Demonstrating error handling patterns...");

    // Example: Invalid API key
    if env::var("OPENAI_API_KEY").is_ok() {
        let config = Config::with_openai(
            "invalid-api-key".to_string(),
            "gpt-4".to_string(),
            1,
            false,
            false,
        );

        match Committor::new(config) {
            Ok(committor) => {
                // This will succeed but the API call will fail
                let sample_diff = r#"
diff --git a/README.md b/README.md
index 1234567..abcdefg 100644
--- a/README.md
+++ b/README.md
@@ -1,3 +1,4 @@
 # My Project

 This is a sample project.
+Added a new line for testing.
"#;

                match committor.generate_commit_messages(sample_diff).await {
                    Ok(_) => println!("🤔 This shouldn't happen with invalid key"),
                    Err(e) => println!("✅ Correctly caught invalid API key error: {}", e),
                }
            }
            Err(e) => println!("✅ Correctly caught config error: {}", e),
        }
    }

    // Example: Ollama not available
    let config = Config::with_ollama(
        "http://localhost:99999".to_string(), // Invalid port
        "nonexistent-model".to_string(),
        1,
        false,
        false,
    );

    match Committor::new(config) {
        Ok(_) => println!("🤔 This shouldn't succeed with invalid URL"),
        Err(e) => println!("✅ Correctly caught Ollama connection error: {}", e),
    }

    // Example: No staged changes
    match diff::get_staged_diff() {
        Ok(diff) => {
            if diff.is_empty() {
                println!("✅ Correctly detected no staged changes");
            } else {
                println!("📝 Found staged changes ({} chars)", diff.len());
            }
        }
        Err(e) => println!("✅ Correctly caught git error: {}", e),
    }

    Ok(())
}

/// Example 6: Working with different diff scenarios
#[allow(dead_code)]
async fn diff_scenarios_example() -> Result<()> {
    println!("📋 Testing different diff scenarios...");

    // This would be used if we had a test repository setup
    let sample_scenarios = vec![
        ("feat: add user authentication", "Added login functionality"),
        (
            "fix: resolve memory leak",
            "Fixed buffer overflow in parser",
        ),
        ("docs: update README", "Updated installation instructions"),
        (
            "refactor: simplify error handling",
            "Consolidated error types",
        ),
    ];

    for (expected_type, description) in sample_scenarios {
        println!("🎯 Scenario: {} - {}", expected_type, description);
        // In a real scenario, we'd create the changes and test the AI generation
    }

    Ok(())
}
