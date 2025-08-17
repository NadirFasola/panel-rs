// src/core/config.rs

use anyhow::{Context, Result};
use serde::Deserialize;
// use std::time::Duration;
use std::fs;

use super::items::battery::BatteryBackendKind;
use super::items::temp::TempBackendKind;

use tracing::info;

use super::config_loader::config_paths;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ModuleConfig {
    #[serde(default)]
    pub clock: ClockConfig,

    #[serde(default)]
    pub cpu: CpuConfig,

    #[serde(default)]
    pub mem: MemConfig,

    #[serde(default)]
    pub battery: BatteryConfig,

    #[serde(default)]
    pub temp: TempConfig,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        ModuleConfig {
            clock: ClockConfig::default(),
            cpu: CpuConfig::default(),
            mem: MemConfig::default(),
            battery: BatteryConfig::default(),
            temp: TempConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct BatteryConfig {
    #[serde(default)]
    pub backend: BatteryBackendKind,
    #[serde(default)]
    pub device: Option<String>,
    #[serde(default)]
    pub refresh_secs: Option<u32>,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        BatteryConfig {
            backend: BatteryBackendKind::Sysfs,
            device: None,
            refresh_secs: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct MemConfig {
    // pub preferred: String,
    #[serde(default)]
    pub refresh_secs: Option<u32>,
}

impl Default for MemConfig {
    fn default() -> Self {
        MemConfig {
            // preferred: "available",
            refresh_secs: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct TempConfig {
    pub backend: TempBackendKind,
    #[serde(default)]
    pub refresh_secs: Option<u32>,
    pub sensors: Vec<String>,
}

impl Default for TempConfig {
    fn default() -> Self {
        TempConfig {
            backend: TempBackendKind::ThermalZone,
            refresh_secs: None,
            sensors: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct CpuConfig {
    #[serde(default)]
    pub refresh_secs: Option<u32>,
}

impl Default for CpuConfig {
    fn default() -> Self {
        CpuConfig { refresh_secs: None }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ClockConfig {
    #[serde(default)]
    pub refresh_secs: Option<u32>,
    #[serde(default)]
    pub format: String,
}

impl Default for ClockConfig {
    fn default() -> Self {
        ClockConfig {
            refresh_secs: None,
            format: "%H:%M:%S".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // Which items to enable in the bar, in order
    #[serde(default = "default_items")]
    pub items: Vec<String>,

    // Refresh interval for items that poll (in seconds)
    #[serde(default = "default_refresh_secs")]
    pub refresh_secs: u32,

    // Module-specific configs
    #[serde(default)]
    pub modules: ModuleConfig,
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
            cfg.modules = user_cfg.modules;
        } else {
            info!(path = ?user, "No user config found; using defaults");
        }

        // 3. Validate config values
        if cfg.refresh_secs == 0 {
            Err(anyhow::anyhow!("refresh_secs must be at least 1"))?
        }

        // 4. Mutate each sub-config in place: fill in missing per-module rates
        let global = cfg.refresh_secs;
        cfg.modules.fill_default_refresh(global);

        info!(?cfg, "Configuration loaded succesfully");
        Ok(cfg)
    }
}

// Default to 1 second if not specified
fn default_refresh_secs() -> u32 {
    1
}

// Default to no items if not specified
fn default_items() -> Vec<String> {
    Vec::new()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            items: Vec::new(),
            refresh_secs: default_refresh_secs(),
            modules: ModuleConfig::default(),
        }
    }
}

pub trait Refreshable {
    // Returns `Some(rate)` if the module overrides it, or `None` otherwise
    fn fill_default_refresh(&mut self, global: u32);
}

impl Refreshable for BatteryConfig {
    fn fill_default_refresh(&mut self, global: u32) {
        self.refresh_secs = self.refresh_secs.or(Some(global));
    }
}

impl Refreshable for ClockConfig {
    fn fill_default_refresh(&mut self, global: u32) {
        self.refresh_secs = self.refresh_secs.or(Some(global));
    }
}

impl Refreshable for CpuConfig {
    fn fill_default_refresh(&mut self, global: u32) {
        self.refresh_secs = Some(global);
    }
}

impl Refreshable for MemConfig {
    fn fill_default_refresh(&mut self, global: u32) {
        self.refresh_secs = self.refresh_secs.or(Some(global));
    }
}

impl Refreshable for TempConfig {
    fn fill_default_refresh(&mut self, global: u32) {
        self.refresh_secs = self.refresh_secs.or(Some(global));
    }
}

impl Refreshable for ModuleConfig {
    fn fill_default_refresh(&mut self, global: u32) {
        self.battery.fill_default_refresh(global);
        self.clock.fill_default_refresh(global);
        self.cpu.fill_default_refresh(global);
        self.mem.fill_default_refresh(global);
        self.temp.fill_default_refresh(global);
    }
}
