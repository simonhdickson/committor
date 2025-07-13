use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use committor::{commit, providers, Committor, Config};
use std::env;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "committor")]
#[command(about = "Generate conventional commit messages automatically based on git diff")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// AI provider to use
    #[arg(long, value_enum, default_value = "openai")]
    provider: AIProviderType,

    /// OpenAI API key (can also be set via OPENAI_API_KEY environment variable)
    #[arg(long, env = "OPENAI_API_KEY")]
    api_key: Option<String>,

    /// Ollama base URL
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,

    /// Timeout for Ollama requests in seconds
    #[arg(long, default_value = "30")]
    ollama_timeout: u64,

    /// Model to use for generation
    #[arg(long, default_value = "llama2:7b")]
    model: String,

    /// Maximum number of commit message options to generate
    #[arg(long, default_value = "3")]
    count: u8,

    /// Automatically use the first generated commit message
    #[arg(long, short = 'y')]
    auto_commit: bool,

    /// Show the git diff before generating commit message
    #[arg(long)]
    show_diff: bool,
}

#[derive(Clone, Debug, ValueEnum)]
enum AIProviderType {
    #[value(name = "openai")]
    OpenAI,
    #[value(name = "ollama")]
    Ollama,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Generate a commit message for staged changes
    Generate,
    /// Generate and commit in one step
    Commit,
    /// Show the current git diff
    Diff,
    /// List available models for the selected provider
    Models,
    /// Check if Ollama is available (only for Ollama provider)
    CheckOllama,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Validate git environment first
    commit::validate_git_environment().context("Git environment validation failed")?;

    match cli.command.clone().unwrap_or(Commands::Generate) {
        Commands::Generate => {
            let committor = create_committor(&cli).await?;
            handle_generate_command(&committor, &cli).await?;
        }
        Commands::Commit => {
            let committor = create_committor(&cli).await?;
            handle_commit_command(&committor, &cli).await?;
        }
        Commands::Diff => {
            handle_diff_command()?;
        }
        Commands::Models => {
            handle_models_command(&cli).await?;
        }
        Commands::CheckOllama => {
            handle_check_ollama_command(&cli).await?;
        }
    }

    Ok(())
}

async fn create_committor(cli: &Cli) -> Result<Committor> {
    let config = match cli.provider {
        AIProviderType::OpenAI => {
            let api_key = cli
                .api_key
                .clone()
                .or_else(|| env::var("OPENAI_API_KEY").ok())
                .context(
                    "OpenAI API key not found. Set OPENAI_API_KEY environment variable or use --api-key",
                )?;

            Config::with_openai(
                api_key,
                cli.model.clone(),
                cli.count,
                cli.auto_commit,
                cli.show_diff,
            )
        }
        AIProviderType::Ollama => {
            // Check if Ollama is available
            if !providers::check_ollama_availability(&cli.ollama_url).await? {
                return Err(anyhow::anyhow!(
                    "Ollama is not available at {}. Please make sure Ollama is running.",
                    cli.ollama_url
                ));
            }

            Config::with_ollama_timeout(
                cli.ollama_url.clone(),
                cli.model.clone(),
                Duration::from_secs(cli.ollama_timeout),
                cli.count,
                cli.auto_commit,
                cli.show_diff,
            )
        }
    };

    Committor::new(config)
}

async fn handle_generate_command(committor: &Committor, cli: &Cli) -> Result<()> {
    let diff_content = committor.get_staged_diff()?;
    if diff_content.is_empty() {
        println!(
            "{}",
            "No staged changes found. Use 'git add' to stage changes first.".yellow()
        );
        return Ok(());
    }

    if cli.show_diff {
        println!("{}", "Current staged diff:".cyan().bold());
        println!("{diff_content}");
        println!("{}", "─".repeat(80).cyan());
    }

    info!("Generating commit messages...");
    let messages = committor.generate_commit_messages(&diff_content).await?;

    commit::display_commit_options(&messages);

    if cli.auto_commit && !messages.is_empty() {
        committor.commit_with_message(&messages[0])?;
    }

    Ok(())
}

async fn handle_commit_command(committor: &Committor, cli: &Cli) -> Result<()> {
    let diff_content = committor.get_staged_diff()?;
    if diff_content.is_empty() {
        println!(
            "{}",
            "No staged changes found. Use 'git add' to stage changes first.".yellow()
        );
        return Ok(());
    }

    if cli.show_diff {
        println!("{}", "Current staged diff:".cyan().bold());
        println!("{diff_content}");
        println!("{}", "─".repeat(80).cyan());
    }

    info!("Generating commit messages...");
    let messages = committor.generate_commit_messages(&diff_content).await?;

    if cli.auto_commit && !messages.is_empty() {
        committor.commit_with_message(&messages[0])?;
    } else if !messages.is_empty() {
        commit::display_commit_options(&messages);
        let choice = commit::prompt_user_choice(messages.len())?;
        if let Some(index) = choice {
            committor.commit_with_message(&messages[index])?;
        } else {
            println!("{}", "Commit cancelled.".yellow());
        }
    } else {
        warn!("No commit messages were generated");
    }

    Ok(())
}

fn handle_diff_command() -> Result<()> {
    use committor::diff;

    let diff_content = diff::get_staged_diff()?;
    if diff_content.is_empty() {
        println!("{}", "No staged changes found.".yellow());
    } else {
        println!("{diff_content}");
    }
    Ok(())
}

async fn handle_models_command(cli: &Cli) -> Result<()> {
    match cli.provider {
        AIProviderType::OpenAI => {
            println!("{}", "Available OpenAI models:".green().bold());
            let models = vec!["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo", "gpt-3.5-turbo-16k"];
            for model in models {
                println!("  {model}");
            }
        }
        AIProviderType::Ollama => {
            if !providers::check_ollama_availability(&cli.ollama_url).await? {
                return Err(anyhow::anyhow!(
                    "Ollama is not available at {}. Please make sure Ollama is running.",
                    cli.ollama_url
                ));
            }

            println!("{}", "Available Ollama models:".green().bold());
            let models = providers::get_ollama_models(&cli.ollama_url).await?;
            if models.is_empty() {
                println!(
                    "  {}",
                    "No models found. You may need to pull some models first.".yellow()
                );
                println!("  {}", "Example: ollama pull llama2".cyan());
            } else {
                for model in models {
                    println!("  {model}");
                }
            }
        }
    }
    Ok(())
}

async fn handle_check_ollama_command(cli: &Cli) -> Result<()> {
    println!(
        "{}",
        format!("Checking Ollama availability at {}...", cli.ollama_url).cyan()
    );

    match providers::check_ollama_availability(&cli.ollama_url).await {
        Ok(true) => {
            println!("{}", "✓ Ollama is available!".green().bold());

            // Also show available models
            match providers::get_ollama_models(&cli.ollama_url).await {
                Ok(models) => {
                    if models.is_empty() {
                        println!(
                            "{}",
                            "No models found. You may need to pull some models first.".yellow()
                        );
                        println!("{}", "Example: ollama pull llama2".cyan());
                    } else {
                        println!(
                            "{}",
                            format!("Available models: {}", models.join(", ")).cyan()
                        );
                    }
                }
                Err(e) => {
                    warn!("Could not fetch models: {}", e);
                }
            }
        }
        Ok(false) => {
            println!("{}", "✗ Ollama is not available".red().bold());
            println!(
                "{}",
                "Make sure Ollama is running and accessible at the specified URL.".yellow()
            );
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Error checking Ollama: {}", e));
        }
    }

    Ok(())
}
