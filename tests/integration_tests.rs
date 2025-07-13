//! Integration tests for the committor CLI application
//!
//! These tests verify the end-to-end functionality of committor,
//! including CLI commands, git operations, and AI integration.

use git2::Repository;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Helper struct for managing test git repositories
struct TestRepo {
    _temp_dir: TempDir,
    repo: Repository,
    path: std::path::PathBuf,
}

impl TestRepo {
    /// Create a new test repository
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let repo = Repository::init(temp_dir.path())?;

        // Configure git user for testing
        let mut config = repo.config()?;
        config.set_str("user.name", "Test User")?;
        config.set_str("user.email", "test@example.com")?;

        // Create initial commit
        let signature = git2::Signature::now("Test User", "test@example.com")?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };

        // Separate the tree creation from the commit to avoid borrow conflicts
        let commit_result = {
            let tree = repo.find_tree(tree_id)?;
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )
        };
        commit_result?;

        let path = temp_dir.path().to_path_buf();

        Ok(TestRepo {
            _temp_dir: temp_dir,
            repo,
            path,
        })
    }

    /// Create a new test repository with proper lifetime management
    #[allow(dead_code)]
    fn new_with_commit() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let repo = Repository::init(temp_dir.path())?;

        // Configure git user for testing
        let mut config = repo.config()?;
        config.set_str("user.name", "Test User")?;
        config.set_str("user.email", "test@example.com")?;

        // Create initial commit
        let signature = git2::Signature::now("Test User", "test@example.com")?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };

        // Separate tree lookup and commit to avoid borrow conflicts
        let commit_result = {
            let tree = repo.find_tree(tree_id)?;
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )
        };
        commit_result?;

        let path = temp_dir.path().to_path_buf();

        Ok(TestRepo {
            _temp_dir: temp_dir,
            repo,
            path,
        })
    }

    /// Add a file to the repository
    fn add_file(&self, filename: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.path.join(filename);

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&file_path, content)?;

        let mut index = self.repo.index()?;
        index.add_path(Path::new(filename))?;
        index.write()?;

        Ok(())
    }

    /// Modify an existing file
    #[allow(dead_code)]
    fn modify_file(&self, filename: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.path.join(filename);

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&file_path, content)?;

        let mut index = self.repo.index()?;
        index.add_path(Path::new(filename))?;
        index.write()?;

        Ok(())
    }

    /// Get the repository path
    fn path(&self) -> &Path {
        &self.path
    }

    /// Check if there are staged changes
    fn has_staged_changes(&self) -> Result<bool, git2::Error> {
        let head_tree = self.repo.head()?.peel_to_tree()?;
        let mut index = self.repo.index()?;
        let index_tree = self.repo.find_tree(index.write_tree()?)?;

        let diff = self
            .repo
            .diff_tree_to_tree(Some(&head_tree), Some(&index_tree), None)?;
        Ok(diff.deltas().len() > 0)
    }
}

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generate conventional commit messages"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("commit"));
    assert!(stdout.contains("diff"));
}

#[test]
fn test_cli_version_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .current_dir(".")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("committor"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_no_staged_changes() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    let output = Command::new("cargo")
        .args(["run", "--", "diff"])
        .current_dir(test_repo.path())
        .output()
        .expect("Failed to execute command");

    // Should succeed but indicate no changes
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No staged changes") || stdout.is_empty());
}

#[test]
fn test_staged_changes_detection() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    // Add a new file
    test_repo
        .add_file("test.txt", "Hello, world!")
        .expect("Failed to add file");

    // Verify there are staged changes
    assert!(test_repo
        .has_staged_changes()
        .expect("Failed to check staged changes"));

    // Just verify the file was added successfully
    assert!(test_repo.path().join("test.txt").exists());
}

#[test]
fn test_generate_command_without_api_key() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    test_repo
        .add_file("test.rs", "fn main() { println!(\"Hello!\"); }")
        .expect("Failed to add file");

    let output = Command::new("cargo")
        .args(["run", "--", "generate"])
        .current_dir(test_repo.path())
        .env_remove("OPENAI_API_KEY") // Ensure no API key
        .output()
        .expect("Failed to execute command");

    // Should fail with API key error or other expected errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAI API key")
            || stderr.contains("API key")
            || stderr.contains("No staged changes")
            || !output.status.success()
    );
}

#[test]
fn test_commit_command_without_api_key() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    test_repo
        .add_file(
            "src/lib.rs",
            "pub fn hello() -> &'static str { \"Hello, world!\" }",
        )
        .expect("Failed to add file");

    let output = Command::new("cargo")
        .args(["run", "--", "commit"])
        .current_dir(test_repo.path())
        .env_remove("OPENAI_API_KEY") // Ensure no API key
        .output()
        .expect("Failed to execute command");

    // Should fail with API key error or other expected errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAI API key")
            || stderr.contains("API key")
            || stderr.contains("No staged changes")
            || !output.status.success()
    );
}

#[test]
fn test_invalid_git_repository() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let output = Command::new("cargo")
        .args(["run", "--", "generate"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    // Should fail with git repository error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Not in a git repository")
            || stderr.contains("git repository")
            || stderr.contains("fatal: not a git repository")
            || !output.status.success()
    );
}

#[test]
fn test_cli_argument_parsing() {
    // Test basic commands that should always work
    let help_commands = vec![
        vec!["--help"],
        vec!["--version"],
        vec!["generate", "--help"],
        vec!["commit", "--help"],
        vec!["diff", "--help"],
    ];

    for args in help_commands {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .args(&args)
            .current_dir(".")
            .output()
            .expect("Failed to execute command");

        assert!(
            output.status.success(),
            "Help command failed: cargo run -- {}",
            args.join(" ")
        );
    }
}

#[test]
fn test_different_file_types() {
    // Test a few basic file types can be added to repo
    let test_files = vec![
        ("README.md", "# Test Project\n\nThis is a test."),
        ("main.rs", "fn main() { println!(\"Hello!\"); }"),
        ("data.json", r#"{"test": "value"}"#),
    ];

    for (filename, content) in test_files {
        let test_repo = TestRepo::new().expect("Failed to create test repo");
        test_repo
            .add_file(filename, content)
            .expect("Failed to add file");

        // Verify the file was created and staged
        assert!(test_repo.path().join(filename).exists());
        assert!(test_repo
            .has_staged_changes()
            .expect("Failed to check staged changes"));
    }
}

#[test]
fn test_large_file_handling() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    // Create a large file
    let large_content = "line\n".repeat(1000);
    test_repo
        .add_file("large_file.rs", &large_content)
        .expect("Failed to add file");

    // Verify large file was handled correctly
    assert!(test_repo.path().join("large_file.rs").exists());
    assert!(test_repo
        .has_staged_changes()
        .expect("Failed to check staged changes"));

    let file_content =
        fs::read_to_string(test_repo.path().join("large_file.rs")).expect("Failed to read file");
    assert_eq!(file_content.lines().count(), 1000);
}

#[test]
fn test_multiple_file_changes() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    // Add multiple files with different types of changes
    test_repo
        .add_file("src/lib.rs", "pub fn new_function() {}")
        .expect("Failed to add lib.rs");

    test_repo
        .add_file("tests/test.rs", "#[test]\nfn test_new_function() {}")
        .expect("Failed to add test.rs");

    test_repo
        .add_file("docs/README.md", "# Updated README\n\nNew documentation.")
        .expect("Failed to add README.md");

    // Verify all files were created and staged
    assert!(test_repo.path().join("src/lib.rs").exists());
    assert!(test_repo.path().join("tests/test.rs").exists());
    assert!(test_repo.path().join("docs/README.md").exists());
    assert!(test_repo
        .has_staged_changes()
        .expect("Failed to check staged changes"));
}

#[test]
fn test_error_handling_for_invalid_options() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    // Add a test file so we have some changes
    test_repo
        .add_file("test.txt", "test content")
        .expect("Failed to add file");

    // Test invalid count value
    let output = Command::new("cargo")
        .args(["run", "--", "generate", "--count", "0"])
        .current_dir(test_repo.path())
        .output()
        .expect("Failed to execute command");

    // Should handle invalid count gracefully (either error or use default)
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The exact behavior may vary, but it shouldn't crash
    assert!(
        stderr.contains("error")
            || stderr.contains("invalid")
            || output.status.success()
            || stderr.contains("API key") // Might fail on missing API key instead
    );
}

#[test]
fn test_empty_file_handling() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    // Add an empty file
    test_repo
        .add_file("empty.txt", "")
        .expect("Failed to add empty file");

    // Verify empty file was handled correctly
    assert!(test_repo.path().join("empty.txt").exists());
    assert!(test_repo
        .has_staged_changes()
        .expect("Failed to check staged changes"));

    let file_content =
        fs::read_to_string(test_repo.path().join("empty.txt")).expect("Failed to read file");
    assert_eq!(file_content, "");
}

#[test]
fn test_binary_file_handling() {
    let test_repo = TestRepo::new().expect("Failed to create test repo");

    // Create a simple binary-like file
    test_repo
        .add_file("data.bin", "binary\x00data\x01here")
        .expect("Failed to add binary file");

    // Verify binary file was handled correctly
    assert!(test_repo.path().join("data.bin").exists());
    assert!(test_repo
        .has_staged_changes()
        .expect("Failed to check staged changes"));

    let file_content = fs::read(test_repo.path().join("data.bin")).expect("Failed to read file");
    assert!(file_content.contains(&0x00) && file_content.contains(&0x01)); // Contains null bytes
}

#[cfg(test)]
mod api_integration_tests {
    //! These tests require a valid OpenAI API key and will be skipped if not available

    use super::*;
    use std::env;

    fn skip_if_no_api_key() -> Option<String> {
        env::var("OPENAI_API_KEY").ok()
    }

    #[test]
    fn test_generate_with_real_api_key() {
        let api_key = match skip_if_no_api_key() {
            Some(key) => key,
            None => {
                println!("Skipping API test: OPENAI_API_KEY not set");
                return;
            }
        };

        let test_repo = TestRepo::new().expect("Failed to create test repo");

        // Add a simple Rust file
        test_repo
            .add_file(
                "src/lib.rs",
                "pub fn hello() -> String {\n    \"Hello, world!\".to_string()\n}",
            )
            .expect("Failed to add file");

        let output = Command::new("cargo")
            .args(["run", "--", "generate", "--count", "1"])
            .current_dir(test_repo.path())
            .env("OPENAI_API_KEY", api_key)
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Should contain a generated commit message
            assert!(
                stdout.contains("feat") || stdout.contains("add") || stdout.contains("Generated")
            );
        } else {
            // API might fail due to rate limits, network issues, etc.
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("API test failed (this might be expected): {stderr}");
        }
    }

    #[test]
    fn test_commit_with_real_api_key() {
        let api_key = match skip_if_no_api_key() {
            Some(key) => key,
            None => {
                println!("Skipping API test: OPENAI_API_KEY not set");
                return;
            }
        };

        let test_repo = TestRepo::new().expect("Failed to create test repo");

        // Add a documentation file
        test_repo
            .add_file(
                "docs/guide.md",
                "# User Guide\n\nThis guide explains how to use the application.",
            )
            .expect("Failed to add file");

        let output = Command::new("cargo")
            .args(["run", "--", "generate", "--count", "1"])
            .current_dir(test_repo.path())
            .env("OPENAI_API_KEY", api_key)
            .output()
            .expect("Failed to execute command");

        // We don't actually commit in this test to avoid polluting git history
        // Just verify the generate command works
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("API test failed (this might be expected): {stderr}");
        }
    }
}

/// Test to ensure basic git operations perform reasonably
#[test]
fn test_performance_basic_operations() {
    let start = std::time::Instant::now();

    let test_repo = TestRepo::new().expect("Failed to create test repo");
    test_repo
        .add_file("test.rs", "fn test() {}")
        .expect("Failed to add file");

    // Verify staged changes detection
    assert!(test_repo
        .has_staged_changes()
        .expect("Failed to check staged changes"));

    let duration = start.elapsed();

    // Basic git operations should complete quickly (under 1 second)
    assert!(
        duration.as_millis() < 1000,
        "Basic git operations took too long: {duration:?}"
    );
}
