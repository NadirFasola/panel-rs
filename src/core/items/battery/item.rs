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
    last_icon: RefCell<Option<String>>,
}

impl BatteryItem {
    pub fn new(cfg: &BatteryConfig) -> Result<Self> {
        let backend: Arc<dyn BatteryBackend> = match cfg.backend {
            BatteryBackendKind::Upower => Arc::new(UpowerBackend::new(cfg)?),
            BatteryBackendKind::Sysfs => Arc::new(SysfsBackend::new(cfg)?),
        };

        let item = Self {
            refresh_secs: cfg.refresh_secs.expect("BatteryConfig.refresh_secs filled"),
            label_slot: RefCell::new(None),
            icon_slot: RefCell::new(None),
            buffer: RefCell::new(String::with_capacity(32)),
            backend,
            timeout_id: RefCell::new(None),
            configured_icon: cfg.icon.clone(),
            last_icon: RefCell::new(None),
        };

        // Pre-warm a small set of typical battery icons for faster first display
        item.prewarm_icons();

        Ok(item)
    }

    fn prewarm_icons(&self) {
        let mut names = vec![
            "battery-charging-symbolic",
            "battery-empty-symbolic",
            "battery-low-symbolic",
            "battery-good-symbolic",
            "battery-full-symbolic",
        ];

        if let Some(cfg) = &self.configured_icon {
            if !cfg.is_empty() && cfg != "auto" {
                names.push(cfg.as_str());
            }
        }

        for n in names {
            let _ = icon::load_paintable(Some(n), 16);
        }
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

    fn ensure_icon(&self) -> Image {
        icon::ensure_icon(
            &self.icon_slot,
            self.configured_icon.as_deref(),
            16,
            Some("battery-icon"),
        )
    }

    fn choose_icon(&self, pct: u8, status: &str) -> String {
        match self.configured_icon.as_deref() {
            Some(name) if name != "auto" => name.to_string(),
            _ => {
                let st = status.to_lowercase();
                if st.contains("charging") {
                    return "battery-charging-symbolic".into();
                }
                match pct {
                    0..=10 => "battery-empty-symbolic",
                    11..=30 => "battery-low-symbolic",
                    31..=60 => "battery-good-symbolic",
                    61..=100 => "battery-full-symbolic",
                    _ => "battery-full-symbolic",
                }
                .into()
            }
        }
    }

    fn update_once(&self) {
        let mut buf = self.buffer.borrow_mut();
        buf.clear();

        match self.backend.read() {
            Ok((cap, status)) => {
                write!(&mut *buf, "{cap}% {status}").ok();

                let desired = self.choose_icon(cap, &status);
                let mut last = self.last_icon.borrow_mut();
                if last.as_ref().map(String::as_str) != Some(desired.as_str()) {
                    let img = self.ensure_icon();
                    icon::apply_paintable(
                        &img,
                        icon::load_paintable(Some(&desired), 16)
                            .ok()
                            .flatten()
                            .as_ref(),
                    );
                    *last = Some(desired);
                }
            }
            Err(_) => {
                buf.push_str("Battery N/A");
            }
        }

        self.ensure_label().set_text(&buf);
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
        container.append(&self.ensure_icon());
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
