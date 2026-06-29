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
            // TODO: Implement explain logic
            let content = std::fs::read_to_string(&file)?;
            println!("\n📄 File: {}", file);
            println!("{}", content);
        }
        Some(Commands::Refactor { file }) => {
            println!("🧠 Refactoring file: {}", file);
            // TODO: Implement refactor logic
            println!("Analysis complete. Suggested refactoring will be available in a future release.");
        }
        Some(Commands::Test { file }) => {
            println!("🧠 Generating tests for: {}", file);
            // TODO: Implement test generation logic
            println!("Test generation will be available in a future release.");
        }
        Some(Commands::Init) => {
            println!("🧠 Initializing BNN Code in current directory...");
            utils::init_project()?;
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
