// tests/basic.rs

use panel_rs::core::window::WindowManager;

#[test]
fn window_manager_new_is_ok() {
    // Should not panic or return Err
    assert!(
        WindowManager::new().is_ok(),
        "WindowManager::new() failed unexpectedly"
    );
}
