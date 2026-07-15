use anyhow::Result;
use ndarray::Array2;
use std::path::Path;

use super::config::ModelConfig;
use super::onnx::OnnxEngine;
use super::tokenizer::Tokenizer;

const DEFAULT_MAX_TOKENS: usize = 4096;

pub struct BnnInference {
    engine: OnnxEngine,
    tokenizer: Tokenizer,
    #[allow(dead_code)]
    config: ModelConfig,
    max_tokens: usize,
}

impl BnnInference {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        let config = ModelConfig::from_dir(model_dir)?.unwrap_or_default();
        let max_tokens = config.max_length.unwrap_or(DEFAULT_MAX_TOKENS);

        let engine = OnnxEngine::new(&model_path)?;
        engine.validate_io(&config)?;

        let tokenizer = Tokenizer::new(&tokenizer_path)?;

        Ok(Self {
            engine,
            tokenizer,
            config,
            max_tokens,
        })
    }

    #[allow(dead_code)]
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }

    pub async fn generate(&mut self, prompt: &str, context: &[String]) -> Result<String> {
        let mut full_prompt = String::from("You are a helpful coding assistant.\n\n");

        full_prompt.push_str("Context:\n");
        for chunk in context {
            full_prompt.push_str(chunk);
            full_prompt.push_str("\n---\n");
        }

        full_prompt.push_str("\nQuery: ");
        full_prompt.push_str(prompt);
        full_prompt.push_str("\n\nResponse:");

        let encoding = self.tokenizer.encode(&full_prompt, self.max_tokens)?;
        let input_ids = Array2::from_shape_vec((1, encoding.len()), encoding.ids.clone())?;
        let attention_mask =
            Array2::from_shape_vec((1, encoding.len()), encoding.attention_mask.clone())?;

        let output = self.engine.run(input_ids, attention_mask)?;

        let output_ids: Vec<i64> = output.iter().map(|&x| x as i64).collect();
        let response = self.tokenizer.decode(&output_ids)?;

        Ok(response)
    }
}
