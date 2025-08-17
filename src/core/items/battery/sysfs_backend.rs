// src/core/items/battery/sysfs_backend.rs

use once_cell::sync::OnceCell;

use super::super::super::config::BatteryConfig;
use super::item::BatteryBackend;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

static SYSFS_PATHS: OnceCell<(PathBuf, PathBuf)> = OnceCell::new();

// Reads battery info from Linux sysfs
pub struct SysfsBackend {
    capacity_path: PathBuf,
    status_path: PathBuf,
}

impl SysfsBackend {
    // Scan `/sys/class/power_supply/` for a `type == Battery` entry
    pub fn new(cfg: &BatteryConfig) -> Result<Self> {
        if let Some(ref want) = cfg.device {
            let cap = PathBuf::from(want).join("capacity");
            let st = PathBuf::from(want).join("status");
            if cap.exists() && st.exists() {
                return Ok(Self {
                    capacity_path: cap,
                    status_path: st,
                });
            } else {
                anyhow::bail!("sysfs battery path `{}` not fount", want);
            }
        }

        let (cap, stat) = SYSFS_PATHS
            .get_or_try_init(|| {
                let base = PathBuf::from("/sys/class/power_supply");
                for entry in fs::read_dir(&base).context("Reading /sys/class/power_supply")? {
                    let entry = entry?;
                    let type_file = entry.path().join("type");
                    let typ = fs::read_to_string(&type_file)
                        .with_context(|| format!("Reading {}", type_file.display()))?;
                    if typ.trim_end() == "Battery" {
                        let cap_p = entry.path().join("capacity");
                        let stat_p = entry.path().join("status");
                        return Ok((cap_p, stat_p));
                    }
                }
                anyhow::bail!("No battery supply found in sysfs");
            })?
            .clone();

        Ok(Self {
            capacity_path: cap,
            status_path: stat,
        })
    }

    /// Read & parse a u64 from one of our pre‑cached files.
    fn read_u64(&self, path: &PathBuf, name: &str) -> Result<u64> {
        // 1. load the file once
        let txt =
            fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
        // 2. trim trailing whitespace in place
        let s = txt.trim_end();
        // 3. parse
        s.parse::<u64>()
            .with_context(|| format!("Parsing {} from sysfs", name))
    }
}

impl BatteryBackend for SysfsBackend {
    fn read(&self) -> Result<(u8, String)> {
        let cap64 = self.read_u64(&self.capacity_path, "capacity")?;
        let cap = u8::try_from(cap64).unwrap_or(0); // clamp if absurd
        let status = fs::read_to_string(&self.status_path)
            .with_context(|| format!("Reading {}", &self.status_path.display()))?;
        let clean = status.trim_end().to_string();
        Ok((cap, clean))
    }
}
