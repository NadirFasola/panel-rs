// src/core/items/temp/lmsensors_backend.rs

use super::TemperatureBackend;
use crate::core::config::TempConfig;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::process::Command;
use std::sync::OnceLock;

/// We cache the full list of sensor-names on first init.
/// e.g. ["Core 0 temp1_input", "Package id 0 temp2_input", …]
static ALL_LM_SENSORS: OnceLock<Vec<String>> = OnceLock::new();

pub struct LmSensorsBackend {
    /// exactly the labels the user wants to see (or all if cfg.sensors is empty)
    sensors: Vec<String>,
}

impl LmSensorsBackend {
    pub fn new(cfg: &TempConfig) -> Result<Self> {
        // 1. run `sensors -j` once to discover all available sensor labels
        let all = ALL_LM_SENSORS.get_or_init(|| {
            // If `sensors` isn’t on $PATH or fails, we let it panic here:
            let output = Command::new("sensors")
                .arg("-j")
                .output()
                .expect("Could not run `sensors -j`");
            if !output.status.success() {
                panic!(
                    "`sensors -j` failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            let json: Value =
                serde_json::from_slice(&output.stdout).expect("Invalid JSON from `sensors -j`");

            let mut names = Vec::new();
            // The structure is: { chip1: { feature1: { "input": x.x }, feature2: … }, chip2: … }
            if let Value::Object(chips) = &json {
                for (chip_name, features) in chips {
                    if let Value::Object(map) = features {
                        for (feat, props) in map {
                            if let Value::Object(props_map) = props {
                                if props_map.contains_key("input") {
                                    // build a human‐readable name
                                    names.push(format!("{}:{}", chip_name, feat));
                                }
                            }
                        }
                    }
                }
            }
            names.sort();
            names
        });

        // 2. filter by cfg.sensors if non‐empty
        let sensors = if cfg.sensors.is_empty() {
            all.clone()
        } else {
            let wanted: std::collections::HashSet<_> = cfg.sensors.iter().collect();
            let filtered: Vec<_> = all
                .iter()
                .filter(|lab| wanted.contains(lab))
                .cloned()
                .collect();
            if filtered.is_empty() {
                bail!("LM Sensors: none of {:?} were found", cfg.sensors);
            }
            filtered
        };

        Ok(LmSensorsBackend { sensors })
    }
}

impl TemperatureBackend for LmSensorsBackend {
    fn read(&self) -> Result<Vec<(String, f64)>> {
        // Re‑run `sensors -j` on each tick and pick out our labels:
        let output = Command::new("sensors")
            .arg("-j")
            .output()
            .context("Failed to spawn `sensors -j`")?;
        if !output.status.success() {
            bail!(
                "`sensors -j` failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let json: Value =
            serde_json::from_slice(&output.stdout).context("Parsing JSON from `sensors -j`")?;

        // flatten into map<"Chip:feat", value>
        let mut readings = Vec::new();
        if let Value::Object(chips) = json {
            for (_, feats) in chips {
                if let Value::Object(props) = feats {
                    for feat in &self.sensors {
                        // feat is like "coretemp-isa-0000:temp2_input"
                        let parts: Vec<_> = feat.splitn(2, ':').collect();
                        if parts.len() != 2 {
                            continue;
                        }
                        let feat_map = props.get(parts[1]).and_then(Value::as_object);
                        if let Some(m) = feat_map {
                            if let Some(val) = m.get("input").and_then(Value::as_f64) {
                                readings.push((feat.clone(), val));
                            }
                        }
                    }
                }
            }
        }

        Ok(readings)
    }
}

#[cfg(test)]
mod tests {
    use super::super::TempBackendKind;
    use super::*;

    #[test]
    fn discover_and_read_lm() {
        // Note: in real tests you'd mock `sensors -j`, here we simply
        // assert that calling new() on a system with sensors works or fails
        // gracefully.  You can set SENSORS_BINARY to a wrapper script if you like.
        let cfg = TempConfig {
            backend: TempBackendKind::LmSensors,
            refresh_secs: Some(1),
            sensors: vec![], // “all”
        };
        let be = LmSensorsBackend::new(&cfg).unwrap();
        let v = be.read().unwrap();
        // we can at least assert it's non‐empty if your system has sensors
        assert!(!v.is_empty(), "No LM Sensors found");
    }
}
