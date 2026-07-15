use anyhow::Result;

/// In-memory conversation history for the REPL session
struct SessionMemory {
    history: Vec<(String, String)>,
    max_turns: usize,
}

impl SessionMemory {
    fn new() -> Self {
        Self {
            history: Vec::new(),
            max_turns: 10,
        }
    }

    fn push(&mut self, query: String, response: String) {
        self.history.push((query, response));
        if self.history.len() > self.max_turns {
            self.history.remove(0);
        }
    }

    fn clear(&mut self) {
        self.history.clear();
    }

    fn format_context(&self) -> String {
        if self.history.is_empty() {
            return String::new();
        }
        let mut ctx = String::from("\nPrevious conversation:\n");
        for (i, (q, r)) in self.history.iter().enumerate() {
            ctx.push_str(&format!("Q{}: {}\nA{}: {}\n", i + 1, q, i + 1, r));
        }
        ctx
    }
}

/// Interactive REPL loop for the BNN Code agent
pub async fn run_repl(path: String) -> Result<()> {
    println!("🧠 BNN Code Interactive Mode");
    println!("Path: {}", path);
    println!("Type 'exit' to quit, '/help' for commands\n");

    let mut indexer = crate::indexer::CodebaseIndexer::new(&path)?;
    println!("Indexing codebase...");
    let num_chunks = indexer.index().await?;
    println!("✓ Indexed {} chunks\n", num_chunks);

    let mut memory = SessionMemory::new();

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
                println!("  /clear          Clear conversation history");
                println!("  <query>         Ask a question about the codebase");
            }
            "/stats" => {
                println!("Codebase indexed at: {}", path);
                println!("Total chunks: {}", num_chunks);
                println!("Conversation turns: {}", memory.history.len());
            }
            "/clear" => {
                memory.clear();
                println!("✓ Conversation history cleared");
            }
            "" => continue,
            query => {
                let session_context = memory.format_context();
                let mut context = if num_chunks > 0 {
                    vec![format!(
                        "Codebase indexed at {} with {} chunks.",
                        path, num_chunks
                    )]
                } else {
                    Vec::new()
                };
                if !session_context.is_empty() {
                    context.push(session_context);
                }

                match crate::inference::generate(query, &context).await {
                    Ok(response) => {
                        println!("\n✨ Response:\n{}\n", response);
                        memory.push(query.to_string(), response);
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
