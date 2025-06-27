// src/core/items/clock.rs
//
// A status-bar item displaying the current local time,
// updating every `refresh_secs` seconds.

use super::super::item::Item;
use anyhow::Result;
use chrono::Local;
use glib::ControlFlow;
use glib::source::timeout_add_seconds_local;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Widget};

// ClockItem show `HH:MM:SS` and refreshes periodically
pub struct ClockItem {
    // How often (in seconds) to update the displayed time
    refresh_secs: u32,
    // The GTK Label widget we'll update on each tick.
    label: Label,
}

impl ClockItem {
    // Create a new ClockItem with the given refresh interval.
    pub fn new(refresh_secs: u32) -> Self {
        // Initialise the Label now, text will be set in widget()/start()
        let label = Label::new(None);
        Self {
            refresh_secs,
            label,
        }
    }
}

impl Item for ClockItem {
    fn name(&self) -> &str {
        "clock"
    }

    fn widget(&self) -> Widget {
        // Build a container forthe clock (in case we add icons or padding)
        let container = GtkBox::new(Orientation::Horizontal, 4);
        // Set initial text
        let now = Local::now().format("%H:%M:%S").to_string();
        self.label.set_text(&now);
        // Pack the label into the box
        container.append(&self.label);
        // Return as a generic Widget
        container.upcast::<Widget>()
    }

    fn start(&self) -> Result<()> {
        let interval = self.refresh_secs as u32;
        let label = self.label.clone();
        // Schedule a repeating timeout on the main context
        timeout_add_seconds_local(interval, move || {
            // Update the label text on each tick
            let now_str = Local::now().format("%H:%M:%S").to_string();
            // SAFETY: we're in the GTK main thread
            label.set_text(&now_str);
            ControlFlow::Continue
        });
        Ok(())
    }
}
