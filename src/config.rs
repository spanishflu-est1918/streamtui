//! Configuration management for StreamTUI
//!
//! Handles config file loading/saving and API key management.
//! Config is stored at ~/.config/streamtui/config.toml

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Bundled TMDB API keys (from freekeys pool)
const TMDB_KEY_POOL: &[&str] = &[
    "fb7bb23f03b6994dafc674c074d01761",
    "e55425032d3d0f371fc776f302e7c09b",
    "8301a21598f8b45668d5711a814f01f6",
    "8cf43ad9c085135b9479ad5cf6bbcbda",
    "da63548086e399ffc910fbc08526df05",
    "13e53ff644a8bd4ba37b3e1044ad24f3",
    "269890f657dddf4635473cf4cf456576",
    "a2f888b27315e62e471b2d587048f32e",
    "8476a7ab80ad76f0936744df0430e67c",
    "5622cafbfe8f8cfe358a29c53e19bba0",
    "ae4bd1b6fce2a5648671bfc171d15ba4",
    "257654f35e3dff105574f97fb4b97035",
    "2f4038e83265214a0dcd6ec2eb3276f5",
    "9e43f45f94705cc8e1d5a0400d19a7b7",
    "af6887753365e14160254ac7f4345dd2",
    "06f10fc8741a672af455421c239a1ffc",
    "09ad8ace66eec34302943272db0e8d2c",
];

/// Application configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Cached TMDB API key
    pub tmdb_api_key: Option<String>,
    /// Default Chromecast device name
    pub default_device: Option<String>,
    /// Preferred quality (4k, 1080p, 720p, 480p)
    pub preferred_quality: Option<String>,
    /// Preferred subtitle languages
    pub subtitle_languages: Option<Vec<String>>,
}

impl Config {
    /// Get config file path (~/.config/streamtui/config.toml)
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("streamtui").join("config.toml"))
    }

    /// Load config from file, or return default if not found
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::path().ok_or_else(|| anyhow::anyhow!("Could not determine config path"))?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    /// Get TMDB API key with fallback chain:
    /// 1. Environment variable TMDB_API_KEY
    /// 2. Cached key from config file
    /// 3. Random key from bundled pool (and cache it)
    pub fn get_tmdb_api_key(&mut self) -> String {
        // 1. Check environment variable first
        if let Ok(key) = std::env::var("TMDB_API_KEY") {
            return key;
        }

        // 2. Check cached key in config
        if let Some(ref key) = self.tmdb_api_key {
            return key.clone();
        }

        // 3. Pick random key from pool and cache it
        let key = Self::random_pool_key();
        self.tmdb_api_key = Some(key.clone());
        let _ = self.save(); // Best effort save
        key
    }

    /// Get a random key from the bundled pool
    pub fn random_pool_key() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as usize)
            .unwrap_or(0);
        let idx = seed % TMDB_KEY_POOL.len();
        TMDB_KEY_POOL[idx].to_string()
    }

    /// Try next key from pool (when current key fails)
    /// Returns None if all keys exhausted
    pub fn try_next_pool_key(&mut self, failed_key: &str) -> Option<String> {
        // Find current key's index
        let current_idx = TMDB_KEY_POOL.iter().position(|&k| k == failed_key);

        // Try next key in pool
        let next_idx = current_idx.map(|i| (i + 1) % TMDB_KEY_POOL.len()).unwrap_or(0);

        // Don't loop forever - if we've tried all keys, give up
        let key = TMDB_KEY_POOL[next_idx].to_string();
        if Some(key.as_str()) == self.tmdb_api_key.as_deref() {
            return None; // We've cycled back to the cached key
        }

        self.tmdb_api_key = Some(key.clone());
        let _ = self.save();
        Some(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_pool_key() {
        let key = Config::random_pool_key();
        assert!(TMDB_KEY_POOL.contains(&key.as_str()));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.tmdb_api_key.is_none());
        assert!(config.default_device.is_none());
    }

    #[test]
    fn test_get_tmdb_api_key_returns_pool_key() {
        let mut config = Config::default();
        let key = config.get_tmdb_api_key();
        assert!(!key.is_empty());
        assert_eq!(key.len(), 32); // TMDB keys are 32 chars
    }
}
