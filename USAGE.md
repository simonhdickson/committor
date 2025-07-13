# Committor Usage Guide

This guide shows you how to use Committor to automatically generate conventional commit messages.

## Quick Start

### 1. Installation

First, build the project:

```bash
cd committor
cargo build --release
```

### 2. Set up your OpenAI API Key

You need an OpenAI API key to use Committor. Get one from [OpenAI's platform](https://platform.openai.com/api-keys).

Set it as an environment variable:

```bash
export OPENAI_API_KEY="your-api-key-here"
```

Or add it to your shell profile (`.bashrc`, `.zshrc`, etc.):

```bash
echo 'export OPENAI_API_KEY="your-api-key-here"' >> ~/.bashrc
source ~/.bashrc
```

### 3. Basic Usage

In any git repository with staged changes:

```bash
# Stage some changes
git add .

# Generate commit message options
./target/release/committor generate

# Or generate and commit in one step
./target/release/committor commit
```

## Commands

### `generate`
Generate commit message options for staged changes.

```bash
# Basic usage
committor generate

# Generate 5 options
committor generate --count 5

# Show the diff before generating
committor generate --show-diff

# Use a different model
committor generate --model gpt-3.5-turbo

# Auto-commit with first suggestion
committor generate --auto-commit
```

### `commit`
Generate commit messages and optionally commit.

```bash
# Generate options and choose one to commit
committor commit

# Auto-commit with first suggestion
committor commit --auto-commit
```

### `diff`
Show the current staged diff (doesn't require API key).

```bash
committor diff
```

## Examples

### Example 1: Adding a new feature

```bash
# Make changes to your code
echo "pub fn new_feature() { println!(\"Hello!\"); }" >> src/lib.rs

# Stage the changes
git add src/lib.rs

# Generate commit messages
committor generate
```

Output:
```
Generated commit message options:

1. feat(lib): add new_feature function
2. feat: implement new_feature in lib module
3. chore(lib): add new_feature function to library
```

### Example 2: Fixing a bug

```bash
# Fix a bug in your code
sed -i 's/bug/fixed_bug/g' src/main.rs

# Stage the changes
git add src/main.rs

# Generate and commit
committor commit --auto-commit
```

### Example 3: Documentation changes

```bash
# Update documentation
echo "## New Section" >> README.md

# Stage the changes
git add README.md

# Generate commit messages
committor generate --show-diff
```

Output:
```
Current staged diff:
diff --git a/README.md b/README.md
index abc123..def456 100644
--- a/README.md
+++ b/README.md
@@ -10,3 +10,5 @@ This is the README.
 ## Installation

 Run `cargo install`.
+
+## New Section
────────────────────────────────────────────────────────────────────────────────

Generated commit message options:

1. docs(readme): add new section to README
2. docs: update README with new section
3. chore(docs): add new section to documentation
```

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `--api-key` | OpenAI API key | From `OPENAI_API_KEY` env var |
| `--model` | Model to use | `gpt-4` |
| `--count` | Number of options to generate | `3` |
| `--auto-commit` | Automatically use first suggestion | `false` |
| `--show-diff` | Show diff before generating | `false` |

## Conventional Commit Types

Committor generates messages following the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that don't affect code meaning
- `refactor`: Code changes that neither fix bugs nor add features
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to build process or auxiliary tools
- `perf`: Performance improvements
- `ci`: Changes to CI configuration
- `build`: Changes affecting build system or dependencies

## Troubleshooting

### "OpenAI API key not found"
Make sure you've set the `OPENAI_API_KEY` environment variable or use the `--api-key` flag.

### "Not in a git repository"
Make sure you're running the command inside a git repository.

### "No staged changes found"
Stage your changes first with `git add <files>`.

### API Rate Limits
If you hit OpenAI's rate limits, wait a moment and try again.

## Tips

1. **Stage meaningful changes**: The tool works best with coherent, focused changes.

2. **Use descriptive file names**: The AI considers file names and paths when generating messages.

3. **Review before committing**: Always review the generated messages before committing.

4. **Branch naming helps**: Use descriptive branch names like `feature/user-auth` or `fix/login-bug` for better context.

5. **Combine with git hooks**: You can integrate Committor into git hooks for automated commit message generation.

## Integration with Git Hooks

You can set up a git hook to automatically suggest commit messages:

```bash
# Create a commit-msg hook
cat > .git/hooks/commit-msg << 'EOF'
#!/bin/bash
if [ -z "$OPENAI_API_KEY" ]; then
    echo "Tip: Set OPENAI_API_KEY to use Committor for automatic commit messages"
    exit 0
fi

# Generate suggestion
SUGGESTION=$(committor generate --count 1 2>/dev/null | tail -n 1)
if [ $? -eq 0 ] && [ -n "$SUGGESTION" ]; then
    echo "Committor suggests: $SUGGESTION"
fi
EOF

chmod +x .git/hooks/commit-msg
```

## Advanced Usage

### Custom Prompts
The tool uses carefully crafted prompts to generate conventional commit messages. The prompts consider:

- File types and extensions
- Change patterns (additions, deletions, modifications)
- Project context (language, framework)
- Commit message best practices

### Multiple Models
You can experiment with different OpenAI models:

```bash
# Use GPT-3.5 Turbo (faster, cheaper)
committor generate --model gpt-3.5-turbo

# Use GPT-4 (default, more accurate)
committor generate --model gpt-4

# Use GPT-4 Turbo
committor generate --model gpt-4-turbo-preview
```

### Batch Processing
For multiple small commits, you can use a simple script:

```bash
#!/bin/bash
for file in $(git diff --cached --name-only); do
    git reset HEAD
    git add "$file"
    MESSAGE=$(committor generate --count 1 --auto-commit 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "Committed $file with: $MESSAGE"
    else
        echo "Failed to generate message for $file"
        git add "$file"  # Re-stage for manual commit
    fi
done
```

This guide should help you get started with Committor and make the most of its AI-powered commit message generation!
