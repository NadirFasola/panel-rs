// src/core/items/battery/battery_backend.rs

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BatteryBackendKind {
    Sysfs,
    Upower,
}
