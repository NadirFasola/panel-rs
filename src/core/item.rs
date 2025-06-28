// src/core/item.rs

use anyhow::Result;
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

#[cfg(test)]
mod tests {
    use super::Item;
    use anyhow::Error;
    use gtk4::prelude::Cast;

    struct DummyItem;
    impl Item for DummyItem {
        fn name(&self) -> &str { "dummy" }
        fn widget(&self) -> gtk4::Widget {
            // we deliberately don’t call Label::new() here
            // a real Item MUST provide a widget(), but for this
            // unit test we simply return a placeholder:
            gtk4::Box::new(gtk4::Orientation::Horizontal, 0).upcast()
        }
        fn start(&self) -> Result<(), Error> { Ok(()) }
    }

    #[test]
    fn dummy_item_behaves() {
        let d = DummyItem;
        // Name is logic only, no GTK needed
        assert_eq!(d.name(), "dummy");
        // widget() may return anything that upcasts to Widget
        assert!(d.start().is_ok());
    }
}
