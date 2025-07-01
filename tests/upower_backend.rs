// tests/upower_backend.rs
use panel_rs::core::items::battery::item::BatteryBackend;
use panel_rs::core::items::battery::upower_backend::UpowerBackend;

#[test]
// #[ignore] // only run on a real desktop with UPower
fn upower_reads_real_battery() {
    let backend = UpowerBackend::new().unwrap();
    let (cap, status) = backend.read().unwrap();
    assert!(cap <= 100);
    assert!(!status.is_empty());
}
