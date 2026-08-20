use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ModelConfig {
    /// Model type (e.g. "bert", "codeberta", "codegen")
    #[serde(alias = "model_type")]
    pub model_type: Option<String>,
    /// Vocabulary size
    pub vocab_size: Option<usize>,
    /// Maximum sequence length
    #[serde(alias = "max_position_embeddings")]
    pub max_length: Option<usize>,
    /// Hidden dimension size
    pub hidden_size: Option<usize>,
    /// Number of attention heads
    pub num_attention_heads: Option<usize>,
    /// Number of hidden layers
    pub num_hidden_layers: Option<usize>,
}

impl ModelConfig {
    pub fn from_dir(dir: &Path) -> Result<Option<Self>> {
        let config_path = dir.join("config.json");
        if !config_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config: {}", config_path.display()))?;
        let config: ModelConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", config_path.display()))?;
        Ok(Some(config))
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_type: Some("codeberta".into()),
            vocab_size: Some(50265),
            max_length: Some(512),
            hidden_size: Some(384),
            num_attention_heads: Some(6),
            num_hidden_layers: Some(6),
        }
    }
}
