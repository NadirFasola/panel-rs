// src/core/items/cpu/item.rs

use super::stat_backend::CpuStatBackend;
use crate::core::config::Config;
use crate::core::item::Item;
use anyhow::Result;
use glib::{ControlFlow, timeout_add_seconds_local};
use gtk4::{Box as GtkBox, Label, Orientation, Widget, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;

pub struct CpuItem {
    refresh_secs: u32,
    label: Label,
    backend: Rc<RefCell<CpuStatBackend>>,
}

impl CpuItem {
    pub fn new(cfg: &Config) -> Result<Self> {
        let backend = Rc::new(RefCell::new(CpuStatBackend::new()?));
        let label = Label::new(None);
        Ok(CpuItem {
            refresh_secs: cfg.refresh_secs as u32,
            label,
            backend,
        })
    }
}

impl Item for CpuItem {
    fn name(&self) -> &str {
        "cpu"
    }

    fn widget(&self) -> Widget {
        let container = GtkBox::new(Orientation::Horizontal, 4);
        // Initial read
        if let Ok(usage) = self.backend.borrow_mut().read() {
            self.label.set_text(&format!("{usage:.0}%"));
        }
        container.append(&self.label);
        container.upcast()
    }

    fn start(&self) -> Result<()> {
        let label = self.label.clone();
        let backend = Rc::clone(&self.backend);
        let interval = self.refresh_secs;
        timeout_add_seconds_local(interval, move || {
            if let Ok(usage) = backend.borrow_mut().read() {
                label.set_text(&format!("{usage:.0}%"));
            }
            ControlFlow::Continue
        });
        Ok(())
    }
}
