// src/core/item.rs

use anyhow::Result;
use gtk4::prelude::*;
use gtk4::Widget;

// Core trait for a status-bar item plugin.
//
// Each item must:
// 1. provide a unique `name()` for identification;
// 2. build and return its root `Widget` via `widget()`;
// 3. start its internal logic (timers, event handlers) once mounted.
pub trait Item {
    // A short, unique identifier for the item
    fn name(&self) -> &str;

    // Construct the GTK widget(s) representing the item.
    // This is called once on `connect_activate`.
    // The returned widget will be appended to the bar's container.
    fn widget(&self) -> Widget;

    // Kick off any ongoing tasks.
    // Called after the widget is in the widget tree and show.
    fn start(&self) -> Result<()>;
}
