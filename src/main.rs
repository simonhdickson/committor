use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use commitor::{commit, Commitor, Config};
use std::env;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "commitor")]
#[command(about = "Generate conventional commit messages automatically based on git diff")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// OpenAI API key (can also be set via OPENAI_API_KEY environment variable)
    #[arg(long, env = "OPENAI_API_KEY")]
    api_key: Option<String>,

    /// Model to use for generation
    #[arg(long, default_value = "gpt-4")]
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

#[derive(Subcommand, Clone)]
enum Commands {
    /// Generate a commit message for staged changes
    Generate,
    /// Generate and commit in one step
    Commit,
    /// Show the current git diff
    Diff,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Validate git environment first
    commit::validate_git_environment().context("Git environment validation failed")?;

    // Check for API key
    let api_key = cli
        .api_key
        .clone()
        .or_else(|| env::var("OPENAI_API_KEY").ok())
        .context(
            "OpenAI API key not found. Set OPENAI_API_KEY environment variable or use --api-key",
        )?;

    // Create configuration
    let config = Config::with_options(
        api_key,
        cli.model.clone(),
        cli.count,
        cli.auto_commit,
        cli.show_diff,
    );

    // Create commitor instance
    let commitor = Commitor::new(config);

    match cli.command.clone().unwrap_or(Commands::Generate) {
        Commands::Generate => {
            handle_generate_command(&commitor, &cli).await?;
        }
        Commands::Commit => {
            handle_commit_command(&commitor, &cli).await?;
        }
        Commands::Diff => {
            handle_diff_command(&commitor)?;
        }
    }

    Ok(())
}

async fn handle_generate_command(commitor: &Commitor, cli: &Cli) -> Result<()> {
    let diff_content = commitor.get_staged_diff()?;
    if diff_content.is_empty() {
        println!(
            "{}",
            "No staged changes found. Use 'git add' to stage changes first.".yellow()
        );
        return Ok(());
    }

    if cli.show_diff {
        println!("{}", "Current staged diff:".cyan().bold());
        println!("{}", diff_content);
        println!("{}", "─".repeat(80).cyan());
    }

    info!("Generating commit messages...");
    let messages = commitor.generate_commit_messages(&diff_content).await?;

    commit::display_commit_options(&messages);

    if cli.auto_commit && !messages.is_empty() {
        commitor.commit_with_message(&messages[0])?;
    }

    Ok(())
}

async fn handle_commit_command(commitor: &Commitor, cli: &Cli) -> Result<()> {
    let diff_content = commitor.get_staged_diff()?;
    if diff_content.is_empty() {
        println!(
            "{}",
            "No staged changes found. Use 'git add' to stage changes first.".yellow()
        );
        return Ok(());
    }

    if cli.show_diff {
        println!("{}", "Current staged diff:".cyan().bold());
        println!("{}", diff_content);
        println!("{}", "─".repeat(80).cyan());
    }

    info!("Generating commit messages...");
    let messages = commitor.generate_commit_messages(&diff_content).await?;

    if cli.auto_commit && !messages.is_empty() {
        commitor.commit_with_message(&messages[0])?;
    } else if !messages.is_empty() {
        commit::display_commit_options(&messages);
        let choice = commit::prompt_user_choice(messages.len())?;
        if let Some(index) = choice {
            commitor.commit_with_message(&messages[index])?;
        } else {
            println!("{}", "Commit cancelled.".yellow());
        }
    } else {
        warn!("No commit messages were generated");
    }

    Ok(())
}

fn handle_diff_command(commitor: &Commitor) -> Result<()> {
    let diff_content = commitor.get_staged_diff()?;
    if diff_content.is_empty() {
        println!("{}", "No staged changes found.".yellow());
    } else {
        println!("{}", diff_content);
    }
    Ok(())
}
