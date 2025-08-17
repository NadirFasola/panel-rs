// src/core/items/clock.rs
use crate::core::config::ClockConfig;
use crate::core::item::Item;
use anyhow::Result;
use chrono::Local;
use glib::{ControlFlow, SourceId, timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Widget};
use std::cell::RefCell;
use std::fmt::Write;

pub struct ClockItem {
    refresh_secs: u32,
    format: String,
    label_slot: RefCell<Option<Label>>,
    buffer: RefCell<String>,
    timeout_id: RefCell<Option<SourceId>>,
}

impl ClockItem {
    pub fn new(cfg: &ClockConfig) -> Result<Self> {
        Ok(Self {
            refresh_secs: cfg
                .refresh_secs
                .expect("ClockConfig.refresh_secs must have been filled"),
            format: cfg.format.clone(),
            label_slot: RefCell::new(None),
            buffer: RefCell::new(String::with_capacity(16)),
            timeout_id: RefCell::new(None),
        })
    }

    fn ensure_label(&self) -> Label {
        let mut slot = self.label_slot.borrow_mut();
        if slot.is_none() {
            let lbl = Label::new(None);
            lbl.style_context().add_class("clock-label");
            *slot = Some(lbl);
        }
        slot.as_ref().unwrap().clone()
    }

    fn update_text(&self) {
        let now = Local::now();
        let mut buf = self.buffer.borrow_mut();
        buf.clear();
        // need Write in scope
        write!(&mut *buf, "{}", now.format(&self.format)).unwrap();
        self.ensure_label().set_text(&buf);
    }

    fn start_timer(&self) {
        // tear down old
        if let Some(old) = self.timeout_id.borrow_mut().take() {
            old.remove();
        }

        let interval = self.refresh_secs;
        // capture raw pointer
        let me: *const ClockItem = self;

        let id = timeout_add_seconds_local(interval, move || {
            // SAFETY: our ClockItem lives for the app’s lifetime
            let this = unsafe { &*me };
            this.update_text();
            ControlFlow::Continue
        });

        *self.timeout_id.borrow_mut() = Some(id);
    }
}

impl Item for ClockItem {
    fn name(&self) -> &str {
        "clock"
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

impl Drop for ClockItem {
    fn drop(&mut self) {
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }
    }
}
