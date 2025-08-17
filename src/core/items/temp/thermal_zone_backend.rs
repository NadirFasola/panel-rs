// src/core/items/temp/thermal_zone_backend.rs

use super::super::super::config::TempConfig;
use super::TemperatureBackend;

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

fn thermal_base() -> PathBuf {
    std::env::var_os("SYS_THERMAL_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/sys/class/thermal"))
}

// stash the *immutable* list of *all* discovered zones
static ZONES: OnceLock<Vec<(String, PathBuf)>> = OnceLock::new();

pub struct ThermalZoneBackend {
    zones: Vec<(String, PathBuf)>,
}

impl ThermalZoneBackend {
    pub fn new(cfg: &TempConfig) -> Result<Self> {
        // **Step 1: do all the fallible FS reads up front**, building a Vec
        //    we’ll panic if no zones are found, or return Err on any IO error.
        let base = thermal_base();
        let mut discovered = Vec::new();
        for entry in fs::read_dir(&base).context("Reading thermal directory")? {
            let dir = entry?.path();
            let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if !name.starts_with("thermal_zone") {
                continue;
            }
            let sensor = fs::read_to_string(dir.join("type"))
                .with_context(|| format!("Reading zone type from {:?}", dir))?
                .trim_end()
                .to_owned();
            let temp_path = dir.join("temp");
            if temp_path.exists() {
                discovered.push((sensor, temp_path));
            }
        }
        if discovered.is_empty() {
            anyhow::bail!("No thermal zones found under {}", base.display());
        }

        // **Step 2: stash it exactly once** into our OnceLock (cloning so we can keep our own copy)
        let all = ZONES.get_or_init(|| discovered.clone());

        // **Step 3: apply the user’s filter** on top of that cached list
        let zones = if cfg.sensors.is_empty() {
            all.clone()
        } else {
            let wanted: HashSet<_> = cfg.sensors.iter().cloned().collect();
            let filtered: Vec<_> = all
                .iter()
                .filter(|(name, _)| wanted.contains(name))
                .cloned()
                .collect();
            if filtered.is_empty() {
                anyhow::bail!("No thermal zones match {:?}", cfg.sensors);
            }
            filtered
        };

        Ok(ThermalZoneBackend { zones })
    }
}

impl TemperatureBackend for ThermalZoneBackend {
    fn read(&self) -> Result<Vec<(String, f64)>> {
        let mut readings = Vec::with_capacity(self.zones.len());
        for (name, path) in &self.zones {
            let raw = fs::read_to_string(path)
                .with_context(|| format!("Reading temperature from {:?}", path))?;
            let milli: f64 = raw
                .trim_end()
                .parse()
                .with_context(|| format!("Parsing {:?} as integer", raw.trim()))?;
            readings.push((name.clone(), milli / 1_000.0));
        }
        Ok(readings)
    }
}

#[cfg(test)]
mod tests {

    use super::super::super::super::config::TempConfig;
    use super::TemperatureBackend;
    use super::ThermalZoneBackend;
    use std::fs;
    use tempfile::TempDir;

    fn make_zone(base: &TempDir, zone: &str, sensor_type: &str, temp: i64) {
        let dir = base.path().join(zone);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("type"), sensor_type).unwrap();
        fs::write(dir.join("temp"), format!("{temp}")).unwrap();
    }

    #[test]
    fn discovery_and_read() {
        let td = TempDir::new().unwrap();
        make_zone(&td, "thermal_zone0", "x86_pkg_temp", 42000);
        make_zone(&td, "thermal_zone1", "acpitz", 30000);

        // point the code at our temp dir
        let orig = std::env::var("SYS_THERMAL_BASE").ok();
        unsafe { std::env::set_var("SYS_THERMAL_BASE", td.path()) };

        // build a default config (no explicit sensors => all zones)
        let cfg = TempConfig::default();
        let backend = ThermalZoneBackend::new(&cfg).unwrap();

        let temps = backend.read().unwrap();
        assert_eq!(temps.len(), 2);
        assert!(
            temps
                .iter()
                .any(|(n, t)| n == "x86_pkg_temp" && (*t - 42.0).abs() < 1e-6)
        );
        assert!(
            temps
                .iter()
                .any(|(n, t)| n == "acpitz" && (*t - 30.0).abs() < 1e-6)
        );

        // restore
        if let Some(v) = orig {
            unsafe { std::env::set_var("SYS_THERMAL_BASE", v) };
        }
    }
}
