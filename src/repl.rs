use anyhow::Result;

/// Interactive REPL loop for the BNN Code agent
pub async fn run_repl(path: String) -> Result<()> {
    println!("🧠 BNN Code Interactive Mode");
    println!("Path: {}", path);
    println!("Type 'exit' to quit, '/help' for commands\n");

    // Initialize indexer
    let mut indexer = crate::indexer::CodebaseIndexer::new(&path)?;
    println!("Indexing codebase...");
    let num_chunks = indexer.index().await?;
    println!("✓ Indexed {} chunks\n", num_chunks);

    // Simple REPL loop
    loop {
        let input = {
            use std::io::{self, Write};
            print!("bnn> ");
            io::stdout().flush()?;
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;
            buf.trim().to_string()
        };

        match input.as_str() {
            "exit" | "quit" | ":q" => {
                println!("Goodbye!");
                break;
            }
            "/help" | "help" => {
                println!("Commands:");
                println!("  exit, quit, :q  Exit REPL");
                println!("  /help           Show this help");
                println!("  /stats          Show index statistics");
                println!("  <query>         Ask a question about the codebase");
            }
            "/stats" => {
                println!("Codebase indexed at: {}", path);
                println!("Total chunks: {}", num_chunks);
            }
            "" => continue,
            query => {
                // Run inference
                match crate::inference::generate(query, &[]).await {
                    Ok(response) => {
                        println!("\n✨ Response:\n{}\n", response);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
