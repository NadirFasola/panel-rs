// src/core/items/mem/item.rs

use super::stat_backend::MemInfo;
use crate::core::config::Config;
use crate::core::item::Item;
use anyhow::Result;
use glib::{ControlFlow, timeout_add_seconds_local};
use gtk4::{Box as GtkBox, Label, Orientation, Widget, prelude::*};

pub struct MemItem {
    refresh_secs: u32,
    label: Label,
}

impl MemItem {
    pub fn new(cfg: &Config) -> Result<Self> {
        let label = Label::new(None);
        Ok(MemItem {
            refresh_secs: cfg.refresh_secs as u32,
            label,
        })
    }
}

impl Item for MemItem {
    fn name(&self) -> &str {
        "mem"
    }

    fn widget(&self) -> Widget {
        let container = GtkBox::new(Orientation::Horizontal, 4);
        let label = self.label.clone();
        // initial read
        if let Ok(info) = MemInfo::read_from_proc() {
            label.set_text(&format!("{:.0}%", info.usage_percent()));
        }
        container.append(&self.label);
        container.upcast()
    }

    fn start(&self) -> Result<()> {
        let label = self.label.clone();
        let interval = self.refresh_secs;
        timeout_add_seconds_local(interval, move || {
            if let Ok(info) = MemInfo::read_from_proc() {
                label.set_text(&format!("{:.0}%", info.usage_percent()));
            }
            ControlFlow::Continue
        });
        Ok(())
    }
}
