// src/core/items/cpu/item.rs

use super::stat_backend::CpuStatBackend;
use crate::core::config::CpuConfig;
use crate::core::item::Item;
use crate::core::utils::icon;
use anyhow::Result;
use glib::{ControlFlow, SourceId, timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, Orientation, Widget};
use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

pub struct CpuItem {
    refresh_secs: u32,
    label_slot: RefCell<Option<Label>>,
    icon_slot: RefCell<Option<Image>>,
    buffer: RefCell<String>,
    backend: Rc<RefCell<CpuStatBackend>>,
    timeout_id: RefCell<Option<SourceId>>,
    icon_spec: Option<String>,
    last_icon: RefCell<Option<String>>,
}

impl CpuItem {
    pub fn new(cfg: &CpuConfig) -> Result<Self> {
        let backend = Rc::new(RefCell::new(CpuStatBackend::new()?));

        Ok(Self {
            refresh_secs: cfg
                .refresh_secs
                .expect("CpuConfig.refresh_secs must have been filled"),
            label_slot: RefCell::new(None),
            icon_slot: RefCell::new(None),
            buffer: RefCell::new(String::with_capacity(8)),
            backend,
            timeout_id: RefCell::new(None),
            icon_spec: cfg.icon.clone(),
            last_icon: RefCell::new(None),
        })
    }

    fn ensure_label(&self) -> Label {
        let mut slot = self.label_slot.borrow_mut();
        if slot.is_none() {
            let lbl = Label::new(None);
            lbl.style_context().add_class("cpu-label");
            *slot = Some(lbl);
        }
        slot.as_ref().unwrap().clone()
    }

    fn ensure_icon(&self) -> Image {
        icon::ensure_icon(
            &self.icon_slot,
            self.icon_spec.as_deref(),
            16,
            Some("cpu-icon"),
        )
    }

    /// Decide which icon to show based on CPU load.
    /// If user provides an explicit non-"auto" icon, we always use it.
    /// Otherwise, map ranges to symbolic icons (dynamic).
    fn choose_icon(&self, usage: f64) -> String {
        match self.icon_spec.as_deref() {
            Some(name) if name != "auto" => name.to_string(),
            _ => match usage as u8 {
                0..=10 => "cpu-low-symbolic",
                11..=50 => "cpu-medium-symbolic",
                51..=80 => "cpu-high-symbolic",
                81..=100 => "cpu-full-symbolic",
                _ => "cpu-medium-symbolic",
            }
            .into(),
        }
    }

    fn update_once(&self) {
        let mut buf = self.buffer.borrow_mut();
        buf.clear();

        let usage = match self.backend.borrow_mut().read() {
            Ok(u) => u,
            Err(_) => {
                buf.push_str("CPU N/A");
                self.ensure_label().set_text(&buf);
                return;
            }
        };

        write!(&mut *buf, "{usage:.0}%").ok();
        self.ensure_label().set_text(&buf);

        // Update dynamic icon if changed
        let desired = self.choose_icon(usage);
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

    fn start_timer(&self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }

        let interval = self.refresh_secs;
        let ptr = self as *const CpuItem;

        let id = timeout_add_seconds_local(interval, move || {
            let item = unsafe { &*ptr };
            item.update_once();
            ControlFlow::Continue
        });

        *self.timeout_id.borrow_mut() = Some(id);
    }
}

impl Item for CpuItem {
    fn name(&self) -> &str {
        "cpu"
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

impl Drop for CpuItem {
    fn drop(&mut self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }
    }
}
