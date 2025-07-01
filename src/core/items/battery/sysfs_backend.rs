// src/core/items/battery/sysfs_backend.rs

use once_cell::sync::OnceCell;

use super::item::BatteryBackend;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

static SYSFS_BATTERY_PATH: OnceCell<PathBuf> = OnceCell::new();

// Reads battery info from Linux sysfs
pub struct SysfsBackend {
    path: PathBuf,
}

impl SysfsBackend {
    // Scan `/sys/class/power_supply/` for a `type == Battery` entry
    pub fn discover() -> Result<Self> {
        let path = SYSFS_BATTERY_PATH
            .get_or_try_init(|| {
                let base = PathBuf::from("/sys/class/power_supply");
                for entry in fs::read_dir(&base).context("Reading /sys/class/power_supply")? {
                    let entry = entry?;
                    let type_file = entry.path().join("type");
                    let typ = fs::read_to_string(&type_file)
                        .with_context(|| format!("Reading {}", type_file.display()))?;
                    if typ.trim() == "Battery" {
                        return Ok(entry.path());
                    }
                }
                anyhow::bail!("No battery supply found in sysfs");
            })?
            .clone();

        Ok(Self { path })
    }

    // Helper to read and parse a file under the battery path
    fn read_file<T: std::str::FromStr>(&self, name: &str) -> Result<T>
    where
        T::Err: std::fmt::Display,
    {
        let file = self.path.join(name);
        let data = fs::read_to_string(&file)
            .with_context(|| format!("Reading sysfs file {}", file.display()))?;
        let parsed = data
            .trim()
            .parse::<T>()
            .map_err(|e| anyhow::anyhow!("Parsing {} from sysfs: {}", file.display(), e))?;
        Ok(parsed)
    }

    pub fn with_path(path: PathBuf) -> Self {
        SysfsBackend { path }
    }
}

impl BatteryBackend for SysfsBackend {
    fn read(&self) -> Result<(u8, String)> {
        let cap: u8 = self.read_file("capacity")?;
        let status: String = fs::read_to_string(self.path.join("status"))
            .with_context(|| "Reading status")?
            .trim()
            .to_string();
        Ok((cap, status))
    }
}

#[cfg(test)]
mod tests {
    use super::BatteryBackend;
    use super::SysfsBackend;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discover_and_read_sysfs() {
        let td = TempDir::new().unwrap();
        let bat_dir = td.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("type"), "Battery").unwrap();
        fs::write(bat_dir.join("capacity"), "75").unwrap();
        fs::write(bat_dir.join("status"), "Charging").unwrap();

        let backend = SysfsBackend::with_path(bat_dir);
        let (cap, status) = backend.read().unwrap();

        assert_eq!(cap, 75);
        assert_eq!(status, "Charging");
    }
}
