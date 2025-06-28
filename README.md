# panel-rs

A compositor-agnostic Wayland panel bar writte in Rust & GTK.

## Overview

- __Layer shell__ based docking
- Workspace switcher, clock, battery, system tray
- Configurable via TOML

## Getting started

1. __Install dependencies__
    - GTK4 dev libraries
    - Rust & Cargo
1. __Build__
    ```bash
    cargo build --release
    ```
1. __Run__
    ```bash
    ./target/release/panel-rs
    ```

## Usage

### Configuration

By default, the system config lives in the repository under `config/deafult.toml`.

To override settings (for example, to change which items appear or tweak refresh intervals), copy this file into your user config directory `$XDG_CONFIG_HOME/panel-rs`. You can then edit `$XDG_CONFIG_HOME/panel-rs/config.toml` to your liking. When you next run `panel-rs`, it will load your user config instead of the bundled default.

## Plugin Architecture

This bar uses a **plugin** system for its items:

1. **`Item` trait**  
   Defined in `src/core/item.rs`, it requires:
   - `fn name(&self) -> &str` — a unique identifier.
   - `fn widget(&self) -> gtk4::Widget` — builds and returns the UI element.
   - `fn start(&self) -> Result<()>` — kicks off any background timers or signals.

2. **`ItemManager`**  
   In `src/core/item_manager.rs`, it:
   - Loads `Config::items: Vec<String>`.
   - Instantiates the matching `Item` implementations (e.g. `ClockItem`).
   - Exposes `items()` so the `WindowManager` can build the UI.

3. **Adding a new item**  
   To introduce a new plugin:
   - Create `src/core/items/<your_item>.rs`.
   - Implement the `Item` trait for your struct.
   - Add a `match` arm in `ItemManager::load()` mapping your item’s name to its constructor.
   - Write unit tests under the module and update README with examples.

### Example: `ClockItem`

```rust
use crate::core::item::Item;
use gtk4::Label;
use std::error::Error;

pub struct ClockItem { /* ... */ }

impl Item for ClockItem {
    fn name(&self) -> &str { "clock" }
    fn widget(&self) -> gtk4::Widget {
        let label = Label::new(None);
        label.set_text("00:00:00");
        label.upcast()
    }
    fn start(&self) -> Result<(), Box<dyn Error>> {
        // schedule updates...
        Ok(())
    }
}

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

MIT &copy; 2025 Nadir Fasola.
