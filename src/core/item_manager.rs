// src/core/item_manager.rs

use super::config::Config;
use super::item::Item;

use super::items::battery::BatteryItem;
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
                "battery" => {
                    // Create a BatteryItem
                    // BatteryItem takes &Config so it can read
                    // cfg.battery_backend
                    let battery = BatteryItem::new(config).expect("Failed to create BatteryItem");
                    items.push(Box::new(battery));
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

#[cfg(test)]
mod tests {
    use super::ItemManager;
    use crate::core::config::Config;

    #[test]
    fn load_empty_list() {
        let cfg = Config {
            items: vec![],
            refresh_secs: 1,
            ..Default::default()
        };
        let manager = ItemManager::load(&cfg);
        assert!(manager.items().is_empty());
    }

    #[test]
    fn preserves_order() {
        let cfg = Config {
            items: vec!["clock".into(), "unknown".into(), "clock".into()],
            refresh_secs: 5,
            ..Default::default()
        };
        let manager = ItemManager::load(&cfg);
        assert_eq!(manager.items().len(), 2);
        assert_eq!(manager.items()[0].name(), "clock");
        assert_eq!(manager.items()[1].name(), "clock");
    }
}
