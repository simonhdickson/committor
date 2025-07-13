# Commitor 🚀

Automatically generate conventional commit messages based on your git diff using AI.

Commitor is a Rust CLI tool that analyzes your staged git changes and generates conventional commit messages using AI models from OpenAI or Ollama. Say goodbye to writer's block when crafting commit messages!

✅ **COMPLETE**: Full implementation with AI-powered analysis and conventional commit generation!

## 🎉 Project Summary

This project successfully demonstrates a complete Rust application that:

- **Integrates with multiple AI providers** - OpenAI GPT models and Ollama local models
- **Analyzes git diffs** to understand code changes
- **Generates conventional commit messages** following industry standards
- **Provides a CLI interface** with multiple commands and options
- **Includes comprehensive error handling** and validation
- **Features modular architecture** with separate modules for different concerns
- **Has extensive test coverage** with unit and integration tests
- **Supports multiple AI models** and configuration options

## Features

- 🤖 **AI-Powered**: Uses OpenAI GPT models or Ollama local models to analyze your code changes
- 📝 **Conventional Commits**: Generates messages following the conventional commit format
- 🎯 **Multiple Options**: Generate multiple commit message suggestions to choose from
- ⚡ **Fast**: Built in Rust for optimal performance
- 🔧 **Flexible**: Supports different providers, models and customization options
- 🏠 **Local Support**: Use Ollama for completely local AI processing
- 🎨 **Beautiful Output**: Colorized terminal output for better readability

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git
- One of the following:
  - OpenAI API key (for OpenAI provider)
  - Ollama installation (for local AI processing)

### Install from source

```bash
git clone https://github.com/simonhdickson/commitor.git
cd commitor
cargo install --path .
```

## Configuration

### OpenAI Setup

Set your OpenAI API key as an environment variable:

```bash
export OPENAI_API_KEY="your-api-key-here"
```

Or pass it directly using the `--api-key` flag.

### Ollama Setup

1. Install Ollama from [ollama.ai](https://ollama.ai)
2. Start the Ollama service:
```bash
ollama serve
```
3. Pull a model (e.g., llama2):
```bash
ollama pull llama2
```

## Usage

### Basic Usage

1. Stage your changes:
```bash
git add .
```

2. Generate commit messages with OpenAI (default):
```bash
commitor generate
```

3. Or use Ollama for local processing:
```bash
commitor --provider ollama --model llama2 generate
```

4. Generate and commit in one step:
```bash
commitor commit
```

### Command Line Options

```bash
commitor [OPTIONS] [COMMAND]

Commands:
  generate      Generate a commit message for staged changes
  commit        Generate and commit in one step
  diff          Show the current git diff
  models        List available models for the selected provider
  check-ollama  Check if Ollama is available (only for Ollama provider)

Options:
  --provider <PROVIDER>        AI provider to use [default: openai] [possible values: openai, ollama]
  --api-key <API_KEY>          OpenAI API key [env: OPENAI_API_KEY]
  --ollama-url <OLLAMA_URL>    Ollama base URL [default: http://localhost:11434]
  --ollama-timeout <TIMEOUT>   Timeout for Ollama requests in seconds [default: 30]
  --model <MODEL>              Model to use for generation [default: gpt-4]
  --count <COUNT>              Maximum number of commit message options to generate [default: 3]
  -y, --auto-commit            Automatically use the first generated commit message
  --show-diff                  Show the git diff before generating commit message
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples

**Generate multiple commit message options with OpenAI:**
```bash
commitor generate --count 5
```

**Use Ollama with a local model:**
```bash
commitor --provider ollama --model llama2 generate
```

**Use a different OpenAI model:**
```bash
commitor generate --model gpt-3.5-turbo
```

**Auto-commit with the first suggestion:**
```bash
commitor commit --auto-commit
```

**Show diff before generating:**
```bash
commitor generate --show-diff
```

**List available models (shows your installed models):**
```bash
commitor models --provider ollama
```

**Check Ollama availability:**
```bash
commitor check-ollama
```

**Use custom Ollama URL:**
```bash
commitor --provider ollama --ollama-url http://localhost:11434 --model codellama generate
```

## Conventional Commit Format

Commitor generates messages following the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>
```

### Supported Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools
- `perf`: A code change that improves performance
- `ci`: Changes to CI configuration files and scripts
- `build`: Changes that affect the build system or external dependencies

### Example Messages

- `feat(auth): add JWT token validation`
- `fix(database): resolve connection timeout issue`
- `docs(readme): update installation instructions`
- `refactor(utils): simplify string parsing logic`
- `test(api): add integration tests for user endpoints`

## Configuration

You can customize the behavior by setting environment variables:

```bash
# Set your OpenAI API key (for OpenAI provider)
export OPENAI_API_KEY="sk-..."

# Set default model (applies to both providers)
export COMMITOR_MODEL="gpt-4"

# Set default count
export COMMITOR_COUNT="3"
```

### Ollama Models

Popular models you can use with Ollama:

- `llama2`: General purpose model
- `codellama`: Optimized for code understanding
- `mistral`: Fast and efficient model
- `neural-chat`: Good for conversational tasks
- `deepseek-coder`: Specialized for coding tasks

Pull models using:
```bash
ollama pull <model-name>
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

1. Clone the repository
2. Install dependencies: `cargo build`
3. Run tests: `cargo test`
4. Run the tool: `cargo run -- generate`

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [rig.rs](https://github.com/0xPlaygrounds/rig) for unified AI provider integration (OpenAI and Ollama)
- [Ollama](https://ollama.ai) for local AI model support
- Inspired by the [Conventional Commits](https://www.conventionalcommits.org/) specification
- Uses [git2](https://github.com/rust-lang/git2-rs) for git operations

## Troubleshooting

### Common Issues

**"Not in a git repository"**
- Make sure you're running the command inside a git repository
- Initialize a git repository with `git init` if needed

**"No staged changes found"**
- Stage your changes first with `git add <files>`
- Check staged changes with `git status`

**"OpenAI API key not found"** (OpenAI provider)
- Set the `OPENAI_API_KEY` environment variable
- Or use the `--api-key` flag

**"Ollama is not available"** (Ollama provider)
- Make sure Ollama is installed and running: `ollama serve`
- Check if Ollama is accessible: `commitor check-ollama`
- Verify the URL is correct with `--ollama-url`

**API rate limits** (OpenAI provider)
- The tool respects OpenAI's rate limits
- If you hit limits, wait a moment and try again

**Model not found** (Ollama provider)
- Pull the model first: `ollama pull <model-name>`
- List your installed models: `commitor models --provider ollama`

### Debug Mode

Run with debug logging:
```bash
RUST_LOG=debug commitor generate
```

## ✅ Implementation Status

**Core Features Implemented:**
- ✅ OpenAI GPT integration using rig.rs
- ✅ Git diff analysis and parsing
- ✅ Conventional commit message generation
- ✅ CLI with multiple commands (generate, commit, diff)
- ✅ Environment variable and flag configuration
- ✅ Multiple commit message options
- ✅ Auto-commit functionality
- ✅ Diff display and validation
- ✅ Comprehensive error handling
- ✅ Unit and integration tests
- ✅ Modular library architecture
- ✅ Installation and usage scripts

**Key Technical Achievements:**
- Built with **Rust** for performance and safety
- Uses **rig.rs** for unified AI provider integration (OpenAI and Ollama)
- Implements **conventional commits** specification
- Features **async/await** for non-blocking operations
- Includes **colored terminal output** for better UX
- Has **comprehensive documentation** and examples
- Supports **multiple AI models** across different providers

**Future Roadmap:**
- [x] Support for local models via Ollama
- [ ] Support for more AI providers (Anthropic, Claude)
- [ ] Configuration file support
- [ ] Advanced git hooks integration
- [ ] Commit message templates
- [ ] Enhanced scope detection
- [ ] Batch processing for multiple commits
- [ ] Custom prompt templates
