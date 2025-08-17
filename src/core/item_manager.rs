// src/core/item_manager.rs

use super::config::Config;

use super::item::Item;

use super::items::battery::BatteryItem;
use super::items::clock::ClockItem;
use super::items::cpu::CpuItem;
use super::items::mem::MemItem;
use super::items::temp::TempItem;

use tracing::warn;

/// Try to build one item, logging a standardized warning on error.
fn make_item<F>(label: &str, f: F) -> Option<Box<dyn Item>>
where
    F: FnOnce() -> anyhow::Result<Box<dyn Item>>,
{
    match f() {
        Ok(item) => Some(item),
        Err(e) => {
            warn!("Failed to create {}: {e:#}", label);
            None
        }
    }
}

// Manages the set of items for the status bar
pub struct ItemManager {
    items: Vec<Box<dyn Item>>,
}

impl ItemManager {
    // Loads all enabled items in the order specified by the config.
    pub fn load(config: &Config) -> Self {
        let modules = &config.modules;

        let items = config
            .items
            .iter()
            .filter_map(|name| match name.as_str() {
                "clock" => make_item("clock", || {
                    ClockItem::new(&modules.clock).map(|i| Box::new(i) as _)
                }),

                "battery" => make_item("battery", || {
                    BatteryItem::new(&modules.battery).map(|i| Box::new(i) as _)
                }),

                "cpu" => make_item("cpu", || {
                    CpuItem::new(&modules.cpu).map(|i| Box::new(i) as _)
                }),

                "mem" => make_item("mem", || {
                    MemItem::new(&modules.mem).map(|i| Box::new(i) as _)
                }),

                "temp" => make_item("temp", || {
                    TempItem::new(&modules.temp).map(|i| Box::new(i) as _)
                }),

                other => {
                    warn!(item = %other, "Unknown item in config, skipping");
                    None
                }
            })
            .collect();

        ItemManager { items }
    }

    /// Borrow the loaded items
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
