use crate::Settings;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// On-disk representation of the optional user configuration file.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub youtube_shorten: Option<bool>,
    #[serde(default)]
    pub fix_twitter: Option<bool>,
    #[serde(default)]
    pub fix_bluesky: Option<bool>,
    #[serde(default)]
    pub clipboard: Option<bool>,
}

impl ConfigFile {
    /// Load a `ConfigFile` from the given path, or return an empty default
    /// if the file does not exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents =
            fs::read_to_string(path).with_context(|| format!("failed to read config file at {}", path.display()))?;
        toml::from_str(&contents).with_context(|| format!("failed to parse config file at {}", path.display()))
    }

    pub fn to_settings(&self) -> Settings {
        Settings {
            youtube_shorten: self.youtube_shorten.unwrap_or(false),
            fix_twitter: self.fix_twitter.unwrap_or(false),
            fix_bluesky: self.fix_bluesky.unwrap_or(false),
        }
    }
}

/// Returns the default path for the configuration file:
/// `~/.config/hl/config.toml` on all platforms.
pub fn default_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("hl").join("config.toml"))
}
