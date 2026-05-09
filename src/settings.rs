use crate::Settings;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// On-disk representation of the optional user configuration file.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    /// Convert YouTube video links to the `youtu.be` short format.
    #[serde(default)]
    pub youtube_shorten: Option<bool>,

    /// Shorten Walmart product links to `walmart.com/ip/{id}`.
    #[serde(default)]
    pub walmart_shorten: Option<bool>,

    /// Replace `twitter.com` / `x.com` with `fxtwitter.com`.
    #[serde(default)]
    pub fix_twitter: Option<bool>,

    /// Replace `bsky.app` with `fxbsky.app`.
    #[serde(default)]
    pub fix_bluesky: Option<bool>,

    /// Amazon affiliate tracking ID appended to Amazon links.
    #[serde(default)]
    pub amazon_tracking_id: Option<String>,
}

impl ConfigFile {
    /// Load a `ConfigFile` from the given path, or return an empty default
    /// if the file does not exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {}", path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file at {}", path.display()))
    }

    /// Convert the on-disk config into the runtime `Settings` struct.
    ///
    /// Values that are `None` in the file remain `None` / default in the
    /// resulting `Settings`.
    pub fn to_settings(&self) -> Settings {
        Settings {
            youtube_shorten: self.youtube_shorten.unwrap_or(false),
            walmart_shorten: self.walmart_shorten.unwrap_or(false),
            fix_twitter: self.fix_twitter.unwrap_or(false),
            fix_bluesky: self.fix_bluesky.unwrap_or(false),
            amazon_tracking_id: self.amazon_tracking_id.clone(),
        }
    }
}

/// Return the default path for the configuration file:
/// Always `~/.config/lc/config.toml` regardless of platform.
pub fn default_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".config").join("lc").join("config.toml"))
}
