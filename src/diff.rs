//! Git diff operations for analyzing staged changes

use crate::types::{DiffChange, DiffChangeType};
use anyhow::{Context, Result};
use git2::{Delta, Repository};
use std::path::Path;

/// Get the staged diff from the current git repository
pub fn get_staged_diff() -> Result<String> {
    let repo = Repository::open(".").context("Not in a git repository")?;
    get_staged_diff_from_repo(&repo)
}

/// Get the staged diff from a specific git repository
pub fn get_staged_diff_from_repo(repo: &Repository) -> Result<String> {
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.include_untracked(false);
    diff_opts.context_lines(3);

    let head_tree = repo.head()?.peel_to_tree()?;
    let mut index = repo.index()?;
    let _index_tree = repo.find_tree(index.write_tree()?)?;

    let diff = repo.diff_tree_to_index(Some(&head_tree), Some(&index), Some(&mut diff_opts))?;

    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
        true
    })?;

    Ok(diff_text)
}

/// Get structured information about staged changes
pub fn get_staged_changes() -> Result<Vec<DiffChange>> {
    let repo = Repository::open(".").context("Not in a git repository")?;
    get_staged_changes_from_repo(&repo)
}

/// Get structured information about staged changes from a specific repository
pub fn get_staged_changes_from_repo(repo: &Repository) -> Result<Vec<DiffChange>> {
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.include_untracked(false);

    let head_tree = repo.head()?.peel_to_tree()?;
    let mut index = repo.index()?;
    let _index_tree = repo.find_tree(index.write_tree()?)?;

    let diff = repo.diff_tree_to_index(Some(&head_tree), Some(&index), Some(&mut diff_opts))?;

    let mut changes = Vec::new();

    diff.foreach(
        &mut |delta, _progress| {
            let file_path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .unwrap_or_else(|| Path::new("unknown"))
                .to_string_lossy()
                .to_string();

            let change_type = match delta.status() {
                Delta::Added => DiffChangeType::Added,
                Delta::Deleted => DiffChangeType::Deleted,
                Delta::Modified => DiffChangeType::Modified,
                Delta::Renamed => DiffChangeType::Renamed,
                Delta::Copied => DiffChangeType::Copied,
                _ => DiffChangeType::Modified,
            };

            changes.push(DiffChange {
                file_path,
                change_type,
                additions: 0, // Will be filled in the hunk callback
                deletions: 0, // Will be filled in the hunk callback
            });

            true
        },
        None,
        None,
        None,
    )?;

    // Get line statistics
    let mut file_stats = std::collections::HashMap::new();

    // First pass: initialize file stats
    for change in &changes {
        file_stats.insert(change.file_path.clone(), (0usize, 0usize));
    }

    // Second pass: count additions and deletions
    diff.foreach(
        &mut |_delta, _progress| true,
        None,
        None,
        Some(&mut |delta, _hunk, line| {
            let file_path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .unwrap_or_else(|| Path::new("unknown"))
                .to_string_lossy()
                .to_string();

            if let Some((additions, deletions)) = file_stats.get_mut(&file_path) {
                match line.origin() {
                    '+' => *additions += 1,
                    '-' => *deletions += 1,
                    _ => {}
                }
            }
            true
        }),
    )?;

    // Update changes with line statistics
    for change in &mut changes {
        if let Some((additions, deletions)) = file_stats.get(&change.file_path) {
            change.additions = *additions;
            change.deletions = *deletions;
        }
    }

    Ok(changes)
}

/// Check if there are any staged changes
pub fn has_staged_changes() -> Result<bool> {
    let repo = Repository::open(".").context("Not in a git repository")?;
    has_staged_changes_from_repo(&repo)
}

/// Check if there are any staged changes in a specific repository
pub fn has_staged_changes_from_repo(repo: &Repository) -> Result<bool> {
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.include_untracked(false);

    let head_tree = repo.head()?.peel_to_tree()?;
    let mut index = repo.index()?;
    let _index_tree = repo.find_tree(index.write_tree()?)?;

    let diff = repo.diff_tree_to_index(Some(&head_tree), Some(&index), Some(&mut diff_opts))?;

    Ok(diff.deltas().len() > 0)
}

/// Get a summary of the staged changes
pub fn get_diff_summary() -> Result<String> {
    let changes = get_staged_changes()?;

    if changes.is_empty() {
        return Ok("No staged changes found.".to_string());
    }

    let mut summary = String::new();
    summary.push_str(&format!("Staged changes ({} files):\n", changes.len()));

    for change in &changes {
        let stats = if change.additions > 0 || change.deletions > 0 {
            format!(" (+{}, -{})", change.additions, change.deletions)
        } else {
            String::new()
        };

        summary.push_str(&format!(
            "  {} {}{}\n",
            change.change_type, change.file_path, stats
        ));
    }

    Ok(summary)
}

/// Filter diff text to remove sensitive information
pub fn sanitize_diff(diff: &str) -> String {
    let lines: Vec<&str> = diff.lines().collect();
    let mut sanitized = String::new();

    for line in lines {
        // Skip lines that might contain sensitive information
        if line.contains("password")
            || line.contains("secret")
            || line.contains("token")
            || line.contains("api_key")
            || line.contains("private_key")
        {
            continue;
        }

        // Limit line length to prevent extremely long lines
        if line.len() > 200 {
            sanitized.push_str(&line[..200]);
            sanitized.push_str("... (truncated)\n");
        } else {
            sanitized.push_str(line);
            sanitized.push('\n');
        }
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> Result<(TempDir, Repository)> {
        let temp_dir = TempDir::new()?;
        let repo = Repository::init(temp_dir.path())?;

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

        Ok((temp_dir, repo))
    }

    #[test]
    fn test_no_staged_changes() -> Result<()> {
        let (_temp_dir, repo) = create_test_repo()?;

        let has_changes = has_staged_changes_from_repo(&repo)?;
        assert!(!has_changes);

        let diff = get_staged_diff_from_repo(&repo)?;
        assert!(diff.is_empty());

        Ok(())
    }

    #[test]
    fn test_staged_changes() -> Result<()> {
        let (temp_dir, repo) = create_test_repo()?;

        // Create a new file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!")?;

        // Stage the file
        let mut index = repo.index()?;
        index.add_path(std::path::Path::new("test.txt"))?;
        index.write()?;

        let has_changes = has_staged_changes_from_repo(&repo)?;
        assert!(has_changes);

        let diff = get_staged_diff_from_repo(&repo)?;
        assert!(!diff.is_empty());
        assert!(diff.contains("Hello, world!"));

        Ok(())
    }

    #[test]
    fn test_get_staged_changes() -> Result<()> {
        let (temp_dir, repo) = create_test_repo()?;

        // Create a new file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!")?;

        // Stage the file
        let mut index = repo.index()?;
        index.add_path(std::path::Path::new("test.txt"))?;
        index.write()?;

        let changes = get_staged_changes_from_repo(&repo)?;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].file_path, "test.txt");
        assert_eq!(changes[0].change_type, DiffChangeType::Added);

        Ok(())
    }

    #[test]
    fn test_sanitize_diff() {
        let diff = r#"
@@ -1,3 +1,4 @@
 normal line
-password=secret123
+password=newsecret456
+api_key=sk-1234567890abcdef
 another normal line
"#;

        let sanitized = sanitize_diff(diff);
        assert!(!sanitized.contains("secret123"));
        assert!(!sanitized.contains("newsecret456"));
        assert!(!sanitized.contains("sk-1234567890abcdef"));
        assert!(sanitized.contains("normal line"));
        assert!(sanitized.contains("another normal line"));
    }
}
