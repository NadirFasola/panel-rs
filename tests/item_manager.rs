// tests/item_manager.rs

use panel_rs::core::config::Config;
use panel_rs::core::item_manager::ItemManager;

#[test]
fn load_clock_item() {
    let cfg = Config {
        items: vec!["bar".into(), "clock".into()],
        refresh_secs: 1,
        ..Default::default()
    };
    let manager = ItemManager::load(&cfg);
    assert_eq!(manager.items().len(), 1);
    assert_eq!(manager.items()[0].name(), "clock");
}

#[test]
fn skip_unknown_items() {
    let cfg = Config {
        items: vec!["foo".into(), "clock".into()],
        refresh_secs: 1,
        ..Default::default()
    };
    let manager = ItemManager::load(&cfg);
    // "foo" is unknown an should be skipped
    assert_eq!(manager.items().len(), 1);
    assert_eq!(manager.items()[0].name(), "clock");
}

#[test]
fn load_temp_item() {
    let mut cfg = Config::default();
    cfg.items = vec!["temp".into()];
    cfg.refresh_secs = 1;
    let mgr = ItemManager::load(&cfg);
    assert_eq!(mgr.items().len(), 1);
    assert_eq!(mgr.items()[0].name(), "temp");
}
