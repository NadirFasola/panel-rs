// src/core/items/temp/item.rs

use super::hwmon_backend::HwmonBackend;
use super::lm_sensors_backend::LmSensorsBackend;
use super::thermal_zone_backend::ThermalZoneBackend;
use super::{TempBackendKind, TemperatureBackend};
use crate::core::config::TempConfig;
use crate::core::item::Item;
use anyhow::Result;
use glib::{ControlFlow, SourceId, source::timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Widget};
use std::cell::RefCell;
use std::fmt::Write;
use std::sync::Arc;

pub struct TempItem {
    refresh_secs: u32,

    // Lazy label
    label_slot: RefCell<Option<Label>>,

    // Reusable buffer
    buffer: RefCell<String>,

    // Chosen backend
    backend: Arc<dyn TemperatureBackend>,

    // Timer source
    timeout_id: RefCell<Option<SourceId>>,
}

impl TempItem {
    pub fn new(cfg: &TempConfig) -> Result<Self> {
        let backend: Arc<dyn TemperatureBackend> = match cfg.backend {
            TempBackendKind::ThermalZone => Arc::new(ThermalZoneBackend::new(cfg)?),
            TempBackendKind::Hwmon => Arc::new(HwmonBackend::new(cfg)?),
            TempBackendKind::LmSensors => Arc::new(LmSensorsBackend::new(cfg)?),
        };

        Ok(Self {
            refresh_secs: cfg
                .refresh_secs
                .expect("TempConfig.refresh_secs must have been filled by Config::load"),
            label_slot: RefCell::new(None),
            buffer: RefCell::new(String::with_capacity(64)),
            backend,
            timeout_id: RefCell::new(None),
        })
    }

    /// Lazily create or retrieve the Label
    fn ensure_label(&self) -> Label {
        let mut slot = self.label_slot.borrow_mut();
        if slot.is_none() {
            let lbl = Label::new(None);
            lbl.style_context().add_class("temp-label");
            *slot = Some(lbl);
        }
        slot.as_ref().unwrap().clone()
    }

    /// Read once, format all sensors into `buffer`, update label
    fn update_text(&self) {
        let mut buf = self.buffer.borrow_mut();
        buf.clear();

        match self.backend.read() {
            Ok(readings) if !readings.is_empty() => {
                // in-place formatting
                for (i, (name, temp)) in readings.into_iter().enumerate() {
                    if i > 0 {
                        buf.push(' ');
                    }
                    write!(&mut *buf, "{name}:{:.0}°C", temp).unwrap();
                }
            }
            _ => buf.push_str("Temp N/A"),
        }

        self.ensure_label().set_text(&buf);
    }

    /// Start or restart the polling timer
    fn start_timer(&self) {
        // Remove previous timer
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }

        // Capture self by raw pointer
        let ptr = self as *const TempItem;
        let interval = self.refresh_secs;

        let id = timeout_add_seconds_local(interval, move || {
            // SAFETY: TempItem lives as long as the bar
            let item = unsafe { &*ptr };
            item.update_text();
            ControlFlow::Continue
        });

        *self.timeout_id.borrow_mut() = Some(id);
    }
}

impl Item for TempItem {
    fn name(&self) -> &str {
        "temp"
    }

    fn widget(&self) -> Widget {
        let container = GtkBox::new(Orientation::Horizontal, 4);

        // Initial display
        self.update_text();
        container.append(&self.ensure_label());

        // Kick off repeating timer
        self.start_timer();

        container.upcast::<Widget>()
    }

    fn start(&self) -> Result<()> {
        // In case start() is invoked separately
        self.start_timer();
        Ok(())
    }
}

impl Drop for TempItem {
    fn drop(&mut self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }
    }
}
