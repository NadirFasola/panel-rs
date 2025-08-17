// src/core/items/temp/mod.rs

pub mod hwmon_backend;
pub mod item;
pub mod lm_sensors_backend;
pub mod temperature_backend;
pub mod thermal_zone_backend;

pub use item::TempItem;
pub use temperature_backend::{TempBackendKind, TemperatureBackend};
