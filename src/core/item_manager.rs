// src/core/item_manager.rs

use super::config::Config;
use super::item::Item;
use super::items::clock::ClockItem;
use tracing::warn;

// Manages the set of items for the status bar
pub struct ItemManager {
    items: Vec<Box<dyn Item>>,
}

impl ItemManager {
    // Loads all enabled items in the order specified by the config.
    pub fn load(config: &Config) -> Self {
        let mut items: Vec<Box<dyn Item>> = Vec::new();

        for name in &config.items {
            match name.as_str() {
                "clock" => {
                    // Create a ClockItem with the configured refresh rate
                    let clock = ClockItem::new(config.refresh_secs as u32);
                    items.push(Box::new(clock));
                }
                other => {
                    warn!(item = %other, "Unknown item in config, skipping");
                }
            }
        }

        ItemManager { items }
    }

    pub fn items(&self) -> &[Box<dyn Item>] {
        &self.items
    }
}
