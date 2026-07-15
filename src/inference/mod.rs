pub mod bnn;
pub mod config;
pub mod onnx;
pub mod tokenizer;

use anyhow::{bail, Result};
use std::path::Path;

/// Run inference with context retrieved from codebase
pub async fn generate(query: &str, context: &[String]) -> Result<String> {
    generate_with_model(query, context, "default").await
}

/// Resolve model directory with fallback:
///   1. models/<name>/
///   2. models/ (default)
///   3. error with download hint
fn resolve_model_dir(name: &str) -> Result<std::path::PathBuf> {
    let candidates = if name == "default" {
        vec![Path::new("models").to_path_buf()]
    } else {
        vec![
            Path::new("models").join(name),
            Path::new("models").to_path_buf(),
        ]
    };

    for dir in &candidates {
        if dir.join("model.onnx").exists() && dir.join("tokenizer.json").exists() {
            return Ok(dir.clone());
        }
    }

    let tried: Vec<String> = candidates.iter().map(|d| d.display().to_string()).collect();
    bail!(
        "No model found in: {}\n\
         Run: bash scripts/download_model.sh --auto",
        tried.join(", ")
    );
}

/// Run inference using a specific model name
pub async fn generate_with_model(query: &str, context: &[String], model: &str) -> Result<String> {
    let model_dir = resolve_model_dir(model)?;
    let mut engine = bnn::BnnInference::new(&model_dir)?;
    let response = engine.generate(query, context).await?;
    Ok(response)
}
