// src/core/items/battery/mod.rs

//! Battery status widgets and backends

pub mod battery_backend;
pub mod item;
pub mod sysfs_backend;
pub mod upower_backend;

// Expose the `BatteryItem` and `BatteryBackendKind` type at the top level
pub use battery_backend::BatteryBackendKind;
pub use item::BatteryItem;
