// src/core/items/cpu/item.rs

use super::stat_backend::CpuStatBackend;
use crate::core::config::CpuConfig;
use crate::core::item::Item;
use anyhow::Result;
use glib::{ControlFlow, SourceId, timeout_add_seconds_local};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Widget};
use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

pub struct CpuItem {
    refresh_secs: u32,

    // Lazy label, created on first widget() call
    label_slot: RefCell<Option<Label>>,

    // Buffer for `"NNN%"` text
    buffer: RefCell<String>,

    // Underlying stats reader
    backend: Rc<RefCell<CpuStatBackend>>,

    // Timer source
    timeout_id: RefCell<Option<SourceId>>,
}

impl CpuItem {
    pub fn new(cfg: &CpuConfig) -> Result<Self> {
        let backend = Rc::new(RefCell::new(CpuStatBackend::new()?));

        Ok(Self {
            refresh_secs: cfg
                .refresh_secs
                .expect("CpuConfig.refresh_secs must have been filled by Config::load"),
            label_slot: RefCell::new(None),
            buffer: RefCell::new(String::with_capacity(8)),
            backend,
            timeout_id: RefCell::new(None),
        })
    }

    /// Lazily create (or fetch) the Label.
    fn ensure_label(&self) -> Label {
        let mut slot = self.label_slot.borrow_mut();
        if slot.is_none() {
            let lbl = Label::new(None);
            lbl.style_context().add_class("cpu-label");
            *slot = Some(lbl);
        }
        slot.as_ref().unwrap().clone()
    }

    /// Read one sample, format into our buffer, update the label.
    fn update_text(&self) {
        let mut buf = self.buffer.borrow_mut();
        buf.clear();

        match self.backend.borrow_mut().read() {
            Ok(usage) => {
                // In-place formatting: no new heap alloc
                write!(&mut *buf, "{:.0}%", usage).unwrap();
            }
            Err(_) => {
                buf.push_str("CPU N/A");
            }
        }

        self.ensure_label().set_text(&buf);
    }

    /// Start or restart the periodic timer.
    fn start_timer(&self) {
        // Remove any existing timer
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }

        // Capture raw pointer to `self` for minimal overhead
        let ptr = self as *const CpuItem;
        let interval = self.refresh_secs;

        let id = timeout_add_seconds_local(interval, move || {
            // SAFETY: `self` lives for the app’s lifetime
            let item = unsafe { &*ptr };
            item.update_text();
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

        // Initial snapshot + display
        self.update_text();
        container.append(&self.ensure_label());

        // Kick off the timer
        self.start_timer();

        container.upcast::<Widget>()
    }

    fn start(&self) -> Result<()> {
        // In case someone calls start() directly
        self.start_timer();
        Ok(())
    }
}

impl Drop for CpuItem {
    fn drop(&mut self) {
        // Clean up timer on drop
        if let Some(id) = self.timeout_id.borrow_mut().take() {
            id.remove();
        }
    }
}
