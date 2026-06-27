use anyhow::Result;
use ndarray::Array2;
use std::path::Path;

use super::onnx::OnnxEngine;
use super::tokenizer::Tokenizer;

pub struct BnnInference {
    engine: OnnxEngine,
    tokenizer: Tokenizer,
    max_tokens: usize,
}

impl BnnInference {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        let engine = OnnxEngine::new(&model_path)?;
        let tokenizer = Tokenizer::new(&tokenizer_path)?;

        Ok(Self {
            engine,
            tokenizer,
            max_tokens: 4096,
        })
    }

    pub async fn generate(&mut self, prompt: &str, context: &[String]) -> Result<String> {
        // Build input with context
        let mut full_prompt = String::from("You are a helpful coding assistant.\n\n");

        // Add context
        full_prompt.push_str("Context:\n");
        for chunk in context {
            full_prompt.push_str(chunk);
            full_prompt.push_str("\n---\n");
        }

        full_prompt.push_str("\nQuery: ");
        full_prompt.push_str(prompt);
        full_prompt.push_str("\n\nResponse:");

        // Tokenize
        let encoding = self.tokenizer.encode(&full_prompt, self.max_tokens)?;
        let input_ids =
            Array2::from_shape_vec((1, encoding.len()), encoding.ids.clone())?;
        let attention_mask = Array2::from_shape_vec(
            (1, encoding.len()),
            encoding.attention_mask.clone(),
        )?;

        // Run inference
        let output = self.engine.run(input_ids, attention_mask)?;

        // Decode output
        let output_ids: Vec<i64> = output.iter().map(|&x| x as i64).collect();
        let response = self.tokenizer.decode(&output_ids)?;

        Ok(response)
    }
}
