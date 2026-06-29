#![allow(dead_code)]
#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct BnnConfig {
    pub version: String,
    pub model: ModelConfig,
    pub indexing: IndexingConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    pub path: String,
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_k: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub max_file_size_kb: usize,
    pub exclude_dirs: Vec<String>,
    pub include_extensions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_context: bool,
    pub stream_output: bool,
}

impl Default for BnnConfig {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            model: ModelConfig {
                path: "models/model.onnx".to_string(),
                max_tokens: 4096,
                temperature: 0.7,
                top_k: 40,
            },
            indexing: IndexingConfig {
                max_file_size_kb: 100,
                exclude_dirs: vec![
                    "node_modules".to_string(),
                    "target".to_string(),
                    ".git".to_string(),
                    "venv".to_string(),
                    "__pycache__".to_string(),
                ],
                include_extensions: vec![
                    "rs".to_string(),
                    "py".to_string(),
                    "js".to_string(),
                    "ts".to_string(),
                    "tsx".to_string(),
                    "jsx".to_string(),
                    "go".to_string(),
                    "java".to_string(),
                    "cpp".to_string(),
                    "c".to_string(),
                    "h".to_string(),
                    "hpp".to_string(),
                    "rb".to_string(),
                    "swift".to_string(),
                    "kt".to_string(),
                ],
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                show_context: true,
                stream_output: true,
            },
        }
    }
}

/// Get config directory path
pub fn config_dir() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push(".bnn");
    path
}

/// Initialize default configuration
pub fn init_config() -> Result<()> {
    let config_dir = config_dir();
    std::fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.json");
    if !config_path.exists() {
        let config = BnnConfig::default();
        let json = serde_json::to_string_pretty(&config)?;
        std::fs::write(&config_path, json)?;
        tracing::info!("Created config at {:?}", config_path);
    } else {
        tracing::debug!("Config already exists at {:?}", config_path);
    }

    Ok(())
}

/// Load configuration
pub fn load_config() -> Result<BnnConfig> {
    let config_path = config_dir().join("config.json");
    if !config_path.exists() {
        return Ok(BnnConfig::default());
    }
    let json = std::fs::read_to_string(&config_path)?;
    let config: BnnConfig = serde_json::from_str(&json)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_default_config() {
        let config = BnnConfig::default();
        assert_eq!(config.version, "0.1.0");
        assert_eq!(config.model.max_tokens, 4096);
        assert_eq!(config.model.temperature, 0.7);
        assert_eq!(config.model.top_k, 40);
        assert_eq!(config.model.path, "models/model.onnx");
        assert_eq!(config.ui.theme, "dark");
        assert!(config.ui.stream_output);
        assert!(config.ui.show_context);
    }

    #[test]
    fn test_indexing_config_defaults() {
        let config = BnnConfig::default();
        assert!(config.indexing.exclude_dirs.contains(&"node_modules".to_string()));
        assert!(config.indexing.exclude_dirs.contains(&"target".to_string()));
        assert!(config.indexing.exclude_dirs.contains(&".git".to_string()));
        assert_eq!(config.indexing.max_file_size_kb, 100);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = BnnConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: BnnConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.version, deserialized.version);
        assert_eq!(config.model.max_tokens, deserialized.model.max_tokens);
        assert_eq!(config.ui.theme, deserialized.ui.theme);
    }

    #[test]
    fn test_config_custom_values() {
        let config = BnnConfig {
            version: "1.0.0".to_string(),
            model: ModelConfig {
                path: "custom.onnx".to_string(),
                max_tokens: 2048,
                temperature: 0.1,
                top_k: 10,
            },
            indexing: IndexingConfig {
                max_file_size_kb: 500,
                exclude_dirs: vec!["build".to_string()],
                include_extensions: vec!["py".to_string()],
            },
            ui: UiConfig {
                theme: "light".to_string(),
                show_context: false,
                stream_output: false,
            },
        };

        assert_eq!(config.model.path, "custom.onnx");
        assert_eq!(config.indexing.max_file_size_kb, 500);
        assert_eq!(config.ui.theme, "light");
    }

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.ends_with(".bnn"));
    }

    #[test]
    fn test_load_config_when_no_file() {
        // When config doesn't exist, load_config returns defaults
        let config = load_config().unwrap_or_default();
        // This should work even without a file
        assert_eq!(config.version, "0.1.0");
    }
}
