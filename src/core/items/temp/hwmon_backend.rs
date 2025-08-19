// src/core/items/temp/hwmon_backend.rs

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use super::temperature_backend::TemperatureBackend;
use crate::core::config::TempConfig;

static HWMON_SENSORS: OnceLock<Vec<(String, PathBuf)>> = OnceLock::new();

pub struct HwmonBackend {
    sensors: Vec<(String, PathBuf)>,
}

impl HwmonBackend {
    pub fn new(cfg: &TempConfig) -> Result<Self> {
        // If we've never populated HWMON_SENSORS, do so now (erroring out if there is none).
        let base = std::env::var_os("SYS_HWMON_BASE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/sys/class/hwmon"));

        // Compute the global list of sensors, if needed:
        if HWMON_SENSORS.get().is_none() {
            let mut sensors = Vec::new();
            for entry in fs::read_dir(&base).context("Reading /sys/class/hwmon")? {
                let dir = entry?.path();
                let chip = fs::read_to_string(dir.join("name"))
                    .map(|s| s.trim().to_owned())
                    .unwrap_or_else(|_| "hwmon".into());

                for child in fs::read_dir(&dir).context("Scanning hwmon entries")? {
                    let fname = child?.file_name().to_string_lossy().into_owned();
                    if fname.starts_with("temp") && fname.ends_with("_input") {
                        let input = dir.join(&fname);
                        let label_file = input.with_file_name(fname.replace("_input", "_label"));
                        let label = fs::read_to_string(&label_file)
                            .map(|s| s.trim().to_owned())
                            .unwrap_or_else(|_| fname.clone());
                        sensors.push((format!("{chip}-{label}"), input));
                    }
                }
            }
            if sensors.is_empty() {
                anyhow::bail!(
                    "No hwmon temperature sensors found under {}",
                    base.display()
                );
            }
            // store our successful list (panics only if someone else raced us)
            HWMON_SENSORS.set(sensors.clone()).unwrap_or_else(|_| {
                // extremely unlikely: someone else set it concurrently
                panic!("Concurrent init of HWMON_SENSORS failed");
            });
        }

        // Now fetch the cached global list, clone it for our own backend
        let mut list = HWMON_SENSORS
            .get()
            .expect("HWMON_SENSORS must have been initialized")
            .clone();

        // If the user specified a subset of sensors, filter down:
        if !cfg.sensors.is_empty() {
            list.retain(|(name, _)| cfg.sensors.iter().any(|want| want == name));
            if list.is_empty() {
                anyhow::bail!("No hwmon sensors match {:?}", cfg.sensors);
            }
        }

        Ok(HwmonBackend { sensors: list })
    }
}

impl TemperatureBackend for HwmonBackend {
    fn read(&self) -> Result<Vec<(String, f64)>> {
        let mut readings = Vec::with_capacity(self.sensors.len());
        for (name, path) in &self.sensors {
            let raw =
                fs::read_to_string(path).with_context(|| format!("Reading hwmon file {path:?}"))?;
            let millideg: f64 = raw
                .trim()
                .parse()
                .with_context(|| format!("Parsing {} as integer", raw.trim()))?;
            readings.push((name.clone(), millideg / 1000.0));
        }
        Ok(readings)
    }
}

#[cfg(test)]
mod tests {
    use super::super::TempBackendKind;
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    /// Create a fake hwmon directory with two sensors.
    fn make_hwmon(base: &TempDir) {
        let dir = base.path().join("hwmon0");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("name"), "chipA").unwrap();
        fs::write(dir.join("temp1_label"), "T1").unwrap();
        fs::write(dir.join("temp1_input"), "42000").unwrap();

        let dir2 = base.path().join("hwmon1");
        fs::create_dir_all(&dir2).unwrap();
        fs::write(dir2.join("name"), "chipB").unwrap();
        fs::write(dir2.join("temp2_input"), "31000").unwrap();
    }

    #[test]
    fn discover_and_read_all() {
        // 1. Create a temp dir and populate it
        let td = TempDir::new().unwrap();
        make_hwmon(&td);

        // 2. Save any existing SYS_HWMON_BASE, then override
        let orig = env::var_os("SYS_HWMON_BASE");
        unsafe { env::set_var("SYS_HWMON_BASE", td.path()) };

        // 3. Run our code under test
        let cfg = TempConfig {
            backend: TempBackendKind::Hwmon,
            refresh_secs: Some(1),
            sensors: vec![],
            icon: None,
        };
        let backend = HwmonBackend::new(&cfg).unwrap();
        let readings = backend.read().unwrap();
        assert_eq!(readings.len(), 2);
        assert!(
            readings
                .iter()
                .any(|(n, t)| n == "chipA-T1" && (*t - 42.0).abs() < 1e-6)
        );
        assert!(
            readings
                .iter()
                .any(|(n, t)| n.starts_with("chipB") && (*t - 31.0).abs() < 1e-6)
        );

        // 4. Restore the original env var (or remove it if none)
        if let Some(val) = orig {
            unsafe { env::set_var("SYS_HWMON_BASE", val) };
        } else {
            unsafe { env::remove_var("SYS_HWMON_BASE") };
        }
    }

    #[test]
    fn filter_by_name() {
        let td = TempDir::new().unwrap();
        make_hwmon(&td);

        let orig = env::var_os("SYS_HWMON_BASE");
        unsafe { env::set_var("SYS_HWMON_BASE", td.path()) };

        let cfg = TempConfig {
            backend: TempBackendKind::Hwmon,
            refresh_secs: Some(1),
            sensors: vec!["chipA-T1".into()],
            icon: None,
        };
        let backend = HwmonBackend::new(&cfg).unwrap();
        let readings = backend.read().unwrap();
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].0, "chipA-T1");
        assert!((readings[0].1 - 42.0).abs() < 1e-6);

        if let Some(val) = orig {
            unsafe { env::set_var("SYS_HWMON_BASE", val) };
        } else {
            unsafe { env::remove_var("SYS_HWMON_BASE") };
        }
    }
}
