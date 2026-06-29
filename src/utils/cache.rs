#![allow(dead_code)]
use anyhow::Result;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use super::config::config_dir;

/// Initialize cache directory
pub fn init_cache_dir() -> Result<()> {
    let cache_dir = config_dir().join("cache");
    std::fs::create_dir_all(&cache_dir)?;
    Ok(())
}

/// Get cache directory path
pub fn cache_dir() -> PathBuf {
    config_dir().join("cache")
}

/// Cache key for a file (based on path + modified time)
pub fn file_cache_key(path: &std::path::Path) -> Result<String> {
    let metadata = std::fs::metadata(path)?;
    let modified = metadata.modified()?;
    // Produce a stable hex hash for the cache key
    let path_str = path.to_string_lossy();
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path_str.hash(&mut hasher);
    duration.as_nanos().hash(&mut hasher);
    let hash = hasher.finish();
    let key = format!("{:016x}", hash);
    Ok(key)
}
