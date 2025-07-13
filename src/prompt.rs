//! Prompt generation for AI-powered commit message creation

use crate::types::{CommitType, DiffChange};

/// Create a detailed prompt for generating conventional commit messages
pub fn create_commit_prompt(diff: &str) -> String {
    let sanitized_diff = sanitize_diff_for_prompt(diff);

    format!(
        r#"You are an expert software engineer who writes clear, concise conventional commit messages.

Based on the following git diff, generate a single conventional commit message that follows these rules:

## Format
<type>(<scope>): <description>

## Types (choose the most appropriate):
- feat: A new feature for the user
- fix: A bug fix
- docs: Documentation only changes
- style: Changes that don't affect code meaning (formatting, missing semi-colons, etc.)
- refactor: Code change that neither fixes a bug nor adds a feature
- test: Adding missing tests or correcting existing tests
- chore: Changes to build process, auxiliary tools, libraries, etc.
- perf: Code change that improves performance
- ci: Changes to CI configuration files and scripts
- build: Changes that affect the build system or external dependencies

## Guidelines:
1. Keep the description under 50 characters
2. Use imperative mood ("add" not "added" or "adds")
3. No period at the end
4. Make scope optional but useful (component, module, file area)
5. Focus on WHAT changed, not HOW it was implemented
6. If multiple changes, choose the most significant one

## Examples:
- feat(auth): add JWT token validation
- fix(database): resolve connection timeout
- docs(readme): update installation guide
- refactor(utils): simplify error handling
- test(api): add user endpoint tests
- chore(deps): update React to v18
- perf(queries): optimize database indexes
- ci(github): add automated testing
- build(webpack): configure production build

## Git Diff:
```
{sanitized_diff}
```

Generate ONE conventional commit message (only the message, no explanation):"#
    )
}

/// Create a prompt for generating multiple commit message options
pub fn create_multiple_commit_prompt(diff: &str, count: u8) -> String {
    let sanitized_diff = sanitize_diff_for_prompt(diff);

    format!(
        r#"You are an expert software engineer who writes clear, concise conventional commit messages.

Based on the following git diff, generate {count} different conventional commit message options that follow these rules:

## Format
<type>(<scope>): <description>

## Types:
- feat: A new feature
- fix: A bug fix
- docs: Documentation changes
- style: Formatting changes
- refactor: Code restructuring
- test: Test additions/changes
- chore: Maintenance tasks
- perf: Performance improvements
- ci: CI/CD changes
- build: Build system changes

## Guidelines:
1. Each message under 50 characters
2. Use imperative mood
3. No period at the end
4. Optional but useful scope
5. Focus on WHAT changed
6. Provide variety in scope and perspective

## Git Diff:
```
{sanitized_diff}
```

Generate {count} different conventional commit messages (one per line, no numbering or explanation):"#
    )
}

/// Create a prompt for analyzing commit message quality
pub fn create_analysis_prompt(message: &str) -> String {
    format!(
        r#"You are an expert in conventional commit standards. Analyze this commit message:

"{message}"

Provide feedback on:
1. Conventional commit format compliance
2. Clarity and conciseness
3. Appropriate type and scope
4. Imperative mood usage
5. Length (should be under 50 characters)

Rate from 1-10 and suggest improvements if needed.

Response format:
Score: X/10
Issues: [list any issues]
Suggestions: [list improvements]"#
    )
}

/// Create a prompt with context about the repository
pub fn create_contextual_commit_prompt(diff: &str, context: &RepositoryContext) -> String {
    let sanitized_diff = sanitize_diff_for_prompt(diff);

    format!(
        r#"You are an expert software engineer writing a conventional commit message.

## Repository Context:
- Language: {}
- Project Type: {}
- Branch: {}
- Files Changed: {}

## Recent Commits:
{}

## Current Changes:
```
{}
```

Based on this context and the git diff, generate a conventional commit message that:
1. Follows the format: <type>(<scope>): <description>
2. Is contextually appropriate for this project
3. Maintains consistency with recent commit style
4. Uses the most appropriate type and scope
5. Keeps description under 50 characters
6. Uses imperative mood

Generate ONE conventional commit message:"#,
        context.language,
        context.project_type,
        context.branch,
        context.files_changed,
        context.recent_commits.join("\n"),
        sanitized_diff
    )
}

/// Create a prompt for fixing an invalid commit message
pub fn create_fix_commit_prompt(invalid_message: &str, issues: &[String]) -> String {
    format!(
        r#"You are an expert in conventional commit standards. Fix this commit message:

Original message: "{}"

Issues found:
{}

Requirements:
1. Use format: <type>(<scope>): <description>
2. Valid types: feat, fix, docs, style, refactor, test, chore, perf, ci, build
3. Description under 50 characters
4. Imperative mood
5. No period at the end
6. Meaningful scope (optional but recommended)

Generate the corrected conventional commit message:"#,
        invalid_message,
        issues
            .iter()
            .enumerate()
            .map(|(i, issue)| format!("{}. {}", i + 1, issue))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Sanitize diff content for use in prompts
fn sanitize_diff_for_prompt(diff: &str) -> String {
    let lines: Vec<&str> = diff.lines().collect();
    let mut sanitized = String::new();
    let mut line_count = 0;
    const MAX_LINES: usize = 100;
    const MAX_LINE_LENGTH: usize = 150;

    for line in lines {
        if line_count >= MAX_LINES {
            sanitized.push_str("... (diff truncated for brevity)\n");
            break;
        }

        // Skip lines that might contain sensitive information
        if contains_sensitive_info(line) {
            sanitized.push_str("... (line with sensitive info removed)\n");
            continue;
        }

        // Truncate very long lines
        if line.len() > MAX_LINE_LENGTH {
            sanitized.push_str(&line[..MAX_LINE_LENGTH]);
            sanitized.push_str("... (line truncated)\n");
        } else {
            sanitized.push_str(line);
            sanitized.push('\n');
        }

        line_count += 1;
    }

    sanitized
}

/// Check if a line contains potentially sensitive information
fn contains_sensitive_info(line: &str) -> bool {
    let line_lower = line.to_lowercase();

    // Common patterns for sensitive information
    let sensitive_patterns = [
        "password",
        "secret",
        "token",
        "api_key",
        "private_key",
        "auth_token",
        "access_token",
        "client_secret",
        "client_id",
        "database_url",
        "connection_string",
        "credentials",
        "ssh_key",
        "private_key",
        "public_key",
        "cert",
        "certificate",
        "bearer",
        "authorization",
        "x-api-key",
        "x-auth-token",
    ];

    sensitive_patterns
        .iter()
        .any(|pattern| line_lower.contains(pattern))
}

/// Get commit type suggestions based on file changes
pub fn suggest_commit_type(changes: &[DiffChange]) -> Vec<CommitType> {
    let mut suggestions = Vec::new();

    // Analyze file patterns to suggest appropriate types
    let has_test_files = changes.iter().any(|c| {
        c.file_path.contains("test")
            || c.file_path.contains("spec")
            || c.file_path.ends_with("_test.rs")
            || c.file_path.ends_with(".test.js")
            || c.file_path.ends_with(".spec.js")
    });

    let has_doc_files = changes.iter().any(|c| {
        c.file_path.contains("README")
            || c.file_path.contains("CHANGELOG")
            || c.file_path.ends_with(".md")
            || c.file_path.contains("docs/")
            || c.file_path.contains("documentation")
    });

    let has_config_files = changes.iter().any(|c| {
        c.file_path.contains("Cargo.toml")
            || c.file_path.contains("package.json")
            || c.file_path.contains("Dockerfile")
            || c.file_path.contains("docker-compose")
            || c.file_path.contains(".yml")
            || c.file_path.contains(".yaml")
            || c.file_path.contains("Makefile")
    });

    let has_ci_files = changes.iter().any(|c| {
        c.file_path.contains(".github/")
            || c.file_path.contains(".gitlab-ci")
            || c.file_path.contains("ci/")
            || c.file_path.contains("scripts/")
    });

    // Suggest types based on file patterns
    if has_test_files {
        suggestions.push(CommitType::Test);
    }
    if has_doc_files {
        suggestions.push(CommitType::Docs);
    }
    if has_config_files {
        suggestions.push(CommitType::Build);
        suggestions.push(CommitType::Chore);
    }
    if has_ci_files {
        suggestions.push(CommitType::Ci);
    }

    // Add common types if no specific patterns found
    if suggestions.is_empty() {
        suggestions.extend([CommitType::Feat, CommitType::Fix, CommitType::Refactor]);
    }

    suggestions
}

/// Repository context for better commit message generation
#[derive(Debug, Clone)]
pub struct RepositoryContext {
    pub language: String,
    pub project_type: String,
    pub branch: String,
    pub files_changed: String,
    pub recent_commits: Vec<String>,
}

impl RepositoryContext {
    /// Create a new repository context
    pub fn new() -> Self {
        Self {
            language: "Unknown".to_string(),
            project_type: "Unknown".to_string(),
            branch: "main".to_string(),
            files_changed: "0".to_string(),
            recent_commits: Vec::new(),
        }
    }

    /// Detect primary language from file extensions
    pub fn detect_language(changes: &[DiffChange]) -> String {
        let mut language_counts = std::collections::HashMap::new();

        for change in changes {
            if let Some(ext) = std::path::Path::new(&change.file_path).extension() {
                let lang = match ext.to_str() {
                    Some("rs") => "Rust",
                    Some("js") | Some("ts") => "JavaScript/TypeScript",
                    Some("py") => "Python",
                    Some("java") => "Java",
                    Some("cpp") | Some("cc") | Some("cxx") => "C++",
                    Some("c") | Some("h") => "C",
                    Some("go") => "Go",
                    Some("rb") => "Ruby",
                    Some("php") => "PHP",
                    Some("cs") => "C#",
                    Some("kt") => "Kotlin",
                    Some("swift") => "Swift",
                    Some("dart") => "Dart",
                    Some("scala") => "Scala",
                    Some("clj") => "Clojure",
                    Some("hs") => "Haskell",
                    Some("elm") => "Elm",
                    Some("ex") => "Elixir",
                    Some("erl") => "Erlang",
                    Some("nim") => "Nim",
                    Some("zig") => "Zig",
                    _ => "Other",
                };

                *language_counts.entry(lang).or_insert(0) += 1;
            }
        }

        language_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang.to_string())
            .unwrap_or_else(|| "Mixed".to_string())
    }

    /// Detect project type from file patterns
    pub fn detect_project_type(changes: &[DiffChange]) -> String {
        let files: Vec<&str> = changes.iter().map(|c| c.file_path.as_str()).collect();

        if files.iter().any(|f| f.contains("Cargo.toml")) {
            "Rust Project"
        } else if files.iter().any(|f| f.contains("package.json")) {
            "Node.js Project"
        } else if files
            .iter()
            .any(|f| f.contains("requirements.txt") || f.contains("setup.py"))
        {
            "Python Project"
        } else if files
            .iter()
            .any(|f| f.contains("pom.xml") || f.contains("build.gradle"))
        {
            "Java Project"
        } else if files.iter().any(|f| f.contains("go.mod")) {
            "Go Project"
        } else if files.iter().any(|f| f.contains("Gemfile")) {
            "Ruby Project"
        } else if files.iter().any(|f| f.contains("composer.json")) {
            "PHP Project"
        } else if files.iter().any(|f| f.contains("pubspec.yaml")) {
            "Dart/Flutter Project"
        } else if files.iter().any(|f| f.contains("Package.swift")) {
            "Swift Project"
        } else {
            "Generic Project"
        }
        .to_string()
    }
}

impl Default for RepositoryContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DiffChangeType;

    #[test]
    fn test_sanitize_diff_for_prompt() {
        let diff = "normal line\npassword=secret123\napi_key=sk-1234567890\nanother line";
        let sanitized = sanitize_diff_for_prompt(diff);

        assert!(!sanitized.contains("secret123"));
        assert!(!sanitized.contains("sk-1234567890"));
        assert!(sanitized.contains("normal line"));
        assert!(sanitized.contains("another line"));
    }

    #[test]
    fn test_contains_sensitive_info() {
        assert!(contains_sensitive_info("password=secret123"));
        assert!(contains_sensitive_info("API_KEY=sk-1234567890"));
        assert!(contains_sensitive_info(
            "database_url=postgres://user:pass@host"
        ));
        assert!(contains_sensitive_info("Authorization: Bearer token123"));

        assert!(!contains_sensitive_info("normal code line"));
        assert!(!contains_sensitive_info("function test() {}"));
    }

    #[test]
    fn test_suggest_commit_type() {
        let test_changes = vec![DiffChange {
            file_path: "src/lib_test.rs".to_string(),
            change_type: DiffChangeType::Modified,
            additions: 5,
            deletions: 2,
        }];

        let suggestions = suggest_commit_type(&test_changes);
        assert!(suggestions.contains(&CommitType::Test));

        let doc_changes = vec![DiffChange {
            file_path: "README.md".to_string(),
            change_type: DiffChangeType::Modified,
            additions: 10,
            deletions: 3,
        }];

        let suggestions = suggest_commit_type(&doc_changes);
        assert!(suggestions.contains(&CommitType::Docs));
    }

    #[test]
    fn test_detect_language() {
        let changes = vec![
            DiffChange {
                file_path: "src/main.rs".to_string(),
                change_type: DiffChangeType::Modified,
                additions: 10,
                deletions: 5,
            },
            DiffChange {
                file_path: "src/lib.rs".to_string(),
                change_type: DiffChangeType::Added,
                additions: 20,
                deletions: 0,
            },
        ];

        let language = RepositoryContext::detect_language(&changes);
        assert_eq!(language, "Rust");
    }

    #[test]
    fn test_detect_project_type() {
        let rust_changes = vec![DiffChange {
            file_path: "Cargo.toml".to_string(),
            change_type: DiffChangeType::Modified,
            additions: 2,
            deletions: 1,
        }];

        let project_type = RepositoryContext::detect_project_type(&rust_changes);
        assert_eq!(project_type, "Rust Project");

        let node_changes = vec![DiffChange {
            file_path: "package.json".to_string(),
            change_type: DiffChangeType::Modified,
            additions: 3,
            deletions: 0,
        }];

        let project_type = RepositoryContext::detect_project_type(&node_changes);
        assert_eq!(project_type, "Node.js Project");
    }

    #[test]
    fn test_create_commit_prompt() {
        let diff = "diff --git a/src/main.rs b/src/main.rs\n+fn new_function() {}";
        let prompt = create_commit_prompt(diff);

        assert!(prompt.contains("conventional commit"));
        assert!(prompt.contains("feat"));
        assert!(prompt.contains("fix"));
        assert!(prompt.contains("docs"));
        assert!(prompt.contains(diff));
    }

    #[test]
    fn test_create_analysis_prompt() {
        let message = "feat(auth): add JWT validation";
        let prompt = create_analysis_prompt(message);

        assert!(prompt.contains("conventional commit"));
        assert!(prompt.contains("Score:"));
        assert!(prompt.contains("Issues:"));
        assert!(prompt.contains("Suggestions:"));
        assert!(prompt.contains(message));
    }
}
