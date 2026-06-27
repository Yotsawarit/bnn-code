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
