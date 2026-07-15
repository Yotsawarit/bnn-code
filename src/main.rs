use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod indexer;
mod inference;
mod math;
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
            println!("🧠 Explaining: {}", file);
            let content = std::fs::read_to_string(&file)?;
            let prompt = format!(
                "Explain the following code. Describe what it does, \
                 its main components, and how they fit together:\n\n```rust\n{}\n```",
                content
            );
            let response = inference::generate_with_model(&prompt, &[], &cli.model).await?;
            println!("{}", response);
        }
        Some(Commands::Refactor { file }) => {
            println!("🧠 Refactoring: {}", file);
            let content = std::fs::read_to_string(&file)?;
            let prompt = format!(
                "Suggest refactoring improvements for the following code. \
                 Identify code smells, duplication, complexity, or style issues, \
                 and provide concrete before/after suggestions:\n\n```rust\n{}\n```",
                content
            );
            let response = inference::generate_with_model(&prompt, &[], &cli.model).await?;
            println!("{}", response);
        }
        Some(Commands::Test { file }) => {
            println!("🧠 Generating tests for: {}", file);
            let content = std::fs::read_to_string(&file)?;
            let prompt = format!(
                "Generate comprehensive unit tests for the following Rust code. \
                 Include tests for normal cases, edge cases, and error conditions. \
                 Use #[cfg(test)] mod tests with #[test] functions. \
                 Only output the test code:\n\n```rust\n{}\n```",
                content
            );
            let response = inference::generate_with_model(&prompt, &[], &cli.model).await?;
            println!("{}", response);
        }
        Some(Commands::Init) => {
            println!("🧠 Initializing BNN Code in current directory...");
            utils::init_project()?;
        }
        Some(Commands::Pi {
            digits,
            algorithm,
            bench,
        }) => {
            let pi = match algorithm.as_str() {
                "ramanujan" => {
                    if bench {
                        math::benchmark_ramanujan(digits)
                    } else {
                        math::ramanujan_pi(digits)
                    }
                }
                _ => {
                    if bench {
                        math::benchmark_chudnovsky(digits)
                    } else {
                        math::chudnovsky_pi(digits)
                    }
                }
            };
            println!("{}", pi);
        }
        Some(Commands::Rogue { category, json }) => {
            use rogue::{format_report, RogueEngine};
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
                run_query(&query, &cli.path, &cli.model).await?;
            } else {
                // REPL mode
                repl::run_repl(cli.path).await?;
            }
        }
    }

    Ok(())
}

async fn run_query(query: &str, path: &str, model: &str) -> Result<()> {
    println!("🧠 BNN Code");
    println!("Query: {}", query);
    println!("Path: {}", path);
    println!("Model: {}", model);

    // Step 1: Index codebase
    let mut indexer = indexer::CodebaseIndexer::new(path)?;
    let num_chunks = indexer.index().await?;
    println!("✓ Indexed {} chunks", num_chunks);

    // Step 2: Retrieve context
    let context = retrieval::search(query, 3).await?;
    println!("✓ Retrieved {} relevant chunks", context.len());

    // Step 3: Generate response
    let response = inference::generate_with_model(query, &context, model).await?;
    println!("\n✨ Response:\n{}", response);

    Ok(())
}
