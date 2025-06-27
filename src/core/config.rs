// src/core/config.rs

use anyhow::{Context, Result};
use serde::Deserialize;
// use std::time::Duration;
use std::fs;

use tracing::info;

use super::config_loader::config_paths;

#[derive(Debug, Deserialize)]
pub struct Config {
    // Which items to enable in the bar, in order
    pub items: Vec<String>,

    // Refresh interval for items that poll (in seconds)
    #[serde(default = "default_refresh_secs")]
    pub refresh_secs: u64,
}

impl Config {
    // Loads system default and then overrides with user config, if present
    pub fn load() -> Result<Self> {
        let (system, user) = config_paths();
        info!(system = ?system, user = ?user, "Loading configuration paths");

        // Ensure the user config directory exists
        if let Some(parent) = user.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Creating config directory at {parent:?}"))?;
        }

        // 1. Read system default (which should always exist in installed package)
        info!(path = ?system, "Reading system default config");
        let base = fs::read_to_string(&system)
            .with_context(|| format!("Reading system default config at {system:?}"))?;
        let mut cfg: Config = toml::from_str(&base).context("Parsing system default config")?;

        // 2. If user config exists, merge/override
        if user.exists() {
            info!(path = ?user, "Overlaying user configuration");
            let overlay = fs::read_to_string(&user)
                .with_context(|| format!("Reading user config at {user:?}"))?;
            let user_cfg: Config = toml::from_str(&overlay).context("Parsing user config")?;

            // Simple merge: replace entire items list & refresh
            cfg.items = user_cfg.items;
            cfg.refresh_secs = user_cfg.refresh_secs;
        } else {
            info!(path = ?user, "No user config found; using defaults");
        }

        // 3. Validate config values
        if cfg.refresh_secs == 0 {
            Err(anyhow::anyhow!("refresh_secs must be at least 1"))?
        }

        info!(?cfg, "Configuration loaded succesfully");
        Ok(cfg)
    }
}

// Default to 1 second if not specified
fn default_refresh_secs() -> u64 {
    1
}
