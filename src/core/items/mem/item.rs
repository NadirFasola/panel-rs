// src/core/items/mem/item.rs

use super::stat_backend::MemInfo;
use crate::core::config::MemConfig;
use crate::core::item::Item;
use crate::core::utils::icon;
use anyhow::Result;
use glib::{ControlFlow, SourceId, source::timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, Orientation, Widget};
use std::cell::RefCell;
use std::fmt::Write;

pub struct MemItem {
    refresh_secs: u32,
    label_slot: RefCell<Option<Label>>,
    icon_slot: RefCell<Option<Image>>,
    buffer: RefCell<String>,
    timeout_id: RefCell<Option<SourceId>>,
    icon_spec: Option<String>,
}

impl MemItem {
    pub fn new(cfg: &MemConfig) -> Result<Self> {
        Ok(Self {
            refresh_secs: cfg
                .refresh_secs
                .expect("MemConfig.refresh_secs must have been filled by Config::load"),
            label_slot: RefCell::new(None),
            icon_slot: RefCell::new(None),
            buffer: RefCell::new(String::with_capacity(8)),
            timeout_id: RefCell::new(None),
            icon_spec: cfg.icon.clone(),
        })
    }

    fn ensure_label(&self) -> Label {
        let mut slot = self.label_slot.borrow_mut();
        if slot.is_none() {
            let lbl = Label::new(None);
            lbl.style_context().add_class("mem-label");
            *slot = Some(lbl);
        }
        slot.as_ref().unwrap().clone()
    }

    /// Determine which icon to show based on memory usage.
    fn choose_dynamic_icon(&self) -> String {
        let usage_pct = match MemInfo::read_from_proc() {
            Ok(info) => info.usage_percent(),
            Err(_) => return "mem-medium-symbolic".into(),
        };

        match self.icon_spec.as_deref() {
            Some(name) if name != "auto" => name.to_string(),
            _ => match usage_pct as u8 {
                0..=30 => "mem-low-symbolic",
                31..=70 => "mem-medium-symbolic",
                71..=100 => "mem-high-symbolic",
                _ => "mem-medium-symbolic",
            }
            .into(),
        }
    }

    fn update_once(&self) {
        let mut buf = self.buffer.borrow_mut();
        buf.clear();

        let usage_pct = match MemInfo::read_from_proc() {
            Ok(info) => {
                write!(&mut *buf, "{:.0}%", info.usage_percent()).ok();
                info.usage_percent()
            }
            Err(_) => {
                buf.push_str("Mem N/A");
                self.ensure_label().set_text(&buf);
                return;
            }
        };

        self.ensure_label().set_text(&buf);

        let icon_closure = || match self.icon_spec.as_deref() {
            Some(name) if name != "auto" => name.to_string(),
            _ => match usage_pct as u8 {
                0..=30 => "mem-low-symbolic",
                31..=70 => "mem-medium-symbolic",
                71..=100 => "mem-high-symbolic",
                _ => "mem-medium-symbolic",
            }
            .into(),
        };

        let _ = icon::ensure_icon(
            &self.icon_slot,
            self.icon_spec.as_deref(),
            Some(&icon_closure),
            16,
            Some("mem-icon"),
        );
    }

    fn start_timer(&self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }

        let interval = self.refresh_secs;
        let ptr = self as *const MemItem;

        let id = timeout_add_seconds_local(interval, move || {
            let item = unsafe { &*ptr };
            item.update_once();
            ControlFlow::Continue
        });

        *self.timeout_id.borrow_mut() = Some(id);
    }
}

impl Item for MemItem {
    fn name(&self) -> &str {
        "mem"
    }

    fn widget(&self) -> Widget {
        let container = GtkBox::new(Orientation::Horizontal, 4);

        if let Some(img) = icon::ensure_icon(
            &self.icon_slot,
            self.icon_spec.as_deref(),
            Some(&|| self.choose_dynamic_icon()),
            16,
            Some("mem-icon"),
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

impl Drop for MemItem {
    fn drop(&mut self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }
    }
}
