// src/core/items/battery/battery_backend.rs

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum BatteryBackendKind {
    #[default]
    Sysfs,
    Upower,
}
