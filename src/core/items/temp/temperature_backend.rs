// sr/core/items/temp/temperature_backend.rs

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum TempBackendKind {
    #[default]
    ThermalZone,
    Hwmon,
    LmSensors,
}

// A unified interface to read one or more temperature sensors
pub trait TemperatureBackend: Send + Sync {
    fn read(&self) -> Result<Vec<(String, f64)>>;
}
