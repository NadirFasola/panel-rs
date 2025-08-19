// src/core/items/temp/item.rs

use super::hwmon_backend::HwmonBackend;
use super::lm_sensors_backend::LmSensorsBackend;
use super::thermal_zone_backend::ThermalZoneBackend;
use super::{TempBackendKind, TemperatureBackend};
use crate::core::config::TempConfig;
use crate::core::item::Item;
use crate::core::utils::icon;
use anyhow::Result;
use glib::{ControlFlow, SourceId, source::timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, Orientation, Widget};
use std::cell::RefCell;
use std::fmt::Write;
use std::sync::Arc;

pub struct TempItem {
    refresh_secs: u32,
    label_slot: RefCell<Option<Label>>,
    icon_slot: RefCell<Option<Image>>,
    buffer: RefCell<String>,
    backend: Arc<dyn TemperatureBackend>,
    timeout_id: RefCell<Option<SourceId>>,
    icon_spec: Option<String>,
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
            icon_slot: RefCell::new(None),
            buffer: RefCell::new(String::with_capacity(64)),
            backend,
            timeout_id: RefCell::new(None),
            icon_spec: cfg.icon.clone(),
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

    /// Lazily create or retrieve the Image
    fn ensure_icon(&self) -> Image {
        icon::ensure_icon(
            &self.icon_slot,
            self.icon_spec.as_deref(),
            16,
            Some("temp-icon"),
        )
    }

    /// Read once, format all sensors into `buffer`, update label
    fn update_text(&self) {
        let mut buf = self.buffer.borrow_mut();
        buf.clear();

        match self.backend.read() {
            Ok(readings) if !readings.is_empty() => {
                for (i, (name, temp)) in readings.into_iter().enumerate() {
                    if i > 0 {
                        buf.push(' ');
                    }
                    write!(&mut *buf, "{name}:{temp:.0}°C").unwrap();
                }
            }
            _ => buf.push_str("Temp N/A"),
        }

        self.ensure_label().set_text(&buf);
    }

    /// Start or restart the polling timer
    fn start_timer(&self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }

        let interval = self.refresh_secs;
        let ptr = self as *const TempItem;

        let id = timeout_add_seconds_local(interval, move || {
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
        container.append(&self.ensure_icon());
        container.append(&self.ensure_label());

        self.update_text();
        self.start_timer();

        container.upcast::<Widget>()
    }

    fn start(&self) -> Result<()> {
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
