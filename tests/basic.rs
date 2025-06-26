// tests/basic.rs
extern crate panel_rs;

use panel_rs::core::window::WindowManager;

#[test]
fn new_does_not_panic() {
    // Seimply ensure new() returns Ok
    assert!(WindowManager::new().is_ok());
}
