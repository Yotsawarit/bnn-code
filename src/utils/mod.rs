pub mod cache;
pub mod config;

use anyhow::Result;

/// Initialize BNN Code configuration in the current directory
pub fn init_project() -> Result<()> {
    config::init_config()?;
    cache::init_cache_dir()?;
    println!("✓ BNN Code initialized");
    println!("  - Created .bnn/config.json");
    println!("  - Created .bnn/cache/");
    println!("\nRun 'bnn' to start the REPL, or 'bnn <query>' for one-shot mode.");
    Ok(())
}
