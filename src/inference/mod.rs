pub mod bnn;
pub mod onnx;
pub mod tokenizer;

use anyhow::Result;
use std::path::Path;

/// Run inference with context retrieved from codebase
pub async fn generate(query: &str, context: &[String]) -> Result<String> {
    let model_dir = Path::new("models");
    let mut engine = bnn::BnnInference::new(model_dir)?;
    let response = engine.generate(query, context).await?;
    Ok(response)
}

/// Run inference with a direct prompt (no codebase retrieval needed).
/// Useful for commands like `commit`, `document`, and `fix` where the
/// input is provided directly rather than retrieved from an index.
pub async fn run_inference(prompt: &str, context: &[String]) -> Result<String> {
    let model_dir = Path::new("models");
    let mut engine = bnn::BnnInference::new(model_dir)?;
    let response = engine.generate(prompt, context).await?;
    Ok(response)
}

/// Run inference on a specific file: read the file, build a prompt with
/// its content, and return the AI-generated response.
pub async fn run_inference_on_file(_command: &str, file_path: &str, instruction: &str) -> Result<String> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file_path, e))?;

    let prompt = format!(
        "{}\n\n```\n{}\n```",
        instruction, content
    );

    let model_dir = Path::new("models");
    let mut engine = bnn::BnnInference::new(model_dir)?;
    let response = engine.generate(&prompt, &[]).await?;
    Ok(response)
}

/// Run inference on the entire codebase: index, retrieve context
/// relevant to the instruction, and generate a response.
pub async fn run_inference_on_codebase(command: &str, instruction: &str) -> Result<String> {
    let path = ".";
    let mut indexer = crate::indexer::CodebaseIndexer::new(path)?;
    let num_chunks = indexer.index().await?;
    tracing::info!("Indexed {} chunks for '{}' command", num_chunks, command);

    let context = crate::retrieval::search(instruction, 5).await?;
    tracing::info!("Retrieved {} relevant chunks", context.len());

    let model_dir = Path::new("models");
    let mut engine = bnn::BnnInference::new(model_dir)?;
    let response = engine.generate(instruction, &context).await?;
    Ok(response)
}
