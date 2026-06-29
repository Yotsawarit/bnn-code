use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod indexer;
mod inference;
mod repl;
mod retrieval;
mod rogue;
mod ui;
mod utils;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("BNN_LOG_LEVEL").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Explain { file }) => {
            println!("🧠 Explaining file: {}", file);
            let response = inference::run_inference_on_file("explain", &file,
                "Explain the purpose, architecture, and key functions of the following code.")
                .await?;
            println!("\n📖 Explanation:\n{}", response);
        }
        Some(Commands::Refactor { file }) => {
            println!("🧠 Refactoring file: {}", file);
            let response = inference::run_inference_on_file("refactor", &file,
                "Suggest refactoring improvements for the following code. \
                 Focus on readability, maintainability, and performance.")
                .await?;
            println!("\n🔧 Refactoring suggestions:\n{}", response);
        }
        Some(Commands::Test { file }) => {
            println!("🧠 Generating tests for: {}", file);
            let response = inference::run_inference_on_file("test", &file,
                "Generate comprehensive unit tests for the following code. \
                 Include edge cases and use the appropriate testing framework.")
                .await?;
            println!("\n🧪 Generated tests:\n{}", response);
        }
        Some(Commands::Init) => {
            println!("🧠 Initializing BNN Code in current directory...");
            utils::init_project()?;
        }
        Some(Commands::Fix { file }) => {
            match file {
                Some(f) => {
                    println!("🧠 Fixing file: {}", f);
                    let response = inference::run_inference_on_file("fix", &f,
                        "Fix bugs, errors, and issues in the following code.")
                        .await?;
                    println!("\n✅ Fix suggestions:\n{}", response);
                }
                None => {
                    println!("🧠 Fixing codebase (scanning all files)...");
                    let response = inference::run_inference_on_codebase("fix",
                        "Scan the entire codebase and fix all bugs, errors, and issues found.")
                        .await?;
                    println!("\n✅ Fix suggestions:\n{}", response);
                }
            }
        }
        Some(Commands::Commit) => {
            println!("🧠 Generating commit message from staged changes...");
            // Read git diff --cached
            let diff_output = std::process::Command::new("git")
                .args(["diff", "--cached"])
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to run git diff: {}. Are you in a git repository?", e))?;

            if !diff_output.status.success() {
                anyhow::bail!("git diff --cached failed. Make sure you're in a git repo with staged changes.");
            }

            let diff = String::from_utf8_lossy(&diff_output.stdout);
            if diff.trim().is_empty() {
                anyhow::bail!("No staged changes found. Run `git add` first.");
            }

            let prompt = format!(
                "Generate a concise, conventional commit message for the following git diff.\n\
                 Follow the Conventional Commits format (type(scope): description).\n\n{}",
                diff
            );
            let response = inference::run_inference(&prompt, &[]).await?;
            println!("\n📝 Suggested commit message:\n");
            println!("{}", response);
            println!("\n💡 Tip: Use `git commit -m \"...\"` with the message above.");
        }
        Some(Commands::Review { file }) => {
            match file {
                Some(f) => {
                    println!("🧠 Reviewing file: {}", f);
                    let response = inference::run_inference_on_file("review", &f,
                        "Review the following code for bugs, security vulnerabilities, \
                         performance issues, and suggest improvements.")
                        .await?;
                    println!("\n📋 Review Results:\n{}", response);
                }
                None => {
                    println!("🧠 Reviewing staged changes...");
                    let diff_output = std::process::Command::new("git")
                        .args(["diff", "--cached"])
                        .output()
                        .map_err(|e| anyhow::anyhow!("Failed to run git diff: {}", e))?;

                    let diff = String::from_utf8_lossy(&diff_output.stdout);
                    if diff.trim().is_empty() {
                        anyhow::bail!("No staged changes to review. Run `git add` first or specify a file.");
                    }

                    let prompt = format!(
                        "Review the following git diff for bugs, security issues, \
                         and code quality problems. Provide specific recommendations.\n\n{}",
                        diff
                    );
                    let response = inference::run_inference(&prompt, &[]).await?;
                    println!("\n📋 Review Results:\n{}", response);
                }
            }
        }
        Some(Commands::Document { file }) => {
            println!("🧠 Generating documentation for: {}", file);
            let response = inference::run_inference_on_file("document", &file,
                "Generate comprehensive documentation for the following code.\n\
                 Include docstrings, function descriptions, parameter explanations, \
                 and usage examples where appropriate.")
                .await?;
            println!("\n📖 Documentation:\n{}", response);
        }
        Some(Commands::Rogue { category, json }) => {
            use rogue::{RogueEngine, format_report};
            let mut engine = RogueEngine::new();
            let report = if let Some(cat) = category {
                engine.run_category(&cat)?
            } else {
                engine.run_all()?
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", format_report(&report, true));
            }
        }
        None => {
            // Interactive or one-shot mode
            if let Some(query) = cli.query {
                // One-shot mode
                run_query(&query, &cli.path).await?;
            } else {
                // REPL mode
                repl::run_repl(cli.path).await?;
            }
        }
    }

    Ok(())
}

async fn run_query(query: &str, path: &str) -> Result<()> {
    println!("🧠 BNN Code");
    println!("Query: {}", query);
    println!("Path: {}", path);

    // Step 1: Index codebase
    let mut indexer = indexer::CodebaseIndexer::new(path)?;
    let num_chunks = indexer.index().await?;
    println!("✓ Indexed {} chunks", num_chunks);

    // Step 2: Retrieve context
    let context = retrieval::search(query, 3).await?;
    println!("✓ Retrieved {} relevant chunks", context.len());

    // Step 3: Generate response
    let response = inference::generate(query, &context).await?;
    println!("\n✨ Response:\n{}", response);

    Ok(())
}
