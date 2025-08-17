// src/core/items/battery.rs
use crate::core::config::BatteryConfig;
use crate::core::item::Item;
use crate::core::items::battery::{
    BatteryBackendKind, sysfs_backend::SysfsBackend, upower_backend::UpowerBackend,
};
use anyhow::Result;
use glib::{ControlFlow, SourceId, timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Widget};
use std::cell::RefCell;
use std::fmt::Write;
use std::sync::Arc;

pub trait BatteryBackend: Send + Sync {
    fn read(&self) -> Result<(u8, String)>;
}

pub struct BatteryItem {
    refresh_secs: u32,
    label_slot: RefCell<Option<Label>>,
    buffer: RefCell<String>,
    backend: Arc<dyn BatteryBackend>,
    timeout_id: RefCell<Option<SourceId>>,
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
            buffer: RefCell::new(String::with_capacity(32)),
            backend,
            timeout_id: RefCell::new(None),
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

    fn update_text(&self) {
        let mut buf = self.buffer.borrow_mut();
        buf.clear();
        match self.backend.read() {
            Ok((cap, status)) => {
                write!(&mut *buf, "{cap}% {status}").unwrap();
            }
            Err(_) => buf.push_str("Battery N/A"),
        }
        self.ensure_label().set_text(&buf);
    }

    fn start_timer(&self) {
        if let Some(old) = self.timeout_id.borrow_mut().take() {
            old.remove();
        }

        let interval = self.refresh_secs;
        // capture raw pointer to self
        let me: *const BatteryItem = self;

        let id = timeout_add_seconds_local(interval, move || {
            // SAFETY: out BatteryItem lives for the app's lifetime
            let this = unsafe { &*me };
            this.update_text();
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
        self.update_text();
        container.append(&self.ensure_label());
        self.start_timer();
        container.upcast()
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
