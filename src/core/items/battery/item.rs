// src/core/items/battery.rs

use super::BatteryBackendKind;
use crate::core::config::Config;
use crate::core::item::Item;
use crate::core::items::battery::sysfs_backend::SysfsBackend;
use crate::core::items::battery::upower_backend::UpowerBackend;
use anyhow::Result;
use glib::{ControlFlow, timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Widget};

// Common trait for reading battery data
pub trait BatteryBackend: Send + Sync {
    // Returns (capcity %, status string)
    fn read(&self) -> Result<(u8, String)>;
}

// A panel item showing battery level & status
pub struct BatteryItem {
    refresh_secs: u32,
    label: Label,
    backend: std::sync::Arc<dyn BatteryBackend>,
}

impl BatteryItem {
    // Factory: picks a backend based on `config.battery_backend`
    pub fn new(cfg: &Config) -> Result<Self> {
        let backend: std::sync::Arc<dyn BatteryBackend> = match cfg.battery_backend {
            BatteryBackendKind::Upower => std::sync::Arc::new(UpowerBackend::new()?),
            BatteryBackendKind::Sysfs => std::sync::Arc::new(SysfsBackend::discover()?),
        };

        let label = Label::new(None);
        Ok(Self {
            refresh_secs: cfg.refresh_secs as u32,
            label,
            backend,
        })
    }
}

impl Item for BatteryItem {
    fn name(&self) -> &str {
        "battery"
    }

    fn widget(&self) -> Widget {
        let container = GtkBox::new(Orientation::Horizontal, 4);
        if let Ok((cap, status)) = self.backend.read() {
            self.label.set_text(&format!("{cap}% {status}"));
        } else {
            self.label.set_text("Battery N/A");
        }
        container.append(&self.label);
        container.upcast::<Widget>()
    }

    fn start(&self) -> Result<()> {
        let label = self.label.clone();
        let backend = self.backend.clone();
        let interval = self.refresh_secs;
        timeout_add_seconds_local(interval, move || {
            if let Ok((cap, status)) = backend.read() {
                label.set_text(&format!("{cap}% {status}"));
            }
            ControlFlow::Continue
        });
        Ok(())
    }
}
