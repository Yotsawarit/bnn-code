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
    pub recovery_codes: Option<RecoveryCodes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecoveryCodes {
    #[serde(rename = "version")]
    pub codes_version: String,
    pub codes: Vec<String>,
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
            recovery_codes: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InnerRecoveryCodes {
    codes_version: String,
    codes: Vec<String>,
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

/// Derive a key from a password for encryption
fn derive_key(password: &str) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

/// XOR encrypt a string with a key
fn xor_encrypt(data: &str, key: &[u8; 32]) -> String {
    let data_bytes = data.as_bytes();
    let mut result = Vec::new();
    for i in 0..data_bytes.len() {
        result.push(data_bytes[i] ^ key[i % key.len()]);
    }
    base64::encode(&result)
}

/// XOR decrypt a string with a key (same function as encrypt)
fn xor_decrypt(encrypted: &str, key: &[u8; 32]) -> anyhow::Result<String> {
    let encrypted_bytes = base64::decode(encrypted)?;
    let data_bytes = encrypted_bytes.iter().map(|b| *b ^ key[(*b as usize) % key.len()]).collect::<Vec<u8>>();
    Ok(String::from_utf8(data_bytes)?)
}

impl BnnConfig {
    /// Generate recovery codes and store them encrypted
    pub fn generate_and_store_recovery_codes(&mut self, master_password: &str) -> anyhow::Result<()> {
        // Generate 15 recovery codes (5-character alphanumeric)
        let mut codes = Vec::new();
        for _ in 0..15 {
            let code: String = (0..5)
                .map(|_| {
                    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
                    let idx = rand::random::<usize>() % CHARSET.len();
                    CHARSET[idx] as char
                })
                .collect();
            codes.push(code);
        }

        self.recovery_codes = Some(RecoveryCodes {
            codes_version: "1.0".to_string(),
            codes,
        });

        // Encrypt the codes and store with marker
        if let Some(ref rc) = self.recovery_codes {
            let inner = InnerRecoveryCodes {
                codes_version: rc.codes_version.clone(),
                codes: rc.codes.clone(),
            };
            let codes_json = serde_json::to_string(&inner)?;
            let key = derive_key(master_password);
            let encrypted = xor_encrypt(&codes_json, &key);
            // Store encrypted data as first code with prefix
            let mut codes_with_marker = rc.codes.clone();
            codes_with_marker.insert(0, format!("ENCRYPTED:{}", encrypted));
            // Update the recovery_codes
            if let Some(ref mut rc) = self.recovery_codes {
                rc.codes = codes_with_marker;
            }
        }

        Ok(())
    }

    /// Get recovery codes (decrypt if encrypted)
    pub fn get_recovery_codes(&self, master_password: &str) -> anyhow::Result<Vec<String>> {
        let Some(ref codes) = self.recovery_codes else {
            return Ok(Vec::new());
        };

        // Check if codes are encrypted (have ENCRYPTED: prefix)
        if codes.codes.first().map_or(false, |c| c.starts_with("ENCRYPTED:")) {
            let encrypted_data = &codes.codes[0][10..]; // Remove "ENCRYPTED:" prefix
            let key = derive_key(master_password);
            let decrypted = xor_decrypt(encrypted_data, &key)?;
            // Parse the decrypted JSON
            let recovered: InnerRecoveryCodes = serde_json::from_str(&decrypted)?;
            // Return all codes except the marker
            Ok(recovered.codes)
        } else {
            // Not encrypted, return as-is
            Ok(codes.codes.clone())
        }
    }

    /// Verify a recovery code
    pub fn verify_recovery_code(&self, code: &str, master_password: &str) -> anyhow::Result<bool> {
        let codes = self.get_recovery_codes(master_password)?;
        Ok(codes.iter().any(|c| c == code))
    }
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
        // Recovery codes should be None by default
        assert!(config.recovery_codes.is_none());
    }

    #[test]
    fn test_indexing_config_defaults() {
        let config = BnnConfig::default();
        assert!(
            config
                .indexing
                .exclude_dirs
                .contains(&"node_modules".to_string())
        );
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
            recovery_codes: None,
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