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

    fn ensure_label(&self) -> Label {
        let mut slot = self.label_slot.borrow_mut();
        if slot.is_none() {
            let lbl = Label::new(None);
            lbl.style_context().add_class("temp-label");
            *slot = Some(lbl);
        }
        slot.as_ref().unwrap().clone()
    }

    /// Determine which icon to show based on maximum temperature
    fn choose_dynamic_icon(&self) -> String {
        let max_temp = match self.backend.read() {
            Ok(readings) if !readings.is_empty() => {
                readings.iter().map(|(_, t)| *t).fold(f64::MIN, f64::max)
            }
            _ => return "temp-medium-symbolic".into(),
        };

        match self.icon_spec.as_deref() {
            Some(name) if name != "auto" => name.to_string(),
            _ => match max_temp as u8 {
                0..=40 => "temp-low-symbolic",
                41..=70 => "temp-medium-symbolic",
                71..=100 => "temp-high-symbolic",
                _ => "temp-high-symbolic",
            }
            .into(),
        }
    }

    fn update_once(&self) {
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

        let _ = icon::ensure_icon(
            &self.icon_slot,
            self.icon_spec.as_deref(),
            Some(&|| self.choose_dynamic_icon()),
            16,
            Some("temp-icon"),
        );
    }

    fn start_timer(&self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }

        let interval = self.refresh_secs;
        let ptr = self as *const TempItem;

        let id = timeout_add_seconds_local(interval, move || {
            let item = unsafe { &*ptr };
            item.update_once();
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

        if let Some(img) = icon::ensure_icon(
            &self.icon_slot,
            self.icon_spec.as_deref(),
            Some(&|| self.choose_dynamic_icon()),
            16,
            Some("temp-icon"),
        ) {
            container.append(&img);
        }

        container.append(&self.ensure_label());

        self.update_once();
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
