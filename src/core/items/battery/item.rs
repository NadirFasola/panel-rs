// src/core/items/battery.rs
use crate::core::config::BatteryConfig;
use crate::core::item::Item;
use crate::core::items::battery::{
    BatteryBackendKind, sysfs_backend::SysfsBackend, upower_backend::UpowerBackend,
};
use crate::core::utils::icon;
use anyhow::Result;
use glib::{ControlFlow, SourceId, timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, Orientation, Widget};
use std::cell::RefCell;
use std::fmt::Write;
use std::sync::Arc;

pub trait BatteryBackend: Send + Sync {
    fn read(&self) -> Result<(u8, String)>;
}

pub struct BatteryItem {
    refresh_secs: u32,
    label_slot: RefCell<Option<Label>>,
    icon_slot: RefCell<Option<Image>>,
    buffer: RefCell<String>,
    backend: Arc<dyn BatteryBackend>,
    timeout_id: RefCell<Option<SourceId>>,
    configured_icon: Option<String>,
}

impl BatteryItem {
    pub fn new(cfg: &BatteryConfig) -> Result<Self> {
        let backend: Arc<dyn BatteryBackend> = match cfg.backend {
            BatteryBackendKind::Upower => Arc::new(UpowerBackend::new(cfg)?),
            BatteryBackendKind::Sysfs => Arc::new(SysfsBackend::new(cfg)?),
        };

        Ok(Self {
            refresh_secs: cfg.refresh_secs.expect("BatteryConfig.refresh_secs filled"),
            label_slot: RefCell::new(None),
            icon_slot: RefCell::new(None),
            buffer: RefCell::new(String::with_capacity(32)),
            backend,
            timeout_id: RefCell::new(None),
            configured_icon: cfg.icon.clone(),
        })
    }

    fn ensure_label(&self) -> Label {
        let mut slot = self.label_slot.borrow_mut();
        if slot.is_none() {
            let lbl = Label::new(None);
            lbl.style_context().add_class("battery-label");
            *slot = Some(lbl);
        }
        slot.as_ref().unwrap().clone()
    }

    fn choose_dynamic_icon(&self) -> Option<String> {
        match self.backend.read() {
            Ok((cap, status)) => match self.configured_icon.as_deref() {
                Some(name) if name != "auto" => Some(name.to_string()),
                _ => {
                    let st = status.to_lowercase();
                    if st.contains("charging") {
                        Some("battery-charging-symbolic".into())
                    } else {
                        let icon_name = match cap {
                            0..=10 => "battery-empty-symbolic",
                            11..=30 => "battery-low-symbolic",
                            31..=60 => "battery-good-symbolic",
                            61..=100 => "battery-full-symbolic",
                            _ => "battery-full-symbolic",
                        };
                        Some(icon_name.into())
                    }
                }
            },
            Err(_) => None,
        }
    }

    fn update_once(&self) {
        let mut buf = self.buffer.borrow_mut();
        buf.clear();

        if let Ok((cap, status)) = self.backend.read() {
            write!(&mut *buf, "{cap}% {status}").ok();
        } else {
            buf.push_str("Battery N/A");
        }

        self.ensure_label().set_text(&buf);

        // Update icon dynamically
        let _ = icon::ensure_icon(
            &self.icon_slot,
            self.configured_icon.as_deref(),
            Some(&|| self.choose_dynamic_icon().unwrap_or_default()),
            16,
            Some("battery-icon"),
        );
    }

    fn start_timer(&self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }

        let interval = self.refresh_secs;
        let ptr = self as *const BatteryItem;

        let id = timeout_add_seconds_local(interval, move || {
            let item = unsafe { &*ptr };
            item.update_once();
            ControlFlow::Continue
        });

        *self.timeout_id.borrow_mut() = Some(id);
    }
}

impl Item for BatteryItem {
    fn name(&self) -> &str {
        "battery"
    }

    fn widget(&self) -> Widget {
        let container = GtkBox::new(Orientation::Horizontal, 4);

        if let Some(img) = icon::ensure_icon(
            &self.icon_slot,
            self.configured_icon.as_deref(),
            Some(&|| self.choose_dynamic_icon().unwrap_or_default()),
            16,
            Some("battery-icon"),
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

impl Drop for BatteryItem {
    fn drop(&mut self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }
    }
}
