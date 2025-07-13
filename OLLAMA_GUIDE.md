# Ollama Integration Guide 🦙

This guide covers how to use Committor with Ollama for completely local AI-powered commit message generation.

## What is Ollama?

[Ollama](https://ollama.ai) is a tool that lets you run large language models locally on your machine. Committor integrates with Ollama using the [rig.rs](https://github.com/0xPlaygrounds/rig) library, providing a unified interface for both local and cloud AI processing. This means:

- 🔒 **Complete Privacy**: Your code never leaves your machine
- ⚡ **No API Costs**: No charges for API usage
- 🌐 **Offline Capable**: Works without internet connection
- 🎛️ **Full Control**: Choose and customize your models
- 🔧 **Unified API**: Same interface as OpenAI provider through rig.rs

## Installation & Setup

### 1. Install Ollama

**macOS:**
```bash
# Using Homebrew
brew install ollama

# Or download from https://ollama.ai
```

**Linux:**
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

**Windows:**
Download the installer from [ollama.ai](https://ollama.ai)

### 2. Start Ollama Service

```bash
ollama serve
```

This starts the Ollama server on `http://localhost:11434` (default).

### 3. Pull Models

Choose and download models for commit message generation:

```bash
# General purpose models
ollama pull llama2         # Meta's Llama 2 (7B)
ollama pull mistral        # Mistral 7B (fast and efficient)

# Code-specialized models (recommended for commit messages)
ollama pull codellama      # Code Llama (better for code understanding)
ollama pull deepseek-coder # Specialized for coding tasks
ollama pull neural-chat    # Good for conversational tasks

# Smaller/faster models
ollama pull tinyllama      # Very fast, smaller model
```

### 4. Verify Setup

```bash
# Check if Ollama is running
committor check-ollama

# List available models (shows actual installed models)
committor models --provider ollama
```

## Usage Examples

### Basic Usage

Generate commit messages using Ollama:

```bash
# Stage your changes
git add .

# Generate with default model (you'll need to specify one)
committor --provider ollama --model llama2 generate

# Generate multiple options
committor --provider ollama --model codellama --count 5 generate
```

### Model-Specific Examples

**Code Llama (Recommended for commit messages):**
```bash
committor --provider ollama --model codellama generate
```

**DeepSeek Coder (Excellent for code analysis):**
```bash
committor --provider ollama --model deepseek-coder generate
```

**Mistral (Fast and efficient):**
```bash
committor --provider ollama --model mistral generate
```

### Advanced Configuration

**Custom Ollama URL:**
```bash
committor --provider ollama --ollama-url http://192.168.1.100:11434 --model llama2 generate
```

**Custom timeout:**
```bash
committor --provider ollama --ollama-timeout 60 --model codellama generate
```

**Auto-commit:**
```bash
committor --provider ollama --model codellama commit --auto-commit
```

**Show diff before generating:**
```bash
committor --provider ollama --model codellama generate --show-diff
```

## Recommended Models by Use Case

### For Commit Messages

1. **CodeLlama** - Best overall for code understanding
   ```bash
   ollama pull codellama
   committor --provider ollama --model codellama generate
   ```

2. **DeepSeek-Coder** - Specialized for coding tasks
   ```bash
   ollama pull deepseek-coder
   committor --provider ollama --model deepseek-coder generate
   ```

3. **Mistral** - Good balance of speed and quality
   ```bash
   ollama pull mistral
   committor --provider ollama --model mistral generate
   ```

### For Different Project Types

**Web Development:**
- `codellama` - Good for JavaScript, TypeScript, CSS
- `deepseek-coder` - Excellent for React, Vue, Angular

**Systems Programming:**
- `codellama` - Great for Rust, C++, Go
- `deepseek-coder` - Good for understanding low-level changes

**Python/Data Science:**
- `codellama` - Excellent for Python understanding
- `neural-chat` - Good for data analysis commits

**Documentation:**
- `llama2` - Good for documentation changes
- `mistral` - Fast for simple doc updates

## Performance Tips

### Model Size vs Speed

| Model | Size | Speed | Quality | Best For |
|-------|------|-------|---------|----------|
| tinyllama | ~1GB | ⚡⚡⚡ | ⭐⭐ | Quick commits |
| mistral | ~4GB | ⚡⚡ | ⭐⭐⭐ | General use |
| llama2 | ~4GB | ⚡⚡ | ⭐⭐⭐ | Balanced |
| codellama | ~4GB | ⚡ | ⭐⭐⭐⭐ | Code commits |
| deepseek-coder | ~7GB | ⚡ | ⭐⭐⭐⭐⭐ | Complex code |

### Hardware Recommendations

**Minimum Requirements:**
- 8GB RAM
- Any modern CPU
- 5GB free disk space

**Recommended:**
- 16GB+ RAM for larger models
- M1/M2 Mac or modern GPU for faster inference
- SSD for faster model loading

### Optimizing Performance

1. **Use smaller models for simple changes:**
   ```bash
   committor --provider ollama --model mistral generate
   ```

2. **Increase timeout for complex diffs:**
   ```bash
   committor --provider ollama --ollama-timeout 60 --model codellama generate
   ```

3. **Keep Ollama running to avoid startup delays:**
   ```bash
   # Start in background
   ollama serve &
   ```

## Troubleshooting

### Common Issues

**"Ollama is not available"**
```bash
# Check if Ollama is running
ps aux | grep ollama

# Start Ollama if not running
ollama serve

# Test connection
curl http://localhost:11434/api/tags
```

**"Model not found"**
```bash
# List available models
ollama list

# Pull the model you want
ollama pull codellama

# Verify it's available
committor models --provider ollama
```

**Slow performance**
```bash
# Try a smaller model
committor --provider ollama --model mistral generate

# Or increase timeout
committor --provider ollama --ollama-timeout 90 --model codellama generate
```

**Connection refused**
```bash
# Check if Ollama is running on different port
netstat -tlnp | grep ollama

# Use custom URL if needed
committor --provider ollama --ollama-url http://localhost:11435 --model llama2 generate
```

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug committor --provider ollama --model codellama generate
```

## Comparison: Ollama vs OpenAI

| Feature | Ollama | OpenAI |
|---------|---------|---------|
| **Privacy** | ✅ Complete | ❌ Sends to API |
| **Cost** | ✅ Free | ❌ Pay per token |
| **Internet** | ✅ Offline | ❌ Required |
| **Speed** | ⚡ Variable | ⚡⚡ Fast |
| **Quality** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Setup** | 🔧 Complex | 🔧 Simple |
| **Resources** | 💾 High | 💾 None |

## Configuration Files

Create a shell alias for convenience:

```bash
# Add to your ~/.bashrc or ~/.zshrc
alias commit-local="committor --provider ollama --model codellama"
alias commit-fast="committor --provider ollama --model mistral"

# Usage
git add .
commit-local generate
```

Or create a wrapper script:

```bash
#!/bin/bash
# Save as ~/bin/commit-ollama
committor --provider ollama --model codellama "$@"
```

## Best Practices

1. **Choose the right model for your project:**
   - Use `codellama` for most programming tasks
   - Use `mistral` when you need speed
   - Use `deepseek-coder` for complex code analysis

2. **Keep models up to date:**
   ```bash
   ollama pull codellama  # Updates to latest version
   ```

3. **Monitor resource usage:**
   ```bash
   # Check memory usage
   htop

   # Check disk usage
   du -sh ~/.ollama/
   ```

4. **Use appropriate timeouts:**
   - Short timeout (15-30s) for simple changes
   - Long timeout (60-120s) for complex diffs

5. **Combine with OpenAI when needed:**
   ```bash
   # Use Ollama for most commits
   committor --provider ollama --model codellama generate

   # Fall back to OpenAI for complex cases
   committor --provider openai --model gpt-4 generate
   ```

## Security Considerations

✅ **Advantages:**
- Code never leaves your machine
- No API keys to manage
- No rate limiting or usage tracking

⚠️ **Considerations:**
- Models are downloaded from Ollama's servers
- Verify model checksums if security is critical
- Keep Ollama updated for security patches

## Getting Help

- **Ollama Documentation**: https://ollama.ai/docs
- **Committor Issues**: Report issues on the project repository
- **Model Issues**: Check Ollama's GitHub for model-specific problems

## Example Workflow

Here's a complete workflow using Ollama:

```bash
# 1. Make some changes
echo "Added user authentication" >> features.md
git add features.md

# 2. Generate commit messages
committor --provider ollama --model codellama --count 3 generate

# 3. Review and select (or auto-commit)
committor --provider ollama --model codellama commit

# 4. Verify the commit
git log -1 --oneline
```

This completes the Ollama integration guide! You now have a powerful, private, and cost-effective way to generate commit messages locally.
