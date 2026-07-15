pub mod bnn;
pub mod onnx;
pub mod tokenizer;

use anyhow::Result;
use std::path::Path;

/// Run inference with context retrieved from codebase
pub async fn generate(query: &str, context: &[String]) -> Result<String> {
    generate_with_model(query, context, "default").await
}

/// Run inference using a specific model name
pub async fn generate_with_model(query: &str, context: &[String], model: &str) -> Result<String> {
    let model_dir = if model == "default" {
        Path::new("models").to_path_buf()
    } else {
        Path::new("models").join(model)
    };
    let mut engine = bnn::BnnInference::new(&model_dir)?;
    let response = engine.generate(query, context).await?;
    Ok(response)
}
